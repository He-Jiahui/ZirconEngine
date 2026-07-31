---
related_code:
  - zircon_runtime/src/bin
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
reference_sources:
  - dev/godot/core/io/pck_packer.cpp
  - dev/godot/editor/export/editor_export_platform.cpp
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ShaderCompiler/ShaderCompiler.cpp
tests:
  - zircon_runtime/src/bin/zircon_export_pack/manifest.rs::performance_contract_tests::included_assets_use_a_manifest_path_index
  - zircon_runtime/src/bin/zircon_shader_prewarm/run.rs::tests::exported_resource_records_move_into_the_overlay
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs::nested_resource_arrays_are_moved_out_of_the_json_document
  - current-source Windows Cargo and cold/warm tool-scale traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime bin逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/bin/**`当前源 **40/40** 个Rust文件、**6,959** 行、**64** 条测试已逐文件阅读。覆盖export validate/pack、font SDF bake、host reflection docs、shader IDE env、shader prewarm的CLI、manifest、resource/permutation registry、shader/material/include扫描及全部测试。工具均不属于逐帧运行时，但export/prewarm直接影响F0构建迭代、CI wall time与大项目峰值RSS。

## 关键瓶颈

- **PERF-MVP-448 / PERF-MVP-357/358 / Render08/Runtime04**：每asset root先导出resource records，再递归扫shader；material阶段重建完整AssetRegistryIndex并再次递归。include dependency按每source clone imports并重走DAG，material线性find并深clone完整shader source。variant正文/版本复制与串行validate/write继续归357。
- **PERF-MVP-449 / Editor15/Runtime04/11**：export pack串行把全部source读入Vec，writer前clone全部`ZrPackInputAsset`，determinism再次`to_vec`，delta又复制target/delta pack并在内存重建整包比较；规模由总asset bytes而非chunk budget决定峰值RSS。
- shader IDE env整项目open+scan/import复用PERF-MVP-075/088/358；font SDF all-cmap和per-glyph离线工作复用PERF-MVP-248/250；host reflection docs仅单次control-plane生成，没有独立热点。

## 本轮直接止损

1. export pack为manifest entries建立first-wins `HashMap<&str, &Entry>`，included asset source lookup从O(A²)收敛到摊销O(A)，保持duplicate manifest原先首项语义与fatal诊断。
2. shader prewarm导出的resource records直接move进overlay，删除一份完整`Vec<ResourceRecord>`及其String/locator clone。
3. resource registry从owned JSON object `remove` nested `resources`/`records`数组，删除整个records `serde_json::Value`深clone。

三项均先得到源码契约RED，再完成GREEN、`rustfmt --edition 2021`与scoped `git diff --check`。current-source Cargo需等待受管CPU槽，没有运行raw Cargo。

## 参考约束与动态验收

Godot export customization用mtime/MD5 cache跳过未变化资源，PCK writer按file payload写入后只保留目录元数据；说明Zircon的resume/content identity应在Cook→Pack边界生效，而不是每次把全项目bytes常驻多份。Bevy把pipeline放入Queued/Creating/Ready状态并用AsyncComputeTaskPool执行；Unreal使用受控local/distributed ShaderCompileWorkers、outstanding job与async result owner。对应Zircon方向是一遍inventory、content-addressed source/chunk table和bounded worker queue，不是让CLI同步多扫几遍。

动态验收需覆盖files/sources/materials/assets 1/100/10k、variants 1/1k/100k、WGSL 4KiB/1MiB、pack 1MiB/1GiB及dedup 0/50/99%，记录directory entries/open/read、DAG visits、WGSL/full-pack clone bytes、hash/compile/write、worker depth/age、peak RSS与cold/warm/resume p95。current-source Cargo和真实cold/warm export/prewarm trace完成前继续留在`pending.md`，不得进入`review.md`。
