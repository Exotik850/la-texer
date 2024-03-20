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

impl std::fmt::Display for Variant {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self {
          Variant::Normal              => write!(f, "normal"),
          Variant::Italic              => write!(f, "italic"),
          Variant::Bold                => write!(f, "bold"),
          Variant::BoldItalic          => write!(f, "bold-italic"),
          Variant::DoubleStruck        => write!(f, "double-struck"),
          Variant::BoldFraktur         => write!(f, "bold-fraktur"),
          Variant::Script              => write!(f, "script"),
          Variant::BoldScript          => write!(f, "bold-script"),
          Variant::Fraktur             => write!(f, "fraktur"),
          Variant::SansSerif           => write!(f, "sans-serif"),
          Variant::BoldSansSerif       => write!(f, "bold-sans-serif"),
          Variant::SansSerifItalic     => write!(f, "sans-serif-italic"),
          Variant::SansSerifBoldItalic => write!(f, "sans-serif-bold-italic"),
          Variant::Monospace           => write!(f, "monospace"),
      }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayStyle {
    Block,
    Inline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Accent {
    True,
    False,
}

impl std::fmt::Display for Accent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self {
          Accent::True  => write!(f, "true"),
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
          LineThickness::Thin   => write!(f, "thin"),
          LineThickness::Medium => write!(f, "medium"),
          LineThickness::Thick  => write!(f, "thick"),
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
          ColumnAlign::Left   => write!(f, "left"),
          ColumnAlign::Right  => write!(f, "right"),
      }
  }
}

/// AST node
#[derive(Debug, Clone, PartialEq)]
pub enum Node<'a> {
    Number(&'a str),
    Letter(&'a str, Variant),
    Operator(&'a str),
    Function(&'a str, Option<Box<Node<'a>>>),
    Space(f32),
    Subscript(Box<Node<'a>>, Box<Node<'a>>),
    Superscript(Box<Node<'a>>, Box<Node<'a>>),
    SubSup {
        target: Box<Node<'a>>,
        sub: Box<Node<'a>>,
        sup: Box<Node<'a>>,
    },
    OverOp(&'a str, Accent, Box<Node<'a>>),
    UnderOp(&'a str, Accent, Box<Node<'a>>),
    Overset {
        over: Box<Node<'a>>,
        target: Box<Node<'a>>,
    },
    Underset {
        under: Box<Node<'a>>,
        target: Box<Node<'a>>,
    },
    Under(Box<Node<'a>>, Box<Node<'a>>),
    UnderOver {
        target: Box<Node<'a>>,
        under: Box<Node<'a>>,
        over: Box<Node<'a>>,
    },
    Sqrt(Option<Box<Node<'a>>>, Box<Node<'a>>),
    Frac(Box<Node<'a>>, Box<Node<'a>>, LineThickness),
    Row(Vec<Node<'a>>),
    Fenced {
        open: &'a str,
        close: &'a str,
        content: Box<Node<'a>>,
    },
    StrechedOp(bool, &'a str),
    OtherOperator(&'a str),
    SizedParen {
        size: &'a str,
        paren: char,
    },
    Text(&'a str),
    Matrix(Box<Node<'a>>, ColumnAlign),
    Ampersand,
    NewLine,
    Slashed(Box<Node<'a>>),
    Style(Option<DisplayStyle>, Box<Node<'a>>),
    Undefined(Token<'a>),
}

impl<'a> Node<'a> {
  pub fn arg(self) -> Box<Node<'a>> {
    match self {
      Node::Fenced { open: "{", close: "}", content } => content,
      _ => Box::new(self),
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