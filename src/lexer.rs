use crate::{models::Variant, token::Token};
use core::str::Chars;

/// Lexer
#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    pub(crate) input: &'a str,
    chars: Chars<'a>,
    pub(crate) index: usize,
    pub cur: char,
    pub peek: char,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let token: Token<'a> = self.next_token();
        if token == Token::EOF {
            None
        } else {
            Some(token)
        }
    }
}

impl<'a> Lexer<'a> {
    /// 入力ソースコードを受け取り Lexer インスタンスを生成する.
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars();
        Lexer {
            input,
            cur: chars.next().unwrap_or('\u{0}'),
            peek: chars.next().unwrap_or('\u{0}'),
            chars,
            index: 0,
        }
    }

    /// 1 文字進む.
    pub fn read_char(&mut self) -> char {
        let c = self.cur;
        self.index += c.len_utf8();
        self.cur = self.peek;
        self.peek = self.chars.next().unwrap_or('\u{0}');
        c
    }

    #[inline]
    pub(crate) fn grab_slice(&self, offset: usize) -> &'a str {
        if offset > self.index {
            panic!("offset is greater than index");
        }

        if offset > self.input.len() {
            return self.input;
        }

        &self.input[self.index - offset..self.index]
    }

    /// 空白文字をスキップする.
    fn skip_whitespace(&mut self) {
        while {
            let cur = self.cur;
            cur == ' ' || cur == '\t' || cur == '\n' || cur == '\r'
        } {
            self.read_char();
        }
    }

    fn read_command(&mut self) -> Token<'a> {
        self.read_char(); // skip '\'
        let start = self.index;
        self.read_char();
        while self.cur.is_ascii_alphabetic() {
            self.read_char();
        }
        Token::from_command(&self.input[start..self.index])
    }

    fn read_number(&mut self) -> Token<'a> {
        let start = self.index;
        let mut has_period = false;
        loop {
            let cur = self.cur;
            if !cur.is_ascii_digit() && !(cur == '.' && !has_period) {
                break;
            }
            if cur == '.' {
                has_period = true;
            }
            self.read_char();
        }
        Token::Number(&self.input[start..self.index])
    }

    pub fn next_token(&mut self) -> Token<'a> {
        self.skip_whitespace();
        let token = match self.cur {
            '=' => Token::Operator("="),
            ';' => Token::Operator(";"),
            ',' => Token::Operator(","),
            '.' => Token::Operator("."),
            '\'' => Token::Operator("\'"),
            '(' => Token::LSeperator("("),
            ')' => Token::RSeperator(")"),
            '{' => Token::LSeperator("{"),
            '}' => Token::RSeperator("}"),
            '[' => Token::LSeperator("["),
            ']' => Token::RSeperator("]"),
            '⌈' => Token::LSeperator("⌈"),
            '⌉' => Token::RSeperator("⌉"),
            '⌊' => Token::LSeperator("⌊"),
            '⌋' => Token::RSeperator("⌋"),
            '⦗' => Token::LSeperator("⦗"),
            '⦘' => Token::RSeperator("⦘"),
            '⟦' => Token::LSeperator("⟦"),
            '⟧' => Token::RSeperator("⟧"),
            '|' => Token::Paren("|"),
            '+' => Token::Operator("+"),
            '-' => Token::Operator("-"),
            '*' => Token::Operator("*"),
            '/' => Token::Operator("/"),
            '!' => Token::Operator("!"),
            '<' => Token::Operator("<"),
            '>' => Token::Operator(">"),
            '_' => Token::Underscore,
            '^' => Token::Circumflex,
            '&' => Token::Ampersand,
            '\u{0}' => Token::EOF,
            ':' => {
                if self.peek == '=' {
                    self.read_char();
                    Token::Paren(":=")
                } else {
                    Token::Operator(":")
                }
            }
            '\\' => {
                return self.read_command();
            }
            c => {
                if c.is_ascii_digit() {
                    return self.read_number();
                } else {
                    Token::Letter(
                        &self.input[self.index..self.index + c.len_utf8()],
                        if c.is_ascii_alphabetic() {
                            Variant::Italic
                        } else {
                            Variant::Normal
                        },
                    )
                }
            }
        };
        self.read_char();
        token
    }
}
