# `baton` — 抗压缩失忆与跨 Agent 交接胶囊

[English](./README.md) | [简体中文](./README.zh-CN.md)

**独立单文件夹微基础设施（`skills/baton/`）** · 零第三方依赖 · 待机仅占 `70 Tokens`。

## 解决什么真实痛点？

长任务中最致命的两个断层：
1. **自动压缩（Compaction）后失忆变傻**：干到关键时刻触发上下文压缩，醒来后把**已经试过并证明行不通的死胡同**全忘了，立刻回头把错路再踩一遍。
2. **跨客户端/跨天信息孤岛**：从 Claude Code 切到 Codex、Grok Build、Antigravity 或个人助手 OpenClaw（或跨天继续处理同一桩个人事务），状态全丢；而重型记忆框架往往要装 Bun/Docker 甚至绑卡付费，还会用 AI 二次总结烧 Token。

`baton` 用纯确定性本地文件（`.baton/`）维护一份严格控制在 **`< 400 Tokens`** 的接力胶囊，明确隔离四个槽位：**当前目标与硬约束**、**已核验事实（绑定行号/链接）**、**已证伪死胡同（`DO NOT RETRY` 严禁重试）**、**下一步唯一动作**，并自动抓取当前 Git 修改状态或近期工作区文件。

## 单独拷走这一个文件夹即可使用

```bash
cp -r skills/baton ~/.agents/skills/
# 或：npx skills add xgbtxy/agent-primitives --skill baton
```

## 常用命令

```bash
# 设定目标与下一步动作
python scripts/baton.py set --goal "修复 Webhook 429 重试风暴" --next "修改 sender.py:118 退避逻辑"

# 记录已验证事实与已证伪死胡同（防止压缩后或换 Agent 后重踩！）
python scripts/baton.py fact "上游 Retry-After 单位为秒而非毫秒" --ref "sender.py:94"
python scripts/baton.py deadend "盲目调大并发线程数" --reason "触发更严厉的 IP 级限流"

# 自动快照（可挂载到 PreCompact / Stop Hook）
python scripts/baton.py checkpoint --note "压缩前自动保存"

# 压缩后或切换任意 Agent 后，一句满血恢复（<400 Tokens）
python scripts/baton.py resume
```
