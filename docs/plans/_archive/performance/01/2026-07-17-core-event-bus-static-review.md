# 2026-07-17 core EventBus 静态审查

## 范围与状态

- 已逐文件读取 `core/runtime/events.rs` 与 `events/{publish,subscribe,prune,failure}.rs` 共 5 个生产 Rust 文件。
- 静态审查完成；fanout、慢消费者、并发 publisher 与产品 trace 尚未完成，目录仍在 `pending.md`。
- 对照 Bevy `dev/bevy/crates/bevy_ecs/src/message/messages.rs` 的双缓冲更新/清理语义。Bevy 的 frame message owner 明确要求周期性 update，避免消息无限保留；Zircon 的 cross-thread EventBus 需要自己的 delivery policy，不能直接照搬。

## 已确认事实

### 已有的低成本路径

- 没有订阅者时 `publish` 在获取 delivery lock 前返回。
- 单订阅者直接 move `EngineEvent`，不 clone payload。
- subscriber 列表用 `Arc<[Sender]>` snapshot；发布不长时间持有 subscriber-map mutex，订阅/断连才复制 sender 列表。
- 1–5 subscriber 的 publish/prune 路径被专门展开，减少常见小 fanout 的临时 `Vec` 分配。

这些优化不能抵消下面的有界性与复制问题。

### 慢消费者队列无界

每次 `subscribe` 都创建 `crossbeam_channel::unbounded()`，EventBus 没有 queue depth、drop count、age 或消费 deadline。任一未 drain 的 receiver 都能让长期 runtime/editor/plugin 会话持续保留完整 `EngineEvent`。

这与 input history 的无界风险相同，但责任在共享事件 delivery owner。修复前必须按事件类别定义 lossless、bounded/drop-oldest、latest-value/coalesced 或 backpressure，而不是统一静默丢弃。

### fanout 深拷贝 payload

`EngineEvent` 是 `{ topic: String, payload: serde_json::Value }`。除最后一个订阅者外，publisher 对每个 subscriber 调用 `event.clone()`，会深拷贝 topic 与 JSON tree；成本约随 subscriber 数与 payload 大小相乘。单条大诊断/资产/editor 消息的 fanout 可以在主线程造成突发分配。

### 全 topic 发布串行

publisher 在 snapshot 后获取一个全局 `delivery_lock`，并在所有 channel sends 与失败订阅者 prune 期间持有。该锁保证某种全局 publish 顺序，但也让无关 topic 和多 producer 完全串行。当前没有 lock wait、publish duration、fanout 或 queue-depth 诊断，无法判断主线程/后台 worker 的竞争程度。

## 验收计划

1. 建立 1/2/5/100 subscriber × 0/1 KiB/1 MiB payload 的 clone/allocation/latency 基线。
2. 运行 1/4/16 publisher 与一个暂停 consumer，记录 lock wait、queue depth、RSS 和 event age。
3. Runtime02/07 先冻结 topic ordering 与 delivery-class 契约，再选择 shared `Arc<EngineEvent>`、bounded/coalesced channel 或 per-topic ordering owner。
4. 验证断连 prune、订阅 snapshot、顺序和 shutdown 不回归；诊断关闭时的开销也必须测量。

