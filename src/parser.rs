use crate::{
    lexer::Lexer,
    models::{Accent, ColumnAlign, LineThickness, Node, Variant},
    token::Token,
};

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    cur: Token<'a>,
    peek: Token<'a>,
}

impl<'a> Iterator for Parser<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.next_node();
        if let Node::Undefined(Token::EOF) = node {
            None
        } else {
            Some(node)
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);
        Self {
            cur: lexer.next_token(),
            peek: lexer.next_token(),
            lexer,
        }
    }

    pub fn parse(mut self) -> Vec<Node<'a>> {
        let mut nodes = Vec::new();
        while let Some(node) = self.next() {
            nodes.push(node);
            self.next_token();
        }
        nodes
    }

    fn next_token(&mut self) {
        self.cur = self.peek;
        self.peek = if self.cur.acts_on_a_digit() && self.lexer.cur.is_ascii_digit() {
            let c = self.lexer.read_char();
            Token::Number(&self.lexer.input[self.lexer.index - c.len_utf8()..self.lexer.index])
        } else {
            self.lexer.next_token()
        };
    }

    fn parse_group(&mut self, end: Token) -> Option<Node<'a>> {
        self.next_token();
        let mut nodes = Vec::new();
        while self.cur != end {
            if self.cur == Token::EOF {
                return None;
            }
            nodes.push(self.next_node());
            self.next_token();
        }
        if nodes.is_empty() {
            None
        } else if nodes.len() == 1 {
            nodes.pop()
        } else {
            Some(Node::Row(nodes))
        }
    }

    fn parse_text(&mut self) -> &'a str {
        self.next_token();
        let mut offset = 0;
        while let Token::Letter(x, _) = self.cur {
            offset += x.len();
            self.next_token();
        }
        &self.lexer.input[self.lexer.index - offset..self.lexer.index]
    }

    fn next_node(&mut self) -> Node<'a> {
        let left = self.single_node();
        match self.peek {
            Token::Underscore => {
                self.next_token();
                self.next_token();
                let right = self.next_node();
                Node::Subscript(Box::new(left), Box::new(right))
            }
            Token::Circumflex => {
                self.next_token();
                self.next_token();
                let right = self.next_node();
                Node::Superscript(Box::new(left), Box::new(right))
            }
            _ => left,
        }
    }

    fn single_node(&mut self) -> Node<'a> {
        let node = match self.cur {
            Token::Number(number) => Node::Number(number),
            Token::Letter(x, v) => Node::Letter(x, v),
            Token::Operator(op) => Node::Operator(op),
            Token::Function(fun) => Node::Function(fun, None),
            Token::Space(space) => Node::Space(space),
            Token::Sqrt => {
                self.next_token();
                let degree = if self.cur == Token::Paren('[') {
                    let degree = self.parse_group(Token::Paren(']'));
                    self.next_token();
                    degree
                } else {
                    None
                };
                let content = self.next_node();
                Node::Sqrt(degree.map(Box::new), Box::new(content))
            }
            Token::Frac => {
                self.next_token();
                let numerator = self.next_node();
                self.next_token();
                let denominator = self.next_node();
                Node::Frac(
                    Box::new(numerator),
                    Box::new(denominator),
                    LineThickness::Medium,
                )
            }
            Token::Binom(display) => {
                self.next_token();
                let numerator = self.next_node();
                self.next_token();
                let denominator = self.next_node();
                let binom = Node::Fenced {
                    open: "(",
                    close: ")",
                    content: Box::new(Node::Frac(
                        Box::new(numerator),
                        Box::new(denominator),
                        LineThickness::Length(0),
                    )),
                };
                match display {
                    Some(display) => Node::Style(Some(display), Box::new(binom)),
                    None => binom,
                }
            }
            Token::Over(op, acc) => {
                self.next_token();
                let target = self.next_node();
                Node::OverOp(op, acc, Box::new(target))
            }
            Token::Under(op, acc) => {
                self.next_token();
                let target = self.next_node();
                Node::UnderOp(op, acc, Box::new(target))
            }
            Token::Overset => {
                self.next_token();
                let over = self.next_node();
                self.next_token();
                let target = self.next_node();
                Node::Overset {
                    over: Box::new(over),
                    target: Box::new(target),
                }
            }
            Token::Underset => {
                self.next_token();
                let under = self.next_node();
                self.next_token();
                let target = self.next_node();
                Node::Underset {
                    under: Box::new(under),
                    target: Box::new(target),
                }
            }
            Token::Overbrace(x) => {
                self.next_token();
                let target = self.single_node();
                if self.peek == Token::Circumflex {
                    self.next_token();
                    self.next_token();
                    let expl = self.single_node();
                    let over = Node::Overset {
                        over: Box::new(expl),
                        target: Box::new(Node::Operator(x)),
                    };
                    Node::Overset {
                        over: Box::new(over),
                        target: Box::new(target),
                    }
                } else {
                    Node::Overset {
                        over: Box::new(Node::Operator(x)),
                        target: Box::new(target),
                    }
                }
            }
            Token::Underbrace(x) => {
                self.next_token();
                let target = self.single_node();
                if self.peek == Token::Underscore {
                    self.next_token();
                    self.next_token();
                    let expl = self.single_node();
                    let under = Node::Underset {
                        under: Box::new(expl),
                        target: Box::new(Node::Operator(x)),
                    };
                    Node::Underset {
                        under: Box::new(under),
                        target: Box::new(target),
                    }
                } else {
                    Node::Underset {
                        under: Box::new(Node::Operator(x)),
                        target: Box::new(target),
                    }
                }
            }
            Token::BigOp(op) => match self.peek {
                Token::Underscore => {
                    self.next_token();
                    self.next_token();
                    let under = self.single_node();
                    if self.peek == Token::Circumflex {
                        self.next_token();
                        self.next_token();
                        let over = self.single_node();
                        Node::UnderOver {
                            target: Box::new(Node::Operator(op)),
                            under: Box::new(under),
                            over: Box::new(over),
                        }
                    } else {
                        Node::Under(Box::new(Node::Operator(op)), Box::new(under))
                    }
                }
                Token::Circumflex => {
                    self.next_token();
                    self.next_token();
                    let over = self.single_node();
                    if self.peek == Token::Underscore {
                        self.next_token();
                        self.next_token();
                        let under = self.single_node();
                        Node::UnderOver {
                            target: Box::new(Node::Operator(op)),
                            under: Box::new(under),
                            over: Box::new(over),
                        }
                    } else {
                        Node::OverOp(op, Accent::False, Box::new(over))
                    }
                }
                _ => Node::Operator(op),
            },
            Token::Lim(lim) => {
                let lim = Node::Function(lim, None);
                if self.peek == Token::Underscore {
                    self.next_token();
                    self.next_token();
                    let under = self.single_node();
                    Node::Under(Box::new(lim), Box::new(under))
                } else {
                    lim
                }
            }
            Token::Slashed => {
                self.next_token();
                self.next_token();
                let node = self.next_node();
                self.next_token();
                Node::Slashed(Box::new(node))
            }
            Token::Style(var) => {
                self.next_token();
                let node = self.next_node();
                set_variant(node, var)
            }
            Token::Integral(int) => match self.peek {
                Token::Underscore => {
                    self.next_token();
                    self.next_token();
                    let sub = self.single_node();
                    if self.peek == Token::Circumflex {
                        self.next_token();
                        self.next_token();
                        let sup = self.single_node();
                        Node::SubSup {
                            target: Box::new(Node::Operator(int)),
                            sub: Box::new(sub),
                            sup: Box::new(sup),
                        }
                    } else {
                        Node::Subscript(Box::new(Node::Operator(int)), Box::new(sub))
                    }
                }
                Token::Circumflex => {
                    self.next_token();
                    self.next_token();
                    let sup = self.single_node();
                    if self.peek == Token::Underscore {
                        self.next_token();
                        self.next_token();
                        let sub = self.single_node();
                        Node::SubSup {
                            target: Box::new(Node::Operator(int)),
                            sub: Box::new(sub),
                            sup: Box::new(sup),
                        }
                    } else {
                        Node::Superscript(Box::new(Node::Operator(int)), Box::new(sup))
                    }
                }
                _ => Node::Operator(int),
            },
            Token::LSeperator(open) => {
                let close = match open {
                    "(" => ")",
                    "[" => "]",
                    "{" => "}",
                    "⌈" => "⌉",
                    "⌊" => "⌋",
                    "⦗" => "⦘",
                    "⟦" => "⟧",
                    "|" => "|",
                    _ => unreachable!("Should not happen"),
                };
                let content = self.parse_group(Token::RSeperator(close));
                match content {
                    Some(inner) => Node::Fenced {
                        open,
                        close,
                        content: Box::new(inner),
                    },
                    None => Node::OtherOperator(open),
                }
            }
            Token::Paren('|') => self
                .parse_group(Token::Paren('|'))
                .unwrap_or(Node::Operator("|")),
            // Token::Left => {
            //     self.next_token();
            //     let open = match self.cur {
            //         Token::Paren(open) => open,
            //         Token::Operator('.') => '\u{0}',
            //         token => todo!("Error handling? {token:?}"),
            //     };
            //     let content = self.parse_group(Token::Right);
            //     self.next_token();
            //     let close = match self.cur {
            //         Token::Paren(close) => close,
            //         Token::Operator('.') => '\u{0}',
            //         token => todo!("Error handling? {token:?}"),
            //     };
            //     Node::Fenced {
            //         open,
            //         close,
            //         content: Box::new(content),
            //     }
            // }
            Token::Middle => {
                let stretchy = true;
                self.next_token();
                match self.single_node() {
                    Node::Operator(op) => Node::StrechedOp(stretchy, op),
                    Node::OtherOperator(op) => Node::StrechedOp(stretchy, op),
                    _ => unimplemented!(),
                }
            }
            Token::Big(size) => {
                self.next_token();
                match self.cur {
                    Token::Paren(paren) => Node::SizedParen { size, paren },
                    token => todo!("Error handling? {token:?}"),
                }
            }
            Token::Begin => {
                self.next_token();
                let environment = self.parse_text();
                let (columnalign, environment) = if environment == "align" {
                    (ColumnAlign::Left, "matrix")
                } else {
                    (ColumnAlign::Center, environment)
                };
                let content = self
                    .parse_group(Token::End)
                    .unwrap_or(Node::Undefined(Token::EOF)); // TODO is this correct?
                let content = Node::Matrix(Box::new(content), columnalign);
                let matrix = match environment {
                    "matrix" => content,
                    "pmatrix" => Node::Fenced {
                        open: "(",
                        close: ")",
                        content: Box::new(content),
                    },
                    "bmatrix" => Node::Fenced {
                        open: "[",
                        close: "]",
                        content: Box::new(content),
                    },
                    "vmatrix" => Node::Fenced {
                        open: "|",
                        close: "|",
                        content: Box::new(content),
                    },
                    environment => Node::Text(environment),
                };
                self.next_token();
                let _ = self.parse_text();
                matrix
            }
            Token::OperatorName => {
                self.next_token();
                let function = self.parse_text();
                Node::Function(function, None)
            }
            Token::Text => {
                self.next_token();
                let text = self.parse_text();
                Node::Text(text)
            }
            Token::Ampersand => Node::Ampersand,
            Token::NewLine => Node::NewLine,
            token => Node::Undefined(token),
        };

        match self.peek {
            Token::Operator("\'") => {
                self.next_token();
                Node::Superscript(Box::new(node), Box::new(Node::Operator("′")))
            }
            _ => node,
        }
    }
}

fn set_variant(node: Node, var: Variant) -> Node {
    match node {
        Node::Letter(x, _) => Node::Letter(x, var),
        Node::Row(vec) => Node::Row(vec.into_iter().map(|node| set_variant(node, var)).collect()),
        node => node,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // #[test]
    // fn test_parser_runs() {
    //   let input = r#"
    //   \frac{\dv}{\dv x}\int_{a(x)}^{b(x)}f(x,t)\dv t = f(x,b(x))\cdot \frac{\dv}{\dv x} b(x) - f(x, a(x))\cdot \frac{\dv}{\dv x}a(x) + \int_{a(x)}^{b(x)}\frac{\partial}{\partial x}f(x,t)\dv t
    //   "#;
    //   let mut parser = Parser::new(input);
    //   let ast = parser.parse();
    // }

    #[test]
    fn test_frac() {
        let input = r#"\frac{x + 1}{y - 2}"#;
        let ast = Parser::new(input).parse();
        assert_eq!(
            ast,
            vec![Node::Frac(
                Box::new(Node::Row(vec![
                    Node::Letter("x", Variant::Italic),
                    Node::Operator("+"),
                    Node::Number("1"),
                ])),
                Box::new(Node::Row(vec![
                    Node::Letter("y", Variant::Italic),
                    Node::Operator("-"),
                    Node::Number("2"),
                ])),
                LineThickness::Medium,
            ),]
        );
    }
}
