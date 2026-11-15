# `splice` — AST 符号级精准读写与跨文件原子补丁器

[English](./README.md) | [简体中文](./README.zh-CN.md)

**独立单文件夹微基建（`skills/splice/`）** · 零第三方依赖（纯 Python 标准库） · 待机仅 `~68 tokens`。

## 为什么必须用它（来自 Codex、Antigravity 与 Grok 185 场真实会话挖掘）

在对 **OpenAI Codex CLI**、**Google Antigravity CLI** 与 **xAI Grok CLI** 的真实开发轨迹做跨框架统计审计时，我们发现了高度一致的底层读写损耗：

- **50.1% 的同会话重复读文件**（Antigravity 中 `660 / 1,318` 次 `read_file` 在反复重读同一文件）以及 **4,835 次命令行盲切行号**（Codex 中频繁执行 `Get-Content | Select-Object -Skip -First` 猜行号）。
- **231 次补丁匹配失败与静默漏改**（`apply_patch` / `edit_file` 因 CRLF 换行符、行尾空格或上方修改引发的行号漂移报错；而手写 Python `str.replace()` 在未命中时甚至静默退出码为 0）。
- **155 次 Windows `cp936`/`gbk` 编码崩溃**（Agent 手写 `Path.read_text()` 未指定 `encoding="utf-8"`，遇到破折号 `—`、箭头 `→` 或中文直接抛 `UnicodeDecodeError`），外加残留 `.pyc` 字节码干扰单测。

`splice` 为 **Coding Agent** 与 **个人助手 Agent** 提供零依赖符号级手术刀：

- **`splice view`**：按 AST 精确读取单个 `Class`、`Class.method`、`function` 或 Markdown 指定 `## 标题` 章节（仅百余 Tokens，无需重读整份大文件）。
- **`splice apply`**：跨文件 JSON 原子事务修改（支持 `replace`、`replace_symbol` 按符号直接替换方法体、`write`）——强制 UTF-8、自动兼容并保留原始 CRLF/LF 换行符、严禁 0 命中静默漏改、写入前统一校验全文件 `ast.parse` / `json.loads` 语法（任一文件语法报错即**全量原子回滚**），并自动清理对应 `.pyc` 缓存。
- **`splice check`**：`< 50ms` 完成全仓 UTF-8 合法性、AST 语法、冲突标记与仓内相对/绝对导入完整性体检。

## 只拷这一个文件夹即可用

```bash
cp -r skills/splice ~/.agents/skills/
```

## 常用命令

```bash
# 按符号精确查看方法体或 Markdown 章节
python scripts/splice.py view src/engine.py --symbol Engine.process
python scripts/splice.py view NOTES.md --heading "待办事项"

# 跨文件原子补丁事务
python scripts/splice.py apply --cwd . --spec edits.json

# 50ms 内完成全仓 UTF-8 + 语法 + 内部导入校验
python scripts/splice.py check .
```
