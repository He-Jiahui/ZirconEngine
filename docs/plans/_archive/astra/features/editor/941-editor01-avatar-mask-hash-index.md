---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-26-avatar-mask-hash-index.md
related_records:
  - docs/plans/astra/features/editor/926-editor09-background-job-hotpaths.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/avatar/image/cache.rs
tests:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/avatar/image/cache/hash_index_tests.rs
---

# Editor941 Editor01 Avatar-Mask Hash Index


| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Stable avatar-mask lookup | 将 64-entry avatar-mask cache 的 ordered `BTreeMap` index 收敛为 `HashMap`，保留完整 resource/dimension/radius key、Arc 像素、LRU promotion、16 MiB budget 和 eviction 语义。 | 行为测试覆盖 Arc identity、hit promotion、duplicate replacement、capacity 和 LRU eviction；source contract 禁止恢复 `BTreeMap` lookup。 |
| 性能门禁 | 4,096 stable hits/frame 从 ordered traversal 转为 expected constant-time hash probes，像素无额外复制。 | ignored marker `EDITOR01_AVATAR_MASK_HASH_INDEX_BENCH_V1` 要求 paired Release P95 至少降低 30%；当前 Editor09 Release 批量会覆盖该 marker，托管回执仍待定。 |


- `cache.rs` 与 `hash_index_tests.rs` 当前 Rustfmt 检查通过。
- 当前 Editor09 Release grouped lane：PTY `99265`；未从启动推断 Cargo 或性能结果。
- 未修改 tooling；该记录登记已有 Runtime/Editor 优化源代码并保留 managed validation gate。
