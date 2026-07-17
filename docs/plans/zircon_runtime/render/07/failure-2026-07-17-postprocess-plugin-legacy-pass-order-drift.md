---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: postprocess-plugin-legacy-pass-order-drift
origin_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
fixing_plan: docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
origin_child_dir: docs/plans/zircon_runtime/render/01
fixing_child_dir: docs/plans/zircon_runtime/render/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/plugin_features.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/descriptor_filtering.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib rendering_plugin_default_features_restore_legacy_ --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib plugin --locked --jobs 1 -- --test-threads=1
---

# Render07：Postprocess plugin legacy pass order drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 来源执行切片：compiled-pipeline frame-derived recomputation testing stage
- 修复责任计划：`docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md`
- 交接原因：失败发生在 externalized postprocess feature descriptor 与 current postprocess filtering/graph ordering 的契约边界；Render01 本轮只把 compiled graph 改为只读 getter，并未改变 pass filtering 或 ordering。

## 失败现象与复现证据

Frameworks02 source-bound broad job `42d5707f448d4589ae3ac390c0a19a1c` / run `c91570084d00499889ad5315db5a5d77` 执行 `cargo test -p zircon_runtime --lib --locked --jobs 1 --color never plugin -- --test-threads=1`，终态为 `820 passed / 18 failed / 2 ignored / 7401 filtered`、exit 101。以下两项是本交接的原始失败：

- `rendering_plugin_default_features_restore_legacy_forward_plus_pass_order`
- `rendering_plugin_default_features_restore_legacy_deferred_pass_order`

两种 pipeline 的 current graph 都把 `bloom-extract` 排在 `depth-of-field-prepare` 之后，并省略 `motion-vector-tile-max`，而 legacy fixture 期望 Bloom 位于 reflection/baked-lighting composite 之前并保留完整 three-pass motion-vector reduction chain。

`git diff` 证明 Render01 对这两个测试只执行 `.graph` 到 `.graph()` 的访问器迁移，未改 descriptor、filtering、pass dependencies 或 expected vector。因此该 RED 不能归因于 compiled-pipeline metadata hard cut，也不能通过回退 getter 解决。

## 最低共享层根因

当前已证明的最低边界是 `pipeline_compile.rs::rendering_post_process_descriptor` 与 production `builtin_render_feature_descriptor/.../post_process.rs`、`descriptor_filtering.rs` 的语义漂移：测试 fixture 仍描述旧的 pluginized postprocess pass/resource topology，而 current filtering 根据 `PostProcessStackDescriptor` 裁剪资源和 motion-vector passes。最终修复必须先确定 externalized plugin descriptor 是否仍承诺恢复 builtin default graph，再让 fixture、filtering 和 pass dependencies共享一个 canonical contract。

## 架构修复验收

- 为 externalized postprocess descriptor 增加 focused contract，证明 default forward-plus/deferred 的 canonical pass set、resource dependencies 与 ordering；测试必须先对 current drift RED，再由最低层修复转 GREEN。
- 两个原始 `rendering_plugin_default_features_restore_legacy_*` 测试在 canonical Rust 1.94.1 下执行并通过。
- 重跑 `plugin` broad filter；本交接的两个失败消失，其余外部 owner 失败单独报告，不得混称 Render07 GREEN。
- 若 legacy restoration 不再是产品契约，必须在 Render07 计划与模块文档中明确新的 canonical plugin filtering contract，并用行为测试证明，而不是只改 expected vector。

## 禁止临时方案

- 不得仅按本次 actual pass vector 修改断言来隐藏 descriptor/filtering 漂移。
- 不得添加 alias、compatibility shim、silent fallback、duplicated truth、test-only bypass 或单调用点例外。
- 不得删除 `motion-vector-tile-max`、Bloom 或 resource-dependency 验证来缩小失败面。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
