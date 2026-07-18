# rhtml5

HTML5相当のブラウザエンジン級パーサー/DOM実装を、一から開発するプロジェクト。
`RHTML5`/`RCSS3`/`RTypeScript`/`RBootStrap`構想の一部(詳細は`CLAUDE.md`参照)。

## 現状

トークナイザ(`Token`列挙型 + `TokenSink`トレイト、`html5ever`のTokenSinkパターンを参考にした疎結合設計)のみ実装済み。
DOM木構築・スタイル計算・レイアウトは未着手。

## 使用例

```rust
use rhtml5::{tokenize, CollectingSink};

let mut sink = CollectingSink::default();
tokenize(r#"<p class="a">Hello</p>"#, &mut sink);
for token in sink.tokens {
    println!("{:?}", token);
}
```

## ビルド・テスト

```bash
cargo test
```

## ライセンス

Apache-2.0 OR MIT
