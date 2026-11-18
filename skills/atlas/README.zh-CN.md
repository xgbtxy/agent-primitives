# `atlas` (`crates/ap-atlas`) — 50ms 内 Tree-sitter 结构与产品功能地图

[English](./README.md) | [简体中文](./README.zh-CN.md)

**独立物理隔离 Rust Crate（`crates/ap-atlas`）与技能包（`skills/atlas/`）** · 内置 6 语言 Tree-sitter 语法树 + `ignore` 遍历器 · 静默成本 `<65 tokens`。

## 解决什么痛点

1. **代码仓与知识库结构图（`atlas map`）**：基于官方 **Tree-sitter C 语法树（`Python`、`TypeScript/TSX`、`JavaScript`、`Go`、`Rust`、`C`）** 与 `.gitignore` 高速遍历，扫描 64 模块仓库仅需 **`16.3 ms`**（**省下 99.0% 定向 Token**），按导入中心度与 `[dirty]` 热点排序并预警 `>90 天` 过期 TODO。
2. **产品功能地图与防屎山雷达（`atlas product`）**：一键从源码提取前端页面控件（`#id <button|input|select>`）、后端已注册 API 路由、**`[BROKEN]` 前后端断链失配**以及 **`[SLOP RADAR]` 代码坏味道**（`: any`、`@ts-ignore`、空 `catch {}` 吞异常），让 AI 测试有据可依、绝不照抄旧屎山。

## 常用命令

```bash
ap atlas map .
ap atlas product . --write PRODUCT_MAP.md
ap atlas map . --focus "exporter" --json
```
