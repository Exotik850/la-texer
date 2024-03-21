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
      Variant::Normal              => "normal",
      Variant::Italic              => "italic",
      Variant::Bold                => "bold",
      Variant::BoldItalic          => "bold-italic",
      Variant::DoubleStruck        => "double-struck",
      Variant::BoldFraktur         => "bold-fraktur",
      Variant::Script              => "script",
      Variant::BoldScript          => "bold-script",
      Variant::Fraktur             => "fraktur",
      Variant::SansSerif           => "sans-serif",
      Variant::BoldSansSerif       => "bold-sans-serif",
      Variant::SansSerifItalic     => "sans-serif-italic",
      Variant::SansSerifBoldItalic => "sans-serif-bold-italic",
      Variant::Monospace           => "monospace",
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
      DisplayStyle::Block  => "block",
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
    // Row(smallvec::SmallVec<[Node<'a>;N]>),
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
        paren: &'a str,
    },
    Text(&'a str),
    Matrix(Box<Node<'a>>, ColumnAlign),
    Ampersand,
    NewLine,
    Slashed(Box<Node<'a>>),
    Style(DisplayStyle, Box<Node<'a>>),
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