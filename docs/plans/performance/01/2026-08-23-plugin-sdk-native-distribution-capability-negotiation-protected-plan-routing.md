---
source_report:
  - docs/plans/performance/01/2026-08-23-plugin-sdk-native-distribution-capability-negotiation-current-architecture-review.md
doc_type: protected-plan-routing
status: routing_pending
---

# Plugin SDK native distribution与能力协商受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：Plugin SDK current Rust **21/21**静态复审完成；current Cargo、native fixture、allocator、
  F0/F4 WPR/RSS/power、capability generation和test-support编译面仍open。本Session不直接编辑受保护ledger。
- PERF-MVP native entry M0：有约束entry的host ABI回调上界R+D -> 1；补跑Rust call-count/语义测试，并在Q=0/1/8/100/1k、
  G=0/1/100/1k、P=1/28/49/100、R=0/1/100记录ABI calls、CString alloc、grant scans与load wall。
- Plugins01 + Runtime06/10：下一host ABI hard-cut在host-owned load generation canonicalize capability slot/bitset，终态能力检查
  O(Q)或O(bitset words)；禁止跨DLL borrowed Rust set、process-global mutable session cache或v3 compat shim。
- Plugins12/13编译性能：将491行`test.rs`从37个生产runtime/editor feature站点拆为显式`test-support`；先记录clean/incremental
  wall、peak RSS、rmeta/codegen与binary map，再修改feature/dev-dependency，要求生产编译test helper=0、integration parity=100%。
- `dist.rs`三个宏保持static ABI生成；源码去重只能按编译时间/维护验收，不得计为产品runtime frame收益。
- `docs/plans/performance/review.md`：只有SDK current Cargo、native fixture、P/Q/G/R allocator+load receipt、test-support编译对照及
  F0/F4动态门通过后迁入。本轮不迁移、不提交milestone、不发送完成企微；非渲染切片不要求RenderDoc。
