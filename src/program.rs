use std::collections::HashMap;

use crate::parser::{Procedure, Inline, Memory};

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub procedures: HashMap<String, Procedure>,
    pub inlines: HashMap<String, Inline>,
    pub memories: HashMap<String, Memory>,
}

impl Program {
    pub fn new() -> Program {
        Program { procedures: HashMap::new(), inlines: HashMap::new(), memories: HashMap::new() }
    }

    pub fn from(procedures: HashMap<String, Procedure>, inlines: HashMap<String, Inline>, memories: HashMap<String, Memory>) -> Program {
        Program { procedures, inlines, memories }
    }
}
