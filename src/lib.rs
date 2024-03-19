mod models;
use std::str::Chars;

use models::{Token, Variant};

/// Lexer
#[derive(Debug, Clone)]
pub(crate) struct Lexer<'a> {
    input: &'a str,
    chars: Chars<'a>,
    index: usize,
    pub(crate) cur: char,
    pub(crate) peek: char,
}

impl<'a> Lexer<'a> {
    /// 入力ソースコードを受け取り Lexer インスタンスを生成する.
    pub(crate) fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input,
            chars: input.chars(),
            cur: '\u{0}',
            peek: '\u{0}',
            index: 0,
        };
        lexer.read_char();
        lexer.read_char();
        lexer
    }

    /// 1 文字進む.
    pub(crate) fn read_char(&mut self) -> char {
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

    /// コマンド一つ分を読み込みトークンに変換する.
    fn read_command(&mut self) -> Token {
        // `\\` を読み飛ばす
        self.read_char();
        let c = self.read_char();
        // 1 文字は確実に読む
        let mut offset = c.len_utf8();
        while self.cur.is_ascii_alphabetic() {
            let c = self.read_char();
            offset += c.len_utf8();
        }
        Token::from_command(&self.input[self.index - offset..self.index])
    }

    /// 数字一つ分を読み込みトークンに変換する.
    fn read_number(&mut self) -> Token {
        let mut offset = 0;
        let mut has_period = false;
        loop {
            let cur = self.cur;
            if !cur.is_ascii_digit() || !(cur == '.' && !has_period) {
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

    /// 次のトークンを生成する.
    pub(crate) fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.cur {
            '=' => Token::Operator('='),
            ';' => Token::Operator(';'),
            ',' => Token::Operator(','),
            '.' => Token::Operator('.'),
            '\'' => Token::Operator('\''),
            '(' => Token::Paren("("),
            ')' => Token::Paren(")"),
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::Paren("["),
            ']' => Token::Paren("]"),
            '|' => Token::Paren("|"),
            '+' => Token::Operator('+'),
            '-' => Token::Operator('-'),
            '*' => Token::Operator('*'),
            '/' => Token::Operator('/'),
            '!' => Token::Operator('!'),
            '<' => Token::Operator('<'),
            '>' => Token::Operator('>'),
            '_' => Token::Underscore,
            '^' => Token::Circumflex,
            '&' => Token::Ampersand,
            '\u{0}' => Token::EOF,
            ':' => {
                if self.peek == '=' {
                    self.read_char();
                    Token::Paren(":=")
                } else {
                    Token::Operator(':')
                }
            }
            '\\' => {
                return self.read_command();
            }
            c => {
                if c.is_ascii_digit() {
                    return self.read_number();
                } else if c.is_ascii_alphabetic() {
                    Token::Letter(c, Variant::Italic)
                } else {
                    Token::Letter(c, Variant::Normal)
                }
            }
        };
        self.read_char();
        token
    }
}

#[cfg(test)]
mod tests {
    use super::models::{Token, Variant};
    use super::Lexer;

    #[test]
    fn lexer_test() {
        let problems = vec![
            (r"3", vec![Token::Number("3")]),
            (r"3.14", vec![Token::Number("3.14")]),
            (r"3.14.", vec![Token::Number("3.14"), Token::Operator('.')]),
            (r"x", vec![Token::Letter('x', Variant::Italic)]),
            (r"\pi", vec![Token::Letter('π', Variant::Italic)]),
            (
                r"x = 3.14",
                vec![
                    Token::Letter('x', Variant::Italic),
                    Token::Operator('='),
                    Token::Number("3.14"),
                ],
            ),
            (
                r"\alpha\beta",
                vec![
                    Token::Letter('α', Variant::Italic),
                    Token::Letter('β', Variant::Italic),
                ],
            ),
            (
                r"x+y",
                vec![
                    Token::Letter('x', Variant::Italic),
                    Token::Operator('+'),
                    Token::Letter('y', Variant::Italic),
                ],
            ),
            (r"\ 1", vec![Token::Space(1.), Token::Number("1")]),
        ];

        for (problem, answer) in problems.iter() {
            let mut lexer = Lexer::new(problem);
            for answer in answer.iter() {
                assert_eq!(&lexer.next_token(), answer);
            }
        }
    }
}
