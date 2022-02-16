use crate::html::node::{document, Document, Node};

pub struct Parser {
    input: String,
    lexer: Lexer
}

pub struct Lexer {
    pos: usize,
    input: String
}

impl Parser {
    fn parse(text: String) -> Node {
        let doc = document(Vec::new());
        for character in text.chars() {

        }
        doc
    }
}