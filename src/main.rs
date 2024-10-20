#![allow(unused, clippy::borrowed_box)]
#![feature(f128)]

// Module declarations
mod lex;
mod parser;
mod transpiler;
mod types;

// Use declarations
use lex::{Lexer, Token, TokenType};
use types::{Function, *};

// Macro definition
macro_rules! c_code {
    // Case with an identifier interpolation
    ($( $line:tt )* #$var:expr , $( $rest:tt )*) => {{
        let mut code = stringify!($($line)*);
        code.push_str(&format!("{}", $var));
        code.push_str(&c_code!($($rest)*));
        code
    }};
    // Base case: no variables, just a simple string
    ($($code:tt)*) => {{
        let code = stringify!($($code)*);
        code.replace("#", "")
    }};
}

// Main function
fn main() {
    cli();
}

// CLI function
fn cli() {
    let args: Vec<String> = std::env::args().collect();
    // 3 options: build, run, transpile
    if args.len() != 3 {
        println!("Usage: cargo run [build|run|transpile] <filename>");
        return;
    }
    let option = &args[1];
    let filename = &args[2];
    let input = std::fs::read_to_string(filename).expect("Failed to read the file");
    if option == "build" {
        transpile(input);
        std::process::Command::new("gcc")
            .arg("output.c")
            .arg("-o")
            .arg("output")
            .output()
            .expect("Failed to compile the C code");
        // delete the output.c file
        std::fs::remove_file("output.c").expect("Failed to delete the output.c file");
    } else if option == "run" {
        transpile(input);
        std::process::Command::new("gcc")
            .arg("output.c")
            .arg("-o")
            .arg("output")
            .output()
            .expect("Failed to compile the C code");
        std::process::Command::new("./output")
            .output()
            .expect("Failed to run the compiled program"); // Run the compiled program in the terminal
                                                           // delete the output.c file
        std::fs::remove_file("output.c").expect("Failed to delete the output.c file");
    } else if option == "transpile" {
        transpile(input);
    } else {
        println!("Invalid option: {}", option);
    }
}

fn transpile(input: String) {
    // Lexical analysis
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    while let Some(token) = lexer.next_token() {
        if token.token_type != TokenType::Newline {
            tokens.push(token);
        }
    }
    tokens.push(Token {
        token_type: TokenType::EOF,
        value: "".to_string(),
    });

    // Parsing
    let mut parser = parser::Parser::new(tokens);
    let mut ast = parser.parse_program();

    // Static dispatch
    let res = transpiler::overloading::static_dispatch(&mut ast);
    if let Err(e) = res {
        println!("Error: {}", e);
    }

    // Name resolution
    ast = transpiler::ir::resolve_names(&ast);

    // Type checking
    let mut type_checker = transpiler::type_checker::TypeChecker::new();
    type_checker.prelude(&ast);
    let res = type_checker.check(&ast);
    match res {
        Ok(t) => {
            // C code generation
            let mut c_code = transpiler::c_bindgen::c_bindgen_prelude()
                + &*transpiler::c_bindgen::c_bindgen(&ast, 0, false);
            std::fs::write("output.c", c_code).unwrap();
        }
        Err(e) => {
            println!("Type error: {}", e);
        }
    }
}
