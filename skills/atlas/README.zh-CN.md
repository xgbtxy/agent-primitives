# `atlas` — 亚秒级零重读代码仓与个人知识库地图

[English](./README.md) | [简体中文](./README.zh-CN.md)

**独立单文件夹微基础设施（`skills/atlas/`）** · 零第三方依赖 · 待机仅占 `73 Tokens`。

## 解决什么真实痛点？

每次开新会话，Agent 往往先盲目调十几次目录遍历与全文读取，还没动手干活就先烧掉几万甚至十几万 Token；而且实证研究表明，编码 Agent 一旦被陈旧无人问津的 `TODO` 注释吸引跑偏，单次会话会多烧 **77% Token**。

`atlas` 无需配置任何向量库或 Docker，在 **`< 0.1 秒`** 内即可单遍扫描**代码仓库**（提取 Python AST 类/方法、多语言符号签名、被引用最多的核心枢纽文件、入口清单及 **`> 90 天` 陈旧 TODO 诱饵预警**）或**个人笔记/文档库**（提取目录分布、Markdown `H1/H2` 大纲与近期活跃笔记），生成一份 `< 800 Tokens` 的结构导航图（较全量读取省 **99.4%**）。

## 单独拷走这一个文件夹即可使用

```bash
cp -r skills/atlas ~/.agents/skills/
# 或：npx skills add xgbtxy/agent-primitives --skill atlas
```

## 常用命令

```bash
# 秒级生成当前代码仓或个人知识库导航图（<800 Tokens）
python scripts/atlas.py map .

# 聚焦特定子系统或目录关键词
python scripts/atlas.py map . --focus "exporter" --max-files 15

# 输出结构化 JSON
python scripts/atlas.py map . --json
```
