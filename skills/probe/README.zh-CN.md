# `probe` — 多语言可达性打包切片与因果执行显微镜

[English](./README.md) | [简体中文](./README.zh-CN.md)

**单文件夹独立微型基础设施（`skills/probe/`）** · 零第三方依赖 · 空闲加载仅 `~62 tokens`。

## 为什么需要它（来自真实前沿 CLI 遥测诊断）

在对 **Grok Build CLI (`grok-4.7`)**、**Google Antigravity CLI (`agy`)** 与 **OpenAI Codex CLI (`gpt-6-astra`)** 的真实多文件排障会话做因果剖析时，我们定位到两个最消耗延迟与推理预算的底层黑洞：

1. **可达性盲区与串行读文件往返**：纯文本工具（`list_dir`、`grep`、`read_file`）无法区分真正处于调用链上的 8–12 个活跃模块与 15–20+ 个未引用的历史/死模块；即便找到活跃文件，智能体仍需串行发起 10–18 次 `read_file` 逐个读取。
2. **脑内模拟堆内存状态空转**：缺乏确定性运行时显微镜时，前沿模型会在思维链里空烧 **25,000–55,000 推理 tokens** 猜测浅拷贝嵌套字典别名共享、入参原地污染、空操作回滚（No-Op）以及 `True == 1` / `hash(True) == hash(1)` / `None` 边界语义。

`probe` 用三个 `< 0.1s` 纯标准库原语升级智能体底层排障能力：

- **`probe slice [--bundle]`**：覆盖 **Python、TypeScript/JavaScript（`.ts/.tsx/.js/.mjs`）、Go（`go.mod`）与 Rust（`mod`/`crate::`）** 的静态可达性切片——裁剪未引用干扰模块，并可通过 `--bundle` **一次调用直接输出全部活跃源码（带行号，默认预算 `28 KB`）**。
- **`probe run`**：带调用点自动去重（200 次循环亦严格控制在 `< 3.5 KB` 绝不爆上下文）的 `sys.settrace` 因果追踪器，自动标记 **`[ALIAS HAZARD]`**、**`[IN-PLACE MUTATION]` / `[MUTATED ON EXCEPTION]`** 与 **`[STATE NO-OP]`**，支持 `--code`、`--stdin`、`--b64`、`--script`、`--unittest`。
- **`probe diff`**：边界与状态缓存差分预言机（自动覆盖 `None`、`True` vs `1`、`False` vs `0`、嵌套容器、连续缓存污染与入参纯度检查）。

## 1 条命令极简接入（或单独拷走本文件夹）

```bash
# 推荐：1 条命令在当前仓库完成接入
python ap.py init --here

# 或仅单独复制本文件夹：
cp -r skills/probe ~/.agents/skills/
```

## 实战命令（Windows PowerShell 与 Bash 双端安全）

```bash
# 1. 多语言静态调用链切片 + 一次性打包活跃模块带行号源码（Py / TS / JS / Go / Rust）
python .agents/ap.py probe slice . --bundle

# 2. 因果执行显微镜（自动按调用点去重；多行脚本在 PowerShell 用 `@'...'@ |`，在 Bash 用 `<<'EOF'`）
python .agents/ap.py probe run --cwd . --focus mypkg --code "from mypkg import Engine; ..."

# 3. 优化前后 / 缓存前后差分边界等价性检验
python .agents/ap.py probe diff --cwd . \
  --setup "from mypkg import Engine; opt=Engine(optimize=True); raw=Engine(optimize=False)" \
  --fn-a "lambda d: opt.eval(d)" \
  --fn-b "lambda d: raw.eval(d)"
```
