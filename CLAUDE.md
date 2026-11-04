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

## API 方針: REST 全廃・gRPC / Connect へ (2026-11-04)

### 日本語
- **REST は全廃する。新しい API は REST で作らない。** ブラウザや `curl` から呼ぶ API は **Connect の単純呼び出し**(`POST /<パッケージ>.<Service>/<Method>`、`content-type: application/json`、本文は JSON)、サービス間は **gRPC / gRPC-web**(RPoem の管理API `open_runo.admin.v1.Admin` と同じ流儀)。ストリーミングは、必要になってから。
- **既存の REST は、触るときに順に移す。** 移したら旧 REST の経路は消す。移行が終わるまで残るが、**新規には追加しない**。
- **JSON は RS-JSON に統一**する(読み込みは `from_str_secure` / `parse_secure` でサイズ・深さ・重複キーの上限つき、書き出しは `rust_json::fast`)。ブラウザの WASM に `prost` は入れない(サイズが増えるため)。
- **例外(自分たちの API ではなく、相手の規格が決めているもの)**: ACME、OAuth/OIDC のコールバック、第三者(GitHub など)からの Webhook、ヘルスチェック、静的ファイル配信、OTLP など。これらは規格どおりの HTTP を受ける。
- 参考実装: `RPoem`(`open_runo.admin.v1.Admin`)、`open-easy-web`(`server/src/connect.rs`、`proto/open_easy_web/admin/v1/admin.proto`)。
- ※ このリポジトリのコードが既に方針どおりかどうかは、各リポジトリの現状による。この節は方針の記録であり、コードの変更を意味しない。

### English
- **REST is being abolished. New APIs are not built as REST.** APIs called from browsers or `curl` use **Connect unary calls** (`POST /<package>.<Service>/<Method>`, `content-type: application/json`, JSON body); service-to-service calls use **gRPC / gRPC-web** (the same style as RPoem's admin API `open_runo.admin.v1.Admin`). Streaming only when it is actually needed.
- **Existing REST endpoints are migrated one by one, when they are touched.** The old REST route is removed once migrated. Until migration is finished they remain, but **no new ones are added**.
- **JSON is unified on RS-JSON**: reading with `from_str_secure` / `parse_secure` (size, depth and duplicate-key limits), writing with `rust_json::fast`. `prost` is not put into browser WASM (it would grow the binary).
- **Exceptions (shapes dictated by someone else's standard, not our own API)**: ACME, OAuth/OIDC callbacks, webhooks from third parties (e.g. GitHub), health checks, static file serving, OTLP, and so on. These accept plain HTTP as the standard requires.
- Reference implementations: `RPoem` (`open_runo.admin.v1.Admin`) and `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Note: whether the code in a given repository already follows this policy depends on that repository. This section records the policy; it does not mean the code has been changed.

### Deutsch (KI-Übersetzung, noch nicht von Muttersprachlern geprüft)
- **REST wird abgeschafft. Neue APIs werden nicht als REST gebaut.** APIs, die aus dem Browser oder mit `curl` aufgerufen werden, nutzen **Connect-Unary-Aufrufe** (`POST /<Paket>.<Service>/<Methode>`, `content-type: application/json`, JSON-Body); zwischen Diensten gilt **gRPC / gRPC-web** (wie bei der Admin-API von RPoem, `open_runo.admin.v1.Admin`). Streaming erst, wenn es wirklich nötig ist.
- **Bestehende REST-Endpunkte werden einzeln migriert, sobald man sie anfasst.** Nach der Migration wird die alte REST-Route entfernt. Bis dahin bleiben sie bestehen, es werden aber **keine neuen hinzugefügt**.
- **JSON wird auf RS-JSON vereinheitlicht**: Lesen mit `from_str_secure` / `parse_secure` (Grenzen für Größe, Tiefe und doppelte Schlüssel), Schreiben mit `rust_json::fast`. `prost` kommt nicht in Browser-WASM (die Datei würde größer).
- **Ausnahmen (Formate, die ein fremder Standard vorgibt, nicht unsere eigene API)**: ACME, OAuth/OIDC-Callbacks, Webhooks von Dritten (z. B. GitHub), Health-Checks, statische Dateien, OTLP usw. Diese nehmen schlichtes HTTP entgegen, wie es der Standard verlangt.
- Referenzimplementierungen: `RPoem` (`open_runo.admin.v1.Admin`) und `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Hinweis: Ob der Code eines Repositorys diese Richtlinie schon befolgt, hängt vom jeweiligen Repository ab. Dieser Abschnitt hält die Richtlinie fest; er bedeutet nicht, dass der Code bereits geändert wurde.

### Русский (машинный перевод, носителями языка ещё не проверен)
- **REST полностью отменяется. Новые API не создаются как REST.** API, вызываемые из браузера или через `curl`, используют **унарные вызовы Connect** (`POST /<пакет>.<Service>/<Method>`, `content-type: application/json`, тело — JSON); между сервисами — **gRPC / gRPC-web** (так же, как административный API RPoem, `open_runo.admin.v1.Admin`). Потоковая передача — только когда она действительно нужна.
- **Существующие REST-эндпоинты переносятся по одному, когда до них доходят руки.** После переноса старый REST-маршрут удаляется. До завершения переноса они остаются, но **новые не добавляются**.
- **JSON унифицируется на RS-JSON**: чтение через `from_str_secure` / `parse_secure` (ограничения на размер, глубину и повторяющиеся ключи), запись через `rust_json::fast`. `prost` не добавляется в браузерный WASM (размер вырос бы).
- **Исключения (форматы, заданные чужим стандартом, а не наш собственный API)**: ACME, обратные вызовы OAuth/OIDC, вебхуки сторонних сервисов (например, GitHub), проверки работоспособности, раздача статических файлов, OTLP и т. п. Они принимают обычный HTTP, как требует стандарт.
- Эталонные реализации: `RPoem` (`open_runo.admin.v1.Admin`) и `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Примечание: соответствует ли код конкретного репозитория этой политике, зависит от самого репозитория. Этот раздел фиксирует политику и не означает, что код уже изменён.
