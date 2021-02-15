use std::{cell::RefCell, mem, str::from_utf8};

use crate::backend::{Signature, UserID};

use comrak::{Arena, ComrakOptions, arena_tree::Node, format_html, nodes::{Ast, AstNode, NodeLink, NodeValue}, parse_document};


pub(crate) trait ToHTML {
    /// Convert this markdown to a safe subset of HTML.
    fn md_to_html(&self) -> String;

    fn md_to_html_with(&self, options: Options) -> String;
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



fn to_string_lossy(bytes: Vec<u8>) -> String {
    let err = match String::from_utf8(bytes) {
        // This is the efficient happy path:
        Ok(s) => return s,
        Err(e) => e,
    };

    // Use a lossy copy instead:
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

