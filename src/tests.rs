use super::*;

fn test_parser(input: &str, expected: Vec<Node>) {
    let parser = Parser::new(input);
    let ast = parser.parse();
    assert_eq!(ast, expected);
}

#[test]
fn test_parser_runs() {
    let input = r#"
  \frac{\dv}{\dv x}\int_{a(x)}^{b(x)}f(x,t)\dv t = f(x,b(x))\cdot \frac{\dv}{\dv x} b(x) - f(x, a(x))\cdot \frac{\dv}{\dv x}a(x) + \int_{a(x)}^{b(x)}\frac{\partial}{\partial x}f(x,t)\dv t
  "#;
    let parser = Parser::new(input);
    let _ast = parser.parse();
}

#[test]
fn test_parser_frac() {
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

#[test]
fn test_parser_subsup() {
    let input = r#"\left\{\sin\left(\frac{1}{n}\right)\right\}_{n}^{\infty}"#;
    let ast = input.parse_latex();
    assert_eq!(
        ast,
        vec![Node::SubSup {
            target: Node::Fenced {
                open: Node::StrechedOp(true, "{").into(),
                close: Node::StrechedOp(true, "}").into(),
                content: Node::Row(vec![
                    Node::Function("sin", None),
                    Node::Fenced {
                        open: Node::StrechedOp(true, "(").into(),
                        close: Node::StrechedOp(true, ")").into(),
                        content: Node::Frac(
                            Node::Number("1").into(),
                            Node::Letter("n", Variant::Italic).into(),
                            LineThickness::Medium
                        )
                        .into()
                    }
                ]).into()
            }
            .into(),
            sub: Node::Letter("n", Variant::Italic).into(),
            sup: Node::Letter("∞", Variant::Normal).into(),
        }]
    );
    let input = r#"\left\{\sin\left(\frac{1}{n}\right)\right\}^{\infty}_{n}"#;
    let ast = input.parse_latex();
    assert_eq!(
        ast,
        vec![Node::SubSup {
            target: Node::Fenced {
                open: Node::StrechedOp(true, "{").into(),
                close: Node::StrechedOp(true, "}").into(),
                content: Node::Row(vec![
                    Node::Function("sin", None),
                    Node::Fenced {
                        open: Node::StrechedOp(true, "(").into(),
                        close: Node::StrechedOp(true, ")").into(),
                        content: Node::Frac(
                            Node::Number("1").into(),
                            Node::Letter("n", Variant::Italic).into(),
                            LineThickness::Medium
                        )
                        .into()
                    }
                ]).into()
            }
            .into(),
            sup: Node::Letter("∞", Variant::Normal).into(),
            sub: Node::Letter("n", Variant::Italic).into(),
        }]
    );
}

#[test]
fn test_parser_int() {
    let input = r#"\int_{a}^bf(x)dv x"#;
    let ast = Parser::new(input).parse();
    assert_eq!(
        ast,
        vec![
            Node::SubSup {
                target: Box::new(Node::Operator("∫")),
                sub: Box::new(Node::Letter("a", Variant::Italic)),
                sup: Box::new(Node::Letter("b", Variant::Italic)),
            },
            Node::Letter("f", Variant::Italic),
            Node::Fenced {
                open: Node::StrechedOp(true, "(").into(),
                close: Node::StrechedOp(true, ")").into(),
                content: Box::new(Node::Letter("x", Variant::Italic))
            },
            Node::Letter("d", Variant::Italic),
            Node::Letter("v", Variant::Italic),
            Node::Letter("x", Variant::Italic),
        ]
    );
}

#[test]
fn test_parser_text() {
    test_parser(
        "\\text{Hello World}   x",
        vec![
            Node::Text("Hello World", Variant::Normal),
            Node::Letter("x", Variant::Italic),
        ],
    );
    test_parser(
        "\\text{Hello World  }x",
        vec![
            Node::Text("Hello World  ", Variant::Normal),
            Node::Letter("x", Variant::Italic),
        ],
    );
}

#[test]
fn test_parser_group() {
    test_parser(
        "{{{a}}} fxb",
        vec![
            Node::Fenced {
                open: Node::StrechedOp(true, "{").into(),
                close: Node::StrechedOp(true, "}").into(),
                content: Node::Fenced {
                    open: Node::StrechedOp(true, "{").into(),
                    close: Node::StrechedOp(true, "}").into(),
                    content: Node::Fenced {
                        open: Node::StrechedOp(true, "{").into(),
                        close: Node::StrechedOp(true, "}").into(),
                        content: Node::Letter("a", Variant::Italic).into(),
                    }
                    .into(),
                }
                .into(),
            },
            Node::Letter("f", Variant::Italic),
            Node::Letter("x", Variant::Italic),
            Node::Letter("b", Variant::Italic),
        ],
    )
}

#[test]
fn test_parser_leftright_single() {
    test_parser(
        r"\left(\frac{a}{b}\right)",
        vec![Node::Fenced {
            open: Node::StrechedOp(true, "(").into(),
            close: Node::StrechedOp(true, ")").into(),
            content: Node::Frac(
                Node::Letter("a", Variant::Italic).into(),
                Node::Letter("b", Variant::Italic).into(),
                LineThickness::Medium,
            )
            .into(),
        }],
    );
    test_parser(
        r"\left\{ a+b\right\}",
        vec![Node::Fenced {
            open: Node::StrechedOp(true, "{").into(),
            close: Node::StrechedOp(true, "}").into(),
            content: Node::Row(vec![
                Node::Letter("a", Variant::Italic),
                Node::Operator("+"),
                Node::Letter("b", Variant::Italic),
            ])
            .into(),
        }],
    );
}

fn test_lexer(inputs: Vec<(&str, Vec<Token>)>) {
    for (problem, answer) in inputs.iter() {
        let mut lexer = Lexer::new(problem);
        for answer in answer.iter() {
            assert_eq!(&lexer.next_token(), answer);
        }
    }
}

#[test]
fn lexer_test_runs() {
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
    test_lexer(problems);
}

#[test]
fn test_lexer_text() {
    test_lexer(vec![(
        "\\text{Hello World}   x",
        vec![
            Token::Text,
            Token::LSeperator("{"),
            Token::Letter("H", Variant::Italic),
            Token::Letter("e", Variant::Italic),
            Token::Letter("l", Variant::Italic),
            Token::Letter("l", Variant::Italic),
            Token::Letter("o", Variant::Italic),
            Token::Letter("W", Variant::Italic),
            Token::Letter("o", Variant::Italic),
            Token::Letter("r", Variant::Italic),
            Token::Letter("l", Variant::Italic),
            Token::Letter("d", Variant::Italic),
            Token::RSeperator("}"),
            Token::Letter("x", Variant::Italic),
        ],
    )]);
}

#[test]
fn test_lexer_frac() {
    let input = vec![
        ("\\fracab", vec![Token::Command("fracab")]),
        (
            "\\frac{a}{b}",
            vec![
                Token::Frac,
                Token::LSeperator("{"),
                Token::Letter("a", Variant::Italic),
                Token::RSeperator("}"),
                Token::LSeperator("{"),
                Token::Letter("b", Variant::Italic),
                Token::RSeperator("}"),
            ],
        ),
        (
            "\\frac{a}{b}c",
            vec![
                Token::Frac,
                Token::LSeperator("{"),
                Token::Letter("a", Variant::Italic),
                Token::RSeperator("}"),
                Token::LSeperator("{"),
                Token::Letter("b", Variant::Italic),
                Token::RSeperator("}"),
                Token::Letter("c", Variant::Italic),
            ],
        ),
        (
            "\\frac{a}{\\frac{d}{e}}c",
            vec![
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
            ],
        ),
        (
            r#"\int_{a}^bf(x)dv x"#,
            vec![
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
            ],
        ),
    ];
    test_lexer(input);
}

#[test]
fn test_lexer_group() {
    test_lexer(vec![
        (
            "(abc(def(ddd)))",
            vec![
                Token::LSeperator("("),
                Token::Letter("a", Variant::Italic),
                Token::Letter("b", Variant::Italic),
                Token::Letter("c", Variant::Italic),
                Token::LSeperator("("),
                Token::Letter("d", Variant::Italic),
                Token::Letter("e", Variant::Italic),
                Token::Letter("f", Variant::Italic),
                Token::LSeperator("("),
                Token::Letter("d", Variant::Italic),
                Token::Letter("d", Variant::Italic),
                Token::Letter("d", Variant::Italic),
                Token::RSeperator(")"),
                Token::RSeperator(")"),
                Token::RSeperator(")"),
            ],
        ),
        (
            "[abc{def[ddd]}]",
            vec![
                Token::LSeperator("["),
                Token::Letter("a", Variant::Italic),
                Token::Letter("b", Variant::Italic),
                Token::Letter("c", Variant::Italic),
                Token::LSeperator("{"),
                Token::Letter("d", Variant::Italic),
                Token::Letter("e", Variant::Italic),
                Token::Letter("f", Variant::Italic),
                Token::LSeperator("["),
                Token::Letter("d", Variant::Italic),
                Token::Letter("d", Variant::Italic),
                Token::Letter("d", Variant::Italic),
                Token::RSeperator("]"),
                Token::RSeperator("}"),
                Token::RSeperator("]"),
            ],
        ),
    ])
}

#[test]
fn test_lexer_int() {
    test_lexer(vec![(
        "\\int^{a}_{b} f(x)",
        vec![
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
        ],
    )]);
}
