# la-texer

la-texer is a Rust library for parsing LaTeX math expressions into an abstract syntax tree (AST). It provides a lexer and parser to tokenize LaTeX input and build a structured representation of the mathematical expression.

## Features

- Tokenizes LaTeX math expressions using a custom lexer
- Parses tokenized input into an abstract syntax tree
- Supports a wide range of LaTeX math commands and symbols
- Provides an `IntoTexNodes` trait for converting strings into `Node` structs
- Includes utility functions for replacing inline and block LaTeX math expressions within text
- Offers extensive unit tests for the lexer and parser

## Installation

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
la-texer = "0.1.0"
```

## Usage

Here's a simple example that demonstrates parsing a LaTeX fraction:

```rust
use la_texer::{IntoTexNodes, LineThickness, Node, Variant};

fn main() {
    let input = r#"\frac{x + 1}{y - 2}"#;
    let ast = input.into_nodes();
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
        )]
    );
}
```

This parses the LaTeX input `\frac{x + 1}{y - 2}` into an AST representation.

The project also provides functions for extracting and replacing LaTeX math expressions within text:

```rust
use la_texer::{replace_latex, TexNode};

let input = r#"This is a text $\alpha 90$ some more text $\frac{a+b}{2}$"#;
let output = replace_latex(input);
assert_eq!(
    output,
    vec![
        TexNode::Text("This is a text "),
        TexNode::Inline(Node::Row(vec![
            Node::Letter("α", Variant::Italic), 
            Node::Number("90"),
        ])),
        TexNode::Text(" some more text "),
        TexNode::Inline(Node::Frac(
            Box::new(Node::Row(vec![
                Node::Letter("a", Variant::Italic),
                Node::Operator("+"),
                Node::Letter("b", Variant::Italic),
            ])),
            Box::new(Node::Number("2")),
            LineThickness::Medium,
        )),
    ]
);
```

## Contributing

Contributions are welcome! If you find any issues or have suggestions for improvements, please open an issue or submit a pull request on the GitHub repository.

## License

This project is licensed under the [MIT License](LICENSE).
