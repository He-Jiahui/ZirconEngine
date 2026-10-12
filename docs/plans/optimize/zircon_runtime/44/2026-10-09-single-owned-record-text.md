# Runtime44 · 单份 owned 日志文本

日期：2026-10-09。承接[批量编码缓冲复用](2026-10-09-bounded-worker-batch-buffer-reuse.md)，对应[原计划](../44-process-diagnostic-log-router-filter-record-queue-sink-durability-rotation-crash-multi-session-product-integration-review.md) R44-P1-12 / R44-P1-25 的 record 物化切片。源码已接入，性能标准待实测。

## 实际修改

私有 LogRecord 原有独立 owned scope 和 message 两个 String，现以一份按总 UTF-8 字节数预留的 String 保存两段完整文本，同时保存 scope_len 字节边界。编码借用其两个切片，record 仍拥有所有文本，不借用 caller 的生命周期。

边界来自同一个有效 scope 字符串的长度，空 scope 或空 message 保持有效。两段均非空时，record 存储层减少一次独立 owned allocation 的机会；尚未测量总 allocation requests/bytes 或 CPU 收益。

公开 API/settings、lazy preflight、enqueue 时间戳在 owned 文本构造后的顺序、原 saturating 字节估算、小写 level Display、换行转义、FIFO、batch/flush/sync、错误统计与 shutdown/join 保持。

## 接入证据

- 两个实际文件：sink/worker.rs 和 sink/worker/batch_buffer_tests.rs，后一文件仅改私有 helper 的 record 构造。
- patch SHA-256：`5c2f167831cc9bab2add1d6f95a0af32aa37fb1af34e6fe47654397b0213e16a`；binding：`22b96ccbf0dc8d1f2b02f886249b132b181102dfb4ff0e6707185e297b186867`。
- 独审 `.codex/outbox/runtime44-owned-record-text-buffer-core-review-v1/review.json` SHA-256：`89946969080b1865c281d6c8f4b27f80661d7ce3aa66105bac285b50abce202b`，源码接受，native/performance pending。
- 实际回执 `E:/Git/ZirconEngine/.codex/outbox/runtime44-owned-record-text-buffer-v1/actual-canonical-apply.json` SHA-256：`46ed74cf5ab75e412edab481a7576ff36604e11186c3e2f8c5ecc03dd296f249`。17 个 raw 守卫含两前像与 15 个未受影响依赖；git apply check/apply、实际 Rust 2021 fmt 与 scoped diff 均 exit 0，共享 index/control 未变。
- 当前 worker LF 后像：`b5342110eb0a5e0b7ab76ccb89e61fa5f2aaaff2c17012bd0be1e58ccef70ff3`；测试文件：`d4cbce5acfcb6aeef9c5d2f31cdf8f7169b4685ca41e915b671bf0ef42103855`。

既有两项缓冲用例正文和[小写日志级别修复](2026-10-09-buffer-test-log-level-contract-correction.md)保持，没有新增测试。较早回执与后像保留为历史证据，本次当前文件以后像为准。

## 同批验证与开放范围

复用 Runtime #2 的 Unicode/多记录字节、batch 时间/字节/FIFO、durability、两项 buffer 与五项 worker 生命周期回归，以及 Release #9 原日志 producers。原 54 行风暴矩阵、20 条 critical admission 原始样本、p95 ≤ 50ms 等标准保持；15 项批次最终来源绑定与实际提交尚未完成。

原生成功、同来源同负载 before/after allocation/CPU 和 p50/p95/p99/max、真实 I/O/console、产品 F0、G35/G36 与 milestone 接受保持开放。完整 arena、静态 callsite、producer clock、record/queue 硬字节 admission 未由此修改完成。P1-12 / P1-24 / P1-25 / P1-50 不以源码减少 String 关闭。

详见[Runtime 完成列表](../../../astra/features/runtime/2026-10-09-runtime44-single-owned-record-text-completion-list.md)。
