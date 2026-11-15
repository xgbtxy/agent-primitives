# 真实提效实战指南：1 条命令接入，让 Coding Agent 与个人助手真正离不开它

[English](./PLAYBOOK.md) | [简体中文](./PLAYBOOK.zh-CN.md) · [总纲手册](./HANDBOOK.zh-CN.md) · [抗淘汰底层架构设计](./DESIGN.zh-CN.md)

再强的工具，如果接入要手动改一堆路径、配半小时 JSON，或者在 Windows PowerShell 下频繁报转义错误，用户用一次就会放弃。本指南基于 **69,963 次真实工具调用遥测** 与 **Grok Build CLI (`1.0.77`)、Google Antigravity CLI (`1.5.8`)、OpenAI Codex CLI (`0.167.0`)、Claude Code** 实测，展示如何用 **1 条命令零配置接入** 并立即跑出真实提效。

---

## 零、第 0 步：先用 `ap.py audit` 测出你本机 Agent 到底在浪费什么

在安装任何文件夹之前，先跑一行**纯本地、零联网**审计命令，扫描本机 `~/.codex`、`~/.claude`、`~/.grok`、`~/.gemini/antigravity-cli` 历史轨迹：

```bash
python ap.py audit --compare
```

几秒内即可量化本机真实发生的**同文件重复重读、手工终端切片、`python -c` 引号崩溃与 `ImportError`、补丁失配、Windows `cp936`/UTF-8 编码报错、`>8 KB` 巨型输出刷屏**，并对比前后半段会话的摩擦变化趋势。

---

## 一、最简一行命令零克隆接入全主流 Harness（`ap.py init`）

无需 `git clone`、无需 `pip install`、无需 `npm`、无需手改路径或手写 JSON：

```bash
# 通用一行自动接入（macOS / Linux）：
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python3 - init

# 通用一行自动接入（Windows PowerShell）：
irm https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python - init

# 仅接入当前项目仓库（自动创建 .agents/skills、.agents/ap.py 并幂等写入 AGENTS.md 围栏）：
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python3 - init --here

# 按需指定 Harness 接入 + 无损挂载门禁 Hook（支持 Claude Code、Codex、Antigravity）：
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python3 - init -a codex,pi,agy,claude --with-hooks

# 各家生态官方原生一行包管理命令同样开箱即用：
pi install git:github.com/xgbtxy/agent-primitives
codex plugin marketplace add xgbtxy/agent-primitives && codex plugin add agent-primitives@agent-primitives
npx skills add xgbtxy/agent-primitives -g

# 随时一行干净卸载（100% 保留你原有的个人规则与配置）：
ap uninstall          # 或：python ap.py uninstall --here
```

### 各大 Harness 底层发现机制精确适配细节
- **OpenAI Codex CLI (`codex`)**：同步 `~/.codex/skills/` + `~/.agents/skills/`，向 `~/.codex/AGENTS.md` 幂等注入 `<10 行` 围栏路由块，`--with-hooks` 自动合并 `~/.codex/hooks.json`；仓库内置 `.agents/plugins/marketplace.json` 与 `.codex-plugin/plugin.json` 原生支持 `codex plugin add`。
- **Pi (`pi` / `pi-coding-agent`)**：同步 `~/.pi/agent/skills/`（项目模式 `-a pi` 同步 `.pi/skills/`）与 `~/.pi/agent/AGENTS.md`；仓库根目录提供零依赖 `package.json` `"pi"` 清单，原生支持 `pi install git:github.com/xgbtxy/agent-primitives`。
- **Google Antigravity (`agy` / IDE)**：同步 `~/.gemini/config/skills/` + `~/.agents/skills/` 与 `~/.gemini/config/GEMINI.md`；`--with-hooks` 按 Antigravity 专属的顶层命名组规范写入 `"agent-primitives-guard"`（`"matcher": "run_command"`），且 `guard.py` 自动识别 camelCase JSON 协议返回 `{"decision": "deny"|"allow"}` 与 `{"decision": "continue"|"stop"}`（含 `continuationActive` 防死循环）。同时内置 `.antigravity-plugin/plugin.json` 与 `rules/AGENTS.md`。
- **Anthropic Claude Code (`claude`)**：同步 `~/.claude/skills/` 与 `~/.claude/CLAUDE.md`，`--with-hooks` 自动备份 `.bak` 并合并 `PreToolUse`（`Bash`）、`PreCompact`（`baton` 自动快照）与 `Stop`（`exit 2`）至 `~/.claude/settings.json`，同时内置 `.claude-plugin/marketplace.json`。
- **xAI Grok Build (`grok`)**：同步 `~/.grok/skills/`（实测证明 Grok 读取全局 `~/.grok/skills` 优先级高于项目目录！）与 `~/.grok/AGENTS.md`。

---

## 二、核心闭环：把 7 个独立原语卡在 Agent 执行的 5 个物理咽喉点

全部 7 个独立文件夹静默加载总计仅 **`486 Tokens`**，精准卡住每轮执行的 **5 个物理阶段**：

```text
[阶段 1：认路与精准唤起] --> atlas  (<0.1s 扫出 AST/文档库结构图 + 标出 >90 天陈旧 TODO 陷阱)
                         --> recall (<6ms 标准库 FTS5 + 中英双栖匹配，只注入 ~40 tok 当前规则)
[阶段 2：切片与因果诊断] --> probe  (slice --bundle 秒级剪除 Py/TS/JS/Go/Rust 死文件并一次性打包活跃源码；
                                     run 按调用点去重抓堆别名/原地篡改/空操作；diff 穷举类型与缓存碰撞)
[阶段 3：外科手术式落盘] --> splice (view 跨文件精准读 Py/TS/JS/Go/Rust 符号或 Markdown 标题 [-94% tok]；
                                     apply 强制 UTF-8/保留 CRLF 原子回滚补丁；check --test 一键验语法+跑测试)
[阶段 4：观测流无损降噪] --> sieve  (>1.2 KB 日志/JSON/HTML 压减 98%+，原文落本地黑匣子按需 peek)
[阶段 5：门禁与跨轮接力] --> guard  (exit-2 拦密钥外泄/高危命令、3 次同错熔断、未过检禁报完工)
                         --> baton  (<400 tok 保存已验证事实 + 严禁重踩的死胡同，跨压缩/跨端秒接续)
```

---

## 三、场景一：Coding Agent 怎么用才能立竿见影？

*(适用于 Claude Code、Codex CLI、Grok Build、Google Antigravity、Cursor、OpenCode)*

1. **1 次调用完成多语言死代码剪枝 + 活跃源码打包（`probe slice --bundle`）**：
   - 面对 **Python、TypeScript/JS、Go、Rust** 多文件仓库，第一条命令直接跑 `python .agents/ap.py probe slice . --bundle`（`25~60 ms`）。
   - 瞬间剪除未引用的干扰死模块（TypeScript `ts-ledger` 实测剪除 **13/24** 个无关文件；Python `fluxpipe` 剪除 **15/34** 个文件），并**一次性输出所有活跃源码（带行号）**，直接省掉 10~16 次串行 `read_file` 往返！
2. **用 `<200ms` 因果显微镜替代 20 分钟文字空想（`probe run` / `diff`）**：
   - 用 `python .agents/ap.py probe run --cwd . --focus <包名> --stdin`（PowerShell 用 `@'...'@ |`，Bash 用 `<<'EOF'`）运行复现代码。内置调用点自动去重确保哪怕循环 200 次输出也稳定在 **`< 3.5 KB`**，一次性标出 **`[ALIAS HAZARD]`**、**`[IN-PLACE MUTATION]`** 与 **`[STATE NO-OP]`**。
   - 涉及编译器、序列化、缓存键时跑 `python .agents/ap.py probe diff`，`<50 ms` 抓出 `None`/`True == 1` 边界与缓存碰撞。
3. **多语言符号精读、原子补丁与自动识别测试门禁（`splice`）**：
   - 跨多文件精读方法：`python .agents/ap.py splice view src/a.ts src/b.ts --symbol MyClass.run`（`~1.6 ms` 直省 **94.4%** Token）。
   - 跨文件原子修改：`python .agents/ap.py splice apply --cwd . --stdin`（或 `--spec _edits.json --unlink-spec` 自动清理临时文件），随后跑 `python .agents/ap.py splice check . --test`，1 次调用完成 UTF-8/语法/相对导入检查并自动运行正确的测试框架（告别盲敲 `pytest` 报 `No module named pytest` 或漏加 Node 22 `--experimental-strip-types --test`）。
4. **长日志降噪、死循环熔断与跨端接力（`sieve` + `guard` + `baton`）**：
   - 长测试输出套 `python .agents/ap.py sieve run -- <cmd>`（压减 **98.8%**）；证伪假设记入 `baton deadend`，新发现的模块暗坑顺手 `recall add`。

---

## 四、场景二：个人助手 Agent 怎么用才能真实提效？

*(适用于 OpenClaw、Hermes、日常研究/知识库/运维办公助手)*

1. **大型笔记库导航与章节级精准读写（`atlas` + `splice`）**：
   - `python .agents/ap.py atlas map ~/Notes` 在 `0.1 秒` 内扫出数百篇 Markdown 提纲；`splice view ~/Notes/handbook.md --heading "应急流程"` 只读目标章节，并用 `splice apply` 原子更新配置与笔记。
2. **零云端隐私安全的中英双栖个人 SOP 记忆（`recall`）**：
   - 报销、合同、服务器规矩存入 `recall`，中英混合检索中位延迟 **`5.5 ms`**（200 条规则 Top-1 准确率 **100%**），零外部 API、零隐私泄露。
3. **网页抓取、大 JSON 接口与日志降噪（`sieve`）**：
   - 管道接入 `| python .agents/ap.py sieve pipe`（400 条 JSON 压减 **99.6%**，复杂网页压减 **98.4%**），按需 `sieve peek <id> --json-path "items.0"`。
4. **跨天事务不断片与安全防线（`baton` + `guard`）**：
   - 跨天任务压缩后用 `baton resume`（`<400 Tokens`）秒级恢复；`guard shield` 用 `exit 2` 拦住密钥外泄与高危命令。

---

## 五、诚实边界：什么时候**不要**用？（拒绝形式主义负优化）

- **已知极短的命令不要套 `sieve`**：如 `pwd`、`git status -s`（`sieve` 对 `<1.2 KB` 本身原样直通）。
- **单行已知拼写修复不要跑 `probe`**：报错栈已指明具体行号时，改完直接跑 `splice check . --test` 最快。
- **每轮必用的全局身份红线留在 `AGENTS.md`，低频条件经验放进 `recall`**：各司其职，信噪比最高。
