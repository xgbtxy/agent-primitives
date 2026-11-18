# `agent-primitives` (`ap` v1.8) — 核心研发理念与持续演进手册

[English](./HANDBOOK.md) | [简体中文](./HANDBOOK.zh-CN.md) · [首页](../README.zh-CN.md) · [架构深潜](./DESIGN.zh-CN.md) · [实战手册](./PLAYBOOK.zh-CN.md)

> **项目定位**：本仓库不做花哨的多智能体外壳，也不写冗长易被忽略的提示词套路（AI Slop）。我们坚持一条朴素而硬核的工程路线——**持续观察真实 Coding Agent 的运行过程与思考链，深度调研并吸收各个成熟开源项目的实战经验，用高性能单文件纯 Rust 夯实大模型的底层 Coding 基础能力**，让哪怕是零基础小白使用免费 Coding Agent 也能获得实打实的效率与代码质量提升。

---

## 1. 我们的三条核心研发理念

### 1.1 从「观察真实 Coding Agent 的实际运行过程」出发，绝不凭空臆想
任何脱离真实轨迹的“提效点子”都是伪需求。我们直接深入 `OpenCode`、`Codex`、`Pi`、`Claude Code`、`Antigravity`、`Qwen Code`、`Gemini CLI`、`Grok` 等智能体在真实任务（小白一句话需求、存量代码排障、空目录从零构建随机项目）中的 **`101,108+` 次工具调用日志与原始 `<thinking>` 推理流**，盯着它怎么跑、卡在哪：
- **前 1–5 轮卡在哪？** 耗费上万思考字符盲猜本机环境（装没装 `node:sqlite`？`--experimental-strip-types` 能不能用 `enum`？有没有浏览器？），或把仓库里未引用的旧废文件读进上下文，照猫画虎写出 `: any` 和空 `catch {}`。
- **第 6–15 轮卡在哪？** 多文件连续修改时出现工作记忆漂移（HTML `id="..."` 与 JS `getElementById` 名字对不上、前端 `fetch('/api/...')` 与后端路由路径不一致），或因 2 空格缩进偏差、Windows PowerShell 引号转义反复打补丁失败。
- **后 70% 的时间与 Token 烧在哪？** 实测原生智能体在 Windows 上有 **`71%–75%` 的步骤**耗在启动服务卡死终端、残留僵尸端口、以及手写上百行 Chrome DevTools 脚本验证页面交互上。
- **为什么交付的代码仍有低级毛病？** 写在 `AGENTS.md` 里的文字提醒在长上下文里极易被忽略，只有工具层的确定性检查与 **`exit 2` 硬门禁**才能让模型自己发现问题、自己改干净。

### 1.2 深度调研成熟开源项目经验，集百家之长而非从 0 到 1 重复造轮子
工业界与开源社区已经沉淀了大量经过实战检验的优秀机制。我们的做法是**深度拆解各家核心精髓，去芜存菁，用零依赖纯 Rust 高性能实现并无缝串联**：
- **吸收 `Aider` 的核心经验**：引入基于 Tree-sitter 导入图的 **Personalized PageRank（8 轮幂迭代中心度排序）** 精准提取仓库核心骨架；引入 **三级补丁容错匹配（精确匹配 → 行尾空白归一化 → 相对缩进自动平移）** 消除缩进偏差导致的补丁重试。
- **吸收 `RTK (rtk-ai/rtk)` 的核心经验**：在 `PreToolUse` Hook 中支持 `updatedInput` 透明重写，自动将高噪音测试命令接入无损压缩，无需依赖模型自觉。
- **吸收 `SWE-agent` (ACI) 的核心经验**：补丁落盘前强制校验 6 语言 Tree-sitter 语法树；匹配失败时直接回传最近锚点行号（`closest anchor at L42`），省去重读全文件；启动即输出宿主运行时指纹，消除环境盲猜。
- **拥抱官方 `MCP` 协议与成熟本地生态**：内置零依赖 **`ap mcp`** 标准 JSON-RPC 服务，让不擅长复杂 Shell 转义的免费模型也能直接做结构化调用；自动驱动系统自带无头 **Edge/Chrome CDP** 完成真实点击输入，并直接复用项目已有的 `tsc`、`biome`、`cargo check`、`pytest` 等成熟检查器。

### 1.3 以「Coding 基础能力与交付质量的真实提升」为唯一标尺（小白友好、零 AI Slop）
- **对用户极简**：一行命令安装（无需装 Python、Node 或 Rust），自动识别本机免费与商业 Coding Agent 完成接线。小白不需要学任何子命令参数，像平时一样用大白话提需求即可。
- **对代码极严（十年老开发准则）**：聚焦性能（核心路径 `0.7–18 ms`、`OnceLock` 零重复正则编译）、代码简洁（7 个物理隔离 Crate `0` 兄弟依赖、`0` `clippy` 警告、静默描述合计仅 `390 tokens`）与结果干净（拦截 `: any`、`@ts-ignore`、吞异常空 `catch {}`）。

---

## 2. 持续演进的四步工程闭环

后续围绕本仓库的每一次迭代更新，严格执行以下四步：

1. **第 1 步：观察实跑过程（Observe）** —— 运行 `ap audit --compare`，并在真实 Coding Agent（含 `$0` 免费智能体）上跑通普通用户任务，拆解工具调用链与 `<thinking>` 损耗点。
2. **第 2 步：调研成熟方案（Survey）** —— 横向调研顶尖开源项目解决同类工程痛点的成熟算法与交互协议，拒绝闭门造车与提示词堆砌。
3. **第 3 步：纯 Rust 精炼实现（Craft）** —— 以最小代码复杂度合入对应独立 Crate（同时覆盖 CLI 与 `ap mcp`），通过 `rustfmt`、`clippy -D warnings` 与跨平台集成测试。
4. **第 4 步：真机对照验效（Verify）** —— 在原生未接入 vs. `+ ap` 条件下做受控实测，核验速度、Token 消耗、一次跑通率与代码洁净度的真实增益，如实记入演进台账。

---

## 3. 架构映射与版本演进台账

```text
[一行接入 ap init] ──► 自动接线 OpenCode / Qwen / Gemini CLI / Goose / Pi / Codex / Claude / Antigravity / Grok
   ├── 定向与排雷 (ap_orient) ──► ap-atlas (PPR 核心地图 + 宿主指纹 + DOM #id/路由断链与防 Slop 雷达) + ap-probe slice
   ├── 精准修改   (ap_edit)   ──► ap-splice (Aider 三级缩进容错 + 零转义多文件原子修改 + 落盘前 Tree-sitter 语法校验)
   ├── 真机自验   (ap_smoke)  ──► ap-probe smoke (空闲端口托管 + 纯 Rust 无头 Edge/Chrome 交互式点击输入 + 杀净进程树)
   ├── 质量硬门禁 (ap_verify) ──► ap-guard (exit-2 防 Slop 规则 + 自动联动 tsc/biome/cargo/测试 + RTK 式自动脱水)
   └── 诊断与续航 (ap_probe)  ──► ap-probe run/diff + ap-sieve (-98.8% 日志压缩) + ap-recall (3.4ms BM25) + ap-baton
```

| 里程碑 | 观察到的真实过程卡点 & 融合的开源经验 | 真机实测提效结果 |
| :--- | :--- | :--- |
| **`v0.1`–`v0.3` (2026-11-15)** | 审计 `6.9 万` 次调用定位重读与日志刷屏；建立 7 原语与多宿主一行接入 | `Grok 1.0.77` **耗时 -26.5%～-49.1%**；`Antigravity 1.5.8` **净 Token -49.8%**、`read_file` **-74.2%** |
| **`v1.0.0` (2026-11-18)** | 消除 Python 启动延迟与环境依赖；重写为**纯 Rust 单二进制 + 7 隔离 Crate**（内嵌 6 语言 Tree-sitter + SQLite FTS5） | `splice view` **`0.71ms`**、`apply` **`2.33ms`**、`recall` **`3.36ms`**；`Codex` 仅用 **`4` 条命令**满分修复 24 文件 `ts-ledger` |
| **`v1.2.0` (2026-11-18)** | 拆解 `<thinking>` 发现 70%+ 验证税与跨文件记忆漂移；融合 **Aider PPR & 三级缩进容错**、**RTK 自动脱水**、**交互式 CDP** 与 **`ap mcp`** | **免费智能体 `OpenCode Free ($0)` 小白需求**：**快 `2.36 倍`（`430s→182s`）、Token `-75.1%`、`7/8→8/8` 零 Slop**；**空目录随机项目 `SplitMate`**：`Codex` **Token `-83.3%`（省 `6.0 倍`）**，`Pi` **`5/6→6/6` 根除吞错 Slop（快 `322s`）** |
| **`v1.3.0` (2026-11-18)** | 扫描 160 个真实 Windows 会话抓出 **`528 次` OS/Shell 翻车**（235 次 GBK/BOM、138 次 PS `&&`/`curl`/heredoc/PATH、82 次端口锁、73 次管道假死）；打造 **Windows 透明命令护盾 (`ap exec` + Base64 `PreToolUse` Hook)**、**UTF-8 BOM/UTF-16LE 自动净化** 与 **`ap stats` 离线提效账单**，并将外部 UX 极简化为 3 个命令（`init`/`doctor`/`stats`） | **Windows 5 大真实陷阱对拍**：原生 PS 5.1 全军覆没（GBK 崩溃/`&&` 与 heredoc `ParserError`/端口占用），`ap exec` **100% 一次自愈通过**，2000 行构建日志自动脱水 **`-99.1%`（`110.3KB → 0.9KB`）**，`ap stats` 实时量化省流收益 |
| **`v1.4.0` (2026-11-18)** | 攻坚跨文件重构 **「改 2 漏 5」影响面盲区** 与系统级孤儿进程：打造 **`<15ms` 语法树引用与影响面引擎 (`atlas impact` / `ap_impact`)**、**改码自动提醒漏改调用方 `[BLAST RADIUS]`**、**Windows NT 内核 `Job Object` 进程树防死锁守卫**，并新增 **`Cursor` / `Windsurf` / `VS Code` 自动接入** | **14 文件跨模块函数签名重构 (`PayLedger`)**：`Codex` **`151.6s → 96.6s`（快 `1.57 倍`）、Token `-57.4%`（`296k → 126k`）、`7/7` 满分**；`Pi (deepseek-flash)` **`102.6s → 90.5s`、Token `-24.0%`（`256k → 195k`）、`7/7` 零 Slop** |
| **`v1.5.0` (2026-11-18)** | 从编程延伸至**所有通用智能体的四大底层原子操作**：打造**免 Python 原生万能读取器 `ap read` / `ap_read`**（直读 `.docx`/`.xlsx`/`.pptx`/`.pdf`/`GBK` 账单与网页防撑爆）、**无 Git 桌面文件夹自动快照与一键撤回 `ap undo`**（含 `>75%` 异常截断预警）及 **`Claude Desktop` 自动接入** | **无 Python 普通人 Windows 电脑非编程办公实测（`.docx` 合同 + `.xlsx` 台账 + `GBK` 银行流水）**：原生先读出乱码又调 Python 失败（`70.5s`、`126.8k` tok）→ **`+ ap v1.5.0`**：**`46.5s`（快 `1.52 倍`）、`58.0k` tok（`-54.3%`）、`5/5` 一次全对** |
| **`v1.6.0` (2026-11-18)** | 攻克大模型多表算数幻觉、无 `rg` 搜不到文档与动态 SPA 空壳网页三大上限：打造**免 Python 跨文件 SQLite 引擎 `ap query` / `ap_query`**（支持多工作表 `.xlsx` 自动合成 `{alias}__all`、与 `GBK .csv`/`.json` 直接联表 SQL）、**语法树符号 + Office/PDF 统一检索 `ap search` / `ap_search`**，以及**无头 Edge 动态 SPA 自动渲染快照 `ap read --render`** | **2,500 行双工作表 `.xlsx` + `GBK` 流水 + `.docx` + 动态 JS 政策页联合稽核**：原生连写 5 遍 XML/Decimal 脚本（`118.4s`、`212.1k` tok）→ **`+ ap v1.6.0`**：**`94.3s`（快 `1.26 倍`）、`137.6k` tok（`-35.1%`）、`5/5` 精确到分（`84,224.82`）** |
| **`v1.7.0` (2026-11-18)** | 复盘 `v1.6` 轨迹并结合 SWE-bench 补丁损耗与 SQLite MCP 高频场景：打造**全域 X 光地图 `atlas map`**（`<15ms` 同步透视代码、`.xlsx` 工作表名、`GBK` 表头与 `.sqlite` 库）、**`ap query` 原生挂载 `.sqlite`/`.db` 跨表联查 + 消除 IEEE-754 浮点尾数噪音**、**`ap read` 原生解构 `.sqlite`/`.gz`/`.tar.gz`/图片尺寸头并自动脱水 `[FATAL]` 压缩日志**，以及 **`ap splice` 零截断 `@@ APPEND:` + Git Unified Diff 自动解析 + 四级空行容错补丁** | **屏蔽 `py / sqlite3` 全栈排障与跨库稽核（`.sqlite` + 3,000 行双表 `.xlsx` + `GBK .csv` + `.log.gz` + 代码单测修复）**：原生手拼 `.NET` 解压与 4 遍脚本（`112.0s`、`235.0k` tok）→ **`+ ap v1.7.0`**：**`94.3s`（快 `1.19 倍`）、`167.5k` tok（`-28.7%`）、`6/6` 精确到分（`150,170.90`）** |
| **`v1.8.0` (2026-11-18)** | 聚焦 **「创造（Creation）与测试（Testing）」** 核心上限，吸收 Meta `ACH` / `Mutahunter` 变异引导与跨文件契约经验：打造 **`<5ms` 跨文件导出名与实参个数静态链接器（`splice check` + `[CONTRACT WARNING]`）**、**全写接口混沌与 Barrier 多线程并发 Fuzzer（`probe fuzz` / `ap_fuzz`）**、**2 秒级多语言语法树变异测试引擎（`probe mutate` / `ap_mutate`）**、**防 `.only(` 跳测作弊与失败栈源码定点提取**，并融合成单次跑完五道关卡的 **`ap guard verify --deep`** | **`FlashVault` 从零创造**：单测变异杀伤率从原生 **`62.5%` (`5/8`) 跃升至 `100.0%` (`8/8` 零漏检、黑盒 `10/10`)**，且 `--deep` 融合门禁将调用步数从 **`21` 步压至 `7` 步（`-66.7%`）**；**`OrderGate` 假绿灯排雷加固**：耗时 **`200.6s → 183.2s`** 反超原生，**`6/6` + 变异率 `100%` + 混沌 Fuzz 全通** |
