/// DSL Parser for trigger scripts
///
/// Parses trigger script syntax into an Abstract Syntax Tree (AST)
/// that can be evaluated by the execution engine.
///
/// **Supported Syntax:**
/// - Actions: `message("text")`, `teleport("room")`, `heal(50)`
/// - Conditions: `has_item("key")`, `flag_set("flag")`, `current_room == "id"`
/// - Operators: `&&` (AND), `||` (OR), `?` (ternary if), `:` (ternary else)
/// - Variables: `$player`, `$object`, `$room`, `$time`

use std::fmt;

/// Token types for lexical analysis
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Identifiers and literals
    Identifier(String),
    String(String),
    Number(i64),
    Variable(String), // $player, $object, etc.
    
    // Operators
    And,              // &&
    Or,               // ||
    Question,         // ?
    Colon,            // :
    Equal,            // ==
    NotEqual,         // !=
    Greater,          // >
    Less,             // <
    GreaterEqual,     // >=
    LessEqual,        // <=
    
    // Delimiters
    LeftParen,        // (
    RightParen,       // )
    Comma,            // ,
    
    // Special
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Identifier(s) => write!(f, "identifier '{}'", s),
            Token::String(s) => write!(f, "string \"{}\"", s),
            Token::Number(n) => write!(f, "number {}", n),
            Token::Variable(v) => write!(f, "variable ${}", v),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::Question => write!(f, "?"),
            Token::Colon => write!(f, ":"),
            Token::Equal => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::Greater => write!(f, ">"),
            Token::Less => write!(f, "<"),
            Token::GreaterEqual => write!(f, ">="),
            Token::LessEqual => write!(f, "<="),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::Comma => write!(f, ","),
            Token::Eof => write!(f, "end of input"),
        }
    }
}

/// Abstract Syntax Tree node types
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    // Actions
    Action {
        name: String,
        args: Vec<AstNode>,
    },
    
    // Conditions/Expressions
    BinaryOp {
        op: BinaryOperator,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    
    Ternary {
        condition: Box<AstNode>,
        then_branch: Box<AstNode>,
        else_branch: Box<AstNode>,
    },
    
    // Literals
    StringLiteral(String),
    NumberLiteral(i64),
    Variable(String),
    
    // Compound (sequence of actions)
    Sequence(Vec<AstNode>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    And,
    Or,
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::And => write!(f, "&&"),
            BinaryOperator::Or => write!(f, "||"),
            BinaryOperator::Equal => write!(f, "=="),
            BinaryOperator::NotEqual => write!(f, "!="),
            BinaryOperator::Greater => write!(f, ">"),
            BinaryOperator::Less => write!(f, "<"),
            BinaryOperator::GreaterEqual => write!(f, ">="),
            BinaryOperator::LessEqual => write!(f, "<="),
        }
    }
}

/// Tokenizer for DSL scripts
pub struct Tokenizer {
    input: Vec<char>,
    position: usize,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }
    
    fn current(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }
    
    fn peek(&self, offset: usize) -> Option<char> {
        self.input.get(self.position + offset).copied()
    }
    
    fn advance(&mut self) -> Option<char> {
        let ch = self.current();
        self.position += 1;
        ch
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn read_string(&mut self) -> Result<String, String> {
        let quote = self.advance().unwrap(); // consume opening quote
        let mut result = String::new();
        
        while let Some(ch) = self.current() {
            if ch == quote {
                self.advance(); // consume closing quote
                return Ok(result);
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.advance() {
                    match escaped {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        '\\' => result.push('\\'),
                        '"' => result.push('"'),
                        '\'' => result.push('\''),
                        _ => result.push(escaped),
                    }
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }
        
        Err("Unterminated string literal".to_string())
    }
    
    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        
        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || ch == '_' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        result
    }
    
    fn read_number(&mut self) -> i64 {
        let mut result = String::new();
        
        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        result.parse().unwrap_or(0)
    }
    
    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();
        
        match self.current() {
            None => Ok(Token::Eof),
            
            Some('(') => {
                self.advance();
                Ok(Token::LeftParen)
            }
            
            Some(')') => {
                self.advance();
                Ok(Token::RightParen)
            }
            
            Some(',') => {
                self.advance();
                Ok(Token::Comma)
            }
            
            Some('?') => {
                self.advance();
                Ok(Token::Question)
            }
            
            Some(':') => {
                self.advance();
                Ok(Token::Colon)
            }
            
            Some('&') => {
                self.advance();
                if self.current() == Some('&') {
                    self.advance();
                    Ok(Token::And)
                } else {
                    Err("Expected '&&', found single '&'".to_string())
                }
            }
            
            Some('|') => {
                self.advance();
                if self.current() == Some('|') {
                    self.advance();
                    Ok(Token::Or)
                } else {
                    Err("Expected '||', found single '|'".to_string())
                }
            }
            
            Some('=') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::Equal)
                } else {
                    Err("Expected '==', found single '='".to_string())
                }
            }
            
            Some('!') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::NotEqual)
                } else {
                    Err("Expected '!=', found single '!'".to_string())
                }
            }
            
            Some('>') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::GreaterEqual)
                } else {
                    Ok(Token::Greater)
                }
            }
            
            Some('<') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::LessEqual)
                } else {
                    Ok(Token::Less)
                }
            }
            
            Some('"') | Some('\'') => {
                let s = self.read_string()?;
                Ok(Token::String(s))
            }
            
            Some('$') => {
                self.advance();
                let var_name = self.read_identifier();
                if var_name.is_empty() {
                    Err("Expected variable name after '$'".to_string())
                } else {
                    Ok(Token::Variable(var_name))
                }
            }
            
            Some(ch) if ch.is_ascii_digit() => {
                let num = self.read_number();
                Ok(Token::Number(num))
            }
            
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();
                Ok(Token::Identifier(ident))
            }
            
            Some(ch) => Err(format!("Unexpected character: '{}'", ch)),
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token()?;
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        
        Ok(tokens)
    }
}

/// Parser for DSL scripts
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }
    
    fn current(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }
    
    fn advance(&mut self) -> &Token {
        let token = self.current();
        self.position += 1;
        self.tokens.get(self.position - 1).unwrap_or(&Token::Eof)
    }
    
    fn expect(&mut self, expected: Token) -> Result<(), String> {
        let current = self.current().clone();
        if std::mem::discriminant(&current) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {}, found {}", expected, current))
        }
    }
    
    /// Parse a primary expression (literals, variables, function calls, parenthesized expressions)
    fn parse_primary(&mut self) -> Result<AstNode, String> {
        match self.current().clone() {
            Token::String(s) => {
                self.advance();
                Ok(AstNode::StringLiteral(s))
            }
            
            Token::Number(n) => {
                self.advance();
                Ok(AstNode::NumberLiteral(n))
            }
            
            Token::Variable(v) => {
                self.advance();
                Ok(AstNode::Variable(v))
            }
            
            Token::Identifier(name) => {
                self.advance();
                
                // Check if this is a function call
                if matches!(self.current(), Token::LeftParen) {
                    self.advance(); // consume '('
                    
                    let mut args = Vec::new();
                    
                    // Parse arguments
                    if !matches!(self.current(), Token::RightParen) {
                        loop {
                            let arg = self.parse_ternary()?;
                            args.push(arg);
                            
                            if matches!(self.current(), Token::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    
                    self.expect(Token::RightParen)?;
                    
                    Ok(AstNode::Action { name, args })
                } else {
                    // Bare identifier (for conditions like flags)
                    Ok(AstNode::Action { name, args: vec![] })
                }
            }
            
            Token::LeftParen => {
                self.advance(); // consume '('
                let expr = self.parse_ternary()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            
            token => Err(format!("Unexpected token in expression: {}", token)),
        }
    }
    
    /// Parse comparison operators (==, !=, >, <, >=, <=)
    fn parse_comparison(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_primary()?;
        
        while let Token::Equal | Token::NotEqual | Token::Greater | Token::Less | Token::GreaterEqual | Token::LessEqual = self.current() {
            let op = match self.current() {
                Token::Equal => BinaryOperator::Equal,
                Token::NotEqual => BinaryOperator::NotEqual,
                Token::Greater => BinaryOperator::Greater,
                Token::Less => BinaryOperator::Less,
                Token::GreaterEqual => BinaryOperator::GreaterEqual,
                Token::LessEqual => BinaryOperator::LessEqual,
                _ => unreachable!(),
            };
            
            self.advance();
            let right = self.parse_primary()?;
            
            left = AstNode::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Parse AND operator (&&)
    fn parse_and(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_comparison()?;
        
        while matches!(self.current(), Token::And) {
            self.advance();
            let right = self.parse_comparison()?;
            
            left = AstNode::BinaryOp {
                op: BinaryOperator::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Parse OR operator (||)
    fn parse_or(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_and()?;
        
        while matches!(self.current(), Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            
            left = AstNode::BinaryOp {
                op: BinaryOperator::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Parse ternary operator (condition ? then : else)
    fn parse_ternary(&mut self) -> Result<AstNode, String> {
        let condition = self.parse_or()?;
        
        if matches!(self.current(), Token::Question) {
            self.advance();
            let then_branch = self.parse_or()?;
            self.expect(Token::Colon)?;
            let else_branch = self.parse_ternary()?;
            
            Ok(AstNode::Ternary {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: Box::new(else_branch),
            })
        } else {
            Ok(condition)
        }
    }
    
    /// Parse the entire script
    pub fn parse(&mut self) -> Result<AstNode, String> {
        let expr = self.parse_ternary()?;
        
        if !matches!(self.current(), Token::Eof) {
            return Err(format!("Unexpected token after expression: {}", self.current()));
        }
        
        Ok(expr)
    }
}

/// Parse a trigger script into an AST
pub fn parse_script(script: &str) -> Result<AstNode, String> {
    let mut tokenizer = Tokenizer::new(script);
    let tokens = tokenizer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenize_simple() {
        let mut tokenizer = Tokenizer::new("message(\"hello\")");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 5); // identifier, (, string, ), eof
        assert!(matches!(tokens[0], Token::Identifier(_)));
        assert!(matches!(tokens[1], Token::LeftParen));
        assert!(matches!(tokens[2], Token::String(_)));
        assert!(matches!(tokens[3], Token::RightParen));
        assert!(matches!(tokens[4], Token::Eof));
    }
    
    #[test]
    fn test_tokenize_operators() {
        let mut tokenizer = Tokenizer::new("&& || == != > < >= <=");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert!(matches!(tokens[0], Token::And));
        assert!(matches!(tokens[1], Token::Or));
        assert!(matches!(tokens[2], Token::Equal));
        assert!(matches!(tokens[3], Token::NotEqual));
        assert!(matches!(tokens[4], Token::Greater));
        assert!(matches!(tokens[5], Token::Less));
        assert!(matches!(tokens[6], Token::GreaterEqual));
        assert!(matches!(tokens[7], Token::LessEqual));
    }
    
    #[test]
    fn test_tokenize_variables() {
        let mut tokenizer = Tokenizer::new("$player $object $room");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert!(matches!(tokens[0], Token::Variable(ref v) if v == "player"));
        assert!(matches!(tokens[1], Token::Variable(ref v) if v == "object"));
        assert!(matches!(tokens[2], Token::Variable(ref v) if v == "room"));
    }
    
    #[test]
    fn test_parse_simple_action() {
        let ast = parse_script("message(\"hello\")").unwrap();
        
        match ast {
            AstNode::Action { name, args } => {
                assert_eq!(name, "message");
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], AstNode::StringLiteral(_)));
            }
            _ => panic!("Expected Action node"),
        }
    }
    
    #[test]
    fn test_parse_condition() {
        let ast = parse_script("has_item(\"key\")").unwrap();
        
        match ast {
            AstNode::Action { name, args } => {
                assert_eq!(name, "has_item");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected Action node"),
        }
    }
    
    #[test]
    fn test_parse_comparison() {
        let ast = parse_script("current_room == \"test\"").unwrap();
        
        match ast {
            AstNode::BinaryOp { op, .. } => {
                assert_eq!(op, BinaryOperator::Equal);
            }
            _ => panic!("Expected BinaryOp node"),
        }
    }
    
    #[test]
    fn test_parse_ternary() {
        let ast = parse_script("has_item(\"key\") ? message(\"yes\") : message(\"no\")").unwrap();
        
        match ast {
            AstNode::Ternary { .. } => {
                // Success
            }
            _ => panic!("Expected Ternary node"),
        }
    }
    
    #[test]
    fn test_parse_and_or() {
        let ast = parse_script("has_item(\"a\") && has_item(\"b\")").unwrap();
        
        match ast {
            AstNode::BinaryOp { op, .. } => {
                assert_eq!(op, BinaryOperator::And);
            }
            _ => panic!("Expected BinaryOp node"),
        }
    }
}
