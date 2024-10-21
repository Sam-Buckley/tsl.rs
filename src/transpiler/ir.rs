use super::type_checker::*;
use crate::parser::*;

// Typed IR - Values aren't stored in the IR, only the types.
#[allow(clippy::only_used_in_recursion)]
pub fn resolve_names(ast: &AstNode) -> AstNode {
    let mut scopes: Vec<String> = vec![];
    let mut scope_argnames: Vec<Vec<String>> = vec![];
    let mut statements: Vec<AstNode> = vec![];

    fn resolve_names_helper(
        ast: &AstNode,
        scopes: &mut Vec<String>,
        statements: &mut Vec<AstNode>,
        scope_argnames: &mut Vec<Vec<String>>,
    ) -> AstNode {
        match ast {
            AstNode::Block { statements: stmts } => {
                let new_statements: Vec<AstNode> = stmts
                    .iter()
                    .map(|stmt| resolve_names_helper(stmt, scopes, statements, scope_argnames))
                    .collect();
                AstNode::Block {
                    statements: new_statements,
                }
            }
            AstNode::Function {
                name,
                arguments,
                body,
                return_type,
            } => {
                scopes.push(name.clone());
                scope_argnames.push(arguments.iter().map(|arg| arg.0.clone()).collect());
                let new_body = resolve_names_helper(body, scopes, statements, scope_argnames);

                scopes.pop();
                scope_argnames.pop();

                AstNode::Function {
                    name: name.clone(),
                    arguments: arguments.clone(),
                    body: Box::new(new_body),
                    return_type: return_type.clone(),
                }
            }
            AstNode::Identifier { value }
                if scope_argnames.last().unwrap_or(&Vec::new()).contains(value) =>
            {
                AstNode::Identifier {
                    value: format!("{}::{}", scopes.last().unwrap_or(&String::new()), value),
                }
            }
            AstNode::Identifier { value } => {
                if let Some(scope) = scopes.last() {
                    if scope.is_empty() {
                        AstNode::Identifier {
                            value: value.clone(),
                        }
                    } else {
                        AstNode::Identifier {
                            value: format!("{}::{}", scope, value),
                        }
                    }
                } else {
                    AstNode::Identifier {
                        value: value.clone(),
                    }
                }
            }
            AstNode::BinaryOperation {
                operator,
                left,
                right,
            } => AstNode::BinaryOperation {
                operator: operator.clone(),
                left: Box::new(resolve_names_helper(
                    left,
                    scopes,
                    statements,
                    scope_argnames,
                )),
                right: Box::new(resolve_names_helper(
                    right,
                    scopes,
                    statements,
                    scope_argnames,
                )),
            },
            AstNode::Return { value } => AstNode::Return {
                value: Box::new(resolve_names_helper(
                    value,
                    scopes,
                    statements,
                    scope_argnames,
                )),
            },
            AstNode::If {
                condition,
                consequence,
                alternative,
            } => {
                let new_condition =
                    resolve_names_helper(condition, scopes, statements, scope_argnames);
                let new_consequence =
                    resolve_names_helper(consequence, scopes, statements, scope_argnames);
                let new_alternative = alternative
                    .as_ref()
                    .map(|alt| resolve_names_helper(alt, scopes, statements, scope_argnames));

                AstNode::If {
                    condition: Box::new(new_condition),
                    consequence: Box::new(new_consequence),
                    alternative: new_alternative.map(Box::new),
                }
            }
            AstNode::Assignment {
                value,
                variable,
                tp,
            } => AstNode::Assignment {
                variable: {
                    let scope = scopes.last().unwrap_or(&String::new()).clone();
                    if scope.is_empty() {
                        variable.clone()
                    } else {
                        // add variable name to scope_argnames
                        scope_argnames.last_mut().unwrap().push(variable.clone());

                        format!("{}::{}", scope, variable)
                    }
                },
                value: Box::new(resolve_names_helper(
                    value,
                    scopes,
                    statements,
                    scope_argnames,
                )),
                tp: tp.clone(),
            },
            AstNode::Variable { value } => {
                let scope = scopes.last().unwrap_or(&String::new()).clone();
                if scope.is_empty() {
                    AstNode::Variable {
                        value: value.clone(),
                    }
                } else {
                    AstNode::Variable {
                        value: format!("{}::{}", scope, value),
                    }
                }
            }
            AstNode::FunctionCall { name, arguments } => {
                //Keep the name the same, but arguments need to be resolved
                let new_args: Vec<AstNode> = arguments
                    .iter()
                    .map(|arg| resolve_names_helper(arg, scopes, statements, scope_argnames))
                    .collect();
                AstNode::FunctionCall {
                    name: name.clone(),
                    arguments: new_args,
                }
            }
            AstNode::Pointer { value } => AstNode::Pointer {
                value: Box::new(resolve_names_helper(
                    value,
                    scopes,
                    statements,
                    scope_argnames,
                )),
            },
            AstNode::Dereference { value } => AstNode::Dereference {
                value: Box::new(resolve_names_helper(
                    value,
                    scopes,
                    statements,
                    scope_argnames,
                )),
            },
            _ => ast.clone(),
        }
    }

    resolve_names_helper(ast, &mut scopes, &mut statements, &mut scope_argnames)
}
