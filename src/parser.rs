use crate::{
    lexer::Lexer,
    models::{Accent, ColumnAlign, LineThickness, Node, Variant},
    token::Token,
};

use alloc::boxed::Box;
use alloc::vec::Vec;

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
        self.next_token();
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
        while self.cur != Token::EOF {
            nodes.push(self.next_node());
            self.next_token();
        }
        nodes
    }

    fn next_token(&mut self) {
        self.cur = self.peek;
        self.peek = if self.cur.acts_on_a_digit() && self.lexer.cur.is_ascii_digit() {
            let c = self.lexer.read_char();
            Token::Number(self.lexer.grab_slice(c.len_utf8()))
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

    fn parse_text(&mut self, end: Token) -> &'a str {
        // self.next_token(); // skip {
        let start = self.lexer.index - self.lexer.cur.len_utf8();
        while self.peek != end {
            self.next_token();
        }
        let end = self.lexer.index - self.lexer.cur.len_utf8();
        self.next_token();
        // self.next_token();
        &self.lexer.input[start..end]
    }

    /// Collects the next node from the input, checking for subscripts and superscripts thereafter
    fn next_node(&mut self) -> Node<'a> {
        let left = self.single_node();
        match self.peek {
            Token::Underscore => {
                self.next_token();
                self.next_token();
                let right = self.single_node().arg();
                match self.peek {
                    Token::Circumflex => {
                        self.next_token();
                        self.next_token();
                        let upper = self.single_node().arg();
                        Node::SubSup {
                            target: left.into(),
                            sub: right,
                            sup: upper,
                        }
                    }
                    _ => Node::Subscript(Box::new(left), right),
                }
            }
            Token::Circumflex => {
                self.next_token();
                self.next_token();
                let right = self.single_node().arg();
                match self.peek {
                    Token::Underscore => {
                        self.next_token();
                        self.next_token();
                        let lower = self.single_node().arg();
                        Node::SubSup {
                            target: left.into(),
                            sub: lower,
                            sup: right,
                        }
                    }
                    _ => Node::Superscript(Box::new(left), right),
                }
            }
            _ => left,
        }
    }

    /// Collects a single node from the input, does not check for any superscript or subscript thereafter
    fn single_node(&mut self) -> Node<'a> {
        let node = match self.cur {
            Token::Number(number) => Node::Number(number),
            Token::Letter(x, v) => Node::Letter(x, v),
            Token::Operator(op) => Node::Operator(op),
            Token::Function(fun) => Node::Function(fun, None),
            Token::Space(space) => Node::Space(space),
            Token::Sqrt => {
                self.next_token();
                let degree = (self.cur == Token::LSeperator("["))
                    .then(|| {
                        let degree = self.parse_group(Token::RSeperator("]"));
                        self.next_token();
                        degree
                    })
                    .flatten();
                let content = self.next_node();
                Node::Sqrt(degree.map(Box::new), Box::new(content))
            }
            Token::Frac => {
                self.next_token();
                let numerator = self.next_node().arg();
                self.next_token();
                let denominator = self.next_node().arg();
                Node::Frac(numerator, denominator, LineThickness::Medium)
            }
            Token::Binom(display) => {
                self.next_token();
                let numerator = self.next_node();
                self.next_token();
                let denominator = self.next_node();
                let binom = Node::Fenced {
                    open: Node::StrechedOp(true, "(").into(),
                    close: Node::StrechedOp(true, ")").into(),
                    content: Box::new(Node::Frac(
                        Box::new(numerator),
                        Box::new(denominator),
                        LineThickness::Length(0),
                    )),
                };
                match display {
                    Some(display) => Node::Style(display, Box::new(binom)),
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
                let over = self.single_node().arg();
                self.next_token();
                let target = self.single_node().arg();
                Node::Overset { over, target }
            }
            Token::Underset => {
                self.next_token();
                let under = self.single_node().arg();
                self.next_token();
                let target = self.single_node().arg();
                Node::Underset { under, target }
            }
            Token::Overbrace(x) => {
                self.next_token();
                let target = self.single_node().arg();
                if self.peek == Token::Circumflex {
                    self.next_token();
                    self.next_token();
                    let expl = self.single_node().arg();
                    let over = Node::Overset {
                        over: expl,
                        target: Box::new(Node::Operator(x)),
                    };
                    Node::Overset {
                        over: Box::new(over),
                        target,
                    }
                } else {
                    Node::Overset {
                        over: Box::new(Node::Operator(x)),
                        target,
                    }
                }
            }
            Token::Underbrace(x) => {
                self.next_token();
                let target = self.single_node().arg();
                if self.peek == Token::Underscore {
                    self.next_token();
                    self.next_token();
                    let expl = self.single_node().arg();
                    let under = Node::Underset {
                        under: expl,
                        target: Box::new(Node::Operator(x)),
                    };
                    Node::Underset {
                        under: Box::new(under),
                        target,
                    }
                } else {
                    Node::Underset {
                        under: Box::new(Node::Operator(x)),
                        target,
                    }
                }
            }
            Token::BigOp(op) => match self.peek {
                Token::Underscore => {
                    self.next_token();
                    self.next_token();
                    let under = self.single_node().arg();
                    if self.peek == Token::Circumflex {
                        self.next_token();
                        self.next_token();
                        let over = self.single_node().arg();
                        Node::UnderOver {
                            target: Box::new(Node::Operator(op)),
                            under,
                            over,
                        }
                    } else {
                        Node::Underset{ target: Box::new(Node::Operator(op)), under }
                    }
                }
                Token::Circumflex => {
                    self.next_token();
                    self.next_token();
                    let over = self.single_node().arg();
                    if self.peek == Token::Underscore {
                        self.next_token();
                        self.next_token();
                        let under = self.single_node().arg();
                        Node::UnderOver {
                            target: Box::new(Node::Operator(op)),
                            under,
                            over,
                        }
                    } else {
                        Node::OverOp(op, Accent::False, over)
                    }
                }
                _ => Node::Operator(op),
            },
            Token::Lim(lim) => {
                let lim = Node::Function(lim, None);
                if self.peek == Token::Underscore {
                    self.next_token();
                    self.next_token();
                    let under = self.single_node().arg();
                    Node::Underset{ target: Box::new(lim), under }
                } else {
                    lim
                }
            }
            Token::Slashed => {
                self.next_token();
                // self.next_token();
                let node = self.single_node().arg();
                // self.next_token();
                Node::Slashed(node)
            }
            Token::Style(var) => {
                self.next_token();
                // self.next_token();
                let node = self.single_node();
                set_variant(node, var)
            }
            Token::Integral(int) => match self.peek {
                Token::Underscore => {
                    self.next_token();
                    self.next_token();
                    let sub = self.single_node().arg();
                    if self.peek == Token::Circumflex {
                        self.next_token();
                        self.next_token();
                        let sup = self.single_node().arg();
                        Node::SubSup {
                            target: Box::new(Node::Operator(int)),
                            sub,
                            sup,
                        }
                    } else {
                        Node::Subscript(Box::new(Node::Operator(int)), sub)
                    }
                }
                Token::Circumflex => {
                    self.next_token();
                    self.next_token();
                    let sup = self.single_node().arg();
                    if self.peek == Token::Underscore {
                        self.next_token();
                        self.next_token();
                        let sub = self.single_node().arg();
                        Node::SubSup {
                            target: Box::new(Node::Operator(int)),
                            sub,
                            sup,
                        }
                    } else {
                        Node::Superscript(Box::new(Node::Operator(int)), sup)
                    }
                }
                _ => Node::Operator(int),
            },
            Token::LSeperator(open) => (|| {
                let token = match open {
                    "(" => ")",
                    "[" => "]",
                    "{" => "}",
                    "⌈" => "⌉",
                    "⌜" => "⌝",
                    "⌞" => "⌟",
                    "⌊" => "⌋",
                    "⦗" => "⦘",
                    "⟦" => "⟧",
                    "|" => "|",
                    open => unreachable!("Invalid open delimiter: {open}"),
                };
                let Some(content) = self.parse_group(Token::RSeperator(token)) else {
                    return Node::Operator(open);
                };
                let close = Node::StrechedOp(true, token).into();
                Node::Fenced {
                    open: Node::StrechedOp(true, open).into(),
                    close,
                    content: Box::new(content),
                }
            })(),
            Token::Left => {
                self.next_token();
                let mut s = None;
                let open = match self.cur {
                    Token::LSeperator(op) => op,
                    Token::Big(size) => {
                        self.next_token();
                        s = Some(size);
                        match self.cur {
                            Token::Paren(paren)
                            | Token::LSeperator(paren)
                            | Token::RSeperator(paren) => paren,
                            token => todo!("Invalid token: {token:?}"),
                        }
                    }
                    token => todo!("Invalid token: {token:?}"),
                };
                // self.next_token();
                // let Some(content) = self.parse_group(Token::Right) else {
                //   return open;
                // };
                let content = self.parse_group(Token::Right).unwrap(); // TODO
                self.next_token();
                let mut s2 = None;
                let close = match self.cur {
                    Token::RSeperator(op) | Token::Paren(op) => op,
                    Token::Big(size) => {
                        self.next_token();
                        s2 = Some(size);
                        match self.cur {
                            Token::Paren(paren)
                            | Token::LSeperator(paren)
                            | Token::RSeperator(paren) => paren,
                            token => todo!("Invalid token: {token:?}"),
                        }
                    }
                    token => todo!("Invalid token: {token:?}"),
                };

                let open = match s {
                    Some(size) => Node::SizedParen { size, paren: open },
                    None => Node::StrechedOp(true, open),
                };
                let close = match s2 {
                    Some(size) => Node::SizedParen { size, paren: close },
                    None => Node::StrechedOp(true, close),
                };

                Node::Fenced {
                    open: Box::new(open),
                    close: Box::new(close),
                    content: Box::new(content),
                }
            }
            Token::Paren("|") => self
                .parse_group(Token::Paren("|"))
                .unwrap_or(Node::Operator("|")),
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
                    Token::Paren(paren) | Token::LSeperator(paren) | Token::RSeperator(paren) => {
                        Node::SizedParen { size, paren }
                    }
                    token => Node::Undefined(token),
                }
            }
            Token::Begin => {
                self.next_token();
                let environment = self.parse_text(Token::RSeperator("}"));
                // let environment = self.single_node().arg(self);
                let (columnalign, environment) = if environment.starts_with("align") {
                    (ColumnAlign::Left, "matrix")
                } else {
                    (ColumnAlign::Center, environment)
                };

                // Do we check here if the environment is the same?
                let Some(content) = self.parse_group(Token::End) else {
                    return Node::Text(environment, Variant::Normal);
                };
                self.next_token();
                let _end_environment = self.parse_text(Token::RSeperator("}")); // TODO check if it's the same as the start
                let content = Node::Matrix(Box::new(content), columnalign);
                let matrix = match environment {
                    // TODO Add more environments, they are not all matrices
                    "matrix" => content,
                    "pmatrix" | "bmatrix" | "vmatrix" | "Bmatrix" | "Vmatrix" => {
                        let (open, close) = match environment {
                            "pmatrix" => ("(", ")"),
                            "bmatrix" => ("[", "]"),
                            "vmatrix" => ("|", "|"),
                            "Bmatrix" => ("{", "}"),
                            "Vmatrix" => ("║", "║"),
                            _ => unreachable!(),
                        };
                        Node::Fenced {
                            open: Box::new(Node::StrechedOp(true, open)),
                            close: Box::new(Node::StrechedOp(true, close)),
                            content: Box::new(content),
                        }
                    }
                    environment => Node::Text(environment, Variant::Normal),
                };

                matrix
            }
            Token::Package | Token::OperatorName | Token::Text | Token::Title => {
                let c = self.cur;
                self.next_token();
                let content = self.parse_text(Token::RSeperator("}"));
                match c {
                    Token::Package => Node::Package(content),
                    Token::OperatorName => Node::Function(content, None),
                    Token::Text => Node::Text(content, Variant::Normal),
                    Token::Title => Node::Title(content),
                    _ => unreachable!(),
                }
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

/// Recursively sets all the letters to the given Variant
fn set_variant(node: Node, var: Variant) -> Node {
    match node {
        Node::Letter(x, _) => Node::Letter(x, var),
        Node::Row(vec) => Node::Row(vec.into_iter().map(|node| set_variant(node, var)).collect()),
        node => node,
    }
}
