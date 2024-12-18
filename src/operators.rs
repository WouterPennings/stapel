#[derive(Debug, PartialEq, Clone)]
pub enum PrefixOperator {
    Plus,
}

impl PrefixOperator {
    pub fn new(s: String) -> PrefixOperator {
        match s.as_str() {
            "++" => PrefixOperator::Plus,
            _ => unreachable!("'{}', is not an arithmetic operator", s),
        }
    }

    pub fn to_x86_64_instruction(&self) -> &str {
        match self {
            PrefixOperator::Plus => "add",
        }
    }
}

impl std::fmt::Display for PrefixOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let value = match self {
            PrefixOperator::Plus => "Plus",

        };
        write!(f, "{}", value)
    }
}


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
            _ => unreachable!("'{}', is not an arithmetic operator", s),
        }
    }

    pub fn to_x86_64_instruction(&self) -> &str {
        match self {
            InfixOperators::Plus => "add",
            InfixOperators::Minus => "sub",
            InfixOperators::Multiply => "mul",
            InfixOperators::Divide => "idiv",
            InfixOperators::Equals => "cmove",
            InfixOperators::NotEquals => "cmovne",
            InfixOperators::GreaterThan => "cmovg",
            InfixOperators::LesserThan => "cmovl",
            InfixOperators::GreaterOrEqualsTo => "cmovge",
            InfixOperators::LesserOrEqualsTo => "cmovle",
            InfixOperators::Modulo => "idiv",
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
            InfixOperators::Modulo => "Modulo"
        };
        write!(f, "{}", value)
    }
}
