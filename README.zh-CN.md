# agent-primitives (`ap` v1.0)

**纯 Rust 单文件二进制、永不过时、零运行时依赖的编程 Agent 与个人助理 Agent 确定性微基础设施。**

[English (Default)](./README.md) | [简体中文](./README.zh-CN.md) · **[Handbook & Constitution (EN)](./HANDBOOK.md)** | **[核心理念与演进总手册 (中文)](./HANDBOOK.zh-CN.md)** · [架构深潜](./DESIGN.zh-CN.md) · [实战手册](./PLAYBOOK.zh-CN.md)

---

## 一行命令接入全部主流 Agent 宿主（`Codex` · `Pi` · `Antigravity` · `Claude Code` · `Grok`）

**无需装 `Python`、无需 `Node`、无需 `Docker`、无需 `git clone`、无常驻守护进程。** 一行命令自动拉取经 SHA-256 校验的原生单文件二进制 `ap`（静态内嵌 **6 语言 Tree-sitter 语法树解析器**与**内置静态 SQLite FTS5**），自动探测本机安装的所有 Agent 宿主，同步 7 个独立原语，并幂等写入自包含 `<10 行` 路由指引：

```bash
# macOS / Linux（优先拉取 x86_64 musl 静态二进制与 aarch64 原生二进制；无 Release 时自动回退 cargo）：
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.sh | sh
```

```powershell
# Windows PowerShell（拉取预编译 x86_64-pc-windows-msvc ap.exe 并校验 SHA-256）：
irm https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.ps1 | iex
```

```bash
# 或使用 Cargo 从源码编译安装（统一入口 `ap`，也可单独安装任意 `crates/ap-*` 子包）：
cargo install --git https://github.com/xgbtxy/agent-primitives --locked && ap init

# 仅接入当前代码仓库（硬链接/复制 .agents/bin/ap + .agents/skills + 围栏式 AGENTS.md 区块）：
ap init --here

# 指定目标宿主并无损合并 <2ms 的 PreToolUse/Stop 物理门禁 Hook：
ap init -a codex,pi,agy,claude --with-hooks

# 随时一行干净卸载（100% 保留你原有的个人规则与配置）：
ap uninstall          # 或卸载当前仓：ap uninstall --here
```

### 各宿主原生插件一行命令与发现路径对照表

不同宿主加载技能、全局指令与 Hook 的机制各不相同。`agent-primitives` 同时原生支持 **`ap init`** 与各宿主的内置插件安装命令：

| Agent 宿主 | 生态原生一行安装命令 | `ap init` 自动接线的技能与规则路径 | Hook 协议（`--with-hooks`） |
| :--- | :--- | :--- | :--- |
| **OpenAI Codex CLI** (`codex`) | `codex plugin marketplace add xgbtxy/agent-primitives && codex plugin add agent-primitives@agent-primitives`<br>*(或 `ap init -a codex`)* | `~/.codex/skills/` + `~/.agents/skills/`<br>规则：`~/.codex/AGENTS.md` | `~/.codex/hooks.json`（`<2ms` `exit 2` snake_case） |
| **Pi** (`pi` / `pi-coding-agent`) | `pi install git:github.com/xgbtxy/agent-primitives`<br>*(或 `ap init -a pi`)* | `~/.pi/agent/skills/`（项目级 `.pi/skills/`）<br>规则：`~/.pi/agent/AGENTS.md` | 通过 `ap guard` / `AGENTS.md` |
| **Google Antigravity** (`agy` / IDE) | `ap init -a agy --with-hooks`<br>*(或 `agy plugin install ./agent-primitives`)* | `~/.gemini/config/skills/` + `~/.agents/skills/`<br>规则：`~/.gemini/config/GEMINI.md` | `~/.gemini/config/hooks.json`（camelCase JSON `deny`/`continue`） |
| **Claude Code** (`claude`) | `/plugin marketplace add xgbtxy/agent-primitives`<br>*(或 `ap init -a claude --with-hooks`)* | `~/.claude/skills/`（项目级 `.claude/skills/`）<br>规则：`~/.claude/CLAUDE.md` | `~/.claude/settings.json`（`<2ms` `exit 2` + `PreCompact`） |
| **xAI Grok Build** (`grok`) | `ap init -a grok` | `~/.grok/skills/` *(优先于仓库同名技能)*<br>规则：`~/.grok/AGENTS.md` | 通过 `ap guard` CLI |
| **OpenClaw / Hermes / 35+ 宿主** | `npx skills add xgbtxy/agent-primitives -g` + `ap init` | `~/.openclaw/skills/`、`~/.hermes/skills/`、`~/.agents/skills/` | 配合 `ap init --with-hooks` |

---

## 为什么全面采用纯 Rust（`v1.0`）——以及为什么模型升级不会淘汰它

市面上 95% 的开源 Agent 项目会在半年内迅速过时，因为它们在造**「教模型怎么思考的拐杖」**（角色扮演、规划清单、重型多智能体包装器或付费向量 RAG），一旦下一代前沿模型完成后训练就会被权重彻底吸收。

**`agent-primitives`** 建立在对本机 **`427` 个真实会话、`101,108` 次工具调用**（横跨 `Codex`、`Pi`、`Antigravity`、`Claude Code`、`Grok`）的过程遥测审计之上。无论 `GPT-6`、`Grok-5`、`Claude-5` 或 `Gemini-4` 推理智商涨到多高，自回归 Token 流依然无法绕开**四道物理边界**：

1. **外部系统/网页高熵输出 vs. `O(N)` KV-Cache 物理计费** — 把 `170 KB` 测试日志、`400` 条 JSON/JSONL 数据或 `15` 个未引用的死代码文件塞进上下文，会物理性拉高首字延迟并在后续每轮重复烧钱。
2. **纯文本脑补推演 vs. 真实运行时内存堆与类型碰撞** — 在文本里硬推浅拷贝嵌套字典别名污染、原地入参修改、`-0.0` 负零漂移或 `True == 1` 类型陷阱，动辄空烧 `35k–55k` 思考 Token（`11–19 分钟`）；而手写临时单行命令又频频撞上引号转义与模块路径崩溃（本机实审 `3,908` 次临时探针）。
3. **BPE Token 生成 vs. 具体语法树（AST）与字节精确落盘** — 行号漂移导致 `50%+` 同文件反复重读、JSON 补丁转义损坏、CRLF 换行报错（实审 `728` 次）及 Windows 编码崩溃（实审 `214` 次）。
4. **上下文压缩失忆与自报完成偏差** — `/compact` 会擦除已证伪的死胡同，而自然语言规则无法在每次工具调用前做到无解释器冷启动延迟（告别 Python `40–90 ms`）的 `<2 ms` **`exit 2` 操作系统物理阻断**。

**为什么用纯 Rust `v1.0` 取代 Python/C**：编译型 Rust 将 AST/SQLite/Hook 核心延迟从 `40–90 ms` 压至 **`0.68–16.3 ms`**，彻底消灭用户机器上的 `Python`/`pip`/`venv` 环境故障，在编译期内存安全保障下静态内嵌 **6 语言 Tree-sitter C 语法引擎**与 **SQLite FTS5**，并在 7 个独立 Crate 间强制执行 **`0` 兄弟依赖**（全套 7 个技能静默 Frontmatter 总计仅占 **`412 tokens`**）。

---

## 第 0 步：几秒钟实测你自己电脑上的 Agent 损耗（`ap audit`）

对本机 `~/.codex`、`~/.pi`、`~/.gemini/antigravity-cli`、`~/.claude` 与 `~/.grok` 历史日志运行 **100% 本地流式零上云体检**：

```bash
ap audit --compare
```

本工作站实扫数据（`427` 个会话、`101,108` 次工具调用，扫描仅耗时 `8.2s`，`0` 字节外传）：
- **命中 `ap probe` 痛点**：`3,908` 次临时拼凑 `python -c` / `node -e` 探针、`249` 次模块路径与引号崩溃
- **命中 `ap splice` 痛点**：`1,104` 次同文件冗余重读、`5,412` 次 Shell 手切行号、`728` 次补丁失配、`214` 次编码崩溃
- **命中 `ap sieve` 痛点**：`24,619` 次超 `8 KB` 大输出灌入上下文、`238` 次截断溢出
- **命中 `ap atlas` 痛点**：`4,380` 次宽泛目录遍历与盲搜
- **命中 `ap guard + baton` 痛点**：`39` 次完全相同的 `3x` 报错死循环重试

---

## 7 个物理隔离 Rust Crate 与 Agent 五阶段闭环映射

| 独立 Crate 与目录 | 突破的物理边界 | 面向编程 Agent（`Py/TS/JS/Go/Rust/C`） | 面向个人助理 Agent | 原生延迟 / 静默税 |
| :--- | :--- | :--- | :--- | ---: |
| **[`ap-atlas`](./skills/atlas/README.zh-CN.md)**<br>([EN](./skills/atlas/README.md)) | **冷启动盲逛**与被过期 TODO 误导多烧 `+77%` Token | `16.3ms` 6 语言 **Tree-sitter** 签名、`.gitignore` 枢纽中心度、`[dirty]` 脏标记与 `>90 天` 过期 TODO 雷达 | `<800-tok` 500 篇个人笔记库目录与 `H1/H2` 结构地图 | `16.3ms` / `58 tok` |
| **[`ap-recall`](./skills/recall/README.zh-CN.md)**<br>([EN](./skills/recall/README.md)) | **静态 `AGENTS.md` / `MEMORY.md` 膨胀**每轮白烧数千 Token | `3.37ms` **内置 SQLite FTS5/BM25** + 路径通配（改 `src/auth/*` 仅注入 `~47-tok` 局部铁律） | `3.37ms` 中英混合双字符组索引，按需唤起联系人、开票与运维 SOP | `3.37ms` / `58 tok` |
| **[`ap-probe`](./skills/probe/README.zh-CN.md)**<br>([EN](./skills/probe/README.md)) | **误读死代码文件、`50k` Token 脑补内存堆**与单行脚本崩溃 | `6.32ms` 6 语言 Tree-sitter `slice --bundle`；内置 Py（`sys.settrace`）、TS/JS（Node V8 `Proxy`）与原生 `rustc` 运行时/差分探针（抓别名污染/原地篡改/`-0.0`） | 批处理前用 `diff` 与 `run` 对财务舍入、税率及 CSV/JSON 清洗脚本做边界验算 | `6.32ms` / `59 tok` |
| **[`ap-splice`](./skills/splice/README.zh-CN.md)**<br>([EN](./skills/splice/README.md)) | **`50%+` 同文件重读**、JSON 补丁转义地狱与写坏语法 | `0.68ms` 6 语言 Tree-sitter `view --symbol`（**-95.4%** tok）；`2.46ms` 零转义 `@@ FILE:` / `@@ SYMBOL:` 原子 `apply`（**落盘前校验 Tree-sitter 语法树** + 自动回滚）+ `--test` | `view --heading` 精读长文档章节；`@@ WRITE:` / `@@ FILE:` 防乱码、保 CRLF 原子更新配置与手册 | `0.68ms` / `61 tok` |
| **[`ap-sieve`](./skills/sieve/README.zh-CN.md)**<br>([EN](./skills/sieve/README.md)) | **上下文腐烂**（`>8 KB` 日志/JSON/JSONL/HTML/CSV 刷屏） | 压缩 CI/测试日志（**-98.8%**）与 JSONL（**-97.0%**）；`5.65ms` 内置 SQLite FTS5 `search` + 无损 `peek` | 压缩 400 条 API JSON（**-99.6%**）、HTML `<table>` 自动转管道表（**-98.4%**）与 CSV/TSV | `5.65ms` / `59 tok` |
| **[`ap-guard`](./skills/guard/README.zh-CN.md)**<br>([EN](./skills/guard/README.md)) | **虚假报完工**、报错死循环与密钥外泄 | `<2ms` 原生双协议 PreToolUse/Stop 门禁（Claude/Codex `exit 2` + Antigravity camelCase JSON）+ 3x 死循环熔断 | 物理阻断密钥泄露（`sk-`/`ghp_`/`xai-`/`AKIA`）、`rm -rf /` 误删与群发邮件/Webhook | `<2ms` / `57 tok` |
| **[`ap-baton`](./skills/baton/README.zh-CN.md)**<br>([EN](./skills/baton/README.md)) | **压缩失忆**与跨会话丢失负向约束 | 在 Codex ↔ Pi ↔ Claude ↔ Grok ↔ Antigravity 间交接已核实 `file:line` 事实与 **`DO NOT RETRY` 死胡同** | 跨天个人调研、采购比选与长清单在 `/compact` 后 `<400 tokens` 无损续接 | `<2ms` / `60 tok` |

---

## 多宿主真实 CLI 受控实跑结果（零虚构、含诚实边界）

在真实 CLI 上针对独立外部隐藏测试集受控实跑（`Task C: ts-ledger`，含 13 个未引用干扰模块与 5 个跨文件舍入/幂等/别名污染/批次原子回滚缺陷的 24 文件 TypeScript/Node 22 清算账本；`Task B: rulelang`，25 文件 Python 编译器/优化器/缓存）：

| CLI 宿主与评测任务 | 模式 | 命令数 / 步数 / 耗时 | 读文件次数与补丁摩擦 | 总 / 净 Token 与费用 | 隐藏测试集 |
| :--- | :--- | ---: | ---: | ---: | :---: |
| **Codex CLI `0.167.0`** (`gpt-6-astra`) · *Task C (`ts-ledger`)* | **`+ Rust ap v1.0 (--here)`** | **全程仅 `4` 条命令** *(首轮实跑 `6` 条命令耗时 `133.7s`)* | **`0` 次手读单文件**；**1 次 `ap splice apply --stdin --test` 原子完成 13 处修改 + 15 项测试**（`0` 报错） | **`161,797 tok`**（缓存命中 **91.8%**，未缓存输入仅 **`12.9k`**） | **`5/5`** |
| **Grok CLI `1.0.77`** (`grok-4.7`) · *Task C (`ts-ledger`)* | 原生 → **`+ ap init`** | `766.6s`（`36` 次）→ **`563.2s`（`23` 次，耗时 -26.5%）** | `18` → **`6` 次 `read_file`（-66.7%）** | `683,173` → **`418,623 tok`（-38.7%，`$0.86`→`$0.56`）** | **`5/5` → `5/5`** |
| **Antigravity `1.5.8`** (`agy`) · *Task B (`rulelang`) & Task C (`ts-ledger`)* | 原生 → **`+ ap init`** | *B*: `1,039s` → **`789.5s`（-24.0%）**；*C*: `31` → **`12` 步（-61.3%）** | *B*: `31` → **`8` 次 `read_file`（-74.2%）**；*C*: 编辑报错 **`7` → `1`（-85.7%）** | *B*: 净 Token **-49.8%**（输入 **-57.8%**）；*C*: 总 Token **-15.2%** | **`5/5` → `5/5`** |
| **Pi CLI `1.5.2`** (`deepseek-flash`) · *Task C (`ts-ledger`)* | 原生 → **`+ Rust ap v1.0`** | `37` → **`35` 次工具调用**（`301.3s` → `352.4s`） | `read` 调用 **`11` → `0`（-100%）** *(Windows Git-Bash 下模型先跑了 bash 循环，见注)* | `538.9k`（`$0.059`）→ `858.1k`（`$0.086`） | **`5/5` → `5/5`** |

> **诚实工程记录**：借助纯 Rust `v1.0` 的零转义 `@@ FILE:` 原子补丁协议与自包含 `AGENTS.md` 语法速查头，**`Codex CLI 0.167`** 在第 1 条命令即调用 `ap probe slice . --bundle` 剪除全部 13 个死代码文件，第 3 条命令单次调用 `ap splice apply --cwd . --stdin --test` 同时改完跨文件 Bug、新建回归测试并跑通全部 15 项测试，**全程仅 `4` 条命令满分通关**（未缓存输入仅 `12.9k` Token）。而在 Windows 上的 **`Pi CLI 1.5.2`（`deepseek-flash`）** 中，`ap v1.0` 虽然将 `read` 工具调用降为 `0`（`-100%`），但因 `Pi` 系统提示词鼓励 Git-Bash 原生循环（`for f in src/*.ts`），`deepseek-flash` 在首轮用 bash 读了干扰文件后才调用 `ap.exe`（已列入 `HANDBOOK` `P4` 优化项）。我们坚持原样呈现全部正负向实测数据，绝不挑拣。

---

## 确定性 Rust `v1.0.0` 微基准测试（`bench/results.json`）

本机运行 `ap bench` 即可在 `<2 秒` 内复现 [`bench/results.json`](./bench/results.json)（Rust `1.100`，Tree-sitter，内置 SQLite FTS5）：

| 原语与测试负载 | 原始基线 | 启用 `ap`（Rust `v1.0`） | 压缩 / 削减率 | 原生延迟 | 保真与安全承诺 |
| :--- | ---: | ---: | ---: | ---: | :--- |
| **`ap-splice`** · Tree-sitter `view` + 零转义 `apply` + `check` | `1,048 tok` 全文件 | `48 tok` 精确函数体 | **-95.4% 读取 Token** | **`0.68 ms`**（`apply`: **`2.46 ms`**） | **落盘前校验 Tree-sitter 语法树**；严格保留 CRLF |
| **`ap-recall`** · 200 条中英混合规则库 | `6,880 tok` 单体文件 | `47.0 tok` *(Top-2)* | **-99.3%** | **`3.37 ms`** | **20/20（100%）** Top-1 准确率（内置 SQLite FTS5） |
| **`ap-probe`** · 24 文件 TypeScript 仓库（`slice`） | `24` 个 `.ts` 文件 | `9` 个活跃（剪除 `15` 个） | **-62.5% 死文件剪除** | **`6.32 ms`** | 6 语言 Tree-sitter 真实导入图（`Py/TS/JS/Go/Rust/C`） |
| **`ap-atlas`** · 64 模块 Rust SDK 语料库（`map`） | `42,338 tok` 全量读取 | `438 tok` 结构地图 | **-99.0%** | **`16.3 ms`** | `.gitignore` 感知 Tree-sitter 签名、枢纽度与过期 TODO |
| **`ap-sieve`** · 2,500 行 CI 日志 / 400 条 JSON / 300 条 JSONL | `43.0k` / `27.0k` / `4.8k tok` | `519` / `105` / `146 tok` | **-98.8% / -99.6% / -97.0%** | `61.4` / `13.5` / `10.3 ms` | **`5.65 ms`** FTS5 全文搜索 + 无损 `peek` |
| **`ap-sieve`** · `<1.2 KB` 短输出 *(负向对照)* | `70 B`（`18 tok`） | `118 B`（`30 tok`） | **0.0%** *(原样直通)* | `9.37 ms` | 短输出绝不强行摘要 |

---

## Cargo 工作区结构与本地自检

```text
agent-primitives/
├── Cargo.toml / src/{main,install,audit,bench}.rs  # 统一单文件二进制 `ap`（静态内嵌全部 7 份 SKILL.md）
├── crates/                                         # 7 个物理隔离独立 Crate（兄弟 ap-* 依赖数为 0）
│   ├── ap-atlas/   ap-recall/   ap-probe/   ap-splice/   ap-sieve/   ap-guard/   ap-baton/
├── install.sh / install.ps1                        # 一行命令 SHA-256 校验预编译二进制安装脚本
├── .github/workflows/{ci.yml,release.yml}          # 三大系统 clippy+测试门禁 & 5 架构预编译二进制自动发版
├── skills/{atlas,recall,probe,splice,sieve,guard,baton}/  # <65-tok 极简 SKILL.md 与中英双语子文档
├── HANDBOOK.md / DESIGN.md / PLAYBOOK.md (+ .zh-CN.md)    # 1:1 同步的中英双语工程宪章、架构与实战手册
└── tests/integration_primitives.rs                 # 11 大端到端物理隔离、AST 回滚、编码与 Hook 测试套件
```

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
ap doctor
```

## 开源协议

[MIT](./LICENSE)
