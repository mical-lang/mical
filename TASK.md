## 目標

crates/config の from_source_file を実装する

## 現状の設計概要

mical言語のCLI向け評価器を実装する。
CLI用途に特化し、値を解析せずに文字列のまま保持しつつ、型情報（Integer, Boolean等）のみをタグとして付与する設計とする。
構造体名は `Config` とし、純粋な設定データコンテナであることを表す。

`Config` の検索・一覧メソッドに関しては実装済みであり、十分な test も書かれている。

## mical 言語

- A minimal, line-oriented configuration language designed for flat structures and readability.

概要は doc/src/overview.md に記載してある。
より詳細な使用は doc/src/specification/ を参考に。以下ファイルがある。

-  block_strings.md
-  keys.md
-  prefix_blocks.md
-  syntax.md
-  values.md

また、AST/CST は crates/syntax/mical.ungram やそこから生成される crates/syntax/src/ast.rs に。

トークン化は crates/lexer が、パースは crates/parser が行う。

パース結果については、 crates/parser/tests/snapshots/ 以下が参考になるかも。

## キーのテキスト処理

- WordKey: word() そのままでOK
- QuotedKey: エスケープ解決(後述)

## バリューのテキスト処理

- LineString: string() をそのままでOK
- Boolean: kind() で true/false
- Integer: 正規化などは不要でそのまま
- QuotedString: エスケープ解決(後述)
- BlockString:
  - パーサーがある程度(base indent検出)とかは頑張っているので、それ以外のものについて
  - lines() だけで足りるのかは要調査 (CSTでNEWLINE見る必要ある？ない？)

## エスケープ解決

specification (doc/src/specification/values.md#quoted-string) には明記されていないので、適切な場所に追記してほしいのですが、以下をエスケープすれば十分です。

- `\"` => ダブルクオート
- `\n` => 改行 "\n"
- `\r` => 改行 "\r"
- `\t` => タブ "\t"
- `\\` => バックスラッシュ
- `\'` => シングルクオート

## エラーハンドリング

エラー型を定義すること。

- 構文的な欠損は、パーサーが検出しているはずなので、from_source_file で見つけたときは、それらを区別せず、すべて同じエラー種別で構わない。
- それ以外のエラーについては、(多分エスケープ解決だけ?) それぞれエラー種別を適切に分けること。
