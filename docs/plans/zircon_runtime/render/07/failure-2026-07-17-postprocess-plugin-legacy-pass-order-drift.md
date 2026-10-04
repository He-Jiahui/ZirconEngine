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
  - cargo +1.94.1 test -p zircon_runtime --lib rendering_plugin_product_defaults_exclude_unqualified_forward_plus_ssao --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib rendering_plugin_product_defaults_keep_deferred_ssao_disabled --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib rendering_plugin_default_features_preserve_motion_vector_and_bloom_composite_contract --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib rendering_plugin_post_process_routes_output_transfer_through_terminal_anti_alias_input --locked --jobs 1 -- --nocapture --test-threads=1
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

Open state: `current-source contract repair present; managed validation pending`.

- The default plugin feature regression now fixes the canonical forward-plus and deferred pass vectors, including Bloom before reflection/baked-lighting composites and the full three-stage motion-vector reduction chain.
- The no-stack plugin filtering path preserves that default post-process motion-vector chain, while stack-driven filtering still owns optional effect removal. This keeps descriptor, filtering, and behavior coverage on one canonical contract rather than replacing the expected vector with incidental output.
- The current focused contract tests are
  `rendering_plugin_product_defaults_exclude_unqualified_forward_plus_ssao`,
  `rendering_plugin_product_defaults_keep_deferred_ssao_disabled`,
  `rendering_plugin_default_features_preserve_motion_vector_and_bloom_composite_contract`,
  and `rendering_plugin_post_process_routes_output_transfer_through_terminal_anti_alias_input`.
  They retain the focused resource/order assertions in current source. No
  current-source Cargo result is claimed; the handoff remains `open` until the
  managed plugin gate returns.

### 2026-09-25 successor current-source reconciliation r2

Fixing Session `failure-roll-01a084c8-render07-postprocess-pass-order-r2`
reconciled the four current producer/test paths. The exact source hashes are:

```text
zircon_runtime/src/graphics/tests/pipeline_compile.rs
  821d3505f4b72ed45178404382ccded914b44877904d52124688fb5598002c3e
zircon_runtime/src/graphics/tests/pipeline_compile/plugin_features.rs
  93fcf47379106218e83c1dcfc15cbc359b8fa92c629742005e63d041b2356881
zircon_runtime/src/graphics/pipeline/render_pipeline_asset/descriptor_filtering.rs
  4c97e50b6734cebb89ad04560099a7ef4b19946c353115b8131df439f8d496a1
zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  1e0546103af9df33d4773f786b4745e65e6dac6d2577b688ee5e7765f461e714
```

The exact current-source probe passed:
`RENDER07_POSTPROCESS_CURRENT_SOURCE_PASS 4 paths; FILTERS=4`. It checked all
four current test names, canonical Bloom/motion-vector/DoF pass identifiers,
the no-stack default-chain preservation branch, scene-velocity resource wiring,
and stack-driven filtering helpers. Scoped `git diff --check` passed. Rustfmt
remains non-passing only for existing import/order and equivalent formatting
drift in the two Rust test owners and the post-process descriptor owner; no
formatter-only rewrite was made.

This is source/static evidence only. Managed exact focused tests plus the broad
`plugin` gate, external `E:/Git/zr_vm` admission, independent review, canonical
`failure return`, fixed status, closeout and WeCom remain pending.

### 2026-09-25 independent current-source review r2

Reviewer Session `review-render07-postprocess-pass-order-r2` rechecked
snapshot `3826` without editing or absorbing foreign changes. All four source
hashes match the manifest. The four exact test functions resolve in
`plugin_features.rs`; static inspection confirms canonical Bloom-before-
reflection/baked-lighting ordering, the three-stage motion-vector chain, DoF
identifiers, no-stack default-chain preservation, stack effect filtering, and
scene-velocity resource routing. Scoped `git diff --check` is clean; the
documented rustfmt drift remains limited to the two Rust test owners and the
post-process descriptor.

Independent result: **Critical=0 / Important=0 / Moderate=0**. No Cargo was
run and no dynamic gate is inferred. Snapshot `3826` predates this receipt's
doc-only append; managed focused tests, broad `plugin`, external
`E:/Git/zr_vm` admission, canonical return, closeout and WeCom remain pending.
