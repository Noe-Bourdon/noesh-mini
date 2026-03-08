
use std::env::args;

use nix::libc::{
    FS_IOC32_SETVERSION, STDIN_FILENO, STDOUT_FILENO, dup2, exit, 
};

use nix::sys::wait::{
    waitpid,
    WaitStatus,
};

use nix::unistd::{
    ForkResult,
    fork,
    getpid,
    getppid,
    read,
    execvp,
};

use std::ffi::CString;

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
    fn flatten(&self, ast: &AST) -> Vec<Command> {
        let mut cmds = Vec::new();
        self.flatten_into(ast, &mut cmds);
        cmds
        
    }
    
    ///ASTから木構造を左から右に順番通りのVec(command)に変換する関数
    fn flatten_into(&self, ast: &AST, out: &mut Vec<Command>) {
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
                   match waitpid(child, None) {
                       Ok(waitstatus) => {
                            println!("Child exited {:?}", waitstatus);
                       }
                       Err(e) => {
                            eprintln!("waitpid error {:?}", e);
                       }
                   }
                }
                //CStringの使用
                Ok(ForkResult::Child) => unsafe {
                    execvp(&bin, &[&bin, &args]).expect("coconush error: failed exec.");
                    exit(0);
                }
                Err(_) => {
                    panic!("Fork failed.");
                }
            };
            }
        }
}

