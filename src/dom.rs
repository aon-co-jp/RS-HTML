//! DOM木構築器。`tokenizer`が生成する`Token`列を`TokenSink`として
//! 受け取り、実際のノード木(`Document`/`Node`)を組み立てる。
//!
//! HTML5仕様の本格的な「ツリー構築アルゴリズム」(insertion mode・
//! adoption agency algorithm等)は再現しない——開いている要素の
//! スタックを使う簡略化版(素直なネスト構造のHTMLであれば正しく
//! 木を構築できる、不正なネスト・省略可能終了タグの自動補完等は
//! 次段階の課題として明記)。

use crate::token::{Attribute, Token, TokenSink};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Element(Element),
    Text(String),
    Comment(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub tag_name: String,
    pub attrs: Vec<Attribute>,
    pub children: Vec<Node>,
}

impl Element {
    fn new(tag_name: String, attrs: Vec<Attribute>) -> Self {
        Self { tag_name, attrs, children: Vec::new() }
    }

    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|a| a.name == name).map(|a| a.value.as_str())
    }
}

/// HTML5の"void elements"(終了タグを持たない要素)。トークナイザは
/// `<br>`のような自己終了記法(`/`)無しの記述でも`self_closing`を
/// falseのまま返すため、ツリー構築側でこの一覧を使って「子要素を
/// 持たず、開いている要素スタックにも積まない」ことを判定する。
const VOID_ELEMENTS: &[&str] =
    &["area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"];

fn is_void_element(tag_name: &str) -> bool {
    VOID_ELEMENTS.contains(&tag_name)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Document {
    pub doctype: Option<String>,
    pub children: Vec<Node>,
}

impl Document {
    /// 文書全体をHTML文字列へシリアライズする。トークナイズ時に
    /// 失われる情報(属性値の引用符の種類、自己終了記法の有無等)は
    /// 正規化された形(常にdouble quote、void要素は`<tag ...>`のみ)
    /// で出力するため、入力と出力がバイト単位で一致するとは限らない
    /// (構造的に等価であることを目的とする)。
    pub fn to_html(&self) -> String {
        let mut out = String::new();
        if let Some(name) = &self.doctype {
            out.push_str(&format!("<!DOCTYPE {name}>"));
        }
        for child in &self.children {
            serialize_node(child, &mut out);
        }
        out
    }
}

/// 単一の`Node`をHTML文字列へ直列化する(`Document::to_html`が文書
/// 全体に対して行うのと同じ規則を、`Document`を経由せず単一ノードに
/// 対して使いたい呼び出し側——たとえばRS-Reactの`VNode`木をSSR時に
/// 直接HTML化する用途——のために公開する)。
pub fn serialize_node(node: &Node, out: &mut String) {
    match node {
        Node::Text(text) => out.push_str(text),
        Node::Comment(text) => out.push_str(&format!("<!--{text}-->")),
        Node::Element(el) => {
            out.push('<');
            out.push_str(&el.tag_name);
            for attr in &el.attrs {
                if attr.value.is_empty() {
                    out.push_str(&format!(" {}", attr.name));
                } else {
                    out.push_str(&format!(" {}=\"{}\"", attr.name, attr.value));
                }
            }
            out.push('>');
            if !is_void_element(&el.tag_name) {
                for child in &el.children {
                    serialize_node(child, out);
                }
                out.push_str(&format!("</{}>", el.tag_name));
            }
        }
    }
}

/// 開いている要素のスタックを使ってトークン列からDOM木を組み立てる
/// `TokenSink`実装。
#[derive(Default)]
pub struct TreeBuilder {
    document: Document,
    open_stack: Vec<Element>,
}

impl TreeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// トークナイズ完了(`Token::Eof`受信)後に呼び、組み立てた
    /// `Document`を取り出す。スタックに閉じられていない要素が
    /// 残っていた場合は、そのままdocumentの子として畳み込む
    /// (省略された終了タグの自動補完、実ブラウザの寛容な挙動を
    /// 簡略化して再現)。
    pub fn finish(mut self) -> Document {
        while let Some(el) = self.open_stack.pop() {
            self.insert_node(Node::Element(el));
        }
        self.document
    }

    fn insert_node(&mut self, node: Node) {
        if let Some(parent) = self.open_stack.last_mut() {
            parent.children.push(node);
        } else {
            self.document.children.push(node);
        }
    }
}

impl TokenSink for TreeBuilder {
    fn process_token(&mut self, token: Token) {
        match token {
            Token::Doctype { name } => {
                self.document.doctype = Some(name);
            }
            Token::StartTag { name, attrs, self_closing } => {
                let element = Element::new(name.clone(), attrs);
                if self_closing || is_void_element(&name) {
                    self.insert_node(Node::Element(element));
                } else {
                    self.open_stack.push(element);
                }
            }
            Token::EndTag { name } => {
                // スタックの中からnameに一致する最も新しい要素を探し、
                // それより内側にある未閉じの要素もろとも畳み込む
                // (不正なネスト・タグ閉じ忘れに対する簡略化した回復)。
                if let Some(pos) = self.open_stack.iter().rposition(|el| el.tag_name == name) {
                    while self.open_stack.len() > pos {
                        let el = self.open_stack.pop().unwrap();
                        self.insert_node(Node::Element(el));
                    }
                }
                // 一致する開始タグが無い終了タグは無視する(仕様上も
                // パースエラーとして無視されるケースの簡略化)。
            }
            Token::Characters(text) => {
                self.insert_node(Node::Text(text));
            }
            Token::Comment(text) => {
                self.insert_node(Node::Comment(text));
            }
            Token::Eof => {
                // finish()で処理するため、ここでは何もしない。
            }
        }
    }
}

/// 便利関数: HTML文字列を直接`Document`へパースする
/// (`tokenizer::tokenize` + `TreeBuilder`を1回で行う)。
pub fn parse_document(input: &str) -> Document {
    let mut builder = TreeBuilder::new();
    crate::tokenizer::tokenize(input, &mut builder);
    builder.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_nested_element_builds_correct_tree() {
        let doc = parse_document("<p>Hello, <b>world</b>!</p>");
        assert_eq!(doc.children.len(), 1);
        let Node::Element(p) = &doc.children[0] else { panic!("expected element") };
        assert_eq!(p.tag_name, "p");
        assert_eq!(p.children.len(), 3);
        assert_eq!(p.children[0], Node::Text("Hello, ".to_string()));
        let Node::Element(b) = &p.children[1] else { panic!("expected element") };
        assert_eq!(b.tag_name, "b");
        assert_eq!(b.children, vec![Node::Text("world".to_string())]);
        assert_eq!(p.children[2], Node::Text("!".to_string()));
    }

    #[test]
    fn void_elements_have_no_children_and_are_not_pushed_onto_the_stack() {
        let doc = parse_document(r#"<div>before<img src="x.png">after</div>"#);
        let Node::Element(div) = &doc.children[0] else { panic!("expected element") };
        // img自体は子要素を持たず、直後のテキストがdivの直接の子になる
        // (imgの子になってしまっていないことを確認)。
        assert_eq!(div.children.len(), 3);
        assert!(matches!(&div.children[1], Node::Element(img) if img.tag_name == "img" && img.children.is_empty()));
        assert_eq!(div.children[2], Node::Text("after".to_string()));
    }

    #[test]
    fn doctype_is_captured_on_the_document() {
        let doc = parse_document("<!DOCTYPE html><html></html>");
        assert_eq!(doc.doctype.as_deref(), Some("html"));
    }

    #[test]
    fn unclosed_elements_are_folded_in_at_eof() {
        // 終了タグを一切書かない不正な入力でもpanicせず、開いている
        // 要素がdocumentの子として畳み込まれることを確認する。
        let doc = parse_document("<div><p>text");
        assert_eq!(doc.children.len(), 1);
        let Node::Element(div) = &doc.children[0] else { panic!("expected element") };
        assert_eq!(div.tag_name, "div");
        let Node::Element(p) = &div.children[0] else { panic!("expected element") };
        assert_eq!(p.tag_name, "p");
        assert_eq!(p.children, vec![Node::Text("text".to_string())]);
    }

    #[test]
    fn serializing_a_parsed_document_round_trips_structurally() {
        let html = r#"<p class="a">Hello, <b>world</b>!</p>"#;
        let doc = parse_document(html);
        assert_eq!(doc.to_html(), html);
    }

    #[test]
    fn void_element_serializes_without_a_closing_tag() {
        let doc = parse_document(r#"<img src="x.png">"#);
        assert_eq!(doc.to_html(), r#"<img src="x.png">"#);
    }
}
