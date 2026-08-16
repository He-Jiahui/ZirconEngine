---
record_kind: failure_return_status
status: fixed
resolved_at: 2026-08-16
summary_slug: hgi-m5-scene-prepare-neutral-sideband
origin_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
fixing_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
plan_link_mode: child_record_only
source_artifact: docs/plans/zircon_runtime/render/18/failure-2026-08-11-hgi-m5-scene-prepare-neutral-sideband.md
---

# hgi-m5-scene-prepare-neutral-sideband 回传摘要

- 状态：`fixed`
- 回传工件：[fixed-2026-08-16-hgi-m5-scene-prepare-neutral-sideband.md](fixed-2026-08-16-hgi-m5-scene-prepare-neutral-sideband.md)
- 摘要：neutral scene-prepare sideband 已在唯一 WGPU prepare 路径闭环；完整 Hybrid GI
  lib-test 为 `209 passed / 20 ignored / 0 failed`，三个 DX12 exporter、三张 PNG 与同源
  RenderDoc replay 通过。300 帧稳定期后的 31 个热样本达到 cache hit、零 scene sync、零
  transient allocation/upload 各 `31/31`，HGI CPU prepare p50/p95/max 为
  `1,431 / 1,914 / 1,988 us`。
