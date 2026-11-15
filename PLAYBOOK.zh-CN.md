# 真实提效实战指南：如何让 Coding Agent 与个人助手 Agent 真正离不开它

[English](./PLAYBOOK.md) | [简体中文](./PLAYBOOK.zh-CN.md)

光把工具下载到本地是不会自动提效的——如果 Agent 不在正确的时机调用它，或者工具本身又塞进一堆臃肿提示词，反而会负优化。本指南直接写清**我们在真实日常研发与个人事务中应该怎么配、怎么用**，才能实打实砍掉无意义 Token 消耗、消灭压缩失忆与死循环。

---

## 一、核心原则：用「确定性窄边界」替代「口头劝说」

市面上大多数 Agent 配置之所以越用越慢、越用越蠢，根源只有三个：
1. **静态提示词肥胖税**：把几百条历史经验全堆进 `CLAUDE.md` / `AGENTS.md` / `MEMORY.md`，每轮对话先白烧 `5,000~15,000 Tokens`，模型注意力被严重稀释，关键规则反而视而不见。
2. **生肉输出直接灌入上下文**：一次 `pytest`、CI 日志、API 大 JSON 或网页抓取（`50~300 KB`）直接塞进对话历史，瞬间诱发 **Context Rot（上下文腐烂）** 并被迫提前触发自动压缩。
3. **无强制力的文字规劝**：在提示词里写十遍“请务必跑通测试再结束”，没有 **`exit 2`** 门禁脚本拦截，Agent 依然会在检查失败时礼貌地宣布“已完成”。

正确的做法，是把这 5 个独立小文件夹精准卡在 Agent 会话的 **5 个物理咽喉点**上：

```text
[1. 新会话冷启动]   --> atlas   (<0.1s 扫出结构地图 <800 tok，顺手标出 >90 天陈旧 TODO 陷阱)
[2. 动手改/办事前]  --> recall  (<5ms 标准库 FTS5 匹配，只把最相关的 1~3 条铁律 ~40 tok 注入当前步)
[3. 跑长命令/抓网页] --> sieve   (>1.2 KB 日志/JSON/HTML 压减 98%+，原文存本地黑匣子按需无损 peek)
[4. 执行前/报完工前] --> guard   (exit-2 拦密钥泄露与高危命令、3 次同错死循环熔断、未过检禁报完工)
[5. 压缩前/切换终端] --> baton   (<400 tok 保存已验证事实 + 严禁重踩的死胡同，跨会话/跨 Agent 秒接续)
```

---

## 二、60 秒落地步骤（零第三方依赖）

### 第 1 步：只装你需要的文件夹（或 5 个全装）

```bash
# 自检本机 Python 3.9+ 与内置 sqlite3 FTS5 状态
python install.py --doctor

# 一键安装到对应 Agent 目录（如果只想装部分，加 --only sieve,baton,guard 即可）：
python install.py --agent agents     # -> ~/.agents/skills (Antigravity / Codex / Cursor / OpenCode)
python install.py --agent claude     # -> ~/.claude/skills (Claude Code)
python install.py --agent grok       # -> ~/.grok/skills   (Grok Build CLI)
python install.py --agent openclaw   # -> ~/.openclaw/skills (OpenClaw 个人助手)
python install.py --agent hermes     # -> ~/.hermes/skills (Hermes 个人/研究助手)
```

### 第 2 步：挂上确定性 Hook（不靠大模型“自觉”）

运行 `python install.py --agent claude --print-hooks` 生成自动填好本机绝对路径的 Hook 配置：
1. **`PreToolUse` -> `guard.py shield`**：在命令真正执行前自动拦截 API Key 外泄（`sk-...`、`ghp_...`、`xai-...`、`AKIA...`）、`git push --force`、`rm -rf` 与失控群发动作（返回 `exit 2` 强制阻断）。
2. **`PreCompact` -> `baton.py checkpoint`**：在上下文自动压缩发生前一秒，自动把当前目标、Git 改动、已验证事实与死胡同快照落盘。
3. **`Stop` -> `guard.py stop-gate`**：在 Agent 准备结束回答时自动检查冲突标记、明文密钥与测试断言——不过关直接 **`exit 2`** 把报错喂回 `stderr` 强制重修。

### 第 3 步：给 `AGENTS.md` / `SOUL.md` 瘦身（严控在 60 行以内）

把 `AGENTS.md` / `SOUL.md` 砍到只剩 **5~10 条每轮必用的全局底线**，贴入 `python install.py --print-agents-md` 生成的 5 行调用协议，然后把所有「只有碰到某模块/某客户/某类事务才用得上」的细分经验全部迁入 `recall`！

---

## 三、场景一：Coding Agent 怎么用才能真实提效？

*(适用于 Claude Code、Codex CLI、Cursor、Google Antigravity、Grok Build、OpenCode)*

1. **新开会话先看地图，别让 Agent 盲摸仓库**：
   - **低效做法**：上来连发十几条 `find` / `grep` / 读全文件，光认路就烧掉 8 万 Token，还顺手去修了一个两年前没人管的 `# TODO`（多烧 77% Token）。
   - **提效做法**：第一条命令跑 `atlas.py map .`（实测扫 64 个源码文件仅 `77 ms`、输出 `788 Tokens`），直接定位高中心度核心模块，自动绕开 `Stale TODO Traps`。
2. **改复杂模块前先查避坑卡，修完顺手沉淀 1 句话**：
   - 动手前跑 `recall.py match --query "<问题>" --files "src/exporter/http.py"`，`3.7 ms` 精准拿到该目录专属避坑规则（`~40 Tokens`）；每次排完一个隐蔽坑，用 `recall.py add` 存一句话，以后换任何 Agent 都不会在同一块石头上绊第二次。
3. **跑测试 / 查 CI 日志一律走 `sieve`**：
   - `sieve.py run -- pytest` 把 `172 KB` 测试刷屏压成 `2 KB` 摘要（**-98.8%**），所有异常栈行号一目了然；需要看上下文直接 `sieve.py peek <id> --lines 905:930`，**不用为看清报错重跑两分钟测试**。
4. **3 次不过立即熔断 + 跨 Agent 接力不踩旧坑**：
   - 调试棘手问题用 `guard.py run --max-repeats 3 -- <cmd>`；一旦证明某条路走不通，立刻执行 `baton.py deadend "方案X" --reason "原因Y"`。哪怕中途触发自动压缩，或者从 Claude Code 切到 Codex / Antigravity 接着干，一句 `baton.py resume`（`<400 Tokens`）瞬间满血续上，绝不重走死胡同。

---

## 四、场景二：个人助手 Agent 怎么用才能真实提效？

*(适用于 OpenClaw、Hermes、日常办公/研究/运维桌面助手)*

1. **大型笔记库与资料夹秒级导航（`atlas`）**：
   - 对着几百篇 Markdown 笔记或项目归档目录跑 `atlas.py map ~/Notes`，`0.1 秒` 拿到目录分布、近期活跃文档与 `H1/H2` 标题树，不用把几十篇长文档塞进上下文。
2. **全本地隐私安全的中英双栖个人 SOP 记忆（`recall`）**：
   - 把报销抬头、常用联系人偏好、服务器登录规范、周报格式等琐碎规则存入 `recall`。原生支持中文单双字 + 英文混合 BM25 检索，`<5 ms` 精准召回，**100% 本地零云端、不绑卡、不泄密**。
3. **网页抓取、大 JSON 接口与系统日志降噪（`sieve`）**：
   - 个人助手调外部 API、抓长网页或查机器日志时，通过管道 `| python sieve.py pipe` 自动剥离 `<script>/<style>` 垃圾（**-98.4%**）与冗长数组（**-99.6%**），只把标题、大纲、告警字段喂给模型；需要查某一条明细再 `sieve.py peek <id> --json-path "items.0"`。
4. **跨天长线事务接续（`baton`）与外发安全防线（`guard`）**：
   - 跨越多天的资料调研、采购比价、流程跟进靠 `baton.py resume` 保持状态不断片；`guard.py shield` 在最外层兜底，防止自治个人助手误删本地目录、泄露密钥或未经确认批量外发消息。

---

## 五、诚实边界：什么时候**不要**用？（拒绝形式主义负优化）

真正的工程提效，必须诚实说明不适用边界：
- **已知很短的命令不要套 `sieve`**：像 `git status -s`、`pwd` 或读一个 30 行的小配置文件，直接执行最快（`sieve` 对 `<1.2 KB` 输出本身也是原样直通，不必多套一层子进程）。只有当输出可能超过 `2 KB`（测试、构建、CI 日志、API JSON、网页 HTML）时才用 `sieve`。
- **100% 轮次都必须遵守的身份底线不要放进 `recall`**：全局最高准则（5~10 条）直接写在 `AGENTS.md` / `SOUL.md` 顶部；`recall` 专门用来装那 **50~1,000 条「平时不用占地方、碰到特定目录/特定事务才必须想起来」的条件规则**。
- **不要把大段聊天记录往 `baton` 里塞**：`baton` 之所以好使，就在于它只存经过验证的硬事实和已证伪死胡同，永远保持在 `<400 Tokens` 的高信噪比状态。
