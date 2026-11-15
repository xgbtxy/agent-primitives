# `recall` — 毫秒级按需触发记忆库（标准库 SQLite FTS5/BM25）

[English](./README.md) | [简体中文](./README.zh-CN.md)

**独立单文件夹微基础设施（`skills/recall/`）** · 零第三方依赖（Python 内置 `sqlite3`） · 待机仅占 `71 Tokens`。

## 解决什么真实痛点？

绝大多数 Agent 记忆失效，本质是**「检索失灵」**而非「存储失灵」：
- 把一两百条模块避坑规则或个人事务 SOP 全塞进 `AGENTS.md` / `CLAUDE.md` / `MEMORY.md`，每轮对话白白烧掉 `6,000~15,000 Tokens`，还会造成注意力稀释，真正该守的那条反而被模型漏看；
- 常驻向量数据库 MCP 光工具 Schema 就先吃掉大块上下文，且部署笨重。

`recall` 直接基于 Python 标准库内置 `sqlite3` FTS5（`BM25` 排序）+ 路径通配符/词边界触发器 + 中英双语混合分词索引。200 条规则实测中位检索耗时仅 **`3.7 ms`**，平均仅向当前步骤注入 **`~40 Tokens`**（较全量文件缩减 **99.4%**），中英文混合评测 **Top-1 命中率 `20/20 (100%)`**。

## 单独拷走这一个文件夹即可使用

```bash
cp -r skills/recall ~/.agents/skills/
# 或：npx skills add xgbtxy/agent-primitives --skill recall
```

## 常用命令

```bash
# 存入研发避坑铁律（触碰对应目录或关键词时唤醒）
python scripts/recall.py add \
  --triggers "src/auth/*,jwt,refresh" \
  --rule "Refresh Token 单次旋转失效，严禁用旧缓存重试 401。" \
  --kind gotcha

# 存入个人助手日常 SOP（原生支持中文双字/单字 + 英文混合检索）
python scripts/recall.py add \
  --triggers "发票,报销,finance" \
  --rule "差旅发票归档需附校验码并抄送财务归档邮箱。" \
  --kind workflow

# 执行动作前毫秒级精准召回（<5ms）
python scripts/recall.py match --query "修复登录刷新 401 循环" --files "src/auth/session.py" --top 2

# 查看统计与清理长期零命中过期条目
python scripts/recall.py stats
python scripts/recall.py prune --stale-days 90
```
