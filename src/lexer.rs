use crate::{models::Variant, token::Token};
use std::str::Chars;

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
        self.cur = self.peek;
        self.peek = self.chars.next().unwrap_or('\u{0}');
        self.index += c.len_utf8();
        c
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
        let c = self.read_char();
        let mut offset = c.len_utf8();
        while self.cur.is_ascii_alphabetic() {
            let c = self.read_char();
            offset += c.len_utf8();
        }
        Token::from_command(&self.input[self.index - offset..self.index])
    }

    fn read_number(&mut self) -> Token<'a> {
        let mut offset = 0;
        let mut has_period = false;
        loop {
            let cur = self.cur;
            if !cur.is_ascii_digit() && !(cur == '.' && !has_period) {
                break;
            }
            if cur == '.' {
                has_period = true;
            }
            let c = self.read_char();
            offset += c.len_utf8();
        }
        Token::Number(&self.input[self.index - offset..self.index])
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
                    todo!()
                    // Token::Paren(":=")
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
                        &self.input[self.index..self.index + self.cur.len_utf8()],
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

#[cfg(test)]
mod tests {
    use super::super::models::Variant;
    use super::super::token::Token;
    use super::Lexer;

    fn handle_test(inputs: Vec<(&str, Vec<Token>)>) {
      for (problem, answer) in inputs.iter() {
        let mut lexer = Lexer::new(problem);
        for answer in answer.iter() {
            assert_eq!(&lexer.next_token(), answer);
        }
      }
    }

    #[test]
    fn lexer_test() {
        let problems = vec![
            (r"3", vec![Token::Number("3")]),
            (r"3.14", vec![Token::Number("3.14")]),
            (r"3.14.", vec![Token::Number("3.14"), Token::Operator(".")]),
            (r"x", vec![Token::Letter("x", Variant::Italic)]),
            (r"\pi", vec![Token::Letter("π", Variant::Italic)]),
            (
                r"x = 3.14",
                vec![
                    Token::Letter("x", Variant::Italic),
                    Token::Operator("="),
                    Token::Number("3.14"),
                ],
            ),
            (
                r"\alpha\beta",
                vec![
                    Token::Letter("α", Variant::Italic),
                    Token::Letter("β", Variant::Italic),
                ],
            ),
            (
                r"x+y",
                vec![
                    Token::Letter("x", Variant::Italic),
                    Token::Operator("+"),
                    Token::Letter("y", Variant::Italic),
                ],
            ),
            (r"\ 1", vec![Token::Space(1.), Token::Number("1")]),
        ];
        handle_test(problems);
    }

    #[test]
    fn test_frac() {
      let input = vec!(
        ("\\fracab", vec!(
          Token::Command("fracab")
        )),
        ("\\frac{a}{b}", vec!(
          Token::Frac,
          Token::LSeperator("{"),
          Token::Letter("a", Variant::Italic),
          Token::RSeperator("}"),
          Token::LSeperator("{"),
          Token::Letter("b", Variant::Italic),
          Token::RSeperator("}"),
        )),
        ("\\frac{a}{b}c", vec!(
          Token::Frac,
          Token::LSeperator("{"),
          Token::Letter("a", Variant::Italic),
          Token::RSeperator("}"),
          Token::LSeperator("{"),
          Token::Letter("b", Variant::Italic),
          Token::RSeperator("}"),
          Token::Letter("c", Variant::Italic),
        )),
        ("\\frac{a}{\\frac{d}{e}}c", vec!(
          Token::Frac,
          Token::LSeperator("{"),
          Token::Letter("a", Variant::Italic),
          Token::RSeperator("}"),
          Token::LSeperator("{"),
          Token::Frac,
          Token::LSeperator("{"),
          Token::Letter("d", Variant::Italic),
          Token::RSeperator("}"),
          Token::LSeperator("{"),
          Token::Letter("e", Variant::Italic),
          Token::RSeperator("}"),
          Token::RSeperator("}"),
          Token::Letter("c", Variant::Italic),
        )),
        (r#"\int_{a}^bf(x)dv x"#, vec![
          Token::Integral("∫"),
          Token::Underscore,
          Token::LSeperator("{"),
          Token::Letter("a", Variant::Italic),
          Token::RSeperator("}"),
          Token::Circumflex,
          Token::Letter("b", Variant::Italic),
          Token::Letter("f", Variant::Italic),
          Token::LSeperator("("),
          Token::Letter("x", Variant::Italic),
          Token::RSeperator(")"),
          Token::Letter("d", Variant::Italic),
          Token::Letter("v", Variant::Italic),
          Token::Letter("x", Variant::Italic),
        
        ])
      );
      handle_test(input);
    }

    #[test]
    fn test_int() {
      handle_test(vec![
        ("\\int^{a}_{b} f(x)", vec![
          Token::Integral("∫"),
          Token::Circumflex,
          Token::LSeperator("{"),
          Token::Letter("a", Variant::Italic),
          Token::RSeperator("}"),
          Token::Underscore,
          Token::LSeperator("{"),
          Token::Letter("b", Variant::Italic),
          Token::RSeperator("}"),
          Token::Letter("f", Variant::Italic),
          Token::LSeperator("("),
          Token::Letter("x", Variant::Italic),
          Token::RSeperator(")"),
        ]),
        // ("", vec![

        // ])
      ]);
    }
}
