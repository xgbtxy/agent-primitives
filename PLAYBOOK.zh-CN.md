# 真实提效实战指南：纯 Rust 单二进制 1 条命令接入全主流 Harness

[English](./PLAYBOOK.md) | [简体中文](./PLAYBOOK.zh-CN.md) · [总纲手册](./HANDBOOK.zh-CN.md) · [抗淘汰底层架构设计](./DESIGN.zh-CN.md)

再强的工具，如果还要折腾 Python/Node 环境版本、配半小时 JSON 或频繁撞上终端引号转义，用户用一次就会放弃。在 **`v1.0`** 中，`agent-primitives` 全面升级为**单文件纯 Rust 原生二进制（`ap` / `ap.exe`）**，将 6 语言官方 **Tree-sitter C 语法树**、`ripgrep` 并行遍历器、**静态编译 SQLite FTS5/BM25** 与全部 7 个 `SKILL.md` 打包进同一个可执行文件——**用户电脑无需安装 Python、Node 甚至无需安装 Rust 编译器**，一行命令秒级下载即用。

---

## 零、第 0 步：先用 `ap audit` 测出你本机 Agent 到底在浪费什么

100% 本地离线扫描 `~/.codex`、`~/.pi`、`~/.claude`、`~/.grok`、`~/.gemini/antigravity-cli` 历史轨迹（零联网、不上传任何内容）：

```bash
ap audit --compare
```

几秒内即可量化本机真实发生的**同文件重复重读、手工终端切行、`python -c`/`node -e` 引号崩溃与 `ImportError`、补丁失配、UTF-8 编码报错、`>8 KB` 巨型输出刷屏与死循环重试**，并对比前后半段会话的摩擦变化趋势。

---

## 一、免环境一行命令接入全主流 Harness（`install.sh` / `install.ps1` / `ap init`）

无需 `git clone`、无需 `python`、无需 `pip`/`npm`、无需手写 JSON：

```bash
# 通用一行预编译单二进制安装 + 自动接入（macOS / Linux）：
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.sh | sh

# 通用一行预编译单二进制安装 + 自动接入（Windows PowerShell）：
irm https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.ps1 | iex

# 已安装 Rust 工具链的开发者也可直接通过 Cargo 源码编译安装：
cargo install --git https://github.com/xgbtxy/agent-primitives --locked && ap init

# 仅接入当前项目仓库（自动植入自包含 .agents/bin/ap、.agents/skills 并幂等写入 AGENTS.md）：
ap init --here

# 按需指定 Harness + 挂载 <2ms 原生 Rust 门禁 Hook（Claude Code、Codex、Antigravity）：
ap init -a codex,pi,agy,claude --with-hooks

# 随时一行干净卸载（100% 保留你原有的个人规则与配置）：
ap uninstall          # 或：ap uninstall --here
```

### 各大 Harness 底层发现机制精确适配细节
- **OpenAI Codex CLI (`codex`)**：同步 `~/.codex/skills/` + `~/.agents/skills/`，向 `~/.codex/AGENTS.md` 注入自包含 `<10 行` 围栏路由块，`--with-hooks` 自动合并 `<2ms` 原生 `ap guard` 至 `~/.codex/hooks.json`。
- **Pi (`pi` / `pi-coding-agent`)**：同步 `~/.pi/agent/skills/`（项目模式 `-a pi` 同步 `.pi/skills/`）与 `~/.pi/agent/AGENTS.md`，并提供 `package.json` `"pi"` 清单。
- **Google Antigravity (`agy` / IDE)**：同步 `~/.gemini/config/skills/` + `~/.agents/skills/` 与 `~/.gemini/config/GEMINI.md`；`--with-hooks` 按 Antigravity 专属命名组写入 `"agent-primitives-guard"`（`"matcher": "run_command"`），原生兼容 camelCase `deny`/`allow`/`continue`/`stop` JSON 协议。
- **Anthropic Claude Code (`claude`)**：同步 `~/.claude/skills/` 与 `~/.claude/CLAUDE.md`，自动备份 `.bak` 并合并 `PreToolUse`、`PreCompact`（`ap baton`）与 `Stop`（`exit 2`）至 `~/.claude/settings.json`。
- **xAI Grok Build (`grok`)**：同步 `~/.grok/skills/` 与 `~/.grok/AGENTS.md`。

---

## 二、核心闭环：全部 7 个原语待机仅占 `412 Tokens`，卡住 5 个物理咽喉点

```text
[阶段 1：认路与精准唤起] --> ap atlas  (16ms 6 语言 Tree-sitter 结构地图 + >90 天陈旧 TODO 预警)
                         --> ap recall (3.3ms 静态 SQLite FTS5 + 中英双栖检索，仅注入 ~47 tok)
[阶段 2：切片与因果诊断] --> ap probe  (7ms slice --bundle 剪除 Py/TS/JS/Go/Rust/C 死文件并打包活跃源码；
                                        run 抓堆别名/原地篡改/空操作/-0；diff 跨 Py/TS/JS/Rust 差分验证)
[阶段 3：外科手术式落盘] --> ap splice (0.8ms Tree-sitter 符号精读 [-95.4% tok]；
                                        3.1ms 零转义 apply --stdin --test + 6 语言语法树预检原子回滚)
[阶段 4：观测流无损降噪] --> ap sieve  (>1.2 KB 日志/JSON/JSONL/HTML表/CSV 压减 97-99% + FTS5 跨归档搜索)
[阶段 5：门禁与跨轮接力] --> ap guard  (<2ms 原生 exit-2 密钥/高危拦截、3 次同错熔断、完工门禁)
                         --> ap baton  (<400 tok 保存已核验事实 + DO NOT RETRY 死胡同，抗压缩秒接续)
```

---

## 三、场景一：Coding Agent 怎么用才能立竿见影？

1. **1 次调用完成 6 语言死代码剪枝 + 清单与活跃源码打包（`ap probe slice . --bundle`）**：
   - 面对 **Python、TypeScript/JS、Go、Rust、C** 多文件仓库，首条命令直接跑 `ap probe slice . --bundle`（`~7 ms`），剪除未引用死模块并一次性输出项目清单与带行号活跃源码，直接省掉 10~16 次串行读文件往返。
2. **用毫秒级因果与差分探针替代 20 分钟文字空想（`ap probe run` / `diff`）**：
   - 自动检出 `[ALIAS HAZARD]`、`[IN-PLACE MUTATION]`、`[STATE NO-OP]`、`-0`/`NaN`、`True == 1` 缓存碰撞及 Rust `i64` 负数整除舍入偏差（`ap probe diff --lang rust`）。
3. **零转义跨文件原子补丁 + 6 语言真 Tree-sitter 语法守门 + 自动跑测（`ap splice`）**：
   - 直接通过管道将 `@@ FILE:`、`@@ SYMBOL:`、`@@ WRITE:` 喂给 `ap splice apply --cwd . --stdin --test --rollback-on-test-fail`（PowerShell 用 `@'...'@ |`，Bash 用 `<<'EOF'`）。**落盘前自动对 Py/TS/JS/Go/Rust/C 全部修改文件执行真 Tree-sitter 语法树校验**，1 轮完成改代码 + 写单测 + 跑测试闭环！

---

## 四、场景二：个人助手 Agent 怎么用才能真实提效？

1. **大型笔记库导航与章节级精准读写（`ap atlas` + `ap splice view --heading`）**：`<20ms` 扫出数百篇 Markdown 提纲并按章节原子读写。
2. **零云端中英双栖个人 SOP 记忆（`ap recall`）**：`3.3 ms` 中英混合检索（200 条规则 Top-1 准确率 **100%**）。
3. **JSONL 流、HTML 表格与 CSV/TSV 降噪（`ap sieve`）**：自动提取网页/CSV 管道表并支持 `ap sieve search` 秒搜历史原文。
4. **跨天不断片与 `<2ms` 安全防线（`ap baton` + `ap guard`）**：压缩后 `<400 Tokens` 恢复现场，物理拦截密钥外泄与高危命令。

---

## 五、诚实边界：什么时候**不要**用？

- **已知极短命令不要套 `sieve`**：如 `pwd`、`git status -s`（`<1.2 KB` 原样直通）。
- **单行已知拼写修复不要跑 `probe`**：改完直接跑 `ap splice check . --test` 最快。
- **全局身份红线留 `AGENTS.md`，低频条件经验放 `recall`**。
