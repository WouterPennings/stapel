use std::process::id;

use crate::operators::{InfixOperators};
use crate::parser::{Block, Instruction, InstructionType, PushType};
use crate::program::Program;

pub struct Compiler {
    pub code: String,
    cursor: usize,
    program: Program,
    strings: Vec<(String, String)>,
    label_count: usize, 
    inline_expansion_stack: Vec<String>,
}

impl Compiler {
    pub fn new(program: Program) -> Compiler {
        Compiler {
            program,
            cursor: 0,
            strings: Vec::new(),
            label_count: 0,
            code: String::from(format!("{}\n", include_str!("start_asm_x86_64.asm"))),
            inline_expansion_stack: Vec::new(),
        }
    }

    /// Generates a unique label ID and increments the counter
    fn next_label(&mut self) -> usize {
        let id = self.label_count;
        self.label_count += 1;
        id
    }

    pub fn compile_x86_64(&mut self) {
        // Compile all procedures
        for procedure in self.program.procedures.clone() {
            self.add_proc(procedure.identifier);
            self.compile_block(&procedure.block);
            
            // Default return for every procedure to prevent falling through
            self.add_instruction("test r13, r13");
            self.add_instruction("jz .exit_proc"); 
            self.add_instruction("mov rdx, [ret_stack + r13 * 8]");
            self.add_instruction("dec r13");
            self.add_instruction("jmp rdx");
            self.code.push_str(".exit_proc:\n");
        }

        // Global exit point
        self.code.push_str("\t; === GLOBAL EXIT ===\n");
        self.add_instruction("mov rax, 60");
        self.add_instruction("mov rdi, 0");
        self.add_instruction("syscall\n");

        // BSS Section (Variables)
        self.code.push_str("section .bss\n");
        for (identifier, memory) in &self.program.memories {
            if identifier == "argv" || identifier == "argc" { continue; }
            self.code.push_str(format!("{}: resb {}\n", memory.identifier, memory.size).as_str());
        }
        self.code.push_str("\n");

        // Data Section (Strings)
        self.code.push_str("section .data\n");
        self.code.push_str("; Strings with null terminators\n");
        for (i, (str, original)) in self.strings.iter().enumerate() {
            self.code.push_str(
                format!("str_{}: {}, 0 ; \"{}\"\n", 
                    i, 
                    self.string_to_asm_data(str.clone()), 
                    original
                ).as_str()
            );
        }
    }

    fn compile_block(&mut self, block: &Block) {
        for instruction in &block.instructions {
            self.add_instruction_comment(&instruction);

            match &instruction.instruction_type {
                InstructionType::Put => {
                    self.add_instruction("pop rdi");
                    self.add_instruction("call print_i64");
                }
                InstructionType::Push(PushType::Int(i)) => {
                    self.add_instruction_string(format!("mov rax, {}", i));
                    self.add_instruction("push rax");
                }
                InstructionType::Push(PushType::Str(str, original)) => {
                    // Pushes [length, address]
                    self.add_instruction_string(format!("push {}", str.len()));
                    self.add_instruction_string(format!("push str_{}", self.strings.len()));
                    self.strings.push((str.clone(), original.clone()));
                }
                InstructionType::InfixOperators(op) => {
                    self.add_instruction("pop rbx"); // Right operand
                    self.add_instruction("pop rax"); // Left operand

                    match op {
                        InfixOperators::Plus | InfixOperators::Minus  => {
                            self.add_instruction_string(format!("{} rax, rbx", op.to_x86_64_instruction()));
                            self.add_instruction("push rax");
                        }
                        InfixOperators::Multiply | InfixOperators::Divide => {
                            // Multiply/Divide use RAX implicitly
                            self.add_instruction("xor rdx, rdx");
                            self.add_instruction_string(format!("{} rbx", op.to_x86_64_instruction()));
                            self.add_instruction("push rax");
                        }
                        InfixOperators::Modulo => {
                            self.add_instruction("xor rdx, rdx");
                            self.add_instruction("cqo");
                            self.add_instruction_string(format!("idiv rbx"));
                            self.add_instruction("push rdx");
                        }
                        InfixOperators::And => {
                            self.add_instruction("cmp rax, 0");
                            self.add_instruction("setne al");
                            self.add_instruction("cmp rbx, 0");
                            self.add_instruction("setne bl");
                            self.add_instruction_string(format!("{} al, bl", op.to_x86_64_instruction()));
                            self.add_instruction("movzx rax, al");
                            self.add_instruction("push rax");
                        }
                        InfixOperators::Or => {
                            self.add_instruction_string(format!("{} rax, rbx", op.to_x86_64_instruction()));
                            self.add_instruction("push rax");
                        }
                        _ => {
                            // Comparison operators
                            self.add_instruction("xor rcx, rcx");
                            self.add_instruction("cmp rax, rbx");
                            self.add_instruction_string(format!("{} cl", op.to_x86_64_instruction()));
                            self.add_instruction("push rcx");
                        }
                    };
                }
                InstructionType::While(whl) => {
                    let start_label = self.next_label();
                    let end_label = self.next_label();

                    self.add_label(start_label);
                    self.compile_block(&whl.condition);

                    self.add_instruction("pop rax");
                    self.add_instruction("cmp rax, 0");
                    self.add_instruction_string(format!("je .addr_{}", end_label));

                    self.compile_block(&whl.block);
                    self.add_instruction_string(format!("jmp .addr_{}", start_label));
                    self.add_label(end_label);
                }
                InstructionType::If(iff) => {
                    let end_label = self.next_label();
                    
                    // --- Compile IF ---
                    let next_branch_label = self.next_label();
                    self.compile_block(&iff.if_block.0); // Condition
                    self.add_instruction("pop rax");
                    self.add_instruction("cmp rax, 0");
                    self.add_instruction_string(format!("je .addr_{}", next_branch_label));
                    
                    self.compile_block(&iff.if_block.1); // Body
                    self.add_instruction_string(format!("jmp .addr_{}", end_label));
                    self.add_label(next_branch_label);

                    // --- Compile ELIFs ---
                    for (cond, body) in &iff.elif_blocks {
                        let next_elif_label = self.next_label();
                        self.compile_block(cond);
                        self.add_instruction("pop rax");
                        self.add_instruction("cmp rax, 0");
                        self.add_instruction_string(format!("je .addr_{}", next_elif_label));
                        
                        self.compile_block(body);
                        self.add_instruction_string(format!("jmp .addr_{}", end_label));
                        self.add_label(next_elif_label);
                    }

                    // --- Compile ELSE ---
                    if let Some(else_block) = &iff.else_block {
                        self.compile_block(else_block);
                    }

                    self.add_label(end_label);
                }
                InstructionType::Pop => {
                    self.add_instruction("pop rax");
                }
                InstructionType::Dup => {
                    self.add_instruction("pop rax");
                    self.add_instruction("push rax");
                    self.add_instruction("push rax");
                }  
                InstructionType::Over => {
                    // ( a b -- a b a )
                    self.add_instruction("pop rax");
                    self.add_instruction("pop rbx");
                    self.add_instruction("push rbx");
                    self.add_instruction("push rax");
                    self.add_instruction("push rbx");
                }
                InstructionType::Pick => {
                    self.add_instruction("pop rax");          
                    self.add_instruction("shl rax, 3");       // rax = N * 8 (shift left by 3 is same as * 8)
                    self.add_instruction("mov rbx, [rsp + rax]"); // Get the value at that memory offset
                    self.add_instruction("push rbx");         
                }             
                InstructionType::Swap => {
                    self.add_instruction("pop rax");
                    self.add_instruction("pop rbx");
                    self.add_instruction("push rax");
                    self.add_instruction("push rbx");
                }
                InstructionType::Rot => {
                    // ( a b c -- b c a )
                    self.add_instruction("pop rcx");
                    self.add_instruction("pop rbx");
                    self.add_instruction("pop rax");
                    self.add_instruction("push rbx");
                    self.add_instruction("push rcx");
                    self.add_instruction("push rax");
                }
                InstructionType::Size => {
                    self.add_instruction("mov rax, rsp");
                    self.add_instruction("sub rax, [ori_stack_ptr]");
                    self.add_instruction("neg rax");
                    self.add_instruction("shr rax, 3"); // Divide by 8 bytes
                    self.add_instruction("push rax");
                }
                InstructionType::Load(size) => {
                    self.add_instruction("pop rax");
                    self.add_instruction("xor rbx, rbx");
                    match size {
                        1 => self.add_instruction("mov bl, [rax]"),
                        2 => self.add_instruction("mov bx, [rax]"),
                        4 => self.add_instruction("mov ebx, [rax]"),
                        8 => self.add_instruction("mov rbx, [rax]"),
                        _ => panic!("Unsupported load size"),
                    };
                    self.add_instruction("push rbx");
                }
                InstructionType::Store(size) => {
                    self.add_instruction("pop rbx");
                    self.add_instruction("pop rax");
                    match size {
                        1 => self.add_instruction("mov [rax], bl"),
                        2 => self.add_instruction("mov [rax], bx"),
                        4 => self.add_instruction("mov [rax], ebx"),
                        8 => self.add_instruction("mov [rax], rbx"),
                        _ => panic!("Unsupported store size"),
                    };
                }
                InstructionType::Syscall(arg_count) => {
                    let regs = ["rax", "rdi", "rsi", "rdx", "r10", "r9", "r8"];
                    for i in 0..*arg_count as usize {
                        self.add_instruction_string(format!("pop {}", regs[i]));
                    }
                    self.add_instruction("syscall");
                    self.add_instruction("push rax"); // Capture result
                }
                InstructionType::Identifier(identifier) => {
                    if let Some(inline) = self.program.inlines.get(identifier) {
                        if self.inline_expansion_stack.contains(identifier) {
                            todo!("Implement error for inline expansion stack")
                        }

                        self.inline_expansion_stack.push(identifier.clone());

                        self.compile_block(&inline.block.clone());

                        self.inline_expansion_stack.pop();
                    } else if self.program.memories.contains_key(identifier) {
                        self.add_instruction(format!("push {}", identifier).as_str());
                    } else {
                        // Function call
                        self.add_instruction_string(format!("push proc_{}", identifier));
                        self.add_instruction("call proc_interceptor");
                    }
                }
                InstructionType::Return => {
                    self.add_instruction("test r13, r13");
                    self.add_instruction("jz stack_underflow");
                    self.add_instruction("mov rdx, [ret_stack + r13 * 8]");
                    self.add_instruction("dec r13");
                    self.add_instruction("jmp rdx");
                }
            }
            self.cursor += 1;
        }
    }

    // --- Helper Functions ---

    fn add_instruction(&mut self, instruction: &str) {
        self.code.push_str(format!("\t{}\n", instruction).as_str());
    }

    fn add_instruction_string(&mut self, instruction: String) {
        self.add_instruction(instruction.as_str());
    }

    fn add_instruction_comment(&mut self, instruction: &Instruction) {
        self.code.push_str(format!("\t; --- {} ---\n", instruction.instruction_type).as_str());
    }

    fn add_label(&mut self, i: usize) {
        self.code.push_str(format!(".addr_{}:\n", i).as_str());
    }

    fn add_proc(&mut self, ident: String) {
        self.code.push_str(format!("\nproc_{}:\n", ident).as_str());
    }

    fn string_to_asm_data(&self, s: String) -> String {
        if s.is_empty() { return "db 0".to_string(); }
        let mut s2 = String::new();
        s.chars().for_each(|c| s2.push_str(format!(",0x{:x}", c as i32).as_str()));
        format!("db {}", &s2[1..])
    }
}