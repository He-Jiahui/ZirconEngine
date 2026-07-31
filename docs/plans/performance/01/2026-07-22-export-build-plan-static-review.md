---
related_code:
  - zircon_runtime/src/plugin/export_build_plan
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_dependencies.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/constructors.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_plugins/09-export-publishing.md
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/godot/editor/export/editor_export_platform.cpp
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/AutomationUtils/CommandUtils.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Android/AndroidPlatform.Automation.cs
tests:
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs::tests::completed_plugin_manifest_is_reused_for_feature_resolution
  - zircon_runtime/src/plugin/export_build_plan/export_build_plan.rs::tests::fatal_presence_check_does_not_materialize_diagnostics
  - zircon_runtime/src/plugin/export_build_plan/materialize/package_lookup.rs::tests::inventory_stops_after_all_nested_selections_resolve
  - zircon_runtime/src/plugin/export_build_plan/materialize/archive.rs::tests::archive_materialization_does_not_preview_then_rescan_each_package
  - zircon_runtime/src/plugin/export_build_plan/materialize/native.rs::tests::native_materialization_indexes_package_export_rows_once
  - zircon_runtime/src/plugin/export_build_plan/materialize/generated.rs::tests::generated_file_writer_reuses_parent_directory_checks
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files.rs::tests::runtime_library_template_does_not_rescan_the_completed_source
  - current-source Windows Cargo and export product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Export build plan逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/plugin/export_build_plan/**`当前 **40/40** 个Rust文件、约5.3k物理行、21个就地tests已逐文件阅读，覆盖profile/manifest投影、generated host/templates、NativeDynamic plan、materialize/preview/ZIP、路径与报告。六个`zircon_runtime/src/tests/plugin_extensions/export_build_plan*.rs`外部测试尚未逐文件验收，继续留在`pending.md`；本记录不把源码门禁或历史39文件审查冒充动态验收。

受管Cargo申请仍被Session `frameworks03-export-profile-explicit-hardcut-r3-20260722`的reservation `a20ce6ab9092442988be11ae77664269`占用，没有运行raw Cargo。该目录本身不执行GPU命令；generated browser/mobile host尚未形成可运行实时渲染session，因而没有伪造RenderDoc capture。

## 已确认的性能形状

- `from_project_manifest`已经有顺序保持的profile/manifest validation projection，但过去仍从`RuntimePluginCatalog::builtin()`深clone全部registration/feature report行，并在插件清单补全后让feature dependency API再次补全整份manifest。PERF-MVP-546改为借用进程级cached builtin generation并复用已补全manifest；完整`CompiledProjectPluginPlan`缓存仍归PERF-MVP-538。
- `has_fatal_diagnostics`过去只为回答bool就clone/format整份effective fatal列表。PERF-MVP-546改为直接检查两个authoritative非空条件；需要文本的materialize/report路径仍显式生成列表。
- `NativePackageInventory`已把旧的per-package整树扫描收敛为单次inventory，但未找到的selection会迫使它继续遍历已解析package的assets/resources payload。PERF-MVP-547增加resolved-root剪枝和“全部selection已解析”早停；ZIP materialize也不再先preview再重新枚举同一package文件。
- native/ZIP materialize过去每个package线性find export row并深clone包含九个ABI String的完整plan；PERF-MVP-547改为单次borrowed index和仅missing fallback拥有临时row。generated file writer复用父目录创建结果，portable path normalization只保留borrowed components。
- generated runtime library先构造完整Rust源码，再用`.replace`全串扫描插入一个ABI函数；PERF-MVP-548改为直接append，删除无意义第二遍文本扫描。
- `ExportValidateReport`仍深clone全部generated contents并再次JSON序列化；validation projection仍拥有多份String索引；materialize/preview/archive仍各自建立inventory，unchanged file仍串行覆盖。这些继续由PERF-MVP-051/054/055和既有Plugins09 failure收口，不能因为局部止损而关闭。
- generated WebGPU/WASM对每个`pointermove`立即JS→WASM，Android/iOS move逐pointer/touch同步跨ABI；resize/viewport metrics也无帧内合并。PERF-MVP-052与`export-host-high-frequency-input-dispatch`保持open。更严重的是generated ABI仍丢弃runtime owner并让callback空转，`woc-mobile-browser-host-noop`保持open。

## 参考引擎核对

- Godot `editor_export_platform.cpp:1319-1744`先建立唯一path set，再由一个`export_project_files` traversal驱动save callback；其`FileExportCache`以mtime fast path、MD5 fallback和saved path复用customized artifact（`1013-1056`）。Zircon的目标不是复制Godot格式，而是让单generation inventory同时服务preview/materialize/archive并以可信digest跳过unchanged写。
- Unreal AutomationTool `CommandUtils.cs:1826-1840`先冻结source/destination pairs再用有上限的`Parallel.ForEach`复制；Android deploy又在`AndroidPlatform.Automation.cs:3829-3844`明确限制并发以避免内存过载。Zircon应采用Runtime11统一I/O预算和确定性report commit，不能照搬默认64线程或无界spawn。

## 本轮直接止损

1. **PERF-MVP-546**：borrow cached builtin catalog；已补全manifest直接进入feature resolution；fatal bool检查零诊断物化。
2. **PERF-MVP-547**：package inventory早停/剪枝；ZIP单次file inventory；native export row一次索引且常见路径零ABI row clone；generated parent mkdir去重与path component单owner。
3. **PERF-MVP-548**：generated runtime library删除完整源码`.replace`扫描，直接append固定ABI函数。

八项源码门禁、行为测试代码、scoped `rustfmt --edition 2021 --check`与`git diff --check`通过。新增nested inventory行为测试以字典序较晚的非法manifest作为poison，要求选中包全部解析后不再触碰余树；current-source Cargo尚未取得执行lane，因此不写成通过。

## 动态验收

需要覆盖packages/features/files **1/100/1k/10k**、payload **1KiB/1MiB/1GiB**、cold/unchanged/1% change、materialize/preview/ZIP三消费者。记录catalog/manifest projection builds与clone bytes、tree enumerate/stat/manifest parse、file inventory passes、mkdir/write/copy bytes、worker/queue age、peak RSS和main-thread p95；并完成generated desktop/browser/mobile产品启动、输入burst、稳定首帧、确定性archive与失败取消。Cargo、产品trace和真实可达GPU capture完成前，本目录不得进入`review.md`。
