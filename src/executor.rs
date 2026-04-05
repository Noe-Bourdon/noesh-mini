

use nix::libc::{
    self, STDIN_FILENO, STDOUT_FILENO, 
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
    /// ASTを真っ直ぐなコマンドリストに変換する関数
    ///
    /// ```
    /// 入力AST:
    /// Pipe
    /// ├─ Command("echo", ["hello"])
    /// └─ Pipe
    ///      ├─ Command("grep", ["h"])
    ///      └─ Command("wc", ["-l"])
    ///
    /// 出力Vec<Command>:
    /// [
    ///     Command { name: "echo", args: ["hello"] },
    ///     Command { name: "grep", args: ["h"] },
    ///     Command { name: "wc", args: ["-l"] },
    /// ]
    /// ```
    pub fn flatten(&self, ast: &AST) -> Vec<Command> {
        let mut cmds = Vec::new();
        self.flatten_into(ast, &mut cmds);
        cmds
    }

    /// ASTから木構造を左から右に順番通りの Vec<Command> に変換する関数
    ///
    /// AST::Command -> そのまま Vec に追加
    /// AST::Pipe    -> 左右の子を順番に再帰的に展開
    fn flatten_into(&self, ast: &AST, out: &mut Vec<Command>) {
        match ast {
            AST::Command(cmd) => out.push(cmd.clone()),
            AST::Pipe(left, right) => {
                self.flatten_into(left, out);
                self.flatten_into(right, out);
            }
        }
    }

    /// 真っ直ぐなコマンドリストを受け取り、実行方法を分岐する関数
    pub fn run_commands(&mut self, cmds: &mut Vec<Command>) {
        match cmds.len() {
            0 => return,
            1 => self.execute(cmds),
            2 => Execute::pipe_execute(cmds),
            _ => panic!("not implemented"),
        }
    }

    /// コマンド実行機（Executor）が、コマンド一覧を順番に実行する関数
    ///
    /// 例: flatten された Vec<Command> の場合
    /// ```
    /// Command { name: "echo", args: ["hello"] }  →  echo hello
    /// Command { name: "grep", args: ["h"] }      →  grep h
    /// Command { name: "wc", args: ["-l"] }       →  wc -l
    /// ```
    /// この関数は上から順番に、1つずつ fork → execvp で実行します
    /// - 親プロセスは子の終了を待機
    /// - 子プロセスはコマンドに置き換えて実行
    pub fn execute(&mut self, cmds: &mut Vec<Command>) {
        for cmd in cmds {
            // デバッグ表示
            println!("{:?}", cmd);

            // コマンド名を CString に変換（C言語の execvp に渡すため）
            let bin = CString::new(cmd.name.clone()).unwrap();

            // 引数リストを CString に変換
            let mut args = Vec::new();
            args.push(bin.clone()); // argv[0] はコマンド名
            for arg in &cmd.args {
                args.push(CString::new(arg.as_str()).unwrap());
            }

            // fork で子プロセスを作成
            match fork() {
                // 親プロセスは子の終了を待つ
                Ok(ForkResult::Parent { child }) => {
                    match waitpid(child, None) {
                        Ok(status) => println!("Child exited {:?}", status),
                        Err(e) => eprintln!("waitpid error {:?}", e),
                    }
                }

                // 子プロセスは execvp でコマンドを実行
                Ok(ForkResult::Child) => {
                    execvp(&bin, &args).expect("failed exec."); // 成功すればここでプロセス置換
                    unsafe { libc::exit(1) };                   // exec 失敗時は終了
                }

                // fork が失敗した場合
                Err(_) => panic!("Fork failed."),
            };
        }
    }

    /// Command 構造体を execvp 用に変換するユーティリティ関数
    ///
    /// 入力:
    /// ```
    /// Command {
    ///     name: "echo".to_string(),
    ///     args: vec!["hello".to_string()]
    /// }
    /// ```
    ///
    /// 出力:
    /// ```
    /// bin  = CString::new("echo")          // コマンド名
    /// args = [CString::new("echo"),        // argv[0] はコマンド名
    ///         CString::new("hello")]       // argv[1..] はコマンドの引数
    /// ```
    ///
    /// execvp に渡す際に必要な形式に変換する
    /// - execvp は C の文字列配列を必要とするため CString に変換
    /// - argv[0] は慣例としてコマンド名を入れる
    fn convert_args(cmd: &Command) -> (CString, Vec<CString>) {
        let bin = CString::new(cmd.name.clone()).unwrap();
        let args: Vec<CString> = std::iter::once(bin.clone())
            .chain(cmd.args.iter().map(|a| CString::new(a.as_str()).unwrap()))
            .collect();
        (bin, args)
    }

    /// 2つのコマンドをパイプで接続して実行する関数
    ///
    /// 入力: cmds = [Command1, Command2]
    /// 例: ["echo hello", "grep h"]
    ///
    /// 処理イメージ:
    /// ```
    /// [child_one: cmds[0]]              [child_two: cmds[1]]
    ///      ┌─────┐ stdout                ┌─────┐ stdin
    ///      │echo │────dup2(write_fd)───> │grep │
    ///      │hello│                       │h    │
    ///      └─────┘                       └─────┘
    ///          │                             │
    ///          │ pipe                        │ pipe
    ///          └─────────read_fd─────────────┘
    ///
    /// ファイルディスクリプタの流れ:
    /// - child_one: stdout を write_fd に置き換え (dup2)
    /// - child_two: stdin を read_fd に置き換え (dup2)
    /// - 親プロセス: read_fd と write_fd は不要なので close
    /// ```
    /// この関数は fork で2つの子プロセスを作り、それぞれ execvp でコマンドを実行します
    /// - child_one: cmds[0] を実行
    /// - child_two: cmds[1] を実行
    /// - 親プロセスは両方の子が終了するまで waitpid で待機
    fn pipe_execute(cmds: &[Command]) {
        let (read_fd, write_fd) = pipe().unwrap();

        match fork().unwrap() {
            ForkResult::Child => {
                // child_one: stdout を write_fd に置き換え
                nix::unistd::dup2(write_fd, STDOUT_FILENO).expect("dup2 failed");

                // 不要なFDを閉じる
                nix::unistd::close(read_fd).expect("close failed");
                nix::unistd::close(write_fd).expect("close failed");

                let (bin, args) = Self::convert_args(&cmds[0]);
                execvp(&bin, &args).unwrap();
            }

            ForkResult::Parent { child: child_one } => {
                match fork().unwrap() {
                    ForkResult::Child => {
                        // child_two: stdin を read_fd に置き換え
                        nix::unistd::dup2(read_fd, STDIN_FILENO).expect("dup2 failed");

                        // 不要なFDを閉じる
                        nix::unistd::close(read_fd).expect("close failed");
                        nix::unistd::close(write_fd).expect("close failed");

                        let (bin, args) = Self::convert_args(&cmds[1]);
                        execvp(&bin, &args).unwrap();
                    }

                    ForkResult::Parent { child: child_two } => {
                        // 親プロセス: パイプFDを閉じ、子の終了を待機
                        unsafe {
                            libc::close(read_fd);
                            libc::close(write_fd);

                            waitpid(child_one, None).expect("wait failed");
                            waitpid(child_two, None).expect("wait failed");
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


