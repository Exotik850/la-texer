use std::borrow::Cow;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineThickness {
    Thin,
    Medium,
    Thick,
    Length(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnAlign {
    Center,
    Left,
    Right,
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

