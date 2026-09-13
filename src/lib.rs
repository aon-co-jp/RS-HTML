//! RHTML5 — HTML5相当のブラウザエンジン級パーサー/DOM実装を、既存
//! ブラウザエンジンのコードを一切流用せず一から開発するプロジェクト
//! (概念構想は`RHTML5/RCSS3/RTypeScript/RBootStrap`構想の一部、
//! 2026-07-18)。将来的にPoem上でのSSR(サーバー側でDOM木構築→
//! スタイル計算→HTML文字列レスポンス)を見据える。
//!
//! ## 現状(第一段)
//! トークナイザ(`tokenizer`モジュール)のみ。DOM木構築・カスケード
//! スタイル計算・レイアウトエンジンは未着手。

pub mod dom;
pub mod token;
pub mod tokenizer;

pub use dom::{parse_document, serialize_node, Document, Element, Node, TreeBuilder};
pub use token::{Attribute, CollectingSink, Token, TokenSink};
pub use tokenizer::{tokenize, Tokenizer};
