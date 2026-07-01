use std::collections::HashMap;

use html2md::TagHandlerFactory;

use crate::atcoder::markdown::handlers::{H3HandlerFactory, PreHandlerFactory, VarHandlerFactory};

mod handlers;

pub fn html_to_md(html: &str) -> String {
    let mut tag_factory: HashMap<String, Box<dyn TagHandlerFactory>> = HashMap::new();
    tag_factory.insert(String::from("var"), Box::new(VarHandlerFactory {}));
    tag_factory.insert(String::from("pre"), Box::new(PreHandlerFactory {}));
    tag_factory.insert(String::from("h3"), Box::new(H3HandlerFactory {}));

    html2md::parse_html_custom(html, &tag_factory)
}
