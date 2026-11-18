# 实战提效手册：免费与主流编程智能体的一行纯 Rust 接入指南

[English](./PLAYBOOK.md) | [简体中文](./PLAYBOOK.zh-CN.md) · [演进总手册](./HANDBOOK.zh-CN.md) · [架构深潜](./DESIGN.zh-CN.md)

`agent-primitives`（`v1.2`）交付为**单个零依赖原生 Rust 可执行文件（`ap` / `ap.exe`）**，静态内嵌 **6 语言 Tree-sitter 语法器**、**内置 SQLite FTS5/BM25**、**纯 Rust 交互式无头 Edge/Chrome CDP 引擎**以及**标准 `ap mcp` JSON-RPC 服务**——用户电脑**无需安装 Python、Node 或 Rust 编译器**。

---

## 1. 零配置一行命令接入（小白开箱即用）

```bash
# macOS / Linux（自动识别并接线 OpenCode、Qwen、Gemini CLI、Goose、Pi、Codex、Antigravity、Claude、Grok）：
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.sh | sh
```

```powershell
# Windows PowerShell：
irm https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.ps1 | iex
```

```bash
# 仅接入当前项目目录（指定 opencode/qwen/gemini-cli 时自动生成项目级 MCP 配置）：
ap init --here -a opencode
ap init --here --with-hooks --with-mcp

# 随时一行无损卸载（完整保留你原有的个人配置）：
ap uninstall          # 或卸载当前项目：ap uninstall --here
```

### 原生支持的免费开源与商业智能体矩阵（$0 零 Key 直接提效）
- **OpenCode（`opencode`，MIT 开源含免费模型）**：自动写入 `~/.config/opencode/opencode.json`（或项目 `opencode.json`）启用 `ap mcp`。哪怕使用 `$0` 免费模型（`opencode/big-pickle`、`glm-5.3-flash-free`、`deepseek-v4.1-flash-free`），也能直接调用结构化工具 `ap_orient`、`ap_edit`、`ap_smoke`、`ap_verify`，彻底告别 Windows 命令引号报错！
- **Qwen Code（`qwen`）与 Google Gemini CLI（`gemini`）**：自动向 `~/.qwen/settings.json` 与 `~/.gemini/settings.json` 注入 `"trust": true` 的 `ap mcp` 配置，免除每次弹窗确认。
- **Pi（`pi`）与 Goose（`goose`）**：自动同步 `~/.pi/agent/skills/`、`~/.pi/agent/AGENTS.md` 与 `package.json` 插件声明。
- **Claude Code（`claude`）、Codex（`codex`）、Antigravity（`agy`）与 Grok（`grok`）**：同步技能、`~130 Token` 极简路由及 `<2ms` 原生 `exit 2` / JSON Hook（并在 Claude `Bash` 下启用 `rtk` 式测试输出自动脱水）。

---

## 2. 5 大原生 MCP 工具（`ap mcp`）与 CLI 原语协同机制

| MCP 工具 / CLI 命令 | 直接消除的大模型思考与系统卡点 | 原生延迟 |
| :--- | :--- | ---: |
| **`ap_orient`** / `ap atlas map` + `atlas product` + `probe slice --bundle` | 消除空仓环境盲猜、误读历史废文件，静态抓出 HTML `#id` ↔ JS 选择器及 `fetch()` ↔ 后端路由断链（Personalized PageRank 排序） | `6–18 ms` |
| **`ap_edit`** / `ap splice apply --cwd . --stdin --test` | 消除多文件串行编辑瀑布、2 空格缩进偏差（Aider 三级容错匹配）、PowerShell 转义崩溃，落盘前强制过 Tree-sitter 语法树 | `2.3 ms` |
| **`ap_smoke`** / `ap probe smoke --cwd . --stdin` | **砍掉 Windows 70% 手工验证税**：自动绑空闲 `PORT`、驱动本机无头 Edge/Chrome 真实点击输入（`OPEN`/`FILL`/`SELECT`/`CLICK`/`EVAL`）、查 `0` 页面/服务端报错并 `taskkill /T /F` 杀净进程树 | `2.5–4.5 s` |
| **`ap_verify`** / `ap guard verify` + `guard rule add` | 根治 `AGENTS.md` 叮嘱失灵与 AI Slop（拦截 `: any`、`@ts-ignore`、空 `catch {}`），自动调用 `tsc`/`biome`/`cargo`/测试（`exit 2` 硬门禁） | `<2 ms` (+测试) |
| **`ap_probe`** / `ap probe run` · `ap sieve` · `ap recall` · `ap baton` | 消除内存别名脑补、`200 KB` 测试日志刷屏（`-98.8%`）、长规则库膨胀（`3.4ms` SQLite FTS5）与上下文压缩失忆 | `3–29 ms` |

---

## 3. 本地过程损耗体检（`ap audit`）

一键扫描本机 `~/.codex`、`~/.pi`、`~/.claude`、`~/.grok`、`~/.gemini/antigravity-cli` 日志（`0` 字节外传）：

```bash
ap audit --compare
```
