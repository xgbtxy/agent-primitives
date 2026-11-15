# `probe` — 因果执行追踪、可达性切片与差分显微镜

[English](./README.md) | [简体中文](./README.zh-CN.md)

**独立单文件夹微基建（`skills/probe/`）** · 零第三方依赖（纯 Python 标准库） · 待机仅 `~78 tokens`。

## 为什么必须用它（来自真实前沿 CLI Agent 轨迹解剖）

在对前沿 Coding Agent（如本地 `Grok Build CLI` 运行 `grok-4.7`）进行多文件真实排障全链路遥测时，我们捕捉到了拖垮大模型基础编码过程的两大核心病灶：

1. **静态调用图可达性盲区**：纯文本工具（`list_dir`、`grep`、`read_file`）无法区分真正处于执行链路上的核心模块与仓库里未被引用的旁路模块，导致前几轮盲读大量无关文件并持续污染后续每一轮的上下文缓存。
2. **「脑内模拟堆内存与别名状态」导致的推理 Token 暴涨**：面对浅拷贝嵌套字典共享引用（Shallow-Copy Aliasing）、原地修改入参（In-Place Mutation）、状态回滚键不一致静默失效（如传入 `event_id` 去删以 `digest` 为键的字典）以及 Python `True == 1` / `hash(True) == hash(1)` / `None` 三值逻辑边界时，模型往往在单轮思考中空转消耗 **2.5 万～5.5 万+ Reasoning Tokens** 反复自我推翻，甚至手写几十行脆弱的临时 Monkeypatch 脚本。

`probe` 用纯标准库在 **`< 0.1 秒`** 内为 Agent 提供三项确定性基础过程能力：

- **`probe slice`（静态可达性切片与跨文件契约对照）**：从入口包或测试出发解析相对/绝对导入与 AST 符号图，直接剪除未引用干扰文件，并将跨文件调用点与目标函数签名并排展示。
- **`probe run`（因果执行与堆状态显微镜）**：基于受控作用域 `sys.settrace`，一次运行自动告警三类高危根因：**`[ALIAS HAZARD]`**（返回新对象却与入参共享底层可变容器引用）、**`[IN-PLACE MUTATION]` / `[MUTATED ON EXCEPTION]`**（原地篡改调用方入参或异常中断后遗留脏状态）、**`[STATE NO-OP]`**（无返回值状态修改方法执行后 `self` 毫无变化，并直接打印当前真实键样本）。
- **`probe diff`（差分语义与状态缓存边界探针）**：自动构造 `None`、`True` vs `1`、`False` vs `0`、嵌套列表/字典及连续状态序列，严格校验值与类型标签一致性（`type(a) is type(b)`）、缓存跨调用串味与入参纯度。

## 只拷这一个文件夹即可用

```bash
cp -r skills/probe ~/.agents/skills/
# 或安装至 Grok CLI / Claude Code：
cp -r skills/probe ~/.grok/skills/
cp -r skills/probe ~/.claude/skills/
```

## 常用命令

```bash
# 1. 静态可达性切片（自动剪除未引用模块并输出跨文件调用契约）
python scripts/probe.py slice . --entry mypkg

# 2. 运行时因果追踪（自动捕获浅拷贝别名共享、入参原地篡改与状态空操作）
python scripts/probe.py run --cwd . --focus mypkg --code "from mypkg import Engine; ..."

# 3. 优化器/缓存差分边界等价性探针
python scripts/probe.py diff --cwd . \
  --setup "from mypkg import Engine; opt=Engine(optimize=True); raw=Engine(optimize=False)" \
  --fn-a "lambda d: opt.eval(d)" \
  --fn-b "lambda d: raw.eval(d)"
```
