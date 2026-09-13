# 開発方針＆開発環境ルール(RS-HTML)

作業ドライブは`F:\runo`。この節は[`open-raid-z`](https://github.com/aon-co-jp/open-raid-z)の`CLAUDE.md`を正本とし、各プロジェクトへコピーして同期する方針に準じる。

## リポジトリ改称(2026-09-13)

`RTHML`(GitHub上の実名、ローカルフォルダ名は`RHTML`)→`RS-HTML`へrename済み
(typoだった`RTHML`表記も同時に解消)。`aruaru.pro`向けのフロント基盤整備に
合わせた`RFrontEnd`傘下のネーミング統一の一環(`RS-CSS`・`RS-GraphQL`・
`RS-Node.js`も同時に改称)。crate名(`rhtml5`)・依存元(`RS-BootStrap`・
`RS-React`の`dom_bridge`フィーチャ)からの参照はpath文字列のみ更新し、
crate名自体は変更していない(依存関係の広さに対して名前変更の効果が
薄いため、今回は見送り)。以下の記述内の`RHTML`/`RTHML`表記は
改称前の履歴として残す。

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

## 現状(第二段、2026-07-18)

- `src/token.rs`: `Token`列挙型(`Doctype`/`StartTag`/`EndTag`/
  `Comment`/`Characters`/`Eof`)、`Attribute`、`TokenSink`トレイト、
  テスト用`CollectingSink`。
- `src/tokenizer.rs`: WHATWG HTML Standardの完全な状態機械(約80状態)
  ではなく、実用上重要な部分状態機械のサブセットを実装
  (Data・TagOpen・TagName・属性名/値(引用符あり/なし)・
  SelfClosingStartTag・コメント・DOCTYPE・EOF)。
- **`src/dom.rs`(2026-07-18新規)**: `Node`(`Element`/`Text`/
  `Comment`)・`Document`・`TreeBuilder`(`TokenSink`実装)。開いている
  要素のスタックでネスト構造を組み立てる簡略化版(HTML5仕様本来の
  insertion mode・adoption agency algorithmは再現しない)。
  void要素(`br`/`img`/`input`等、終了タグを持たない要素)を認識し
  子要素を持たせない。閉じ忘れの終了タグは無視、対応する開始タグが
  無いままEOFに達した要素は文書の子として畳み込む(実ブラウザの
  寛容な挙動を簡略化して再現)。`Document::to_html()`でHTML文字列へ
  シリアライズ可能(正規化された出力、入力とバイト単位で一致すると
  限らないが構造的に等価)。
- **未対応(次段階)**: CDATA区間、文字参照(`&amp;`等)のデコード、
  `<script>`/`<style>`の生テキストモード、仕様上のパースエラー
  回復規則の厳密な再現。
- **検証**: `cargo test`で13件全green(トークナイザ7件+DOM構築6件、
  ネスト要素構築・void要素の子無し確認・DOCTYPE捕捉・閉じ忘れ要素の
  畳み込み・**パース→シリアライズの構造的往復**を含む)。警告0件。

## 次にすべきこと

1. RCSS3(パーサー→カスケード→スタイル計算、インラインstyle出力まで)
2. RHTML5+RCSS3を使った最小のPoem SSRエンドポイント(最初の
   マイルストーン)
3. レイアウトエンジン(flexbox/grid、後回し可)
4. RBootStrap → RTypeScript(RTypeScriptの実装案はB案「TS風構文の
   RustネイティブDSL、Wasm直接コンパイル」から着手しC案
   「swcでASTのみ取り込み、DOM操作サブセットを独自インタプリタで実行」
   へ拡張するのが現実的、という方針)

## 関連プロジェクト

- [open-raid-z](https://github.com/aon-co-jp/open-raid-z) — 開発ルールの正本
- RCSS3 — 実装済み(未作成ではなくなった。2026-07-18時点で子孫結合子まで対応)
- [RTypeScript](https://github.com/aon-co-jp/RTypeScript) — 2026-07-18に最小スコープ(トークナイザ+型注釈除去)で新設済み
- RBootStrap — 未作成
- [RReact](https://github.com/aon-co-jp/RReact) — 別プロジェクトで並行開発中(本構想とは別スコープ)だが、2026-07-18に`dom_bridge`フィーチャ経由で本クレート(`rhtml5`)・`rcss3`と接続済み(RHTMLでパースしたDOM木→RCSSでスタイル解決→RReactのVNode木、という最小のEnd-to-Endパイプライン。詳細はRReact側CLAUDE.md参照)

## HANDOFF

- **2026-07-18 RReact/RCSSとの相互統合が(RReact側`dom_bridge`フィーチャ経由で)実現**:
  本クレート自体にコード変更は無いが、`RReact`が`rhtml5`(本クレート)を
  optional path依存として取り込み、`parse_document`で作った
  `Document`/`Element`をRCSSでスタイル解決してRReactの`VNode`木へ
  変換する処理を実装した。3プロジェクトが独立実装のまま繋がって
  いなかった状態を解消。詳細・テストはRReact側CLAUDE.md参照。
