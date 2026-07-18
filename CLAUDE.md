# 開発方針＆開発環境ルール(rhtml5)

作業ドライブは`F:\open-runo`。この節は[`open-raid-z`](https://github.com/aon-co-jp/open-raid-z)の`CLAUDE.md`を正本とし、各プロジェクトへコピーして同期する方針に準じる。

## このプロジェクトの構想(2026-07-18新設)

`RHTML5`/`RCSS3`/`RTypeScript`/`RBootStrap`という4プロジェクト構想の1つ。
HTML5相当の**ブラウザエンジン級**パーサー/DOM実装を、既存ブラウザエンジンの
コードを一切流用せず一から開発する(簡易テンプレートエンジンではない)。
将来的にPoem上でのSSR(サーバー側でRHTML5がDOM木構築 → RCSS3でスタイル計算 →
HTML文字列としてレスポンス、クライアントはブラウザ標準でパースし、後から
RTypeScript(Wasm)がハイドレーション)を見据える。

**開発順序(確定)**: RHTML5/RCSS3(土台)→ RBootStrap → RTypeScript
(`RReact`は別プロジェクトで並行開発、本構想のスコープ外)。

**最初のマイルストーン**: 「RHTML5 + RCSS3だけでPoemから静的相当の
完全なHTMLが返せる」こと。レイアウトエンジン(flexbox/grid)は
クライアント側ブラウザが最終的にレイアウトするため、初期SSRでは後回し。

## 設計思想の参考(コードは流用せず、方針のみ踏襲)

- **`html5ever`(Servo由来)の`TokenSink`パターン**: トークナイザが
  `Token`列挙型を生成し、`TokenSink`トレイトを実装したDOM木構築器に
  渡す疎結合設計。本クレートの`token::TokenSink`はこの思想を踏襲。
- **`cssparser`(Servo由来)のライフタイム分離パーサー**: 将来のRCSS3
  実装時に参考にする(本クレートは現段階で`String`ベースの単純な
  実装、ゼロコピー化は次段階の課題として明記)。

## 現状(第一段、2026-07-18)

- `src/token.rs`: `Token`列挙型(`Doctype`/`StartTag`/`EndTag`/
  `Comment`/`Characters`/`Eof`)、`Attribute`、`TokenSink`トレイト、
  テスト用`CollectingSink`。
- `src/tokenizer.rs`: WHATWG HTML Standardの完全な状態機械(約80状態)
  ではなく、実用上重要な部分状態機械のサブセットを実装
  (Data・TagOpen・TagName・属性名/値(引用符あり/なし)・
  SelfClosingStartTag・コメント・DOCTYPE・EOF)。
- **未対応(次段階)**: CDATA区間、文字参照(`&amp;`等)のデコード、
  `<script>`/`<style>`の生テキストモード、仕様上のパースエラー
  回復規則の厳密な再現。
- **検証**: `cargo test`で7件全green(プレーンテキスト・開始/終了タグ・
  引用符あり/なし/boolean属性・自己終了タグ・コメント・DOCTYPE・
  完全なHTML文書形状の一気通貫)。警告0件。

## 次にすべきこと

1. DOM木構築器(`TokenSink`実装、`Node`/`Element`/`Document`型)
2. RCSS3(パーサー→カスケード→スタイル計算、インラインstyle出力まで)
3. RHTML5+RCSS3を使った最小のPoem SSRエンドポイント(最初の
   マイルストーン)
4. レイアウトエンジン(flexbox/grid、後回し可)
5. RBootStrap → RTypeScript(RTypeScriptの実装案はB案「TS風構文の
   RustネイティブDSL、Wasm直接コンパイル」から着手しC案
   「swcでASTのみ取り込み、DOM操作サブセットを独自インタプリタで実行」
   へ拡張するのが現実的、という方針)

## 関連プロジェクト

- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — 開発ルールの正本
- RCSS3 / RBootStrap / RTypeScript — 未作成(次段階でリポジトリ新設予定)
- RReact — 別プロジェクトで並行開発中(本構想とは別スコープ)
