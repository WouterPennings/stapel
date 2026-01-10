use std::collections::{HashSet};
use std::fmt::Display;

use crate::operators::{InfixOperators};
use crate::program::{Program};
use crate::tokens::{Token, TokenType};
use crate::{throw_exception, throw_exception_span};

#[derive(Debug, PartialEq, Clone)]
pub struct Instruction {
    pub instruction_type: InstructionType,
}

impl Instruction {
    pub fn new(instruction_type: InstructionType) -> Instruction {
        Instruction {
            instruction_type,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum InstructionType {
    Put,
    Push(PushType),
    InfixOperators(InfixOperators),
    While(While),
    If(If),
    Pop,
    Dup,
    Over,
    Pick,
    Swap,
    Rot,
    Size,
    Load(usize),
    Store(usize),
    Identifier(String),
    Return,
    Syscall(u8),
}

impl InstructionType {
    pub fn pops(&self) -> u8 {
        match self {
            InstructionType::Push(_) => 0,
            InstructionType::InfixOperators(_) => 2,
            InstructionType::While(_) => 1,
            InstructionType::If(_) => 1,
            InstructionType::Pop => 1,
            InstructionType::Swap => 2,
            InstructionType::Rot => 3,
            InstructionType::Over => 2,
            InstructionType::Put => 1,
            InstructionType::Pick => 0,
            InstructionType::Dup => 1,
            InstructionType::Size => 0,
            InstructionType::Return => 0,
            InstructionType::Load(_) => 1,
            InstructionType::Store(_) => 2,
            InstructionType::Syscall(registers) => *registers,
            InstructionType::Identifier(_) => 0,
        }
    }
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
            InstructionType::Rot => format!("Rot"),
            InstructionType::Over => format!("Over"),
            InstructionType::Pick => format!("Pick"),
            InstructionType::Put => format!("Put"),
            InstructionType::Dup => format!("Dup"),
            InstructionType::Size => format!("Size"),
            InstructionType::Return => format!("Return"),
            InstructionType::Load(i) => format!("Load({})", i),
            InstructionType::Store(i) => format!("Store({})", i),
            InstructionType::Syscall(syscall) => format!("Syscall({})", syscall),
            InstructionType::Identifier(str) => format!("Custom({})", str),
        };

        write!(f, "{}", value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub instructions: Vec<Instruction>,
}

impl Block {
    pub fn parse(p: &mut Parser, closing_tokens: &[TokenType]) -> Result<Block, ()> {
        let mut instructions: Vec<Instruction> = vec![];

        while p.current_token().is_ok() && !closing_tokens.contains(&p.current_token().unwrap().token) {
            let instruction = p.parse_instruction()?;

            instructions.push(instruction);

            let _ = p.next_token();
        }

        Ok(Block { instructions })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PushType {
    Str(String, String),
    Int(i64),
}

#[derive(Debug, PartialEq, Clone)]
pub struct If {
    pub if_block: (Block, Block),             // (Condition, Body)
    pub elif_blocks: Vec<(Block, Block)>,     // List of (Condition, Body)
    pub else_block: Option<Block>,
}

impl If {
    pub fn parse(p: &mut Parser) -> Result<InstructionType, ()> {
        p.next_token()?; // Skip 'if'

        // 1. Primary IF
        let if_cond = Block::parse(p, &[TokenType::Do])?;
        p.next_token()?; // Skip 'do'
        let if_body = Block::parse(p, &[TokenType::Elif, TokenType::Else, TokenType::End])?;

        let mut elif_blocks = Vec::new();

        // 2. Loop for ELIFs
        while p.current_token()?.token == TokenType::Elif {
            p.next_token()?; // Skip 'elif'
            let cond = Block::parse(p, &[TokenType::Do])?;
            p.next_token()?; // Skip 'do'
            let body = Block::parse(p, &[TokenType::Elif, TokenType::Else, TokenType::End])?;
            elif_blocks.push((cond, body));
        }

        // 3. Optional ELSE
        let mut else_block = None;
        if p.current_token()?.token == TokenType::Else {
            p.next_token()?; // Skip 'else'
            else_block = Some(Block::parse(p, &[TokenType::End])?);
        }

        // Current token should be 'End' now
        if p.current_token()?.token != TokenType::End {
            return Err(()); // Syntax error
        }

        Ok(InstructionType::If(If {
            if_block: (if_cond, if_body),
            elif_blocks,
            else_block,
        }))
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
        let condition = Block::parse(p, &[TokenType::Do])?;

        p.next_token()?;
        let block = Block::parse(p, &[TokenType::End])?;

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

        let TokenType::Identifier(identifier) = identifier.token.clone() else { // Getting the IDENTIFIER
            throw_exception_span(&identifier.span, "Define a procudure as: proc <identifier> do <block> end. You forgot the identifier".to_string());
            unreachable!();
        };

        // Checking if identifier already exists
        if p.procedures_identifiers.contains(&identifier) {
            throw_exception_span(&p.current_token().unwrap().span, format!("inline named '{}', is already a procedure name", identifier));
        } else if p.inline_statements.contains(&identifier) {
            throw_exception_span(&p.current_token().unwrap().span, format!("inline named '{}', is already an inline name", identifier));
        } else if p.memories.contains(&identifier) {
            throw_exception_span(&p.current_token().unwrap().span, format!("inline named '{}', is already a memory ", identifier));
        }

        p.next_token()?; // Going to the DO token
        if !p.current_token_is(TokenType::Do) {
            throw_exception_span(&p.current_token().unwrap().span, "Define a procudure as: proc <identifier> do <block> end. You forgot the \"do\" instruction".to_string());
        }
        p.next_token()?; // Skipping over the DO token, going to block
        
        let mut block = Block::parse(p, &[TokenType::End])?; // Getting the procedure block
        let _ = p.next_token(); // Is Err(()) when at end of file
        if (block.instructions.is_empty() || block.instructions.last().unwrap().instruction_type != InstructionType::Return)
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
pub struct Inline {
    pub identifier: String,
    pub block: Block,
}

impl Inline {
    pub fn parse(p: &mut Parser) -> Result<Inline, ()> {
        let identifier = p.next_token()?; // Skipping the INLINE token

        let TokenType::Identifier(identifier) = identifier.token.clone() else { // Getting the IDENTIFIER
            throw_exception_span(&identifier.span, "Define a procudure as: inline <identifier> <block> end. You forgot the identifier".to_string());
            unreachable!();
        };

        // Checking if identifier already exists
        if p.procedures_identifiers.contains(&identifier) {
            throw_exception_span(&p.current_token().unwrap().span, format!("inline named '{}', is already a procedure name", identifier));
        } else if p.inline_statements.contains(&identifier) {
            throw_exception_span(&p.current_token().unwrap().span, format!("inline named '{}', is already an inline name", identifier));
        } else if p.memories.contains(&identifier) {
            throw_exception_span(&p.current_token().unwrap().span, format!("inline named '{}', is already a memory ", identifier));
        }

        p.next_token()?; // Skipping over the IDENTIFIER token, going to block

        let block = Block::parse(p, &[TokenType::End])?; // Getting the procedure block
        let _ = p.next_token(); // Is Err(()) when at end of file

        Ok(Inline {identifier, block})
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Memory {
    pub identifier: String,
    /// Size in bytes
    pub size: usize,
}

impl Memory {
    pub fn parse(p: &mut Parser) -> Result<Memory, ()> {
        let identifier = p.next_token()?; // Skipping the INLINE token

        let TokenType::Identifier(identifier) = identifier.token.clone() else { // Getting the IDENTIFIER
            throw_exception_span(&identifier.span, "Define a procudure as: memory <identifier> <size> end. You forgot the identifier".to_string());
            unreachable!();
        };

        // Checking if identifier already exists
        if p.procedures_identifiers.contains(&identifier) {
            throw_exception_span(&p.current_token().unwrap().span, format!("inline named '{}', is already a procedure name", identifier));
        } else if p.inline_statements.contains(&identifier) {
            throw_exception_span(&p.current_token().unwrap().span, format!("inline named '{}', is already an inline name", identifier));
        } else if p.memories.contains(&identifier) {
            throw_exception_span(&p.current_token().unwrap().span, format!("inline named '{}', is already a memory ", identifier));
        }

        let size = p.next_token()?; // Skipping over the IDENTIFIER token, going to SIZE

        let TokenType::PushInt(size) = size.token.clone() else { // Getting the IDENTIFIER
            throw_exception_span(&size.span, "Define a procudure as: memory <identifier> <size> end. You forgot the identifier".to_string());
            unreachable!();
        };
        let _ = p.next_token(); // skipping over SIZE
        let _ = p.next_token(); // skipping over END

        Ok(Memory {identifier, size: size as usize})
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parser {
    pub program: Program,
    procedures_identifiers: HashSet<String>,
    inline_statements: HashSet<String>,
    memories: HashSet<String>,
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        let mut program = Program::new();
        program.memories.insert("argc".to_string(), Memory { identifier: "argc".to_string(), size: 64 });
        program.memories.insert("argv".to_string(), Memory { identifier: "argv".to_string(), size: 64 });
        
        let mut memories = HashSet::new();
        memories.insert(String::from("argv"));
        memories.insert(String::from("argc"));

        Parser {
            tokens,
            cursor: 0,
            program,
            procedures_identifiers: HashSet::new(),
            inline_statements: HashSet::new(),
            memories,
        }
    }

    pub fn parse(&mut self) {
        loop {
            if let Ok(token) = self.current_token() {
                if let TokenType::Procedure = token.token {
                    let Ok(proc) = Procedure::parse(self) else {
                        panic!("Cannot parse procedure")
                    };

                    self.procedures_identifiers.insert(proc.identifier.clone());
                    self.program.procedures.insert(proc.identifier.clone(), proc);
                } else if let TokenType::Memory = token.token {
                    let Ok(memory) = Memory::parse(self) else {
                        panic!("Cannot parse memory statement")
                    };

                    self.memories.insert(memory.identifier.clone());
                    self.program.memories.insert(memory.identifier.clone(), memory);
                }
                else if let TokenType::Inline = token.token {
                    let Ok(inline) = Inline::parse(self) else {
                        panic!("Cannot parse inline statement")
                    };
                    
                    self.inline_statements.insert(inline.identifier.clone());
                    self.program.inlines.insert(inline.identifier.clone(), inline);
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
            TokenType::Rot => InstructionType::Rot,
            TokenType::Over => InstructionType::Over,
            TokenType::Pick => InstructionType::Pick,
            TokenType::Put => InstructionType::Put,
            TokenType::Dup => InstructionType::Dup,
            TokenType::Size => InstructionType::Size,
            TokenType::Return => InstructionType::Return,
            TokenType::Load(i) => InstructionType::Load(*i),
            TokenType::Store(i) => InstructionType::Store(*i),
            TokenType::Syscall(i) => InstructionType::Syscall(*i),
            TokenType::Identifier(identifier) => InstructionType::Identifier(identifier.to_string()),
            TokenType::While => While::parse(self)?,
            TokenType::If => If::parse(self)?,
            TokenType::Elif => unreachable!("Should not encouter ELIF here",),
            TokenType::Else => unreachable!("Should not encounter ELSE here"),
            TokenType::Do => unreachable!("Should not encounter DO here"),
            TokenType::End => unreachable!("Should not encounter END here"),
            TokenType::Memory => unreachable!("Should not encounter MEMORY here"),
            TokenType::Procedure => unreachable!("Should not encounter PROC here"),
            TokenType::Inline => unreachable!("Should not encounter INLINE here"),
        };

        Ok(Instruction {
            instruction_type,
        })
    }

    fn current_token(&self) -> Result<&Token, ()> {
        if self.tokens.get(self.cursor).is_some() {
            Ok(self.tokens.get(self.cursor).unwrap())
        } else {
            Err(())
        }
    }

    fn current_token_is(&mut self, tokentype: TokenType) -> bool {
        self.current_token().is_ok() && self.current_token().unwrap().token == tokentype
    }

    fn next_token(&mut self) -> Result<&Token, ()> {
        self.cursor += 1;
        self.current_token()
    }

    // fn next_token_is(&mut self, tokentype: TokenType) -> bool {
    //     self.peek_token().is_ok() && self.peek_token().unwrap().token == tokentype
    // }

    // fn peek_token(&self) -> Result<&Token, ()> {
    //     if self.tokens.get(self.cursor + 1).is_some() {
    //         Ok(self.tokens.get(self.cursor + 1).unwrap())
    //     } else {
    //         Err(())
    //     }
    // }
}
