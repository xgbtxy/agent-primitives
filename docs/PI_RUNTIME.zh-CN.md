# Pi Coding Agent 运行时维护手册（`ap pi` + `zz-ap-accelerator.ts`）

`ap` 统一维护面向 `@earendil-works/pi-coding-agent` v1.5+ 的零提示词开销扩展（`zz-ap-accelerator.ts`，经 `ap init -a pi --with-hooks --no-rules` 安装）与纯 Rust 幂等运行时补丁器（`ap pi`）。

## 常用命令

```bash
# 查看自动发现的 @earendil-works/pi-coding-agent 安装目录与 6 项补丁状态
ap pi status

# 幂等应用全部 6 项通用运行时补丁（原文件安全备份至 ~/.ap/pi-backups/<version>/，不污染 node_modules）
ap pi patch

# 仅预演不写盘
ap pi patch --dry-run

# 从 ~/.ap/pi-backups/<version>/ 一键还原原始文件
ap pi restore
```

若 Pi 安装在自定义目录，可传 `--pi-dir <path>` 或设置环境变量 `PI_PACKAGE_DIR`。

## 维护的 6 项通用 Pi v1.5+ 运行时补丁

1. **`before_agent_start_entries`**：扩展 `emitBeforeAgentStart` 与 `AgentSession._prepareRun`（同时覆盖打包后的 `dist/bundle/chunks/*.js` 与未打包的 `dist/core/*` + `types.d.ts`），允许 `before_agent_start` 钩子返回 `entries?: SessionBoundaryDraft[]`，从而在新提示词发往模型前 0ms 原位提交自保留零历史边界（`{ type: "compaction", firstKeptEntryId: null, summary }`），等效程序化 `/new` 且保留完整会话记录与前缀缓存。
2. **`windows_python_utf8`**：在 Windows 子进程环境中自动注入 `PYTHONUTF8=1` 与 `PYTHONIOENCODING=utf-8`，根治 GBK 编解码崩溃。
3. **`codemode_sleep_and_workers`**：向 QuickJS `codemode` 沙箱注入可中止的 `await sleep(ms)`（上限 30s），并将并发 worker 上限 `DEFAULT_MAX_CODEMODE_WORKERS` 从 `2` 提升至 `4`。
4. **`http_keepalive_and_transient_retry`**：加固 HTTP/TCP Keep-Alive 与连接超时（20s 连接、25s 保活），补充瞬时断流/TLS 握手中断/连接超时/网关错误重试集合（`ECONNRESET`、`ETIMEDOUT`、`EPIPE`、`ECONNREFUSED`、`UND_ERR_SOCKET`、`socket disconnected`、`before secure tls`、`connect timeout`、`this operation was aborted`、`bad gateway`、`gateway timeout`）及抖动指数退避。
5. **`virtual_model_cache_warmer`**：修复 `CacheWarmer` 对虚拟模型（`api === "pi-virtual"`，如 `deepseek/flash-auto`）的匹配判断，确保动态思考档位仍可正常触发 KV-Cache 保温。
6. **`codemode_web_builtins`**：直接向 QuickJS `codemode` 沙箱注入标准 Web/JS 全局对象（基于 `sleep` 的 `setTimeout`/`clearTimeout`、`atob`/`btoa`、`TextEncoder`/`TextDecoder`，以及支持 `utf8`/`base64`/`base64url`/`hex`/`latin1`/`concat` 的轻量 `Buffer` 兼容层），根治 `ReferenceError: setTimeout/Buffer is not defined`。

## 联动守卫感知 `ap atlas surface` 与 `ap baton frontier` 消除长任务疲劳

长会话后期仅靠上下文记忆容易误判覆盖度或重复死胡同，推荐将清脑轮换（`/ap fresh [goal]` 或 `before_agent_start` 零历史边界）与外部确定性探索前沿搭配使用：

```bash
# 1. 一次扫描源码、配置清单、归档及原生 PE/ELF/Mach-O 二进制，按 9 条正交轴线与守卫位置标注（none_nearby/guard_after/guard_before）建立前沿
ap atlas surface . --include-binaries --json | ap baton frontier seed --stdin

# 2. 按“未探轴线绝对优先”策略取出下一批 Top-K 未测目标
ap baton frontier next -k 5

# 3. 标记已测/已停项，确保每次清脑切换后绝不重跑死胡同
ap baton frontier mark <id-or-target> --status stopped --reason "已核实"
```
