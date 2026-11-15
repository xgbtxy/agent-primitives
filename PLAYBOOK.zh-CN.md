# 真实提效实战指南：如何让 Coding Agent 与个人助手 Agent 真正离不开它

[English](./PLAYBOOK.md) | [简体中文](./PLAYBOOK.zh-CN.md) · [抗淘汰底层架构设计](./DESIGN.zh-CN.md)

光把工具下载到本地是不会自动提效的——如果 Agent 不在正确的物理边界调用它，或者工具本身又塞进一堆臃肿提示词，反而会负优化。本指南基于本机 **185 个真实历史会话**审计与 **Grok Build CLI (`1.0.77`)、OpenAI Codex CLI (`0.167.0`)、Google Antigravity CLI (`1.5.8`)、Claude Code** 实测，写清**日常研发与个人事务怎么配、怎么用才能立竿见影**。

---

## 零、第 0 步：先用 `audit/` 测出你本机 Agent 到底在浪费什么

在安装任何文件夹之前，先跑一行**纯本地、零联网**审计脚本，直接扫描你机器上的 `~/.codex`、`~/.claude`、`~/.gemini/antigravity-cli` 历史轨迹：

```bash
python audit/audit_sessions.py
```

不到 `5 秒`，`audit_sessions.py` 就会算出你本机真实发生的**同文件重复重读次数、手工命令行切片次数、`python -c` 引号崩溃与 `ImportError` 次数、补丁匹配失败次数、Windows `cp936`/UTF-8 编码报错次数、`>8 KB` 巨型输出刷屏次数**，并自动给出最契合你痛点的 `--only ...` 最小安装命令。

---

## 一、核心闭环：把 7 个独立原语卡在 Agent 执行的 5 个物理咽喉点

主流 Agent 在真实任务里变慢、变贵、变蠢，根源都在四个物理边界上：
1. **盲摸死代码 + 静态提示词肥胖税**：冷启动误读大量未被引用的死模块，外加 `AGENTS.md` / `MEMORY.md` 堆满几百条低频规则。
2. **靠文字“脑补”内存运行态 + 临时 `python -c` 翻车**：遇到浅拷贝别名共享、原地修改未回滚、`None`/`bool`/`int` 边界碰撞时，纯靠自然语言推演动辄烧掉 `3.5万~5.5万` 推理 Token（耗时十多分钟），手写多行 `python -c` 又频繁撞上 PowerShell/Bash 转义崩溃。
3. **改完重读全文件 + 脆弱补丁事务**：改完几行导致行号偏移后反复重读整个文件（Antigravity 实测同文件重复读取率高达 **50.1%**），Windows 下裸调 `read_text()` 触发 `cp936` 乱码或残留旧 `.pyc` 缓存。
4. **巨型输出灌爆上下文 + 无强制力口头规劝**：`50~300 KB` 测试日志/JSON/HTML 直接塞进历史引发上下文腐烂，缺少原生 **`exit 2`** 门禁阻止虚报完工。

把 **7 个零依赖独立文件夹**（全部装齐静默开销合计仅 **`492 Tokens`**）精准接入 Agent 每轮执行的 **5 个阶段**：

```text
[阶段 1：认路与精准唤起] --> atlas  (<0.1s 扫出 AST/文档库结构图 + 标出 >90 天陈旧 TODO 陷阱)
                         --> recall (<4ms 标准库 FTS5 + 中英双栖匹配，只注入 ~40 tok 当前规则)
[阶段 2：切片与因果诊断] --> probe  (slice 秒级剪除死代码；run --stdin 抓堆共享别名、原地篡改
                                     与状态空操作；diff 自动扫描边界/类型/缓存碰撞)
[阶段 3：外科手术式落盘] --> splice (view 按 AST 符号或 Markdown 标题读取 [-94% tok]；
                                     apply 强制 UTF-8/保留 CRLF/语法预检原子回滚补丁 + 自动清 .pyc)
[阶段 4：观测流无损降噪] --> sieve  (>1.2 KB 日志/JSON/HTML 压减 98%+，原文落本地黑匣子按需 peek)
[阶段 5：门禁与跨轮接力] --> guard  (exit-2 拦密钥外泄/高危命令、3 次同错熔断、未过检禁报完工)
                         --> baton  (<400 tok 保存已验证事实 + 严禁重踩的死胡同，跨压缩/跨端秒接续)
```

---

## 二、60 秒落地配置（零第三方包、全平台标准库）

### 第 1 步：安装全部 7 个文件夹（或按需只装几个）

```bash
# 自检本机 Python 3.9+ 与内置 sqlite3 FTS5
python install.py --doctor

# 一键安装（只想装子集可加 --only probe,splice,sieve）：
python install.py --agent agents --print-agents-md   # ~/.agents/skills (Antigravity / Codex / Cursor / OpenCode)
python install.py --agent claude --print-hooks       # ~/.claude/skills (Claude Code + Hook JSON)
python install.py --agent grok                       # ~/.grok/skills   (Grok Build CLI)
python install.py --agent openclaw                   # ~/.openclaw/skills (OpenClaw 个人助手)
python install.py --agent hermes                     # ~/.hermes/skills (Hermes 个人/研究助手)
```

### 第 2 步：挂上确定性 Hook（不靠大模型自觉）

运行 `python install.py --agent claude --print-hooks` 生成本机绝对路径 Hook：
1. **`PreToolUse` -> `guard.py shield`**：执行前用 **`exit 2`** 硬拦截 API Key 外泄、`git push --force`、`rm -rf` 与失控群发动作。
2. **`PreCompact` -> `baton.py checkpoint`**：自动压缩触发前一秒将目标、Git 改动、已验证事实与死胡同落盘。
3. **`Stop` -> `guard.py stop-gate`**：存在未解冲突标记、明文密钥或测试未过时直接 **`exit 2`** 打回重修。

### 第 3 步：`AGENTS.md` / `SOUL.md` 瘦身至 60 行内

只保留 **5~10 条每轮必用的全局红线** + `python install.py --print-agents-md` 生成的极简路由协议；其余所有特定模块/特定客户的条件经验全部迁入 `recall`。

---

## 三、场景一：Coding Agent 怎么用才能立竿见影？

*(适用于 Claude Code、Codex CLI、Grok Build、Google Antigravity、Cursor、OpenCode)*

1. **冷启动认路 + 静态可达性剪枝（`atlas` + `probe slice`）**：
   - 进陌生大仓先跑 `atlas.py map .`（`~20 ms` 生成 `<800 tok` 地图并标出 `>90 天` 陈旧 TODO）。
   - 排查具体包或入口故障时，第一条命令跑 `probe.py slice . --entry <包名>`（`20~50 ms`），瞬间剪除未被引用的干扰死模块（`fluxpipe` 实测剪掉 **15/34** 个无关文件），直接看清活跃函数签名与跨文件调用契约。
2. **用 `<100ms` 真实运行显微镜替代 20 分钟文字空想（`probe run` / `diff`）**：
   - 严禁靠几万字思维链硬猜对象引用，也别手写容易转义翻车的 `python -c`。直接通过 `--stdin` 把 5 行复现输入喂给 `probe.py run --cwd . --focus <包名> --stdin`，一次性自动抓出 **`[ALIAS HAZARD]`**（浅拷贝嵌套可变对象共享）、**`[IN-PLACE MUTATION]`**（入参被原地污染或异常未回滚）与 **`[STATE NO-OP]`**（调用了修改方法但内部状态毫无变化）。
   - 涉及编译器优化、序列化、缓存键或校验逻辑时，跑 `probe.py diff` 自动穷举 `None`/`bool`/`int`（如 `True == 1` 哈希碰撞）与状态污染差异。
3. **AST 符号级精读 + 原子多文件补丁（`splice`）**：
   - 只想看 500 行文件里的某一个方法？跑 `splice.py view path/mod.py --symbol Cls.method`（`1.5 ms` 直省 **94.4%** Token）。
   - 修改多个源码文件与测试时，用 `splice.py apply --cwd . --stdin` **单次原子事务提交**：强制 UTF-8 编码、自动保留原文件 CRLF/LF 换行符、落盘前预跑 `ast.parse`/`json.loads`（任一文件语法报错则 100% 字节级回滚），并自动清理陈旧 `.pyc` 缓存，随后跑 `splice.py check .`（`~12 ms`）一步验完。
4. **长日志降噪、死循环熔断与跨端接力（`sieve` + `guard` + `baton`）**：
   - 测试与构建输出套上 `sieve.py run -- pytest`（压减 **98.8%**），按需 `sieve.py peek <id> --lines`；易卡死命令套 `guard.py run --max-repeats 3`；证伪方案记入 `baton.py deadend`，排完隐蔽坑顺手 `recall.py add` 永久免疫。

---

## 四、场景二：个人助手 Agent 怎么用才能真实提效？

*(适用于 OpenClaw、Hermes、日常研究/知识库/运维办公助手)*

1. **大型笔记库导航与章节级精准读写（`atlas` + `splice`）**：
   - 对着数百篇 Markdown 知识库跑 `atlas.py map ~/Notes`，`0.1 秒` 获取目录分布与 `H1/H2` 提纲。
   - 用 `splice.py view ~/Notes/handbook.md --heading "应急流程"` 只读指定章节，并用 `splice.py apply` 原子更新配置文件或长笔记，彻底杜绝 Windows 编码乱码与半截写坏。
2. **零云端隐私安全的中英双栖个人 SOP 记忆（`recall`）**：
   - 把报销规范、合同审核要点、联系人偏好、服务器运维口令规矩存入 `recall`，中英文混合检索中位延迟仅 **`4.0 ms`**（200 条规则 Top-1 准确率 **100%**），零外部 API、零隐私外泄。
3. **网页抓取、大 JSON 接口与日志降噪（`sieve`）**：
   - 调 REST 接口或抓网页时接管道 `| python sieve.py pipe`（400 条 JSON 压减 **99.6%**，带脚本样式的网页压减 **98.4%**），需要某条明细再 `sieve.py peek <id> --json-path "items.0"`。
4. **跨天事务不断片与安全防线（`baton` + `guard`）**：
   - 跨天调研、采购比价靠 `baton.py resume`（`<400 Tokens`）在自动压缩后满血恢复；`guard.py shield` 在最外层用 `exit 2` 拦住密钥泄露、误删目录或失控批量外发。

---

## 五、诚实边界：什么时候**不要**用？（拒绝形式主义负优化）

- **已知极短的命令不要套 `sieve`**：如 `pwd`、`git status -s` 或 30 行小文件（`sieve` 对 `<1.2 KB` 本身原样直通，无需多套一层）。
- **单行已知拼写修复不要跑 `probe`**：报错栈已经指明某行变量名拼错时，直接 `splice.py apply` 修复最快；当涉及多文件联动、引用/状态污染或隐蔽边界时才上 `probe`。
- **每轮必用的全局身份红线留在 `AGENTS.md`，低频条件经验放进 `recall`**：各司其职，信噪比最高。
