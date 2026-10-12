# Runtime44 · Worker 批量编码缓冲复用

日期：2026-10-09。对应[原计划](../44-process-diagnostic-log-router-filter-record-queue-sink-durability-rotation-crash-multi-session-product-integration-review.md) R44-P1-25 的缓冲复用切片，承接[直接编码记录](2026-10-09-direct-batch-log-encoding.md)。实际源码已接入，性能标准仍待测量。

## 实际代码

Worker 现在持有一份最终编码 Vec，原七处 flush 触发均传入同一缓冲。每次非空批次先 clear，再以 saturating fold 估算所需字节，借用原直接编码函数写入。Console/file write、flush、sync、FIFO、触发阈值、单批时间戳和 metrics 顺序保留。

每次非空批次完成输出和统计后再次 clear。若 Vec capacity 超过 `min(normalized max_batch_bytes, DEFAULT_DIAGNOSTIC_LOG_BATCH_BYTES)`，立即释放编码 Vec；否则保留供下批复用。使用既有默认 256KiB 作为保留上限，没有新配置或预算值。超大已接收记录仍完整输出，不截断；成功与失败输出均释放超过上限的编码存储。

这是每次非空 flush 后的 Vec 逻辑保留容量限制。超大记录编码期间的瞬时峰值、owned record/queue 字节、allocator 物理开销仍没有完整硬上限，不把 capacity 宣称为 RSS 资格。producer 时间戳、arena、静态 callsite、完整字节 admission 和 sink 隔离仍属原计划开放范围。

## 实际证据

- 候选 patch SHA-256：`0d3a0d57128b5853557f3b2bf30fbf81b978723e6697ca47c4d50524d9ca3c31`。
- 独审 `runtime44-bounded-batch-buffer-reuse-core-review-v1/review.json` SHA-256：`dc0abbe45956ed20ca4fb9e4b490378b6c930fbc2e607e9821ca461dc396d025`，源码接受，零 P1/P2，native/performance pending。
- 实际接入 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/runtime44-bounded-batch-buffer-reuse-v1/actual-canonical-apply.json` SHA-256：`7b08234834b84802f8e0b56d04f0a5e906d0c38508e2cc4e85d01ea13b3d26d8`。一生产文件及一新回归文件；前像、新路径缺失及 16 依赖匹配，实际 LF 后像一致；apply check/apply、实际 Rust 2021 fmt 和 scoped diff exit 0，共享 index/control 未变。

新增两项回归源码：独立批次的精确字节、Unicode/LF、复用指针/容量、空 flush 无重复与 delivery metrics；超大记录完整输出、后续批次可用、失败输出不计 written，以及成功/失败都释放超限容量。回归源码就绪，尚未 native 执行。

## 同批验证与剩余门

两项精确新例及五项既有 retry/concurrent join/失败输出/output drop/TLS destruction 用例追加进原 Runtime #2，用于覆盖同一 retained worker buffer 的关闭和卸载门。原六项 batching/durability 与 FIFO 已在该 arm，不重复计数。原最低 11 保守加七个 unique cases 为 18，不增加 Cargo 配置。

任务草案 `batch-native-v4/commands-draft-v5.json` SHA-256：`b9a12e8e64ae7cb26f52cda3265e8d9231222caaab33acf698669a8dd37737c1`，仍为 15 项，source-bound/提交/执行均待完成。原 13 个 Release producer 与 critical companion 内容未变，54 行矩阵及 20 admission 样本/原阈值保留。

需要真实 native 通过、同来源同负载 allocation requests/bytes、worker/producer CPU、完整 p50/p95/p99/max 与真实 I/O 矩阵。P1-12 / P1-24 / P1-25 / P1-50、G35 / G36 和产品门保持开放，不能以复用源码及指针断言声称性能达标。

详见[Runtime 完成列表](../../../astra/features/runtime/2026-10-09-runtime44-bounded-batch-buffer-completion-list.md)。
