#![cfg_attr(not(test), no_std)]
extern crate alloc;
use core::{iter::Peekable, str::CharIndices};

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

pub struct TexReplacer<'a> {
  input: &'a str,
  last: usize,
  chars: Peekable<CharIndices<'a>>,
}

impl<'a> TexReplacer<'a> {
  fn scan_to_delimiter(&mut self, is_block: bool) -> Option<(usize, bool)> {
      while let Some((i, c)) = self.chars.next() {
          if c == '$' {
              if is_block {
                  if let Some((_, next_c)) = self.chars.peek() {
                      if *next_c == '$' {
                          self.chars.next(); // Consume second '$'
                          return Some((i, true));
                      }
                  }
              } else {
                  return Some((i, false));
              }
          }
      }
      None
  }

  fn process_text_segment(&mut self, end: usize) -> Option<TexNode<'a>> {
      if self.last < end {
          let text = &self.input[self.last..end];
          if !text.is_empty() {
              return Some(TexNode::Text(text));
          }
      }
      None
  }

  fn process_math_segment(&mut self, start: usize, end: usize, is_block: bool) -> Option<TexNode<'a>> {
      let offset = if is_block { 2 } else { 1 };
      let content = &self.input[start + offset..end];
      let mut nodes = content.into_nodes();
      let node = if nodes.len() == 1 {
          nodes.pop().unwrap()
      } else {
          Node::Row(nodes)
      };
      
      Some(if is_block {
          TexNode::Block(node)
      } else {
          TexNode::Inline(node)
      })
  }
}

impl<'a> Iterator for TexReplacer<'a> {
  type Item = TexNode<'a>;

  fn next(&mut self) -> Option<Self::Item> {
      while let Some((i, c)) = self.chars.peek().copied() {
          if c != '$' {
              self.chars.next();
              continue;
          }

          // Check if we have text to emit before processing the delimiter
          if let Some(text_node) = self.process_text_segment(i) {
              self.last = i;
              return Some(text_node);
          }

          self.chars.next(); // Consume first '$'
          let is_block = if let Some((_, next_c)) = self.chars.peek() {
              if *next_c == '$' {
                  self.chars.next(); // Consume second '$'
                  true
              } else {
                  false
              }
          } else {
              return Some(TexNode::Text(&self.input[i..]));
          };

          if let Some((end, found_block)) = self.scan_to_delimiter(is_block) {
              if is_block == found_block {
                  let node = self.process_math_segment(i, end, is_block);
                  self.last = end + if is_block { 2 } else { 1 };
                  return node;
              }
          }

          // If we didn't find matching delimiters, treat as text
          return Some(TexNode::Text(&self.input[i..]));
      }

      // Handle any remaining text
      if self.last < self.input.len() {
          let text = &self.input[self.last..];
          self.last = self.input.len();
          if !text.is_empty() {
              return Some(TexNode::Text(text));
          }
      }

      None
  }
}

pub trait ParseLatex<'a> {
  fn parse_latex(&'a self) -> impl Iterator<Item = TexNode<'a>> + 'a;
}

impl<'a, T> ParseLatex<'a> for T
where
  T: AsRef<str> + 'a,
{
  fn parse_latex(&'a self) -> impl Iterator<Item = TexNode<'a>> + 'a {
      let input = self.as_ref();
      TexReplacer {
          input,
          chars: input.char_indices().peekable(),
          last: 0,
      }
  }
}

#[must_use] pub fn replace_latex(input: &str) -> Vec<TexNode<'_>> {
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
