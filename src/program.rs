use std::fmt::Display;

use crate::parser::Procedure;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub procedures: Vec<Procedure>,
}

impl Program {
    pub fn new() -> Program {
        Program { procedures: vec![] }
    }

    pub fn from(procedures: Vec<Procedure>) -> Program {
        Program { procedures }
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
