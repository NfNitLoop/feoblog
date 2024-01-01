use std::{cell::RefCell, mem, num::NonZeroUsize, str::from_utf8};

use crate::backend::{Signature, UserID};

use comrak::{Arena, ComrakOptions, arena_tree::Node, format_html, nodes::{Ast, AstNode, NodeLink, NodeValue}, parse_document};

// TODO: Getting all these individually requires parsing multiple times. Optimize? (Seems premature.)
pub(crate) trait ToHTML {
    /// Convert this markdown to a safe subset of HTML.
    fn md_to_html(&self) -> String;

    fn md_to_html_with(&self, options: Options) -> String;

    /// Find all images embedded in the Markdown.
    fn md_get_images(&self) -> Vec<Image>;

    /// Get a text summary:
    fn md_get_summary(&self, max_len: usize) -> String;
}

impl ToHTML for str {
    fn md_to_html(&self) -> String {
        self.md_to_html_with(Options::default())
    }

    fn md_to_html_with(&self, options: Options) -> String {

        let md_options = ComrakOptions::default();

        let arena = Arena::new();
        let root = parse_document(&arena, self, &md_options);
        
        fix_relative_links(&arena, root, &options);

        let mut html = vec![];

        format_html(root, &md_options, &mut html).expect("Should be no I/O errors writing to a vec![]");
        to_string_lossy(html)
    }

    fn md_get_images(&self) -> Vec<Image> {
        let md_options = ComrakOptions::default();

        let arena = Arena::new();
        let root = parse_document(&arena, self, &md_options);

        let mut images = vec![];

        iter_nodes_mut(root, &mut |node| {
            if let NodeValue::Image(ref img) = node.data.borrow().value {
                images.push(image_from_node(node, img));
            }
        });


        return images
    }

    fn md_get_summary(&self, max_len: usize) -> String {
        let mut out = String::new();

        let md_options = ComrakOptions::default();

        let arena = Arena::new();
        let root = parse_document(&arena, self, &md_options);

        iter_nodes_mut(root, &mut |node| {
            if out.len() >= max_len { return }

            match node.data.borrow().value {
                NodeValue::Text(ref text) => {
                    if in_image(node) { return }

                    if !out.is_empty() && !out.ends_with(' ') { out.push(' '); }
                    let text = to_string_lossy(text.clone());
                    out.push_str(text.trim());
                    if is_heading(node) && !text.ends_with(":") { out.push(':') }
                },
                _ => {},
            }
        });

        if out.len() > max_len {
            safe_truncate(&mut out, max_len - 1);
            out.push('…');
        }

        out
    }
}


fn safe_truncate(value: &mut String, mut len: usize) {
    if value.len() <= len { return }

    // truncate panics if you try to truncate at a non-char-boundary. >.< 
    while !value.is_char_boundary(len) {
        len -= 1;
    }

    value.truncate(len);
}

fn is_heading(node: &Node<RefCell<Ast>>) -> bool {
    if let Some(parent) = node.parent() {
        if let NodeValue::Heading(_) = parent.data.borrow().value {
            return true;
        }
    }
    false
}

fn in_image(node: &Node<RefCell<Ast>>) -> bool {
    let parent = match node.parent() {
        Some(p) => p,
        _ => return false,
    };

    match parent.data.borrow().value {
        NodeValue::Image(_) => true,
        _ => false,
    }
}

fn image_from_node<'a>(node: &'a Node<'a, RefCell<Ast>>, img: &NodeLink) -> Image {
    
    let url = to_string_lossy(img.url.clone());

    // A node's alt text is stored as a child text node:
    let alt = node.children().next().map(|it| {
        if let NodeValue::Text(ref text) = it.data.borrow().value {
            return Some(to_string_lossy(text.clone()));
        }
        return None;
    }).flatten();

    Image{url, alt}
}

pub(crate) struct Image {
    pub url: String,
    pub alt: Option<String>,
}

fn fix_relative_links<'a>(arena: &Arena<Node<RefCell<Ast>>>, root: &'a AstNode<'a>, options: &Options) {
    let (user_id, signature) = match (options.user_id, options.signature) {
        (Some(u), Some(s)) => (u, s),
        _ => return,
    };

    let abs_root = format!("/u/{}/i/{}/", user_id.to_base58(), signature.to_base58());

    iter_nodes(root, &|node| {
        match &mut node.data.borrow_mut().value {
            &mut NodeValue::Link(ref mut node_link) => { fix_link(node_link, &abs_root); }
            &mut NodeValue::Image(ref mut node_link) => { fix_link(node_link, &abs_root); }
            _ => (),
        }
    });
}

fn fix_link(node_link: &mut NodeLink, abs_root: &String) -> () {
    let url = std::str::from_utf8(node_link.url.as_slice());
    let url = match url {
        Ok(u) => u,
        // We won't even bother otherwise:
        Err(e) => return,
    };

    if url.starts_with("/") || url.contains("//") {
        // Host-absolute url like /foo/bar
        // protocol-relative urL like //example.com/foo/bar
        // or absolute:  http://example.com/foo/bar
        return
    }

    let url = format!("{}{}", abs_root, url);
    node_link.url = url.into();
}



fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
where F : Fn(&'a AstNode<'a>)
{
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}

fn iter_nodes_mut<'a, F>(node: &'a AstNode<'a>, f: &mut F)
where F : FnMut(&'a AstNode<'a>)
{
    f(node);
    for c in node.children() {
        iter_nodes_mut(c, f);
    }
}



fn to_string_lossy(bytes: Vec<u8>) -> String {
    let err = match String::from_utf8(bytes) {
        // This is the efficient happy path: (in-place conversion)
        Ok(s) => return s,
        Err(e) => e,
    };

    // Use a lossy copy instead: (allocates)
    let s = String::from_utf8_lossy(err.as_bytes());
    String::from(s)
}

#[derive(Default)]
pub(crate) struct Options<'a> {

    /// If both user_id and signature are specified, we can convert relative URLS to absolute.
    /// This lets them work in feeds as well as the Item page.
    pub user_id: Option<&'a UserID>,
    pub signature: Option<&'a Signature>,
}

#[test]
fn test_get_summary() {
    let plaintext = "Hello, world! This is a test.";
    assert_eq!(plaintext, plaintext.md_get_summary(1000));

    let heading = "
Heading
=======

Hello, world!
";
    assert_eq!("Heading: Hello, world!", heading.md_get_summary(1000));

    let heading2 = "
Heading:
=======

Hello, world!
";
    assert_eq!("Heading: Hello, world!", heading2.md_get_summary(1000));

    let comment = "
<!--- This is an HTML comment and should not be rendered. -->
Hello, world!
";
    assert_eq!("Hello, world!", comment.md_get_summary(1000));

    let inline_html = r#"
<div id="foo">
This is some inline HTML.  We strip this out.
</div>

Plain text.
"#;
    assert_eq!("Plain text.", inline_html.md_get_summary(1000));

    let links = r#"

If there are [links] I expect [just][1] the text.

[links]: https://www.google.com/
[1]: https://www.twitter.com/

"#;
    assert_eq!("If there are links I expect just the text.", links.md_get_summary(1000));

    let paragraphs = r#"
If there are multiple paragraphs

I expect them to get joined with a space.
"#;

    assert_eq!("If there are multiple paragraphs I expect…", paragraphs.md_get_summary(42));

    let nihongo = "日本語はかわいいです。";
    // 日 is 4 bytes in utf-8. The first part of 本 will get dropped to be safe:
    assert_eq!("日…", nihongo.md_get_summary(5));

    let image = r#"
Here's an image: ![image]

[image]: files/image.png
"#;

    assert_eq!("Here's an image:", image.md_get_summary(1000));
}