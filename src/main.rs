use std::io::{self, Read, Write};

use crate:: parser::Parser;

use nix::unistd::isatty;
use std::os::unix::io::AsRawFd;

//ファイルをインポート
mod lexer;
mod parser;
mod executor;

//

fn main() {
    let stdin_fd = std::io::stdin().as_raw_fd();

    //TTY
    let is_tyy = isatty(stdin_fd).unwrap();

    if is_tyy {
        //REPL
        shell_loop();
    } else {
        //パイプ処理に切り替え
        pipe_mode();
    }
}

fn shell_loop() {

    loop {
        let prompt = "noesh-mini";
        print!("\x1b[32m{prompt}\x1b[0m > ");
        std::io::stdout().flush().unwrap();
        
        match standard_input() {
            Ok(cmd) if !cmd.is_empty() => {

                //lexer
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

                // execute
                let mut execute = executor::Execute{};
                let mut cmds = execute.flatten(&ast);
                execute.run_commands(&mut cmds);
 
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

//パイプモード
fn pipe_mode() {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer).unwrap();

    if buffer.trim().is_empty() {
        return;
    }

    //lexer
    let mut lex = lexer::Lexer::new();
    let tokens = lex.lexar_allocation(&buffer).unwrap();

    //parser
    let mut parser = Parser::new(tokens);
    let ast = parser.parser();
    println!("{:?}", ast);

    // execute
    let mut execute = executor::Execute{};
    let mut cmds = execute.flatten(&ast);
    execute.run_commands(&mut cmds);
}
