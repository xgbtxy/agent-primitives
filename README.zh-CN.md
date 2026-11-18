<div align="center">
  <img src="./assets/logo.svg" width="60" height="60" alt="ap logo" />
  <h1>ap · Coding Agent 执行提效引擎</h1>
  <p><strong>别再让你的 AI 编程助手把 50%～80% 的 Token 白白烧在 Windows / Mac 终端报错、乱码、端口卡死和刷屏日志上了。</strong></p>
  <p>
    <a href="https://github.com/xgbtxy/agent-primitives/actions/workflows/ci.yml"><img src="https://github.com/xgbtxy/agent-primitives/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
    <a href="https://github.com/xgbtxy/agent-primitives/releases/latest"><img src="https://img.shields.io/github/v/release/xgbtxy/agent-primitives?color=22c55e&label=%E7%9B%B4%E6%8E%A5%E4%B8%8B%E8%BD%BD" alt="Latest Release" /></a>
    <img src="https://img.shields.io/badge/%E5%B9%B3%E5%8F%B0-Windows%20%7C%20macOS%20%7C%20Linux-0ea5e9" alt="Platform" />
    <img src="https://img.shields.io/badge/%E8%BF%90%E8%A1%8C%E6%97%B6%E4%BE%9D%E8%B5%96-0%20(Rust%20%E5%8D%95%E6%96%87%E4%BB%B6)-f59e0b" alt="Zero Dependencies" />
    <a href="./LICENSE"><img src="https://img.shields.io/badge/license-MIT-64748b" alt="MIT License" /></a>
  </p>
  <p>
    <a href="./README.md"><strong>English</strong></a> · <strong>简体中文</strong> · <a href="./docs/HANDBOOK.zh-CN.md">核心研发理念</a> · <a href="https://github.com/xgbtxy/agent-primitives/releases/latest">安装包下载</a>
  </p>
  <img src="./assets/hero-comparison.zh-CN.svg" width="100%" alt="未安装 ap 与安装 ap 后的真实体验对比" />
</div>

---

## 10 秒一行安装，装完即忘

不需要装 `Python`、不需要装 `Node`、不需要装 400MB `Playwright`，**更不需要你学任何新命令**。复制一行执行，自动识别并接好你电脑上的所有 Coding Agent（`Claude Code`、`Codex`、`OpenCode`、`Gemini CLI`、`Qwen Code`、`Pi`、`Goose`、`Antigravity`、`Grok`）：

**Windows（打开 PowerShell 粘贴回车）：**
```powershell
irm https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.ps1 | iex
```

**macOS / Linux：**
```bash
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.sh | sh
```

> **喜欢直接下载单文件？** 点击直接下载（**[Windows 64位 `ap.exe` (.zip)](https://github.com/xgbtxy/agent-primitives/releases/latest/download/ap-x86_64-pc-windows-msvc.zip)** · **[macOS M芯片](https://github.com/xgbtxy/agent-primitives/releases/latest/download/ap-aarch64-apple-darwin.tar.gz)** · **[macOS Intel](https://github.com/xgbtxy/agent-primitives/releases/latest/download/ap-x86_64-apple-darwin.tar.gz)** · **[Linux x64](https://github.com/xgbtxy/agent-primitives/releases/latest/download/ap-x86_64-unknown-linux-musl.tar.gz)**），解压运行 `ap init` 即可。想卸载随时敲 `ap uninstall` 一键还原。

---

## 装完之后怎么用？（完全不改变你的使用习惯）

1. **平时怎么跟 AI 说话，现在还怎么说。** `ap` 在底层通过原生 MCP 与毫秒级 Hook 自动帮模型避坑、省流、验页面。
2. **任何时候想看看它帮你省了多少 Token**，在终端敲一下 **`ap stats`**（或敲 `ap doctor` 体检环境）：

```text
$ ap stats
[ap stats] Local Token & Windows Execution Ledger (100% offline)
----------------------------------------------------------------
  Accelerated Agent Actions : 28 calls
  Context & Log Diet        : 612.4 KB -> 19.2 KB (-96.9%)
  Windows Pitfalls Healed   : 11 auto-fixes (UTF-8/GBK, PS `&&`/`curl`, PATH, locks)
  AST Edits & Smoke Gates   : 14 verified checks (Tree-sitter + Edge CDP + 0-slop)
  Total Value Delivered     : ~186,800 tokens saved
```

---

## 为什么能实打实省下 50%～83% 的 Token？

主流大模型 90% 以上的训练语料都是 Linux Bash，但大多数开发者都在 **Windows 和 macOS** 上用 Coding Agent。扫描本机 160 个真实 Windows 会话（`101,108` 次工具调用），模型共撞上 **528 次本可避免的环境报错**——每报错一次，几万字的上下文就要重新发一遍去盲猜重试。

`ap` 顺着模型干活的 **5 个真实阶段**，把白烧 Token 的坑在底层一次性填平：

| 模型干活阶段 | 没装 `ap`（你的 Token 死在哪里） | 装上 `ap` 后（后台自动完成，无需人工干预） |
| :--- | :--- | :--- |
| **① 看懂项目** | 上来连读十几个完整文件（连废弃旧代码一起塞进上下文），瞎猜本机环境。 | **<30 ms** 生成核心代码地图 + 前后端 `#id`/路由断链雷达 + Windows 环境底牌（省 `99%` 盲读 Token）。 |
| **② 修改代码** | 被 Windows `CRLF` 换行、PowerShell 生成的 `UTF-8 BOM` / `UTF-16LE` 编码或缩进偏差卡住报错。 | 自动剥离 **UTF-8 BOM 与转换 UTF-16LE**，3 级缩进容错匹配，写盘前用 **6 语言语法树秒级验错**。 |
| **③ 跑终端命令** | **Windows 头号黑洞**：Python/Node 中文报 `'gbk' codec`，PS 5.1 敲 `&&` / `curl` / `<<EOF` 报语法错，几万行日志刷屏。 | 强制锁死无 BOM `UTF-8`，自动自愈 PS `&&`/`curl.exe`/URL `&`/POSIX 管道与 PATH，超长日志自动脱水 **-90%+**。 |
| **④ 起服务点页面** | **70%+ 轮次耗在验证**：后台起服务锁死管道卡住终端、`EADDRINUSE` 端口冲突、或去装 400MB Playwright。 | 自动清端口分新端口，纯 Rust 直连**系统自带无头 Edge / Chrome** 真机点按钮抓报错，跑完清干净进程树（**<4.5s**）。 |
| **⑤ 交卷检查** | 偷懒留下 `: any`、`@ts-ignore`、空 `catch {}` 吞错等 AI Slop，或没跑通就停下。 | **<2 ms `exit 2` 硬门禁**：交卷前自动拦住反 Slop 违规与前后端接口断链，逼模型修干净再交差。 |

---

## Windows 11 真机实测对比（零水分）

在 Windows 11 真机上用 `$0` 免费智能体与主流商业 CLI 实测（覆盖小白一句话需求、全栈升级 `WorkPulse`、空目录从零开发随机项目 `SplitMate`）：

| 测试 Agent 与场景 | 未装 `ap` → **安装 `ap` 后**（耗时） | Token 总消耗对比 | 功能与零 Slop 验收 |
| :--- | :--- | :--- | :---: |
| **`OpenCode` 免费版 (`$0` `big-pickle`)** · *小白一句话需求* | `430.2s` → **`182.4s`（快 2.36 倍）** | `1,613,689` → **`401,930`（省 75.1%）** | `7/8` *(留 Slop)* → **`8/8`（0 Slop）** |
| **`Codex CLI` (`gpt-6-astra`)** · *空目录随机新项目* | `345.3s` → **`244.6s`（快 29.1%）** | `1,116,369` → **`186,379`（省 83.3% / 少花 6 倍）** | **`6/6` → `6/6`** |
| **`Codex CLI` (`gpt-6-astra`)** · *全栈功能升级* | `333.6s` → **`294.3s`（少调 54% 工具）** | `1,053,580` → **`364,682`（省 65.4%）** | **`8/8` → `8/8`** |
| **`Pi CLI` (`deepseek-flash`)** · *全栈功能升级* | `483.0s` → **`271.5s`（快 43.8%）** | `3,628,149` → **`2,295,801`（省 36.7%）** | **`8/8` → `8/8`** |
| **`Pi CLI` (`deepseek-flash`)** · *空目录随机新项目* | `1,821.1s` → **`1,499.3s`（快 322 秒）** | `15.55M` → `19.94M` *(模型额外跑了子集和暴力对拍)* | `5/6` *(2 处 Slop)* → **`6/6`（0 Slop）** |

<details>
<summary><strong>底层架构与硬核工程细节（给想看源码的开发者）</strong></summary>

<br/>

- **纯 Rust 零耦合单文件架构**：1 个原生 `ap` 二进制内聚 7 个物理隔离 crate（`crates/ap-{atlas,splice,probe,sieve,guard,recall,baton}`），子 crate 之间 `0` 相互依赖，内置 SQLite FTS5 全文检索引擎与 6 语言编译态 Tree-sitter C 语法树，常驻提示词开销仅 `390` Tokens。
- **一键复现基准测试**：运行 `ap bench`（详见 [`bench/results.json`](./bench/results.json)）。
- **深度文档**：
  - [核心研发理念与演进手册 (`docs/HANDBOOK.zh-CN.md`)](./docs/HANDBOOK.zh-CN.md) | [R&D Philosophy (`docs/HANDBOOK.md`)](./docs/HANDBOOK.md)
  - [系统架构设计 (`docs/DESIGN.zh-CN.md`)](./docs/DESIGN.zh-CN.md) | [Architecture (`docs/DESIGN.md`)](./docs/DESIGN.md)
  - [高阶实战手册 (`docs/PLAYBOOK.zh-CN.md`)](./docs/PLAYBOOK.zh-CN.md) | [Playbook (`docs/PLAYBOOK.md`)](./docs/PLAYBOOK.md)

</details>

## 开源协议

[MIT](./LICENSE)
