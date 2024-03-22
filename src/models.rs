use crate::token::Token;

/// mi mathvariant attribute
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Variant {
    Normal,
    Italic,
    Bold,
    BoldItalic,
    DoubleStruck,
    BoldFraktur,
    Script,
    BoldScript,
    Fraktur,
    SansSerif,
    BoldSansSerif,
    SansSerifItalic,
    SansSerifBoldItalic,
    Monospace,
}

impl Variant {
    pub fn to_str(self) -> &'static str {
        match self {
            Variant::Normal => "normal",
            Variant::Italic => "italic",
            Variant::Bold => "bold",
            Variant::BoldItalic => "bold-italic",
            Variant::DoubleStruck => "double-struck",
            Variant::BoldFraktur => "bold-fraktur",
            Variant::Script => "script",
            Variant::BoldScript => "bold-script",
            Variant::Fraktur => "fraktur",
            Variant::SansSerif => "sans-serif",
            Variant::BoldSansSerif => "bold-sans-serif",
            Variant::SansSerifItalic => "sans-serif-italic",
            Variant::SansSerifBoldItalic => "sans-serif-bold-italic",
            Variant::Monospace => "monospace",
        }
    }
}

impl std::fmt::Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayStyle {
    Block,
    Inline,
}

impl DisplayStyle {
    pub fn to_str(self) -> &'static str {
        match self {
            DisplayStyle::Block => "block",
            DisplayStyle::Inline => "inline",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Accent {
    True,
    False,
}

impl std::fmt::Display for Accent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Accent::True => write!(f, "true"),
            Accent::False => write!(f, "false"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineThickness {
    Thin,
    Medium,
    Thick,
    Length(u8),
}

impl std::fmt::Display for LineThickness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineThickness::Thin => write!(f, "thin"),
            LineThickness::Medium => write!(f, "medium"),
            LineThickness::Thick => write!(f, "thick"),
            LineThickness::Length(l) => write!(f, "{}", l),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnAlign {
    Center,
    Left,
    Right,
}

impl std::fmt::Display for ColumnAlign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnAlign::Center => write!(f, "center"),
            ColumnAlign::Left => write!(f, "left"),
            ColumnAlign::Right => write!(f, "right"),
        }
    }
}

pub trait ParseNodes<'a> {
    fn parse_latex(&'a self) -> Vec<Node<'a>>;
}

impl<'a, T> ParseNodes<'a> for T
where
    T: AsRef<str>,
{
    fn parse_latex(&'a self) -> Vec<Node<'a>> {
        crate::Parser::new(self.as_ref()).parse()
    }
}

type NodeBox<'a> = Box<Node<'a>>;

// What other nodes need variant?
/// AST node
#[derive(Debug, Clone, PartialEq)]
pub enum Node<'a> {
    Package(&'a str),
    Title(&'a str),
    Number(&'a str),
    Operator(&'a str),
    Text(&'a str, Variant),
    Letter(&'a str, Variant),
    Function(&'a str, Option<NodeBox<'a>>),
    Space(f32),
    Subscript(NodeBox<'a>, NodeBox<'a>),
    Superscript(NodeBox<'a>, NodeBox<'a>),
    SubSup {
        target: NodeBox<'a>,
        sub: NodeBox<'a>,
        sup: NodeBox<'a>,
    },
    OverOp(&'a str, Accent, NodeBox<'a>),
    UnderOp(&'a str, Accent, NodeBox<'a>),
    Overset {
        over: NodeBox<'a>,
        target: NodeBox<'a>,
    },
    Underset {
        under: NodeBox<'a>,
        target: NodeBox<'a>,
    },
    Under(NodeBox<'a>, NodeBox<'a>),
    UnderOver {
        target: NodeBox<'a>,
        under: NodeBox<'a>,
        over: NodeBox<'a>,
    },
    Sqrt(Option<NodeBox<'a>>, NodeBox<'a>),
    Frac(NodeBox<'a>, NodeBox<'a>, LineThickness),
    // Row(smallvec::SmallVec<[Node<'a>;N]>),
    Row(Vec<Node<'a>>),
    Fenced {
        open: NodeBox<'a>,
        close: NodeBox<'a>,
        content: NodeBox<'a>,
    },
    StrechedOp(bool, &'a str),
    OtherOperator(&'a str),
    SizedParen {
        size: &'a str,
        paren: &'a str,
    },
    Matrix(NodeBox<'a>, ColumnAlign),
    Ampersand,
    NewLine,
    Slashed(NodeBox<'a>),
    Style(DisplayStyle, NodeBox<'a>),
    Undefined(Token<'a>),
}

impl<'a> Node<'a> {
    pub fn arg(self) -> NodeBox<'a> {
        match self {
            Node::Fenced {
                open,
                close,
                content,
            } if *open == Node::StrechedOp(true, "{") && *close == Node::StrechedOp(true, "}") => content,
            _ => Box::new(self),
        }
    }

    /// Returns the inner string of the node if it is a Node that contains a string.
    pub fn inner_str(&'a self) -> Option<&'a str> {
        match self {
            Node::Number(s)
            | Node::Letter(s, _)
            | Node::Operator(s)
            | Node::Function(s, _)
            | Node::OtherOperator(s)
            | Node::StrechedOp(_, s)
            | Node::UnderOp(s, _, _)
            | Node::OverOp(s, _, _) => Some(s),
            Node::Text(s, _) | Node::SizedParen { paren: s, .. } => Some(s),
            _ => None,
        }
    }
}

// pub struct NodeIter<'a> {
//   node: &'a Node<'a>,
//   index: usize,
// }

// impl<'a> Iterator for NodeIter<'a> {
//   type Item = &'a Node<'a>;

//   fn next(&mut self) -> Option<Self::Item> {
//     match self.node {
//       Node::Row(nodes) => {
//         if self.index < nodes.len() {
//           let node = &nodes[self.index];
//           self.index += 1;
//           Some(node)
//         } else {
//           None
//         }
//       },
//       node if self.index == 0 => {
//         self.index += 1;
//         Some(node)
//       },
//       _ => None,
//     }
//   }
// }

// impl<'a> Node<'a> {
//   pub fn iter(&'a self) -> NodeIter<'a> {
//     NodeIter {
//       node: self,
//       index: 0,
//     }
//   }

//   pub fn len(&self) -> usize {
//     match self {
//       Node::Row(nodes) => nodes.len(),
//       _ => 1,
//     }
//   }
// }
