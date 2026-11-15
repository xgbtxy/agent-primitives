# `splice` — 多语言符号级精准读写、原子补丁与自动测试门禁

[English](./README.md) | [简体中文](./README.zh-CN.md)

**单文件夹独立微型基础设施（`skills/splice/`）** · 零第三方依赖 · 空闲加载仅 `~64 tokens`。

## 为什么需要它（源自四大 CLI 共 69,963 次真实工具调用遥测）

跨 **OpenAI Codex CLI**、**Google Antigravity CLI**、**xAI Grok CLI** 与 **Claude Code** 的本地会话审计暴露了普遍存在的读/改/测摩擦：

- **50.1% 同一文件重复全量读取** 与 **4,835 次手动终端按行切片读取**，只为看一个 20 行函数或确认修改后偏移的行号。
- **244 次补丁目标失配报错**（CRLF/LF 换行符、行尾空格、行号漂移）+ **101+ 次 Windows `cp936`/`gbk` 编码崩溃** + 旧 `__pycache__/*.pyc` 字节码缓存陷阱。
- **盲猜测试运行器空耗轮次**：在纯 `unittest` 仓库盲敲 `python -m pytest` 报错、在 Node 22 原生 TypeScript 仓库漏掉 `--experimental-strip-types --test`、或额外多花一轮命令去删除临时 JSON 补丁文件。

`splice` 为 **Coding Agent** 与 **个人助手 Agent** 提供确定性的精准读、原子写与一键验真原语：

- **`splice view`**：支持一次传入 **1 个或多个文件**，在 `< 5 ms` 内精准提取 **Python、TypeScript/JavaScript、Go、Rust** 的类/方法/函数符号区间（`--symbol`）、显式行区间（`--lines`）或 Markdown 标题章节（`--heading`）。
- **`splice apply`**：通过 `--stdin`、`--b64` 或 `--spec ... --unlink-spec`（成功后自动清理临时文件）原子化执行跨文件事务（`replace`、支持 Py/TS/JS/Go/Rust 的 `replace_symbol`、`write`）——强制 UTF-8、保留原文件 CRLF/LF、落盘前校验语法并在异常时**全量回滚**，同时自动清理过期 `.pyc`。
- **`splice check [--test]`**：亚 50ms 校验全仓 UTF-8、语法、冲突标记与 Python + TypeScript/JS 相对导入；加上 **`--test`** 自动识别并运行仓库真实测试套件（`pytest`、`unittest`、`node --experimental-strip-types --test`、`node --test`、`go test`、`cargo test`）且自动截断冗余日志、完整保留报错断言。

## 1 条命令极简接入（或单独拷走本文件夹）

```bash
# 推荐：1 条命令在当前仓库完成接入
python ap.py init --here

# 或仅单独复制本文件夹：
cp -r skills/splice ~/.agents/skills/
```

## 实战命令（Windows PowerShell 与 Bash 双端安全）

```bash
# 一次精准读取多个文件中的指定类/方法（Py / TS / JS / Go / Rust）或 Markdown 章节
python .agents/ap.py splice view src/ledger.ts src/money.ts --symbol LedgerEngine.postBatch --symbol computePlatformFee
python .agents/ap.py splice view NOTES.md --heading "Action Items"

# 跨文件原子修改并自动清理临时 spec 文件（也可在 PowerShell 用 `@'...'@ |` 或 Bash 用 `<<'EOF'` 走 --stdin）
python .agents/ap.py splice apply --cwd . --spec _edits.json --unlink-spec

# 1 次调用完成全仓静态校验 + 自动识别运行测试套件
python .agents/ap.py splice check . --test
```
