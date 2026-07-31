---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: submit-context-camera-target-sharing-anchor-drift
origin_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
fixing_plan: docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
origin_child_dir: docs/plans/zircon_runtime/text/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots/submit_context/camera_loop_sharing.rs
---

# Runtime07：submit context camera target sharing 锚点漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 来源执行切片：Text01 current-source default/UI lib-test 的 Runtime07 submit-context guard
- 修复责任计划：`docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md`
- 交接原因：camera target 投影与 submit-context 共享语义由 Runtime07 所有，Text01 只记录误命中的 current-source 失败。

## 失败现象与复现证据

Text01 current-source default/UI lib-test job `f9f5581fb83b40c2a3cc81aa15f5bcaa`、run
`b98dc769094b4bd9b96fc445fd8a1332` 因 `text::` 子串命中 `context::` 而执行 Runtime07 守卫。
`runtime_07_submit_context_shares_large_extract_payloads` 在 `camera_loop_sharing.rs:75` 失败，当前
`camera_loop.rs` 缺少既定锚点：

```text
.map(|submission| submission.camera.target.clone())
```

job 于 `2026-07-22T19:24:42.482382+00:00` 自然结束并 release，exit `101`、live PIDs 为空；
完整批次为 `776 passed / 7 failed / 2 ignored / 8083 filtered`，原始日志位于
`.codex/state/session-coordinator/cargo-runs/f9f5581fb83b40c2a3cc81aa15f5bcaa/b98dc769094b4bd9b96fc445fd8a1332/`。

## 最低共享层根因

Runtime07 的生产锚点与“camera target 来自共享 submission source”的守卫语义发生漂移，Text01 的过滤误命中只是暴露该问题的执行入口，不是最低修复层。

## 架构修复验收

- Runtime07 核对 camera target 投影是否迁移为等价的 shared-source 语义；更新生产路径或守卫，但不得恢复整份 `RenderFrameExtract` clone。
- 运行 Runtime07 精确守卫、上行门禁、独立 review 与 managed commit；Text01 不修改 Runtime07 路径。

## 禁止临时方案

不得恢复整份 `RenderFrameExtract` clone、弱化或删除目标守卫，也不得添加只为匹配文本断言的兼容锚点。

## 修复结果与回传

Open state: `待 Runtime07 核对 shared-source 等价语义并取得精确守卫与上行门禁的有效动态证据`。
