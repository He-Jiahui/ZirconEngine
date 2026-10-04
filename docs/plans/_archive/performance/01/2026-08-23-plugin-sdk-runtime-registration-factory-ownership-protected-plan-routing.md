---
source_report:
  - docs/plans/performance/01/2026-08-23-plugin-sdk-runtime-registration-factory-ownership-current-architecture-review.md
doc_type: protected-plan-routing
status: routing_pending
---

# Plugin SDK runtime registration factory ownership受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：SDK registration/prelude current 2/2完成，SDK累计17/21；current Cargo、allocator counter、
  runtime registry全owner复审以及F0/F4动态trace仍open。本Session不直接编辑受保护ledger。
- PERF-MVP runtime registration M0：SDK builder保留具体`F`到runtime registry边界；源码门为SDK factory Arc allocation/system=0、
  extra SDK dynamic dispatch/instance build=0，语义门为private callback state与ordering/clock/revoke parity=100%。
- Plan02 + Plugins01/12后续：完整复审`RuntimeExtensionRegistry`的factory/build两层擦除、registration clone、owner revoke和scene
  scheduler消费，再决定是否合并为单个erased build closure；禁止引入compat shim或跨session全局factory实例。
- 动态矩阵：S=0/1/100/1k、I=1/2/100、C=0/1/8/100、R=0/1/100；受管release记录alloc count/bytes、dispatch counter、
  registration/build p50/p95、RSS与revoke残留。源码静态差值不能冒充allocator或frame数据。
- `docs/plans/performance/review.md`：只有SDK 21/21、current Cargo、allocator receipt、runtime registry结构门及F0/F4 WPR/RSS/power
  通过后迁入。本轮不迁移、不提交milestone、不发送完成企微；非渲染切片不要求RenderDoc。
