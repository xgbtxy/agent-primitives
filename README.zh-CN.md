# agent-primitives

**一个文件夹 = 一个不怕模型迭代淘汰的零依赖微基础设施（Coding Agent 与个人助手 Agent 底层过程提效底座）**

[English (Default)](./README.md) | [简体中文](./README.zh-CN.md) · **[核心理念与持续演进总手册 (中文)](./HANDBOOK.zh-CN.md)** | **[Handbook & Roadmap (EN)](./HANDBOOK.md)** · [底层架构](./DESIGN.zh-CN.md) · [实战指南](./PLAYBOOK.zh-CN.md)

---

## 最简一行命令接入全主流 Harness（`Codex` · `Pi` · `Antigravity` · `Claude Code` · `Grok`）

无需 `git clone`、无需 `pip install`、无需任何 `npm` 依赖或常驻后台。**复制一行命令**即可自动探测本机已装的 Agent Harness，同步全部 7 个独立原语，安装统一 `ap` 命令，并以幂等围栏注入不到 10 行的路由说明：

```bash
# macOS / Linux（自动识别并接入 Codex、Pi、Antigravity、Claude Code、Grok、OpenClaw、Hermes）：
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python3 - init
```

```powershell
# Windows PowerShell（一行零克隆即装即用）：
irm https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python - init
```

```bash
# 仅接入当前项目仓库（自动创建 .agents/skills、.agents/ap.py 并幂等写入 AGENTS.md 围栏）：
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python3 - init --here

# 指定特定 Harness 接入 + 自动无损挂载 PreToolUse / Stop 门禁 Hook：
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python3 - init -a codex,pi,agy,claude --with-hooks

# 随时一行命令干净卸载（100% 保留你原有的个人规则与配置）：
ap uninstall          # 或：python ap.py uninstall --here
```

### 各大主流 Harness 发现机制与原生一行安装对照表

不同 Harness 的全局技能目录、全局规则文件与 Hook 协议各不相同，`agent-primitives` 同时完成了**底层目录精确适配（`ap.py init`）**与**各生态官方原生插件清单适配**：

| 目标 Harness | 生态原生一行安装命令 | `ap init` 自动同步的技能与规则路径 | Hook 协议兼容（`--with-hooks`） |
| :--- | :--- | :--- | :--- |
| **OpenAI Codex CLI** (`codex`) | `codex plugin marketplace add xgbtxy/agent-primitives && codex plugin add agent-primitives@agent-primitives`<br>*(或一行 `ap.py init -a codex`)* | `~/.codex/skills/` + `~/.agents/skills/`<br>全局规则：`~/.codex/AGENTS.md` | `~/.codex/hooks.json` (`exit 2` snake_case) |
| **Pi** (`pi` / `pi-coding-agent`) | `pi install git:github.com/xgbtxy/agent-primitives`<br>*(或一行 `ap.py init -a pi`)* | `~/.pi/agent/skills/`（项目级 `.pi/skills/`）<br>全局规则：`~/.pi/agent/AGENTS.md` | `guard run` / `AGENTS.md` 路由 |
| **Google Antigravity** (`agy` / IDE) | `irm .../ap.py \| python - init -a agy --with-hooks`<br>*(或 `agy plugin install ./agent-primitives`)* | `~/.gemini/config/skills/` + `~/.agents/skills/`<br>全局规则：`~/.gemini/config/GEMINI.md` | `~/.gemini/config/hooks.json`（原生 camelCase JSON `deny`/`continue`） |
| **Claude Code** (`claude`) | `/plugin marketplace add xgbtxy/agent-primitives`<br>*(或一行 `ap.py init -a claude --with-hooks`)* | `~/.claude/skills/`（项目级 `.claude/skills/`）<br>全局规则：`~/.claude/CLAUDE.md` | `~/.claude/settings.json` (`exit 2` + `PreCompact` 自动快照) |
| **xAI Grok Build** (`grok`) | `irm .../ap.py \| python - init -a grok` | `~/.grok/skills/` *(实测优先级高于项目目录)*<br>全局规则：`~/.grok/AGENTS.md` | `guard` 命令门禁 |
| **通用生态 CLI** (`skills.sh`) | `npx skills add xgbtxy/agent-primitives -g` | 自动分发至 35+ Agent 技能目录 | 可叠加 `ap init --with-hooks` |

---

## 为什么要这样设计仓库？（为什么模型越强它越不会被淘汰？）

市面上 95% 的 Agent 开源项目活不过半年，原因只有一个：它们做的是**“思维拐杖”**（角色扮演提示词、步骤清单模板、重型编排外壳、绑卡向量库）——只要 `GPT-6`、`Grok-5`、`Claude-5`、`Gemini-4` 升级一轮后训练权重，这些拐杖就会瞬间沦为累赘。

**`agent-primitives`** 完全从本机 **`69,963` 次真实 CLI 工具调用轨迹**（覆盖 `Grok Build CLI`、`OpenAI Codex CLI`、`Google Antigravity CLI`、`Claude Code`）的实证瓶颈倒推设计。无论未来基座模型智商涨到多高，自回归 Token 流与真实操作系统之间永远横亘着**四道不可靠堆参数消除的物理鸿沟**：

1. **外部环境高熵噪音 vs. `O(N)` 上下文窗口物理定律** — `170 KB` 测试日志、`400` 条 API JSON 或十几个未被引用的死模块一旦灌入上下文，必然拖慢首字延迟、稀释注意力，并在后续每一轮重复计费。
2. **自然语言“脑补运行” vs. 真实内存堆与类型边界** — 让模型在文字里空想浅拷贝嵌套字典共享、原地修改异常未回滚、`None`/`bool`/`int`（如 `True == 1`）缓存碰撞，单题动辄空烧 `3.5万~5.5万` 推理 Token（耗时 `11~19 分钟`）；而手写临时 `python -c` 又在本机实测触发了 `2,716` 次试探与 `177` 次引号转义/`ImportError` 崩溃。
3. **BPE Token 生成流 vs. 字节/AST 级精确文件事务** — 修改几行导致行号偏移后被迫反复重读同一文件（Antigravity 实测同文件重复读取率高达 **`50.1%`**），外加 CRLF 补丁失配（`528` 次）与 Windows `cp936`/UTF-8 编码报错（`160` 次）。
4. **自动压缩失忆与自我汇报偏差** — `/compact` 会抹掉“哪条路已经试错失败”的负向约束，纯文字劝说也永远替代不了操作系统级确定性门禁拦截。

**核心设计准则**：**一个子文件夹 = 一个完全独立、零第三方依赖的确定性底层原语**（`cp -r skills/probe ~/.agents/skills/` 拷走单目录即用）。100% 仅依赖 Python 3.9+ 标准库（`0` pip 包、`0` 后台守护进程、`0` 云端 API Key，**7 个文件夹全装待机开销合计仅 `486 Tokens`**）。

---

## 第 0 步：先用 `ap.py audit` 测出你本机 Agent 的真实浪费（零上传）

安装前后随时运行仓库自带的**纯本地零联网审计器**，扫描本机 `~/.codex`、`~/.claude`、`~/.grok`、`~/.gemini/antigravity-cli` 历史日志并对比前后趋势：

```bash
python ap.py audit --compare
```

本机工作站实测输出（扫描最近 `178` 个真实会话、**`69,963` 次工具调用**，`0` 字节外发）：
- **命中 `skills/probe`**：捕获 `2,716` 次手写 `python -c` 临时试探、`106` 次 `ImportError`/`sys.path` 报错、`71` 次 Shell 引号转义崩溃；
- **命中 `skills/splice`**：捕获 `679` 次同文件重复重读、`3,918` 次手工命令行切片、`528` 次补丁失配、`160` 次 UTF-8/CP936 编码崩溃；
- **命中 `skills/sieve`**：捕获 `17,110` 次 `>8 KB` 巨型工具输出、`171` 次超长截断溢出；
- **命中 `skills/atlas`**：捕获 `2,994` 次宽泛目录/全文盲搜；
- **命中 `skills/guard + baton`**：捕获 `26` 轮完全相同的报错死循环重试。

---

## 7 个独立微基础设施：严丝合缝映射 Agent 真实 5 步执行闭环

```text
[ap.py audit --compare] ──► 量化本机真实摩擦损耗与前后提效趋势（100% 离线）
  ├─► 阶段 1：认路与精准唤起 ──► skills/atlas/  +  skills/recall/
  ├─► 阶段 2：切片与因果诊断 ──► skills/probe/   (slice --bundle · run · diff)
  ├─► 阶段 3：外科手术式落盘 ──► skills/splice/  (view · apply · check --test)
  ├─► 阶段 4：观测流无损降噪 ──► skills/sieve/   (run · pipe · peek)
  └─► 阶段 5：门禁与跨轮接力 ──► skills/guard/  +  skills/baton/
```

| 独立小项目文件夹 | 解决的物理边界瓶颈 | Coding Agent 真实提效 | 个人助手 Agent 真实提效 | 待机开销 |
| :--- | :--- | :--- | :--- | ---: |
| **[`skills/atlas/`](./skills/atlas/README.zh-CN.md)**<br>([EN](./skills/atlas/README.md)) | **冷启动盲摸全仓** + 被陈旧 `TODO` 诱饵多耗 `77%` Token | `~20ms` 提取 AST 类/函数签名、高引用核心枢纽与 **`>90天` 陈旧 TODO 陷阱预警** | `<800 Tokens` 秒级生成大型个人 Markdown 知识库目录分布与 `H1/H2` 标题树 | `73 tok` |
| **[`skills/recall/`](./skills/recall/README.zh-CN.md)**<br>([EN](./skills/recall/README.md)) | **`AGENTS.md` / `MEMORY.md` 静态肥胖税**每轮白烧几千 Token | `<5ms` 标准库 `sqlite3` FTS5/BM25 + 路径通配符（碰 `src/auth/*` 只注入 `~40 tok` 铁律） | 原生中英双栖混合检索，`<5ms` 按报销、合同、客户、服务器精准唤醒个人 SOP | `71 tok` |
| **[`skills/probe/`](./skills/probe/README.zh-CN.md)**<br>([EN](./skills/probe/README.md)) | **误读死代码、5 万 Token 文字脑补内存**与 `python -c` 崩溃 | 多语言 `slice --bundle` 1 次调用剪除 Py/TS/JS/Go/Rust 死文件并打包活跃源码；循环自动去重 `run` 秒抓 `[ALIAS HAZARD]`/`[IN-PLACE MUTATION]`/`[STATE NO-OP]`；`diff` 测边界碰撞 | 对个人数据清洗、财务对账、格式转换脚本预跑 `diff`/`run`，批处理前自动揪出空值与精度越界 | `62 tok` |
| **[`skills/splice/`](./skills/splice/README.zh-CN.md)**<br>([EN](./skills/splice/README.md)) | **`50.1%` 同文件重读**、CRLF 失配与盲猜测试命令报错 | 多语言 `view --symbol`（省 **94.4%** Token）；强制 UTF-8/保留 CRLF 原子回滚 `apply` + 自动清 `.pyc`；`check --test` 自动识别并跑通 `pytest`/`unittest`/`node --test`/`go`/`cargo` | `view --heading` 精读长手册指定章节；`apply` 原子更新长篇笔记与 JSON 配置，杜绝乱码写坏 | `64 tok` |
| **[`skills/sieve/`](./skills/sieve/README.zh-CN.md)**<br>([EN](./skills/sieve/README.md)) | **长日志/大 JSON/网页灌爆上下文（Context Rot）** | `pytest`/`cargo`/CI 日志压减 **98.8%**；需要细节直接 `peek <id>` 毫秒级无损回放，免重跑慢测试 | 400 条 API JSON 压减 **99.6%**、噪音网页 HTML 压减 **98.4%**，支持 `--json-path` 精查 | `83 tok` |
| **[`skills/guard/`](./skills/guard/README.zh-CN.md)**<br>([EN](./skills/guard/README.md)) | **未过检虚报“已完成”**、死循环重试与密钥外泄 | 双协议自适应完工门与事前护盾（兼容 Claude/Codex `exit 2` 与 Antigravity camelCase `deny`/`continue` JSON）+ 3 次同错死循环熔断 | 事前硬拦截 API Key 外泄、`rm -rf` 误删与未经确认的批量邮件/Webhook 外发事故 | `63 tok` |
| **[`skills/baton/`](./skills/baton/README.zh-CN.md)**<br>([EN](./skills/baton/README.md)) | **自动压缩失忆**与跨终端/跨 Agent 丢失试错结论 | 在 Claude ↔ Codex ↔ Pi ↔ Grok ↔ Antigravity 间无损传递硬事实、Git 状态与 **`DO NOT RETRY` 死胡同** | 跨天推进深度调研、采购比价与长线待办，压缩后 `<400 Tokens` 满血恢复 | `70 tok` |

---

## 三大主流 CLI 跨语言实机受控 A/B 实测（100% 真实轨迹，零编造）

我们在本机对 **Grok Build CLI (`1.0.77`, `grok-4.7`)**、**Google Antigravity CLI (`1.5.8`)** 与 **OpenAI Codex CLI (`0.167.0`, `gpt-6-astra`)** 进行了 Python 与 TypeScript 双语言真实多文件受控 A/B 实测（`Task C: ts-ledger` 24 文件 TypeScript/Node 22 清算账本，含 13 个未引用干扰 `.ts` 模块与 5 重跨文件分摊精度/幂等碰撞/浅拷贝别名/批次非原子回滚缺陷；`Task B: rulelang` 25 文件 Python 规则编译器/优化器/缓存 5 重缺陷），统一由外部隐藏测试集（Oracle）独立判分：

| 实测 CLI 与任务 | 对照模式 | 墙钟耗时 / 工具步数 | `read_file` / 编辑报错次数 | 内部推理/思维 Token | 总吞吐 / 净 Token 与实付账单 | 隐藏 Oracle |
| :--- | :--- | ---: | ---: | ---: | ---: | :---: |
| **Grok CLI `1.0.77`** (`grok-4.7`) · *Task C (TypeScript `ts-ledger`)* | 原生基线 | `766.6s` / `36` 步 (`14` 轮) | `18` 次 `read_file` | `37,426 tok` | `683,173 tok` (`$0.86`) | `5/5` |
| **Grok CLI `1.0.77`** (`grok-4.7`) · *Task C (TypeScript `ts-ledger`)* | **`+ ap init (v0.3)`** | **`563.2s` (-26.5% 净省 3.4 分钟)** / **`23` 步 (-36.1%)** | **`6` 次 `read_file` (-66.7%)** | **`31,748 tok` (-15.2%)** | **`418,623 tok` (-38.7%，账单 `$0.56` = 立省 34.4%)** | **`5/5`** |
| **Antigravity `1.5.8`** (`agy`) · *Task B (Python `rulelang`)* | 原生基线 | `1,039.1s` / `42` 步 | `31` 次 `read_file`、`1` 次崩溃 | `50,680 tok` | `401,064 net tok` (`334.4k` 输入) | `5/5` |
| **Antigravity `1.5.8`** (`agy`) · *Task B (Python `rulelang`)* | **`+ ap init (v0.3)`** | **`789.5s` (-24.0% 净省 4.2 分钟)** / **`24` 步 (-42.9%)** | **`8` 次 `read_file` (-74.2%)**、零转义崩溃 | **`43,060 tok` (-15.0%)** | **`201,342 net tok` (-49.8%，输入 `-57.8%`)** | **`5/5`** |
| **Antigravity `1.5.8`** (`agy`) · *Task C (TypeScript `ts-ledger`)* | 原生 → **`+ ap init`** | `31` → **`12` 步 (-61.3%)** | `15` → **`7` 次 `read_file` (-53.3%)**；`7` → **`1` 次报错 (-85.7%)** | `22,210` → `24,063 tok` | `435,630` → **`369,397 gross` (-15.2%，净省 `6.6万`)** | **`5/5` → `5/5`** |
| **Grok CLI `1.0.77`** (`grok-4.7`) · *Task B (Python `rulelang`)* | 原生 → **`+ probe + splice`** | `693.1s` → **`352.6s` (-49.1% 提速近一倍)** | `8` → **`0` 次 `search_replace`** | `36,013` → **`12,477 tok` (-65.4%)** | `$0.59` → **`$0.54` (-7.3%)** | **`5/5` → `5/5`** |
| **Codex CLI `0.167.0`** (`gpt-6-astra`) · *Task B (Py) & Task C (TS)* | 原生 → **`+ ap init`** | *Task C*：`136.9s` → **`126.9s` (-7.3%)**；*Task B*：`7` → **`6` 步 (-14.3%)** | 全程 `0` 报错（`probe slice --bundle` + `splice check --test`） | *Task C*：`31` → **`24` (-22.6%)**；*Task B*：`96` → **`53` (-44.8%)** | *Task B*：`180.8k` → **`174.7k` (-3.3%)** *(Task C 见诚实注记)* | **`5/5` → `5/5`** |

> **诚实工程结论**：在多轮交互型 Harness（`Grok CLI`、`Antigravity CLI`）上，一行 `ap init` 在 Python 与 TypeScript 项目中均将 `read_file` 次数砍掉 **53%~74%**、编辑报错砍掉 **85.7%**、净 Token 与实付 API 成本降低 **15%~50%**、墙钟耗时缩短 **24%~49%**。在自带原生内核 `apply_patch` 的 5 步极速流（`Codex CLI` `Task C`）上，`ap init` 依然实现了 **7.3% 墙钟提速（`126.9s` vs `136.9s`）与 22.6% 推理精简**，同时因构造 JSON 原子事务相比内置补丁指令多用约 `4.6 万` 缓存输入 Token——真实数据全量公开，绝不报喜不报忧。

---

## 可复现确定性微基准实测（含诚实负对照）

本机运行 `python bench/run_bench.py` 即可复现 [`bench/results.json`](./bench/results.json)（Python 3.12，固定种子 `20261115`，零外部包）：

| 基础设施与测试负载 | 原始体积 / 基线 | 接入原语后 | 缩减 / 提效幅度 | 本机延迟 | 保真度与确定性保障 |
| :--- | ---: | ---: | ---: | ---: | :--- |
| **`probe`** · 25 文件 Python + 24 文件 TS 切片与追踪 | `25` Py / `24` TS 源码文件 | `3` Py / `9` TS 活跃文件（剪除 `20` / `15` 个死文件） | **-80.0% / -62.5% 死模块** | `24.3 ms` / `14.4 ms` | 自动捕获 `2` 处堆别名、`1` 处原地污染、`1` 处状态空操作 |
| **`splice`** · 30 方法 CRLF 模块精读 + 原子补丁 | `1,376 tok` 读全文件 | `77 tok` 精读目标方法 | **-94.4% 读取开销** | `1.7 ms` / `5.5 ms` | 严格保留 CRLF 与 UTF-8 中文头；语法报错 100% 字节级回滚 |
| **`sieve`** · 2,500 行 CI/pytest 报错日志 | `172,075 B` (`~43,019 tok`) | `2,080 B` (`~520 tok`) | **-98.8%** | `31.0 ms` | 100% 异常栈保留 + 无损 `peek` |
| **`sieve`** · 400 条记录 API JSON 响应 | `107,887 B` (`~26,972 tok`) | `423 B` (`~106 tok`) | **-99.6%** | `5.1 ms` | 顶层结构与告警完整保留 + `--json-path` 精查 |
| **`sieve`** · 含大量 JS/CSS 噪音的网页 HTML | `48,160 B` (`~12,040 tok`) | `780 B` (`~195 tok`) | **-98.4%** | `12.0 ms` | 标题、H1–H3 大纲与正文核心完整保留 |
| **`sieve`** · `<1.2 KB` 短输出 *(诚实负对照)* | `69 B` (`~18 tok`) | `117 B` (`~30 tok`) | **0.0%** *(原样直通)* | `2.2 ms` | 短输出不强行摘要，杜绝形式主义负优化 |
| **`recall`** · 200 条中英双栖规则库检索 | `6,380 tok` *(全量塞入)* | `39.9 tok` *(Top-2 注入)* | **-99.4%** | `4.9 ms` | **20/20 (100%)** Top-1 命中率 |
| **`atlas`** · 64 模块 SDK 代码仓 | `42,710 tok` *(全量读取)* | `446 tok` *(结构地图)* | **-99.0%** | `36.7 ms` | AST 类/函数签名、核心枢纽与入口完整提取 |

---

## 仓库目录结构与测试校验

```text
agent-primitives/
├── ap.py                            # 一行零克隆引导器、多 Harness 自动接入器与统一 CLI
├── README.md / README.zh-CN.md      # 默认英文首页 + 完整中文首页（含一行接入矩阵与跨语言实测数据）
├── HANDBOOK.md / HANDBOOK.zh-CN.md  # 核心理念宪章、六条工程红线、五步持续迭代闭环与演进台账
├── DESIGN.md / DESIGN.zh-CN.md      # 抗淘汰四道物理边界底层架构深潜
├── PLAYBOOK.md / PLAYBOOK.zh-CN.md  # 各大 Harness 一行接入详解与 Coding / 个人助手分场景指南
├── package.json / .*-plugin/        # 零依赖原生插件清单（兼容 Pi `pi install`、Codex、Antigravity 与 Claude）
├── audit/audit_sessions.py          # 本地零云端会话病理审计器 + `--compare` 前后周期对比
├── skills/                          # 一个子文件夹 = 一个独立零依赖微基础设施
│   ├── atlas/                       # 阶段 1：亚秒级 AST 与知识库结构地图 + >90天陈旧 TODO 雷达
│   ├── recall/                      # 阶段 1：<5ms SQLite FTS5/BM25 中英双栖触发式微记忆
│   ├── probe/                       # 阶段 2：多语言可达性打包切片、堆别名/篡改显微镜与差分预言机
│   ├── splice/                      # 阶段 3：多语言符号/章节精读、原子回滚补丁与自动测试门禁
│   ├── sieve/                       # 阶段 4：无损输出降噪压缩器与本地 FTS5 黑匣子
│   ├── guard/                       # 阶段 5：双协议自适应完工真相门、3 次同错熔断与密钥护盾
│   └── baton/                       # 阶段 5：抗压缩状态胶囊与跨 Agent 死胡同接力棒
├── bench/run_bench.py               # 确定性基准测试套件（含诚实负对照）
└── tests/test_primitives.py         # 14 组物理隔离、零克隆引导、多语言、双协议 Hook 与回滚单测
```

```bash
python ap.py doctor
python -m unittest discover -s tests -v
```

## 开源协议

[MIT](./LICENSE)
