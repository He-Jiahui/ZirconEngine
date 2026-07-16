---
related_code:
  - zircon_runtime_interface/src/export/mod.rs
  - zircon_runtime_interface/src/export/artifact.rs
  - zircon_runtime_interface/src/export/report.rs
  - zircon_runtime_interface/src/export/stage.rs
  - zircon_editor/src/core/export/mod.rs
  - zircon_editor/src/core/export/pipeline.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/progress.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller.rs
  - tools/zircon_build.py
  - tools/zircon_export
  - zircon_runtime/src/asset/pack
  - zircon_runtime/src/asset/virtual_geometry_cook
  - zircon_runtime/src/plugin/export_build_plan/mod.rs
  - zircon_runtime/src/core/framework/project/export_profile.rs
reference_sources:
  - dev/godot/editor/export/editor_export_plugin.h
  - dev/godot/editor/export/editor_export_platform.h
  - dev/godot/editor/export/editor_export_preset.h
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Scripts/BuildCookRun.Automation.cs
plan_sources:
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/zircon_plugins/09-export-publishing.md
  - docs/cli-and-tooling/zircon-export-tool.md
status: in_progress
---

# 15 发行与生成（Build / Export / Publishing）

## 参照证据（dev/）

**godot 导出三件套**（`dev/godot/editor/export/`）：

```cpp
// editor_export_preset.h:65-72 —— preset 是工程内持久化配置
ExportFilter export_filter; String include_filter, exclude_filter;
String export_path; HashMap<String, FileExportMode> customized_files;
// editor_export_platform.h —— 平台抽象承担流程
virtual Error export_project(const Ref<EditorExportPreset>&, bool p_debug, const String& p_path, BitField<DebugFlags>) = 0;  // :356
Error export_project_files(...);  Error save_pack(...); Error save_zip(...);
Error export_pack_patch(...); Error export_zip_patch(...);   // :323-360，patch 也是一等输出
// editor_export_plugin.h:98-116 —— 插件挂钩逐文件
virtual void _export_begin(const HashSet<String>& features, bool debug, const String& path, int flags);
virtual void _export_file(const String& path, const String& type, const HashSet<String>& features);
virtual void _export_end();
void add_file(path, data, remap);  void add_shared_object(path, tags, target);
```

要点：preset（工程数据）/ platform（流程实现）/ plugin（逐文件挂钩）三权分立；`_export_file` 让插件**注入/改写/剔除**任何文件。

**UE BuildCookRun**（`AutomationTool/Scripts/BuildCookRun.Automation.cs`）：单命令编排 Build→Cook→Stage→Package→Archive 多阶段，阶段以 `-build -cook -stage -pak -archive` flag 独立开关（`ParseParam` 家族，:31-71 实测 `-foreign/-cookdir/-ddc` 等）——**阶段可单独重跑**是大规模导出的运维底线。

## 现状与证据（zircon）

**导出阶段枚举已存在**——`export_build/wizard/progress.rs:204-215` 实测（类型名 `ExportPipelineStage`，`export_pipeline_stages() -> [ExportPipelineStage; 8]` 定序数组）：

```
Validate, SourceTemplate, NativeDynamic, CompileHost, CookAssets, Pack, PlatformBundle, Report
```

阶段状态 `ExportStageProgressKind::{Pending, Running, Passed, Fatal}`（:4-9）；执行器 `ExportWizardJobController`（`controller.rs:27-78`，14 计划将迁 job 门面）。

**平台词汇也已定型**（`core/framework/project/export_profile.rs` 实读）：`ExportTargetPlatform` 八值（Windows/Linux/Macos/Android/Ios/WebGpu/Wasm/Headless，带 serde alias）、`ExportPlatformHostKind` 四值（Desktop/MobileApp/Browser/Headless）、`ExportPlatformResourceStrategy` 三值（FilesystemBundle/MobileAssetBundle/BrowserFetch）；且 **`ProjectManifest.export_profiles: Vec<ExportProfile>` 已内嵌工程 manifest**（10 已核）。**本计划真实工作**：把阶段枚举从向导私有物提升为三方契约，并在既有 profile（平台/策略）之上补 preset 层（内容筛选/入口/插件子集）。

**工具链三层已有**：

- `tools/zircon_build.py`：`TARGETS=("hub","editor","runtime","plugins")`、`PLUGIN_CARRIERS=("all","native_dynamic","rlib_static")`、`build_native_dynamic_plugin(config, package)`（:108-110 + def 清单实测）——引擎/编辑器自身的 staged 构建。
- `tools/zircon_export/`：30+ Python 模块（pack staging / plugin build / validation，`docs/cli-and-tooling/zircon-export-tool.md` 实测指认）——工程导出的既有工具面。
- runtime 侧：`asset/pack`（打包模块）、`asset/virtual_geometry_cook`（专项 cook 已有先例）、`plugin/export_build_plan/` + `core/framework/project/export_profile.rs`（插件导出计划/档案类型）。

**权威缺口清单**（`docs/plans/zircon_plugins/09-export-publishing.md` 现状节实测）：三路径（编辑器向导/CLI/CI）闭环不完整、**无 zrpack 容器**、无平台模板包、**无统一导出 CLI**。

**本计划与 zircon_plugins/09 分工**：那边管「插件如何被发布」（三包分发/载体策略），本计划管「**工程**如何被发布」，插件打包阶段（`NativeDynamic`）消费其产物契约。

## 目标

1. **`ExportPipeline` 契约化**（现八阶段提升为共用契约，DTO 入 `zircon_runtime_interface/src/export/`）：

```rust
pub enum ExportStage {          // 现 progress.rs 八值直译 + 语义定稿
    Validate,        // preset/引用闭包/平台策略前置校验
    SourceTemplate,  // 平台模板包展开（现缺口，M3）
    NativeDynamic,   // 插件打包（消费 zircon_plugins/09 契约）
    CompileHost,     // 宿主可执行构建（包装 zircon_build.py）
    CookAssets,      // 资产收集+11 二进制转换（现空转，M2 实体化）
    Pack,            // zrpack 容器（现缺口，M2）
    PlatformBundle,  // 平台化布局（staged ZirconEngine/ 树）
    Report,          // 导出报告归档
}
pub struct StageIo { pub inputs: Vec<ArtifactRef>, pub outputs: Vec<ArtifactRef>, pub fingerprint: Digest }
// 每阶段输入输出显式化 → 单阶段重跑（BuildCookRun flag 语义）与增量跳过（fingerprint 等 → skip）
```

2. **preset 分层于既有 profile**（godot preset 直译；**不重复平台词汇**）：`ExportProfile`（既有，manifest 内嵌）继续承担「平台/宿主形态/资源策略」；新建 `ExportPreset`（`<root>/export/<name>.zpreset`，11 版本壳）承担「本次导出的内容与选项」并**引用**一个 profile：`{ profile_ref: String, target_mode: ClientRuntime|ServerRuntime（对齐 ZrRuntimeTargetMode）, debug: bool, include_filter/exclude_filter, entry_scenes: Vec<AssetRef>, keep_list: Vec<AssetRef>, plugin_subset: Option<ProjectPluginManifest 子集>, cook: CookOptions, customized_files: BTreeMap<RelPath, FileExportMode> }`。
3. **CookAssets 实体化**：入口 = preset.entry_scenes + keep_list → 10 registry `dependencies` 闭包遍历 → 11 `Format::Binary` 转换 → cook 缓存（`.zircon/cache/cook/<digest>`，source_digest+cook 参数联合指纹）；`virtual_geometry_cook` 既有专项 cook 注册为 cook 步骤的一类（cook 步骤注册表，渲染管线等后续专项 cook 同位接入）。
4. **Pack：zrpack 容器**（09-export-publishing 点名缺口）：索引段（guid→offset，消费 10 `AssetGuid`）+ 数据段（压缩块）+ 校验段；runtime `asset/pack` 既有模块为读取侧归宿；patch 输出（godot `export_pack_patch` 语义）列远期留位。
5. **ExportPlugin 挂钩**（godot 三钩直译）：`export_begin(preset)/export_file(entry)->FileVerdict{Keep|Replace(data)|Skip}/export_end(report)`——12 贡献族一类，进程内与 cdylib（序列化裁决）双轨。
6. **三路径闭环**：编辑器向导（preset 驱动 + 14 job + 进度中心）/ `export` commandlet（16，同一管线）/ CI（commandlet + 退出码）——`ExportPipeline` 单实现三入口，Validate 消费既有 `ZR_EXPORT_CONTRACT_PLATFORM` 平台策略契约测试口径。

## 非目标

- 商店上架/签名公证；`zircon_build.py`/`zircon_export` Python 层重写（管线以子进程封装，吸收与否待管线稳定后依 frameworks 计划评估）；插件三包分发契约本体（zircon_plugins/09）；热更新/patch 分发体系（zrpack 留 patch 位即止）。

## 架构设计

### 模块布局

```
zircon_runtime_interface/src/export/    # ExportStage/StageIo/preset DTO/报告 DTO
zircon_editor/src/core/export/
  mod.rs
  pipeline.rs        # 阶段编排：拓扑执行/指纹跳过/失败续跑
  stages/            # 八阶段执行体各一文件
  preset.rs          # zpreset 读写（11 壳）
  plugin_hooks.rs    # ExportPlugin 裁决链
zircon_runtime/src/asset/cook/          # cook 步骤注册表 + 依赖闭包收集（registry 消费）
zircon_runtime/src/asset/pack/          # 既有模块扩：zrpack 写入器（读取侧已归此）
# 向导 UI（export_build/wizard）改为 pipeline 的呈现层；commandlet 在 16 注册
```

### 增量与续跑语义

- 阶段指纹 = 输入 artifact digest + 阶段参数 digest；等值 → `Skipped(fingerprint)` 记录进报告（**显式记录跳过**，不静默）。
- 失败续跑：`Report` 记录各阶段 `StageIo` 与状态 → 重启导出时 `--resume` 从首个非 Passed 阶段起（commandlet 与向导同语义）。

### 现物迁移

| 现物 | 去向 |
| --- | --- |
| `progress.rs` 八阶段枚举 | 提升为 interface `ExportStage`（向导侧删除私有枚举，引用契约） |
| `ExportWizardJobController` | 14 M2 已迁 job 门面；本计划向导改为 pipeline 呈现层 |
| `plugin/export_build_plan/` / `core/framework/project/export_profile.rs` | `NativeDynamic` 阶段执行体消费的现行计划与 profile owner；旧扁平文件不再保留 |
| `tools/zircon_export` 校验模块 | `Validate` 阶段执行体子进程封装（清单执行时定稿） |

### 深度测试

夹具工程（3 资产互引 + 1 入口场景 + 1 native_dynamic 夹具插件 + 1 ExportPlugin 夹具）全管线端到端：产物被 `runtime_preview` 直接运行；单阶段篡改产物 → `--resume` 只重跑该阶段及下游；ExportPlugin 的 Replace/Skip 裁决在产物中可验证。

## 里程碑

### M1 管线契约与 preset

- 切片 1.1：interface `export/` DTO + `pipeline.rs`（拓扑/指纹/续跑，夹具阶段验证）；八阶段枚举契约化迁移。
- 切片 1.2：`preset.rs`（zpreset + 11 壳）+ 向导改 preset 驱动；`CompileHost/PlatformBundle` 执行体包装 zircon_build.py（staged 布局校验：assets 合并树 + runtime 库伴随 editor + hub 默认启动器——CLAUDE.md 既定布局断言）。
- 测试阶段：`cargo test -p zircon_runtime_interface --locked` + `cargo test -p zircon_editor --lib --locked`（pipeline 拓扑/跳过/续跑矩阵）+ `ZR_EXPORT_CONTRACT_PLATFORM=windows cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked` 不回归。更新 `docs/cli-and-tooling/zircon-export-tool.md`。

### M2 CookAssets 与 zrpack

- 切片 2.1：`asset/cook/`：依赖闭包收集（10 registry）+ 11 二进制转换 + cook 缓存指纹 + `virtual_geometry_cook` 注册为步骤。
- 切片 2.2：zrpack 写入器（索引/数据/校验三段）+ runtime 读取侧（pack 模块）+ guid 解析表接线。
- 测试阶段：`cargo test -p zircon_runtime --lib --locked`（闭包收集正确性：入口外资产不入包 + keep_list 强制入包；zrpack 写读往返；缓存命中跳过断言）；端到端：夹具工程产物 `runtime_preview` 运行验证（集成测试）。

### M3 挂钩、三路径与增量

- 切片 3.1：`ExportPlugin` 三钩（12 贡献族 + cdylib 序列化裁决）+ 夹具插件注入/剔除用例。
- 切片 3.2：`export` commandlet（16 注册，`--preset <name> --resume` flag）；CI 可选 job；`SourceTemplate` 平台模板包（windows 首个）。
- 测试阶段：三路径产物逐字节等价断言（向导/commandlet/CI 同 preset）；二次导出增量耗时基线（全 Skipped 场景）记状态节。

## 风险与开放问题

- cook 按引用裁剪 vs 脚本动态取资产：keep_list 显式保留 + 导出报告列「registry 中未入包资产」清单供人工核对——规则文档化，不做运行时兜底。
- zrpack 压缩选型（zstd/lz4 分块）与 streaming 读取的耦合，需 runtime asset/load owner 会签读取侧约束后定格式头——M2 前置确认项。
- `SourceTemplate` 平台模板包的内容边界（图标/清单/签名占位）windows 之外平台依据 profile-selection 文档矩阵逐平台立案，不在本计划内铺开。

## 产出记录与时间

请将产出记录放置在子计划中，此处仅展示当前现状的概述

当前状态：M1 正在执行。M1.1 shared export DTO、core pipeline、八阶段 interface hard-cut 与 focused tests 已完成；M1.2 preset/生产向导/core stage executor 已完成实现、整改并获得 `SPEC APPROVED` 与 `QUALITY APPROVED`。Editor15 自有 current-binary focused 矩阵 36/36，Windows 平台策略当前二进制 1/1，Layout15 已回传 `paint_template_nodes` 696/696 与 `ui::layouts` 76/76；Render11 E0425/E0716 编译阻断也已修复并回传 fixed。当前源码 full gate 的最低执行阻塞已继续下沉为 Runtime02 service registry 与 `EditorUiHost.core` 的强 `CoreHandle` 自拥有环；Runtime11 的 task-pool/asset-worker 双预算改为该生命周期修复后的独立复测项。UI03/UI06/UI08 的既有产品失败仍由各自功能计划处理。M1 未取得全量自然 summary，因此不关闭；M2 与 M3 未开始。

fixed 已修复：[export-build-string-error-boundary](14/fixed-2026-07-12-export-build-string-error-boundary.md)

本次 owner 产出记录：[2026-07-12-typed-export-error-hard-cutover.md](15/2026-07-12-typed-export-error-hard-cutover.md)

M1.1 产出记录：[2026-07-12-m1-1-export-pipeline-contract.md](15/2026-07-12-m1-1-export-pipeline-contract.md)

M1.2 产出记录：[2026-07-12-m1-2-preset-production-pipeline.md](15/2026-07-12-m1-2-preset-production-pipeline.md)

本计划测试硬切记录：[compile-host-report-test-hard-cutover](15/2026-07-12-compile-host-report-test-hard-cutover.md)

当前测试阶段记录（`in_progress`）：[m1-full-lib-partition-validation](15/2026-07-12-m1-full-lib-partition-validation.md)

- 当前最低执行阻塞（`open / Runtime02 service-registry 强引用环`）：[service-corehandle-retention-cycle](../../zircon_runtime/runtime/02/failure-2026-07-13-service-corehandle-retention-cycle.md)

- 后续资源预算复测（`open / Runtime11 task-pool 与 asset worker`）：[editor-full-harness-runtime-thread-budget](../../zircon_runtime/runtime/11/failure-2026-07-13-editor-full-harness-runtime-thread-budget.md)

- fixed 已修复：[export-cargo-single-worker-windows-output-hang](09/fixed-2026-07-13-export-cargo-single-worker-windows-output-hang.md)

- fixed 已修复：[retained-painter-component-contract-regressions](15/fixed-2026-07-13-retained-painter-component-contract-regressions.md)

- fixed 已修复：[lightmap-forward-bind-group-integration-compile](15/fixed-2026-07-13-lightmap-forward-bind-group-integration-compile.md)

fixed 已修复：[rich-table-runtime-export-and-layout-boxes](15/fixed-2026-07-12-rich-table-runtime-export-and-layout-boxes.md)

fixed 已修复：[blend-space-visual-test-target-lock](15/fixed-2026-07-12-blend-space-visual-test-target-lock.md)

fixed 已修复：[navigation-query-filter-serde-array](15/fixed-2026-07-12-navigation-query-filter-serde-array.md)

fixed 已修复：[rich-table-layout-provider-visibility](15/fixed-2026-07-12-rich-table-layout-provider-visibility.md)

fixed 已修复：[subsurface-profile-mask-test-inference](15/fixed-2026-07-13-subsurface-profile-mask-test-inference.md)
