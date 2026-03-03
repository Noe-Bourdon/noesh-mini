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
    name: String,
    args: Vec<String>,
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

    pub fn parser(&mut self) -> AST {
       self.parser_pipe()
    }

    /// 値が借りる関数
    /// 所有者は取らない
    /// トークンは消費しない
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// 値をcloneして新しいTokenの所有権を返す関数
    fn advance(&mut self) -> Option<Token> {
        if self.position < self.tokens.len() {
            let tok = self.tokens[self.position].clone();
            self.position += 1;
            Some(tok)
        } else {
            None
        }
    }

    ///パイプの場合の関数
    /// AST::Pipe (
    ///     Box::new(AST::Command
    ///     Box::new(AST::Command
    ///  )
    pub fn parser_pipe(&mut self) -> AST {
        let mut left = self.parser_command();

        while let Some(&Token::Pipe) = self.peek() {
           self.advance();
           let right = self.parser_command();
           left = AST::Pipe(Box::new(left), Box::new(right)) 
        }
        left
    }

    /// コマンド　Command {name: echo, args: ["hello"] }
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