#[derive(Debug, PartialEq, Clone)]
pub enum InfixOperators {
    Plus,
    Minus,
    Multiply,
    Divide,
    Equals,
    NotEquals,
    GreaterThan,
    LesserThan,
    GreaterOrEqualsTo,
    LesserOrEqualsTo,
    Modulo,
    And,
    Or,
}

impl InfixOperators {
    pub fn new(s: String) -> InfixOperators {
        match s.as_str() {
            "+" => InfixOperators::Plus,
            "-" => InfixOperators::Minus,
            "*" => InfixOperators::Multiply,
            "/" => InfixOperators::Divide,
            "=" => InfixOperators::Equals,
            "!=" => InfixOperators::NotEquals,
            "<" => InfixOperators::LesserThan,
            ">" => InfixOperators::GreaterThan,
            ">=" => InfixOperators::GreaterOrEqualsTo,
            "<=" => InfixOperators::LesserOrEqualsTo,
            "%" => InfixOperators::Modulo,
            "and" => InfixOperators::And,
            "or" => InfixOperators::Or,
            _ => unreachable!("'{}', is not an arithmetic operator", s),
        }
    }

    pub fn to_x86_64_instruction(&self) -> &str {
        match self {
            InfixOperators::Plus => "add",
            InfixOperators::Minus => "sub",
            InfixOperators::Multiply => "imul",
            InfixOperators::Divide => "idiv",
            InfixOperators::Equals => "sete",
            InfixOperators::NotEquals => "setne",
            InfixOperators::GreaterThan => "setg",
            InfixOperators::LesserThan => "setl",
            InfixOperators::GreaterOrEqualsTo => "setge",
            InfixOperators::LesserOrEqualsTo => "setle",
            InfixOperators::Modulo => "idiv",
            InfixOperators::And => "and",
            InfixOperators::Or => "or",
        }
    }
}

impl std::fmt::Display for InfixOperators {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let value = match self {
            InfixOperators::Plus => "Plus",
            InfixOperators::Minus => "Minus",
            InfixOperators::Multiply => "Multiply",
            InfixOperators::Divide => "Divide",
            InfixOperators::Equals => "Equals",
            InfixOperators::NotEquals => "NotEquals",
            InfixOperators::GreaterThan => "GreaterThan",
            InfixOperators::LesserThan => "LesserThan",
            InfixOperators::GreaterOrEqualsTo => "GreaterOrEqualsTo",
            InfixOperators::LesserOrEqualsTo => "LesserOrEqualsTo",
            InfixOperators::Modulo => "Modulo",
            InfixOperators::And => "And",
            InfixOperators::Or => "Or",
        };
        write!(f, "{}", value)
    }
}
