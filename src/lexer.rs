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
/// `Lexer::new()` は以下の初期状態を用意する:
/// ```rust
/// Lexer {
///     parts: [],      // トークンに変換された単語を格納するリスト
///     state: Normal,  // 通常状態（クォート中などではない）
///     position: 0,    // 現在処理中の文字位置
///     store: [],      // 文字を一時的に貯めておくバッファ
/// }
/// ```
pub struct Lexer {
    pub parts: Vec<Token>,  //完成した単語を入れる箱
    _state: LexerState, //今のレキサーの状態
    position: usize, //　インデックス
    store: Vec<usize>, //単語の最初の位置を入れる箱
}

impl Lexer {
    // 初期化関数　コンストラクタ的
    pub fn new() -> Self {
        Lexer {
            //　字句解析中に分割された文字列パーツを入れる
            parts: Vec::new(),
            //　レキサーの状態を表す　最初は通常モード
            _state: LexerState::Nomarl,
            //　今のどの文字かを示すインデックス
            position: 0,
            //　一時的にトークンを作成するためのバッファー
            store: Vec::new(),
        }
    }

    /// 入力文字列`cmd`をposition(現在位置)に従って
    /// １文字ずつ読み取る関数
    /// レキサーが `cmd` をパースするとき、
    /// - `"echo hello"` のような文字列を
    /// - 先頭から `e` → `c` → `h` → `o` → ` ` → ...
    /// の順で読みたい。
    fn new_state(&mut self, cmd: &str) -> Option<char> {
        //　インデックスを見ながら１文字分けていく
        let mut iter = cmd[self.position..].chars();
        //　next関数で次の文字に読み取る
        let ch = iter.next()?;
        //　読み取った文字だけバイト数だけ進める
        self.position += ch.len_utf8();
        //　読み取った１文字を結果で返す
        Some(ch)
    }

    /// 入力文字列 `cmd` をトークン列に変換するレキサーのメイン関数
    /// 例えば入力:
    /// ```text
    /// echo hello | grep h
    /// ```
    /// 出力されるトークン列:
    /// ```rust
    /// [
    ///     Token::Word("echo"),
    ///     Token::Word("hello"),
    ///     Token::Pipe,
    ///     Token::Word("grep"),
    ///     Token::Word("h"),
    /// ]
    /// ```
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

    /// 通常状態での文字処理関数
    /// 
    ///  現在状態が `Nomarl` のときに呼ばれる。
    /// 文字に応じて次の動作を決定する。
    /// 処理のイメージ（例: `echo hello | grep h`）:
    /// - 英数字 → 単語開始として位置を記録、状態を `InWord` に
    /// - 空白 → 無視（単語の区切りは `InWord` 側で処理）
    /// ```
    /// 入力: "echo hello | grep h"
        ///状態遷移:
        ///[Nomal] --e--> [InWord] --c,h,o--> [InWord] --空白--> [Nomarl] 
        ///[Nomal] --h--> [InWord] ...
        ///[Nomal] --|--> Token::Pipe
    /// ```
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

    /// 単語読み取り中の文字処理関数
    /// 現在の状態が`InWord`のときに呼ばれる
    /// 空白が出たら単語の終了
    /// それまでの文字`Token::Word`とする
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
