use crate::operators::InfixOperators;

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
    Size,
    Mem,
    Load(usize),
    Store(usize),
}

impl std::fmt::Display for OpCodes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let value = match self {
            OpCodes::Push(_) => "Push",
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
                    self.add_instruction("pop rax");    // pointer to memory
                    self.add_instruction("mov rbx, 0");
                    self.add_instruction_string(format!("mov bl, [rax*{}]", (size/8) as u8));
                    self.add_instruction("push rbx");
                },
                OpCodes::Store(size) => {
                    self.add_instruction("pop rbx");    // Value to store
                    self.add_instruction("pop rax");    // pointer to memory
                    self.add_instruction_string(format!("mov [rax*{}], bl", (size/8) as u8));
                },
            }
            i += 1
        }

        self.code.push_str("\t; === END OF PROGRAM ===\n");
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
            OpCodes::Size => {
                stack.push((stack.len()+1) as i32);
            }
            OpCodes::Mem => todo!(),
            OpCodes::Load(_) => todo!(),
            OpCodes::Store(_) => todo!(),
        }
        i += 1
    }
}

pub fn parse(mut input: String) -> Vec<OpCodes> {
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