#[derive(Debug, Clone, PartialEq)]
///トークン
pub enum Token {
    Word(String), //単語
    Pipe,         // |
    And,          //　&&
    //バックグラウンドで実行も追加
}

#[derive(Debug, PartialEq)]
///レキサーの状態
enum LexerState {
    Nomarl,    //通常
    InWord,    //単語
    InNextAnd, //&の次が&&かを判定
}

#[derive(Debug)]
///管理状態
pub struct Lexer {
    pub parts: Vec<Token>,  //完成した単語を入れる箱
    _state: LexerState, //今のレキサーの状態
    position: usize,
    store: Vec<usize>, //単語の最初の位置を入れる箱
}

impl Lexer {
    pub fn new() -> Self {
        Lexer {
            parts: Vec::new(),
            _state: LexerState::Nomarl,
            position: 0,
            store: Vec::new(),
        }
    }

    ///positionを使って、cmdの文字をひとずつ進める関数
    fn new_state(&mut self, cmd: &str) -> Option<char> {
        let mut iter = cmd[self.position..].chars();
        let ch = iter.next()?;
        self.position += ch.len_utf8();
        Some(ch)
    }

    pub fn lexar_allocation(&mut self, cmd: &str) -> Result<Vec<Token>, String> {
        //lldbにて確認
        while self.position < cmd.len() {
            let ch = self.new_state(&cmd).unwrap();
            match self._state {
                LexerState::Nomarl => self.lexar_nomal(cmd, ch).unwrap(),
                LexerState::InWord => self.lexar_inword(cmd, ch).unwrap(),
                LexerState::InNextAnd => self.lexar_nextand(cmd, ch).unwrap(),
            }
        }

        //単語の終了判定
        if self._state == LexerState::InWord {
            let start = self.store.pop().unwrap();
            let word = &cmd[start..self.position];
            self.parts.push(Token::Word(word.to_string()));
        }
        Ok(self.parts.clone())
    }

    fn lexar_nomal(&mut self, _cmd: &str, ch: char) -> Result<(), String> {
        match ch {
            //何もしてないのpushしてるから説
            ch if ch.is_alphanumeric() => {
                self.store.push(self.position - ch.len_utf8());
                self._state = LexerState::InWord;
            }
            //パイプが来た場合パイプ決定
            '|' => self.parts.push(Token::Pipe),
            '&' => self._state = LexerState::InNextAnd,
            //空白の場合はなにもしない
            
            ch if ch.is_whitespace() => {
                
            }
            _ => panic!(""),
        }

        Ok(())
    }

    fn lexar_inword(&mut self, cmd: &str, ch: char) -> Result<(), String> {
        //から文字で終了
        if ch.is_whitespace() {
            if let Some(start) = self.store.pop() {
                let word = &cmd[start..self.position - ch.len_utf8()];
                self.parts.push(Token::Word(word.to_string()));
            }
            self._state = LexerState::Nomarl;
        } else {
            //文字列は継続
            self._state = LexerState::InWord;
        }
        Ok(())
    }

    fn lexar_nextand(&mut self, cmd: &str, ch: char) -> Result<(), String> {
        let mut iter = cmd.chars();
        if ch == '&' {
            iter.next();
            self.parts.push(Token::And);
            self._state = LexerState::Nomarl;
            self.store.push(self.position - ch.len_utf8());
        }
        Ok(())
    }
}

//テスト
#[cfg(test)]
mod lexer {
    use crate::lexer::{self, Lexer};

    #[test]
    fn test_pipe() {
        let mut lexer = Lexer::new();
        lexer.lexar_allocation("echo hello | grep h").unwrap();
        dbg!(&lexer);
        let e = vec![
            lexer::Token::Word("echo".into()),
            lexer::Token::Word("hello".into()),
            lexer::Token::Pipe,
            lexer::Token::Word("grep".into()),
            lexer::Token::Word("h".into()),
        ];

        assert_eq!(lexer.parts, e);
    }
}

//＆＆のテスト
