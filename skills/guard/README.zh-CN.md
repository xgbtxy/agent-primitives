# `guard` (`crates/ap-guard`) — 2ms 内 Exit-2 防屎山硬规则、自动验证门禁与安全护盾

[English](./README.md) | [简体中文](./README.zh-CN.md)

**独立物理隔离 Rust Crate（`crates/ap-guard`）与技能包（`skills/guard/`）** · `<2 ms` 原生二进制 · 静默成本 `<65 tokens`。

## 解决什么痛点

写在 `AGENTS.md` 里的文字叮嘱很容易被模型忽略；`ap guard` 把代码规范与验证要求直接硬化为**不可绕过的 `exit 2` 工具门禁**：
1. **防屎山硬规则与防复发沉淀（`guard rule` + `guard verify`）**：在修改/新增文件中强制拦截 `: any`、`@ts-ignore` 与空 `catch {}` 吞异常；每次发现 AI 犯错，一行 `guard rule add` 即可写入 `.agents/guard.json` 让它永远没机会重犯，并自动串联单元测试与 `.agents/smoke.ap` 冒烟校验。
2. **双协议完成门禁与死循环熔断（`stop-gate` / `shield` / `run`）**：原生兼容 Claude/Codex `exit 2` 与 Antigravity camelCase JSON Hook，拦截密钥外泄与高危删库命令，并在连续 3 次同签名报错时强制熔断。

## 常用命令

```bash
ap guard verify
ap guard rule init
ap guard rule add --id no-slop-csv --glob "src/*.ts" --forbid "old_csv_dump" --msg "请使用符合 RFC4180 的 CSV 模块"
ap guard run --max-repeats 3 -- npm test
ap guard stop-gate
```
