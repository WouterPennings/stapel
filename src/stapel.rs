use std::fmt::write;

use crate::Operators::InfixOperators;

#[derive(Debug, PartialEq, Clone)]
pub enum OpCodes {
    Push(i32),
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
    Empty,
}

impl std::fmt::Display for OpCodes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let value = match self {
            OpCodes::Push(_) => "Push {}",
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
            OpCodes::Empty => "Empty",
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
        let mut i = 0;
    
        while i < self.program.len() {
            let op = self.program[i].clone();
            self.add_opcode_comment(&op);
            match op {
                OpCodes::Push(int) => {
                    self.add_instruction_string(format!("push {}", int));
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
                        self.code.push_str(format!("    {} rax, rbx", op.to_x86_64_instruction()).as_str());
                        self.code.push_str("    push rax")
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
                OpCodes::Empty => {
                    self.add_instruction("mov esp, 0");
                }
            }
            i += 1
        }

        self.code.push_str("\t; === END OF PROGRAM ===");
        self.add_instruction("mov rax, 60");
        self.add_instruction("mov rdi, 0");
        self.add_instruction("syscall");
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
} 

pub fn compile_x86_64(ops: Vec<OpCodes>) -> String {
    let mut output = String::from(format!("{}\n", include_str!("start_asm_x86_64.asm")));
    let mut ops = ops;
    let mut i = 0;

    while i < ops.len() {
        let op = ops[i].clone();
        match op {
            OpCodes::Push(int) => {
                output.push_str(format!("    ; === PUSH {} ===\n", int).as_str());
                output.push_str(format!("    push {}\n", int).as_str());
            }
            OpCodes::Pop => {
                output.push_str("    ; === POP ===\n");
                output.push_str("    pop rax\n");
            }
            OpCodes::InfixOperators(op) => {
                output.push_str(format!("    ; === INFIX: {} ===\n", op).as_str());
                output.push_str("    pop rax\n");
                output.push_str("    pop rbx\n");

                if op == InfixOperators::Plus 
                    || op == InfixOperators::Minus 
                    || op == InfixOperators::Divide 
                    || op == InfixOperators::Multiply  
                {
                    output.push_str(format!("    {} rax, rbx\n", op.to_x86_64_instruction()).as_str());
                    output.push_str("    push rax\n")
                } else {
                    output.push_str("    mov rcx, 0\n");
                    output.push_str("    mov rdx, 1\n");
                    output.push_str("    cmp rax, rbx\n");
                    output.push_str(format!("    {} rcx, rdx\n", op.to_x86_64_instruction()).as_str());
                    output.push_str("    push rcx\n"); 
                }
            }
            OpCodes::Swap => {
                output.push_str("    ; === SWAP ===\n");
                output.push_str("    pop rax\n");
                output.push_str("    pop rbx\n");
                output.push_str("    push rax\n");
                output.push_str("    push rbx\n");
            }
            OpCodes::Put => {
                output.push_str("    ; === PUT ===\n");
                output.push_str("    pop rdi\n");
                output.push_str("    call print_i32\n");
            }
            OpCodes::Dup => {
                output.push_str("    ; === DUP ===\n");
                output.push_str("    pop rax\n");
                output.push_str("    push rax\n");
                output.push_str("    push rax\n");
            }
            OpCodes::If(jump) => {
                output.push_str("    ; === IF ===\n");
                output.push_str("    pop rax\n");
                output.push_str("    cmp rax, 0\n");
                output.push_str(format!("    je .addr_{}\n", jump).as_str())
            }
            OpCodes::Else(jump, _) => { 
                output.push_str("    ; === ELSE ===\n");
                output.push_str(format!("    jmp .addr_{}\n", jump).as_str());
                output.push_str(format!(".addr_{}:\n", i).as_str())
            },
            OpCodes::While => {
                output.push_str("    ; === WHILE ===\n");
                output.push_str(format!(".addr_{}:\n", i).as_str());
            }
            OpCodes::Do(jump) => {
                output.push_str("    ; === DO ===\n");
                output.push_str("    pop rax\n");
                output.push_str("    cmp rax, 0\n");
                output.push_str(format!("    je .addr_{}\n", jump).as_str())
            }
            OpCodes::End(is_jump, index) => {
                output.push_str("    ; === END ===\n");
                if is_jump {
                    output.push_str(format!("    jmp .addr_{}\n", index).as_str());
                }
                output.push_str(format!(".addr_{}:\n", i).as_str());
            }
            OpCodes::Empty => {
                output.push_str("    ; === EMPTY ===\n");
                output.push_str("    mov esp, 0\n");
            }
        }
        i += 1
    }

    output.push_str("    ; === END OF PROGRAM ===\n    mov rax, 60\n    mov rdi, 0\n    syscall");
    output
}

pub fn run(ops: Vec<OpCodes>) {
    let mut ops = ops;
    let mut stack: Vec<i32> = Vec::new();
    let mut i = 0;
    while i < ops.len() {
        let op = ops[i].clone();
        match op {
            OpCodes::Push(int) => {
                stack.push(int);
            }
            OpCodes::InfixOperators(op) => {
                let Some(first) = stack.pop() else {
                    panic!("EQUALS operant poped empty stack");
                };
                let Some(second) = stack.pop() else {
                    panic!("EQUALS operant poped empty stack");
                };

                match op {
                    InfixOperators::Plus => stack.push(second + first),
                    InfixOperators::Minus => stack.push(first - second),
                    InfixOperators::Multiply => stack.push(first * second),
                    InfixOperators::Divide => stack.push((first / second) as i32),
                    InfixOperators::Equals => stack.push((first == second) as i32),
                    InfixOperators::NotEquals => stack.push((first != second) as i32),
                    InfixOperators::GreaterThan => stack.push((first > second) as i32),
                    InfixOperators::LesserThan => stack.push((first < second) as i32),
                    InfixOperators::GreaterOrEqualsTo => stack.push((first >= second) as i32),
                    InfixOperators::LesserOrEqualsTo => stack.push((first <= second) as i32),
                }
            }
            OpCodes::Pop => {
                let Some(_) = stack.pop() else {
                    panic!("POP operant poped empty stack");
                };
            }
            OpCodes::Swap => {
                let Some(first) = stack.pop() else {
                    panic!("SWAP operant poped empty stack");
                };
                let Some(second) = stack.pop() else {
                    panic!("SWAP operant poped empty stack");
                };

                stack.push(first);
                stack.push(second);
            }
            OpCodes::Put => {
                let Some(value) = stack.pop() else {
                    panic!("PUT operant poped stack, but stack was empty");
                };
                
                println!("{}", value);
            }
            OpCodes::If(jump) => {
                let Some(a) = stack.pop() else {
                    panic!("IF operant poped stack, but stack was empty");
                };
                
                if a == 0 {
                    i = jump-1;
                } else if let OpCodes::Else(j, _) = ops[jump] {
                    ops[jump] = OpCodes::Else(j, false);
                }
            }
            OpCodes::Else(jump, do_else)  => { 
                if !do_else {
                    i = jump-1;
                }
            },
            OpCodes::While => { }
            OpCodes::Do(jump) => {
                let Some(a) = stack.pop() else {
                    panic!("While-Do operant poped stack, but stack was empty");
                };

                if a == 0 {
                    ops[jump] = OpCodes::End(false, 0);
                    i = jump-1;
                }
            }
            OpCodes::End(is_jump, index) => {
                if is_jump {
                    i = index-1;
                }
            }
            OpCodes::Dup => {
                let Some(a) = stack.pop() else {
                    panic!("DUP operant poped stack, but stack was empty");
                };

                stack.push(a);
                stack.push(a);
            }
            OpCodes::Empty => {
                stack = Vec::new();
            }
        }
        i += 1
    }
}

pub fn parse(input: String) -> Vec<OpCodes> {
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
            "empty" => ops.push(OpCodes::Empty),
            _ => {
                match word.parse::<i32>() {
                    Ok(int) => {
                        ops.push(OpCodes::Push(int));
                    },
                    Err(_) => {
                        panic!("{}, Is not an interger", word);
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