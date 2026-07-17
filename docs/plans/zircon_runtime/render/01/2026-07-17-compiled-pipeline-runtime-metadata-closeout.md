---
record_kind: milestone_output
status: completed
completed_at: 2026-07-17
plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
milestone: RG-M3
slice: compiled-pipeline-frame-derived-metadata-hard-cut
source_manifest_fingerprint: 786c6aed151cbea8063c76ded9ede7eb0b7c297467a828fe12179d46c24c5d34
related_code:
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline/runtime_metadata.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline/runtime_feature_flags.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline/resource_write_index.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
tests:
  - render01_compiled_pipeline_runtime_metadata_builds_resource_write_index_once_for_scaled_graphs
  - render01_compiled_pipeline_runtime_metadata_freezes_descriptor_capability_flags
  - render01_compiled_pipeline_executor_validation_cache_skips_stable_10_100_500_pass_rescans
  - render01_compiled_pipeline_executor_revoke_invalidates_cache_before_submission
  - render01_compiled_pipeline_cached_sources_are_immutable_and_frame_flags_are_precomputed
  - plugin_feature_buffer_minimum_size_survives_graph_resource_planning
---

# Compiled pipeline runtime metadata hard cut closeout

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| RG-M3 performance slice | compiled pipeline 派生 metadata、resource write index、executor validation generation cache | `completed` | 2026-07-17 | source fingerprint `786c6aed...`; managed jobs `9cfa78c7...` 与 `80390edb...`; failure return receipt [`2026-07-17-compiled-pipeline-frame-derived-recomputation-return.md`](2026-07-17-compiled-pipeline-frame-derived-recomputation-return.md) |
| M3 | M3-T compiled-pipeline-runtime-metadata | 通过 | 2026-07-17 | `5 passed / 0 failed` 加 exact buffer-planning `1 passed / 0 failed`；两条 job 均 exit 0、released/no PID |

## 完成项目

- `CompiledRenderPipeline::from_parts` 成为 crate 内唯一 metadata 构造边界；`graph`、`enabled_features` 与 runtime metadata 不再允许消费者替换。
- compile miss 时一次性冻结 runtime feature flags、named-resource hash index 与 write bitset；scene/frame submission 路径只读取不可变派生状态。
- `RenderPassExecutorRegistry` 维护 registry generation；register、replace、unregister 都会失效成功验证缓存。
- 成功验证缓存严格绑定 `{pipeline metadata generation, registry generation}`；失败结果不缓存，revoke 后下一次 submit 在 GPU 编码前恢复 typed missing-executor failure。
- 所有 hard-cut consumers 与插件 fixture 已迁移到只读 getter；测试 fixture 的 async-compute pass 具备 mandatory fixed workload，不再在目标 buffer-size 断言前失败。
- 模块文档已记录构造边界、O(1) frame-path 查询、cache invalidation 与禁止恢复逐帧扫描的约束。

## 性能与正确性不变量

- 10/100/500 pass 稳态重复验证只执行一次 full scan；128 次重复调用不增加 scan count。
- hot reload/replacement 推进 registry generation 并触发一次重新验证。
- executor revoke 后缓存不能接受 stale executor；失败验证不得污染后续缓存。
- resource-write 查询不分配临时 `String`，不遍历 pass/resource access，也不改变冻结存储指针或容量。
- compiled pipeline 的公开 `PartialEq` 不包含进程内 validation identity，避免 generation 破坏结构等价语义。

## 受管验证

- Job `9cfa78c793b94b2eb4eff253c14b3650` / run `3a001e665bca4528bfd1564470fb4241`：
  `cargo test -p zircon_runtime --lib --locked --jobs 1 --color never render01_compiled_pipeline_ -- --nocapture --test-threads=1`；`5 passed / 0 failed / 8236 filtered`，exit 0，released/no PID。
- Job `80390edb8b1c4060ab639b3751acbc6d` / run `c08a17d5d5f44ce09811cf3965f368f0`：
  exact `plugin_feature_buffer_minimum_size_survives_graph_resource_planning`；`1 passed / 0 failed / 8240 filtered`，exit 0，released/no PID。
- 两条命令均绑定 canonical Rust 1.94.1、同一 68-path validation source fingerprint `786c6aed151cbea8063c76ded9ede7eb0b7c297467a828fe12179d46c24c5d34`。其中 4 条 F2 product source 仅参与 current-source 编译绑定，不属于本里程碑提交清单。
- `rustfmt --check` 覆盖 68/68 Rust 路径；scoped `git diff --check` 通过；handoff validator 为 `238 artifacts / 0 errors`。

## Failure 回传

- lifecycle `compiled-pipeline-frame-derived-recomputation` 已由 coordinator 原子返回到 Performance child：
  `docs/plans/performance/01/fixed-2026-07-17-compiled-pipeline-frame-derived-recomputation.md`。
- fixing child 只保留 coordinator 生成的 return receipt，不保留重复 canonical artifact。

## 后续范围

- 本记录只关闭 RG-M3 的 compiled-pipeline frame-derived metadata performance slice，不宣称完整 RG-M3 pass culling/cache 或 RG-M4 RenderDoc 诊断完成。
- MVP F2 persisted scene + input + WGPU + deterministic teardown 的产品测试与 PNG 仍由独立 F2 切片继续，且必须等待 Plugins02 Sound current-source closeout。
