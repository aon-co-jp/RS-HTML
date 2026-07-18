//! HTML5トークン列挙型と、トークナイザ→DOM構築器の橋渡し役
//! `TokenSink`トレイト。設計思想は`html5ever`(Servo由来)の
//! `TokenSink`パターンを参考にした(コード自体は流用せず、一から実装)。
//! トークナイザが`Token`を生成する都度`TokenSink::process_token`へ
//! 渡す疎結合設計により、DOM構築ロジックをトークナイザから完全に分離する。

/// タグの属性(`name="value"`)。属性値なし(`<input disabled>`のような
/// boolean属性)は空文字列として保持する(HTML5仕様上、値なし属性は
/// 空文字列の値を持つものとして扱われる)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Doctype {
        name: String,
    },
    StartTag {
        name: String,
        attrs: Vec<Attribute>,
        self_closing: bool,
    },
    EndTag {
        name: String,
    },
    Comment(String),
    /// 連続する文字データ。html5everの`CharacterTokens`に相当。
    Characters(String),
    Eof,
}

/// トークナイザが生成した各`Token`を受け取る側のトレイト。
/// DOM木構築器はこれを実装し、トークナイザから完全に疎結合のまま
/// トークン列を消費できる。
pub trait TokenSink {
    fn process_token(&mut self, token: Token);
}

/// テスト・デバッグ用途の単純な`TokenSink`実装(受け取ったトークンを
/// そのままベクタに蓄積するだけ)。
#[derive(Default)]
pub struct CollectingSink {
    pub tokens: Vec<Token>,
}

impl TokenSink for CollectingSink {
    fn process_token(&mut self, token: Token) {
        self.tokens.push(token);
    }
}
