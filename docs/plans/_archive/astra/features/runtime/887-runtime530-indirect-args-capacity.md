---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime530-indirect-args-capacity.md
related_records:
  - docs/plans/astra/features/runtime/883-runtime02-event-task-hotpaths.md
  - docs/plans/astra/features/editor/940-editor09-release-all-reservations-capacity.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_batcher.rs
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_batcher.rs
---

# Runtime887 Runtime530 Indirect-Args Capacity


| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Indirect argument staging | 支持 indirect draw 的路径按 command 数量为 `args_cpu` 预留精确上界；`batches` 保持需求增长，避免相邻命令合并场景的过量预留。 | 既有分组、fallback 和 per-draw 行为测试覆盖语义；source contract 要求 `Vec::with_capacity(commands.len())` 仅用于 `args_cpu`，并禁止为 `batches` 使用 command 上界。 |
| 性能门禁 | 32,768 帧、每帧 256 commands 的模型中，argument-vector growth events 从正数降至零。 | ignored marker `RUNTIME530_INDIRECT_ARGS_CAPACITY_BENCH_V1` 要求 Release 批量输出零优化 growth events；托管 Runtime02 编译和 Release 回执仍待定。 |


- 当前源代码已包含容量预留和 source/ignored evidence tests。
- 该记录只登记已有 Runtime02 优化，不修改 tooling；托管验证须与 Editor 波次批量执行。


### Current grouped validation candidate (2026-09-25)

The Runtime530 slice is included in the current grouped wave with Runtime
development PTY `27946` and Runtime02 Release PTY `88515`, submitted alongside
the Editor lanes. The wrappers remain intentionally unpolled; no compiler,
focused-test, or Release performance result is inferred from launch.
