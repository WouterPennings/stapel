use uuid::Uuid;

use crate::operators::InfixOperators;
use crate::parser::{Block, Instruction, InstructionType, PushType};
use crate::program::Program;

pub struct Compiler {
    pub code: String,
    cursor: usize,
    program: Program,
    strings: Vec<(String, String)>,
}

impl Compiler {
    pub fn new(program: Program) -> Compiler {
        Compiler {
            program,
            cursor: 0,
            strings: Vec::new(),
            code: String::from(format!("{}\n", include_str!("start_asm_x86_64.asm"))),
        }
    }

    pub fn compile_x86_64(&mut self) {
        for procedure in self.program.procedures.clone() {
            self.add_proc(procedure.identifier);

            self.compile_block(procedure.block);
        }

        self.code.push_str("\t; === END OF PROGRAM (ADDED DURING COMPILATION) ===\n");
        self.add_instruction("mov rax, 60");
        self.add_instruction("mov rdi, 0");
        self.add_instruction("syscall\n");

        self.code.push_str("section .bss\n");
        self.code.push_str("mem: resq 1024 ; Pointer to start of memory\n");
        self.code.push_str("\n");

        self.code.push_str("section .data\n");
        self.code.push_str("ori_stack_ptr: dd 0 ; Pointer to start of stack\n");
        self.code.push_str("ret_stack: TIMES 1024 DQ 0; Stack for the return adresses\n");
        self.code.push_str("ret_stack_cursor: DQ 0; Pointer to start of memory\n");
        self.code.push_str("; Strings defined by user in the program\n");
        for (i, (str, original)) in self.strings.iter().enumerate() {
            self.code
                .push_str(format!("str_{}: {} ; \"{}\"", i, self.string_to_asm_data(str.clone()), original).as_str());
        }
    }

    fn compile_block(&mut self, block: Block) {
        for instruction in block.instructions {
            // self.add_label(self.cursor);
            self.add_instruction_comment(&instruction);

            match instruction.instruction_type {
                InstructionType::Push(PushType::Int(i)) => {
                    self.add_instruction_string(format!("push {}", i));
                }
                InstructionType::Push(PushType::Str(str, original)) => {
                    // NOTE: C-style string, fixed size.
                    self.add_instruction_string(format!("push {}", str.len()));
                    self.add_instruction_string(format!("push str_{}", self.strings.len()));
                    self.strings.push((str, original));
                }
                InstructionType::InfixOperators(op) => {
                    self.add_instruction("pop rax");
                    self.add_instruction("pop rbx");

                    match op {
                        InfixOperators::Plus | InfixOperators::Minus  => {
                            self.add_instruction_string(format!("{} rax, rbx", op.to_x86_64_instruction()));
                            self.add_instruction("push rax");
                        }
                        InfixOperators::Multiply | InfixOperators::Divide => {
                            self.add_instruction_string(format!("{} rbx", op.to_x86_64_instruction()));
                            self.add_instruction("push rax");
                        }
                        InfixOperators::Modulo => {
                            self.add_instruction_string(format!("{} rbx", op.to_x86_64_instruction()));
                            self.add_instruction("push rdx");
                        }
                        _ => {
                            self.add_instruction("mov rcx, 0");
                            self.add_instruction("mov rdx, 1");
                            self.add_instruction("cmp rax, rbx");
                            self.add_instruction_string(format!("{} rcx, rdx", op.to_x86_64_instruction()));
                            self.add_instruction("push rcx");
                        }
                    };
                }
                InstructionType::While(whl) => {
                    let start_cond = whl.condition.instructions[0].id;
                    let end = whl.block.instructions.last().unwrap().id;

                    self.add_label_uuid(start_cond);
                    self.compile_block(whl.condition); // Compiling the conditional block

                    self.add_instruction("pop rax");
                    self.add_instruction("cmp rax, 0");
                    self.add_instruction_string(format!("je .addr_{}", end.as_u128()));

                    self.compile_block(whl.block); // Compiling the conditional block
                    self.add_instruction_string(format!("jmp .addr_{}", start_cond.as_u128()));
                    self.add_label_uuid(end);
                }
                InstructionType::If(iff) => {
                    self.add_instruction("pop rax");
                    self.add_instruction("cmp rax, 0");

                    if iff.else_block.is_some() {
                        let else_start = iff.else_block.clone().unwrap().instructions[0].id;
                        let end_end = iff.else_block.clone().unwrap().instructions.last().unwrap().id;

                        self.add_instruction_string(format!("je .addr_{}", else_start.as_u128()));
                        self.compile_block(iff.if_block);

                        self.add_instruction_string(format!("jmp .addr_{}", end_end.as_u128()));
                        self.add_label_uuid(else_start);
                        self.compile_block(iff.else_block.unwrap());
                        self.add_label_uuid(end_end);
                    } else {
                        let end = iff.if_block.instructions[0].id;

                        self.add_instruction_string(format!("je .addr_{}", end.as_u128()));
                        self.compile_block(iff.if_block);
                        self.add_label_uuid(end);
                    }
                }
                InstructionType::Pop => {
                    self.add_instruction("pop rax");
                }
                InstructionType::Swap => {
                    self.add_instruction("pop rax");
                    self.add_instruction("pop rbx");
                    self.add_instruction("push rax");
                    self.add_instruction("push rbx");
                }
                InstructionType::Put => {
                    self.add_instruction("pop rdi");
                    self.add_instruction("call print_i32");
                }
                InstructionType::Dup => {
                    self.add_instruction("pop rax");
                    self.add_instruction("push rax");
                    self.add_instruction("push rax");
                }
                InstructionType::Size => {
                    self.add_instruction("mov rax, rsp");
                    self.add_instruction("sub rax, [ori_stack_ptr]");
                    self.add_instruction("neg rax");
                    self.add_instruction("add rax, 8");
                    self.add_instruction("push rax");
                }
                InstructionType::Mem => {
                    self.add_instruction("push mem");
                }
                InstructionType::Load(size) => {
                    self.add_instruction("pop rax        ; Pointer to memory"); // pointer to memory
                    self.add_instruction("mov rbx, 0");
                    self.add_instruction_string(format!("mov bl, [rax*{}]", (size / 8) as u8));
                    self.add_instruction("push rbx");
                }
                InstructionType::Store(size) => {
                    self.add_instruction("pop rbx        ; Value to store");
                    self.add_instruction("pop rax        ; Pointer to memory");
                    self.add_instruction_string(format!("mov [rax*{}], bl", (size / 8) as u8));
                }
                InstructionType::Syscall(arg_count) => {
                    let mut reg_priority = ["rax", "rdi", "rsi", "rdx", "r10", "r8", "r9"].iter();
                    for _ in 0..arg_count as usize {
                        let reg = *reg_priority.next().unwrap();
                        self.add_instruction_string(format!("pop {}", reg));
                    }
                    self.add_instruction("syscall");
                }
                InstructionType::Procedure(proc) => panic!(
                    "Procedure named: \"{}\", cannot define an procedure inside another procedure",
                    proc.identifier
                ),
                InstructionType::CustomInstruction(identifier) => {
                    self.add_instruction_string(format!(
                        "push .proc_{} ; Pushing the label address of the pointer to the stack",
                        identifier
                    ));
                    self.add_instruction("call proc_interceptor");
                }
                InstructionType::Return => {
                    self.add_instruction("mov rax, [ret_stack_cursor]");
                    self.add_instruction("mov rdx, [ret_stack+rax*8]");
                    self.add_instruction("dec rax");
                    self.add_instruction("mov [ret_stack_cursor], rax");
                    self.add_instruction("jmp rdx");
                }
            }
            self.cursor += 1;
        }
        self.add_comment("=== end ===")
    }

    fn add_comment(&mut self, comment: &str) {
        self.code.push_str(format!("\t; {}\n", comment).as_str());
    }

    fn add_comment_string(&mut self, comment: String) {
        self.code.push_str(format!("\t; {}\n", comment).as_str());
    }

    fn add_instruction_comment(&mut self, instruction: &Instruction) {
        self.code.push_str(format!("\t; === {} === ({})\n", instruction.instruction_type, instruction.id).as_str());
    }

    fn add_instruction_string(&mut self, instruction: String) {
        self.add_instruction(instruction.as_str());
    }

    fn add_instruction(&mut self, instruction: &str) {
        self.code.push_str(format!("\t{}\n", instruction).as_str());
    }

    fn add_label(&mut self, i: usize) {
        self.code.push_str(format!(".addr_{}:\n", i).as_str());
    }

    fn add_label_uuid(&mut self, id: Uuid) {
        self.code.push_str(format!(".addr_{}:\n", id.as_u128()).as_str());
    }

    fn add_proc(&mut self, ident: String) {
        self.code.push_str(format!("\n\t; === proc {} do ===\n", ident).as_str());
        self.code.push_str(format!(".proc_{}:\n", ident).as_str());
    }

    fn string_to_asm_data(&self, s: String) -> String {
        let mut s2 = String::new();
        s.chars().for_each(|c| s2.push_str(format!(",0x{:x}", c as i32).as_str()));
        format!("db {}", &s2[1..])
    }
}
