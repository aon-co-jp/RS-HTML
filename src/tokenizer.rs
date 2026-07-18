//! HTML5トークナイザ。WHATWGのHTML Standardが定義する完全な状態機械
//! (80近い状態)ではなく、実用上重要な部分状態機械のサブセットを
//! 一から実装したもの——「一から開発して互換再現する」方針の第一段。
//! 設計上の参考(コードは流用せず思想のみ踏襲):
//! - `html5ever`の`TokenSink`パターン(疎結合なトークナイザ→DOM構築器)
//! - `cssparser`のライフタイム分離パーサー設計(将来のゼロコピー化を
//!   見据え、現段階では`String`ベースの単純な実装から始める)
//!
//! 対応済み: テキスト・開始タグ(引用符あり/なし属性値、
//! 自己終了タグ)・終了タグ・コメント・DOCTYPE・EOF。
//! 未対応(次段階): CDATA区間、文字参照(`&amp;`等)のデコード、
//! `<script>`/`<style>`の生テキストモード、不正入力に対する
//! パースエラー回復規則(仕様上のerror recovery)。

use crate::token::{Attribute, Token, TokenSink};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Data,
    TagOpen,
    EndTagOpen,
    TagName,
    BeforeAttributeName,
    AttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    MarkupDeclarationOpen,
    CommentStart,
    Comment,
    CommentEndDash,
    CommentEnd,
    Doctype,
    BeforeDoctypeName,
    DoctypeName,
}

pub struct Tokenizer<'sink, S: TokenSink> {
    sink: &'sink mut S,
    state: State,
    chars: Vec<char>,
    pos: usize,
    char_buf: String,
    current_tag_name: String,
    current_tag_self_closing: bool,
    current_tag_is_end: bool,
    current_attrs: Vec<Attribute>,
    current_attr_name: String,
    current_attr_value: String,
    comment_buf: String,
    doctype_name_buf: String,
}

impl<'sink, S: TokenSink> Tokenizer<'sink, S> {
    pub fn new(sink: &'sink mut S) -> Self {
        Self {
            sink,
            state: State::Data,
            chars: Vec::new(),
            pos: 0,
            char_buf: String::new(),
            current_tag_name: String::new(),
            current_tag_self_closing: false,
            current_tag_is_end: false,
            current_attrs: Vec::new(),
            current_attr_name: String::new(),
            current_attr_value: String::new(),
            comment_buf: String::new(),
            doctype_name_buf: String::new(),
        }
    }

    /// 入力文字列全体をトークナイズし、生成したトークンを都度
    /// `TokenSink`へ渡す。最後に必ず`Token::Eof`を1回発行する。
    pub fn run(mut self, input: &str) {
        self.chars = input.chars().collect();
        self.pos = 0;
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            self.step(c);
            self.pos += 1;
        }
        self.flush_characters();
        self.sink.process_token(Token::Eof);
    }

    fn flush_characters(&mut self) {
        if !self.char_buf.is_empty() {
            let text = std::mem::take(&mut self.char_buf);
            self.sink.process_token(Token::Characters(text));
        }
    }

    fn emit_start_or_end_tag(&mut self) {
        let name = std::mem::take(&mut self.current_tag_name);
        if self.current_tag_is_end {
            self.sink.process_token(Token::EndTag { name });
        } else {
            let attrs = std::mem::take(&mut self.current_attrs);
            self.sink.process_token(Token::StartTag {
                name,
                attrs,
                self_closing: self.current_tag_self_closing,
            });
        }
        self.current_tag_self_closing = false;
        self.current_tag_is_end = false;
    }

    fn commit_current_attr(&mut self) {
        if !self.current_attr_name.is_empty() {
            self.current_attrs.push(Attribute {
                name: std::mem::take(&mut self.current_attr_name),
                value: std::mem::take(&mut self.current_attr_value),
            });
        } else {
            self.current_attr_name.clear();
            self.current_attr_value.clear();
        }
    }

    fn step(&mut self, c: char) {
        match self.state {
            State::Data => {
                if c == '<' {
                    self.flush_characters();
                    self.state = State::TagOpen;
                } else {
                    self.char_buf.push(c);
                }
            }
            State::TagOpen => {
                if c == '/' {
                    self.state = State::EndTagOpen;
                } else if c == '!' {
                    self.state = State::MarkupDeclarationOpen;
                } else if c.is_ascii_alphabetic() {
                    self.current_tag_is_end = false;
                    self.current_tag_name.clear();
                    self.current_tag_name.push(c.to_ascii_lowercase());
                    self.state = State::TagName;
                } else {
                    // 不正な `<` の使い方: 仕様上はcharacter tokenへ
                    // フォールバックするが、ここでは簡略化しData状態へ戻す。
                    self.char_buf.push('<');
                    self.char_buf.push(c);
                    self.state = State::Data;
                }
            }
            State::EndTagOpen => {
                if c.is_ascii_alphabetic() {
                    self.current_tag_is_end = true;
                    self.current_tag_name.clear();
                    self.current_tag_name.push(c.to_ascii_lowercase());
                    self.state = State::TagName;
                } else {
                    self.state = State::Data;
                }
            }
            State::TagName => {
                if c.is_whitespace() {
                    self.state = State::BeforeAttributeName;
                } else if c == '/' {
                    self.state = State::SelfClosingStartTag;
                } else if c == '>' {
                    self.emit_start_or_end_tag();
                    self.state = State::Data;
                } else {
                    self.current_tag_name.push(c.to_ascii_lowercase());
                }
            }
            State::BeforeAttributeName => {
                if c.is_whitespace() {
                    // skip
                } else if c == '/' {
                    self.state = State::SelfClosingStartTag;
                } else if c == '>' {
                    self.emit_start_or_end_tag();
                    self.state = State::Data;
                } else {
                    self.current_attr_name.clear();
                    self.current_attr_value.clear();
                    self.current_attr_name.push(c.to_ascii_lowercase());
                    self.state = State::AttributeName;
                }
            }
            State::AttributeName => {
                if c.is_whitespace() {
                    self.state = State::BeforeAttributeName;
                    self.commit_current_attr();
                } else if c == '=' {
                    self.state = State::BeforeAttributeValue;
                } else if c == '/' {
                    self.commit_current_attr();
                    self.state = State::SelfClosingStartTag;
                } else if c == '>' {
                    self.commit_current_attr();
                    self.emit_start_or_end_tag();
                    self.state = State::Data;
                } else {
                    self.current_attr_name.push(c.to_ascii_lowercase());
                }
            }
            State::BeforeAttributeValue => {
                if c.is_whitespace() {
                    // skip
                } else if c == '"' {
                    self.state = State::AttributeValueDoubleQuoted;
                } else if c == '\'' {
                    self.state = State::AttributeValueSingleQuoted;
                } else if c == '>' {
                    self.commit_current_attr();
                    self.emit_start_or_end_tag();
                    self.state = State::Data;
                } else {
                    self.current_attr_value.push(c);
                    self.state = State::AttributeValueUnquoted;
                }
            }
            State::AttributeValueDoubleQuoted => {
                if c == '"' {
                    self.commit_current_attr();
                    self.state = State::AfterAttributeValueQuoted;
                } else {
                    self.current_attr_value.push(c);
                }
            }
            State::AttributeValueSingleQuoted => {
                if c == '\'' {
                    self.commit_current_attr();
                    self.state = State::AfterAttributeValueQuoted;
                } else {
                    self.current_attr_value.push(c);
                }
            }
            State::AttributeValueUnquoted => {
                if c.is_whitespace() {
                    self.commit_current_attr();
                    self.state = State::BeforeAttributeName;
                } else if c == '>' {
                    self.commit_current_attr();
                    self.emit_start_or_end_tag();
                    self.state = State::Data;
                } else {
                    self.current_attr_value.push(c);
                }
            }
            State::AfterAttributeValueQuoted => {
                if c.is_whitespace() {
                    self.state = State::BeforeAttributeName;
                } else if c == '/' {
                    self.state = State::SelfClosingStartTag;
                } else if c == '>' {
                    self.emit_start_or_end_tag();
                    self.state = State::Data;
                } else {
                    // 仕様上はパースエラーでBeforeAttributeNameへ再консume。
                    self.state = State::BeforeAttributeName;
                    self.step(c);
                }
            }
            State::SelfClosingStartTag => {
                if c == '>' {
                    self.current_tag_self_closing = true;
                    self.emit_start_or_end_tag();
                    self.state = State::Data;
                } else {
                    self.state = State::BeforeAttributeName;
                    self.step(c);
                }
            }
            State::MarkupDeclarationOpen => {
                // 簡略化: 次の2文字が"--"ならコメント、"DOCTYPE"(大小無視)
                // ならDOCTYPEとみなす。厳密な逐次状態遷移の代わりに、
                // 残り入力を軽く覗き見て判定する(第一段実装としての簡略化)。
                let rest: String = self.chars[self.pos..].iter().collect();
                if rest.starts_with("--") {
                    self.pos += 1; // 2つ目の'-'を読み飛ばす(1つ目はこのcで消費済み)
                    self.comment_buf.clear();
                    self.state = State::CommentStart;
                } else if rest.to_ascii_uppercase().starts_with("DOCTYPE") {
                    self.pos += "DOCTYPE".len() - 1;
                    self.doctype_name_buf.clear();
                    self.state = State::Doctype;
                } else {
                    // 未対応の宣言(CDATA等)はコメントとして飲み込む簡略挙動。
                    self.comment_buf.clear();
                    self.state = State::Comment;
                }
            }
            State::CommentStart => {
                self.state = State::Comment;
                self.step_comment(c);
            }
            State::Comment => {
                self.step_comment(c);
            }
            State::CommentEndDash => {
                if c == '-' {
                    self.state = State::CommentEnd;
                } else {
                    self.comment_buf.push('-');
                    self.state = State::Comment;
                    self.step_comment(c);
                }
            }
            State::CommentEnd => {
                if c == '>' {
                    let text = std::mem::take(&mut self.comment_buf);
                    self.sink.process_token(Token::Comment(text));
                    self.state = State::Data;
                } else if c == '-' {
                    self.comment_buf.push('-');
                } else {
                    self.comment_buf.push_str("--");
                    self.state = State::Comment;
                    self.step_comment(c);
                }
            }
            State::Doctype => {
                if c.is_whitespace() {
                    self.state = State::BeforeDoctypeName;
                } else if c == '>' {
                    self.emit_doctype();
                    self.state = State::Data;
                }
            }
            State::BeforeDoctypeName => {
                if c.is_whitespace() {
                    // skip
                } else if c == '>' {
                    self.emit_doctype();
                    self.state = State::Data;
                } else {
                    self.doctype_name_buf.push(c.to_ascii_lowercase());
                    self.state = State::DoctypeName;
                }
            }
            State::DoctypeName => {
                if c == '>' {
                    self.emit_doctype();
                    self.state = State::Data;
                } else {
                    self.doctype_name_buf.push(c.to_ascii_lowercase());
                }
            }
        }
    }

    fn step_comment(&mut self, c: char) {
        if c == '-' {
            self.state = State::CommentEndDash;
        } else {
            self.comment_buf.push(c);
        }
    }

    fn emit_doctype(&mut self) {
        let name = std::mem::take(&mut self.doctype_name_buf);
        self.sink.process_token(Token::Doctype { name });
    }
}

/// 便利関数: `input`をトークナイズし、生成された全トークンを
/// `sink`へ渡す。
pub fn tokenize<S: TokenSink>(input: &str, sink: &mut S) {
    Tokenizer::new(sink).run(input);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{CollectingSink, Token};

    fn tokenize_all(input: &str) -> Vec<Token> {
        let mut sink = CollectingSink::default();
        tokenize(input, &mut sink);
        sink.tokens
    }

    #[test]
    fn plain_text_becomes_a_single_characters_token_plus_eof() {
        let tokens = tokenize_all("hello world");
        assert_eq!(tokens, vec![Token::Characters("hello world".to_string()), Token::Eof]);
    }

    #[test]
    fn simple_start_and_end_tag_round_trip() {
        let tokens = tokenize_all("<p>hi</p>");
        assert_eq!(
            tokens,
            vec![
                Token::StartTag { name: "p".to_string(), attrs: vec![], self_closing: false },
                Token::Characters("hi".to_string()),
                Token::EndTag { name: "p".to_string() },
                Token::Eof,
            ]
        );
    }

    #[test]
    fn attributes_with_double_single_and_unquoted_values() {
        let tokens = tokenize_all(r#"<a href="https://example.com" class='btn' disabled>x</a>"#);
        let Token::StartTag { name, attrs, self_closing } = &tokens[0] else {
            panic!("expected a start tag token");
        };
        assert_eq!(name, "a");
        assert!(!self_closing);
        assert_eq!(
            attrs,
            &vec![
                Attribute { name: "href".to_string(), value: "https://example.com".to_string() },
                Attribute { name: "class".to_string(), value: "btn".to_string() },
                Attribute { name: "disabled".to_string(), value: String::new() },
            ]
        );
    }

    #[test]
    fn self_closing_tag_is_flagged() {
        let tokens = tokenize_all(r#"<img src="x.png"/>"#);
        assert_eq!(
            tokens[0],
            Token::StartTag {
                name: "img".to_string(),
                attrs: vec![Attribute { name: "src".to_string(), value: "x.png".to_string() }],
                self_closing: true,
            }
        );
    }

    #[test]
    fn comment_is_captured_verbatim() {
        let tokens = tokenize_all("<!-- a comment -->text");
        assert_eq!(
            tokens,
            vec![
                Token::Comment(" a comment ".to_string()),
                Token::Characters("text".to_string()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn doctype_html5_is_recognized() {
        let tokens = tokenize_all("<!DOCTYPE html>");
        assert_eq!(tokens, vec![Token::Doctype { name: "html".to_string() }, Token::Eof]);
    }

    #[test]
    fn full_document_shape_tokenizes_without_panicking() {
        let html = r#"<!DOCTYPE html>
<html lang="ja">
<head><title>Test</title></head>
<body><p class="a b">Hello, <b>world</b>!</p></body>
</html>"#;
        let tokens = tokenize_all(html);
        assert_eq!(tokens.last(), Some(&Token::Eof));
        assert!(tokens.iter().any(|t| matches!(t, Token::Doctype { name } if name == "html")));
        assert!(tokens.iter().any(|t| matches!(t, Token::StartTag { name, .. } if name == "html")));
        assert!(tokens.iter().any(|t| matches!(t, Token::EndTag { name } if name == "html")));
    }
}
