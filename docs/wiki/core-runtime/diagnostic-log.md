---
related_code:
  - zircon_runtime/src/diagnostic_log/mod.rs
  - zircon_runtime/src/diagnostic_log/level.rs
  - zircon_runtime/src/diagnostic_log/settings.rs
  - zircon_runtime/src/diagnostic_log/sink.rs
  - zircon_runtime/src/diagnostic_log/diagnostics.rs
  - zircon_runtime/src/diagnostic_log/platform.rs
implementation_files:
  - zircon_runtime/src/diagnostic_log/level.rs
  - zircon_runtime/src/diagnostic_log/settings.rs
  - zircon_runtime/src/diagnostic_log/sink.rs
  - zircon_runtime/src/diagnostic_log/sink/worker.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/mvp/index.md
tests:
  - zircon_runtime/src/diagnostic_log/level/borrowed_parse_tests.rs
  - zircon_runtime/src/diagnostic_log/diagnostics/tests
  - zircon_runtime/src/diagnostic_log/sink/tests
doc_type: module-detail
---

# 诊断日志

`zircon_runtime::diagnostic_log` 是进程级、可配置的诊断输出层。它把模块日志写入有界队列，由专用 sink worker 批量写到控制台和文件；调用方不需要知道文件路径、线程或平台目录。该模块由 `diagnostic-log` feature 控制，未启用时产品入口不应假设日志文件存在。

## 处理链

```text
write_* / write_*_lazy
        |
        v
scope filter（全局 + 最长前缀规则）
        |
        v
bounded queue（记录数/字节数/critical timeout）
        |
        v
sink worker（batch + flush interval）
        |                  |
        v                  v
console output       file output + sync_data
```

过滤在入队前完成。`write_*_lazy` 只有在 scope/level 允许时才执行消息闭包，适合昂贵的格式化或序列化。日志行包含 timestamp、level、scope 和经过换行转义的消息；日志 sink 的内部锁和 worker 线程不属于调用方 API。

## 日志级别与过滤

`DiagnosticLogLevel` 按严重程度排列为 `Verbose`、`Debug`、`Log`、`Warn`、`Error`。`DiagnosticLogFilter::Minimum(level)` 允许该级别及更高等级，`Off` 完全关闭。Debug 构建默认是 `Verbose`，release 构建默认是 `Log`。

`DiagnosticLogFilterConfig` 支持全局级别和按 scope 前缀的规则。匹配多个规则时使用最长前缀，例如 `runtime::asset::loader` 覆盖 `runtime::asset`，而不会把规则相加。解析失败返回 `DiagnosticLogLevelParseError`，环境变量覆盖会记录一条 warning 后回到默认值。

环境变量优先级如下：

1. `ZIRCON_LOG_LEVEL`：全局最低级别。
2. `ZIRCON_LOG_FILTER`：全局级别与 `scope=level` 规则。
3. `ZIRCON_LOG`：兼容别名，仅在上项为空时使用。
4. `RUST_LOG`：最后的兼容别名。

```rust
use zircon_runtime::diagnostic_log::{
    DiagnosticLogFilter, DiagnosticLogFilterConfig, DiagnosticLogLevel,
};

let config = DiagnosticLogFilterConfig::parse(
    "warn,zircon_runtime::asset=debug,zircon_runtime::asset::import=verbose",
    DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Log),
)?;

assert!(config.allows(
    DiagnosticLogLevel::Verbose,
    "zircon_runtime::asset::import::gltf",
));
assert!(!config.allows(DiagnosticLogLevel::Log, "zircon_runtime::ui"));
# Ok::<(), zircon_runtime::diagnostic_log::DiagnosticLogLevelParseError>(())
```

## 初始化与输出位置

`DiagnosticLogSettings::new(channel)` 创建一套控制台和文件都开启的配置，默认使用 `DiagnosticLogLocation::LocalFirst`。channel 会被清洗为只含 ASCII 字母、数字、`-` 和 `_` 的文件名；空 channel 回退为 `runtime`。

文件目录按候选顺序尝试：

| 配置 | 候选顺序 |
| --- | --- |
| `LocalFirst` | `ZIRCON_LOG_ROOT`、可执行文件目录下的 `logs/<timestamp>`、当前目录下的同名目录、Unity 兼容用户目录 |
| `UnityCompatibleFirst` | `ZIRCON_LOG_ROOT`、平台 Unity 兼容用户目录、可执行文件目录、当前目录 |

设置 `ZIRCON_LOG_ROOT` 可以把第一候选固定到受控卷。目录创建或文件打开失败不会让进程启动失败；sink 会保留诊断，若所有文件候选失败仍可只写控制台。

## Rust 调用接口

初始化函数返回实际文件路径；`None` 表示没有可用文件 sink，并不表示日志调用会 panic。重复初始化由进程级 controller 合并到当前 sink，调用方不应创建第二个全局 worker。

```rust
use std::time::Duration;
use zircon_runtime::diagnostic_log::{
    flush_process_log, initialize_process_log_with_settings, shutdown_process_log,
    write_error, write_log, DiagnosticLogFilter, DiagnosticLogLevel,
    DiagnosticLogSettings,
};

let settings = DiagnosticLogSettings::new("editor")
    .with_filter(DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Debug));
let log_path = initialize_process_log_with_settings(settings);

write_log("editor::startup", "editor host initialized");
write_error("editor::startup", "example diagnostic");

let flushed = flush_process_log(Duration::from_millis(250));
let closed = shutdown_process_log(Duration::from_secs(2));
// `flushed`/`closed` 为 false 时应把输出故障报告给宿主。
let _ = (log_path, flushed, closed);
```

按等级的快捷函数是 `write_diagnostic_log`（Verbose）、`write_debug_log`、`write_log`、`write_warn` 和 `write_error`；需要动态等级时使用 `write_diagnostic_log_at`。对应的 lazy 版本是 `write_*_lazy` 和 `write_diagnostic_log_lazy_at`。

过滤检查可以在构造昂贵消息前调用：

```rust
use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows_for_scope, write_debug_log_lazy, DiagnosticLogLevel,
};

if diagnostic_log_allows_for_scope(DiagnosticLogLevel::Debug, "runtime::asset") {
    write_debug_log_lazy("runtime::asset", || format!("cache entries={}", 42));
}
```

## Sink 配置与背压

`DiagnosticLogSinkSettings` 的默认值是：队列 4096 条、每批最多 256 条或 256 KiB、50 ms flush interval、critical enqueue 最多等待 2 ms。`with_queue_capacity`、`with_max_batch_records` 和 `with_max_batch_bytes` 会把 0 规范化为至少 1；过小的批次会增加系统调用和文件同步成本。

队列满时普通记录可以被丢弃并在 metrics 中计数；`Warn`/`Error` 等 critical 记录在限定时间内等待入队，超时仍不能阻塞游戏帧。输出错误、关闭状态和队列深度可通过 `diagnostic_log_sink_snapshot()` 读取 `DiagnosticLogSinkSnapshot`，用于诊断页和健康检查。

## 诊断快照与周期写入

`format_diagnostic_store_snapshot` 和 `format_diagnostic_store_current_snapshot` 把 Core 的诊断序列转换为稳定文本；`write_diagnostic_store_snapshot` / `write_diagnostic_store_current_snapshot` 通过日志层写出当前值。`DiagnosticStoreLogSchedule::repeating(wait)` 是无分配的周期调度器：一次 tick 可能发现多个到期周期，额外周期会计入 `coalesced_periods`，不会补发一串重复日志。

```rust
use std::time::Duration;
use zircon_runtime::diagnostic_log::DiagnosticStoreLogSchedule;

let mut schedule = DiagnosticStoreLogSchedule::repeating(Duration::from_secs(1));
if schedule.tick(Duration::from_millis(1100)) {
    // 读取一次诊断快照并写出；schedule.last_periods_due() 可用于观测积压。
}
```

## Panic、flush 与关闭

`install_process_log_panic_flush(timeout)` 安装一次进程级 panic hook。hook 在 sink worker 之外尝试有界 flush，再调用之前的 hook；它不会把 panic 转成成功。宿主在崩溃边界或动态库卸载前应显式调用 `flush_process_log`，并检查返回值。

`shutdown_process_log(timeout)` 会等待队列排空并对文件执行 `sync_data`。返回 `false` 表示超时或输出失败，进程退出本身不能作为“日志已持久化”的证据。动态 runtime 卸载必须先停止新的日志调用、完成 flush，再释放库中的函数和 sink worker。

## 平台与状态

- **已实现**：级别/前缀过滤、环境变量解析、控制台与文件双 sink、有界批处理、metrics 快照、panic flush 和有序关闭。
- **受限**：文件目录依赖平台权限；所有候选不可写时只有控制台输出。
- **受限**：模块由 `diagnostic-log` feature 控制，服务器或最小 profile 可能不编译该 facade。
- **内部实现**：`ProcessLogController`、`SinkRuntime`、worker channel 和动态 session lease。

相关测试覆盖过滤别名和最长前缀、目录候选顺序、批处理/背压、输出 durability、生命周期、所有权以及 release 性能门槛。调用方应把日志当作诊断通道，而不是业务数据存储或跨进程协议。
