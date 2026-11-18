# `agent-primitives` (`ap` v1.2) — 核心工程宪章与演进总手册

[English](./HANDBOOK.md) | [简体中文](./HANDBOOK.zh-CN.md) · [首页](./README.zh-CN.md) · [架构深潜](./DESIGN.zh-CN.md) · [实战手册](./PLAYBOOK.zh-CN.md)

> **总纲使命**：绝不生产任何 AI Slop（提示词废话、糊弄代码、重复造轮子）。围绕**大模型真实的 `<thinking>` 思考瓶颈、工作记忆漂移与 Windows 验证死穴**打造 `<20ms` 纯 Rust 确定性工具——让哪怕是零基础小白，在 Windows 上用 **$0 免费开源编程智能体**也能获得十年老开发的工程质量与数量级提效。

---

## 1. 核心理念：顺着大模型的思考链路做确定性赋能，绝不靠提示词说教

### 1.1 把每一个原语精准钉在大模型的四大 `<thinking>` 认知卡点上
通过对本机 **`101,108` 次真实工具调用**及数十万字原始 `<thinking>` 推理流的逐帧解剖（横跨 `Codex`、`OpenCode`、`Pi`、`Antigravity`、`Claude Code`、`Grok`），大模型写代码慢、爱出 Bug、烧 Token 的根源高度集中在**四个认知与环境断层**：

1. **冷启动环境盲猜与旧屎山污染（第 1–5 轮）**：空目录开局时模型动辄耗费 `10k–35k` 思考字符盲猜本机运行时（有没有 `node:sqlite`？`--experimental-strip-types` 支持 `enum` 吗？有没有无头浏览器？）；存量项目则把未引用的历史废文件读进上下文照抄 `: any` 与空 `catch {}`。
   - **物理器官（`ap-atlas` + `ap-probe slice`）**：参考 Aider 的 **Personalized PageRank（8 轮幂迭代导入图中心度）** + 空仓冷启动宿主能力指纹 + `6.35 ms` 死代码剥离。
2. **多文件工作记忆漂移与补丁缩进/转义报错（第 6–15 轮）**：从零写 6+ 个文件时，模型极易写串 HTML `id="..."` 与 JS `getElementById`、写歪前端 `fetch('/api/...')` 与后端路由，或因 2 空格缩进偏差与 PowerShell 转义导致补丁失败重来。
   - **物理器官（`ap-atlas product` + `ap-splice` + `ap mcp`）**：`<17 ms` 静态交叉核验 **DOM `#id` ↔ JS 选择器** 与 **前端请求 ↔ 后端路由**；融合 Aider **三级容错匹配（精确 → 行尾空白归一化 → 相对缩进自动平移）** 且落盘前强制校验 Tree-sitter 语法树（`2.33 ms`）；并通过原生 **`ap mcp`** 彻底消灭 Shell 引号转义。
3. **高达 70%+ 的手工验证 Token 税与 Windows 进程挂死（第 16–60 轮）**：实测对照组在 Windows 上有 **`71%–75%` 的步骤**（`16–35` 轮、`85 万+` Token）耗在后台服务挂死终端、残留端口冲突、以及手写 150 行 Chrome DevTools WebSocket 脚本点按钮上。
   - **物理器官（`ap-probe smoke`）**：纯 Rust 临时端口防挂起服务托管（文件落盘日志 = `0` 管道死锁）+ 原生 `TcpStream` **交互式无头 Edge/Chrome CDP**（`OPEN`、`FILL`、`SELECT`、`CLICK`、`EVAL` + 自动捕获页面 `pageerror`/`console.error`）+ `taskkill /T /F` 整树清理，**1 次 `<4.5s` 调用跑完真人级页面交互验证**。
4. **`AGENTS.md` 文字叮嘱失灵与 AI Slop 泛滥**：规则写得再多，模型在长上下文下也会忽略或理解过头，照写 `: any`、`@ts-ignore` 和吞异常空 `catch {}`。
   - **物理器官（`ap-guard` + `ap-sieve` + `ap-baton` + `ap-recall`）**：`<2 ms` **`exit 2` 物理阻断门禁**（`.agents/guard.json`）、自动联动已有生态工具（`tsc`/`biome`/`cargo check`/测试）、融合 `rtk` 思路的 `PreToolUse` `updatedInput` 测试洪流自动脱水，以及 `<400-tok` 防压缩失忆胶囊。

### 1.2 小白零门槛体验 + 借力成熟开源标准（拒绝从 0 到 1 闭门造轮子）
- **一行命令全自动接线（`ap init`）**：自动探测本机**免费与主流智能体**（`OpenCode`、`Qwen Code`、`Gemini CLI`、`Goose`、`Pi`、`Codex`、`Antigravity`、`Claude Code`、`Grok`），零参数自动写入 **`ap mcp`**（`opencode.json`、`.qwen/settings.json`、`.gemini/settings.json`）、`<140-tok` 极简路由与安全门禁。小白完全不需要知道任何 `ap` 命令参数，免费模型即可直接调用结构化工具 `ap_orient`、`ap_edit`、`ap_smoke`、`ap_verify`。
- **充分复用成熟生态**：遵循官方 **MCP JSON-RPC 2.0** 标准，静态内嵌官方 **Tree-sitter** C 语法器与 **SQLite FTS5**，直接驱动 Windows 自带 **Edge/Chrome CDP**，自动复用项目已装的 `tsc`/`biome`/`cargo`/`pytest`，绝不重复造低质轮子。

---

## 2. 十年老开发的六条硬核红线

1. **零 AI Slop**：禁止冗长提示词说教、禁止热循环内重复编译正则（`OnceLock` 全局单次编译）、禁止空 `catch {}` 吞错、禁止 `: any` 糊弄、全工作区 `clippy -D warnings` 零警告。
2. **Crate 物理隔离**：7 个子 Crate（`crates/ap-*`）之间严格保持 **`0` 兄弟依赖**。
3. **零外部运行时前置依赖**：单文件原生二进制 `ap` / `ap.exe`，覆盖 Linux（`musl` 静态 + `arm64`）、macOS、Windows MSVC。
4. **极致性能与低静默税**：单目录 `SKILL.md` Frontmatter **`48–61 tokens`**（7 个合计仅 **`390 tokens`**）；AST/SQLite/Hook 核心路径 **`0.7–18 ms`**。
5. **约束交给代码而非文字**：每次发现模型犯错，必须沉淀为 `ap guard rule add` 硬规则、`atlas product` 契约检查或 `probe smoke` 回归步骤。
6. **实证诚实**：在存量项目与空目录随机新项目、付费与 `$0` 免费智能体上用普通大白话提示词真实跑通，原样披露正负向数据。

---

## 3. 版本演进台账

| 里程碑 | 核心工程升级 | 真实受控实跑数据 |
| :--- | :--- | :--- |
| **`v0.1`–`v0.3` (2026-11-15)** | 确立 7 大原语、本地 4 宿主审计、多语言 `slice --bundle` 与一行接入 | `Grok 1.0.77` **耗时 -26.5%～-49.1%**；`Antigravity 1.5.8` **净 Token -49.8%**、`read_file` **-74.2%** |
| **`v1.0.0` (2026-11-18)** | **纯 Rust 单二进制重写（`0` `.py` 残留）**：7 隔离 Crate、6 语言 Tree-sitter、内置 SQLite FTS5、零转义补丁、5 架构预编译发版 | `splice view` **`0.71ms`**、`apply` **`2.33ms`**、`recall` **`3.36ms`**；`Codex` 仅用 **`4` 条命令**满分修复 24 文件 `ts-ledger` |
| **`v1.2.0` (2026-11-18)** | **思考链深潜 + 免费智能体零配置 MCP 引擎**：原生交互式 CDP（`OPEN`/`FILL`/`SELECT`/`CLICK`/`EVAL`）、DOM `#id`↔JS 与路由断链雷达、Personalized PageRank、Aider 三级缩进容错补丁、`rtk` 式自动脱水及 **`ap mcp`** | **普通用户需求 `WorkPulse`**：`Codex` 总 Token **-65.4%（`105.4万 → 36.5万`）**；`Pi` 耗时 **-43.8%（`483s → 272s`）**、思考量 **-42.7%**。**空目录随机新项目 `SplitMate`**：`Codex` 总 Token **-83.3%（`111.6万 → 18.6万`，直降 `6.0 倍`！）** 且耗时 **-29.1%**；`Pi` 从 **`5/6`（写出两处 `empty-catch` 屎山）跃升至 `6/6`（9 文件零 Slop，快 `322s`）** |
