use crate::lexer::Token;
/// ASTのパイプの場合の設計図
/// AST::Pipe (
///     Box::new(AST::Command(Command {name: echo, args: ["hello"] })),
///     Box::new(AST::Command(Command {name: grep, args: ["h"] })),
/// )　
/// 

/// コマンド Command {name: echo, args: ["hello"] }
#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
}

/// AST::Pipe (
///     Box::new(AST::Command
///     Box::new(AST::Command
///  )
#[derive(Debug, Clone)]
pub enum AST {
    Command(Command),
    Pipe(Box<AST>, Box<AST>),
}

#[derive(Debug, Clone)]
pub struct Parser {
    pub tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    /// パーサーのエントリポイント
    /// トークン列を解析してASTを生成
    pub fn parser(&mut self) -> AST {
       self.parser_pipe()
    }

    /// 値が借りる関数
    /// 所有者は取らない
    /// トークンは消費しない
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// 次のトークンを取得して進める関数
    /// トークンを`clone`して所有権を返す
    /// 取得後は位置を１進める(トークンは消費する)
    fn advance(&mut self) -> Option<Token> {
        if self.position < self.tokens.len() {
            let tok = self.tokens[self.position].clone();
            self.position += 1;
            Some(tok)
        } else {
            None
        }
    }

    /// パイプ構文を解析する関数
    ///
    /// 入力トークン列にパイプが含まれる場合、
    /// 左右のコマンドをネストしたAST::Pipeで表現する。
    ///
    /// 例: `echo hello | grep h`
    /// 入力トークン: [Word("echo"), Word("hello"), Pipe, Word("grep"), Word("h")]
    /// ```
    /// 出力AST:
    /// AST::Pipe(
    ///     Box::new(AST::Command("echo", ["hello"])),
    ///     Box::new(AST::Command("grep", ["h"]))
    /// )
    /// ```
    pub fn parser_pipe(&mut self) -> AST {
        // 左側のコマンドを解析
        let mut left = self.parser_command();

        while let Some(&Token::Pipe) = self.peek() {
           self.advance();
           let right = self.parser_command();
           left = AST::Pipe(Box::new(left), Box::new(right)) 
        }
        left
    }

    /// 単一コマンドを解析する関数
    ///
    /// ```
    /// 例: `echo hello`
    /// 入力トークン: [Word("echo"), Word("hello")]
    /// 出力AST: AST::Command(Command { name: "echo", args: ["hello"] })
    /// ```
    pub fn parser_command(&mut self) -> AST {
        let name = match self.advance() {
            Some(Token::Word(w)) => w,
            _ => panic!("無効な値"),
        };

        let mut args = Vec::new();
        while let Some(&Token::Word(_)) = self.peek() {
            if let Some(Token::Word(w)) = self.advance()  {
                args.push(w);
            }
        }
        AST::Command(Command { name, args})
    }


}

///テスト AST構造になっているか
#[cfg(test)]
mod parser {
    use crate::lexer::Token;
    use crate::parser::{ Parser};

    #[test]
    fn test_pipe_parser() {
        let tokens = vec![
            Token::Word("echo".to_string()),
            Token::Word("hello".to_string()),
            Token::Pipe,
            Token::Word("grep".to_string()),
            Token::Word("h".to_string()),
        ];

        let mut parser = Parser::new(tokens);

        let ast = parser.parser_pipe();
        dbg!(&ast);
        println!("{:#?}",ast);
        
    }
    
}