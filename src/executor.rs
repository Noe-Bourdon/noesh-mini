
use nix::{libc::{STDIN_FILENO, STDOUT_FILENO, dup2, fork}, sys::wait::waitpid, unistd::{ForkResult, fork, getpid, getppid, read}}

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
    /// 真っ直ぐなリストに変換する
    /// ```rust
    /// [
    ///     Command(echo),
    ///     Command(grep),
    ///     Command(wc)
    /// ]
    /// ```
    fn flatten(&self) -> Vec<Command>{
        let mut cmds = Vec::new();
        self.flatten_into(ast, &mut cmds);
        cmds
        
    }

    fn flatten_into(&mut self, ast: &AST, out: &mut Vec<Command>) {
        match ast {
            AST::Command(cmd) => out.push(cmd.clone()),
            AST::Pipe(left, right) => {
                self.flatten_into(left, out);
                self.flatten_into(right, out);
            }
        }
    }
    

    ///ASTを取得し振る舞い関数
    pub fn execute(&mut self, ast: AST,) {
        match ast {
            AST::Command(cmd) => self.run_command(cmd)
        }
    }

    fn run_command(&mut self, cmd: ) {
        
    }

    fn run_pipe(&mut self, left: AST, right: AST) {
        
    }

    fn execute_pipeline(&mut self, ) {
        
        // //プロセスの生成
        // match unsafe { fork() } {
        //     //親プロセス
        //     Ok(ForkResult::Parent { child }) => {
                
        //     }            
        // }

    }
}

