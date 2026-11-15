# agent-primitives

**一个文件夹 = 一个不怕模型迭代淘汰的零依赖微基础设施（Coding Agent 与个人助手 Agent 底层过程提效底座）**

[English (Default)](./README.md) | [简体中文](./README.zh-CN.md) · **[核心理念与持续演进总手册 (中文)](./HANDBOOK.zh-CN.md)** | **[Handbook & Roadmap (EN)](./HANDBOOK.md)** · [底层架构](./DESIGN.zh-CN.md) · [实战指南](./PLAYBOOK.zh-CN.md)

---

## 为什么要这样设计仓库？（为什么模型越强它越不会被淘汰？）

市面上 95% 的 Agent 开源项目活不过半年，原因只有一个：它们做的是**“思维拐杖”**（角色扮演提示词、步骤清单模板、重型编排外壳、绑卡向量库）——只要 `GPT-6`、`Grok-5`、`Claude-5`、`Gemini-4` 升级一轮后训练权重，这些拐杖就会瞬间沦为负资产。

**`agent-primitives`** 完全从本机 **`69,953` 次真实 CLI 工具调用轨迹**（覆盖 `Grok Build CLI`、`OpenAI Codex CLI`、`Google Antigravity CLI`、`Claude Code`）的实证瓶颈倒推设计。无论未来基座模型智商涨到多高，自回归 Token 流与真实操作系统之间永远横亘着**四道不可靠涨参数消除的物理鸿沟**：

1. **外部环境高熵噪音 vs. `O(N)` 上下文窗口物理定律** — `170 KB` 测试日志、`400` 条 API JSON 或 `20` 个未被引用的死模块一旦灌入上下文，必然拖慢首字延迟、稀释注意力，并在后续每一轮重复计费。
2. **自然语言“脑补运行” vs. 真实内存堆与类型边界** — 让模型在文字里空想浅拷贝嵌套字典共享、原地修改异常未回滚、`None`/`bool`/`int`（如 `True == 1`）缓存碰撞，单题动辄空烧 `3.5万~5.5万` 推理 Token（耗时 `11~19 分钟`）；而手写临时 `python -c` 又在本机实测触发了 `2,716` 次试探与 `177` 次引号转义/`ImportError` 崩溃。
3. **BPE Token 生成流 vs. 字节/AST 级精确文件事务** — 修改几行导致行号偏移后被迫反复重读同一文件（Antigravity 实测同文件重复读取率高达 **`50.1%`**），外加 CRLF 补丁失配（本机审计 `528` 次）与 Windows `cp936`/UTF-8 编码报错（`160` 次）。
4. **自动压缩失忆与自我汇报偏差** — `/compact` 会抹掉“哪条路已经试错失败”的负向约束，纯文字劝说也永远替代不了 **`exit 2`** 物理门禁拦截。

**核心设计准则**：**一个子文件夹 = 一个完全独立、零第三方依赖的确定性底层原语**（`cp -r skills/probe ~/.agents/skills/` 拷走单目录即用）。100% 仅依赖 Python 3.9+ 标准库（`0` pip 安装、`0` 后台守护进程、`0` 云端 API Key，**7 个文件夹全装待机开销合计仅 `492 Tokens`**）。就像更聪明的程序员不会淘汰 `git`、`strace` 和 `sqlite` 一样，**基座模型越强，调用确定性原语就越准、越快**。

---

## 第 0 步：先用 `audit/` 测出你本机 Agent 的真实浪费（零上传）

安装任何工具前，先用仓库自带的**纯本地零联网审计器**扫描你机器上的 `~/.codex`、`~/.claude`、`~/.grok`、`~/.gemini/antigravity-cli` 历史日志：

```bash
python audit/audit_sessions.py
```

本机工作站实测输出（扫描最近 `178` 个真实会话、**`69,953` 次工具调用**，`0` 字节外发）：
- **命中 `skills/probe`**：捕获 `2,716` 次手写 `python -c` 临时试探、`106` 次 `ImportError`/`sys.path` 报错、`71` 次 Shell 引号转义崩溃；
- **命中 `skills/splice`**：捕获 `679` 次同文件重复重读、`3,918` 次手工命令行切片、`528` 次补丁失配、`160` 次 UTF-8/CP936 编码崩溃；
- **命中 `skills/sieve`**：捕获 `17,110` 次 `>8 KB` 巨型工具输出、`171` 次超长截断溢出；
- **命中 `skills/atlas`**：捕获 `2,994` 次宽泛目录/全文盲搜；
- **命中 `skills/guard + baton`**：捕获 `26` 轮完全相同的报错死循环重试。

---

## 7 个独立微基础设施：严丝合缝映射 Agent 真实 5 步执行闭环

```text
[audit/audit_sessions.py] ──► 先量化本机真实摩擦损耗（100% 离线）
  ├─► 阶段 1：认路与精准唤起 ──► skills/atlas/  +  skills/recall/
  ├─► 阶段 2：切片与因果诊断 ──► skills/probe/   (slice · run · diff)
  ├─► 阶段 3：外科手术式落盘 ──► skills/splice/  (view · apply · check)
  ├─► 阶段 4：观测流无损降噪 ──► skills/sieve/   (run · pipe · peek)
  └─► 阶段 5：门禁与跨轮接力 ──► skills/guard/  +  skills/baton/
```

| 独立小项目文件夹 | 解决的物理边界瓶颈 | Coding Agent 真实提效 | 个人助手 Agent 真实提效 | 待机开销 |
| :--- | :--- | :--- | :--- | ---: |
| **[`skills/atlas/`](./skills/atlas/README.zh-CN.md)**<br>([EN](./skills/atlas/README.md)) | **冷启动盲摸全仓** + 被陈旧 `TODO` 诱饵多耗 `77%` Token | `~20ms` 提取 AST 类/函数签名、高引用核心枢纽与 **`>90天` 陈旧 TODO 陷阱预警** | `<800 Tokens` 秒级生成大型个人 Markdown 知识库目录分布与 `H1/H2` 标题树 | `73 tok` |
| **[`skills/recall/`](./skills/recall/README.zh-CN.md)**<br>([EN](./skills/recall/README.md)) | **`AGENTS.md` / `MEMORY.md` 静态肥胖税**每轮白烧几千 Token | `<4ms` 标准库 `sqlite3` FTS5/BM25 + 路径通配符（碰 `src/auth/*` 只注入 `~40 tok` 铁律） | 原生中英双栖混合检索，`<4ms` 按报销、合同、客户、服务器精准唤醒个人 SOP | `71 tok` |
| **[`skills/probe/`](./skills/probe/README.zh-CN.md)**<br>([EN](./skills/probe/README.md)) | **误读死代码、5 万 Token 文字脑补内存运行态**与 `python -c` 崩溃 | `slice` 秒级剪除死模块（剪除率 **80%**）；`run --stdin` 自动抓出 `[ALIAS HAZARD]`、`[IN-PLACE MUTATION]` 与 `[STATE NO-OP]`；`diff` 测边界碰撞 | 对个人数据清洗、财务对账、格式转换脚本预跑 `diff`/`run`，在批处理前自动揪出空值与精度越界 | `64 tok` |
| **[`skills/splice/`](./skills/splice/README.zh-CN.md)**<br>([EN](./skills/splice/README.md)) | **`50.1%` 同文件重复重读**、CRLF 失配与 Windows 编码崩溃 | `view --symbol Cls.fn`（省 **94.4%** Token）；`apply` 强制 UTF-8/保留 CRLF/语法预检**原子多文件回滚补丁** + 自动清 `.pyc`；`<15ms` `check` | `view --heading` 精读长手册指定章节；`apply` 原子更新长篇笔记与 JSON 配置，杜绝乱码写坏 | `68 tok` |
| **[`skills/sieve/`](./skills/sieve/README.zh-CN.md)**<br>([EN](./skills/sieve/README.md)) | **长日志/大 JSON/网页灌爆上下文（Context Rot）** | `pytest`/`cargo`/CI 日志压减 **98.8%**；需要细节直接 `peek <id>` 毫秒级无损回放，免重跑慢测试 | 400 条 API JSON 压减 **99.6%**、噪音网页 HTML 压减 **98.4%**，支持 `--json-path` 精查 | `83 tok` |
| **[`skills/guard/`](./skills/guard/README.zh-CN.md)**<br>([EN](./skills/guard/README.md)) | **未过检虚报“已完成”**（`exit 1` 拦不住！）、死循环与密钥外泄 | 原生 **`exit 2`** 完工真相门、连续 3 次同错死循环自动熔断、拦截 `git push --force` | 事前硬拦截 API Key 外泄、`rm -rf` 误删与未经确认的批量邮件/Webhook 外发事故 | `63 tok` |
| **[`skills/baton/`](./skills/baton/README.zh-CN.md)**<br>([EN](./skills/baton/README.md)) | **自动压缩失忆**与跨终端/跨 Agent 丢失试错结论 | 在 Claude ↔ Codex ↔ Grok ↔ Antigravity 间无损传递硬事实、Git 状态与 **`DO NOT RETRY` 死胡同** | 跨天推进深度调研、采购比价与长线待办，压缩后 `<400 Tokens` 满血恢复 | `70 tok` |

---

## 三大主流 CLI 实机受控 A/B 对照实测（100% 真实轨迹，零编造）

我们在本机对 **Grok Build CLI (`1.0.77`, `grok-4.7`)**、**Google Antigravity CLI (`1.5.8`)** 与 **OpenAI Codex CLI (`0.167.0`, `gpt-6-astra`)** 进行了同题受控 A/B 实测（测试仓均包含大量未引用干扰模块与复合隐蔽缺陷：`Task B: rulelang` 25 文件规则编译器/优化器/缓存 5 重缺陷；`Task A: fluxpipe` 34 文件 Webhook 投递引擎 6 重别名/单位/TTL 缺陷），统一由外部隐藏测试集（Oracle）独立判分：

| 实测 CLI 与任务 | 对照模式 | 墙钟耗时 / 工具步数 | 内部推理/思维 Token | 单轮最高思维尖峰 | 总吞吐 Token（含缓存） | 隐藏 Oracle 得分 |
| :--- | :--- | ---: | ---: | ---: | ---: | :---: |
| **Grok CLI `1.0.77`** (`grok-4.7`) · *Task B (`rulelang`)* | 原生基线 | `693.1s` (`23` 步, `8` 次 `search_replace`) | `36,013 tok` | `14,285 tok` | `418,542 tok` (`$0.59`) | `5/5` |
| **Grok CLI `1.0.77`** (`grok-4.7`) · *Task B (`rulelang`)* | **`+ probe + splice`** | **`352.6s` (-49.1% 提速近一倍)** (`0` 次 `search_replace`) | **`12,477 tok` (-65.4%)** | **`4,302 tok` (-69.9%)** | `522,141 tok` (**`$0.54`, -7.3%**) | **`5/5`** |
| **Antigravity CLI `1.5.8`** (`agy`) · *Task B (`rulelang`)* | 原生基线 | `41` 工具步 *(现场发生 `python -c` 引号崩溃)* | `50,680 tok` | — | `948,334 gross` (`401,064` net) | `5/5` |
| **Antigravity CLI `1.5.8`** (`agy`) · *Task B (`rulelang`)* | **`+ probe + splice`** | **`24` 工具步 (-41.5%)** *(零转义崩溃)* | **`29,449 tok` (-41.9%)** | — | **`442,327 gross` (-53.4% 净省50万)** | **`5/5`** |
| **Codex CLI `0.167.0`** (`gpt-6-astra`) · *Task B (`rulelang`)* | 原生基线 | `7` 条命令 *(需额外一轮手删 `__pycache__`)* | `96 tok` | — | `180,757 tok` | `5/5` |
| **Codex CLI `0.167.0`** (`gpt-6-astra`) · *Task B (`rulelang`)* | **`+ probe + splice`** | **`6` 条命令 (-14.3%)** *(自动清 `.pyc`)* | **`53 tok` (-44.8%)** | — | **`174,709 tok` (-3.3%)** | **`5/5`** |
| **Grok CLI `1.0.77`** (`grok-4.7`) · *Task A (`fluxpipe`)* | 原生 → **`+ probe + splice`** | `40` → **`31` 步 (-22.5%)** (`13` → `0` `search_replace`) | `55,398` → **`41,499` (-25.1%)** | `24,700` → **`10,039` (-59.4%)** | `1.033M` → `1.050M` *(首次读取技能文档持平)* | **`6/6` → `6/6`** |

> **诚实工程结论**：当面对需要跨文件追踪状态/引用/语义边界的任务时，`probe + splice` 能为 `Grok` 砍掉 **49.1% 耗时与 65.4% 推理 Token**，为 `Antigravity` 砍掉 **53.4% 总吞吐 Token（单题立省 50.6 万 Token）与 41.5% 工具步数**；而在极快模型（如 `Codex` 8 步速通局）首次现场读 `SKILL.md` 时会多花 1 条读取命令（`8 → 9` 步），通过 `install.py` 预装为全局技能即可彻底摊薄首次发现成本。

---

## 可复现确定性微基准实测（含诚实负对照）

本机运行 `python bench/run_bench.py` 即可复现 [`bench/results.json`](./bench/results.json)（Python 3.12，固定种子 `20261115`，零外部包）：

| 基础设施与测试负载 | 原始体积 / 基线 | 接入原语后 | 缩减 / 提效幅度 | 本机延迟 | 保真度与确定性保障 |
| :--- | ---: | ---: | ---: | ---: | :--- |
| **`probe`** · 25 模块包静态剪枝 + 动态追踪 | `25` 个源码文件 | `3` 个活跃文件（剪除 `20` 个死文件） | **-80.0% 无关模块** | `18.8 ms` / `12.0 ms` | 自动捕获 `2` 处堆别名、`1` 处原地污染、`1` 处状态空操作 |
| **`splice`** · 30 方法 CRLF 模块精读 + 原子补丁 | `1,376 tok` 读全文件 | `77 tok` 精读目标方法 | **-94.4% 读取开销** | `1.5 ms` / `4.0 ms` | 严格保留 CRLF 与 UTF-8 中文头；语法报错 100% 字节级回滚 |
| **`sieve`** · 2,500 行 CI/pytest 报错日志 | `172,075 B` (`~43,019 tok`) | `2,080 B` (`~520 tok`) | **-98.8%** | `23.7 ms` | 100% 异常栈保留 + 无损 `peek` |
| **`sieve`** · 400 条记录 API JSON 响应 | `107,887 B` (`~26,972 tok`) | `423 B` (`~106 tok`) | **-99.6%** | `3.0 ms` | 顶层结构与告警完整保留 + `--json-path` 精查 |
| **`sieve`** · 含大量 JS/CSS 噪音的网页 HTML | `48,160 B` (`~12,040 tok`) | `780 B` (`~195 tok`) | **-98.4%** | `8.5 ms` | 标题、H1–H3 大纲与正文核心完整保留 |
| **`sieve`** · `<1.2 KB` 短输出 *(诚实负对照)* | `69 B` (`~18 tok`) | `117 B` (`~30 tok`) | **0.0%** *(原样直通)* | `2.7 ms` | 短输出不强行摘要，杜绝形式主义负优化 |
| **`recall`** · 200 条中英双栖规则库检索 | `6,380 tok` *(全量塞入)* | `39.9 tok` *(Top-2 注入)* | **-99.4%** | `4.0 ms` | **20/20 (100%)** Top-1 命中率 |
| **`atlas`** · 64 模块 SDK 代码仓 | `42,710 tok` *(全量读取)* | `446 tok` *(结构地图)* | **-99.0%** | `19.3 ms` | AST 类/函数签名、核心枢纽与入口完整提取 |

---

## 60 秒按需安装与接入

```bash
# 0. 先离线扫描你本机 CLI 会话的真实痛点
python audit/audit_sessions.py

# 1. 自检 Python 3.9+ 与内置 sqlite3 FTS5
python install.py --doctor

# 2. 一键安装全部 7 个文件夹（或加 --only probe,splice 只装子集）：
python install.py --agent agents --print-agents-md   # ~/.agents/skills (Antigravity / Codex / Cursor / OpenCode)
python install.py --agent claude --print-hooks       # ~/.claude/skills 并打印可直接粘贴的 Hook JSON
python install.py --agent grok                       # ~/.grok/skills   (Grok Build CLI)
python install.py --agent openclaw                   # ~/.openclaw/skills (OpenClaw 个人助手)
python install.py --agent hermes                     # ~/.hermes/skills (Hermes 助手)
```

---

## 仓库目录结构与测试校验

```text
agent-primitives/
├── README.md / README.zh-CN.md      # 默认英文首页 + 完整中文首页（含实机 A/B 实测数据）
├── HANDBOOK.md / HANDBOOK.zh-CN.md  # 核心理念宪章、六条工程红线、五步持续迭代闭环与路线图
├── DESIGN.md / DESIGN.zh-CN.md      # 抗淘汰四道物理边界底层架构深潜
├── PLAYBOOK.md / PLAYBOOK.zh-CN.md  # Coding Agent 与个人助手分场景接入实战指南
├── AGENTS.md                        # 极简（<40行）仓内自治守则，供后续 AI 迭代本仓时自动遵守
├── audit/audit_sessions.py          # 本地零云端会话病理审计器（支持 Codex/Claude/Grok/AGY）
├── install.py                       # 零依赖按需安装器、Hook 生成器与环境自检器
├── skills/                          # 一个子文件夹 = 一个独立零依赖微基础设施
│   ├── atlas/                       # 阶段 1：亚秒级 AST 与知识库结构地图 + >90天陈旧 TODO 雷达
│   ├── recall/                      # 阶段 1：<4ms SQLite FTS5/BM25 中英双栖触发式微记忆
│   ├── probe/                       # 阶段 2：静态可达性切片、堆别名/原地篡改显微镜与差分预言机
│   ├── splice/                      # 阶段 3：AST/Markdown 符号精读与 UTF-8/CRLF 原子回滚补丁器
│   ├── sieve/                       # 阶段 4：无损输出降噪压缩器与本地 FTS5 黑匣子
│   ├── guard/                       # 阶段 5：Exit-2 完工真相门、3 次同错死循环熔断与密钥护盾
│   └── baton/                       # 阶段 5：抗压缩状态胶囊与跨 Agent 死胡同接力棒
├── bench/run_bench.py               # 确定性基准测试套件（含诚实负对照）
└── tests/test_primitives.py         # 10 组单目录物理隔离、原子回滚、编码与端到端单元测试
```

```bash
python -m unittest discover -s tests -v
```

## 开源协议

[MIT](./LICENSE)
