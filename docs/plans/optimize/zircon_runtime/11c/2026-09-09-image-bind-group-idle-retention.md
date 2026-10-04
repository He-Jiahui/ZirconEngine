---
title: Runtime11C UI Image Bind-Group Idle Retention
category: zircon_runtime
report_id: Runtime11C-ui-image-bind-group-idle-retention-2026-09-09
date: 2026-09-09
session_id: root-runtime-editor-optimize-20260909
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11C UI Image Bind-Group Idle Retention

## Scope

`ScreenSpaceUiImageBindingCache` now retains recent GPU bind-group products across a short empty
frame or missing-streamer gap. Entries remain eligible for at most two prepare epochs when idle,
while products still referenced by a retained image segment stay pinned. The nominal 512-entry
cap removes only unpinned non-current entries under pressure; pinned active products are never
dropped solely to satisfy the cap. Texture Arc identity remains
the replacement key; resource resolution and upload generations are unchanged.

The change restores the Runtime79/Editor01 cache contract without reintroducing the retired handle
wrapper: the segment owns an `Arc<ScreenSpaceUiImageBindingProduct>`, and the cache only owns the
bounded reusable index.

## Regression Evidence

- `screen_space_ui_image_binding_cache_keeps_recent_idle_epochs_bounded` covers the two-epoch
  window and wrap-safe conservative boundary.
- `screen_space_ui_image_binding_cache_never_trims_the_active_epoch` keeps the current prepare
  epoch out of idle trimming.
- `tools.tests.test_runtime_ui_gpu_image_cache_performance_contract` passed `10/10` after the
  current `PreparedScreenSpaceUi` and `binding_product_generation` owner names were synchronized.
- The merged Runtime/Editor performance-contract batch passed `1723/1723` (`Runtime 1143/1143`,
  `Editor 580/580`).
- The follow-up bounded Runtime/Editor source-contract batch covering retained image/text render
  dependencies and Editor12 lifecycle ownership passed `70/70`; this does not substitute for
  managed WGPU timing.
- `rustfmt --edition 2021 --check` passed for the image production/test owners; scoped
  `git diff --check` reported only the repository's existing line-ending notices.

Source fingerprints at record time: `image.rs`
`1C4927D5C00A37306154BDB73F76ADBFEC6F4FAC89DB7C611B687BC0C6E35068`, image tests
`7D257E32D78D8AF49BD7EB81DEF331F2F3B7A169A69972B167CD249CAB222C21`, and the Python contract
`41FF89C0BBFD8FD55C72501B636A2720F1F31BE158E4AA32CD6A1E6F1894E427`.

The existing deterministic image-prepare pressure model (4,096 frames, 1,024 batches per frame,
16,384 registry records, four management generations) reports registry visits falling from
`2,415,919,104` to `2,359,296` (1,024x) while retaining `4,194,304` texture-cache lookups. This
is an operation-count model, not a device timing or release percentile.

## Managed Gate

No Cargo, WGPU device, bind-group creation, CPU/RSS, or release percentile run was started in this
slice. The ignored Windows release benchmark remains the owner of exact P50/P95/P99 evidence and
the final performance threshold.
