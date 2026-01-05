use std::fmt::Display;

use crate::operators::{InfixOperators};

#[derive(Debug, PartialEq, Clone)]
pub struct Span {
    pub file: String,
    pub row: usize,
    pub column: usize,
}

impl Span {
    pub fn new(file: String, row: usize, column: usize) -> Span {
        Span { file, row, column }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "File: \"{}\", Row: {}, Column: {}",
            self.file, self.row, self.column
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    PushInt(i64),
    PushStr(String, String),
    InfixOperators(InfixOperators),
    Pop,
    Swap,
    Rot,
    Over,
    Pick,
    Put,
    While,
    If,
    Elif,
    Else,
    Do,
    End,
    Dup,
    Size,
    Memory,
    Return,
    Procedure,
    Inline,
    Load(usize),
    Store(usize),
    Syscall(u8),
    Identifier(String),
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let value = match self {
            TokenType::PushInt(i) => format!("PushInt({})", i),
            TokenType::PushStr(_, original) => format!("PushStr(\"{}\")", original),
            TokenType::InfixOperators(op) => format!("InfixOperators({})", op),
            TokenType::Pop => String::from("Pop"),
            TokenType::Swap => String::from("Swap"),
            TokenType::Rot => String::from("Rot"),
            TokenType::Over => String::from("Over"),
            TokenType::Pick => String::from("Pick"),
            TokenType::Put => String::from("Put"),
            TokenType::While => String::from("While"),
            TokenType::If => String::from("If"),
            TokenType::Elif => String::from("Elif"),
            TokenType::Else => String::from("Else"),
            TokenType::Do => String::from("Do"),
            TokenType::End => String::from("End"),
            TokenType::Dup => String::from("Dup"),
            TokenType::Size => String::from("Size"),
            TokenType::Memory => String::from("Memory"),
            TokenType::Procedure => String::from("Procedure"),
            TokenType::Return => String::from("Return"),
            TokenType::Inline => String::from("Inline"),
            TokenType::Load(_) => String::from("Load"),
            TokenType::Store(_) => String::from("Store"),
            TokenType::Syscall(i) => format!("Syscall{}", i),
            TokenType::Identifier(_) => String::from("Custom"),
        };

        write!(f, "{}", value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token: TokenType,
    pub span: Span,
}

impl Token {
    pub fn new(token: TokenType, span: Span) -> Token {
        Token { token, span }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [ {} ]", self.token, self.span)
    }
}
