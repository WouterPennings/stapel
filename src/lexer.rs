use crate::operators::{InfixOperators};
use crate::throw_exception_span;
use crate::tokens::{Span, Token, TokenType};

#[derive(Debug, PartialEq, Clone)]
pub struct Lexer {
    pub input_chars: Vec<char>,
    pub file_name: String,
    pub tokens: Vec<Token>,
    cursor: usize,
    current_char: Option<char>,
    peek_char: Option<char>,
    row: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String, file_name: String) -> Lexer {
        // let mut base: Vec<char> = include_str!("../std/std.spl").chars().collect();
        let input: Vec<char> = input.chars().collect();
        // base.append(&mut input);

        Lexer {
            current_char: input.get(0).copied(),
            peek_char: input.get(1).copied(),
            input_chars: input,
            file_name,
            tokens: Vec::new(),
            cursor: 0,
            row: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) {
        while self.current_char.is_some() {
            let c = self.current_char.unwrap();

            // Skip whitespace
            if c.is_whitespace() {
                self.next_character();
                continue;
            }

            let row = self.row;
            let col = self.column;
            let span = Span::new(self.file_name.clone(), row, col);

            match c {
                '"' => {
                    self.next_character();
                    let mut raw_str = String::new();
                    while self.current_char.is_some() && self.current_char.unwrap() != '"' {
                        raw_str.push(self.current_char.unwrap());
                        self.next_character();
                    }
                    let filtered = self.filter_escape_sequences(raw_str.clone());
                    self.tokens.push(Token::new(TokenType::PushStr(filtered, raw_str), span));
                    self.next_character();
                }
                '\'' => {
                    let ascii_value = self.parse_char_literal();
                    self.tokens.push(Token::new(TokenType::PushInt(ascii_value), span));
                    // parse_char_literal lands on the closing ', so we move past it
                    self.next_character();
                }
                ';' => {
                    // Line comment
                    while self.next_character().is_some() && self.current_char.unwrap() != '\n' {}
                }
                '!' => {
                    self.next_character();
                    if self.current_char == Some('=') {
                        self.tokens.push(Token::new(TokenType::InfixOperators(InfixOperators::new("!=".to_string())), span));
                        self.next_character();
                    } else {
                        // Logic for !8, !1 etc (Load)
                        let num = self.parse_num();
                        if [1, 2, 4, 8].contains(&num) {
                            self.tokens.push(Token::new(TokenType::Load(num as usize), span));
                        } else {
                            throw_exception_span(&span, format!("'{}' is not a supported bit amount", num));
                        }
                        // parse_num already moved cursor to next non-digit
                    }
                }
                '@' => {
                    // Logic for @8, @1 etc (Store)
                    self.next_character();
                    let num = self.parse_num();
                    if [1, 2, 4, 8].contains(&num) {
                        self.tokens.push(Token::new(TokenType::Store(num as usize), span));
                    } else {
                        throw_exception_span(&span, format!("'{}' is not a supported bit amount", num));
                    }
                }
                '=' => {
                    self.tokens.push(Token::new(TokenType::InfixOperators(InfixOperators::new(c.to_string())), span));
                    self.next_character();
                }
                '+' | '-' | '*' | '/' | '%' => {
                    if c == '-' && self.peek_char.map_or(false, |p| p.is_numeric()) {
                        let num = self.parse_num();
                        self.tokens.push(Token::new(TokenType::PushInt(num), span));
                    } else {
                        self.tokens.push(Token::new(TokenType::InfixOperators(InfixOperators::new(c.to_string())), span));
                        self.next_character();
                    }
                }
                '<' | '>' => {
                    if self.peek_char == Some('=') {
                        let mut op_str = c.to_string();
                        op_str.push('=');
                        self.tokens.push(Token::new(TokenType::InfixOperators(InfixOperators::new(op_str)), span));
                        self.next_character();
                        self.next_character();
                    } else {
                        self.tokens.push(Token::new(TokenType::InfixOperators(InfixOperators::new(c.to_string())), span));
                        self.next_character();
                    }
                }
                '#' => {
                    // Comment until end of line
                    while self.current_char.is_some() && self.current_char.unwrap() != '\n' {
                        self.next_character();
                    }
                }
                _ => {
                    if c.is_numeric() {
                        let num = self.parse_num();
                        self.tokens.push(Token::new(TokenType::PushInt(num), span));
                    } else {
                        self.parse_word();
                    }
                }
            }
        }
    }

    fn parse_word(&mut self) {
        let row = self.row;
        let col = self.column;
        let mut word = String::new();
        
        while self.current_char.is_some() && !self.current_char.unwrap().is_whitespace() {
            word.push(self.current_char.unwrap());
            self.next_character();
        }

        let span = Span::new(self.file_name.clone(), row, col);
        match word.as_str() {
            "and" => self.tokens.push(Token::new(TokenType::InfixOperators(InfixOperators::And), span)),
            "or" => self.tokens.push(Token::new(TokenType::InfixOperators(InfixOperators::Or), span)),
            "pop" => self.tokens.push(Token::new(TokenType::Pop, span)),
            "swap" => self.tokens.push(Token::new(TokenType::Swap, span)),
            "rot" => self.tokens.push(Token::new(TokenType::Rot, span)),
            "over" => self.tokens.push(Token::new(TokenType::Over, span)),
            "pick" => self.tokens.push(Token::new(TokenType::Pick, span)),
            "put" => self.tokens.push(Token::new(TokenType::Put, span)),
            "if" => self.tokens.push(Token::new(TokenType::If, span)),
            "elif" => self.tokens.push(Token::new(TokenType::Elif, span)),
            "else" => self.tokens.push(Token::new(TokenType::Else, span)),
            "end" => self.tokens.push(Token::new(TokenType::End, span)),
            "do" => self.tokens.push(Token::new(TokenType::Do, span)),
            "while" => self.tokens.push(Token::new(TokenType::While, span)),
            "dup" => self.tokens.push(Token::new(TokenType::Dup, span)),
            "size" => self.tokens.push(Token::new(TokenType::Size, span)),
            "memory" => self.tokens.push(Token::new(TokenType::Memory, span)),
            "return" => self.tokens.push(Token::new(TokenType::Return, span)),
            "proc" => self.tokens.push(Token::new(TokenType::Procedure, span)),
            "inline" => self.tokens.push(Token::new(TokenType::Inline, span)),
            _ => {
                if word.starts_with("syscall") && word.len() == 8 {
                    let last_char = word.chars().last().unwrap();
                    if last_char.is_digit(10) {
                        let val = last_char.to_digit(10).unwrap() as u8;
                        if val <= 6 {
                            self.tokens.push(Token::new(TokenType::Syscall(val), span));
                            return;
                        }
                    }
                }
                self.tokens.push(Token::new(TokenType::Identifier(word), span));
            }
        }
    }

    fn parse_num(&mut self) -> i64 {
        let mut num_str = String::new();
        
        if self.current_char == Some('-') {
            num_str.push('-');
            self.next_character();
        }

        while self.current_char.is_some() && self.current_char.unwrap().is_numeric() {
            num_str.push(self.current_char.unwrap());
            self.next_character();
        }

        num_str.parse::<i64>().unwrap_or_else(|_| {
            let span = Span::new(self.file_name.clone(), self.row, self.column);
            throw_exception_span(&span, format!("'{}' is not a valid i64", num_str));
            unreachable!()
        })
    }

    fn parse_char_literal(&mut self) -> i64 {
        self.next_character(); // Move past opening '
        
        let c = self.current_char.expect("Unexpected EOF in char literal");
        let value = if c == '\\' {
            self.next_character();
            let escaped = self.current_char.expect("Unexpected EOF after \\");
            match escaped {
                'n' => 10,
                'r' => 13,
                't' => 9,
                '\\' => 92,
                '\'' => 39,
                '0' => 0,
                _ => {
                    let span = Span::new(self.file_name.clone(), self.row, self.column);
                    throw_exception_span(&span, format!("Unknown escape char \\{}", escaped));
                    unreachable!()
                }
            }
        } else {
            c as i64
        };

        self.next_character(); // Move to closing '
        if self.current_char != Some('\'') {
            let span = Span::new(self.file_name.clone(), self.row, self.column);
            throw_exception_span(&span, "Unclosed character literal".to_string());
        }

        value
    }

    fn filter_escape_sequences(&mut self, mut string: String) -> String {
        string = string.replace("\\n", "\n");
        string = string.replace("\\r", "\r");
        string = string.replace("\\t", "\t");
        string = string.replace("\\\"", "\"");
        string = string.replace("\\'", "'");
        string = string.replace("\\\\", "\\");
        string
    }

    fn next_character(&mut self) -> Option<char> {
        self.cursor += 1;
        self.current_char = self.input_chars.get(self.cursor).copied();
        self.peek_char = self.input_chars.get(self.cursor + 1).copied();

        if let Some('\n') = self.current_char {
            self.row += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }

        self.current_char
    }
}