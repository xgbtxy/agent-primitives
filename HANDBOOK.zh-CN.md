# `agent-primitives` 核心理念与持续演进总手册（Living Handbook）

[English](./HANDBOOK.md) | [简体中文](./HANDBOOK.zh-CN.md) · [仓库首页](./README.zh-CN.md) · [底层架构](./DESIGN.zh-CN.md) · [实战指南](./PLAYBOOK.zh-CN.md)

> **本手册定位**：这是 `agent-primitives` 仓库的**最高设计宪章与长期迭代作战手册**。把我们的底层理念一次性讲透，并规定今后每一次围绕本仓库持续迭代、打磨、扩充时的标准闭环——确保无论经过多少轮更新、无论外部大模型如何换代，这个仓库始终保持**高信噪比、零臃肿、用了就离不开**。

---

## 一、我们的核心理念（Why & Philosophy）

### 1. 第一性原理：绝不造「思维拐杖」，只造「物理器官」
开源社区里 95% 的 Agent 项目之所以活不过半年，是因为它们试图在提示词或外层框架里**教大模型怎么思考**（角色扮演模板、步骤清单、重型编排器、有损向量检索）。这类“思维拐杖”在下一代前沿基座模型（`GPT-6`、`Grok-5`、`Claude-5`、`Gemini-4`）的后训练强化学习面前不堪一击——模型智商一涨，拐杖就成了拖慢速度的累赘。

我们的理念恰恰相反：**越是聪明的大模型，越不需要别人教它怎么说话，反而越渴求精确、极速、确定性的物理世界工具。**
无论未来基座模型多强，它本质上依然是在追加式 KV-Cache 上逐个生成 Token 的概率机器，永远横亘着**四道靠堆参数消除不了的物理鸿沟**：
1. **外部高熵输入鸿沟**：`200 KB` 日志、大 JSON、噪音网页或几十个死代码模块一旦塞进上下文，物理上必然拉慢首字延迟、稀释注意力并按轮重复计费。
2. **内存运行态因果鸿沟**：在自然语言里“脑补”浅拷贝嵌套引用共享、原地修改异常未回滚、`None`/`True == 1` 类型碰撞既慢又容易自欺欺人；临时手写 `python -c` 又频繁撞上 Shell 引号转义崩溃。
3. **字节与语法树落盘鸿沟**：模型吐出的是 BPE Token，而磁盘文件要求精确的 UTF-8 编码、CRLF/LF 换行一致性、AST 语法合法性与字节码缓存同步。
4. **会话压缩失忆与自证偏见鸿沟**：自动压缩（`/compact`）会抹掉“哪些方案已经试错失败”的负向记忆；概率文本也永远无法替代操作系统级的 **`exit 2`** 物理门禁。

**结论**：我们只在这四道物理边界上打造类似 **Unix Coreutils（`git` / `strace` / `sqlite` / `diff`）** 的确定性工具。程序员水平越高越离不开 `git` 和 `strace`；同理，**AI 基座越进化，调用 `agent-primitives` 就越准、越快、收益越大**。

### 2. 微基础设施美学：一个文件夹 = 一个独立完整的小项目
我们坚决拒绝做成动辄几十个依赖、强绑 `Docker`/`Node`/云端 API Key 的大一统框架：
- **单目录自治**：`skills/` 下的每一个子文件夹都是一个独立闭环的微基础设施。用户只痛日志刷屏，就只拷走 `skills/sieve/`；只痛改代码反复重读与转义翻车，就只拷走 `skills/probe/` 和 `skills/splice/`。
- **零外部负担**：纯 Python 3.9+ 标准库实现（`0` pip 包、`0` 常驻后台守护进程、`0` 联网上传、`0` 绑卡墙），Linux / macOS / Windows 开箱即跑。
- **极致静默预算**：每个文件夹待机仅占 **`63~83 Tokens`**（全部 7 个装齐不到 `500 Tokens`），绝不未战先吃掉一万 Token 上下文。

### 3. Coding Agent 与个人助手 Agent 双栖同源
写代码的 Agent（`Claude Code`、`Codex`、`Grok Build`、`Antigravity`、`Cursor`）与处理日常事务的个人助手 Agent（`OpenClaw`、`Hermes`、知识库/运维/调研助手），表面场景不同，**底层过程瓶颈完全同构**：
- 代码仓需要 AST 地图，几百篇个人 Markdown 笔记库同样需要目录与标题树（`atlas`）；
- 代码子目录有专属避坑铁律，个人报销/合同/客户沟通同样有低频高优 SOP（`recall`）；
- 代码逻辑需要运行态显微镜，个人财务核算与数据清洗脚本同样需要跑批前边界预检（`probe`）；
- 改源码需要按函数精读与原子补丁，改长篇手册与 JSON 配置同样需要按章节精读与防乱码原子写入（`splice`）；
- `pytest` 刷屏需要无损压缩，个人助手调外部 API JSON 与抓网页 HTML 同样需要剥离噪音（`sieve`）；
- 防强推代码需要安全护盾与完工门禁，防个人助手泄密或误删/乱群发同样需要 `exit 2` 硬拦截（`guard`）；
- 跨终端换 Coding CLI 需要传接力棒，跨天长线个人调研在自动压缩后同样需要不断片恢复（`baton`）。

---

## 二、仓库六条铁律（Non-Negotiable Constitution）

后续任何迭代更新，一旦违反以下任意一条，直接一票否决：

1. **零纯话术铁律**：绝不收录没有确定性执行脚本支撑的“纯 Markdown 提示词模板 / 角色设定”。
2. **物理隔离铁律**：`skills/<name>/` 内部必须 100% 自治，**严禁跨文件夹 `import` 兄弟模块**，确保用户单独复制任一文件夹到空白环境都能独立运行（由 `test_standalone_no_cross_imports` 强制守护）。
3. **零第三方依赖与全平台铁律**：仅允许使用 Python 3.9+ 标准库，必须在 Ubuntu、macOS、Windows（特别是 Windows PowerShell 与 `cp936` 默认页环境）三端全部实测通过。
4. **性能与上下文红线**：单文件夹 `SKILL.md` YAML 头静默开销 **`< 85 Tokens`**；核心命令本地延迟 **`< 100 ms`**；默认输出收敛在 **`< 1,000 Tokens`**（完整原文走本地索引按需 `peek`）。
5. **宁做深、不滥生铁律**：优先把现有 7 个正交原语打磨到极致（演进子命令、跨语言支持、降低调用摩擦），除非发现现有 5 步执行闭环完全无法归类的全新物理边界，**绝不为了凑数量乱建新文件夹**。
6. **实证与诚实铁律**：所有提效数据必须来自真实 CLI 轨迹审计（`audit_sessions.py`）与受控实机 A/B 盲测；基准测试必须包含**诚实负对照（Negative Control）**，明确写出什么情况下**不该调用**。

---

## 三、现有「1 + 7」底座版图与职责边界

```text
[离线诊断入口] audit/audit_sessions.py  ──► 扫描本机 Codex / Claude / Grok / Antigravity 真实历史损耗
   │
   ├── 阶段 1：认路与唤起 ──► skills/atlas/  （代码 AST / 笔记库秒级地图 + >90天陈旧 TODO 诱饵预警）
   │                      ──► skills/recall/ （<4ms SQLite FTS5 + 中英双栖触发式微记忆，替代臃肿 AGENTS.md）
   ├── 阶段 2：切片与诊断 ──► skills/probe/  （静态可达性剪除死文件 + sys.settrace 堆别名/原地污染显微镜 + 边界差分）
   ├── 阶段 3：手术式落盘 ──► skills/splice/ （AST 符号/Markdown 章节精读 + 强制 UTF-8/保留 CRLF/语法预检原子回滚补丁）
   ├── 阶段 4：观测流降噪 ──► skills/sieve/  （>1.2 KB 长日志/大 JSON/网页 HTML 压减 98%+，本地 FTS5 黑匣子无损回放）
   └── 阶段 5：门禁与接力 ──► skills/guard/  （原生 exit 2 密钥/高危命令护盾、3 次同错死循环熔断、完工真相门）
                          ──► skills/baton/  （<400 tok 抗压缩状态胶囊 + DO NOT RETRY 死胡同跨端接力棒）
```

---

## 四、后续持续迭代的标准五步闭环（How We Iterate）

今后每次围绕本仓库做迭代更新，统一遵循以下五步工程节奏：

### 第 1 步：从真实 CLI 过程里抓真问题（Telemetry First）
在日常使用 `Grok`、`Codex`、`Antigravity`、`Claude Code`、`OpenClaw` 一段时间后，运行：
```bash
python audit/audit_sessions.py
```
或者直接复盘最近一次让 Agent 卡壳、空烧几万推理 Token、报错重试的真实会话日志。**只立项真实发生的高频过程摩擦，绝不闭门造车想象需求。**

### 第 2 步：过「抗淘汰灵魂四问」立项审查
动手写代码前先回答四个问题：
1. *下一代模型智商翻倍后，这个问题会自动消失，还是依然受限于物理边界？*（只做后者）
2. *这是现有 7 个文件夹中某一个的自然延伸（例如给 `splice` 加 TypeScript 符号解析），还是全新的第 8 个正交维度？*（90% 情况应归入现有 7 个文件夹做深做强）
3. *Agent 调用它时需要多花几轮学习成本？能不能做到一行 `--stdin` 零转义、免查 `--help` 一次调通？*
4. *在极简单任务上调用它会不会反而变慢？触发阈值设在哪里最诚实？*

### 第 3 步：零依赖外科手术实现 + 严苛单测守护
- 修改或扩充对应 `skills/<folder>/scripts/<name>.py` 与 `SKILL.md`（检查 `bench/run_bench.py` 输出的 `idle_frontmatter_tokens_per_folder` 确保 `< 85 tok`）。
- 在 `tests/test_primitives.py` 补齐对应的边界测试、Windows 编码/换行测试、错误回滚测试，确保单目录拷入空白临时目录仍能独立通过。

### 第 4 步：本机实机 CLI 受控 A/B 对照验证
- 在本机拉起真实 CLI（`grok` / `codex` / `agy` / `claude`），用同一道真实多文件任务跑对比，挂外部独立 Oracle 判分。
- 重点盯住四个硬指标：**墙钟耗时、推理/思维 Token 消耗、工具步数/报错崩溃次数、总吞吐成本**。如果发现模型调用姿势别扭（如参数名不直觉），当场打磨脚本接口兼容度（例如同时兼容 `"content"` 与 `"new"` 字段）。

### 第 5 步：中英双语同步、六路矩阵 CI 绿灯与演进记账
- 默认英文文档（`README.md`、`HANDBOOK.md`、`PLAYBOOK.md` 及子目录 `README.md`）与完整中文文档（`*.zh-CN.md`）保持 1:1 同步更新。
- 本地跑通 `python -m unittest discover -s tests -v` 与 `python bench/run_bench.py` 后推送主干，确认 GitHub Actions（`ubuntu` / `macos` / `windows` × `Python 3.10` / `3.12`）全绿，并在下方第 6 节登记本次迭代成果。

---

## 五、下一阶段高价值迭代路线图（Active Iteration Backlog）

以下方向全部源自我们在真实 CLI 轨迹审计与多轮实机 A/B 对照中亲眼抓到的具体机会，按投入产出比排序推进：

- [x] **P1 · 多语言 AST/结构扩展（`probe slice --bundle` + `splice view/apply`）**：已在 `v0.3` 落地——零第三方依赖支持 **Python / TypeScript / JavaScript / Go / Rust** 的导入可达性剪枝、活跃源码一键打包（`--bundle`）与符号级精准读写（精确区分 Rust `'a` 生命周期与字符字面量）。
- [x] **P1 · 零摩擦测试运行器自动探测（`splice check --test`）**：已在 `v0.3` 落地——自动识别 `pytest` / 标准库 `unittest` / `node --experimental-strip-types --test` / `go test` / `cargo test` 并限长保真输出报错栈。
- [x] **P2 · `audit_sessions.py` 前后周期提效对比面板（`--compare`）**：已在 `v0.3` 落地——按 Harness 自动对比前后半段会话的病理摩擦变化。
- [x] **P2 · 全主流 Harness 最简一行零克隆接入（`ap.py init` + 原生插件清单）**：已在 `v0.3` 落地——一行 `curl`/`irm | python - init` 自动探测并接入 **`Codex`、`Pi`、`Antigravity`、`Claude Code`、`Grok`、`OpenClaw`、`Hermes`**，`guard.py` 原生双协议兼容 Claude/Codex `exit 2` 与 Antigravity camelCase `deny`/`continue` Hook，并提供 `pi install`、`codex plugin`、`agy plugin` 与 `Claude` 原生插件清单。
- [ ] **P3 · 多语言核心枢纽与陈旧 TODO 地图扩展（`atlas`）**：将 `atlas map` 的导入中心度与函数签名摘要扩展至大型 TypeScript / Go / Rust 单体仓。
- [ ] **P3 · 个人助手长文档表格/邮件串/分页 JSON 增强（`sieve` + `splice`）**：强化复杂 HTML 表格、Markdown 大表格行级检索与 JSON Lines 流式聚合摘要。

---

## 六、演进里程碑台账（Evolution Log）

| 日期 / 里程碑 | 核心演进内容 | 解决的真实过程痛点 | 实测核验凭据 |
| :--- | :--- | :--- | :--- |
| **2026-11-15 · `v0.1` 创仓立基** | 首发 `sieve`、`baton`、`recall`、`guard`、`atlas` 五大独立微基础设施 + `install.py` + 确定性基准套件（含负对照） | 上下文刷屏腐烂、压缩失忆、`AGENTS.md` 肥胖、虚报完工、冷启动盲摸与陈旧 TODO 诱饵 | CI 日志压减 **98.8%**、JSON 压减 **99.6%**、200 规则检索 **`4.0ms` (100% 命中)**，全装待机 `360 tok` |
| **2026-11-15 · `v0.2` 跨 CLI 实证升级** | 审计本机 4 大 CLI `69,963` 次调用；新增 `audit/audit_sessions.py`、`skills/probe/`（可达性切片 + 堆别名/篡改显微镜 + 差分）与 `skills/splice/`（AST/章节精读 + UTF-8/CRLF 原子回滚补丁） | `50.1%` 同文件重读、`5.5万` 推理 Token 文字脑补内存、手写 `python -c` 转义崩溃、Windows `cp936` 编码报错与残留 `.pyc` | `Grok 1.0.77` 耗时 **-49.1%**、推理 Token **-65.4%**；`Antigravity 1.5.8` 总吞吐 **-53.4%（省 50.6 万 tok）**、步数 **-41.5%**；`10/10` 单测与三端六路 CI 全绿 |
| **2026-11-15 · `v0.2.1` 演进手册立宪** | 建立中英双语《核心理念与持续演进总手册》（`HANDBOOK.zh-CN.md` / `HANDBOOK.md`）与仓内自治守则 `AGENTS.md` | 固化抗淘汰四边界理论、六条工程红线与五步实证迭代闭环，为长期持续演进立规 | 双语文档全链路对齐 + CI 验证通过 |
| **2026-11-15 · `v0.3` 多语言扩展与一行全生态接入** | 发布零克隆 `ap.py`（`curl`/`irm \| python - init`）、四大原生插件清单（`pi`、`codex`、`agy`、`claude`）、Py/TS/JS/Go/Rust 多语言 `probe slice --bundle` 与 `splice` + `check --test`、`probe run` 循环去重、双协议 `guard.py` Hook 与 `audit --compare` | 消除 15+ 文件串行 `read_file` 往返、循环追踪 82KB 截断溢出、PowerShell 转义翻车、Grok 全局旧技能遮蔽及各 Harness 接入繁琐痛点 | TypeScript `ts-ledger` 实机盲测（Oracle `5/5`）：`Grok 1.0.77` 耗时 **-26.5%**、`read_file` **-66.7%**、总 Token **-38.7%（账单 -34.4%）**；`Antigravity 1.5.8` 步数 **-61.3%**、报错 **-85.7%**（Python `rulelang` 耗时 **-24.0%**、`read_file` **-74.2%**、净 Token **-49.8%**）；`Codex 0.167` 耗时 **-7.3%**；`14/14` 单测全绿 |
