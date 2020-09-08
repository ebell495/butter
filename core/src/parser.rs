use op::BinaryOp;
use op::UnaryOp;
use util::tree::TreeVec;

mod op;

#[derive(Clone, Copy)]
pub enum Num {
    UInt(u64),
    Float(f64),
}
#[derive(Clone, Copy)]
enum NodeKind {
    Splat,
    Rest,
    Label,

    CharInside(u8),

    True,
    False,
    Null,
    Ident,
    Char,
    Str,
    Num(Num),
    Path,

    Clone,
    Abort,
    Break,
    Continue,

    Unary(UnaryOp),
    Binary(BinaryOp),

    Declare,
    Assign,

    Array,
    Struct,

    Block(bool),
    Fun(bool),
    If,
    Else,
    For,
    While,
    Loop,
}
pub struct BigAst<'a>(TreeVec<(&'a str, NodeKind)>);
