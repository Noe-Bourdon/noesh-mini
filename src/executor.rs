
use nix::{libc::{STDIN_FILENO, STDOUT_FILENO, dup2}, sys::wait::waitpid, unistd::{ForkResult, fork, getpid, getppid}}

use crate::parser::AST;

pub struct Execute {

}

impl Execute {
    ///ASTを取得し振る舞い関数
    pub fn execute(&mut self, ast: AST,) {
        match ast {
            AST::Command(cmd) => self.run_command(cmd),
            AST::Pipe(left, right) => {
                self.execute(*left);
                self.execute(*right)
            },
        }
    }

    fn run_command(&mut self, cmd: ) {
        
    }

    fn run_pipe(&mut self, left: AST, right: AST) {
        
        let (read_fg, write_fg) = nix::unistd::pipe()?;
        
        //親fock 子exec
        //左: write_fgで書き込む ->  右read_fg読み取り
        //stdout -> write_fg -> pip -> read_fg -> stdin
        unsafe {
            //左
            
            let write = dup2(write_fg, STDOUT_FILENO);

            //右
            let read = dup2(read_fg, STDIN_FILENO);
        }
    }
}
