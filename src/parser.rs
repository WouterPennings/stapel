use std::collections::HashSet;
use std::fmt::Display;

use uuid::Uuid;

use crate::operators::InfixOperators;
use crate::program::Program;
use crate::tokens::{Token, TokenType};
use crate::{throw_exception, throw_exception_span};

#[derive(Debug, PartialEq, Clone)]
pub struct Instruction {
    pub instruction_type: InstructionType,
    pub id: Uuid,
}

impl Instruction {
    pub fn new(instruction_type: InstructionType) -> Instruction {
        Instruction {
            instruction_type,
            id: Uuid::new_v4(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum InstructionType {
    Push(PushType),
    InfixOperators(InfixOperators),
    While(While),
    If(If),
    Pop,
    Swap,
    Put,
    Dup,
    Size,
    Mem,
    Return,
    Load(usize),
    Store(usize),
    Syscall(u8),
    Procedure(Procedure),
    CustomInstruction(String),
}

impl Display for InstructionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            InstructionType::Push(PushType::Int(i)) => format!("PushInt({})", i),
            InstructionType::Push(PushType::Str(_, original)) => {
                format!("PushStr(\"{}\")", original)
            }
            InstructionType::InfixOperators(op) => format!("InfixOperator({})", op),
            InstructionType::While(_) => String::from("While"),
            InstructionType::If(_) => format!("If"),
            InstructionType::Pop => format!("Pop"),
            InstructionType::Swap => format!("Swap"),
            InstructionType::Put => format!("Put"),
            InstructionType::Dup => format!("Dup"),
            InstructionType::Size => format!("Size"),
            InstructionType::Mem => format!("Mem"),
            InstructionType::Return => format!("Return"),
            InstructionType::Load(i) => format!("Load({})", i),
            InstructionType::Store(i) => format!("Store({})", i),
            InstructionType::Syscall(syscall) => format!("Syscall({})", syscall),
            InstructionType::Procedure(proc) => format!("Procedure({})", proc.identifier),
            InstructionType::CustomInstruction(str) => format!("Custom({})", str),
        };

        write!(f, "{}", value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub instructions: Vec<Instruction>,
}

impl Block {
    pub fn parse(p: &mut Parser, closing_token: TokenType) -> Result<Block, ()> {
        let mut instructions: Vec<Instruction> = vec![];

        while p.current_token().is_ok() && p.current_token().unwrap().token != closing_token {
            let instruction = p.parse_instruction()?;

            instructions.push(instruction);

            p.next_token();
        }

        // if p.current_token().is_ok() && p.current_token().unwrap().token == closing_token {
        //     p.next_token();
        // } else {
        //     println!("ERROR :::: {:?}", p.current_token());
        //     panic!("Block is not closed, there is not END instruction");
        // }

        Ok(Block { instructions })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PushType {
    Str(String, String),
    Int(i32),
}

#[derive(Debug, PartialEq, Clone)]
pub struct If {
    pub if_block: Block,
    pub else_block: Option<Block>,
}

impl If {
    pub fn parse(p: &mut Parser) -> Result<InstructionType, ()> {
        let mut instructions: Vec<Instruction> = vec![];

        p.next_token()?; // Skipping over IF token
        while p.current_token().is_ok()
            && (p.current_token().unwrap().token != TokenType::Else
                && p.current_token().unwrap().token != TokenType::End)
        {
            let instruction = p.parse_instruction()?;

            instructions.push(instruction);

            p.next_token();
        }
        let if_block = Block { instructions };

        if p.current_token().unwrap().token == TokenType::End {
            Ok(InstructionType::If(If {
                if_block,
                else_block: None,
            }))
        } else {
            p.next_token()?;
            let else_block = Block::parse(p, TokenType::End)?;

            Ok(InstructionType::If(If {
                if_block,
                else_block: Some(else_block),
            }))
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct While {
    pub condition: Block,
    pub block: Block,
}

impl While {
    pub fn parse(p: &mut Parser) -> Result<InstructionType, ()> {
        p.next_token()?; // Skipping over WHILE token
        let condition = Block::parse(p, TokenType::Do)?;

        p.next_token()?;
        let block = Block::parse(p, TokenType::End)?;

        Ok(InstructionType::While(While { condition, block }))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Procedure {
    pub identifier: String,
    pub block: Block,
}

impl Procedure {
    pub fn parse(p: &mut Parser) -> Result<Procedure, ()> {
        let identifier = p.next_token()?; // Skipping the PROC token

        let TokenType::Custom(identifier) = identifier.token.clone() else { // Getting the IDENTIFIER
            throw_exception_span(&identifier.span, "Define a procudure as: proc <identifier> do <block> end. You forgot the identifier".to_string());
            unreachable!();
        };

        p.procedures_identifiers.insert(identifier.clone());

        if !p.next_token_is(TokenType::Do) {
            throw_exception_span(&p.current_token().unwrap().span, "Define a procudure as: proc <identifier> do <block> end. You forgot the \"do\" instruction".to_string());
        }
        p.next_token()?; // Going to the DO token
        p.next_token()?; // Skipping over the DO token

        let mut block = Block::parse(p, TokenType::End)?; // Getting the procedure block
        p.next_token(); // Is Err(()) when at end of file

        if block.instructions.last().unwrap().instruction_type != InstructionType::Return
            && identifier != "main"
        {
            block
                .instructions
                .push(Instruction::new(InstructionType::Return));
        }

        Ok(Procedure { identifier, block })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parser {
    pub program: Program,
    procedures_identifiers: HashSet<String>,
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            cursor: 0,
            program: Program::new(),
            procedures_identifiers: HashSet::new(),
        }
    }

    pub fn parse(&mut self) {
        loop {
            if let Ok(token) = self.current_token() {
                if let TokenType::Procedure = token.token {
                    let Ok(proc) = Procedure::parse(self) else {
                        panic!("Cannot parse procedure")
                    };

                    self.program.procedures.push(proc);
                } else {
                    throw_exception_span(&token.span, format!("\"{:?}\" should be a procedure declaration, no instructions are allowed on toplevel", token.token));
                }
            } else {
                // No tokens left
                break;
            }
        }

        if !self.procedures_identifiers.contains("main") {
            throw_exception("No entry point is found in this program. Make sure there is a procedure named \"main\"".to_string())
        }
    }

    fn parse_instruction(&mut self) -> Result<Instruction, ()> {
        let token = self.current_token()?;
        let instruction_type = match &token.token {
            TokenType::PushInt(int) => InstructionType::Push(PushType::Int(*int)),
            TokenType::PushStr(original, value) => {
                InstructionType::Push(PushType::Str(original.clone(), value.clone()))
            }
            TokenType::InfixOperators(operator) => {
                InstructionType::InfixOperators(operator.clone())
            }
            TokenType::Pop => InstructionType::Pop,
            TokenType::Swap => InstructionType::Swap,
            TokenType::Put => InstructionType::Put,
            TokenType::Dup => InstructionType::Dup,
            TokenType::Size => InstructionType::Size,
            TokenType::Mem => InstructionType::Mem,
            TokenType::While => While::parse(self)?,
            TokenType::If => If::parse(self)?,
            TokenType::Procedure => panic!("Should not encounter PROC here"),
            TokenType::Else => panic!("Should not encounter ELSE here"),
            TokenType::Do => panic!("Should not encounter DO here"),
            TokenType::End => panic!("Should not encounter END here"),
            TokenType::Load(i) => InstructionType::Load(*i),
            TokenType::Store(i) => InstructionType::Store(*i),
            TokenType::Syscall(i) => InstructionType::Syscall(*i),
            TokenType::Custom(identifier) => {
                if self.procedures_identifiers.contains(identifier) {
                    InstructionType::CustomInstruction(identifier.to_string())
                } else {
                    throw_exception_span(
                        &token.span,
                        format!(
                            "A procedure named: \"{}\", has not been defined",
                            identifier
                        ),
                    );
                    unreachable!()
                }
            }
            TokenType::Return => InstructionType::Return,
        };

        Ok(Instruction {
            instruction_type,
            id: Uuid::new_v4(),
        })
    }

    fn current_token(&self) -> Result<&Token, ()> {
        if self.tokens.get(self.cursor).is_some() {
            Ok(self.tokens.get(self.cursor).unwrap())
        } else {
            Err(())
        }
    }

    fn peek_token(&self) -> Result<&Token, ()> {
        if self.tokens.get(self.cursor + 1).is_some() {
            Ok(self.tokens.get(self.cursor + 1).unwrap())
        } else {
            Err(())
        }
    }

    fn next_token(&mut self) -> Result<&Token, ()> {
        self.cursor += 1;
        self.current_token()
    }

    fn next_token_is(&mut self, tokentype: TokenType) -> bool {
        self.peek_token().is_ok() && self.peek_token().unwrap().token == tokentype
    }
}
