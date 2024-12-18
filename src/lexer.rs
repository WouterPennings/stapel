use crate::operators::{InfixOperators, PrefixOperator};
use crate::throw_exception_span;
use crate::tokens::{Span, Token, TokenType};

#[derive(Debug, PartialEq, Clone)]
pub struct Lexer {
    pub input: String,
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
        Lexer {
            current_char: input.chars().nth(0),
            peek_char: input.chars().nth(1),
            input,
            file_name,
            tokens: Vec::new(),
            cursor: 0,
            row: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) {
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
                    let filtered = self.filter_escape_sequences(str.clone());
                    self.tokens.push(Token::new(TokenType::PushStr(filtered, str), span))
                }
                ';' => while self.next_character().unwrap() != '\n' {},
                '!' => {
                    self.next_character();
                    if self.current_char.is_some() && self.current_char.unwrap() == '=' {
                        self.next_character();
                        self.tokens
                            .push(Token::new(TokenType::InfixOperators(InfixOperators::new("!=".to_string())), span));
                    } else {
                        let num = self.parse_num();
                        if num == 8 {
                            self.tokens.push(Token::new(
                                TokenType::Load(num as usize),
                                Span::new(self.file_name.clone(), row, col),
                            ));
                        } else {
                            throw_exception_span(&span, format!("'{}', is not a support bit amount", num));
                            unreachable!()
                        }
                    }
                }
                '@' => {
                    self.next_character();
                    let num = self.parse_num();
                    if num == 8 {
                        self.tokens.push(Token::new(TokenType::Store(num as usize), span));
                    } else {
                        throw_exception_span(&span, format!("'{}', is not a support bit amount", num));
                        unreachable!()
                    }
                }
                '=' => {
                    let op = InfixOperators::new(String::from(c));
                    self.tokens.push(Token::new(TokenType::InfixOperators(op), span));
                }
                '+' | '-' | '*' | '/' | '%' => {
                    if self.peek_char.unwrap() == '+' {
                        let op = PrefixOperator::new(String::from("++"));
                        self.tokens.push(Token::new(TokenType::PrefixOperator(op), span));
                        self.next_character();
                    } else if c == '-' && self.peek_char.unwrap().is_numeric() {
                        let num = self.parse_num();
                        self.tokens.push(Token::new(TokenType::PushInt(num), span));
                    } else {
                        let op = InfixOperators::new(String::from(c));
                        self.tokens.push(Token::new(TokenType::InfixOperators(op), span));
                    }
                }
                '<' | '>' => {
                    if self.peek_char.is_some() && self.peek_char.unwrap() == '=' {
                        self.next_character();
                        let op = InfixOperators::new(format!("{}=", c).to_string());
                        self.tokens.push(Token::new(TokenType::InfixOperators(op), span));
                    } else {
                        let op = InfixOperators::new(String::from(c));
                        self.tokens.push(Token::new(TokenType::InfixOperators(op), span));
                    }
                }
                '#' => {
                    self.next_character();

                    let start_index = self.cursor - 1;
                    let mut possible_char = self.input.chars().nth(self.cursor);

                    while possible_char.is_some() && self.current_char.unwrap() != '\n' { 
                        possible_char = self.input.chars().nth(self.cursor);
                        self.next_character();
                    }

                    // Replacing the whole comment with spaces.
                    // That way implementing line and column with an error is way easier.
                    let replacement = " ".repeat(self.cursor - start_index);
                    self.input.replace_range(start_index..self.cursor, &replacement);
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
            self.next_character();
        }
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
            "pop" => self.tokens.push(Token::new(TokenType::Pop, span)),
            "swap" => self.tokens.push(Token::new(TokenType::Swap, span)),
            "put" => self.tokens.push(Token::new(TokenType::Put, span)),
            "if" => self.tokens.push(Token::new(TokenType::If, span)),
            "else" => self.tokens.push(Token::new(TokenType::Else, span)),
            "end" => self.tokens.push(Token::new(TokenType::End, span)),
            "do" => self.tokens.push(Token::new(TokenType::Do, span)),
            "while" => self.tokens.push(Token::new(TokenType::While, span)),
            "dup" => self.tokens.push(Token::new(TokenType::Dup, span)),
            "size" => self.tokens.push(Token::new(TokenType::Size, span)),
            "mem" => self.tokens.push(Token::new(TokenType::Mem, span)),
            "return" => self.tokens.push(Token::new(TokenType::Return, span)),
            "proc" => self.tokens.push(Token::new(TokenType::Procedure, span)),
            _ => {
                if str.starts_with("syscall")
                    && str.len() == 8
                    && str.chars().last().unwrap() as u8 >= 48
                    && str.chars().last().unwrap() as u8 <= 54
                {
                    self.tokens.push(Token::new(TokenType::Syscall(str.chars().last().unwrap() as u8 - 48), span));
                } else {
                    self.tokens.push(Token::new(TokenType::Custom(str), span));
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
                throw_exception_span(&span, format!("'{}', is not an i32", num));
                unreachable!();
            }
        }
    }

    fn filter_escape_sequences(&mut self, mut string: String) -> String {
        let ori = string.clone();
        string = string.replace("\\\\", &char::from_u32(7).unwrap().to_string());
        string = string.replace("\\n", "\n");
        string = string.replace("\\r", "\r");
        string = string.replace("\\t", "\t");
        string = string.replace("\\\"", "\"");
        string = string.replace("\\'", "'");
        string = string.replace(&char::from_u32(7).unwrap().to_string(), "\\\\");
        if let Some(index) = string.find("\\") {
            let row = self.row;
            let col = self.column;
            let span = Span::new(self.file_name.clone(), row, col);
            throw_exception_span(
                &span,
                format!("Escape sequence '{}' in string: '{}' is not supported", &string[index..index + 2], ori),
            );
        }
        string = string.replace("\\\\", "\\");
        string
    }

    fn next_character(&mut self) -> Option<char> {
        self.cursor += 1;
        self.current_char = self.peek_char;
        self.peek_char = self.input.chars().nth(self.cursor+1);

        if self.current_char.unwrap_or('_') == '\n' {
            self.row += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }

        self.current_char
    }
}
