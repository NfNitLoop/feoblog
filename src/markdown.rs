pub(crate) trait ToHTML {
    /// Convert this markdown to a safe subset of HTML.
    fn md_to_html(&self) -> String;
}

impl ToHTML for str {
    fn md_to_html(&self) -> String {
        let parser = pulldown_cmark::Parser::new(self);
        use pulldown_cmark::Event::*;
        let parser = parser.map(|event| match event {
            Html(value) => Code(value),
            InlineHtml(value) => Text(value),
            x => x,
        });

        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, parser);
        html
    }
}

