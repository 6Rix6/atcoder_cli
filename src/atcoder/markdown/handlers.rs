use html2md::{Handle, NodeData, StructuredPrinter, TagHandler, TagHandlerFactory};

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

pub struct VarHandler;

impl TagHandler for VarHandler {
    fn handle(&mut self, tag: &html2md::Handle, printer: &mut html2md::StructuredPrinter) {
        printer.append_str("$");
        printer.append_str(&extract_text(tag));
    }

    fn after_handle(&mut self, printer: &mut html2md::StructuredPrinter) {
        printer.append_str("$");
    }

    fn skip_descendants(&self) -> bool {
        true
    }
}

pub struct VarHandlerFactory;

impl TagHandlerFactory for VarHandlerFactory {
    fn instantiate(&self) -> Box<dyn TagHandler> {
        Box::new(VarHandler {})
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

