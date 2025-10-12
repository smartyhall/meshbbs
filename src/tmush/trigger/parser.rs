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

/// Natural language token types for beginner-friendly scripting
#[derive(Debug, Clone, PartialEq)]
pub enum NaturalToken {
    // Keywords for actions
    Say,              // "Say <text>"
    SayToRoom,        // "Say to room <text>"
    Give,             // "Give player <item/number> <unit>"
    Take,             // "Take from player <item>"
    Remove,           // "Remove this object"
    Teleport,         // "Teleport player to <room>"
    Unlock,           // "Unlock <direction>"
    Lock,             // "Lock <direction>"
    
    // Keywords for conditions
    If,               // "If <condition>:"
    Otherwise,        // "Otherwise:"
    
    // Logic keywords
    And,              // "and"
    Or,               // "or"
    
    // Phrases (parsed text between keywords)
    Phrase(String),   // "player has key", "room flag safe"
    
    // Literals
    Text(String),     // Quoted or unquoted text
    Number(i64),      // Numeric values
    
    // Structural
    Colon,            // ":" at end of If/Otherwise
    Newline,          // Line break
    Indent,           // Indentation (for block structure)
    
    // Special
    Eof,
}

impl fmt::Display for NaturalToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NaturalToken::Say => write!(f, "Say"),
            NaturalToken::SayToRoom => write!(f, "Say to room"),
            NaturalToken::Give => write!(f, "Give"),
            NaturalToken::Take => write!(f, "Take"),
            NaturalToken::Remove => write!(f, "Remove"),
            NaturalToken::Teleport => write!(f, "Teleport"),
            NaturalToken::Unlock => write!(f, "Unlock"),
            NaturalToken::Lock => write!(f, "Lock"),
            NaturalToken::If => write!(f, "If"),
            NaturalToken::Otherwise => write!(f, "Otherwise"),
            NaturalToken::And => write!(f, "and"),
            NaturalToken::Or => write!(f, "or"),
            NaturalToken::Phrase(s) => write!(f, "phrase '{}'", s),
            NaturalToken::Text(s) => write!(f, "text '{}'", s),
            NaturalToken::Number(n) => write!(f, "number {}", n),
            NaturalToken::Colon => write!(f, ":"),
            NaturalToken::Newline => write!(f, "newline"),
            NaturalToken::Indent => write!(f, "indent"),
            NaturalToken::Eof => write!(f, "end of input"),
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

/// Script syntax types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxType {
    /// Natural language syntax (beginner-friendly)
    Natural,
    /// Advanced DSL syntax (function-based)
    Advanced,
}

/// Detect which syntax type a script is using
///
/// Checks for natural language keywords (Say, Give, If) or DSL syntax
/// (function calls with parentheses). Defaults to Advanced if ambiguous.
pub fn detect_syntax_type(script: &str) -> SyntaxType {
    let lowercase = script.to_lowercase();
    
    // Natural language keywords (case-insensitive)
    let natural_keywords = [
        "say to room",
        "give player",
        "take from player",
        "remove this object",
        "teleport player to",
        "unlock ",
        "lock ",
        "if player has",
        "if room flag",
        "if object flag",
        "otherwise:",
        "say \"",
        "give ",
        "take ",
    ];
    
    // Check for natural language indicators
    for keyword in &natural_keywords {
        if lowercase.contains(keyword) {
            return SyntaxType::Natural;
        }
    }
    
    // Advanced DSL indicators: function calls with parentheses
    if script.contains("(") && script.contains(")") {
        return SyntaxType::Advanced;
    }
    
    // Default to Advanced for backward compatibility
    SyntaxType::Advanced
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

/// Natural Language Tokenizer for beginner-friendly scripts
/// 
/// Tokenizes natural language syntax like:
/// ```text
/// Say "Hello!"
/// If player has key:
///   Unlock north
/// Otherwise:
///   Say "It's locked."
/// ```
pub struct NaturalLanguageTokenizer {
    input: Vec<char>,
    position: usize,
}

impl NaturalLanguageTokenizer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<NaturalToken>, String> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            self.skip_whitespace_except_newlines();
            
            if self.is_at_end() {
                break;
            }
            
            // Check for newlines
            if self.current() == Some('\n') {
                tokens.push(NaturalToken::Newline);
                self.advance();
                continue;
            }
            
            // Check for indentation at start of line
            if tokens.is_empty() || matches!(tokens.last(), Some(NaturalToken::Newline)) {
                let indent_count = self.count_leading_spaces();
                if indent_count >= 2 {
                    tokens.push(NaturalToken::Indent);
                }
            }
            
            // Try to parse keywords
            if let Some(token) = self.try_parse_keyword()? {
                tokens.push(token);
                continue;
            }
            
            // Try to parse numbers
            if self.current().map_or(false, |c| c.is_ascii_digit()) {
                tokens.push(self.parse_number()?);
                continue;
            }
            
            // Try to parse quoted text
            if self.current() == Some('"') {
                tokens.push(self.parse_quoted_text()?);
                continue;
            }
            
            // Skip colons (they're part of If/Otherwise keywords)
            if self.current() == Some(':') {
                self.advance();
                continue;
            }
            
            // If we get here and there's nothing left but whitespace until newline,
            // just skip to the newline to avoid infinite loops
            let remaining_on_line = self.peek_until_newline();
            if remaining_on_line.trim().is_empty() {
                // Skip to newline or end
                while !self.is_at_end() && self.current() != Some('\n') {
                    self.advance();
                }
                continue;
            }
            
            // Parse as phrase (everything else until newline/colon)
            tokens.push(self.parse_phrase()?);
        }
        
        tokens.push(NaturalToken::Eof);
        Ok(tokens)
    }
    
    fn try_parse_keyword(&mut self) -> Result<Option<NaturalToken>, String> {
        let start_pos = self.position;
        
        // Try multi-word keywords first
        if self.match_keyword_sequence(&["Say", "to", "room"]) {
            return Ok(Some(NaturalToken::SayToRoom));
        }
        
        if self.match_keyword_sequence(&["Give", "player"]) {
            return Ok(Some(NaturalToken::Give));
        }
        
        if self.match_keyword_sequence(&["Take", "from", "player"]) {
            return Ok(Some(NaturalToken::Take));
        }
        
        if self.match_keyword_sequence(&["Remove", "this", "object"]) {
            return Ok(Some(NaturalToken::Remove));
        }
        
        if self.match_keyword_sequence(&["Teleport", "player", "to"]) {
            return Ok(Some(NaturalToken::Teleport));
        }
        
        // Reset position for single-word keyword check
        self.position = start_pos;
        
        // Try single-word keywords
        let word = self.peek_word();
        let token = match word.as_str() {
            "Say" => Some(NaturalToken::Say),
            "Unlock" => Some(NaturalToken::Unlock),
            "Lock" => Some(NaturalToken::Lock),
            "If" => Some(NaturalToken::If),
            "Otherwise" => Some(NaturalToken::Otherwise),
            "and" => Some(NaturalToken::And),
            "or" => Some(NaturalToken::Or),
            _ => None,
        };
        
        if token.is_some() {
            self.consume_word();
            
            // Check for colon after If/Otherwise
            self.skip_whitespace_except_newlines();
            if self.current() == Some(':') {
                self.advance();
                // Don't return Colon token separately, it's implied with If/Otherwise
            }
        }
        
        Ok(token)
    }
    
    fn match_keyword_sequence(&mut self, keywords: &[&str]) -> bool {
        let start_pos = self.position;
        
        for (i, keyword) in keywords.iter().enumerate() {
            if i > 0 {
                self.skip_whitespace_except_newlines();
            }
            
            if self.peek_word().to_lowercase() != keyword.to_lowercase() {
                self.position = start_pos;
                return false;
            }
            
            self.consume_word();
        }
        
        true
    }
    
    fn peek_word(&self) -> String {
        let mut word = String::new();
        let mut pos = self.position;
        
        while pos < self.input.len() {
            let ch = self.input[pos];
            if ch.is_alphanumeric() || ch == '_' {
                word.push(ch);
                pos += 1;
            } else {
                break;
            }
        }
        
        word
    }
    
    fn consume_word(&mut self) {
        while !self.is_at_end() {
            let ch = self.current().unwrap();
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn parse_number(&mut self) -> Result<NaturalToken, String> {
        let mut num_str = String::new();
        
        while !self.is_at_end() {
            if let Some(ch) = self.current() {
                if ch.is_ascii_digit() {
                    num_str.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        num_str.parse::<i64>()
            .map(NaturalToken::Number)
            .map_err(|_| format!("Invalid number: {}", num_str))
    }
    
    fn parse_quoted_text(&mut self) -> Result<NaturalToken, String> {
        self.advance(); // Skip opening quote
        let mut text = String::new();
        
        while !self.is_at_end() {
            match self.current() {
                Some('"') => {
                    self.advance(); // Skip closing quote
                    return Ok(NaturalToken::Text(text));
                }
                Some('\\') => {
                    self.advance();
                    if let Some(escaped) = self.current() {
                        text.push(match escaped {
                            'n' => '\n',
                            't' => '\t',
                            '\\' => '\\',
                            '"' => '"',
                            _ => escaped,
                        });
                        self.advance();
                    }
                }
                Some(ch) => {
                    text.push(ch);
                    self.advance();
                }
                None => break,
            }
        }
        
        Err("Unterminated string".to_string())
    }
    
    fn parse_phrase(&mut self) -> Result<NaturalToken, String> {
        let mut phrase = String::new();
        
        while !self.is_at_end() {
            match self.current() {
                Some('\n') | Some(':') => break,
                Some(ch) => {
                    phrase.push(ch);
                    self.advance();
                }
                None => break,
            }
        }
        
        let trimmed = phrase.trim().to_string();
        
        // If phrase is empty, this is an error - we should have consumed something
        if trimmed.is_empty() {
            return Err("Unexpected empty phrase - possible infinite loop detected".to_string());
        }
        
        Ok(NaturalToken::Phrase(trimmed))
    }
    
    fn count_leading_spaces(&mut self) -> usize {
        let mut count = 0;
        let start_pos = self.position;
        
        while !self.is_at_end() {
            if self.current() == Some(' ') {
                count += 1;
                self.advance();
            } else {
                break;
            }
        }
        
        // If only spaces until newline, don't count as indent
        if self.is_at_end() || self.current() == Some('\n') {
            self.position = start_pos;
            return 0;
        }
        
        count
    }
    
    fn skip_whitespace_except_newlines(&mut self) {
        while !self.is_at_end() {
            if let Some(ch) = self.current() {
                if ch == ' ' || ch == '\t' || ch == '\r' {
                    self.advance();
                } else {
                    break;
                }
            }
        }
    }
    
    fn peek_until_newline(&self) -> String {
        let mut result = String::new();
        let mut pos = self.position;
        
        while pos < self.input.len() {
            let ch = self.input[pos];
            if ch == '\n' {
                break;
            }
            result.push(ch);
            pos += 1;
        }
        
        result
    }
    
    fn current(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }
    
    fn advance(&mut self) {
        if self.position < self.input.len() {
            self.position += 1;
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

/// Natural Language Compiler - converts natural language tokens to AST
/// 
/// Transforms beginner-friendly syntax like:
/// ```text
/// Say "Hello!"
/// If player has key:
///   Unlock north
/// Otherwise:
///   Say "Locked"
/// ```
/// 
/// Into AST nodes that the evaluator can execute.
pub struct NaturalLanguageCompiler {
    tokens: Vec<NaturalToken>,
    position: usize,
}

impl NaturalLanguageCompiler {
    pub fn new(tokens: Vec<NaturalToken>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }
    
    pub fn compile(&mut self) -> Result<AstNode, String> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            // Skip newlines at statement level
            if matches!(self.current(), NaturalToken::Newline) {
                self.advance();
                continue;
            }
            
            // Skip indentation at top level
            if matches!(self.current(), NaturalToken::Indent) {
                self.advance();
                continue;
            }
            
            if self.is_at_end() {
                break;
            }
            
            let statement = self.compile_statement()?;
            statements.push(statement);
        }
        
        // If only one statement, return it directly
        // Otherwise, wrap in Sequence
        match statements.len() {
            0 => Err("Empty script".to_string()),
            1 => Ok(statements.into_iter().next().unwrap()),
            _ => Ok(AstNode::Sequence(statements)),
        }
    }
    
    fn compile_statement(&mut self) -> Result<AstNode, String> {
        match self.current() {
            NaturalToken::If => self.compile_if_statement(),
            NaturalToken::Say => self.compile_say(),
            NaturalToken::SayToRoom => self.compile_say_to_room(),
            NaturalToken::Give => self.compile_give(),
            NaturalToken::Take => self.compile_take(),
            NaturalToken::Remove => self.compile_remove(),
            NaturalToken::Teleport => self.compile_teleport(),
            NaturalToken::Unlock => self.compile_unlock(),
            NaturalToken::Lock => self.compile_lock(),
            NaturalToken::Eof => Err("Unexpected end of script".to_string()),
            token => Err(format!("Unexpected token at statement level: {}", token)),
        }
    }
    
    fn compile_if_statement(&mut self) -> Result<AstNode, String> {
        self.advance(); // consume 'If'
        
        // Parse condition phrase
        let condition = self.compile_condition()?;
        
        // Skip newline after condition
        self.skip_newlines();
        
        // Parse then branch (may be indented)
        let then_statements = self.compile_block()?;
        
        // Check for Otherwise
        let else_statements = if matches!(self.current(), NaturalToken::Otherwise) {
            self.advance(); // consume 'Otherwise'
            self.skip_newlines();
            self.compile_block()?
        } else {
            // No else branch - create empty action
            AstNode::Action {
                name: "noop".to_string(),
                args: vec![],
            }
        };
        
        Ok(AstNode::Ternary {
            condition: Box::new(condition),
            then_branch: Box::new(then_statements),
            else_branch: Box::new(else_statements),
        })
    }
    
    fn compile_block(&mut self) -> Result<AstNode, String> {
        let mut statements = Vec::new();
        
        // Check if block is indented
        let is_indented = matches!(self.current(), NaturalToken::Indent);
        if is_indented {
            self.advance(); // consume indent
        }
        
        loop {
            // Stop at end of file
            if self.is_at_end() {
                break;
            }
            
            // Stop at Otherwise (for if/else)
            if matches!(self.current(), NaturalToken::Otherwise) {
                break;
            }
            
            // Stop at unindented content (end of block)
            if is_indented && !matches!(self.current(), NaturalToken::Indent | NaturalToken::Newline) {
                break;
            }
            
            // Skip newlines
            if matches!(self.current(), NaturalToken::Newline) {
                self.advance();
                continue;
            }
            
            // Skip indentation markers
            if matches!(self.current(), NaturalToken::Indent) {
                self.advance();
            }
            
            // Parse statement
            if !self.is_at_end() && !matches!(self.current(), NaturalToken::Otherwise | NaturalToken::Newline) {
                let stmt = self.compile_statement()?;
                statements.push(stmt);
            }
            
            // Skip trailing newline
            if matches!(self.current(), NaturalToken::Newline) {
                self.advance();
            }
        }
        
        match statements.len() {
            0 => Ok(AstNode::Action { name: "noop".to_string(), args: vec![] }),
            1 => Ok(statements.into_iter().next().unwrap()),
            _ => Ok(AstNode::Sequence(statements)),
        }
    }
    
    fn compile_condition(&mut self) -> Result<AstNode, String> {
        // Parse condition phrase like "player has key" or "room flag safe"
        let phrase = match self.current() {
            NaturalToken::Phrase(ref p) => p.clone(),
            token => return Err(format!("Expected condition phrase, found {}", token)),
        };
        self.advance();
        
        // Parse the phrase to determine condition type
        let words: Vec<&str> = phrase.split_whitespace().collect();
        
        // Check more specific patterns first!
        if words.len() >= 4 && words[0] == "player" && words[1] == "has" && words[2] == "quest" {
            // "player has quest <name>" → has_quest("name")
            let quest_name = words[3..].join(" ");
            Ok(AstNode::Action {
                name: "has_quest".to_string(),
                args: vec![AstNode::StringLiteral(quest_name)],
            })
        } else if words.len() >= 3 && words[0] == "player" && words[1] == "has" {
            // "player has <item>" → has_item("item")
            let item_name = words[2..].join(" ");
            Ok(AstNode::Action {
                name: "has_item".to_string(),
                args: vec![AstNode::StringLiteral(item_name)],
            })
        } else if words.len() >= 3 && words[0] == "room" && words[1] == "flag" {
            // "room flag <flag>" → room_flag("flag")
            let flag_name = words[2..].join(" ");
            Ok(AstNode::Action {
                name: "room_flag".to_string(),
                args: vec![AstNode::StringLiteral(flag_name)],
            })
        } else if words.len() >= 3 && words[0] == "object" && words[1] == "flag" {
            // "object flag <flag>" → flag_set("flag")
            let flag_name = words[2..].join(" ");
            Ok(AstNode::Action {
                name: "flag_set".to_string(),
                args: vec![AstNode::StringLiteral(flag_name)],
            })
        } else if words.len() >= 5 && words[1] == "in" && words[3] == "chance" {
            // "<num> in <total> chance" → random_chance(percentage)
            let numerator = words[0].parse::<i64>()
                .map_err(|_| format!("Invalid number in chance: {}", words[0]))?;
            let denominator = words[2].parse::<i64>()
                .map_err(|_| format!("Invalid number in chance: {}", words[2]))?;
            
            let percentage = if denominator > 0 {
                (numerator * 100) / denominator
            } else {
                return Err("Denominator cannot be zero in chance".to_string());
            };
            
            Ok(AstNode::Action {
                name: "random_chance".to_string(),
                args: vec![AstNode::NumberLiteral(percentage)],
            })
        } else {
            Err(format!("Unknown condition format: {}", phrase))
        }
    }
    
    fn compile_say(&mut self) -> Result<AstNode, String> {
        self.advance(); // consume 'Say'
        
        let text = match self.current() {
            NaturalToken::Text(ref t) => t.clone(),
            NaturalToken::Phrase(ref p) => p.clone(),
            token => return Err(format!("Expected text after Say, found {}", token)),
        };
        self.advance();
        
        Ok(AstNode::Action {
            name: "message".to_string(),
            args: vec![AstNode::StringLiteral(text)],
        })
    }
    
    fn compile_say_to_room(&mut self) -> Result<AstNode, String> {
        self.advance(); // consume 'Say to room'
        
        let text = match self.current() {
            NaturalToken::Text(ref t) => t.clone(),
            NaturalToken::Phrase(ref p) => p.clone(),
            token => return Err(format!("Expected text after 'Say to room', found {}", token)),
        };
        self.advance();
        
        Ok(AstNode::Action {
            name: "message_room".to_string(),
            args: vec![AstNode::StringLiteral(text)],
        })
    }
    
    fn compile_give(&mut self) -> Result<AstNode, String> {
        self.advance(); // consume 'Give player'
        
        // Next should be either a number or item name
        match self.current() {
            NaturalToken::Number(n) => {
                let amount = *n;
                self.advance();
                
                // Get the unit (health, gold, item name)
                let unit = match self.current() {
                    NaturalToken::Phrase(ref p) => p.clone(),
                    NaturalToken::Text(ref t) => t.clone(),
                    token => return Err(format!("Expected unit after number, found {}", token)),
                };
                self.advance();
                
                if unit == "health" {
                    Ok(AstNode::Action {
                        name: "heal".to_string(),
                        args: vec![AstNode::NumberLiteral(amount)],
                    })
                } else {
                    // For now, treat other units as item grants
                    Ok(AstNode::Action {
                        name: "grant_item".to_string(),
                        args: vec![AstNode::StringLiteral(unit), AstNode::NumberLiteral(amount)],
                    })
                }
            }
            NaturalToken::Phrase(ref item) | NaturalToken::Text(ref item) => {
                let item_name = item.clone();
                self.advance();
                
                Ok(AstNode::Action {
                    name: "grant_item".to_string(),
                    args: vec![AstNode::StringLiteral(item_name)],
                })
            }
            token => Err(format!("Expected item name or number after 'Give player', found {}", token)),
        }
    }
    
    fn compile_take(&mut self) -> Result<AstNode, String> {
        self.advance(); // consume 'Take from player'
        
        let item = match self.current() {
            NaturalToken::Phrase(ref p) => p.clone(),
            NaturalToken::Text(ref t) => t.clone(),
            token => return Err(format!("Expected item name after 'Take from player', found {}", token)),
        };
        self.advance();
        
        Ok(AstNode::Action {
            name: "take_item".to_string(),
            args: vec![AstNode::StringLiteral(item)],
        })
    }
    
    fn compile_remove(&mut self) -> Result<AstNode, String> {
        self.advance(); // consume 'Remove this object'
        
        Ok(AstNode::Action {
            name: "consume".to_string(),
            args: vec![],
        })
    }
    
    fn compile_teleport(&mut self) -> Result<AstNode, String> {
        self.advance(); // consume 'Teleport player to'
        
        let room = match self.current() {
            NaturalToken::Phrase(ref p) => p.clone(),
            NaturalToken::Text(ref t) => t.clone(),
            token => return Err(format!("Expected room name after 'Teleport player to', found {}", token)),
        };
        self.advance();
        
        Ok(AstNode::Action {
            name: "teleport".to_string(),
            args: vec![AstNode::StringLiteral(room)],
        })
    }
    
    fn compile_unlock(&mut self) -> Result<AstNode, String> {
        self.advance(); // consume 'Unlock'
        
        let direction = match self.current() {
            NaturalToken::Phrase(ref p) => p.clone(),
            NaturalToken::Text(ref t) => t.clone(),
            token => return Err(format!("Expected direction after 'Unlock', found {}", token)),
        };
        self.advance();
        
        Ok(AstNode::Action {
            name: "unlock_exit".to_string(),
            args: vec![AstNode::StringLiteral(direction)],
        })
    }
    
    fn compile_lock(&mut self) -> Result<AstNode, String> {
        self.advance(); // consume 'Lock'
        
        let direction = match self.current() {
            NaturalToken::Phrase(ref p) => p.clone(),
            NaturalToken::Text(ref t) => t.clone(),
            token => return Err(format!("Expected direction after 'Lock', found {}", token)),
        };
        self.advance();
        
        Ok(AstNode::Action {
            name: "lock_exit".to_string(),
            args: vec![AstNode::StringLiteral(direction)],
        })
    }
    
    fn skip_newlines(&mut self) {
        while matches!(self.current(), NaturalToken::Newline) {
            self.advance();
        }
    }
    
    fn current(&self) -> &NaturalToken {
        if self.position < self.tokens.len() {
            &self.tokens[self.position]
        } else {
            &NaturalToken::Eof
        }
    }
    
    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }
    
    fn is_at_end(&self) -> bool {
        matches!(self.current(), NaturalToken::Eof) || self.position >= self.tokens.len()
    }
}

/// Compile natural language script to AST
pub fn compile_natural_language(script: &str) -> Result<AstNode, String> {
    let mut tokenizer = NaturalLanguageTokenizer::new(script);
    let tokens = tokenizer.tokenize()?;
    
    let mut compiler = NaturalLanguageCompiler::new(tokens);
    compiler.compile()
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
///
/// Automatically detects whether the script uses natural language syntax
/// or advanced DSL syntax, then routes to the appropriate parser.
///
/// # Examples
///
/// ```ignore
/// // Natural language (beginner-friendly)
/// let ast = parse_script("Say \"Hello!\"\nGive player 50 health").unwrap();
///
/// // Advanced DSL (power users)
/// let ast = parse_script("message(\"Hello!\") && heal(50)").unwrap();
/// ```
pub fn parse_script(script: &str) -> Result<AstNode, String> {
    match detect_syntax_type(script) {
        SyntaxType::Natural => compile_natural_language(script),
        SyntaxType::Advanced => {
            let mut tokenizer = Tokenizer::new(script);
            let tokens = tokenizer.tokenize()?;
            let mut parser = Parser::new(tokens);
            parser.parse()
        }
    }
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
    
    // Natural Language Tokenizer Tests
    
    #[test]
    fn test_nl_tokenize_simple_say() {
        let mut tokenizer = NaturalLanguageTokenizer::new("Say \"Hello!\"");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 3); // Say, Text, Eof
        assert_eq!(tokens[0], NaturalToken::Say);
        assert_eq!(tokens[1], NaturalToken::Text("Hello!".to_string()));
        assert_eq!(tokens[2], NaturalToken::Eof);
    }
    
    #[test]
    fn test_nl_tokenize_give_health() {
        let mut tokenizer = NaturalLanguageTokenizer::new("Give player 50 health");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 4); // Give, Number, Phrase, Eof
        assert_eq!(tokens[0], NaturalToken::Give);
        assert_eq!(tokens[1], NaturalToken::Number(50));
        assert_eq!(tokens[2], NaturalToken::Phrase("health".to_string()));
        assert_eq!(tokens[3], NaturalToken::Eof);
    }
    
    #[test]
    fn test_nl_tokenize_if_otherwise() {
        let script = "If player has key:\n  Unlock north\nOtherwise:\n  Say \"Locked\"";
        let mut tokenizer = NaturalLanguageTokenizer::new(script);
        let tokens = tokenizer.tokenize().unwrap();
        
        // Should have: If, Phrase, Newline, Indent, Unlock, Phrase, Newline, Otherwise, Newline, Indent, Say, Text, Eof
        assert!(tokens.contains(&NaturalToken::If));
        assert!(tokens.contains(&NaturalToken::Otherwise));
        assert!(tokens.contains(&NaturalToken::Unlock));
        assert!(tokens.contains(&NaturalToken::Say));
    }
    
    #[test]
    fn test_nl_tokenize_multi_word_keywords() {
        let mut tokenizer = NaturalLanguageTokenizer::new("Say to room \"Everyone hears this\"");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 3); // SayToRoom, Text, Eof
        assert_eq!(tokens[0], NaturalToken::SayToRoom);
        assert_eq!(tokens[1], NaturalToken::Text("Everyone hears this".to_string()));
    }
    
    #[test]
    fn test_nl_tokenize_remove_this_object() {
        let mut tokenizer = NaturalLanguageTokenizer::new("Remove this object");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 2); // Remove, Eof
        assert_eq!(tokens[0], NaturalToken::Remove);
        assert_eq!(tokens[1], NaturalToken::Eof);
    }
    
    // Natural Language Compiler Tests
    
    #[test]
    fn test_nl_compile_simple_say() {
        let ast = compile_natural_language("Say \"Hello!\"").unwrap();
        
        match ast {
            AstNode::Action { name, args } => {
                assert_eq!(name, "message");
                assert_eq!(args.len(), 1);
                match &args[0] {
                    AstNode::StringLiteral(s) => assert_eq!(s, "Hello!"),
                    _ => panic!("Expected string literal"),
                }
            }
            _ => panic!("Expected Action node, got {:?}", ast),
        }
    }
    
    #[test]
    fn test_nl_compile_give_health() {
        let ast = compile_natural_language("Give player 50 health").unwrap();
        
        match ast {
            AstNode::Action { name, args } => {
                assert_eq!(name, "heal");
                assert_eq!(args.len(), 1);
                match &args[0] {
                    AstNode::NumberLiteral(n) => assert_eq!(*n, 50),
                    _ => panic!("Expected number literal"),
                }
            }
            _ => panic!("Expected Action node"),
        }
    }
    
    #[test]
    fn test_nl_compile_unlock() {
        let ast = compile_natural_language("Unlock north").unwrap();
        
        match ast {
            AstNode::Action { name, args } => {
                assert_eq!(name, "unlock_exit");
                assert_eq!(args.len(), 1);
                match &args[0] {
                    AstNode::StringLiteral(s) => assert_eq!(s, "north"),
                    _ => panic!("Expected string literal"),
                }
            }
            _ => panic!("Expected Action node"),
        }
    }
    
    #[test]
    fn test_nl_compile_remove() {
        let ast = compile_natural_language("Remove this object").unwrap();
        
        match ast {
            AstNode::Action { name, .. } => {
                assert_eq!(name, "consume");
            }
            _ => panic!("Expected Action node"),
        }
    }
    
    #[test]
    fn test_nl_compile_if_otherwise() {
        let script = "If player has key:\n  Unlock north\nOtherwise:\n  Say \"Locked\"";
        let ast = compile_natural_language(script).unwrap();
        
        match ast {
            AstNode::Ternary { condition, then_branch, else_branch } => {
                // Check condition
                match *condition {
                    AstNode::Action { ref name, .. } => {
                        assert_eq!(name, "has_item");
                    }
                    _ => panic!("Expected Action in condition"),
                }
                
                // Check then branch
                match *then_branch {
                    AstNode::Action { ref name, .. } => {
                        assert_eq!(name, "unlock_exit");
                    }
                    _ => panic!("Expected Action in then branch"),
                }
                
                // Check else branch
                match *else_branch {
                    AstNode::Action { ref name, .. } => {
                        assert_eq!(name, "message");
                    }
                    _ => panic!("Expected Action in else branch"),
                }
            }
            _ => panic!("Expected Ternary node"),
        }
    }
    
    #[test]
    fn test_nl_compile_condition_player_has_quest() {
        let script = "If player has quest ancient_ruins:\n  Say \"Quest active!\"";
        let ast = compile_natural_language(script).unwrap();
        
        match ast {
            AstNode::Ternary { condition, .. } => {
                match *condition {
                    AstNode::Action { ref name, ref args } => {
                        assert_eq!(name, "has_quest");
                        match &args[0] {
                            AstNode::StringLiteral(s) => assert_eq!(s, "ancient_ruins"),
                            _ => panic!("Expected string literal"),
                        }
                    }
                    _ => panic!("Expected Action in condition"),
                }
            }
            _ => panic!("Expected Ternary node"),
        }
    }
    
    #[test]
    fn test_nl_compile_multiple_statements() {
        let script = "Say \"First\"\nGive player 25 health\nRemove this object";
        let ast = compile_natural_language(script).unwrap();
        
        match ast {
            AstNode::Sequence(statements) => {
                assert_eq!(statements.len(), 3);
                
                // Check first statement
                match &statements[0] {
                    AstNode::Action { name, .. } => assert_eq!(name, "message"),
                    _ => panic!("Expected Action"),
                }
                
                // Check second statement
                match &statements[1] {
                    AstNode::Action { name, .. } => assert_eq!(name, "heal"),
                    _ => panic!("Expected Action"),
                }
                
                // Check third statement
                match &statements[2] {
                    AstNode::Action { name, .. } => assert_eq!(name, "consume"),
                    _ => panic!("Expected Action"),
                }
            }
            _ => panic!("Expected Sequence node"),
        }
    }

    // ==================== Auto-Detection Tests ====================
    
    #[test]
    fn test_detect_natural_language() {
        // Test natural language keyword detection
        assert_eq!(detect_syntax_type("Say \"Hello!\""), SyntaxType::Natural);
        assert_eq!(detect_syntax_type("If player has quest ancient_ruins: Say \"Found\""), SyntaxType::Natural);
        assert_eq!(detect_syntax_type("Give player 50 health"), SyntaxType::Natural);
        assert_eq!(detect_syntax_type("Say to room \"Everyone hears this\""), SyntaxType::Natural);
        assert_eq!(detect_syntax_type("Unlock north"), SyntaxType::Natural);
        assert_eq!(detect_syntax_type("Otherwise:\n  Say \"No\""), SyntaxType::Natural);
    }
    
    #[test]
    fn test_detect_advanced_dsl() {
        // Test DSL function call detection
        assert_eq!(detect_syntax_type("message(\"Hello!\")"), SyntaxType::Advanced);
        assert_eq!(detect_syntax_type("has_item(\"key\") ? message(\"Yes\") : message(\"No\")"), SyntaxType::Advanced);
        assert_eq!(detect_syntax_type("heal(50) && message(\"Healed!\")"), SyntaxType::Advanced);
        
        // Empty or ambiguous defaults to Advanced
        assert_eq!(detect_syntax_type(""), SyntaxType::Advanced);
        assert_eq!(detect_syntax_type("random text"), SyntaxType::Advanced);
    }
    
    #[test]
    fn test_parse_script_routes_natural() {
        // Verify parse_script correctly routes natural language
        let script = "Say \"Hello from natural language!\"";
        let ast = parse_script(script).unwrap();
        
        match ast {
            AstNode::Action { name, args } => {
                assert_eq!(name, "message");
                assert_eq!(args.len(), 1);
                match &args[0] {
                    AstNode::StringLiteral(s) => assert_eq!(s, "Hello from natural language!"),
                    _ => panic!("Expected StringLiteral"),
                }
            }
            _ => panic!("Expected Action node"),
        }
    }
    
    #[test]
    fn test_parse_script_routes_advanced() {
        // Verify parse_script correctly routes DSL
        let script = "message(\"Hello from DSL!\")";
        let ast = parse_script(script).unwrap();
        
        match ast {
            AstNode::Action { name, args } => {
                assert_eq!(name, "message");
                assert_eq!(args.len(), 1);
                match &args[0] {
                    AstNode::StringLiteral(s) => assert_eq!(s, "Hello from DSL!"),
                    _ => panic!("Expected StringLiteral"),
                }
            }
            _ => panic!("Expected Action node"),
        }
    }
    
    #[test]
    fn test_parse_script_backward_compatible() {
        // Verify existing DSL scripts still parse correctly
        let script = "has_item(\"key\") ? teleport(\"treasure_room\") : message(\"Need key\")";
        let ast = parse_script(script).unwrap();
        
        // Should parse as ternary
        match ast {
            AstNode::Ternary { condition, then_branch, else_branch } => {
                // Condition should be has_item
                match *condition {
                    AstNode::Action { name, .. } => assert_eq!(name, "has_item"),
                    _ => panic!("Expected Action in condition"),
                }
                
                // Then should be teleport
                match *then_branch {
                    AstNode::Action { name, .. } => assert_eq!(name, "teleport"),
                    _ => panic!("Expected Action in then branch"),
                }
                
                // Else should be message
                match *else_branch {
                    AstNode::Action { name, .. } => assert_eq!(name, "message"),
                    _ => panic!("Expected Action in else branch"),
                }
            }
            _ => panic!("Expected Ternary node"),
        }
    }
    
    #[test]
    fn test_parse_script_natural_with_if() {
        // Test natural language if/otherwise through parse_script
        let script = "If player has quest ancient_ruins:\n  Say \"Quest complete!\"\nOtherwise:\n  Say \"Quest not found\"";
        let ast = parse_script(script).unwrap();
        
        match ast {
            AstNode::Ternary { condition, then_branch, else_branch } => {
                // Condition should be has_quest
                match *condition {
                    AstNode::Action { name, args } => {
                        assert_eq!(name, "has_quest");
                        assert_eq!(args.len(), 1);
                    }
                    _ => panic!("Expected Action in condition"),
                }
                
                // Then should be message
                match *then_branch {
                    AstNode::Action { name, .. } => assert_eq!(name, "message"),
                    _ => panic!("Expected Action in then branch"),
                }
                
                // Else should be message
                match *else_branch {
                    AstNode::Action { name, .. } => assert_eq!(name, "message"),
                    _ => panic!("Expected Action in else branch"),
                }
            }
            _ => panic!("Expected Ternary node"),
        }
    }
}
