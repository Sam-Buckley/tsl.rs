use crate::lex::*;
use crate::transpiler::type_checker::*;

#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Assignment {
        variable: String,
        tp: Option<String>,
        value: Box<AstNode>,
    },
    BinaryOperation {
        operator: Token,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    Block {
        statements: Vec<AstNode>,
    },
    Function {
        name: String,
        arguments: Vec<(String, String)>, // (type, name)
        return_type: String,
        body: Box<AstNode>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<AstNode>,
    },
    Identifier {
        value: String,
    },
    If {
        condition: Box<AstNode>,
        consequence: Box<AstNode>,
        alternative: Option<Box<AstNode>>,
    },
    Number {
        value: i128,
    },
    Return {
        value: Box<AstNode>,
    },
    String {
        value: String,
    },
    Variable {
        value: String,
    },
    Bool {
        value: String,
    },
    While {
        condition: Box<AstNode>,
        body: Box<AstNode>,
    },
    Null,
    Struct {
        name: String,
        fields: Vec<(String, String)>,
    },
    Uninit {
        tp: String,
    },
    Pointer {
        value: Box<AstNode>,
    },
    Dereference {
        value: Box<AstNode>, // value to dereference
    },
    Eof,
}

impl AstNode {
    pub fn get_statements(&self) -> Vec<AstNode> {
        match self {
            AstNode::Block { statements } => statements.clone(),
            _ => vec![self.clone()],
        }
    }
    pub fn get_func_args(&self) -> Vec<(String, String)> {
        match self {
            AstNode::Function {
                arguments, name, ..
            } => {
                let mut args = arguments.clone();
                args.push(("".to_string(), name.clone()));
                args
            }
            _ => vec![],
        }
    }
    pub fn children_mut(&mut self) -> Vec<&mut AstNode> {
        match self {
            AstNode::Assignment { value, .. } => vec![value.as_mut()],
            AstNode::BinaryOperation { left, right, .. } => vec![left.as_mut(), right.as_mut()],
            AstNode::Block { statements } => statements.iter_mut().collect(),
            AstNode::Function { body, .. } => vec![body.as_mut()],
            AstNode::FunctionCall { arguments, .. } => arguments.iter_mut().collect(),
            AstNode::If {
                condition,
                consequence,
                alternative,
            } => {
                let mut children = vec![condition.as_mut(), consequence.as_mut()];
                if let Some(alt) = alternative {
                    children.push(alt.as_mut());
                }
                children
            }
            AstNode::Return { value } => vec![value.as_mut()],
            _ => vec![],
        }
    }
    pub fn get_type(&self) -> Type {
        match self {
            AstNode::Number { .. } => Type::Integer,
            AstNode::String { .. } => Type::String,
            AstNode::Identifier { .. } => Type::NotMentioned,
            AstNode::Variable { .. } => Type::NotMentioned,
            AstNode::FunctionCall { name, .. } => Type::NotMentioned,
            AstNode::BinaryOperation { operator, .. } => Type::from(operator.value.as_str()),
            AstNode::Assignment { tp, .. } => Type::from(tp.as_ref().unwrap().as_str()),
            AstNode::Return { value } => value.get_type(),
            AstNode::Uninit { tp } => Type::from(tp.as_str()),
            AstNode::Bool { .. } => Type::Bool,
            _ => Type::NotMentioned,
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    temp_checker: TypeChecker,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
            temp_checker: TypeChecker::new(),
        }
    }

    pub fn parse_program(&mut self) -> AstNode {
        let mut statements = Vec::new();
        while self.current_token().token_type != TokenType::EOF {
            if let Some(statement) = self.parse_statement() {
                statements.push(statement);
            } else {
                self.position += 1; // Move to the next token to avoid infinite loop
            }
        }
        statements.push(AstNode::Eof);
        AstNode::Block { statements }
    }

    fn parse_void(&mut self) -> AstNode {
        self.position += 1; // Skip 'void'
        AstNode::Null
    }

    fn parse_statement(&mut self) -> Option<AstNode> {
        match (
            self.current_token().token_type,
            self.current_token().value.as_str(),
        ) {
            (TokenType::Identifier, _) => {
                if self.peek_token().token_type == TokenType::Assignment {
                    self.parse_assignment()
                } else {
                    Some(self.parse_expression())
                }
            }
            (TokenType::Keyword, "let") => self.parse_let_statement(),
            (TokenType::Keyword, "if") => self.parse_if_statement(),
            (TokenType::Keyword, "while") => self.parse_while_statement(),
            (TokenType::Keyword, "func") => self.parse_function(),
            (TokenType::Keyword, "return") => self.parse_return_statement(),
            (TokenType::Keyword, "struct") => self.parse_struct(),
            (TokenType::Keyword, "true") => Some(self.parse_true()),
            (TokenType::Keyword, "false") => Some(self.parse_false()),
            (TokenType::Void, _) => Some(self.parse_void()),
            (TokenType::True, _) => Some(self.parse_true()),
            (TokenType::False, _) => Some(self.parse_false()),
            // (TokenType::Block, _) => None,
            _ => Some(self.parse_expression()),
        }
    }

    fn parse_let_statement(&mut self) -> Option<AstNode> {
        self.position += 1; // Skip 'let'
        let variable = self.current_token().value.clone();
        self.position += 1; // Skip variable name
        let tp = if self.current_token().token_type == TokenType::Identifier
            || self.current_token().token_type == TokenType::TypeName
        {
            let tp = self.current_token().value.clone();
            self.position += 1; // Skip type name
            Some(tp)
        } else {
            Some("NotMentioned".to_string())
        };
        self.position += 1; // Skip '='
        if self.current_token().token_type == TokenType::Newline {
            self.position -= 1;
            return Some(AstNode::Assignment {
                variable,
                tp: tp.clone(),
                value: Box::new(AstNode::Uninit { tp: tp.unwrap() }),
            });
        }
        let value = self.parse_expression();
        let tp = if let Some(tp) = tp.clone() {
            if tp == "NotMentioned" {
                let value_type = self.temp_checker.check(&value);
                let value_type = value_type.unwrap();
                Some(String::from(value_type))
            } else {
                Some(tp)
            }
        } else {
            tp
        };
        self.temp_checker
            .symbol_table
            .insert(variable.clone(), Type::from(tp.clone().unwrap()));
        self.skip_semicolon();
        Some(AstNode::Assignment {
            variable,
            tp,
            value: Box::new(value),
        })
    }

    fn parse_true(&mut self) -> AstNode {
        self.position += 1; // Skip 'true'
        AstNode::Bool {
            value: "true".to_string(),
        }
    }

    fn parse_false(&mut self) -> AstNode {
        self.position += 1; // Skip 'false'
        AstNode::Bool {
            value: "false".to_string(),
        }
    }

    fn parse_assignment(&mut self) -> Option<AstNode> {
        let variable = self.current_token().value.clone();
        self.position += 1; // Skip variable name
        let tp = if self.current_token().token_type == TokenType::Identifier
            || self.current_token().token_type == TokenType::TypeName
        {
            let tp = self.current_token().value.clone();
            self.position += 1; // Skip type name
            Some(tp)
        } else {
            Some("NotMentioned".to_string())
        };
        self.position += 1; // Skip '='
        if self.current_token().token_type == TokenType::Newline {
            self.position -= 1;
            return Some(AstNode::Assignment {
                variable,
                tp: tp.clone(),
                value: Box::new(AstNode::Uninit { tp: tp.unwrap() }),
            });
        }
        let value = self.parse_expression();
        let tp = if let Some(tp) = tp.clone() {
            if tp == "NotMentioned" {
                let value_type = self.temp_checker.check(&value).unwrap();
                Some(String::from(value_type))
            } else {
                Some(tp)
            }
        } else {
            tp
        };
        self.skip_semicolon();
        Some(AstNode::Assignment {
            variable,
            tp,
            value: Box::new(value),
        })
    }

    fn parse_if_statement(&mut self) -> Option<AstNode> {
        self.position += 1; // Skip 'if'
        let condition = self.parse_expression();
        self.position += 1; // Skip ':'
        let consequence = self.parse_block();
        let alternative = if self.current_token().value == "else" {
            self.position += 2; // Skip 'else' and ':'
            Some(Box::new(self.parse_block()))
        } else {
            None
        };
        Some(AstNode::If {
            condition: Box::new(condition),
            consequence: Box::new(consequence),
            alternative,
        })
    }

    fn parse_while_statement(&mut self) -> Option<AstNode> {
        self.position += 1; // Skip 'while'
        let condition = self.parse_expression();
        let body = self.parse_block();
        Some(AstNode::While {
            condition: Box::new(condition),
            body: Box::new(body),
        })
    }

    fn parse_function(&mut self) -> Option<AstNode> {
        self.position += 1; // Skip 'func'
        let name = self.current_token().value.clone();
        self.position += 1; // Skip function name
        self.position += 1; // Skip '('
        let mut arguments = Vec::new();
        while self.current_token().token_type == TokenType::Identifier {
            let arg_name = self.current_token().value.clone();
            self.position += 1; // Skip argument name
            let arg_type = self.current_token().value.clone();
            self.position += 1; // Skip argument type
            arguments.push((arg_type, arg_name));
            if self.current_token().token_type == TokenType::Comma {
                self.position += 1; // Skip ','
            }
        }
        self.position += 1; // Skip ')'
        let return_type = self.current_token().value.clone();
        self.position += 1; // Skip return type
        self.position += 1;
        let body = Box::new(self.parse_block());
        Some(AstNode::Function {
            name,
            arguments,
            return_type,
            body,
        })
    }

    fn parse_return_statement(&mut self) -> Option<AstNode> {
        self.position += 1; // Skip 'return'
        let value = self.parse_expression();
        self.skip_semicolon();
        Some(AstNode::Return {
            value: Box::new(value),
        })
    }

    fn parse_struct(&mut self) -> Option<AstNode> {
        self.position += 1; // Skip 'struct'
        let name = self.current_token().value.clone();
        self.position += 1; // Skip struct name
        self.position += 1; // Skip '|'
        let mut fields = Vec::new();
        while self.current_token().token_type == TokenType::Identifier {
            let field_type = self.current_token().value.clone();
            self.position += 1; // Skip field type
            let field_name = self.current_token().value.clone();
            self.position += 1; // Skip field name
            fields.push((field_type, field_name));
        }
        self.position += 1; // Skip '|'
        Some(AstNode::Struct { name, fields })
    }

    fn parse_block(&mut self) -> AstNode {
        let mut statements = Vec::new();
        while self.current_token().token_type != TokenType::Block {
            if let Some(statement) = self.parse_statement() {
                statements.push(statement);
            } else {
                self.position += 1; // Move to the next token to avoid infinite loop
            }
        }
        self.position += 1; // Skip '|'
        AstNode::Block { statements }
    }

    fn parse_expression(&mut self) -> AstNode {
        let left = self.parse_primary();
        if self.position < self.tokens.len()
            && self.is_operator(self.current_token().value.as_str())
        {
            let operator = self.current_token().clone();
            self.position += 1; // Skip operator
            let right = self.parse_expression();
            AstNode::BinaryOperation {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            }
        } else {
            left
        }
    }

    fn parse_primary(&mut self) -> AstNode {
        match self.current_token().token_type {
            TokenType::Identifier => {
                let identifier = self.parse_identifier();
                if self.current_token().token_type == TokenType::LeftParen {
                    self.parse_function_call(identifier)
                } else if self.current_token().token_type == TokenType::Dollar {
                    self.position += 1; // Skip '$'
                    let mut arguments = Vec::new();
                    // until newline or EOF parse arguments
                    while self.current_token().token_type != TokenType::SemiColon {
                        let argument = self.parse_expression();
                        arguments.push(argument);
                        if self.current_token().token_type == TokenType::Comma {
                            self.position += 1; // Skip ','
                        }
                    }
                    self.position += 1; // Skip ';'
                    AstNode::FunctionCall {
                        name: match identifier {
                            AstNode::Identifier { value } => value,
                            _ => panic!("Expected identifier for function call"),
                        },
                        arguments,
                    }
                } else {
                    identifier
                }
            }
            TokenType::Number => self.parse_number(),
            TokenType::StringLiteral => self.parse_string(),
            TokenType::LeftParen => self.parse_grouped_expression(),
            TokenType::True => self.parse_true(),
            TokenType::False => self.parse_false(),
            TokenType::Void => self.parse_void(),
            TokenType::Ampersand => self.parse_pointer(),
            TokenType::Deref => self.parse_deref(),
            _ => panic!("Unexpected token: {:?}", self.current_token()),
        }
    }
    fn parse_pointer(&mut self) -> AstNode {
        self.position += 1; // Skip '&'
        let value = self.parse_expression();
        AstNode::Pointer {
            value: Box::new(value),
        }
    }
    fn parse_deref(&mut self) -> AstNode {
        self.position += 1; // Skip '*'
        let value = self.parse_expression();
        AstNode::Dereference {
            value: Box::new(value),
        }
    }
    fn parse_function_call(&mut self, identifier: AstNode) -> AstNode {
        let name = if let AstNode::Identifier { value } = identifier {
            value
        } else {
            panic!("Expected identifier for function call");
        };
        let mut arguments = Vec::new();
        self.position += 1; // Skip '('
        while self.current_token().token_type != TokenType::RightParen {
            let argument = self.parse_expression();
            arguments.push(argument);
            if self.current_token().token_type == TokenType::Comma {
                self.position += 1; // Skip ','
            }
        }
        self.position += 1; // Skip ')'
        AstNode::FunctionCall { name, arguments }
    }

    fn parse_number(&mut self) -> AstNode {
        let value = self.current_token().value.parse().unwrap();
        self.position += 1;
        AstNode::Number { value }
    }

    fn parse_string(&mut self) -> AstNode {
        let value = self.current_token().value.clone();
        self.position += 1;
        AstNode::String { value }
    }

    fn parse_identifier(&mut self) -> AstNode {
        let value = self.current_token().value.clone();
        self.position += 1;
        AstNode::Identifier { value }
    }

    fn parse_grouped_expression(&mut self) -> AstNode {
        self.position += 1; // Skip '('
        let expression = self.parse_expression();
        self.position += 1; // Skip ')'
        expression
    }

    fn current_token(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn peek_token(&self) -> &Token {
        if self.position + 1 >= self.tokens.len() {
            return &self.tokens[self.position];
        }
        &self.tokens[self.position + 1]
    }

    fn next_token(&mut self) {
        self.position += 1;
    }

    fn skip_semicolon(&mut self) {
        if self.position < self.tokens.len()
            && self.current_token().token_type == TokenType::SemiColon
        {
            self.position += 1; // Skip ';'
        }
    }

    fn is_operator(&self, value: &str) -> bool {
        matches! {
            value,
            "+" | "-" | "*" | "/" | ">" | "<" | "==" | "!="
        }
    }
}
