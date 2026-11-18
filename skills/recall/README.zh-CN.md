# `recall` (`crates/ap-recall`) — 毫秒级触发式微记忆库（内置静态 SQLite FTS5/BM25）

[English](./README.md) | [简体中文](./README.zh-CN.md)

**独立纯 Rust Crate（`crates/ap-recall`）与技能目录（`skills/recall/`）** · 内置静态 SQLite FTS5 · 待机仅 `58 tokens`。

## 解决什么真实痛点？

把几百条子模块暗坑或个人办公规矩全塞进 `AGENTS.md` / `MEMORY.md`，每轮白烧 `6,880+ tokens` 且稀释模型注意力。`ap recall` 在单文件二进制中静态链接 SQLite 3（`FTS5` + `BM25` 排序），结合路径通配符与中英双栖二元分词：200 条规则库实测检索中位延迟 **`3.3 ms`**，Top-1 准确率 **`100% (20/20)`**，每轮仅注入 **`~47 tokens`（压减 99.3%）**，零外部数据库依赖。

## 常用命令

```bash
ap recall add \
  --triggers "src/auth/*,jwt,refresh" \
  --rule "Refresh token 单次轮换，禁止拿缓存旧 token 重试 401。" \
  --kind gotcha

ap recall add \
  --triggers "invoice,报销,差旅" \
  --rule "差旅报销须附 PDF 哈希并抄送归档邮箱 finance-archive@example.com。" \
  --kind workflow

ap recall match --query "401 刷新死循环" --files "src/auth/session.rs" --top 2
ap recall stats
ap recall prune --stale-days 90
```
