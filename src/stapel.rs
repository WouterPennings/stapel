use crate::operators::InfixOperators;

#[derive(Debug, PartialEq, Clone)]
pub enum OpCodes {
    PushInt(i32),
    PushStr(String),
    InfixOperators(InfixOperators),
    Pop,
    Swap,
    Put,
    While,
    If(usize),
    Else(usize, bool),
    Do(usize),
    End(bool, usize),
    Dup,
    Size,
    Mem,
    Load(usize),
    Store(usize),
    Syscall(u8),
}

impl std::fmt::Display for OpCodes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let value = match self {
            OpCodes::PushInt(_) => "PushInt",
            OpCodes::PushStr(_) => "PushStr",
            OpCodes::InfixOperators(_) => "InfixOperators",
            OpCodes::Pop => "Pop",
            OpCodes::Swap => "Swap",
            OpCodes::Put => "Put",
            OpCodes::While => "While",
            OpCodes::If(_) => "If",
            OpCodes::Else(_, _) => "Else",
            OpCodes::Do(_) => "Do",
            OpCodes::End(_, _) => "End",
            OpCodes::Dup => "Dup",
            OpCodes::Size => "Size",
            OpCodes::Mem => "Mem",
            OpCodes::Load(_) => "Load",
            OpCodes::Store(_) => "Store",
            OpCodes::Syscall(_) => "Syscall"
        };

        write!(f, "{}", value)
    }
}

pub struct Compiler {
    program: Vec<OpCodes>,
    pub code: String,
}

impl Compiler {
    pub fn new(program: Vec<OpCodes>) -> Compiler {
        Compiler { program, code: String::from(format!("{}\n", include_str!("start_asm_x86_64.asm"))) }
    }

    pub fn compile_x86_64(&mut self) {
        // self.program.push(OpCodes::PushStr("Hello, World!\n".to_string()));
        // self.program.push(OpCodes::PushInt(1));
        // self.program.push(OpCodes::PushInt(1));
        // self.program.push(OpCodes::Syscall(52));
        let mut strings: Vec<String> = Vec::new();
        let mut i = 0;
    
        while i < self.program.len() {
            let op = self.program[i].clone();
            self.add_opcode_comment(&op);
            match op {
                OpCodes::PushInt(int) => {
                    self.add_instruction_string(format!("push {}", int));
                }
                OpCodes::PushStr(str) => {
                    // NOTE: C-style string, fixed size. 
                    self.add_instruction_string(format!("push {}", str.len()));
                    self.add_instruction_string(format!("push str_{}", strings.len()));
                    strings.push(str);
                }
                OpCodes::Pop => {
                    self.add_instruction("pop rax");
                }
                OpCodes::InfixOperators(op) => {
                    self.add_instruction("pop rax");
                    self.add_instruction("pop rbx");
    
                    if op == InfixOperators::Plus 
                        || op == InfixOperators::Minus 
                        || op == InfixOperators::Divide 
                        || op == InfixOperators::Multiply  
                    {
                        self.add_instruction_string(format!("{} rax, rbx", op.to_x86_64_instruction()));
                        self.add_instruction("push rax");
                    } else {
                        self.add_instruction("mov rcx, 0");
                        self.add_instruction("mov rdx, 1");
                        self.add_instruction("cmp rax, rbx");
                        self.add_instruction_string(format!("{} rcx, rdx", op.to_x86_64_instruction()));
                        self.add_instruction("push rcx"); 
                    }
                }
                OpCodes::Swap => {
                    self.add_instruction("pop rax");
                    self.add_instruction("pop rbx");
                    self.add_instruction("push rax");
                    self.add_instruction("push rbx");
                }
                OpCodes::Put => {
                    self.add_instruction("pop rdi");
                    self.add_instruction("call print_i32");
                }
                OpCodes::Dup => {
                    self.add_instruction("pop rax");
                    self.add_instruction("push rax");
                    self.add_instruction("push rax");
                }
                OpCodes::If(jump) => {
                    self.add_instruction("pop rax");
                    self.add_instruction("cmp rax, 0");
                    self.add_instruction_string(format!("je .addr_{}", jump));
                }
                OpCodes::Else(jump, _) => { 
                    self.add_instruction_string(format!("jmp .addr_{}\n", jump));
                    self.add_label(i);
                },
                OpCodes::While => {
                    self.add_label(i);
                }
                OpCodes::Do(jump) => {
                    self.add_instruction("pop rax");
                    self.add_instruction("cmp rax, 0");
                    self.add_instruction_string(format!("je .addr_{}", jump));
                }
                OpCodes::End(is_jump, index) => {
                    if is_jump {
                        self.add_instruction_string(format!("jmp .addr_{}", index));
                    }
                    self.add_label(i);
                }
                OpCodes::Size => {
                    self.add_instruction("mov rax, rsp");
                    self.add_instruction("sub rax, [ori_stack_ptr]");
                    self.add_instruction("neg rax");
                    self.add_instruction("add rax, 8");
                    self.add_instruction("push rax");
                }
                OpCodes::Mem => {
                    self.add_instruction("push mem");
                },
                OpCodes::Load(size) => {
                    self.add_instruction("pop rax        ; Pointer to memory");    // pointer to memory
                    self.add_instruction("mov rbx, 0");
                    self.add_instruction_string(format!("mov bl, [rax*{}]", (size/8) as u8));
                    self.add_instruction("push rbx");
                },
                OpCodes::Store(size) => {
                    self.add_instruction("pop rbx        ; Value to store"); 
                    self.add_instruction("pop rax        ; Pointer to memory");   
                    self.add_instruction_string(format!("mov [rax*{}], bl", (size/8) as u8));
                },
                OpCodes::Syscall(arg_count) => {
                    let mut reg_priority = ["rax", "rdi", "rsi", "rdx", "r10", "r8", "r9"].iter();
                    for _ in 0..arg_count as usize {
                        let reg = *reg_priority.next().unwrap();
                        self.add_instruction_string(format!("pop {}", reg));
                    }
                    self.add_instruction("syscall");
                }
            }
            i += 1
        }

        self.code.push_str("\t; === END OF PROGRAM ===\n");
        self.add_instruction("mov rax, 60");
        self.add_instruction("mov rdi, 0");
        self.add_instruction("syscall");

        self.code.push_str("section .bss\n");
        self.code.push_str("mem: resq 1024\n");
        self.code.push_str("section .data\n");
        self.code.push_str("ori_stack_ptr: dd 0\n");
        for (i, str) in strings.iter().enumerate() {
            self.code.push_str(format!("str_{}: {}", i, self.string_to_asm_data(str.clone())).as_str());
        }
    }

    fn add_opcode_comment(&mut self, opcode: &OpCodes) {
        self.code.push_str(format!("\t; === {} ===\n", opcode).as_str());
    }

    fn add_instruction_string(&mut self, instruction: String) {
        self.add_instruction(instruction.as_str());
    }

    fn add_instruction(&mut self, instruction: &str) {
        self.code.push_str(format!("\t{}\n", instruction).as_str());
    }

    fn add_label(&mut self, i: usize) {
        self.code.push_str(format!(".addr_{}\n", i).as_str());
    }

    fn string_to_asm_data(&self, s: String) -> String {
        let mut s2 = String::new();
        s.chars().for_each(|c| s2.push_str(format!(",0x{:x}", c as i32).as_str()));
        format!("db {}", &s2[1..])
    }
} 

pub fn parse(mut input: String) -> Vec<OpCodes> {
    // TODO: Allow for spaces in strings
    // TODO: Error location reporting
    remove_comments(&mut input);

    let data = input.replace("\n", " ");
    let program: Vec<String> = data.split(" ")
        .filter(|word| !word.is_empty())
        .map(|s| s.to_string())
        .collect();
    
    let mut ops: Vec<OpCodes> = Vec::new();
    for word in program {
        match word.as_str() {
            "+" | "-" | "*" | "/" | "%" | "=" | "!=" | "<" | ">" | ">=" | "<=" => {
                ops.push(OpCodes::InfixOperators(InfixOperators::new(word)))
            }
            "pop" => ops.push(OpCodes::Pop),
            "swap" => ops.push(OpCodes::Swap),
            "put" => ops.push(OpCodes::Put),
            "if" => ops.push(OpCodes::If(0)),
            "else" => ops.push(OpCodes::Else(0, false)),
            "end" => ops.push(OpCodes::End(false, 0)),
            "do" => ops.push(OpCodes::Do(0)),
            "while" => ops.push(OpCodes::While),
            "dup" => ops.push(OpCodes::Dup),
            "size" => ops.push(OpCodes::Size),
            "mem" => ops.push(OpCodes::Mem),
            "@8" | "@16" | "@32" => {
                let size = &word[1..];
                match size.parse::<usize>() {
                    Ok(int) => {
                        ops.push(OpCodes::Store(int));
                    },
                    Err(_) => {
                        panic!("{}, Is not an interger", word);
                    }
                }
            },
            "!8" | "!16" | "!32" => {
                let size = &word[1..];
                match size.parse::<usize>() {
                    Ok(int) => {
                        ops.push(OpCodes::Load(int));
                    },
                    Err(_) => {
                        panic!("{}, Is not an interger", word);
                    }
                }
            }
            _ => {
                if word.starts_with("\"") && word.ends_with("\"") {
                    let mut word = word.replace("\\n", "\n");
                    word = word.replace("\\t", "\t");
                    word = word.replace("\\\"", "\"");
                    word = word.replace("\\'", "'");
                    ops.push(OpCodes::PushStr(String::from(&word[1..(word.len()-1) as usize])));
                } else if word.starts_with("syscall") 
                    && word.len() == 8 
                    && word.chars().last().unwrap() as u8 >= 48 
                    && word.chars().last().unwrap() as u8 <= 54 {
                    ops.push(OpCodes::Syscall(word.chars().last().unwrap() as u8 - 48));
                } else {
                    match word.parse::<i32>() {
                        Ok(int) => {
                            ops.push(OpCodes::PushInt(int));
                        },
                        Err(_) => {
                            panic!("{}, Is not an interger", word);
                        }
                    }
                }
            }
        }
    }
    crossreference_blocks(ops)
}

pub fn crossreference_blocks(mut program: Vec<OpCodes>) -> Vec<OpCodes> {
    let mut stack: Vec<(OpCodes, usize)> = Vec::new();
    for i in 0..program.len() {
        match program[i] {
            OpCodes::If(_) => {
                stack.push((OpCodes::If(0), i));
            }
            OpCodes::While => {
                stack.push((OpCodes::While, i));
            }
            OpCodes::Else(_, _) => {
                let if_i = stack.pop();
                assert!(if_i.is_some(), "'else' is not closing an if block"); 
                program[if_i.unwrap().1] = OpCodes::If(i);
                stack.push((OpCodes::Else(0, true), i));
            }
            OpCodes::Do(_) => {
                let while_i = stack.pop();
                assert!(while_i.is_some(), "While does not exist before end"); 
                assert!(while_i.clone().unwrap().0 == OpCodes::While, "Do operation only works on 'while', not {:?}", while_i.unwrap().0);
                stack.push((OpCodes::Do(while_i.unwrap().1), i));
            }
            OpCodes::End(_, _) => {
                let if_i = stack.pop();
                assert!(if_i.is_some(), "If or Do does not exist before end"); 
                match if_i.clone().unwrap().0 {
                    OpCodes::Do(while_i) => {
                        program[if_i.unwrap().1] = OpCodes::Do(i);
                        program[i] = OpCodes::End(true, while_i);
                    }
                    OpCodes::If(_) => {
                        program[if_i.unwrap().1] = OpCodes::If(i);
                    }
                    OpCodes::Else(_, _) => {
                        program[if_i.unwrap().1] = OpCodes::Else(i, true);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }   

    program
}

fn remove_comments(input: &mut String){
    let mut b = false;
    for (i, c) in input.clone().chars().enumerate() {
        if c == '#' {
            b = true;
        } else if c == '\n' {
            b = false;
        } 
        
        if b {
            input.replace_range(i..i+1, " ");
        }
    }
}