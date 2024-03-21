use std::io::Write;
use la_texer::{DisplayStyle, Node, Parser};

fn main() {
    let input = r#"\begin{Vmatrix} \text{Whataburger} & 2 & 3 \\ a & b & c \end{Vmatrix}"#;
    let parser = Parser::new(input);
    let mut output = Vec::new();
    {
        let mut bufwriter = std::io::BufWriter::new(&mut output);
        write!(bufwriter, "<math xmlns=\"http://www.w3.org/1998/Math/MathML\">").unwrap();
        let tokens = parser.parse();
        println!("{tokens:#?}");
        for token in tokens.into_iter() {
            expand_node(&token, &mut bufwriter).unwrap();
        }
        write!(bufwriter, "</math>").unwrap();
    }
    println!("{}", String::from_utf8(output).unwrap());
}

fn expand_node(node: &Node, buf: &mut dyn Write) -> std::io::Result<()> {
    match node {
        Node::Number(number) => write!(buf, "<mn>{number}</mn>"),
        Node::Letter(letter, var) => match var {
            la_texer::Variant::Italic => write!(buf, "<mi>{letter}</mi>"),
            _ => write!(buf, "<mi mathvariant=\"{var}\">{letter}</mi>"),
        },
        Node::Operator(op) => {
            if *op == "" {
                write!(buf, "<mo mathvariant=\"italic\">∂</mo>")
            } else {
                write!(buf, "<mo>{op}</mo>")
            }
        }
        Node::Function(fun, arg) => match arg {
            Some(arg) => {
                write!(buf, "<mi>{fun}</mi><mo>&#x2061;</mo>")?;
                expand_node(arg, buf)
            }
            None => write!(buf, "<mi>{fun}</mi>"),
        },
        Node::Space(space) => write!(buf, "<mspace width=\"{space}em\"/>"),
        Node::Subscript(a, b) => {
            write!(buf, "<msub>")?;
            expand_node(a, buf)?;
            expand_node(b, buf)?;
            write!(buf, "</msub>")
        }
        Node::Superscript(a, b) => {
            write!(buf, "<msup>")?;
            expand_node(a, buf)?;
            expand_node(b, buf)?;
            write!(buf, "</msup>")
        }
        Node::SubSup { target, sub, sup } => {
            write!(buf, "<msubsup>")?;
            expand_node(target, buf)?;
            expand_node(sub, buf)?;
            expand_node(sup, buf)?;
            write!(buf, "</msubsup>")
        }
        Node::OverOp(op, acc, target) => {
            write!(buf, "<mover>")?;
            expand_node(target, buf)?;
            write!(buf, "<mo accent=\"{acc}\">{op}</mo></mover>")
        }
        Node::UnderOp(op, acc, target) => {
            write!(buf, "<munder>")?;
            expand_node(target, buf)?;
            write!(buf, "<mo accent=\"{acc}\">{op}</mo></munder>")
        }
        Node::Overset { over, target } => {
            write!(buf, "<mover>")?;
            expand_node(target, buf)?;
            expand_node(over, buf)?;
            write!(buf, "</mover>")
        }
        Node::Underset { under, target } | Node::Under(target, under) => {
            write!(buf, "<munder>")?;
            expand_node(target, buf)?;
            expand_node(under, buf)?;
            write!(buf, "</munder>")
        }
        Node::UnderOver {
            target,
            under,
            over,
        } => {
            write!(buf, "<munderover>")?;
            expand_node(target, buf)?;
            expand_node(under, buf)?;
            expand_node(over, buf)?;
            write!(buf, "</munderover>")
        }
        Node::Sqrt(degree, content) => match degree {
            Some(degree) => {
                write!(buf, "<mroot>")?;
                expand_node(content, buf)?;
                expand_node(degree, buf)?;
                write!(buf, "</mroot>")
            }
            None => {
                write!(buf, "<msqrt>")?;
                expand_node(content, buf)?;
                write!(buf, "</msqrt>")
            }
        },
        Node::Frac(num, denom, lt) => {
            write!(buf, "<mfrac linethickness=\"{lt}\">")?;
            expand_node(num, buf)?;
            expand_node(denom, buf)?;
            write!(buf, "</mfrac>")
        }
        Node::Row(nodes) => {
            write!(buf, "<mrow>")?;
            for node in nodes {
                expand_node(node, buf)?;
            }
            write!(buf, "</mrow>")
        }
        Node::Fenced {
            open,
            close,
            content,
        } => {
            write!(
                buf,
                "<mrow>\
                "
                // <mo stretchy=\"true\" form=\"prefix\">"
            )?;
            expand_node(open, buf)?;
            // write!(buf, "</mo>")?;
            expand_node(content, buf)?;
            // write!(buf, "<mo stretchy=\"true\" form=\"postfix\">")?;
            expand_node(close, buf)?;
            write!(buf, "</mrow>")
        }
        Node::StrechedOp(stretchy, op) => write!(buf, "<mo stretchy=\"{stretchy}\">{op}</mo>"),
        Node::OtherOperator(op) => write!(buf, "<mo>{op}</mo>"),
        Node::SizedParen { size, paren } => {
            write!(buf, "<mrow><mo maxsize=\"{size}\">{paren}</mo></mrow>")
        }
        Node::Text(text) => write!(buf, "<mtext>{text}</mtext>"),
        Node::Matrix(content, align) => {
            write!(buf, "<mtable align=\"{align}\"><mtr><mtd>")?;

            match content.as_ref() {
                Node::Row(nodes) => {
                    for (i, node) in nodes.iter().enumerate() {
                      match node {
                        Node::NewLine => {
                          write!(buf, "</mtd></mtr>")?;
                          if i < nodes.len() {
                              write!(buf, "<mtr><mtd>")?;
                          }
                        }
                        Node::Ampersand => {
                          write!(buf, "</mtd>")?;
                          if i < nodes.len() {
                              write!(buf, "<mtd>")?;
                          }
                        }
                        node => expand_node(node, buf)?,
                      }
                    }
                },
                node => expand_node(node, buf)?,
            }
            write!(buf, "</mtr></mtd></mtable>")
        }
        Node::Ampersand => write!(buf, "<mo>&#x0026;</mo>"),
        Node::NewLine => write!(buf, "<mspace linebreak=\"newline\"/>"),
        Node::Slashed(content) => {
            write!(buf, "<menclose notation=\"updiagonalstrike\">")?;
            expand_node(content, buf)?;
            write!(buf, "</menclose>")
        }
        Node::Style(display, content) => {
            match display {
                DisplayStyle::Block => write!(buf, "<mstyle displaystyle=\"true\">"),
                DisplayStyle::Inline => write!(buf, "<mstyle displaystyle=\"false\">"),
            }?;
            expand_node(content, buf)?;
            write!(buf, "</mstyle>")
        }
        Node::Undefined(token) => write!(buf, "<merror>{token:?}</merror>"),
    }
}
