// This stores the type checker for the transpiler.
// Structs we will need: Type Registry, Type Checker, Type, Function, and TypeName.

use crate::lex::*;
use crate::parser::*;
use std::collections::HashMap;

use super::c_bindgen::AsC;

impl AsC for Type {
    fn as_c(&self) -> String {
        match self {
            Type::Integer => "int".to_string(),
            Type::Float => "float".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "String".to_string(),
            Type::Function => "void".to_string(),
            Type::Void => "void".to_string(),
            Type::DataTp(name) => name.clone(),
            Type::NotMentioned => "NotMentioned".to_string(),
            Type::Pointer(tp) => format!("{}*", tp.as_c()),
            Type::Char => "char".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,
    Float,
    Bool,
    String,
    Function,
    Void,
    DataTp(String),
    NotMentioned,
    Pointer(Box<Type>),
    Char,
}

impl From<&str> for Type {
    fn from(name: &str) -> Self {
        match name {
            "Int" => Type::Integer,
            "Float" => Type::Float,
            "Bool" => Type::Bool,
            "String" => Type::String,
            "Func" => Type::Function,
            "Void" => Type::Void,
            "NotMentioned" => Type::NotMentioned,
            "Int*" => Type::Pointer(Box::new(Type::Integer)),
            "Float*" => Type::Pointer(Box::new(Type::Float)),
            "Bool*" => Type::Pointer(Box::new(Type::Bool)),
            "String*" => Type::Pointer(Box::new(Type::String)),
            "Func*" => Type::Pointer(Box::new(Type::Function)),
            "Void*" => Type::Pointer(Box::new(Type::Void)),
            "Char" => Type::Char,
            "Char*" => Type::Pointer(Box::new(Type::Char)),
            _ => {
                if name.ends_with('*') {
                    Type::Pointer(Box::new(Type::DataTp(
                        name.trim_end_matches('*').to_owned(),
                    )))
                } else {
                    Type::DataTp(name.to_string())
                }
            } // Custom data type - might not exist :)
        }
    }
}

impl From<String> for Type {
    fn from(name: String) -> Self {
        match name.as_str() {
            "Int" => Type::Integer,
            "Float" => Type::Float,
            "Bool" => Type::Bool,
            "String" => Type::String,
            "Func" => Type::Function,
            "Void" => Type::Void,
            "NotMentioned" => Type::NotMentioned,
            "Int*" => Type::Pointer(Box::new(Type::Integer)),
            "Float*" => Type::Pointer(Box::new(Type::Float)),
            "Bool*" => Type::Pointer(Box::new(Type::Bool)),
            "String*" => Type::Pointer(Box::new(Type::String)),
            "Func*" => Type::Pointer(Box::new(Type::Function)),
            "Void*" => Type::Pointer(Box::new(Type::Void)),
            "Char" => Type::Char,
            "Char*" => Type::Pointer(Box::new(Type::Char)),
            _ => {
                if name.ends_with('*') {
                    Type::Pointer(Box::new(Type::DataTp(
                        name.trim_end_matches('*').to_owned(),
                    )))
                } else {
                    Type::DataTp(name)
                }
            } // Custom data type - might not exist :)
        }
    }
}

impl From<Type> for String {
    fn from(tp: Type) -> Self {
        match tp {
            Type::Integer => "Int".to_owned(),
            Type::Float => "Float".to_owned(),
            Type::Bool => "Bool".to_owned(),
            Type::String => "String".to_owned(),
            Type::Function => "Func".to_owned(),
            Type::Void => "Void".to_owned(),
            Type::DataTp(name) => name,
            Type::NotMentioned => "UnNamed".to_owned(),
            Type::Pointer(tp) => format!("{}*", String::from(*tp)),
            Type::Char => "Char".to_owned(),
        }
    }
}

pub struct TypeChecker {
    pub symbol_table: HashMap<String, Type>,
    pub function_table: HashMap<String, (Vec<Type>, Vec<Type>)>,
    scope_name: String,
    reached_eof: bool,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut symbol_table = HashMap::new();
        symbol_table.insert("true".to_string(), Type::Bool);
        symbol_table.insert("false".to_string(), Type::Bool);
        let function_table = HashMap::new();

        TypeChecker {
            symbol_table,
            function_table,
            scope_name: "".to_string(),
            reached_eof: false,
        }
    }

    pub fn prelude(&mut self, node: &AstNode) {
        match node {
            AstNode::Function {
                name,
                arguments,
                return_type,
                body: _,
            } => {
                let arg_types = arguments
                    .clone()
                    .iter()
                    .map(|(tp, _)| Type::from(tp.clone()))
                    .collect();
                let return_type = Type::from(return_type.clone());
                self.function_table
                    .insert(name.clone(), (arg_types, vec![return_type.clone()]));

                for (tp, argname) in arguments {
                    self.symbol_table
                        .insert(format!("{}::{}", name, argname), Type::from(tp.clone()));
                }
            }
            AstNode::Block { statements } => {
                for statement in statements {
                    self.prelude(statement);
                }
            }
            _ => {}
        }

        // log_char: string -> void

        self.function_table
            .insert("log_char".to_string(), (vec![Type::Char], vec![Type::Void]));

        //toChar: int -> char
        self.function_table.insert(
            "asChar".to_string(),
            (vec![Type::Integer], vec![Type::Char]),
        );

        // toInt: char -> int
        self.function_table
            .insert("asInt".to_string(), (vec![Type::Char], vec![Type::Integer]));

        // log: string -> void
        self.function_table
            .insert("log".to_string(), (vec![Type::String], vec![Type::Void]));

        // newStr -> string
        self.function_table.insert(
            "newStr".to_string(),
            (vec![], vec![Type::DataTp("Str".to_owned())]),
        );

        // log_int int -> void
        self.function_table.insert(
            "log_int".to_string(),
            (vec![Type::Integer], vec![Type::Void]),
        );

        // new_buffer: int -> string
        self.function_table.insert(
            "new_buffer".to_string(),
            (vec![Type::Integer], vec![Type::String]),
        );

        // boolToInt: bool -> int
        self.function_table.insert(
            "boolToInt".to_string(),
            (vec![Type::Bool], vec![Type::Integer]),
        );

        // scanf: string -> void
        self.function_table
            .insert("input".to_string(), (vec![Type::String], vec![Type::Void]));

        // appendStr: Str*, char* -> void
        self.function_table.insert(
            "appendStr".to_string(),
            (
                vec![
                    Type::Pointer(Box::new(Type::DataTp("Str".to_owned()))),
                    Type::String,
                ],
                vec![Type::Void],
            ),
        );

        self.symbol_table.insert("char_t".to_string(), Type::Char);
        self.symbol_table.insert("int_t".to_string(), Type::Integer);
        self.symbol_table.insert("float_t".to_string(), Type::Float);
        self.symbol_table.insert("bool_t".to_string(), Type::Bool);
        self.symbol_table
            .insert("string_t".to_string(), Type::String);
        self.symbol_table.insert("void_t".to_string(), Type::Void);
    }

    pub fn check(&mut self, node: &mut AstNode) -> Result<Type, String> {
        match node {
            AstNode::Number { value: _ } => Ok(Type::Integer),
            AstNode::String { value: _ } => Ok(Type::String),
            AstNode::BinaryOperation {
                operator,
                left,
                right,
            } => {
                let left_type = self.check(left)?;
                let right_type = self.check(right)?;

                match &*operator.value {
                    "+" | "-" | "*" | "/" | "%" => {
                        if left_type == Type::Integer && right_type == Type::Integer {
                            Ok(Type::Integer)
                        } else if let Type::Pointer(tp) = left_type.clone()
                            && right_type == Type::Integer
                        {
                            Ok(left_type)
                        } else if left_type == Type::String && right_type == Type::Integer {
                            Ok(Type::String)
                        } else {
                            Err(format!(
                                "cannot apply {:?} to {:?} and {:?}",
                                operator, left_type, right_type
                            ))
                        }
                    }
                    "==" | "!=" => {
                        if left_type == right_type {
                            Ok(Type::Bool)
                        } else {
                            Err(format!(
                                "cannot compare {:?} and {:?}",
                                left_type, right_type
                            ))
                        }
                    }
                    "&&" | "||" => {
                        if left_type == Type::Bool && right_type == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(format!(
                                "cannot apply {:?} to {:?} and {:?}",
                                operator, left_type, right_type
                            ))
                        }
                    }
                    "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                        if left_type == Type::Integer && right_type == Type::Integer {
                            Ok(Type::Bool)
                        } else {
                            Err(format!(
                                "cannot compare {:?} and {:?}",
                                left_type, right_type
                            ))
                        }
                    }
                    _ => Err("Unsupported operator".to_string()),
                }
            }
            AstNode::Assignment {
                variable,
                tp,
                value,
            } => {
                let expected = match tp {
                    Some(t) => Type::from(t.clone()),
                    None => Type::NotMentioned,
                };
                let value_type = self.check(value)?;

                // If type is NotMentioned, check the type of the variable

                if let Some(existing_type) = self.symbol_table.get(variable) {
                    if *existing_type != value_type && *existing_type != Type::NotMentioned {
                        return Err(format!(
                            "Type error: cannot assign {:?} to variable of type {:?}",
                            value_type, existing_type
                        ));
                    }
                }
                // Debug print all relevant variables
                // println!(
                //     "Variable: {:?}, Expected: {:?}, Value: {:?}",
                //     variable, expected, value_type
                // );
                if expected != Type::NotMentioned && expected != value_type {
                    self.symbol_table
                        .insert(variable.clone(), value_type.clone());
                    return Err(format!(
                        "Expected type {:?} but found {:?}",
                        expected, value_type
                    ));
                }

                self.symbol_table
                    .insert(variable.clone(), value_type.clone());
                Ok(value_type)
            }
            AstNode::Variable { value } => self
                .symbol_table
                .get(value)
                .cloned()
                .ok_or_else(|| format!("Undefined variable: {}", value)),
            AstNode::If {
                condition,
                consequence,
                alternative,
            } => {
                let condition_type = self.check(condition)?;

                if condition_type != Type::Bool {
                    return Err(format!(
                        "Type error: if condition must be a boolean, found {:?}",
                        condition_type
                    ));
                }

                self.check(consequence)?;
                if let Some(alt) = alternative {
                    self.check(alt)?;
                }

                Ok(Type::Void)
            }
            AstNode::Function {
                name,
                arguments,
                return_type,
                body,
            } => {
                let mut errors = Vec::new();
                let mut return_found = false;

                // Add function arguments to symbol table
                for (tp, argname) in arguments.iter() {
                    self.symbol_table
                        .insert(format!("{}::{}", name, argname), Type::from(tp.clone()));
                }

                fn check_return_statements(
                    node: &mut AstNode,
                    expected_return_type: &Type,
                    func_name: &str,
                    type_checker: &mut TypeChecker,
                    errors: &mut Vec<String>,
                    return_found: &mut bool,
                ) {
                    match node {
                        AstNode::Return { ref mut value } => match type_checker.check(value) {
                            Ok(ret_type) => {
                                if ret_type != *expected_return_type {
                                    errors.push(format!(
                                        "Function '{}' returns {:?} but declared as {:?}",
                                        func_name, ret_type, expected_return_type
                                    ));
                                }
                                *return_found = true;
                            }
                            Err(e) => errors.push(e),
                        },
                        AstNode::If {
                            condition,
                            consequence,
                            alternative,
                        } => {
                            check_return_statements(
                                condition,
                                expected_return_type,
                                func_name,
                                type_checker,
                                errors,
                                return_found,
                            );
                            check_return_statements(
                                consequence,
                                expected_return_type,
                                func_name,
                                type_checker,
                                errors,
                                return_found,
                            );
                            if let Some(alt) = alternative {
                                check_return_statements(
                                    alt,
                                    expected_return_type,
                                    func_name,
                                    type_checker,
                                    errors,
                                    return_found,
                                );
                            }
                        }
                        AstNode::While { condition, body } => {
                            check_return_statements(
                                condition,
                                expected_return_type,
                                func_name,
                                type_checker,
                                errors,
                                return_found,
                            );
                            check_return_statements(
                                body,
                                expected_return_type,
                                func_name,
                                type_checker,
                                errors,
                                return_found,
                            );
                        }
                        AstNode::Block { statements } => {
                            for statement in statements {
                                check_return_statements(
                                    statement,
                                    expected_return_type,
                                    func_name,
                                    type_checker,
                                    errors,
                                    return_found,
                                );
                            }
                        }
                        AstNode::Function {
                            name,
                            arguments,
                            return_type,
                            body,
                        } => {
                            let arg_types = arguments
                                .iter()
                                .map(|(tp, _)| Type::from(tp.clone()))
                                .collect();
                            let return_type = Type::from(return_type.clone());
                            type_checker
                                .function_table
                                .insert(name.clone(), (arg_types, vec![return_type.clone()]));
                            check_return_statements(
                                body,
                                &return_type,
                                name,
                                type_checker,
                                errors,
                                return_found,
                            );
                        }
                        AstNode::Assignment {
                            variable,
                            tp,
                            value,
                        } => {
                            let expected = match tp {
                                Some(t) => Type::from(t.clone()),
                                None => Type::NotMentioned,
                            };
                            let value_type_res = type_checker.check(value);

                            // if error, panic
                            let value_type = match value_type_res {
                                Ok(tp) => tp,
                                Err(e) => panic!("Error: {}", e),
                            };

                            if let Some(existing_type) = type_checker.symbol_table.get(variable) {
                                if *existing_type != value_type {
                                    errors.push(format!(
                                        "Type error: cannot assign {:?} to variable of type {:?}",
                                        value_type, existing_type
                                    ));
                                }
                            }
                            if expected != Type::NotMentioned && expected != value_type {
                                type_checker
                                    .symbol_table
                                    .insert(variable.clone(), value_type.clone());
                                errors.push(format!(
                                    "Expected type {:?} but found {:?}",
                                    expected,
                                    value_type.clone()
                                ));
                            }

                            type_checker
                                .symbol_table
                                .insert(variable.clone(), value_type.clone());
                        }
                        _ => match type_checker.check(node) {
                            Ok(_) => {}
                            Err(e) => errors.push(e),
                        },
                    }
                }

                check_return_statements(
                    body,
                    &Type::from(return_type.clone()),
                    name,
                    self,
                    &mut errors,
                    &mut return_found,
                );

                if !return_found {
                    errors.push(format!(
                        "Function '{}' must have a return statement",
                        name.split("_").collect::<Vec<&str>>()[0]
                    ));
                }

                if errors.is_empty() {
                    let arg_types = arguments
                        .iter()
                        .map(|(tp, _)| Type::from(tp.clone()))
                        .collect();
                    let return_type = Type::from(return_type.clone());
                    self.function_table
                        .insert(name.clone(), (arg_types, vec![return_type.clone()]));
                    Ok(return_type)
                } else {
                    Err(errors.join("\n"))
                }
            }
            AstNode::Return { value } => {
                let return_type = self.check(value)?;
                Ok(return_type)
            }
            AstNode::While { condition, body } => {
                let condition_type = self.check(condition)?;

                if condition_type != Type::Bool {
                    return Err(format!(
                        "Type error: while condition must be a boolean, found {:?}",
                        condition_type
                    ));
                }

                self.check(body)?;
                Ok(Type::Void)
            }
            AstNode::Block { statements } => {
                let mut errors = Vec::new();
                for statement in statements {
                    match self.check(statement) {
                        Ok(_) => {}
                        Err(e) => errors.push(e),
                    }
                }
                if errors.is_empty() {
                    Ok(Type::Void)
                } else {
                    Err(errors.join("\n"))
                }
            }
            AstNode::Identifier { value } => self
                .symbol_table
                .get(value)
                .cloned()
                .ok_or_else(|| format!("Undefined variable: {}", value)),
            AstNode::FunctionCall { name, arguments } => {
                // If function is printf, we don't need to check the arguments
                if name == "newStr" {
                    return Ok(Type::Pointer(Box::new(Type::DataTp("Str".to_owned()))));
                } else if name == "new_buffer" {
                    return Ok(Type::String);
                } else if name == "appendStr" {
                    return Ok(Type::Void);
                }
                if self.reached_eof {
                    Ok(Type::Void)
                } else if let Some((arg_types, return_types)) =
                    self.function_table.get(name).cloned()
                {
                    if arg_types.len() != arguments.len() {
                        return Err(format!(
                            "Function {} Expected {} arguments but found {}",
                            name,
                            arg_types.len(),
                            arguments.len()
                        ));
                    }
                    for (arg, expected) in arguments.iter_mut().zip(arg_types.iter()) {
                        let actual = self.check(arg)?;
                        if actual != *expected {
                            return Err(format!(
                                "{} Expected argument of type {:?} but found {:?}",
                                name, expected, actual
                            ));
                        }
                    }

                    Ok(return_types[0].clone())
                } else {
                    Err(format!("Undefined function: {}", name))
                }
            }
            AstNode::Char { value: _ } => Ok(Type::Char),
            AstNode::Bool { value: _ } => Ok(Type::Bool),
            AstNode::Null => Ok(Type::Void),
            AstNode::Eof => {
                self.reached_eof = true;
                Ok(Type::Void)
            }
            AstNode::Pointer { value } => Ok(Type::Pointer(Box::new(self.check(value)?))),
            AstNode::Dereference { value } => {
                match self.check(value)? {
                    Type::Pointer(tp) => {
                        // If type is a String pointer, return a char
                        if *tp == Type::DataTp("Str".to_owned()) {
                            Ok(Type::Char)
                        } else {
                            Ok(*tp)
                        }
                    }
                    Type::String => Ok(Type::Char),
                    _ => Err("Cannot dereference non-pointer type".to_string()),
                }
            }
            AstNode::Comment { value: _ } => Ok(Type::Void),
            _ => Err(format!("Unsupported node: {:?}", node)),
        }
    }
}
