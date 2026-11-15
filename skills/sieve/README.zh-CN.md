# `sieve` — 无损输出漏斗与黑匣子回放器

[English](./README.md) | [简体中文](./README.zh-CN.md)

**独立单文件夹微基础设施（`skills/sieve/`）** · 零第三方依赖 · 待机仅占 `83 Tokens`。

## 解决什么真实痛点？

一次 `pytest` / `cargo` 测试、一份 CI 构建日志、一段大 JSON 接口返回或抓回来的长网页（动辄 `50~300 KB`），会瞬间把 Agent 上下文灌满（Context Rot），导致后续每一轮对话都在重复为这堆噪音付费。而市面上常见的命令行代理采用**有损裁剪**——一旦把关键报错行裁掉了，Agent 只能重跑整条耗时命令。

`sieve` 采用**「高信噪比摘要 + 本地环形黑匣子无损回放」**双层机制（默认上限 50 MB 自动淘汰旧文件）：
- **测试 / 构建 / 运维日志**：自动折叠重复刷屏行，精准提取去重后的异常调用栈并标注精确行号（`L915-L920`），压减 **98.8%**。
- **JSON 接口数据**：提取顶层字段、数组长度、元素 Schema 与 `error`/`warnings` 告警字段，400 条记录实测压减 **99.6%**。
- **网页 HTML**：剥离 `<script>`、`<style>`、`<svg>` 噪音，保留标题、`H1–H3` 大纲与核心正文，压减 **98.4%**。
- **短输出保护（`< 1.2 KB`）**：短结果（如 `git status`）自动原样直通，绝不画蛇添足。

## 单独拷走这一个文件夹即可使用

```bash
cp -r skills/sieve ~/.agents/skills/
# 或：npx skills add xgbtxy/agent-primitives --skill sieve
```

## 常用命令

```bash
# Coding Agent：包裹冗长测试或编译命令
python scripts/sieve.py run -- pytest -q

# 个人助手 Agent：通过管道压缩 API JSON、网页 HTML 或长日志
curl -s https://api.example.com/events | python scripts/sieve.py pipe --label events

# 毫秒级无损回放原文切片（无需重跑原命令！）
python scripts/sieve.py peek <id> --lines 910:930
python scripts/sieve.py peek <id> --grep "AssertionError|KeyError" -C 6
python scripts/sieve.py peek <id> --json-path "items.0"
```
