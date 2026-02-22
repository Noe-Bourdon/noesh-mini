use std::io;

use crate::{lexer::Token, parser::Parser};

//ファイルをインポート
mod lexer;
mod parser;

//

fn main() {
    shell_loop();
}

fn shell_loop() {
    loop {
        match standard_input() {
            Ok(cmd) if !cmd.is_empty() => {
                let mut lex = lexer::Lexer::new();
                let tokens = match lex.lexar_allocation(&cmd) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("Lexer error {}", e);
                        continue;
                    }
                };

                //parser
                let mut parser = Parser::new(tokens);
                let ast = parser.parser();
                println!("{:?}", ast);
            }
            Err(e) => println!("{}", e),
            _ => return,
        }
    }
}

fn standard_input() -> Result<String, String> {
    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read line");
    Ok(buffer.trim().to_string())
}
