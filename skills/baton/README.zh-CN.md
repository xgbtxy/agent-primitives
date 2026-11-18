# `baton` (`crates/ap-baton`) — 抗压缩状态胶囊与跨 Agent 接力棒

[English](./README.md) | [简体中文](./README.zh-CN.md)

**独立纯 Rust Crate（`crates/ap-baton`）与技能目录（`skills/baton/`）** · 零兄弟依赖 · 待机仅 `60 tokens`。

## 解决什么真实痛点？

1. **压缩失忆**：长任务触发自动上下文压缩（`/compact`）后，模型忘记**哪些方案已经试错失败**，反复重踩同一个坑。
2. **跨工具断片**：在 Claude Code、Codex、Grok Build、Antigravity、Pi 或 git worktree 之间切换时，任务上下文丢失。

`ap baton` 在本地 `.baton/` 维护确定性 `< 400 tokens` 状态胶囊，严格分离：**目标与约束**、**已核验事实（带源码/链接凭据）**、**已证伪死胡同（`DO NOT RETRY`）** 与 **下一步动作**，并自动抓取当前 Git 分支与脏文件快照。

## 一键接入

```bash
# Windows PowerShell 一行安装：
irm https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.ps1 | iex
# 或仅在当前项目启用 baton：
ap init --here --only baton
```

## 常用命令

```bash
ap baton set --goal "消除 Webhook 发送器 429 重试风暴" --next "修复 sender.rs:118 退避抖动"
ap baton fact "上游 Retry-After 单位是秒而非毫秒" --ref "sender.rs:94"
ap baton deadend "盲目翻倍工作线程数" --reason "直接触发更严厉的 IP 级限流"
ap baton checkpoint --note "上下文压缩前自动快照"
ap baton resume
```
