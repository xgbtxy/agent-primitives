# `guard` — Exit-2 真相门禁、死循环熔断与高危拦截盾

[English](./README.md) | [简体中文](./README.zh-CN.md)

**独立单文件夹微基础设施（`skills/guard/`）** · 零第三方依赖 · 待机仅占 `63 Tokens`。

## 解决什么真实痛点？

自治 Agent 日常最烧钱、最危险的三类事故：
1. **带病谎报“已完成”（`exit 1` 陷阱）**：在 Claude Code、Codex、Grok Build 等主流运行时的 `Stop` Hook 中，脚本返回 `exit 1` 只会被当成非阻断提示，Agent 照样结束对话！**只有返回 `exit 2` 并向 `stderr` 输出错误详情**，才能把 Agent 强制打回重修。
2. **原地死循环烧额度**：同一条报错命令一字不改连跑五六遍，纯烧 Token 毫无进展。
3. **密钥泄露与高危外发事故**：把 `sk-...` / `ghp_...` / `AKIA...` 私钥拼进命令或写入文件，或误执行 `git push --force`、`rm -rf`、批量群发外部邮件/Webhook。

`guard` 将这三道防线统一封装为确定性门禁，并**自动识别双协议**：同时原生兼容 **Claude Code / Codex**（`exit 2` + `stderr`）与 **Google Antigravity**（camelCase JSON `{"decision": "deny"|"allow"}` / `{"decision": "continue"|"stop"}` 及防死循环保护）。

## 单独拷走这一个文件夹即可使用

```bash
cp -r skills/guard ~/.agents/skills/
# 或：npx skills add xgbtxy/agent-primitives --skill guard
```

## 常用命令

```bash
# 1. 事前护盾：检测高危命令与密钥外泄（支持 --text 或直接读取 Hook stdin JSON，违规即 exit 2）
python scripts/guard.py shield --text "git push --force origin main"

# 2. 死循环熔断器：同一报错指纹连续出现 3 次立即触发 exit 2 强制换思路
python scripts/guard.py run --max-repeats 3 -- pytest tests/test_payment.py

# 3. 完工真相门：检查无冲突标记、无明文密钥且通过指定验收命令，否则 exit 2 打回
python scripts/guard.py stop-gate --check "python -m unittest discover -s tests"
```
