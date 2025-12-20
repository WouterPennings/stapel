use std::collections::HashMap;

use crate::parser::{Procedure, Inline, InstructionType, Memory};

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub procedures: Vec<Procedure>,
    pub inlines: HashMap<String, Inline>,
    pub memories: HashMap<String, Memory>
}

impl Program {
    pub fn new() -> Program {
        let mut memories = HashMap::new();
        // Pre-register argc and argv so the parser knows them

        Program { procedures: vec![], inlines: HashMap::new(), memories }
    }

    pub fn from(procedures: Vec<Procedure>, inlines: HashMap<String, Inline>, memories: HashMap<String, Memory>) -> Program {
        Program { procedures, inlines, memories }
    }

    /// Inlines all inline blocks at the indentifiers, and removed the identifiers
    pub fn inline(&mut self) {
        for proc in &mut self.procedures {
            // We use a simple index loop and go backwards
            let mut i = proc.block.instructions.len();
            
            while i > 0 {
                i -= 1;
                
                // Check the instruction at the current index
                if let InstructionType::Identifier(ref identifier) = proc.block.instructions[i].instruction_type {
                    if let Some(inline_proc) = self.inlines.get(identifier) {
                        // Perform the splice. 
                        // This is safe because we are going backwards; 
                        // shifting elements at 'i' doesn't affect indices 0 to i-1.
                        proc.block.instructions.splice(
                            i..i+1, 
                            inline_proc.block.instructions.clone()
                        );
                    }
                }
            }
        }
}
}
