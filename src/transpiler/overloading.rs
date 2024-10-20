use super::type_checker::Type;
use crate::parser::*;
use crate::types::*;
use std::collections::HashMap;

pub type FunctionSignatures = HashMap<String, HashMap<String, Vec<Type>>>;

pub fn static_dispatch(ast: &mut AstNode) -> Result<(), String> {
    let mut function_counter: HashMap<String, usize> = HashMap::new();
    let mut function_signatures: FunctionSignatures = HashMap::new();

    // First pass: Count function definitions and store return types
    fn count_function_definitions(
        node: &AstNode,
        function_counter: &mut HashMap<String, usize>,
        function_signatures: &mut FunctionSignatures,
    ) {
        match node {
            AstNode::Function {
                name,
                arguments,
                return_type,
                ..
            } => {
                *function_counter.entry(name.clone()).or_insert(0) += 1;
                let arg_types: Vec<Type> = arguments
                    .iter()
                    .map(|(ty, _)| Type::from(ty.clone()))
                    .collect();
                function_signatures
                    .entry(name.clone())
                    .or_default()
                    .insert(name.clone(), arg_types);
            }
            AstNode::Block { statements } => {
                for stmt in statements {
                    count_function_definitions(stmt, function_counter, function_signatures);
                }
            }
            _ => {}
        }
    }

    // Check if functions with the same name have the same return type
    fn check_return_types(function_signatures: &FunctionSignatures) -> Result<(), String> {
        for (name, signatures) in function_signatures {
            if signatures.len() > 1 {
                let first_return_type = &signatures.values().next().unwrap();
                for (_, arg_types) in signatures {
                    if arg_types != *first_return_type {
                        return Err(format!(
                            "Function '{}' has multiple definitions with different return types",
                            name
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    // Second pass: Rename functions that have multiple definitions
    fn rename_functions(
        node: &mut AstNode,
        function_counter: &mut HashMap<String, usize>,
        function_signatures: &mut FunctionSignatures,
    ) {
        match node {
            AstNode::Function {
                name,
                arguments,
                body,
                return_type,
            } => {
                if let Some(&count) = function_counter.get(name) {
                    if count > 1 {
                        let arg_types: Vec<Type> = arguments
                            .iter()
                            .map(|(ty, _)| Type::from(ty.clone()))
                            .collect();
                        let new_name = format!("{}_{}", name, function_counter[name] - 1);
                        *function_counter.get_mut(name).unwrap() -= 1;
                        function_signatures
                            .entry(name.clone())
                            .or_default()
                            .insert(new_name.clone(), arg_types);
                        *name = new_name;
                    }
                }
                rename_functions(body, function_counter, function_signatures);
            }
            AstNode::Block { statements } => {
                for stmt in statements {
                    rename_functions(stmt, function_counter, function_signatures);
                }
            }
            _ => {}
        }
    }

    // Third pass: Rename function calls
    fn rename_function_calls(node: &mut AstNode, function_signatures: &FunctionSignatures) {
        match node {
            AstNode::FunctionCall { name, arguments } => {
                if let Some(signatures) = function_signatures.get(name) {
                    if signatures.len() > 1 {
                        let arg_types: Vec<Type> =
                            arguments.iter().map(|arg| arg.get_type()).collect();
                        for (new_name, sig_args) in signatures {
                            if *sig_args == arg_types {
                                *name = new_name.clone();
                                break;
                            }
                        }
                    }
                }
                for arg in arguments {
                    rename_function_calls(arg, function_signatures);
                }
            }
            AstNode::Block { statements } => {
                for stmt in statements {
                    rename_function_calls(stmt, function_signatures);
                }
            }
            AstNode::Function { body, .. } => {
                rename_function_calls(body, function_signatures);
            }
            AstNode::Return { value } => {
                rename_function_calls(value, function_signatures);
            }
            AstNode::Pointer { value } => {
                rename_function_calls(value, function_signatures);
            }
            AstNode::Dereference { value } => {
                rename_function_calls(value, function_signatures);
            }
            _ => {
                for child in node.children_mut() {
                    rename_function_calls(child, function_signatures);
                }
            }
        }
    }

    // First pass: Count function definitions and store return types
    count_function_definitions(ast, &mut function_counter, &mut function_signatures);

    // Check return types
    check_return_types(&function_signatures)?;

    // Second pass: Rename functions that have multiple definitions
    rename_functions(ast, &mut function_counter, &mut function_signatures);

    // Third pass: Rename function calls
    rename_function_calls(ast, &function_signatures);

    Ok(())
}
