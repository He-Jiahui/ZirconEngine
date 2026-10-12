# Runtime44 · 诊断日志直接编码进批量缓冲

日期：2026-10-09。对应[原计划](../44-process-diagnostic-log-router-filter-record-queue-sink-durability-rotation-crash-multi-session-product-integration-review.md)的 R44-P1-12 / R44-P1-25 剩余物化路径；本记录不关闭整个条目。

## 实际代码修复

`sink/worker.rs::flush_pending` 先构建最终批量 `Vec<u8>`，原先再为每条记录创建完整日志 `String`，其中 `message.replace` 还创建换行转义字符串，最后把完整日志复制进批量缓冲。现在 `append_diagnostic_log_line` 借用时间戳、level、scope 和消息片段，直接写入同一批量缓冲。

实际接入四个文件：`zircon_runtime/src/diagnostic_log/sink.rs`、`sink/worker.rs`、`sink/tests/mod.rs`、`sink/tests/batching.rs`。只把消息中的真实 LF 转为字面 `\n`，普通反斜杠、字面 `\n`、Unicode 和 CR 保持原字节。最终 owned Vec、每批时间戳、FIFO、字节/记录/时间触发、实际输出、flush/sync、镜像失败统计和关闭/join 合同保留。

## 实际证据

- 候选 patch SHA-256：`9238282c2e62b42c18de853fe38d9dda05a6ba1a5d570ac3ecf1c7d28be1db93`。
- 独审 `.codex/outbox/runtime44-direct-batch-encoding-review-v1.json` SHA-256：`836c526326ccc9e84a4b50205821471d8ea06c9e91e956638ccfce0d458d5e34`，接受源码候选，零阻断。
- 实际接入 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/runtime44-direct-batch-encoding-v1/actual-canonical-apply.json` SHA-256：`7e4846edaca646895365ba6745f78752e640a11df5af2b83d8fac05699398490`。四个前像及 29 个依赖守卫匹配；apply check/apply、Rust 2021 fmt check、scoped diff check 均 exit 0，共享 index/control 未变。

源码增加两个回归：字面反斜杠与 Unicode 的精确字节检查，以及真实 worker 四记录同批输出、FIFO、flush/shutdown、bytes/records/errors 检查。原格式 golden 断言保留。原生执行尚未发生，三项筛选对应一项旧用例和两项新用例。

## 合并批量验证

向现有 Runtime #2 的七个 OR filters 追加上述三项精确名称，并明确选入六项既有 batching/lifecycle/durability 回归。配置仍为 `target-editor-host,test-support`，不增加 feature 或独立构建。

现有 Release #9 的原 flags 和两个原 selectors 保留，追加原日志风暴、critical admission 两个 ignored producers。54 个 `requested_rate` 矩阵行与同前缀的一行 companion 分开核对；这是两个 native cases，不能记为 54 个测试。critical 保留 20 个原始 `admission_ns`、2ms timeout 和 p95 ≤ 50ms，未添加 max/p99 阈值。

## 尚未完成

需要实际 native 通过、完整非零矩阵、同负载 before/after 分配和 CPU 测量、完整分位数及真实 NVMe/慢盘/console 验收。当前每批最终 Vec 仍新建，producer 时间戳、bounded arena 和更完整 record/callsite 路径仍是原计划后续范围。R44-P1-12 / P1-25 / P1-50、G35 / G36 和产品性能门保持开放；不能以源码减少临时 String 声称性能达标。

完成状态见[Runtime 完成列表](../../../astra/features/runtime/2026-10-09-runtime44-direct-batch-log-encoding-completion-list.md)。
