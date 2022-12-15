use crate::operators::InfixOperators;

#[derive(Debug, PartialEq, Clone)]
pub struct Span {
    pub file: String,
    pub row: usize,
    pub column: usize,
}

impl Span {
    pub fn new(file: String, row: usize, column: usize) -> Span {
        Span { file, row, column }
    }
}

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
    Custom(String),
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
            OpCodes::Syscall(_) => "Syscall",
            OpCodes::Custom(_) => "Custom",
        };

        write!(f, "{}", value)
    }
}

pub struct Compiler {
    program: Vec<(OpCodes, Span)>,
    pub code: String,
}

impl Compiler {
    pub fn new(program: Vec<(OpCodes, Span)>) -> Compiler {
        Compiler { program, code: String::from(format!("{}\n", include_str!("start_asm_x86_64.asm"))) }
    }

    pub fn compile_x86_64(&mut self) {
        let mut strings: Vec<String> = Vec::new();
        let mut i = 0;
    
        while i < self.program.len() {
            let op = self.program[i].clone();
            self.add_opcode_comment(&op.0);
            match op.0 {
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
                OpCodes::Custom(_) => {

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

pub struct Parser {
    pub input: Vec<char>,
    pub file_name: String,
    pub ops: Vec<(OpCodes, Span)>,
    cursor: usize,
    current_char: Option<char>,
    peek_char: Option<char>,
    row: usize, 
    column: usize,
}

impl Parser {
    pub fn new(input: String, file_name: String) -> Parser {
        let chars: Vec<char> = input.chars().map(|c| c).collect();
        Parser { 
            input: chars, 
            file_name, 
            ops: Vec::new(), 
            cursor: 0, 
            current_char: input.chars().nth(0), 
            peek_char: input.chars().nth(1),
            row: 1,
            column: 1,
        }
    }


    pub fn parse(&mut self) {
        while self.current_char.is_some() {
            let c = self.current_char;
            if c.is_none() || c.unwrap() == ' ' || c.unwrap() == '\n' {
                self.next_character();
                continue;
            }
            
            let c = c.unwrap();
            let row = self.row;
            let col = self.column;
            let span = Span::new(self.file_name.clone(), row, col);
            match c {
                '"' => {
                    self.next_character();
                    let mut str = String::new();
                    while self.current_char.unwrap() != '"' {
                        str.push(self.current_char.unwrap());
                        self.next_character();
                    }
                    self.ops.push((OpCodes::PushStr(str), span))
                }
                ';' => {
                    while self.next_character().unwrap() != '\n' {}
                }
                '!' => {
                    self.next_character();
                    if self.current_char.is_some() && self.current_char.unwrap() == '=' {
                        self.next_character();
                        self.ops.push((OpCodes::InfixOperators(InfixOperators::new("!=".to_string())), span));
                    } else {
                        self.next_character();
                        let num = self.parse_num();
                        if num == 8 {
                            self.ops.push((OpCodes::Load(num as usize), Span::new(self.file_name.clone(), row, col)));
                        } else {
                            self.throw_exception(span, format!("'{}', is not a support bit amount", num));
                            unreachable!()
                        }
                    }
                }
                '@' => {
                    self.next_character();
                    let num = self.parse_num();
                    if num == 8 {
                        self.ops.push((OpCodes::Store(num as usize), span));
                    } else {
                        self.throw_exception(span, format!("'{}', is not a support bit amount", num));
                        unreachable!()
                    }
                }
                '=' => {
                    let op = InfixOperators::new(String::from(c));
                        self.ops.push((OpCodes::InfixOperators(op), span));
                }
                '+' | '-' | '*' | '/' | '%' => {
                    if c == '-' && self.peek_char.unwrap().is_numeric() {
                        let num = self.parse_num();
                        self.ops.push((OpCodes::PushInt(num), span));
                    } else {
                        let op = InfixOperators::new(String::from(c));
                        self.ops.push((OpCodes::InfixOperators(op), span));
                    }
                }
                '<' | '>' => {
                    if self.peek_char.is_some() && self.peek_char.unwrap() == '=' {
                        self.next_character();
                        let op = InfixOperators::new(format!("{}=", c).to_string());
                        self.ops.push((OpCodes::InfixOperators(op), span));
                    } else {
                        let op = InfixOperators::new(String::from(c));
                        self.ops.push((OpCodes::InfixOperators(op), span));
                    }
                }
                _ => {
                    if c.is_numeric() {
                        let num = self.parse_num();
                        self.ops.push((OpCodes::PushInt(num), span));
                    } else {
                        self.parse_word();
                    }
                }
            }
            self.next_character();
        }
        self.crossreference_blocks();
    }

    fn parse_word(&mut self) {
        let row = self.row;
        let col = self.column;
        let mut str = String::new();
        while self.current_char.is_some() && self.current_char.unwrap() != ' ' && self.current_char.unwrap() != '\n' {
            str.push(self.current_char.unwrap());
            self.next_character();
        }

        let span = Span::new(self.file_name.clone(), row, col);
        match str.as_str() {
            "pop" => self.ops.push((OpCodes::Pop, span)),
            "swap" => self.ops.push((OpCodes::Swap, span)),
            "put" => self.ops.push((OpCodes::Put, span)),
            "if" => self.ops.push((OpCodes::If(0), span)),
            "else" => self.ops.push((OpCodes::Else(0, false), span)),
            "end" => self.ops.push((OpCodes::End(false, 0), span)),
            "do" => self.ops.push((OpCodes::Do(0), span)),
            "while" => self.ops.push((OpCodes::While, span)),
            "dup" => self.ops.push((OpCodes::Dup, span)),
            "size" => self.ops.push((OpCodes::Size, span)),
            "mem" => self.ops.push((OpCodes::Mem, span)),
            _ => {
                if str.starts_with("syscall")
                    && str.len() == 8 
                    && str.chars().last().unwrap() as u8 >= 48 
                    && str.chars().last().unwrap() as u8 <= 54 {
                    self.ops.push((OpCodes::Syscall(str.chars().last().unwrap() as u8 - 48), span));
                } else {
                    self.ops.push((OpCodes::Custom(str), span));
                }
            }
        } 
    }

    fn parse_num(&mut self) -> i32 {
        let mut num = String::from(self.current_char.unwrap());
        while self.next_character().is_some() && self.current_char.unwrap().is_numeric() {
            num.push(self.current_char.unwrap());
        }
        match num.parse::<i32>() {
            Ok(i) => {
                return i;
            }
            Err(_) => {
                let row = self.row;
                let col = self.column;
                let span = Span::new(self.file_name.clone(), row, col);
                self.throw_exception(span, format!("'{}', is not an i32", num));
                unreachable!();
            }
        }
    }

    pub fn crossreference_blocks(&mut self) {
        let mut stack: Vec<(OpCodes, usize)> = Vec::new();
        for i in 0..self.ops.len() {
            match self.ops[i].0 {
                OpCodes::If(_) => {
                    stack.push((OpCodes::If(0), i));
                }
                OpCodes::While => {
                    stack.push((OpCodes::While, i));
                }
                OpCodes::Else(_, _) => {
                    let if_i = stack.pop();
                    if if_i.is_none() {
                        self.throw_exception(self.ops[i].1.clone(), "'else' is not closing an if block".to_string());
                    }
                    self.ops[if_i.unwrap().1] = (OpCodes::If(i), self.ops[i].clone().1);
                    stack.push((OpCodes::Else(0, true), i));
                }
                OpCodes::Do(_) => {
                    let while_i = stack.pop();
                    if while_i.is_none() {
                        self.throw_exception(self.ops[i].1.clone(), "'while' does not exist before end".to_string());
                    }
                    if while_i.clone().unwrap().0 != OpCodes::While {
                        self.throw_exception(self.ops[i].1.clone(), format!("Do operation only works on 'while', not {:?}", while_i.clone().unwrap().0));
                    }
                    stack.push((OpCodes::Do(while_i.unwrap().1), i));
                }
                OpCodes::End(_, _) => {
                    let if_i = stack.pop();
                    if if_i.is_none() {
                        self.throw_exception(self.ops[i].1.clone(), "'if' or 'do' does not exist before end".to_string());
                    }
                    match if_i.clone().unwrap().0 {
                        OpCodes::Do(while_i) => {
                            self.ops[if_i.unwrap().1] = (OpCodes::Do(i), self.ops[i].clone().1);
                            self.ops[i] = (OpCodes::End(true, while_i), self.ops[i].clone().1);
                        }
                        OpCodes::If(_) => {
                            self.ops[if_i.unwrap().1] = (OpCodes::If(i), self.ops[i].clone().1);
                        }
                        OpCodes::Else(_, _) => {
                            self.ops[if_i.unwrap().1] = (OpCodes::Else(i, true), self.ops[i].clone().1);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }   
    }

    fn next_character(&mut self) -> Option<char> {
        self.current_char = self.peek_char;
        self.cursor += 1;
        if self.cursor < self.input.len() {
            self.current_char = Some(self.input[self.cursor]);
        } else {
            self.current_char = None;
        }

        if self.current_char.unwrap_or('_') == '\n' {
            self.row += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }

        self.current_char
    }

    fn throw_exception(&mut self, span: Span, message: String) {
        println!("{} [{}:{}] ->\n\t{}", span.file, span.row, span.column, message);
        std::process::exit(1);
    }
}
