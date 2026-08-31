---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: ssr-history-resource-descriptor-drift
origin_plan: docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
fixing_plan: docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/02
fixing_child_dir: docs/plans/zircon_runtime/render/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/tests/pipeline_compile/feature_descriptors.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/core/framework/render/post_process/color_space.rs
  - docs/plans/zircon_runtime/render/06-temporal-pipeline.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
tests:
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --color never graphics::tests::pipeline_compile::feature_descriptors::compiled_pipeline_resources_use_extract_viewport_hdr_and_msaa_descriptors -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --color never descriptor -- --test-threads=1
---

# Render07：SSR history resource descriptor contract drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md`
- 来源执行切片：M3 RuntimePlugin lifecycle `descriptor` focused gate
- 修复责任计划：`docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md`
- 交接原因：失败发生在 post-process graph resource descriptor 语义，不在 RuntimePlugin descriptor、registration report 或 lifecycle hard cut 范围内。

## 失败现象与复现证据

Windows managed job `386e3d872b224189b488a0d37e564f34` / run `53f2e2c699974d9cb51bcbb9b3b040e5` 执行：

```text
cargo test -p zircon_runtime --lib --locked --jobs 1 --color never descriptor -- --test-threads=1
```

结果为 242 passed / 3 failed / 7996 filtered out，exit 101。其中本交接只覆盖：

```text
graphics::tests::pipeline_compile::feature_descriptors::compiled_pipeline_resources_use_extract_viewport_hdr_and_msaa_descriptors
```

断言在 `feature_descriptors.rs:362` 的 post-process resource tuple loop 失败。该测试先验证 `scene-color` / `scene-depth` 继承 1280x720、HDR 与 MSAA=4，再要求列出的 post-process products 为单采样且匹配各自尺寸/格式。

当前源码静态对照显示：loop 第一项 `SCENE_VELOCITY` 的期望与 production 都是 `Rg16Float`；第二项 `SCREEN_SPACE_REFLECTION_HISTORY` 的测试期望仍为 `Rgba8UnormSrgb`，而 `resource_descriptors.rs` 将它归入 `post_process_high_quality_hdr_format()`，当前 canonical constant 为 `Rgba16Float`。因此当前最强根因推断是 SSR history 的 legacy LDR fixture 与 HDR history descriptor 已漂移；owner 的 focused reproduction 必须让失败消息携带 resource name 或逐项断言，以最终确认而不是依赖本次短路位置猜测。

本次 reservation 的 41-path source manifest 只冻结 Frameworks02 RuntimePlugin owner/test 文件；它证明 Frameworks02 hard-cut 源未漂移，但不替代 Render07 对上述 render files 自己生成 current-source manifest 和 fresh reproduction。

## 最低共享层根因

`SCREEN_SPACE_REFLECTION_HISTORY` 的图资源格式只应有一份 canonical 契约，且要与 SSR history 的跨帧 copy/bind、HDR scene-color 语义和 test fixture 同步。目前 production descriptor 与 aggregate test fixture 不是同一事实源，宽 `descriptor` filter 因此被非 RuntimePlugin 的旧期望阻断。

## 架构修复验收

- 先用 focused test 精确报告失败的 resource name、actual `TextureDesc` 与 expected contract；不得仅凭 aggregate `matches!` 短路更新断言。
- 若 SSR history canonical contract 为 HDR，测试、Render07/Render06 文档及 history copy/bind 行为统一到明确 HDR format；若应为 LDR，则修正 production descriptor 并证明没有丢失 HDR scene-color 信息。
- focused `compiled_pipeline_resources_use_extract_viewport_hdr_and_msaa_descriptors` 通过。
- Frameworks02 重跑完整 `descriptor` filter 时本项消失；另两项 Runtime15 失败独立处理，不得混称全门 GREEN。

## 禁止临时方案

- 不得删除 SSR history tuple、放宽为任意格式、忽略 sample count，或只把 expected 改成当前 actual 而没有 canonical contract 证据。
- 不得恢复直接字段访问、兼容 getter、legacy resource alias、双描述符或 test-only bypass。

## 修复结果与回传

Open state: `current-source descriptor contract repair present; managed validation pending`.

- `SCREEN_SPACE_REFLECTION_HISTORY` is a single-sample HDR intermediate with canonical `Rgba16Float` production descriptor, and the focused pipeline descriptor fixture asserts the same format.
- The fixture reports the resource name, expected dimensions/format/sample count, and actual descriptor on mismatch, so a later resource cannot be mistaken for the original aggregate-loop failure.
- No current-source `descriptor` Cargo gate is claimed. The handoff remains `open` until managed validation confirms this failure is absent from the broader filter.
- 2026-08-30 source follow-up: the same canonical `Rgba16Float` View-sized contract now also governs `HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION`. The resolve pass declares a fragment sampled full-texture external access, the history binder publishes the physical texture/view/descriptor, and the executor resolves that access through the compiled pass instead of reading `SceneFrameHistoryTextures` directly.
- Static rustfmt, locked metadata, source-contract, and scoped diff checks passed. The handoff remains `open`: no managed descriptor test, WGPU frame, consecutive-frame PNG, or RenderDoc capture has confirmed the repair dynamically.
