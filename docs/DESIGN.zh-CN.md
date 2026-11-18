# 反淘汰架构深潜：瞄准思考链与物理边界的纯 Rust 原语设计（`ap` v1.2）

[English](./DESIGN.md) | [简体中文](./DESIGN.zh-CN.md) · [演进总手册](./HANDBOOK.zh-CN.md) · [实战手册](./PLAYBOOK.zh-CN.md)

## 1. 为什么 95% 的开源 Agent 项目活不过半年？

- **纯提示词角色扮演与套路清单最先消亡**：前沿模型每次强化学习后训练都会吸收这些软套路，冗长的 `AGENTS.md` 说教反而退化为注意力噪音与延迟税。
- **重型多智能体包装器其次消亡**：各大官方宿主（`codex`、`opencode`、`pi`、`claude`、`agy`、`qwen`、`gemini`、`grok`）原生掌控对话循环与子智能体调度。
- **有损向量 RAG 包装器随之淘汰**：近似语义切片不仅需要 Docker 与 API Key，更比不上毫秒级精确语法树地图与确定性检索。

**架构铁律**：绝不生产 AI Slop，只在**自回归 Token 生成与操作系统/具体语法树/浏览器进程发生物理错位的四道边界**上，用**纯 Rust 单文件二进制（`ap` / `ap.exe`）** 同时输出 **Unix CLI** 与 **标准 `ap mcp` JSON-RPC 服务**。

---

## 2. 大模型不可逾越的四道认知与物理边界

| `<thinking>` 思考链与系统边界 | 本机 `10.1 万+` 次调用与真实推理流实测损耗 | 纯 Rust 确定性器官与开源顶尖算法融合（`v1.2`） |
| :--- | :--- | :--- |
| **1. 冷启动高熵盲猜与旧屎山污染** | `3 万+` 次超 `8 KB` 输出刷屏；开局耗费 `10k–35k` 思考字符盲猜环境；误读历史废文件照抄 `: any` 与空 `catch {}`。 | **`ap-atlas`**（融合 Aider **Personalized PageRank** + 宿主运行时指纹 + DOM `#id`/API 断链与防屎山雷达）+ **`ap-sieve`**（日志 `-98.8%`、`5.2ms` FTS5）+ **`ap-probe slice`**（`6.3ms` 死代码剪除） |
| **2. 跨文件记忆漂移与补丁缩进/转义报错** | 跨 6+ 文件写串 HTML `#id` ↔ JS `getElementById`；因 2 空格缩进偏差产生 `735` 次补丁报错；Windows PowerShell 转义崩溃。 | **`ap-splice`**（`0.7ms` Tree-sitter 精读 + `2.3ms` 零转义及 **Aider 三级精确→行尾去空→相对缩进平移** 语法树预检 `apply`）+ **`ap mcp`**（`ap_edit` 纯 JSON 零转义） |
| **3. Windows 70% 手工验证税与内存脑补** | **71%–75% 的步骤（`85 万+` Token）** 耗在 Windows 后台服务卡死、残留端口及手写 150 行 CDP 脚本；单次内存别名脑补空烧 `55k` 思考 Token。 | **`ap-probe smoke`**（自动绑空闲 `PORT` + 文件日志防死锁 + **纯 Rust `TcpStream` 交互式无头 Edge/Chrome CDP** `OPEN`/`FILL`/`SELECT`/`CLICK`/`EVAL` + `taskkill /T /F`）+ **`ap-probe run/diff`** |
| **4. `AGENTS.md` 叮嘱失灵与上下文压缩失忆** | 长会话下模型无视文字叮嘱写出吞异常空 `catch {}`；`/compact` 擦除已证伪的死胡同。 | **`ap-guard`**（`<2ms` `exit 2` `.agents/guard.json` 硬规则 + 自动联动 `tsc`/`biome`/`cargo` + **`rtk` 式 `PreToolUse` `updatedInput` 自动脱水**）+ **`ap-baton`** + **`ap-recall`**（`3.4ms` BM25） |

---

## 3. 为什么选择纯 Rust（`v1.2`）+ 原生内置 `ap mcp`？

1. **编译期零开销安全与微秒启动（`Rust > C & Python`）**：所有权安全、`OnceLock` 全局单次编译自动机（热循环零正则重编译）、零 GC 停顿、告别 Python Hook 每次 `45ms` 冷启动税。
2. **让 `$0` 免费模型与小白直接满血提效（`ap mcp`）**：免费轻量模型（`opencode/big-pickle`、`glm-5.3-flash-free`、`deepseek-flash`、`qwen`）在 Windows Shell 下极易撞上引号转义地狱；通过原生 MCP 暴露 `ap_orient`、`ap_edit`、`ap_smoke`、`ap_verify`、`ap_probe`，100% 抹平跨平台命令摩擦。
3. **7 个物理隔离 Crate（`0` 兄弟依赖）**：每个 `crates/ap-*` 均可独立编译安装，合体为单个约 `11 MB` 原生可执行文件（全量 7 技能静默仅占 **`390 tokens`**）。
