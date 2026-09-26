use html2md::{Handle, NodeData, StructuredPrinter, TagHandler, TagHandlerFactory, common};
use crate::atcoder::markdown::tex;

pub struct PreHandler;

impl TagHandler for PreHandler {
    fn handle(&mut self, _tag: &Handle, printer: &mut StructuredPrinter) {
        printer.append_str("```\n");
    }

    fn after_handle(&mut self, printer: &mut StructuredPrinter) {
        printer.append_str("```");
    }
}

pub struct PreHandlerFactory;

impl TagHandlerFactory for PreHandlerFactory {
    fn instantiate(&self) -> Box<dyn TagHandler> {
        Box::new(PreHandler {})
    }
}

pub struct VarHandler {
    replace_mathjax: bool,
}

impl TagHandler for VarHandler {
    fn handle(&mut self, tag: &html2md::Handle, printer: &mut html2md::StructuredPrinter) {
        printer.append_str("$");
        let text = extract_text(tag);
        if self.replace_mathjax {
            printer.append_str(&tex::replace_mathjax_aliases(&text));
        }else{
        printer.append_str(&text);
        }
    }

    fn after_handle(&mut self, printer: &mut html2md::StructuredPrinter) {
        printer.append_str("$");
    }

    fn skip_descendants(&self) -> bool {
        true
    }
}

pub struct VarHandlerFactory{
    pub replace_mathjax: bool,
}

impl TagHandlerFactory for VarHandlerFactory {
    fn instantiate(&self) -> Box<dyn TagHandler> {
        Box::new(VarHandler {
            replace_mathjax: self.replace_mathjax,
        })
    }
}

pub struct H3Handler;

impl TagHandler for H3Handler {
    fn handle(&mut self, _tag: &Handle, printer: &mut StructuredPrinter) {
        printer.insert_newline();
        printer.insert_newline();
        printer.append_str("### ");
    }

    fn after_handle(&mut self, printer: &mut StructuredPrinter) {
        printer.insert_newline();
        printer.insert_newline();
    }
}

pub struct H3HandlerFactory;

impl TagHandlerFactory for H3HandlerFactory {
    fn instantiate(&self) -> Box<dyn TagHandler> {
        Box::new(H3Handler {})
    }
}

#[derive(Default)]
pub struct SpanHandler {
    hidden: bool,
}

impl TagHandler for SpanHandler {
    fn handle(&mut self, tag: &Handle, _printer: &mut StructuredPrinter) {
        self.hidden = common::get_tag_attr(tag, "aria-hidden")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
    }

    fn after_handle(&mut self, _printer: &mut StructuredPrinter) {
    }

    fn skip_descendants(&self) -> bool {
        self.hidden
    }
}

pub struct SpanHandlerFactory;

impl TagHandlerFactory for SpanHandlerFactory {
    fn instantiate(&self) -> Box<dyn TagHandler> {
        Box::new(SpanHandler::default())
    }
}

fn extract_text(handle: &Handle) -> String {
    let mut text = String::new();
    if let NodeData::Text { ref contents } = handle.data {
        text.push_str(&contents.borrow());
    }
    for child in handle.children.borrow().iter() {
        text.push_str(&extract_text(child));
    }
    text
}

