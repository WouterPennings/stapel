use std::fmt::Display;

use crate::operators::{InfixOperators, PrefixOperator};

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
    PushInt(i32),
    PushStr(String, String),
    InfixOperators(InfixOperators),
    PrefixOperator(PrefixOperator),
    Pop,
    Swap,
    Put,
    While,
    If,
    Else,
    Do,
    End,
    Dup,
    Size,
    Mem,
    Return,
    Procedure,
    Load(usize),
    Store(usize),
    Syscall(u8),
    Custom(String),
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let value = match self {
            TokenType::PushInt(i) => format!("PushInt({})", i),
            TokenType::PushStr(_, original) => format!("PushStr(\"{}\")", original),
            TokenType::InfixOperators(op) => format!("InfixOperators({})", op),
            TokenType::PrefixOperator(op) => format!("PrefixOperator({})", op),
            TokenType::Pop => String::from("Pop"),
            TokenType::Swap => String::from("Swap"),
            TokenType::Put => String::from("Put"),
            TokenType::While => String::from("While"),
            TokenType::If => String::from("If"),
            TokenType::Else => String::from("Else"),
            TokenType::Do => String::from("Do"),
            TokenType::End => String::from("End"),
            TokenType::Dup => String::from("Dup"),
            TokenType::Size => String::from("Size"),
            TokenType::Mem => String::from("Mem"),
            TokenType::Return => String::from("Return"),
            TokenType::Load(_) => String::from("Load"),
            TokenType::Store(_) => String::from("Store"),
            TokenType::Syscall(i) => format!("Syscall{}", i),
            TokenType::Custom(_) => String::from("Custom"),
            TokenType::Procedure => String::from("Procedure"),
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
