# astrid-capsule-memory

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![MSRV: 1.94](https://img.shields.io/badge/MSRV-1.94-blue)](https://www.rust-lang.org)

[English](README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja.md)

**面向 [Astrid OS](https://github.com/unicity-astrid/astrid) 智能体的跨会话记忆。**

在操作系统模型中，此 capsule 相当于持久化交换文件。它跨越会话边界携带上下文，使智能体能够记住上次发生的事情。

## 工作原理

接入 `prompt_builder.v1.hook.before_build`。每次组装提示词时：

1. 通过 VFS 从工作区读取 `.astrid/memory.md`
2. 将内容包装在 `# Memory` 章节中
3. 向每个请求对应的响应主题发布 `appendSystemContext` 钩子响应

如果文件不存在或为空，此 capsule 不执行任何操作。

## 大小上限

智能体写入的内容可能无限增长（不同于由人编写的 `AGENTS.md`），因此设置 32KB 的硬性上限，以防上下文窗口用量无界增长。超出上限的内容会在 UTF-8 字符边界处截断，并添加 `[Memory truncated]` 标记。

## 只读

此 capsule 仅处理读取/注入操作。智能体使用 `astrid-capsule-fs` 提供的现有文件系统工具（`write_file`、`replace_in_file`）写入 `.astrid/memory.md`，无需新增工具。

## 开发

```bash
cargo build --target wasm32-unknown-unknown --release
```

## 许可证

采用 [MIT](LICENSE-MIT) 和 [Apache 2.0](LICENSE-APACHE) 双重许可。

版权所有 (c) 2025-2026 Joshua J. Bouw 和 Unicity Labs。
