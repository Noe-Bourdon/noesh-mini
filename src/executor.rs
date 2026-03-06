
use nix::{libc::{FS_IOC32_SETVERSION, STDIN_FILENO, STDOUT_FILENO, dup2, fork}, sys::wait::{WaitStatus, waitpid}, unistd::{ForkResult, fork, getpid, getppid, read}}

use crate::parser::{AST, Command};

pub struct Execute {

}

impl Execute {
    /// ASTを真っ直ぐにする関数
    /// ASTから
    /// ```rust
    /// Pipe
    /// ├─ Command("echo", ["hello"])
    /// └─ Pipe
    ///      ├─ Command("grep", ["h"])
    ///      └─ Command("wc", ["-l"])
    /// ```
    /// ASTを真っ直ぐなリストに変換する
    /// ```rust
    /// [
    ///     Command(echo),
    ///     Command(grep),
    ///     Command(wc)
    /// ]
    /// ```
    fn flatten(&self) -> Vec<Command> {
        let mut cmds = Vec::new();
        self.flatten_into(s, &mut cmds);
        cmds
        
    }
    
    ///ASTから木構造を左から右に順番通りのVec(command)に変換する関数
    fn flatten_into(&mut self, ast: &AST, out: &mut Vec<Command>) {
        match ast {
            AST::Command(cmd) => out.push(cmd.clone()),
            AST::Pipe(left, right) => {
                self.flatten_into(left, out);
                self.flatten_into(right, out);
            }
        }
    }

    pub fn execute(&mut self, dnf: &mut Vec<Command>) {
        
        for cmd in dnf {
            println!("{:?}", cmd);

            match unsafe {fork() } {
                Ok(ForkResult::Parent { child }) => {
                   println!("Main")
                }

                Ok(ForkResult::Child) => unsafe {
                    
                }
                Err(_) => {
                    panic!("Fork failed.");
                }
            };
            
            match waitpid(h, None) {
                Ok(waitstatus) => {
                    println!("Child exied {:?}", waitstatus)
                }
                Err(_) => {
                    panic!("wait error");
                }
            }
        }
    }
}

