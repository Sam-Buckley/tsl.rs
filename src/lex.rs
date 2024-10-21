#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    Keyword,
    Identifier,
    Number,
    Operator,
    Assignment,
    Comparison,
    Char,
    StringLiteral,
    LeftParen,
    RightParen,
    Block,
    Newline,
    Whitespace,
    SemiColon,
    EOF,
    TypeName,
    Dollar,
    Comma,
    True,
    False,
    Void,
    Ptr, // a star
    LeftSquare,
    RightSquare,
    Ampersand,
    Deref, // a caret
    Comment,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
}

pub struct Lexer {
    input: String,
    position: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Lexer { input, position: 0 }
    }

    fn lex_comment(&mut self) -> Token {
        let start = self.position;
        while self.current_char() != '\n' {
            self.position += 1;
        }
        Token {
            token_type: TokenType::Comment,
            value: self.input[start..self.position].to_string(),
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        if self.position >= self.input.len() {
            return None;
        }

        let current_char = self.current_char();

        if current_char.is_alphabetic() {
            return Some(self.lex_identifier_or_keyword());
        } else if current_char == '/' && self.peek_char() == '/' {
            self.position += 2;
            return Some(self.lex_comment());
        } else if current_char.is_ascii_digit() {
            return Some(self.lex_number());
        } else if current_char == '"' {
            return Some(self.lex_string_literal());
        } else if current_char == '=' {
            if self.peek_char() == '=' {
                self.position += 2;
                return Some(Token {
                    token_type: TokenType::Comparison,
                    value: "==".to_string(),
                });
            } else {
                self.position += 1;
                return Some(Token {
                    token_type: TokenType::Assignment,
                    value: "=".to_string(),
                });
            }
        } else if current_char == '|' || current_char == '{' || current_char == '}' {
            self.position += 1;
            return Some(Token {
                token_type: TokenType::Block,
                value: "|".to_string(),
            });
        } else if current_char == '\n' {
            self.position += 1;
            return Some(Token {
                token_type: TokenType::Newline,
                value: "\n".to_string(),
            });
        } else if current_char == '+'
            || current_char == '-'
            || current_char == '*'
            || current_char == '/'
            || current_char == '%'
        {
            return Some(self.lex_operator());
        } else if current_char == '(' {
            self.position += 1;
            return Some(Token {
                token_type: TokenType::LeftParen,
                value: "(".to_string(),
            });
        } else if current_char == ')' {
            self.position += 1;
            return Some(Token {
                token_type: TokenType::RightParen,
                value: ")".to_string(),
            });
        } else if current_char == '>' {
            if self.peek_char() == '=' {
                self.position += 2;
                return Some(Token {
                    token_type: TokenType::Comparison,
                    value: ">=".to_string(),
                });
            } else {
                self.position += 1;
                return Some(Token {
                    token_type: TokenType::Comparison,
                    value: ">".to_string(),
                });
            }
        } else if current_char == '<' {
            if self.peek_char() == '=' {
                self.position += 2;
                return Some(Token {
                    token_type: TokenType::Comparison,
                    value: "<=".to_string(),
                });
            } else {
                if self.peek_char().is_alphabetic() {
                    self.position += 1;
                    let tp = self.lex_identifier_or_keyword().value;
                    self.position += 1;
                    return Some(Token {
                        token_type: TokenType::TypeName,
                        value: tp,
                    }); // skip the >
                }
                self.position += 1;
                return Some(Token {
                    token_type: TokenType::Comparison,
                    value: "<".to_string(),
                });
            }
        } else if current_char == '!' {
            if self.peek_char() == '=' {
                self.position += 2;
                return Some(Token {
                    token_type: TokenType::Comparison,
                    value: "!=".to_string(),
                });
            } else {
                self.position += 1;
                return Some(Token {
                    token_type: TokenType::Operator,
                    value: "!".to_string(),
                });
            }
        } else if current_char == ';' {
            self.position += 1;
            return Some(Token {
                token_type: TokenType::SemiColon,
                value: ";".to_string(),
            });
        } else if current_char == '$' {
            self.position += 1;
            return Some(Token {
                token_type: TokenType::Dollar,
                value: "$".to_string(),
            });
        } else if current_char == ',' {
            self.position += 1;
            return Some(Token {
                token_type: TokenType::Comma,
                value: ",".to_string(),
            });
        } else if current_char == '&' {
            self.position += 1;
            return Some(Token {
                token_type: TokenType::Ampersand,
                value: "&".to_string(),
            });
        } else if current_char == '[' {
            self.position += 1;
            return Some(Token {
                token_type: TokenType::LeftSquare,
                value: "[".to_string(),
            });
        } else if current_char == ']' {
            self.position += 1;
            return Some(Token {
                token_type: TokenType::RightSquare,
                value: "]".to_string(),
            });
        } else if current_char == '*' {
            self.position += 1;
            return Some(Token {
                token_type: TokenType::Ptr,
                value: "*".to_string(),
            });
        } else if current_char == '^' {
            self.position += 1;
            return Some(Token {
                token_type: TokenType::Deref,
                value: "^".to_string(),
            });
        } else if current_char == '\'' {
            let res = self.lex_char();
            if res.value == "\r" {
                // skip the \r
                self.position += 1;
                return Some(Token {
                    token_type: TokenType::Newline,
                    value: "\r".to_string(),
                });
            }
            return Some(res);
        } else {
            self.position += 1;
            return Some(Token {
                token_type: TokenType::Whitespace,
                value: current_char.to_string(),
            });
        }

        None
    }

    fn lex_char(&mut self) -> Token {
        self.position += 1; // Skip the opening quote
        let value = self.current_char().to_string();
        self.position += 1; // Skip the closing quote
        Token {
            token_type: TokenType::Char,
            value,
        }
    }

    fn lex_identifier_or_keyword(&mut self) -> Token {
        let start = self.position;
        while self.current_char().is_alphanumeric()
            || self.current_char() == '_'
            || self.current_char() == '*'
        {
            self.position += 1;
        }
        let value = self.input[start..self.position].to_string();
        let token_type = match value.as_str() {
            "let" | "if" | "else" | "while" | "func" | "return" | "struct" => TokenType::Keyword,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "void" => TokenType::Void,
            _ => TokenType::Identifier,
        };
        Token { token_type, value }
    }

    fn lex_number(&mut self) -> Token {
        let start = self.position;
        while self.current_char().is_ascii_digit() {
            self.position += 1;
        }
        Token {
            token_type: TokenType::Number,
            value: self.input[start..self.position].to_string(),
        }
    }

    fn lex_string_literal(&mut self) -> Token {
        self.position += 1; // Skip the opening quote
        let start = self.position;
        while self.current_char() != '"' {
            self.position += 1;
        }
        let value = self.input[start..self.position].to_string();
        self.position += 1; // Skip the closing quote
        Token {
            token_type: TokenType::StringLiteral,
            value,
        }
    }

    fn lex_operator(&mut self) -> Token {
        let value = self.current_char().to_string();
        self.position += 1;
        Token {
            token_type: TokenType::Operator,
            value,
        }
    }

    fn peek_char(&self) -> char {
        self.input.chars().nth(self.position + 1).unwrap_or('\0')
    }

    fn skip_whitespace(&mut self) {
        while self.current_char().is_whitespace() && self.current_char() != '\n' {
            self.position += 1;
        }
    }

    fn current_char(&self) -> char {
        self.input.chars().nth(self.position).unwrap_or('\0')
    }
}
