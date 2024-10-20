use super::type_checker::Type;
use crate::lex::*;
use crate::parser::*;

pub trait AsC {
    fn as_c(&self) -> String;
}

pub struct CType {
    pub name: String,
    pub is_pointer: bool,
    pub is_const: bool,
}

impl CType {
    pub fn new(name: String, is_pointer: bool, is_const: bool) -> Self {
        CType {
            name,
            is_pointer,
            is_const,
        }
    }
}

impl AsC for CType {
    fn as_c(&self) -> String {
        let mut result = self.name.clone();
        if self.is_pointer {
            result.push('*');
        }
        if self.is_const {
            result.push_str(" const");
        }
        result
    }
}

pub fn c_bindgen_prelude() -> String {
    // Read cargo_manifest_dir/src/transpiler/prelude.c
    let prelude = include_str!("prelude.c");
    prelude.to_string()
}

pub fn c_bindgen(ast: &AstNode, indent: usize, is_in_function: bool) -> String {
    let mut result = String::new();
    let indent_str = "    ".repeat(indent);

    match ast {
        AstNode::Block { statements } => {
            for stmt in statements {
                result.push_str(&c_bindgen(stmt, indent, is_in_function));
            }
        }
        AstNode::Function {
            name,
            arguments,
            body,
            return_type,
        } => {
            result.push_str(&format!(
                "{}{} {}({}) {{\n",
                indent_str,
                Type::from(return_type.clone()).as_c(),
                name,
                arguments
                    .iter()
                    .map(|(ty, name)| format!("{} {}", Type::from(ty.clone()).as_c(), name))
                    .collect::<Vec<String>>()
                    .join(", ")
            ));
            result.push_str(&c_bindgen(body, indent + 1, is_in_function));
            result.push_str(&format!("{}}}\n", indent_str));
        }
        AstNode::Identifier { value } => {
            result.push_str(&format!(
                "{}{}",
                indent_str,
                value.split("::").last().unwrap()
            ));
        }
        AstNode::BinaryOperation {
            operator,
            left,
            right,
        } => {
            result.push_str(&c_bindgen(left, 0, true));
            result.push_str(&operator.value);
            result.push_str(&c_bindgen(right, 0, true));
        }
        AstNode::Variable { value } => {
            result.push_str(&format!(
                "{}{}",
                indent_str,
                value.split("::").last().unwrap()
            ));
        }
        AstNode::Return { value } => {
            if **value == AstNode::Null {
                result.push_str(&format!("{}return;\n", indent_str));
            } else {
                result.push_str(&format!(
                    "{}return {};\n",
                    indent_str,
                    c_bindgen(value, 0, is_in_function)
                ));
            }
        }
        AstNode::If {
            condition,
            consequence,
            alternative,
        } => {
            result.push_str(&format!(
                "{}if ({}) {{\n",
                indent_str,
                c_bindgen(condition, 0, is_in_function)
            ));
            result.push_str(&c_bindgen(consequence, indent + 1, is_in_function));
            result.push_str(&format!("{}}}\n", indent_str));
            if let Some(else_body) = alternative {
                result.push_str(&format!("{}else {{\n", indent_str));
                result.push_str(&c_bindgen(else_body, indent + 1, is_in_function));
                result.push_str(&format!("{}}}\n", indent_str));
            }
        }
        AstNode::While { condition, body } => {
            result.push_str(&format!(
                "{}while ({}) {{\n",
                indent_str,
                c_bindgen(condition, 0, is_in_function)
            ));
            result.push_str(&c_bindgen(body, indent + 1, is_in_function));
            result.push_str(&format!("{}}}\n", indent_str));
        }
        AstNode::Assignment {
            value,
            tp,
            variable,
        } => {
            result.push_str(&format!(
                "{}{} {} = {};\n",
                indent_str,
                Type::from(tp.clone().unwrap()).as_c(),
                variable.split("::").last().unwrap(),
                c_bindgen(value, 0, is_in_function)
            ));
        }
        AstNode::FunctionCall { name, arguments } => {
            result.push_str(&format!(
                "{}{}({})",
                indent_str,
                name.split("::").last().unwrap(),
                arguments
                    .iter()
                    .map(|arg| c_bindgen(arg, 0, true))
                    .collect::<Vec<String>>()
                    .join(", ")
            ));
            if !is_in_function {
                result.push_str(";\n");
            }
        }
        AstNode::Number { value } => {
            result.push_str(&format!("{}{}", indent_str, value));
        }
        AstNode::String { value } => {
            result.push_str(&format!("{}\"{}\"", indent_str, value));
        }
        AstNode::Bool { value } => {
            result.push_str(&format!("{}{}", indent_str, value));
        }
        AstNode::Null => {
            result.push_str(&format!("{}Void", indent_str));
        }
        AstNode::Pointer { value } => {
            result.push_str(&format!(
                "{}&{}",
                indent_str,
                c_bindgen(value, 0, is_in_function)
            ));
        }
        AstNode::Dereference { value } => {
            result.push_str(&format!(
                "{}*{}",
                indent_str,
                c_bindgen(value, 0, is_in_function)
            ));
        }
        _ => {}
    }
    result
}
