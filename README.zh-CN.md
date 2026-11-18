# agent-primitives (`ap` v1.2)

**纯 Rust 单文件二进制、零 AI Slop、直击大模型思考链与系统物理边界的免费/旗舰智能体提效基座。**

[English (Default)](./README.md) | [简体中文](./README.zh-CN.md) · **[Engineering Handbook (EN)](./HANDBOOK.md)** | **[核心工程宪章 (中文)](./HANDBOOK.zh-CN.md)** · [架构深潜](./DESIGN.zh-CN.md) · [实战手册](./PLAYBOOK.zh-CN.md)

---

## 一行命令零配置接入（支持免费 `$0` 与主流编程智能体 · Windows / macOS / Linux）

**无需安装 `Python`、无需 `Node`、无需 `Docker`、无需 `git clone`、无需记忆任何参数。** 一行命令下载经 SHA-256 校验的原生二进制 `ap`（静态内嵌 **6 语言 Tree-sitter C 语法器**、**内置 SQLite FTS5**、**纯 Rust 交互式无头 Edge/Chrome CDP 引擎**与**标准 `ap mcp` JSON-RPC 服务**），自动完成本机全部智能体接线——**哪怕是零基础小白在 Windows 上使用 `$0` 免费智能体，也能立刻获得 `2–6 倍` 真实提效与零 Slop 代码质量**：

```bash
# macOS / Linux（自动探测并接线 OpenCode、Qwen、Gemini CLI、Goose、Pi、Codex、Antigravity、Claude、Grok）：
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.sh | sh
```

```powershell
# Windows PowerShell：
irm https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.ps1 | iex
```

```bash
# 使用 Cargo 从源码构建，或仅接入当前代码目录：
cargo install --git https://github.com/xgbtxy/agent-primitives --locked && ap init
ap init --here -a opencode                 # 自动生成 opencode.json 启用 `ap mcp` 与防屎山硬规则
ap init --here --with-hooks --with-mcp     # 完整注入项目级 Hook 与 MCP (.mcp.json / .qwen / .gemini)
ap uninstall                               # 一行干净卸载（100% 恢复原有配置）
```

### 原生支持的免费（`$0`）与商业编程智能体矩阵

| 智能体宿主 | 免费 / 付费 | `ap init` 全自动接线路径 | 通信协议与安全门禁 |
| :--- | :---: | :--- | :--- |
| **OpenCode** (`opencode`) | **`$0` 零 Key 免费模型** + 自带 Key | `~/.config/opencode/opencode.json` + `AGENTS.md` | 原生 **`ap mcp`**（`ap_orient`、`ap_edit`、`ap_smoke`、`ap_verify`、`ap_probe`） |
| **Qwen Code** (`qwen`) | **每日免费额度** + 开源 | `~/.qwen/settings.json`（`"trust": true`）+ `QWEN.md` | 原生 **`ap mcp`**（免弹窗确认、零转义） |
| **Google Gemini CLI** (`gemini`) | **免费额度** + 云端 | `~/.gemini/settings.json`（`"trust": true`）+ `GEMINI.md` | 原生 **`ap mcp`** + `<140-tok` 极简路由 |
| **Pi** (`pi`) & **Goose** (`goose`) | 开源 + 低成本 API | `~/.pi/agent/skills/`、`~/.config/goose/` + `AGENTS.md` | `package.json` 插件清单 + CLI / MCP |
| **OpenAI Codex CLI** (`codex`) | 订阅 / API | `~/.codex/skills/` + `~/.codex/AGENTS.md` + `hooks.json` | `<2ms` `exit 2` 门禁 + 零转义 CLI |
| **Claude Code** (`claude`) | 订阅 / API | `~/.claude/skills/` + `CLAUDE.md` + `settings.json` | `<2ms` `exit 2` 完成门禁 + **`rtk` 式 `PreToolUse` 自动脱水** + `.mcp.json` |
| **Antigravity** (`agy`) & **Grok** | 内置 / API | `~/.gemini/config/`、`~/.grok/` | CamelCase JSON `deny`/`continue` Hook + CLI |

---

## 为什么 `ap` v1.2 能做到极致提效：解剖大模型 `<thinking>` 推理流 + 融合顶尖开源算法

通过对本机 **`101,108` 次工具调用**与原始 `<thinking>` 推理流的逐帧拆解，我们拒绝重复造低质轮子，将开源界最锋利的算法凝练进单文件纯 Rust 引擎，精准击穿四大卡点：

1. **冷启动盲猜与旧屎山污染（第 1–5 轮）** → **`ap-atlas` + `ap-probe slice`**
   - 吸收 **Aider Personalized PageRank**（8 轮幂迭代导入图中心度）、空仓 **宿主运行时指纹**（`node:sqlite`、`--experimental-strip-types` 禁用 `enum` 预警、无头 CDP 状态）与 `6.35 ms` 死代码剥离，杜绝模型误读废文件照抄 `: any` 与空 `catch {}`。
2. **跨文件工作记忆漂移与补丁缩进报错（第 6–15 轮）** → **`ap-atlas product` + `ap-splice` + `ap mcp`**
   - `<17 ms` 静态交叉核验 **HTML `id="..."` ↔ JS `getElementById`/`querySelector`** 及 **前端 `fetch()` ↔ 后端路由**；融合 **Aider 三级容错匹配（精确 → 行尾去空 → 相对缩进自动平移）**（`2.33 ms`），**落盘前强制校验 6 语言 Tree-sitter 具体语法树**（MCP 模式下 `0` 引号转义）。
3. **Windows 高达 70%+ 的手工验证 Token 税（第 16–60 轮）** → **`ap-probe smoke`（`ap_smoke`）**
   - 对照组在 Windows 上有 **`71%–75%` 的步骤**（`16–35` 轮、`85 万+` Token）耗在后台服务挂死终端、查杀端口及手写 150 行 Chrome DevTools 脚本。`ap probe smoke` 自动绑空闲 `PORT`、文件落盘日志防死锁、通过**纯 Rust `TcpStream` 驱动无头 Edge/Chrome 真实点击输入**（`OPEN`/`FILL`/`SELECT`/`CLICK`/`EVAL`）、校验 `0` 页面/服务端报错并 `taskkill /T /F` 杀净进程树，**1 次 `<4.5s` 调用替代 20+ 轮手工验证**。
4. **`AGENTS.md` 文字叮嘱失灵与 AI Slop** → **`ap-guard`（`ap_verify`）+ `ap-sieve`**
   - `<2 ms` 强制执行 `.agents/guard.json` **`exit 2` 防屎山硬规则**（拦截 `: any`、`@ts-ignore`、吞错空 `catch {}`），自动复用已装的 `tsc`/`biome`/`cargo check`/测试，并实现 **`rtk` 式 `PreToolUse` `updatedInput` 测试洪流自动脱水**（日志压缩 `-98.8%`）。

---

## Windows 真实受控实跑矩阵（涵盖小白一句话提示词、存量全栈与空目录随机新项目）

| 智能体宿主与实测场景 | 模式 | 耗时与工具调用数 | 思考链与验证开销变化 | 总 Token 与费用 | 独立真机验收与防 Slop |
| :--- | :--- | ---: | ---: | ---: | :---: |
| **`OpenCode 1.19` 免费版（`$0` `big-pickle`）**<br>*小白一句话提示词（`WorkPulse`）* | 原生 → **`+ ap v1.2 (MCP)`** | `430.2s`（`60` 次）→ **`182.4s`（`23` 次，快 2.36 倍 / -57.6%）** | 推理 Token **-18.2%**；仅 **`1` 次 `bash`**（告别查端口死循环） | `1,613,689` → **`401,930 tok`（-75.1%，`$0.00`）** | `7/8` *(残留 `empty-catch`)* → **`8/8`（`0` Slop！）** |
| **`Codex CLI 0.167` (`gpt-6-astra`)**<br>*空目录从零做随机项目（`SplitMate`）* | 原生 → **`+ ap v1.2`** | `345.3s`（`23` 次）→ **`244.6s`（`9` 次，耗时 -29.1%）** | `16` 轮 `cua_repl` → **`0` 轮**（`1` 次 CDP `probe smoke` 搞定！） | `1,116,369` → **`186,379 tok`（-83.3% / 省下 6.0 倍！）** | **`6/6` → `6/6`** |
| **`Pi CLI 1.5.2` (`deepseek-flash`)**<br>*空目录从零做随机项目（`SplitMate`）* | 原生 → **`+ ap v1.2`** | `1,821.1s` → **`1,499.3s`（耗时 -17.7% / 快 `322s`）** | 物理拦截并根除前端与存储层吞异常坏代码 | `15.55M` → `19.94M` *(模型自写子集和暴力对拍，诚实披露)* | `5/6` *(2 处 `empty-catch` 屎山)* → **`6/6`（9 文件 `0` Slop！）** |
| **`Codex CLI 0.167` (`gpt-6-astra`)**<br>*普通用户需求升级（`WorkPulse`）* | 原生 → **`+ ap v1.2`** | `333.6s`（`24` 次）→ **`294.3s`（`11` 次，调用 -54.2%）** | `75%` 手工验证轮次 → **单次 `probe smoke` + `guard verify`** | `1,053,580` → **`364,682 tok`（-65.4% / 省 `68.9 万`）** | **`8/8` → `8/8`** |
| **`Pi CLI 1.5.2` (`deepseek-flash`)**<br>*普通用户需求升级（`WorkPulse`）* | 原生 → **`+ ap v1.2`** | `483.0s`（`81` 次）→ **`271.5s`（`70` 次，耗时 -43.8%）** | `<thinking>` 字符数 **`165,048` → `94,584`（-42.7%！）** | `3,628,149`（`$0.117`）→ **`2,295,801 tok`（-36.7%，`$0.075`）** | **`8/8` → `8/8`** |
| **`Grok 1.0.77` & `Antigravity 1.5.8`**<br>*24 文件 TS `ts-ledger` & 25 文件 Py `rulelang`* | 原生 → **`+ ap init`** | *Grok*: **耗时 -26.5%～-49.1%**；*Agy*: **步数 -42.9%～-61.3%** | *Grok*: **推理 -65.4%**；*Agy*: **`read_file` -74.2%**、**报错 -85.7%** | *Grok*: **Token -38.7%**；*Agy*: **净 Token -49.8%** | **`5/5` → `5/5`** |

---

## 确定性微基准测试（[`bench/results.json`](./bench/results.json) · 全量静默仅 `390 Tokens`）

| 独立 Crate 与原语 | 原始基线 | 启用 `ap`（Rust `v1.2`） | 压缩 / 削减率 | 原生延迟 | 保真与安全承诺 |
| :--- | ---: | ---: | ---: | ---: | :--- |
| **`ap-splice`**（`view` / 三级容错 `apply`） | `1,048 tok` 全文件 | `48 tok` 精确函数体 | **-95.4% 读取 Token** | **`0.71 ms`**（`apply`: **`2.33 ms`**） | Aider 三级缩进容错 + **落盘前校验 Tree-sitter 语法树** |
| **`ap-recall`**（200 条中英混合规则库） | `6,880 tok` 单体文件 | `47.0 tok` *(Top-2)* | **-99.3%** | **`3.36 ms`** | **20/20（100%）** Top-1 准确率（内置 SQLite FTS5） |
| **`ap-probe`**（24 文件 TS `slice` + CDP `smoke`） | `24` 个 `.ts` 文件 | `9` 个活跃（剪除 `15` 个） | **-62.5% 死文件剪除** | **`6.35 ms`** | 原生交互式无头 Edge/Chrome CDP + Py/V8/`rustc` 探针 |
| **`ap-atlas`**（64 模块 Rust SDK PPR `map`） | `42,338 tok` 全量读取 | `438 tok` PPR 地图 | **-99.0%** | **`17.2 ms`** | Personalized PageRank + DOM `#id`/API 断链与防屎山雷达 |
| **`ap-sieve`**（2,500 行日志 / JSON / HTML） | `43.0k` / `27.0k` / `12.0k tok` | `518` / `105` / `198 tok` | **-98.8% / -99.6% / -98.4%** | `28.7` / `12.4` / `11.4 ms` | **`5.25 ms`** FTS5 搜索 + `<1.2 KB` 原样直通（负向对照 `0.0%`） |

---

## Cargo 工作区结构（`0` 兄弟 Crate 依赖 · `14/14` 端到端测试套件）

```text
agent-primitives/
├── Cargo.toml / src/{main,mcp,install,audit,bench}.rs  # 单文件原生二进制 `ap` + 标准 `ap mcp` 服务
├── crates/ap-{atlas,recall,probe,splice,sieve,guard,baton}/  # 7 个物理隔离独立 Crate（0 兄弟依赖）
├── install.sh / install.ps1                            # 一行 SHA-256 预编译二进制安装脚本
├── .github/workflows/{ci.yml,release.yml}              # 三大系统 clippy+测试门禁 & 5 架构自动发版
└── tests/integration_primitives.rs                     # 14 项物理隔离、MCP、CDP、缩进容错与 Hook 测试
```

## 开源协议

[MIT](./LICENSE)
