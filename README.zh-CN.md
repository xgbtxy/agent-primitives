# agent-primitives

**一个文件夹 = 一个零膨胀微基础设施（Coding Agent 与个人助手 Agent 真实提效底座）**

[English (Default)](./README.md) | [简体中文](./README.zh-CN.md) · [真实提效实战指南 (中文)](./PLAYBOOK.zh-CN.md) | [Efficiency Playbook (EN)](./PLAYBOOK.md)

---

## 为什么要做这个仓库？

纵观 2026 年 GitHub 与 X 上的主流 Agent 工具生态，存在一个非常割裂的断层：
- **纯 Markdown 提示词技能库**虽然做到了“单文件夹轻便拷贝”，但**没有任何确定性运行时执行力**（光在文字里劝大模型“请验证后再结束”，测试挂了它照样报告“已完成”；遇到 200 KB 报错日志也无法无损压缩）；
- **重型 Harness 框架与记忆插件**虽然有运行时能力，却往往膨胀成五六十个目录的庞然大物——强绑 `Node`/`Bun`、`Docker`、外部向量库、30 天试用绑卡墙，或者常驻 MCP 工具 Schema 还没开始干活就先吃掉 `8,000~15,000 Tokens`。

**`agent-primitives`** 坚持极简工程纪律：**一个子文件夹 = 一个独立完整的小项目（微基础设施）**。
- **物理级解耦，拷走单文件夹即用**：想要哪一个就复制哪一个（`cp -r skills/sieve ~/.agents/skills/`），文件夹之间**零交叉引用**。
- **零第三方依赖**：100% 仅依赖 Python 3.9+ 标准库（内置 `sqlite3` FTS5、`ast`、`hashlib`、`html.parser`、`subprocess`），Windows / macOS / Linux 开箱即跑。
- **全装待机仅占 `360 Tokens`**：每个文件夹的 `SKILL.md` 头信息待机仅占 **`63~83 Tokens`**，彻底告别上下文税。

---

## 5 个独立微基础设施小项目（`skills/*`）

点击任一文件夹均可查看其独立的中英文说明文档：

| 独立小项目文件夹 | 解决的核心卡点 | Coding Agent 真实提效 | 个人助手 Agent 真实提效 | 待机开销 |
| :--- | :--- | :--- | :--- | ---: |
| **[`skills/sieve/`](./skills/sieve/README.zh-CN.md)**<br>([EN](./skills/sieve/README.md)) | 长日志/JSON/网页灌爆上下文（**Context Rot**），传统代理裁剪又会丢关键报错行 | `pytest`/`cargo`/CI 日志压减 **98.8%**；需要细节直接 `peek <id>` 毫秒级无损回放，免重跑慢测试 | 400 条 API JSON 压减 **99.6%**、噪音网页 HTML 压减 **98.4%**，支持 `--json-path` / 行号精查 | `83 tok` |
| **[`skills/baton/`](./skills/baton/README.zh-CN.md)**<br>([EN](./skills/baton/README.md)) | 自动压缩（**Compaction**）后失忆变傻，或切换终端/Agent/Worktree 丢状态 | 在 Claude Code ↔ Codex ↔ Grok 间无损传递已验证事实、Git 状态与**严禁重踩的死胡同（`DO NOT RETRY`）** | 跨天推进个人调研、比价与复杂事务待办，压缩后 `<400 Tokens` 满血恢复（零云端、零二次烧 Token） | `70 tok` |
| **[`skills/recall/`](./skills/recall/README.zh-CN.md)**<br>([EN](./skills/recall/README.md)) | 记忆失灵本质是**检索失灵**；把 200 条规则堆进 `AGENTS.md` 每轮白烧几千 Token | 标准库 `sqlite3` FTS5 + 路径通配符（碰 `src/auth/*` 自动注入对应 `~40 tok` 避坑铁律，`<5ms`） | 原生中英混合分词 BM25 检索，`<5ms` 按报销、客户、服务器、周报精准唤醒个人 SOP | `71 tok` |
| **[`skills/guard/`](./skills/guard/README.zh-CN.md)**<br>([EN](./skills/guard/README.md)) | 检查未过仍**谎报“已完成”**（普通 `exit 1` 拦不住！）、原地死循环烧钱、密钥外泄 | 原生 **`exit 2`** 完工真相门、连续 3 次同报错指纹死循环自动熔断、拦截 `git push --force` | 事前拦截 API Key/私钥泄露、`rm -rf` 误删与未经确认的批量邮件/Webhook 外发事故 | `63 tok` |
| **[`skills/atlas/`](./skills/atlas/README.zh-CN.md)**<br>([EN](./skills/atlas/README.md)) | 冷启动盲读全仓烧掉数万 Token，或被陈旧 `TODO` 带偏多耗 **77% Token** | `0.08s` 提取代码仓 AST 类/函数签名、高引用核心枢纽与 **`>90天` 陈旧 TODO 诱饵预警** | `<800 Tokens` 秒级生成大型个人 Markdown 笔记库、资料夹目录分布与 `H1/H2` 标题大纲 | `73 tok` |

---

## 我们应该怎么做，才能真实为两类 Agent 提效？

> 完整各客户端接入配置、Hook JSON 与分场景 SOP 详见：**[PLAYBOOK.zh-CN.md（真实提效实战指南）](./PLAYBOOK.zh-CN.md)** · **[PLAYBOOK.md (English)](./PLAYBOOK.md)**

要让这套工具在每天真实干活时产生复利，而不是沦为摆设，只需卡住 **5 个关键动作点**：

### 1. 60 秒按需安装与生成配置（`install.py`）

```bash
# 自检本机 Python 3.9+ 与内置 sqlite3 FTS5
python install.py --doctor

# 一键安装全部 5 个文件夹（若只需某几个，加 --only sieve,baton,guard 即可）：
python install.py --agent agents --print-agents-md   # ~/.agents/skills (Antigravity / Codex / Cursor / OpenCode)
python install.py --agent claude --print-hooks       # ~/.claude/skills 并打印可直接粘贴的 Hook JSON
python install.py --agent grok                       # ~/.grok/skills   (Grok Build CLI)
python install.py --agent openclaw                   # ~/.openclaw/skills (OpenClaw 个人助手)
python install.py --agent hermes                     # ~/.hermes/skills (Hermes 助手)

# 也支持开放 Agent Skills CLI 按需单装某一个文件夹：
npx skills add xgbtxy/agent-primitives --skill sieve
```

### 2. 日常真实提效五步闭环（Coding Agent + 个人助手通用）

1. **开局先绘地图，拒绝盲读（`atlas`）**：进陌生代码仓或大型个人文档库，第一步先跑 `python skills/atlas/scripts/atlas.py map .`（`<0.1s` 输出 `<800 Tokens`），直奔核心文件并无视 `>90 天` 陈旧 TODO 诱饵，省掉开局十几轮盲目 `find`/`grep`。
2. **`AGENTS.md` 砍到 60 行以内，细分经验迁入 `recall`**：全局文件只留每轮必守的 5~10 条铁律；把特定目录的研发坑点、特定客户/报销/运维的个人 SOP 全部用 `recall.py add` 存起来，干活前 `recall.py match` 用 `3.7 ms` 精准注入 `~40 Tokens`，**踩过一次的坑永不重犯**。
3. **凡是可能超过 `2 KB` 的输出一律过 `sieve`**：跑测试、看 CI、抓网页、调长 JSON 接口统一走 `sieve.py run -- <cmd>` 或管道 `| python sieve.py pipe`；需要看某处报错栈或某条 JSON 明细时，用 `sieve.py peek <id> --lines / --grep / --json-path` 毫秒级无损调取，绝不重跑慢命令。
4. **把安全与验收交给 `exit 2` 硬门禁（`guard`）**：挂载 `guard.py shield` 事前拦截密钥外泄与高危命令；棘手调试用 `guard.py run --max-repeats 3` 自动掐断原地死循环；`Stop` Hook 挂载 `guard.py stop-gate`，测试没过直接 `exit 2` 打回重修。
5. **压缩前与切换客户端前传好接力棒（`baton`）**：挂载 `PreCompact` 自动执行 `baton.py checkpoint`，每次试错失败立刻 `baton.py deadend` 记下死胡同；无论是自动压缩后还是从 Claude Code 切到 Codex / Antigravity / OpenClaw，一句 `baton.py resume`（`<400 Tokens`）无缝续接。

### 3. 诚实反膨胀铁律（什么时候**不要**用）

- **已知极短的命令别套 `sieve`**（如 `pwd`、`git status -s`、读 30 行小配置）——`sieve` 只用于可能产生 `>2 KB` 噪音的命令或接口。
- **每轮必用的全局规则别塞 `recall`**，低频条件触发的规则也别塞 `AGENTS.md`——两者各司其职才最省 Token。
- **`baton` 只记硬事实与死胡同，绝不塞聊天流水账**——严控 `<400 Tokens` 才能保证压缩后唤醒即巅峰。

---

## 可复现真实基准实测（含诚实负对照）

本机运行 `python bench/run_bench.py` 即可复现 [`bench/results.json`](./bench/results.json)（Python 3.12，固定随机种子 `20261115`，零第三方包）：

| 基础设施与测试负载 | 原始输入体积 | 实际注入上下文 | 体积缩减率 | 本机耗时 | 保真度 / 召回准确率 |
| :--- | ---: | ---: | ---: | ---: | :--- |
| **`sieve`** · 2,500 行 CI/pytest 报错日志 | `172,075 B` (`~43,019 tok`) | `2,080 B` (`~520 tok`) | **-98.8%** | `23.9 ms` | 100% 异常栈保留 + 无损 `peek` |
| **`sieve`** · 400 条记录 API JSON 响应 | `107,887 B` (`~26,972 tok`) | `423 B` (`~106 tok`) | **-99.6%** | `2.7 ms` | 顶层结构与告警完整保留 |
| **`sieve`** · 含大量 JS/CSS 噪音的网页 HTML | `48,160 B` (`~12,040 tok`) | `780 B` (`~195 tok`) | **-98.4%** | `13.5 ms` | 标题、H1–H3 大纲与正文核心完整保留 |
| **`sieve`** · `<1.2 KB` 短输出 *(诚实负对照)* | `69 B` (`~18 tok`) | `117 B` (`~30 tok`) | **0.0%** *(原样直通)* | `2.0 ms` | 短输出不强行摘要，避免负优化 |
| **`recall`** · 200 条中英双栖规则库检索 | `6,380 tok` *(全量塞入)* | `39.9 tok` *(Top-2 注入)* | **-99.4%** | `3.7 ms` | **20/20 (100%)** Top-1 命中率 |
| **`atlas`** · 64 模块 SDK 代码仓（`bench/run_bench.py`） | `42,710 tok` *(全量读取)* | `446 tok` *(结构地图)* | **-99.0%** | `22.5 ms` | AST 类/函数签名、核心枢纽与入口完整提取 |

## 开源协议

[MIT](./LICENSE)
