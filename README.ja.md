# astrid-capsule-memory

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![MSRV: 1.94](https://img.shields.io/badge/MSRV-1.94-blue)](https://www.rust-lang.org)

[English](README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja.md)

**[Astrid OS](https://github.com/unicity-astrid/astrid) エージェント向けのセッション間メモリ。**

OS モデルでは、この capsule は永続的なスワップファイルに相当します。セッションの境界を越えてコンテキストを引き継ぐため、エージェントは前回の出来事を記憶できます。

## 仕組み

`prompt_builder.v1.hook.before_build` にフックします。プロンプトを組み立てるたびに、次の処理を行います。

1. VFS を介してワークスペースから `.astrid/memory.md` を読み込む
2. 内容を `# Memory` セクションで囲む
3. リクエストごとのレスポンストピックに `appendSystemContext` フックレスポンスを発行する

ファイルが存在しないか空の場合、この capsule は何も行いません。

## サイズ上限

エージェントが書き込む内容は、人が作成する `AGENTS.md` とは異なり、際限なく増える可能性があります。そのため、コンテキストウィンドウの消費量が無制限に増えないよう、32KB のハードリミットを設けています。上限を超えた内容は UTF-8 の文字境界で切り詰められ、`[Memory truncated]` マーカーが追加されます。

## 読み取り専用

この capsule は読み取りと注入だけを処理します。エージェントは `astrid-capsule-fs` の既存ファイルシステムツール（`write_file`、`replace_in_file`）を使用して `.astrid/memory.md` に書き込みます。新しいツールは必要ありません。

## 開発

```bash
cargo build --target wasm32-unknown-unknown --release
```

## ライセンス

[MIT](LICENSE-MIT) と [Apache 2.0](LICENSE-APACHE) のデュアルライセンスです。

Copyright (c) 2025-2026 Joshua J. Bouw and Unicity Labs.
