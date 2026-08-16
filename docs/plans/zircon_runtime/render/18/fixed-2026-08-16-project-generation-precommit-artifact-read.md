---
handoff_kind: fixed
status: fixed
created_at: 2026-08-16
resolved_at: 2026-08-16
summary_slug: project-generation-precommit-artifact-read
origin_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
fixing_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
origin_child_dir: docs/plans/zircon_runtime/render/18
fixing_child_dir: docs/plans/zircon_runtime/frameworks/01
failure_scope: cross_plan
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/shader_import_dependencies.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/targeted.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/artifact/cache_payload/model.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs
  - zircon_runtime/src/asset/tests/assets/artifact_store/binary_payloads.rs
  - zircon_runtime/src/asset/tests/project/binary_artifact_cache_assertions.rs
  - zircon_runtime/src/asset/tests/project/manager/restore_failure_migration.rs
  - zircon_runtime/tests/shader_import_dependency_contract.rs
  - zircon_runtime/src/core/resource/io/transaction/engine/tests.rs
  - zircon_runtime/src/core/resource/io/transaction/pathing.rs
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_hybrid_gi_runtime -LibTests -TestFilter render_framework_stats_expose_scene_representation_screen_probe_and_radiance_cache_counts
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_hybrid_gi_runtime -LibTests
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter collector_context_exposes_viewport_size_extract_and_prepared_sidebands
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -TestTarget shader_import_dependency_contract -TestFilter project_shader_dependency_index_merges_restored_consumers_with_new_providers
---


# Frameworks01: project generation reads transactional artifacts before commit

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 来源执行切片：Render18 Hybrid GI package regression closure for the App01 editor-host gate
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 交接原因：最低根因位于 Frameworks01 新增的 project-generation durable transaction
  prepare/commit boundary，不属于 Hybrid GI fixture 或 Runtime04 importer selection。

## 失败现象与复现证据

Frameworks01 hard cut 将新 artifact manifest 改为 `PreparedFileWrite`，直到
`PreparedFullProjectGeneration::commit` 才原子发布；但 dependency projection 仍调用
`ShaderImportDependencyIndex::from_artifacts` 从最终路径重读本轮所有新工件。新项目的第一个
artifact manifest 尚不存在，因此 Windows 返回 `Io(NotFound, os error 3)`。Hybrid GI 完整
测试由 201 passed / 7 failed 收敛到 206 passed / 2 failed 后，剩余两项都稳定复现该错误；
生产包构建已通过，说明失败是运行时 project open 行为而非编译漂移。

## 最低共享层根因

事务重构只迁移了 artifact manifest 的写路径，没有同步迁移依赖索引的数据来源。缓存恢复
记录拥有已提交 artifact，可以安全读盘；本轮成功导入的 payload 已在 prepare 阶段内存中，
必须直接参与索引投影，禁止在 commit 前读取最终路径。

## 架构修复验收

- 全量 generation 对缓存恢复记录读取已提交 artifact，对本轮导入记录消费内存 payload。
- 同代内存投影以 `ShaderAsset` 强类型迭代器为边界；全量 generation 只保留 shader payload，
  峰值内存不再随 texture、model、audio 或 terrain 等非 shader 解码负载累计。
- 恢复索引在任何 artifact read 前按 `AssetKind::Shader` 过滤，避免恢复验证完成后再次读取并
  解码全部非 shader artifact；缺失 Data artifact locator 的索引层回归锁定该读盘边界。
- 重启回归覆盖“已提交 consumer + 同代新 provider”：consumer artifact locator 保持不变，
  依赖索引仍在 commit 前解析到新 provider。
- commit 失败仍可回滚全部 manifest、sidecar 与 registry，不提前发布 artifact manifest。
- Hybrid GI 两项原始 framework-stat 复现通过，随后完整包测试通过。
- Frameworks01 全量 generation focused tests 在当前 lib-test 图可编译时通过；若被外部测试编译
  漂移阻断，保留精确编译证据且不得以此替代向上产品测试。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses,
  or call-site exceptions.
- 不恢复 prepare 阶段直接写最终 manifest，不让 Hybrid GI 预生成缓存或跳过真实 AssetManager。
- 不吞掉 `NotFound`，不降低 dependency projection 或 durable commit 验收。

## 修复结果与回传

- 根因：Precommit dependency projection reread unpublished artifacts; ModelAsset optional serde fields corrupted bincode shape; Windows verbatim prefix probing rejected valid paths.
- 架构修复：Filter restored records to shader kind before artifact I/O, merge those committed shaders with a shader-only same-generation in-memory projection, use an always-encoded model cache DTO under ZRARTM05 schema v5, and skip standalone verbatim prefix metadata probes.
- 验证：Managed HGI exact WGPU evidence 5/5; the post-review full 229-test registered plugin matrix passes 2/2 and its production build passes; post-review zircon_runtime production build passes; the exact restart regression and the complete two-case `shader_import_dependency_contract` integration target pass. The internal lib-test binary remains blocked by 326 unrelated existing test compile errors (first failures are missing test modules and unresolved text/render test imports, with no owned-path compile error).
- 回传：Frameworks01 project generation and model cache repaired; Render18 project-open and HGI gates pass.
