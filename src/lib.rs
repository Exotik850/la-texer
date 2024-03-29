#![cfg_attr(not(test), no_std)]
extern crate alloc;
use alloc::vec::Vec;

mod lexer;
mod models;
mod parser;
mod token;

pub use lexer::Lexer;
pub use models::{Accent, ColumnAlign, DisplayStyle, IntoTexNodes, LineThickness, Node, Variant};
pub use parser::Parser;
pub use token::Token;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub enum TexNode<'a> {
    Text(&'a str),
    Inline(Node<'a>),
    Block(Node<'a>),
}

// TODO Make this a trait that returns an iterator

pub fn replace_latex<'a>(input: &'a str) -> Vec<TexNode<'a>> {
    let mut out = Vec::new();
    let mut last = 0;
    let mut chars = input.char_indices().peekable();
    while let Some((mut i, c)) = chars.next() {
        if c != '$' {
            continue;
        }
        if last < i {
            out.push(TexNode::Text(&input[last..i]));
        }
        let block = if matches!(chars.peek(), Some((_, '$'))) {
            chars.next();
            i += 1;
            true
        } else {
            false
        };
        let Some((end, _)) = chars.find(|(_, c)| *c == '$') else {
            out.push(TexNode::Text(&input[i..]));
            break;
        };
        if block && matches!(chars.peek(), Some((_, '$'))) {
            chars.next();
        } else if block {
            out.push(TexNode::Text(&input[i..]));
            break;
        }
        let mut nodes = Parser::new(&input[i + 1..end]).parse();
        let nodes = if nodes.len() == 1 {
            nodes.pop().unwrap()
        } else {
            Node::Row(nodes)
        };
        out.push(if block {
            TexNode::Block(nodes)
        } else {
            TexNode::Inline(nodes)
        });
        last = end + if block { 2 } else { 1 };
    }
    if last < input.len() {
        out.push(TexNode::Text(&input[last..]));
    }

    out
}
