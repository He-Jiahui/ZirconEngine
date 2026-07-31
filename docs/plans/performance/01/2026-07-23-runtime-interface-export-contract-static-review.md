---
related_code:
  - zircon_runtime_interface/src/export
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
reference_sources:
  - dev/godot/editor/export/editor_export_preset.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Cooker/CookGenerationHelper.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Cooker/CookSavePackage.cpp
tests:
  - zircon_runtime_interface/src/export/tests.rs
  - zircon_editor/src/core/export/tests.rs
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface export contract 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/export/**`当前 **6/6** 个受跟踪且clean的 Rust文件、**528** 行已逐文件阅读，并反查Editor core pipeline/preset store、Build/Export wizard消费者及`dev/` Godot/UE导出参考。本轮未修改Rust源码。

## 性能结论

- `load_export_preset()`先把整份文档解为`StrictPresetDocument { payload: Value }`，随后`load_versioned()`又通过多段borrowed probe重扫同一bytes/envelope/header并最终typed decode；generic current reader已不再物化第二个payload Value，但preset仍保留一个不必要的完整DOM并叠加多轮parse，精确补强 **PERF-MVP-570**。严格schema/version/unknown-field拒绝必须保留，但header probe不得拥有payload，current version应直接typed decode，只有旧版迁移才物化单一`Value`。
- `ExportStage::from_str()`为输入和8个固定stage name分别构造normalized `String`，单次最多9个小分配；它只在CLI/配置边界调用、规模固定，本批不把它误列为MVP热路径。若后续counter证明频繁，再改为无分配separator-insensitive比较。
- `ExportPipelineReport::record()`为线性find，但stage universe由`ExportStage::ALL`固定为8；Editor topological order同样最多8项。固定上限是正向基线，不需要为此引入第二份stage index。
- report/stage I/O只存artifact key/locator/digest，不携带artifact contents；resume仅复制最多8阶段的artifact refs与diagnostics。Editor15须保持这一窄合同，并为diagnostic/string/report bytes设硬限，不能把PERF-MVP-055/449的文件正文或pack bytes吸入report DTO。
- preset的plugin package去重使用借用key的`BTreeSet`，没有逐项String clone；它属于authoring/load边界。需要1/1k/100k package/customized-file规模计数，确认validation与serde的O(N log N)成本不落UI frame，并复用PERF-MVP-570输入预算。
- Godot `EditorExportPreset`集中持有selected/customized-file集合并在变更时更新；UE cooker显式保留iterative modified status，save package拆为async task。Zircon保持single preset owner、fingerprint resume与有界异步I/O，不复制参考引擎的全局对象模型。

## 动态验收

1. `.zpreset` 1 KiB/64 MiB、current/v0/future/unknown-field、depth 1/64/128/129：记录whole/envelope/header/payload JSON passes与bytes visited、DOM owners、payload/String copy、peak RSS与p95；current完整envelope≤1遍、payload typed≤1遍且payload `Value` owner=0。
2. plugin packages/customized files 1/1k/100k：记录BTree comparisons、alloc/bytes、validate/load p95；UI frame只提交intent，不执行大preset decode/validate。
3. report 1/8 stages、diagnostic 0/4 KiB/1 MiB：记录report bytes、clone bytes、read/write/fsync与resume result；stage count≤8，report硬有界且不包含artifact contents。
4. cold/resume/1% changed export记录stage prepare/execute、artifact read/write/hash与Skipped I/O；unchanged Skipped实际artifact read/write/copy=0，错误顺序与atomic report语义不变。
5. current-source interface/editor export合同、F4 Build/Export产品trace通过。

current-source Cargo、规模counter与F4 export产品trace未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。
