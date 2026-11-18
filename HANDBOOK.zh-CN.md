# `agent-primitives` 核心理念与持续演进总手册

[English](./HANDBOOK.md) | [简体中文](./HANDBOOK.zh-CN.md) · [首页](./README.zh-CN.md) · [架构深潜](./DESIGN.zh-CN.md) · [实战手册](./PLAYBOOK.zh-CN.md)

> **手册定位**：本文档是 `agent-primitives` 项目的**工程宪法与持续研发总纲**。它把我们的底层理念、不可逾越的设计红线以及后续每一次迭代的闭环流程彻底讲透——确保本仓库在未来大模型不断升级换代的过程中始终极度克制、真实可测、永不过时。

---

## 1. 核心理念（为什么做它，以及为什么模型升级不会淘汰它）

### 1.1 绝不做「教模型怎么思考的拐杖」，只做「突破物理边界的确定性器官」
市面上 95% 的开源 Agent 项目会在半年内迅速过时，因为它们试图在 Prompt 或厚重的外层编排框架里**教大模型怎么思考**（角色扮演、固定拆解模板、多智能体状态机、有损向量 RAG）。前沿模型每一次后训练迭代（`GPT-6`、`Grok-5`、`Claude-5`、`Gemini-4`、`DeepSeek`）都会把这些软拐杖内化进模型权重，让昨天的脚手架变成今天的延迟税。

我们的理念恰恰相反：**基础模型越聪明，越讨厌冗长的 Prompt 说教；但它使用微秒/毫秒级、内存安全、100% 确定性的操作系统/AST/内存堆/字节级精密仪器时，反而会越用越锋利。**

无论模型智商涨到多高，LLM 本质上仍然是运行在只读追加 KV-Cache 上的自回归 Token 生成器，它与真实计算机及个人数据之间永远横亘着**四道物理边界**：
1. **外部高熵输出 vs. `O(N)` KV-Cache 物理成本**：把 `200 KB` 的测试日志、巨型 JSON/JSONL 接口返回、充满噪声的网页 HTML 或 15 个根本没被引用的死代码文件灌进上下文，会物理性拉高首字延迟、稀释注意力，并在后续每一轮重复计费。
2. **纯文本脑补推演 vs. 真实运行时内存堆与类型系统**：让模型用自然语言在脑子里硬推浅拷贝嵌套字典别名污染、原地修改入参、`-0.0` 负零漂移或 `None` / `True == 1` 类型碰撞，动辄空烧 `35k–55k` 推理 Token（耗时 `11–19 分钟`）；而临时拼单行脚本又极易撞上 Shell 引号转义崩溃与模块路径错误。
3. **BPE Token 概率流 vs. 字节/语法树精确落盘事务**：模型吐出的是 BPE Token，而磁盘文件要求严格的 UTF-8 编码、原样保留的 CRLF/LF 换行符、写入磁盘**前**即通过真实 Tree-sitter 具体语法树校验，以及零 JSON 转义破坏的补丁协议。
4. **上下文压缩失忆与自报完成偏差**：会话触发 `/compact` 压缩后会抹掉极其关键的负向约束（“哪条路已经试过证实走不通”），而概率性文本永远无法替代 `<2 ms` 操作系统级的 **`exit 2` 物理阻断门禁**。

**第一性铁律**：我们用**纯 Rust** 在这四道物理边界上打造 **Unix 式的确定性原语**（Agent 世界的 `ripgrep` / `git` / `strace` / `sqlite`）。再顶尖的人类工程师也不会因为变聪明就扔掉 `ripgrep` 和 `git`——越强大的模型，驾驭单文件原生二进制 `ap` 时的杠杆率越高。

### 1.2 微基础设施美学：纯 Rust 单文件二进制 + 7 个物理隔离独立 Crate
我们坚决拒绝需要装 `Python`、`Node`/`Bun`、`Docker`、常驻守护进程、向量数据库或付费 API Key 的重型全家桶：
- **零运行时前置依赖（`ap` / `ap.exe`）**：单个预编译原生 Rust 可执行文件静态内嵌了 **6 语言 Tree-sitter 语法解析器**（`Python`、`TypeScript/TSX`、`JavaScript`、`Go`、`Rust`、`C`）、**内置静态编译 SQLite 3（`FTS5` + `BM25`）**、原生 `.gitignore` 高速遍历器与全部 7 份 `SKILL.md`。用户机器上既不需要装 Python 也不需要装 Rust，一行 `curl ... | sh` 或 `irm ... | iex` 秒级接入。
- **Crate 级物理自治**：7 个原语各自对应 `crates/ap-{atlas,recall,probe,splice,sieve,guard,baton}` 独立 Cargo Crate，**兄弟 `ap-*` 依赖数为 `0`**，且每个 Crate 都自带独立 `main.rs`（支持单独 `cargo install --path crates/ap-splice`）。
- **近乎为零的静默上下文税**：每个目录的 `SKILL.md` Frontmatter 平时仅占 **`57–61 tokens`**（全部 7 个全装合计仅 **`412 tokens`**）。

### 1.3 编程 Agent 与个人助理 Agent 的同构底层基座
编程智能体（`Codex`、`Pi`、`Antigravity`、`Claude Code`、`Grok Build`、`Cursor`）与个人助理智能体（`OpenClaw`、`Hermes`、知识库/科研/运维助理）在底层遭遇的是**完全同构的过程瓶颈**：
- 代码仓需要 6 语言 Tree-sitter 核心结构图；500 篇个人笔记库同样需要目录与 `H1/H2` 提纲地图（`atlas`）。
- 代码子模块有局部防坑规则；客户开票、合同审阅与服务器运维同样有按需触发的中英双语 SOP（`recall`）。
- 修 Bug 需要可达性切片与内存/边界追踪；财务对账与数据清洗脚本在批处理前同样需要差分边界验算（`probe`）。
- 源码需要 `<1 ms` Tree-sitter 符号读取与零转义 `@@ FILE:` 原子修改；长手册与 JSON 配置同样需要按章节读取与防乱码原子写入（`splice`）。
- `pytest`/`cargo` 洪流需要无损压缩；REST API JSON/JSONL 流、CSV/TSV 报表与网页 HTML `<table>` 同样需要脱水提纯与 `<6 ms` FTS5 检索（`sieve`）。
- 强推代码需要 `<2 ms` 执行前护盾与测试门禁；自主个人助理同样需要拦截密钥外泄与群发爆炸半径的 **`exit 2` 物理护盾**（`guard`）。
- 跨 CLI 编程协作需要状态胶囊；跨天个人调研与采购比选同样需要防压缩失忆接力（`baton`）。

---

## 2. 六条不可妥协的工程红线（立项宪法）

任何违背以下任一红线的功能提议或改动一律否决：

1. **零纯 Prompt 水分**：绝不新增没有确定性原生计算支撑的纯提示词/角色设定目录。
2. **严格的 Crate 物理隔离**：每个 `crates/ap-<name>/` 必须在**零兄弟 `ap-*` 依赖**下独立编译运行（由 CI 中的 `test_crate_physical_isolation` 强制守护）。
3. **零外部运行时依赖与全平台原生一致**：提供预编译单文件 Rust 二进制（Linux x64 `musl` 纯静态链接、macOS 原生 Mach-O、Windows MSVC 原生 `.exe`），静态内置 SQLite FTS5 与 Tree-sitter；必须在 Linux、macOS、Windows（含 PowerShell 管道早关、Windows `\\?\` 长路径与 CRLF 换行保留）上零警告通过 `clippy -D warnings` 与全量集成测试。
4. **硬性上下文与延迟预算**：单目录静默 Frontmatter **`< 85 tokens`**（实测 `57–61`）；核心 CLI 延迟 **`< 50 ms`**（AST/SQLite 热路径实测 `0.68 ms`–`16.3 ms`）；默认标准输出 **`< 1,000 tokens`** 并支持无损 `search` / `peek` 回溯。
5. **先做深做透，绝不盲目凑目录数**：除非证实存在全新的第 5 道物理边界，否则永远优先打磨现有 7 个正交原语，绝不为了凑数量拆第 8 个包。
6. **实证诚实与负向对照**：所有结论必须绑定真实会话审计（`ap audit`）、确定性基准测试（`ap bench`）与真实 CLI 实跑日志——必须诚实披露**何时不该调用该工具（负向对照）及不同宿主环境的真实表现边界**。

---

## 3. 当前「1 + 7」纯 Rust 架构全景与职责边界

```text
[离线过程体检] ap audit [--compare]  ──► 扫描本地 5 大宿主：Codex / Pi / Antigravity / Claude / Grok 会话日志（0 上云）
   │
   ├── 阶段 1：定向与唤起  ──► crates/ap-atlas/  (16ms 6语言 Tree-sitter 结构/知识库地图 + >90天过期 TODO 雷达)
   │                       ──► crates/ap-recall/ (3.4ms 内置 SQLite FTS5/BM25 + 中英混合按需触发微规则库)
   ├── 阶段 2：切片与诊断  ──► crates/ap-probe/  (6.3ms 6语言 Tree-sitter slice --bundle + Py/TS/JS/Rust 运行时/差分探针)
   ├── 阶段 3：外科手术    ──► crates/ap-splice/ (0.7ms AST 符号精读 + 2.5ms 零转义 AST 预检原子补丁与自动测跑)
   ├── 阶段 4：输出脱水    ──► crates/ap-sieve/  (98%+ 日志/JSON/JSONL/HTML表格/CSV 压缩 + 5.7ms FTS5 搜索与 peek)
   └── 阶段 5：门禁与续航  ──► crates/ap-guard/  (<2ms 双协议 exit-2 / JSON Hook 密钥护盾、3x 死循环熔断与门禁)
                           ──► crates/ap-baton/  (<400-tok 抗压缩状态胶囊 + DO NOT RETRY 跨工具接力)
```

---

## 4. 持续迭代的五步闭环纪律

1. **第 1 步：从真实轨迹中挖痛点（`ap audit --compare`）**：在 `Codex`、`Pi`、`Antigravity`、`Claude Code`、`Grok` 日常实战后运行本地审计（已在工作站实扫 `427` 个会话、`101,108` 次工具调用）。**没有真实观测到的轨迹摩擦，绝不凭空脑补新功能。**
2. **第 2 步：过「反淘汰四问」立项关**：(a) 明年模型智商翻倍后该痛点会自动消失还是属于物理边界？(b) 能否收进现有 7 个 Crate？(c) 模型能否不看 `--help` 直接通过 `--stdin` 零转义一把调通？(d) 负向开销边界在哪？
3. **第 3 步：独立 Rust Crate 实现与零警告门禁**：在 `crates/ap-<name>/` 内实现，确保 `SKILL.md` Frontmatter `< 85 tokens`、`cargo clippy --workspace --all-targets -- -D warnings` 零警告，并在 `tests/integration_primitives.rs` 补齐回滚、CRLF/UTF-8 编码与物理隔离测试。
4. **第 4 步：多宿主真实 CLI 受控实跑验效**：在真实 CLI（`codex`、`pi`、`agy`、`grok`、`claude`）中用同一任务与外部隐藏测试集对比实跑，深挖工具调用轨迹中的任何微小摩擦并打磨到底。
5. **第 5 步：双语同步、三端矩阵 CI 与跨平台二进制发布**：保持英文（`*.md`）与简体中文（`*.zh-CN.md`）1:1 同步，通过 Linux/macOS/Windows CI（`ci.yml`），并通过 `release.yml` 自动打包带 SHA-256 校验的 5 架构预编译二进制。

---

## 5. 已落地里程碑与下一阶段迭代清单

- [x] **P1 · 多语言可达性打包切片与零猜测测试运行器（`probe` + `splice`）**：已在 `v0.3` 落地，并在 `v1.0.0` 全面升级为原生 6 语言 **Tree-sitter** 具体语法树引擎。
- [x] **P2 · 一行命令多宿主零克隆接入与前后对比审计（`ap init` + `ap audit --compare`）**：已在 `v0.3` 落地，并在 `v1.0.0` 升级为零 Python 依赖原生二进制 + 覆盖 `Pi` 的 5 宿主审计。
- [x] **P3 · 6 语言 Tree-sitter 核心结构图与过期 TODO 雷达（`ap-atlas`）**：已在 `v1.0.0` 落地——原生支持 Python、TS/TSX、JS、Go、Rust、C 的 Tree-sitter 签名提取、`.gitignore` 感知中心度排序、`[dirty]` 脏文件标记与 `>90 天` 过期 TODO 预警（64 个 Rust 模块仅需 `16.3 ms`）。
- [x] **P3 · 个人助理 HTML `<table>`、CSV/TSV、JSONL 流提纯与 FTS5 检索（`ap-sieve` + `ap-splice`）**：已在 `v1.0.0` 落地——支持 HTML `<table>` 自动转 Markdown 管道表、CSV/TSV 摘要、JSONL 结构归纳、`<6 ms` 内置 SQLite FTS5 `sieve search`，以及零转义 `@@ FILE:` / `@@ SYMBOL:` / `@@ WRITE:` 补丁语法。
- [ ] **P4 · 跨语言调用图符号影响锥分析（`ap-probe impact <symbol>`）**：修改某个函数/类型签名时，在 `<15 ms` 内精准列出 Py/TS/Go/Rust/C 下游真实调用点，消除改漏调用方的返工。
- [ ] **P4 · Windows 下 `Pi`（`pi-coding-agent`）Git-Bash 路径零摩擦桥接**：针对 Windows 上 `Pi` 默认在 Git-Bash 中优先手写 `for f in src/*` 循环的习惯，自动注入 POSIX 斜杠路径与首轮调用引导。

---

## 6. 版本演进台账（Evolution Log）

| 日期 / 里程碑 | 核心交付能力 | 解决的真实过程痛点 | 实证验效数据 |
| :--- | :--- | :--- | :--- |
| **2026-11-15 · `v0.1` 基座立项** | 交付 `sieve`、`baton`、`recall`、`guard`、`atlas` + 含负向对照的确定性基准套件 | 上下文腐烂、压缩失忆、`AGENTS.md` 膨胀、虚假报完工与过期 TODO 陷阱 | CI 日志 **-98.8%**、JSON **-99.6%**、200 条中英规则唤起 **100% Top-1** |
| **2026-11-15 · `v0.2`–`v0.2.1` 多 CLI 实证升级与总手册** | 实扫 `69,963` 次调用；新增 `probe`（`slice`/`run`/`diff`）、`splice`（`view`/`apply`/`check`）与双语 `HANDBOOK` | `50.1%` 同文件重读、`55k` Token 脑补内存别名、引号转义报错与 Windows `cp936` 崩溃 | `Grok 1.0.77` 耗时 **-49.1%**、推理 **-65.4%**；`Antigravity 1.5.8` 净 Token **-49.8%**、`read_file` **-74.2%** |
| **2026-11-15 · `v0.3` 多语言与一行全宿主接入** | 一行 `init` 接入 `Codex`/`Pi`/`Antigravity`/`Claude`/`Grok`、`slice --bundle`、`check --test`、双协议 Hook 与 `audit --compare` | 15 文件串行 `read_file` 瀑布、82KB 循环追踪刷屏与多宿主配置割裂 | TS `ts-ledger`（`5/5` 验收）：`Grok 1.0.77` **耗时 -26.5%**、**Token -38.7%**；`Antigravity 1.5.8` **步数 -61.3%**、**编辑报错 -85.7%** |
| **2026-11-18 · `v1.0.0` 纯 Rust 单二进制引擎全面重构** | **100% 纯 Rust 重写（仓库 `0` `.py` 文件）**：单文件原生二进制 `ap` + 7 个物理隔离 Crate（`ap-*`）、6 语言 **Tree-sitter** AST（`Py/TS/JS/Go/Rust/C`）、**内置静态 SQLite FTS5**、零转义 `@@ FILE:` 语法树预检原子补丁、Node V8 `Proxy` + `rustc` 探针、`sieve search` + HTML表格/JSONL/CSV、`101,108` 次 5 宿主 `ap audit`，以及 `install.sh`/`install.ps1` + 5 架构预编译 Release | 彻底消除 Python 解释器启动开销（`40–90ms` → `0.7–16ms`）与环境依赖、词法括号启发式边界误差、JSON 补丁转义错误、管道 BrokenPipe 崩溃与 Windows `\\?\` 长路径兼容陷阱 | `splice view` **`0.68ms`（-95.4% tok）**、`apply` **`2.46ms`**、`recall` **`3.37ms`（20/20）**、`probe slice` **`6.32ms`**、`atlas` **`16.3ms`**，全量静默仅 **`412 tok`**；`Codex 0.167 + Rust ap v1.0` 实跑 24 文件 TS `ts-ledger`（`5/5` 满分）全程仅用 **`4` 条命令**（`161.8k tok`，`0` 次手读单文件，单次 `ap splice apply --stdin --test` 原子完成 13 处跨文件补丁 + 15 项测试）；`11/11` 集成测试全绿 & `0` Clippy 警告 |
