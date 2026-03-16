

use nix::libc::{
    self, STDIN_FILENO,  close, dup2,
};

use nix::sys::wait::{
    waitpid,
};

use nix::unistd::{
    ForkResult, execvp, fork, pipe, 
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
    ///     Command(echo ["hello"]),
    ///     Command(grep ["h"]),
    ///     Command(wc ["-l"]),
    /// ]
    /// ```
    pub fn flatten(&self, ast: &AST) -> Vec<Command> {
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

    pub fn  run_commands(&mut self, cmds: &mut Vec<Command>) {
        match cmds.len() {
            0 => return,
            1 => self.execute(cmds),
            2 => Execute::pipe_execute(cmds),
            _ => panic!("not implemented")
        }
    }

    ///マンド実行機（Executor）が、コマンドの一覧を1つずつ実行する部分
    pub fn execute(&mut self, dnf: &mut Vec<Command>) {
        for cmd in dnf {
            println!("{:?}", cmd);

            //execvはC言語の関数なので変換しなければならない
            // name = "echo"
            // args = ["hello", "world"]
            let bin = CString::new(cmd.name.clone()).unwrap();

            let mut args = Vec::new();

            args.push(bin.clone()); //argv[0]

            for args_push in &cmd.args {
                args.push(CString::new(args_push.as_str()).unwrap());
            }

            //forkで子プロセスを作る
            match fork() {
                //親 -> waitpidで子が終わるのを待つ
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
                //子 -> execvpでコマンドに変更
                Ok(ForkResult::Child) => {
                    execvp(&bin, &args).expect("coconush error: failed exec.");
                    unsafe {libc::exit(1)};
                }
                Err(_) => {
                    panic!("Fork failed.");  
                }
            };
        }
    }

    fn pipe_execute(cmds: &Vec<Command>) {
        let (read_fg, write_fg) = pipe().unwrap();

        let cmd_one = &cmds[0];
        let cmd_two = &cmds[1];

        match  fork().unwrap() {
            ForkResult::Child => {
                unsafe {
                    dup2(write_fg, nix::libc::STDOUT_FILENO);
                    close(read_fg);

                    let bin = CString::new(cmd_one.name.clone()).unwrap();
                    let args: Vec<CString> = std::iter::once(bin.clone())
                        .chain(cmd_one.args.iter().map(|a| CString::new(a.as_str()).unwrap()))
                        .collect();

                    execvp(&bin, &args).unwrap();
                }
            }

            ForkResult::Parent { child: child_one } => {
                match fork().unwrap() {
                    ForkResult::Child => {
                        unsafe {
                            dup2(read_fg, STDIN_FILENO);
                            close(write_fg);

                            let bin = CString::new(cmd_two.name.clone()).unwrap();
                            let args: Vec<CString> = std::iter::once(bin.clone())
                                .chain(cmd_two.args.iter().map(|a| CString::new(a.as_str()).unwrap()))
                                .collect();

                            execvp(&bin, &args).unwrap(); 
                        }
                    }
                    ForkResult::Parent { child: child_two } => {
                        unsafe {
                            close(read_fg);
                            close(write_fg);

                            let _ = waitpid(child_one, None);
                            let _ = waitpid(child_two, None);
                        }
                    }
                }
            }
        }
    }
} 

//テスト
#[cfg(test)]
    #[test]      
    fn shell_test() {
        let mut executor = Execute{};
        let ast = AST::Command(Command {
            name: "echo".into(),
            args: vec!["name".into()],
        });

        let mut cmd = executor.flatten(&ast);
        executor.run_commands(&mut cmd);
    }


