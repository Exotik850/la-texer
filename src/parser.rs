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

    fn parse_text(&mut self) -> &'a str {
        // self.next_token(); // skip {
        let start = self.lexer.index - self.lexer.cur.len_utf8();
        while self.peek != Token::RSeperator("}") {
            self.next_token();
        }
        let end = self.lexer.index - self.lexer.cur.len_utf8();
        self.next_token();
        // self.next_token();
        &self.lexer.input[start..end]
    }

    fn next_node(&mut self) -> Node<'a> {
        let left = self.single_node();
        match self.peek {
            Token::Underscore => {
                self.next_token();
                self.next_token();
                let right = self.next_node().arg();
                Node::Subscript(Box::new(left), right)
            }
            Token::Circumflex => {
                self.next_token();
                self.next_token();
                let right = self.next_node().arg();
                Node::Superscript(Box::new(left), right)
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
                let over = self.next_node().arg();
                self.next_token();
                let target = self.next_node().arg();
                Node::Overset { over, target }
            }
            Token::Underset => {
                self.next_token();
                let under = self.next_node().arg();
                self.next_token();
                let target = self.next_node().arg();
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
                        Node::Under(Box::new(Node::Operator(op)), under)
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
                    let under = self.next_node();
                    Node::Under(Box::new(lim), Box::new(under))
                } else {
                    lim
                }
            }
            Token::Slashed => {
                self.next_token();
                self.next_token();
                let node = self.next_node().arg();
                self.next_token();
                Node::Slashed(node)
            }
            Token::Style(var) => {
                self.next_token();
                self.next_token();
                let node = self.next_node();
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
                let open = if open == "left" {
                    // self.next_token();
                    self.next_token();
                    self.single_node()
                } else {
                    Node::Operator(open)
                };
                let token = match &open {
                    Node::Operator("(") => Token::RSeperator(")"),
                    Node::Operator("[") => Token::RSeperator("]"),
                    Node::Operator("{") => Token::RSeperator("}"),
                    Node::Operator("⌈") => Token::RSeperator("⌉"),
                    Node::Operator("⌊") => Token::RSeperator("⌋"),
                    Node::Operator("⦗") => Token::RSeperator("⦘"),
                    Node::Operator("⟦") => Token::RSeperator("⟧"),
                    Node::Operator("|") => Token::RSeperator("|"),
                    _ => Token::RSeperator("right"),
                };
                let content = self.parse_group(token);
                let Some(content) = content else {
                    return open;
                };
                let open = if let Node::Operator(op) = open {
                    Node::StrechedOp(true, op)
                } else {
                    open
                };
                let close = if self.cur == Token::RSeperator("right") {
                    // self.next_token();
                    self.next_token();
                    self.single_node()
                } else {
                    Node::StrechedOp(true, token.to_str().unwrap())
                };
                Node::Fenced {
                    open: open.arg(),
                    close: close.arg(),
                    content: Box::new(content),
                }
            })(),
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
                    Token::Paren(paren) => Node::SizedParen { size, paren },
                    token => Node::Undefined(token),
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
                    .unwrap_or(Node::Undefined(Token::EOF));
                let content = Node::Matrix(Box::new(content), columnalign);
                let matrix = match environment {
                    "matrix" => content,
                    "pmatrix" => Node::Fenced {
                        open: Node::StrechedOp(true,"(").into(),
                        close: Node::StrechedOp(true,")").into(),
                        content: Box::new(content),
                    },
                    "bmatrix" => Node::Fenced {
                        open: Node::StrechedOp(true,"[").into(),
                        close: Node::StrechedOp(true,"]").into(),
                        content: Box::new(content),
                    },
                    "vmatrix" => Node::Fenced {
                        open: Node::StrechedOp(true,"|").into(),
                        close: Node::StrechedOp(true,"|").into(),
                        content: Box::new(content),
                    },
                    "Bmatrix" => Node::Fenced {
                        open: Node::StrechedOp(true,"{").into(),
                        close: Node::StrechedOp(true,"}").into(),
                        content: Box::new(content),
                    },
                    "Vmatrix" => Node::Fenced {
                        open: Node::StrechedOp(true,"║").into(),
                        close: Node::StrechedOp(true,"║").into(),
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