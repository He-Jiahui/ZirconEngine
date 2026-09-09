---
related_code:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/foundation/mod.rs
  - zircon_runtime/src/builtin/mod.rs
  - zircon_runtime/src/render_graph/mod.rs
  - zircon_runtime/src/rhi.rs
  - zircon_runtime/src/text/mod.rs
  - zircon_runtime/src/text/font_sdf_build_tool/mod.rs
  - zircon_runtime/src/animation/mod.rs
  - zircon_runtime/src/navigation/mod.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/core/resource/mod.rs
  - zircon_runtime/src/engine_module/mod.rs
  - zircon_runtime/src/diagnostic_log/mod.rs
  - zircon_runtime/src/dynamic_api/mod.rs
implementation_files:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/crates/zr_rhi/src/lib.rs
  - zircon_runtime/crates/zr_resource/src/lib.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 运行时 API Wiki 说明书
tests:
  - zircon_runtime/src/foundation/tests.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/mod.rs
  - zircon_runtime/src/render_graph/tests/mod.rs
  - zircon_runtime/crates/zr_rhi/src/tests/mod.rs
  - zircon_runtime/src/script/vm/tests.rs
  - zircon_runtime/src/dynamic_api/tests/api_table.rs
doc_type: api-reference
---

# zircon_runtime 公开模块 API 参考

本页是 ZirconEngine 高层运行时 crate 的 API 入口索引。它面向 Rust 产品入口、运行时插件和宿主集成方：先说明模块何时可编译，再给出 crate root 可直接导入的公开符号、常见调用形状和失败处理边界。数据字段、算法细节和私有辅助项仍以链接的 Rust 源码与测试为准。

## 范围与导入规则

`zircon_runtime` 是运行时吸收层，而不是单一“引擎对象”。应用通常先由 `builtin` 生成模块组合，再由 `engine_module` 与 `core` 启动服务；业务系统通过功能模块取得专门能力。`resource` 是对 `core::resource` 的 crate-root 再导出，因此其稳定数据协议由 `zr_resource` 和 `zircon_runtime_interface` 共同拥有。

本文只覆盖下列 crate-root 路径：

~~~text
zircon_runtime::{
    foundation, builtin, render_graph, rhi, text, animation, navigation,
    script, resource, engine_module, diagnostic_log, dynamic_api
}
~~~

导入 feature-gated 模块前，先在依赖方启用对应的 Cargo feature。Rust 模块路径仍使用 `diagnostic_log` 与 `dynamic_api`；只有 Cargo feature 名称使用连字符，即 `diagnostic-log` 与 `dynamic-api`。

| 模块 | 编译条件 | 直接职责 | 主要消费者 |
| --- | --- | --- | --- |
| `foundation` | 始终可用 | 持久化配置管理器和 kernel 级模块描述。 | 产品启动、配置面板、服务注册。 |
| `builtin` | 始终可用 | 根据目标、Profile、清单和插件报告编译最终模块图。 | `zircon_app`、动态会话、导出产品。 |
| `render_graph` | `graphics` | 声明资源版本、pass 依赖、队列和 transient 分配计划。 | 图形运行时、渲染 provider。 |
| `rhi` | `graphics` | 后端中立 RHI 合约及 WGPU UI surface 工厂。 | 渲染后端、UI surface、诊断读回。 |
| `text` | `text` | 字体选择、富文本编译、文字排版和 Unicode 快照。 | UI、文本渲染、编辑器。 |
| `animation` | `animation` | 动画模块、播放设置、已编译序列对 World 的应用。 | Scene 更新、动画插件。 |
| `navigation` | `navigation` | 内建导航回退、重寻路预算和操作处理器。 | 动态会话、场景代理。 |
| `script` | `script` | VM 后端、宿主能力、插件生命周期、热重载和反射。 | 脚本插件、编辑器扩展。 |
| `resource` | 始终可用 | 稳定资源身份、强类型 handle、注册表、事件与变更批次。 | Asset、Scene、资源管理 UI。 |
| `engine_module` | 始终可用 | 模块/服务 descriptor、工厂和生命周期契约。 | 所有 runtime module 作者。 |
| `diagnostic_log` | `diagnostic-log` | 进程级异步日志、筛选、落盘和诊断快照。 | 产品入口、脚本、动态库。 |
| `dynamic_api` | `dynamic-api` | C ABI 表、链接会话创建和 shader 预热。 | 外部宿主、动态运行库。 |

`dynamic-api` 会启用 `animation`、`diagnostic-log`、`graphics`、`navigation`、`script`、`ui`；`graphics` 会启用 `text`。单独使用某个模块时仍应显式声明所需 feature，避免依赖“碰巧被其他 feature 间接启用”的构建形状。

### 错误处理约定

- `Result<T, E>`：调用者必须处理。通常 `E` 已携带可显示的诊断，不能在产品边界无条件 `unwrap()`。
- `Option<T>`：通常表示“没有匹配项、未初始化或当前不适用”，而不是一个可恢复的详细错误。
- 返回报告或 receipt 的 API：即使没有 Rust `Err`，也必须读取 report 中的 warning、状态或 counters。
- trait 方法：如果方法定义在 `core::framework` 中，需要把 trait 一同引入作用域，才能在具体实现类型上调用。

## foundation

`foundation` 安装 kernel 级 `ConfigManager`。它将内存中的 JSON 值与磁盘持久化 worker 连接起来；模块描述中把服务登记为 immediate manager，因此正常运行时应从 core 服务容器取得它，而不是在任意业务对象中反复构造。

源码：[模块入口](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/foundation/mod.rs)、[模块描述](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/foundation/module.rs)、[配置实现](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/foundation/runtime/config_manager.rs)。

### 公开符号

~~~text
常量
  FOUNDATION_MODULE_NAME
struct
  FoundationModule
  DefaultConfigManager
函数
  module_descriptor() -> ModuleDescriptor
framework contract
  trait ConfigManager
  enum ConfigManagerError
  struct ConfigPersistenceReport
~~~

`FoundationModule` 实现 `EngineModule`，其 `descriptor()` 返回 kernel 初始化级别的配置 manager 描述。`DefaultConfigManager::new(&CoreHandle) -> Result<DefaultConfigManager, CoreError>` 会恢复已有配置、建立提交栅栏并启动持久化 worker；这不是轻量值对象。

`ConfigManager` 的真实调用面位于 `zircon_runtime::core::framework::foundation`：

~~~rust
use std::time::Duration;
use serde_json::json;
use zircon_runtime::core::framework::foundation::ConfigManager;

fn save_ui_scale(config: &dyn ConfigManager) -> Result<(), Box<dyn std::error::Error>> {
    config.set_value("ui.scale", json!(1.25))?;
    config.flush(Duration::from_secs(2))?;
    let report = config.persistence_report();
    if let Some(error) = report.last_error {
        eprintln!("configuration persistence reported: {error}");
    }
    Ok(())
}
~~~

完整 trait 形状为 `set_value(&self, &str, serde_json::Value) -> Result<(), ConfigManagerError>`、`get_value(&self, &str) -> Option<Value>`、`flush(&self, Duration) -> Result<(), ConfigManagerError>`、`persistence_report(&self) -> ConfigPersistenceReport`，以及基于 `get_value` 的默认 `contains_key`。

### 失败与验证

`DefaultConfigManager::new` 可返回 `CoreError`，例如配置文件恢复、JSON 解析或 worker 启动失败。已取得的 manager 则用 `ConfigManagerError` 表达三类运行中问题：`RuntimeUnavailable`、`Persistence { path, reason }`、`FlushTimedOut { path, timeout }`。`ConfigPersistenceReport` 中的 `dirty_generation`、`persisted_generation`、`pending_flushes`、`failed_writes` 和 `last_error` 是 UI 或 shutdown 逻辑判断“是否真正持久化”的依据。

验证入口：[foundation tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/foundation/tests.rs)、[config manager tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/foundation/runtime/config_manager_tests.rs)。

## builtin

`builtin` 不直接启动 runtime；它把目标模式、运行时 Profile、项目插件清单、链接插件和注册报告归并为一个按依赖顺序排列的 `RuntimeModuleCompositionPlan`。产品入口应把成功 plan 的 `modules()` 交给 core，而不是自行猜测模块顺序。

源码：[公开入口](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/builtin/runtime_modules.rs)、[组合编译器](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/builtin/runtime_modules/composition/compiler.rs)、[结果与拒绝](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/builtin/runtime_modules/composition/outcome.rs)。

### 公开符号

~~~text
enum / struct / type
  BuiltinRuntimeModuleId
  RuntimePluginId
  RuntimeModuleCompositionCompiler<'a>
  RuntimeModuleCompositionIdentity
  RuntimeModuleCompositionPlan
  RuntimeModuleCompositionRejection
  RuntimeModuleCompositionResult
  RuntimeModuleLoadDiagnostic
清单与核心模块函数
  runtime_core_modules()
  default_manifest_for_target(RuntimeTargetMode) -> ProjectPluginManifest
  manifest_for_runtime_profile(RuntimeProfileId) -> ProjectPluginManifest
  manifest_with_mode_baseline(RuntimeTargetMode, Option<&ProjectPluginManifest>)
      -> ProjectPluginManifest
组合函数
  runtime_modules_for_compiled_project_plugin_plan(&CompiledProjectPluginPlan)
      -> RuntimeModuleCompositionResult
  runtime_modules_for_runtime_profile_compiled_project_plugin_plan(RuntimeProfileId, &CompiledProjectPluginPlan)
      -> RuntimeModuleCompositionResult
  runtime_modules_for_target(RuntimeTargetMode, Option<&ProjectPluginManifest>)
      -> RuntimeModuleCompositionResult
  runtime_modules_for_target_with_linked_plugins(RuntimeTargetMode, Option<&ProjectPluginManifest>, impl IntoIterator<Item = impl AsRef<str>>)
      -> RuntimeModuleCompositionResult
  runtime_modules_for_target_with_plugin_registration_reports(
      RuntimeTargetMode, Option<&ProjectPluginManifest>,
      impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>
  ) -> RuntimeModuleCompositionResult
  runtime_modules_for_target_with_plugin_and_feature_registration_reports(
      RuntimeTargetMode, Option<&ProjectPluginManifest>,
      impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
      impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>
  ) -> RuntimeModuleCompositionResult
  runtime_modules_for_runtime_profile(RuntimeProfileId) -> RuntimeModuleCompositionResult
  runtime_modules_for_runtime_profile_with_plugin_registration_reports(
      RuntimeProfileId,
      impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>
  ) -> RuntimeModuleCompositionResult
  runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports(
      RuntimeProfileId, &ProjectPluginManifest,
      impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>
  ) -> RuntimeModuleCompositionResult
  runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports(
      RuntimeProfileId,
      impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
      impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>
  ) -> RuntimeModuleCompositionResult
  runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports(
      RuntimeProfileId, &ProjectPluginManifest,
      impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
      impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>
  ) -> RuntimeModuleCompositionResult
~~~

`RuntimeModuleCompositionResult` 是 `Result<RuntimeModuleCompositionPlan, RuntimeModuleCompositionRejection>`。`RuntimeModuleCompositionCompiler::new(&plan)` 可以继续调用 `for_runtime_profile(profile)`、`with_host_module(Arc<dyn EngineModule>)` 或 `with_host_modules(...)`，最后 `compile()`。`RuntimeModuleCompositionPlan` 提供 `modules()`、`module_descriptors()`、`runtime_plugin_availability()`、`diagnostics()`、`warning_messages()` 和 `identity()`；identity 提供 catalog generation、清单 fingerprint、目标模式、Profile 和十六进制组合 hash。

`RuntimeTargetMode`、`RuntimeProfileId`、`ProjectPluginManifest`、`CompiledProjectPluginPlan` 和各类 `RuntimePluginRegistrationReport` 属于 `zircon_runtime::core::framework::platform`、`zircon_runtime::core::framework::project` 或 `zircon_runtime::plugin`；它们是组合函数的输入，不是 `zircon_runtime::builtin` 根 re-export。跨 crate 调用时按签名来源导入。

上表中的 `'a` 是调用者持有注册报告的共享借用生命周期；这些组合函数只读取报告，不取得报告所有权。`runtime_modules_for_target_with_linked_plugins` 则消费 `IntoIterator<Item = impl AsRef<str>>`，适合从外部插件 key 临时构建输入。

~~~rust
use zircon_runtime::builtin::RuntimeModuleCompositionCompiler;
use zircon_runtime::core::framework::project::RuntimeProfileId;

fn compose(
    project_plan: &zircon_runtime::plugin::CompiledProjectPluginPlan,
) -> Result<(), Box<dyn std::error::Error>> {
    let plan = RuntimeModuleCompositionCompiler::new(project_plan)
        .for_runtime_profile(RuntimeProfileId::Client3d)
        .compile()
        .map_err(|rejection| {
            eprintln!("{}", rejection.required_missing_summary());
            rejection
        })?;
    for warning in plan.warning_messages() {
        eprintln!("runtime module warning: {warning}");
    }
    assert_eq!(plan.module_descriptors().len(), plan.modules().len());
    Ok(())
}
~~~

`BuiltinRuntimeModuleId` 覆盖 foundation、log、tasks、time、frame count、diagnostics、platform、input、asset、scene，以及在相应 feature 下的 text、graphics、script；使用 `module_name()` 与 `for_module_name(&str)` 进行稳定映射。`RuntimePluginId` 提供内建常量，例如 `Ui`、`Navigation`、`Animation`、`Rendering`、`ZrVmLanguage`；外部输入应使用 `parse_key` 或 `FromStr`，不要使用会在非法 key 上 panic 的 `new`。`FromStr::Err` 是私有 `ids` 子模块中的 `RuntimePluginIdParseError`，不能按 crate-root 名称直接导入。

### 拒绝语义与验证

组合器在缺少 required plugin、feature 被禁用、模块图不合法或出现 fatal load diagnostic 时返回 `RuntimeModuleCompositionRejection`。失败对象不是空 plan：读取 `runtime_plugin_availability()`、`diagnostics()`、`required_missing()` 和 `fatal_messages()` 后再决定拒绝启动或降级产品。`RuntimeModuleLoadDiagnostic` 能封装 `CoreError`、未知插件、feature 阻断、插件计划和 asset importer 诊断。

验证入口：[组合测试](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/builtin/runtime_modules/tests/mod.rs)、[清单测试](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/builtin/runtime_modules/tests/manifest.rs)、[注册报告测试](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/builtin/runtime_modules/tests/registration/composition.rs)。

## render_graph

`render_graph` 是声明式 render pass 编译器。调用者创建 transient 或 external resource、声明每个 pass 的读取/写入范围和版本，然后调用 `compile()`；编译器据此验证资源、推导依赖、裁剪无根 pass、规划状态转换和 transient 分配。它不执行 GPU 命令。

可用条件：Cargo feature `graphics`。源码：[入口](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/render_graph/mod.rs)、[builder](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/render_graph/builder.rs)、[编译器](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/render_graph/builder/compile.rs)。

### 公开符号

~~~text
构建与错误
  RenderGraphBuilder
  RenderGraphError
资源 handle、版本和声明
  RgTextureHandle, RgBufferHandle, RenderPassId
  RenderGraphResource, RenderGraphResourceKind, RenderGraphResourceDesc
  RenderGraphResourceDeclaration, RenderGraphResourceLifetime
  RenderGraphResourceUsageFlags, RenderGraphResourceVersion
  RenderGraphResourceVersionToken, RenderGraphTextureViewAlias
  ExternalResource, RenderGraphExternalResourceBinding
  RenderGraphExternalResourceRequirement, RenderGraphExternalResourceType
访问范围与同步 metadata
  RenderGraphBufferRange, RenderGraphTextureSubresourceRange
  RenderGraphTextureAspect, RenderGraphShaderStages
  RenderGraphResourceAccessId, RenderGraphResourceAccessIntent
  RenderGraphResourceAccessMetadata, RenderGraphResourceAccessRange
  RenderGraphVersionedAccessKey, RenderGraphResourceAccessKind
  RenderGraphPassResourceAccess
pass、compute 与 attachment
  QueueLane, PassFlags
  RenderGraphAttachmentLoadOp, RenderGraphAttachmentStoreOp, RenderGraphAttachmentOps
  RenderGraphComputeWorkload, RenderGraphComputeDispatchExtent
  RenderGraphComputePassMetadata, ComputeBindingKind, BindingSchemaEntry
  RenderGraphComputePipelineFamily, RenderGraphComputePipelineFallbackPolicy
  RenderGraphComputePipelineResolution, RenderGraphComputePipelineResolutionStatus
  RenderGraphComputeShaderSource, RenderGraphBufferBindingRange
编译产物与诊断
  CompiledRenderGraph, CompiledRenderPass, CompiledRenderGraphStats
  CompiledRenderGraphResourceStatePlan, CompiledRenderGraphResourceStateTransition
  CompiledRenderGraphAccessAllocationBinding, CompiledRenderGraphAccessAllocationTable
  CompiledRenderGraphComputeBindingAccess, CompiledRenderGraphComputeBindingAccessPacket
  CompiledRenderGraphComputeDispatchAccess, CompiledRenderGraphComputeDispatchAccessPacket
  CompiledRenderGraphExternalAccess, RenderGraphResourceState
  CompiledRenderGraphTransientAllocation, CompiledRenderGraphTransientAllocationId
  CompiledRenderGraphTransientAllocationPlan, CompiledRenderGraphTransientSlotReservation
  RenderGraphPhysicalAllocationId
  RenderGraphDump, RenderGraphDumpPassRow, RenderGraphDumpPassResourceRow
  RenderGraphDumpResourceDesc, RenderGraphDumpResourceRow, RenderGraphDumpTransientSlotRow
  RenderGraphAttachmentBandwidthLedger, RenderGraphAttachmentBandwidthRow
  RenderGraphStoreLintKind, RenderGraphStoreLintReport, RenderGraphStoreLintRow
资源 schema
  RenderResourceSchema, RenderResourceFallback, RenderBufferSchema
  RenderTextureSchema, RenderTextureExtentPolicy, RenderTextureExtentReference
  RenderTextureExtentRounding
~~~

### 构建、版本和编译

`RenderGraphBuilder::new(name)` 创建一个 generation-scoped builder。`add_pass`、`add_pass_with_executor` 和 `add_pass_with_executor_and_declared_queue` 返回 `RenderPassId`。资源入口包括 `create_texture(TextureDesc)`、`create_buffer(BufferDesc)`、`create_texture_view_alias(...)`、`import_external_resource(...)` 及带 binding/usage 的 external texture/buffer 变体。`mark_persistent` 和 `mark_readback` 为资源加入额外生命周期或观察约束。

~~~rust
use zircon_runtime::render_graph::{
    QueueLane, RenderGraphAttachmentOps, RenderGraphBuilder,
};
use zircon_runtime::rhi::{TextureDesc, TextureDimension, TextureFormat, TextureUsage};

fn build_graph() -> Result<(), zircon_runtime::render_graph::RenderGraphError> {
    let mut graph = RenderGraphBuilder::new("deferred-frame");
    let gbuffer = graph.create_texture(TextureDesc::new(
        "gbuffer",
        1280,
        720,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ).with_dimension(TextureDimension::D2));
    let output = graph.import_present_external_resource("viewport-output");
    let gbuffer_pass = graph.add_pass("gbuffer", QueueLane::Graphics);
    let version = graph.write_texture_with_ops_versioned(
        gbuffer_pass,
        gbuffer,
        RenderGraphAttachmentOps::clear_store(),
    )?;
    let lighting = graph.add_pass("lighting", QueueLane::Graphics);
    graph.read_texture_from_version(lighting, version)?;
    graph.write_external(lighting, output)?;
    graph.add_dependency(gbuffer_pass, lighting)?;
    let compiled = graph.compile()?;
    println!("compiled {} passes", compiled.dump().pass_rows.len());
    Ok(())
}
~~~

完整访问族还包括 `read_texture`、`read_texture_with_access`、`access_texture`、`read_texture_with_access_from_version`、`write_texture`、`write_texture_versioned`、`write_texture_with_access_versioned`、`write_storage_texture`、`write_storage_texture_versioned`、`write_texture_with_ops`、`write_texture_with_ops_from_version`，以及对 buffer 与 external resource 的对应 `read_*`/`write_*` 变体。写入的 versioned 版本返回 `RenderGraphResourceVersionToken`；消费者应首选 `*_from_version`，以把依赖锚定到精确生产者，而不是依赖同名资源的隐式最近写入。

### 失败与验证

多数 builder 操作和 `compile(mut self)` 返回 `Result<_, RenderGraphError>`。典型失败是无效或跨 builder handle、资源声明缺失、非法 alias、重复 pass/resource 名称、读在生产前、没有 cull root、循环依赖、compute metadata 与资源 binding 不一致，以及纹理/缓冲区访问范围不合法。`CompiledRenderGraph::dump()`、store lint 报告和 attachment bandwidth ledger 用于在成功编译后解释实际安排。

验证入口：[builder validation](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/render_graph/tests/builder_validation.rs)、[资源依赖](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/render_graph/tests/resource_dependencies.rs)、[cycle 与 culling](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/render_graph/tests/cycles.rs)。

## rhi

`rhi` 是对 `zr_rhi` 的精选 re-export facade；除 `create_default_ui_surface_presenter` 外，大多数类型和 trait 的实现在 `zr_rhi` 子 crate。RHI 描述符本身不拥有 backend；实际 `RenderDevice`、submission 和 UI presenter 必须按其 trait contract 使用。

可用条件：Cargo feature `graphics`。源码：[facade](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/rhi.rs)、[RHI 根](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/crates/zr_rhi/src/lib.rs)、[UI surface](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/crates/zr_rhi/src/ui_surface.rs)。

### 公开符号

~~~text
核心 trait 与错误
  RenderDevice, CommandList, CommandListCommand, UiSurfacePresenter, RhiError
adapter、设备、内存与 submission
  DeviceId, DeviceGeneration, RenderAdapterInfo, RenderBackendCaps
  RenderDeviceLimits, RenderDeviceProfile, RenderDebugInstrumentationStatus
  GpuMemoryBudget, GpuMemoryClass, GpuMemorySnapshot, TransientAllocatorStats
  SubmissionHistory, SubmissionLimits, SubmissionPollReceipt, SubmissionStatus
  SubmissionTicket, RhiSubmissionPacket, RenderQueueClass
资源 handle 与 descriptor
  BufferHandle, TextureHandle, TextureViewHandle, SamplerHandle, ShaderModuleHandle
  BindGroupHandle, BindGroupLayoutHandle, PipelineHandle, PipelineLayoutHandle
  BufferDesc, TextureDesc, TextureViewDesc, SamplerDesc, ShaderModuleDesc
  BindGroupDesc, BindGroupEntryDesc, BindGroupEntryResource
  BindGroupLayoutDesc, BindGroupLayoutEntryDesc, PipelineDesc, PipelineLayoutDesc
  VertexAttributeDesc, VertexBufferLayoutDesc, VertexInputLayoutDesc
  StorageTextureBindingDesc, ColorTargetDesc, DepthStencilStateDesc
  BlendComponentDesc, BlendStateDesc, PrimitiveStateDesc, RasterPipelineStateDesc
  RenderPassColorAttachmentDesc, RenderPassDepthStencilAttachmentDesc
  RenderPassColorLoadOp, RenderPassDepthLoadOp, RenderPassStencilLoadOp
  RenderPassStoreOp, RenderPassTextureViewDesc, SwapchainDesc, RenderSurfaceDescriptor
资源、状态与 enum
  BufferUsage, TextureUsage, TextureDimension, TextureFormat, TextureResidency
  TextureSampleType, TextureViewAspect, TextureViewDimension
  TextureCopyAspect, TextureCopyRegion, AddressMode, FilterMode, MipmapFilterMode
  BindingResourceType, SamplerBindingType, StorageTextureAccess
  ShaderStage, PipelineKind, PresentMode, PrimitiveTopology, VertexFormat, VertexStepMode
  BlendFactor, BlendOperation, CompareFunction, CullMode, FrontFace, IndexFormat
  ColorWriteMask, RenderClearColor, RenderScissorRect, RenderViewportDesc
  RenderNativeSurfaceTarget, SurfaceAcquireOutcome, SurfaceRetryReason
  AccelerationStructureCaps
  SurfaceSessionCreateOutcome, SurfaceSessionReceipt
UI surface
  UiSurfaceDescriptor, UiSurfaceDrawList, UiSurfaceCommand, UiSurfaceCommandKind
  UiSurfaceRect, UiSurfaceTextStyle, UiSurfaceImagePayload, UiSurfaceImageUvRect
  UiSurfaceImageResource, UiSurfaceImageResourceTable
  UiSurfaceStyle, UiSurfaceStyleHandle, UiSurfaceStyledPayload
  UiSurfaceResolvedCommandKind, UiSurfacePresentOutcome, UiSurfacePresentStats
  create_default_ui_surface_presenter(UiSurfaceDescriptor)
      -> Result<Box<dyn UiSurfacePresenter>, RhiError>
诊断读回
  DiagnosticFrameKey, DiagnosticReadbackAdmission, DiagnosticReadbackBudget
  DiagnosticReadbackError, DiagnosticReadbackKind, DiagnosticReadbackReceipt
  DiagnosticReadbackRequestId, DiagnosticReadbackTerminal, DiagnosticReadbackTracker
~~~

### UI surface 的真实调用形状

`UiSurfaceDescriptor::headless(label, width, height)` 和 `native(label, width, height, target)` 创建描述符；`validate()` 在宽或高为零时返回 `RhiError::InvalidSurfaceDescriptor`。factory 始终创建 WGPU presenter，因此设备/表面创建失败也以 `RhiError` 返回。

~~~rust
use zircon_runtime::rhi::{
    create_default_ui_surface_presenter, UiSurfaceDescriptor, UiSurfaceDrawList,
};

fn create_headless_ui() -> Result<(), zircon_runtime::rhi::RhiError> {
    let mut presenter = create_default_ui_surface_presenter(
        UiSurfaceDescriptor::headless("runtime-ui", 1280, 720).with_gpu_timing(),
    )?;
    let draw_list = UiSurfaceDrawList::new((1280, 720), None, Vec::new());
    let stats = presenter.present(&draw_list)?;
    println!("presented commands: {}", stats.visible_command_count);
    Ok(())
}
~~~

`UiSurfacePresenter` 的必需方法为 `resize(&mut self, u32, u32) -> Result<(), RhiError>`、`present(&mut self, &UiSurfaceDrawList) -> Result<UiSurfacePresentStats, RhiError>` 和 `last_present_stats(&self)`；可选的 `is_image_resource_resident` 与默认 `present_owned` 支持运行时 image registry。`UiSurfaceImagePayload` 的 `rgba` 是 straight-alpha RGBA8，生产者不能预先 premultiply。

### 失败与验证

所有 backend 创建、资源创建、命令提交和 surface 操作都可能返回 `RhiError`。`SubmissionTicket`、`SubmissionPollReceipt`、`SubmissionStatus` 与 diagnostic readback receipt 是异步工作完成性的来源，不能把“已提交”解释为“GPU 已完成”。`SurfaceAcquireOutcome` 和 `SurfaceRetryReason` 表示可重试的交换链状态，调用者应按照 outcome 决定重建或跳帧。

验证入口：[RHI 边界测试](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/crates/zr_rhi/src/tests/boundary.rs)、[submission 测试](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/crates/zr_rhi/src/tests/submission.rs)、[UI surface 测试](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/crates/zr_rhi/src/ui_surface/tests.rs)。

## text

text 将字体数据库、Unicode 数据版本、富文本解析、排版和共享布局服务组合成一个 runtime module。它是有状态服务：同一个 core 解析到的 TextRuntimeContext 保持字体 collection 和生命周期一致；关闭模块时 context 进入 Draining 或 Closed，新的布局工作会被拒绝。

可用条件：Cargo feature `text`（`graphics` 会间接启用它）。`font_sdf_build_tool` 子模块另需 `font-sdf-build-tool`。源码：[模块入口](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/text/mod.rs)、[font-SDF 子模块](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/text/font_sdf_build_tool/mod.rs)、[富文本 parser](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/text/rich/parser_registry.rs)、[context](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/text/context.rs)。

### 公开符号清单

~~~text
模块与 context
  TextModule, TEXT_MODULE_NAME, text_runtime_context_for_core
  TextRuntimeContext, TextRuntimeContextId, TextSessionId
  TextRuntimeContextLifecycleState, TextRuntimeContextAccessError
  TextRuntimeContextHealthSnapshot, TextSystemFontPolicy
字体 model
  CompositeFontDescriptor, FaceIndex, FontCultureTag, FontFaceDescriptor
  FontFaceId, FontFamilyDescriptor, FontFamilyName, FontMatch, FontQuery
  FontScript, FontScriptTag, FontStretch, FontStyle, FontWeight
  InstancedFaceId, SubFontRange, VariationCoords
富文本 model
  normalized_open_type_features, InlineBaseline, InlineObjectRef, Iso15924Tag
  LaidOutLine, LaidOutText, LayoutItem, LineBreakTailoringProfile, LinkRef
  OpenTypeFeature, ParagraphOverride, RichIconAssetId, RichInlineWidgetSlotId
  RichListItem, RichListItemKind, RichOrderedListMarker, RichParseResult
  RichTable, RichTableCell, RichTableCellBoxStyle, RichTableCellPadding
  RichTableColumn, RichTextAuthoringDiagnostic, RichTextAuthoringDiagnosticCode
  RichTextAuthoringDiagnosticSeverity, RichTextAuthoringRecovery, RichTextFormat
  ShapedGlyph, ShapedGlyphBreakSafety, ShapedGlyphClusterFlags
  ShapedGlyphLineBreakOpportunity, ShapedGlyphLineBreakReceipt, ShapedGlyphRotation
  ShapedGlyphRun, ShapedGlyphScript, ShapedHardLine, StyleOverride, StyledRun
  TextAlign, TextHorizontalCompositionReceipt, TextOrientation, TextRange, TextWrap
  VerticalGlyphDecision, VerticalMode, MAX_RICH_TABLE_ROW_SPAN
富文本 parser、cache 与布局
  CompiledRichText, RichParseBudget, RichTextContentTrust, RichTextDecoration
  RichTextDecorator, RichTextDecoratorRegistrationError, RichTextDependency
  RichTextParseError, RichTextParser, EmojiShortcodeRegistrationError
  CompiledRichTextCacheReport, TextLayoutFallbackReport
  SharedTextLayoutService, shared_text_layout_service
shaping 与 Unicode
  TextShapingBudgetKind, TextShapingFailureCode, TextShapingFailureDependency
  TextShapingFailureDisposition, TextShapingFailurePhase
  TextShapingFailureReceipt, TextShapingFailureReport
  TextVerticalGlyphDecision, TextVerticalGlyphDecisionBasis
  TextVerticalGlyphFallbackReason, TextVerticalGlyphFeatureSet
  TextVerticalGlyphOrientation, TextVerticalGlyphSubstitution
  compiled_unicode_data_snapshot, compiled_unicode_data_snapshot_id
  TextDataVersion, UnicodeDataSnapshot, UnicodeDataSnapshotId
  UnicodeProviderSnapshot
font-sdf-build-tool feature 下的公开子模块
  zircon_runtime::text::font_sdf_build_tool::{
    FontSdfBakeArtifact, FontSdfBakeReport, FontSdfBakeError,
    FontSdfArtifactInspection, FontSdfBakeMode, FontSdfBakeRequest,
    FontSdfGlyphSelection, bake_font_sdf_artifact, inspect_font_sdf_artifact
  }
~~~

TextModule::for_target(RuntimeTargetMode) 在 client/editor 选择 DiscoverPlatform，server 选择 PackagedOnly；也可用 with_system_font_policy 显式设置。外部代码不能直接调用 context 的 new_*（这些构造器是 crate-private），应先激活 TextModule，再通过 text_runtime_context_for_core(&CoreHandle) 取得 Arc<TextRuntimeContext>。

~~~rust
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::text::{
    text_runtime_context_for_core, TextModule, TextRuntimeContextLifecycleState,
};

fn open_text_context() -> Result<(), zircon_runtime::core::CoreError> {
    let runtime = CoreRuntime::new();
    runtime.register_module(TextModule::default().descriptor())?;
    runtime.activate_module("TextModule")?;
    let context = text_runtime_context_for_core(&runtime.handle())?;
    assert_eq!(context.lifecycle_state(), TextRuntimeContextLifecycleState::Active);
    println!("text context id={}", context.id().get());
    Ok(())
}
~~~

### 富文本解析与预算

RichTextParser::default() 安装内建 decorator/emoji 表；with_budget(RichParseBudget) 建立请求上限。常用方法是 budget()、register_decorator、register_emoji_shortcode、compile(markup, RichTextFormat) 和 compile_with_content_trust(markup, format, trust)。编译结果是共享的 Arc<CompiledRichText>，可用 source_markup、parsed、text、run_for_range、inline_runs、link_runs、dependencies 查询。

~~~rust
use zircon_runtime::text::{
    RichParseBudget, RichTextContentTrust, RichTextFormat, RichTextParser,
};

fn parse_markup() -> Result<(), zircon_runtime::text::RichTextParseError> {
    let budget = RichParseBudget::new(64 * 1024, 64 * 1024)
        .with_max_tokens(4096)
        .with_max_active_tag_depth(32);
    let parser = RichTextParser::with_budget(budget);
    let compiled = parser.compile_with_content_trust(
        "[b]Hello[/b] :smile:",
        RichTextFormat::BbCodeV1,
        RichTextContentTrust::Untrusted,
    )?;
    println!("semantic text: {}", compiled.text());
    Ok(())
}
~~~

Untrusted 是默认且安全的入口；只有来自作者工具链、并经过校验的内容才可选择 TrustedAuthoring（允许平衡的 legacy bidi embedding/override）。超出源字节、token、嵌套、输出、表格或 projection 上限时返回 RichTextParseError，注册 decorator/emoji 还可能返回各自的 registration error；注册会增加 generation 并清空该 parser 的 compiled cache。TextRuntimeContextAccessError::Unavailable 等错误表示 context 已 draining/closed，不应通过重试绕过生命周期。

### 验证与 feature

共享布局服务入口是 `shared_text_layout_service() -> &'static dyn TextLayoutService`；`TextLayoutService` 属于 `zircon_runtime::core::framework::text`，不是 `zircon_runtime::text` 根 re-export。它返回进程内无状态 facade，具体 session admission 由 context 管理。`compiled_unicode_data_snapshot()` 与 `compiled_unicode_data_snapshot_id()` 提供编译时 Unicode 版本，跨缓存或网络传输时应同时保存 `TextDataVersion` 和 snapshot id。

验证入口：[rich parser tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/text/rich/tests.rs)、[shaping tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/text/shaping/tests.rs)、[font database tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/text/font/database/tests.rs)、[layout session tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/text/layout_session/tests.rs)。

## animation

animation 把 animation framework 的编译序列投影到一个具体 Scene World。绑定解析只在编译或编辑边界发生；帧内应用使用已编译 property writer，避免重复解析实体路径。

可用条件：Cargo feature `animation`。源码：[模块入口](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/animation/mod.rs)、[序列 projection](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/animation/sequence/compiled.rs)、[manager](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/animation/manager/mod.rs)。

### 公开符号

~~~text
常量
  PLUGIN_ID = "animation"
  ANIMATION_PLAYBACK_CONFIG_KEY = "animation.playback_settings"
  ANIMATION_MODULE_NAME, ANIMATION_DRIVER_NAME, DEFAULT_ANIMATION_MANAGER_NAME
struct / trait
  AnimationModule, AnimationDriver, DefaultAnimationManager
  ProjectAnimationClipEventSampler
  CompiledAnimationSequence, CompiledAnimationSequenceApplyStats
函数
  module_descriptor()
  compile_sequence_for_world(&mut World, &AnimationCompiledSequence)
      -> SceneResult<CompiledAnimationSequence>
  apply_compiled_sequence_to_world(&mut World, &CompiledAnimationSequence, Real, bool)
      -> AnimationResult<CompiledAnimationSequenceApplyStats>
~~~

DefaultAnimationManager::new(Option<&CoreHandle>) 创建 framework AnimationManager 实现；store_playback_settings(AnimationPlaybackSettings) 将设置持久化到 foundation config。ProjectAnimationClipEventSampler::new(&ProjectAssetManager) 从项目 asset manager 解析 clip event。

~~~rust
use zircon_runtime::animation::{
    apply_compiled_sequence_to_world, compile_sequence_for_world,
};

fn apply_frame(
    world: &mut zircon_runtime::scene::World,
    source: &zircon_runtime::core::framework::animation::compiler::sequence::AnimationCompiledSequence,
) -> Result<usize, Box<dyn std::error::Error>> {
    let compiled = compile_sequence_for_world(world, source)?;
    let stats = apply_compiled_sequence_to_world(world, &compiled, 0.25, true)?;
    if !compiled.missing_tracks().is_empty() {
        eprintln!("{} animation tracks remain unbound", compiled.missing_tracks().len());
    }
    Ok(stats.applied_tracks)
}
~~~

CompiledAnimationSequence::missing_tracks() 返回未解析的 AnimationTrackPath；is_current_for(&World) 用 world binding/schema generation 判断是否需要重新编译。apply_compiled_sequence_to_world 的 looping 参数控制时间是否回绕，返回的 applied_tracks/missing_tracks 是固定大小统计，缺失绑定不会在帧路径上分配诊断字符串。

### 失败与验证

编译阶段可能返回 SceneError/SceneResult，例如实体路径不存在、property writer 无法生成或 binding catalog 失效；应用阶段返回 AnimationError。不要把 missing track 当作 panic；可在层级变化后重新调用 compile_sequence_for_world。验证入口：[sequence tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/animation/sequence/tests.rs)、[manager tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/animation/manager/state_machine/borrowed_state_index_tests.rs)。

## navigation

navigation 提供可工作的 baked-navmesh fallback。当前内建 manager 能加载、查询和驱动 agent，但不会从 World 烘焙表面；需要 per-query filter 或烘焙时应注册 navigation plugin。

可用条件：Cargo feature `navigation`。源码：[模块入口](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/navigation/mod.rs)、[manager](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/navigation/runtime.rs)、[framework contract](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/core/framework/navigation/manager.rs)。

### 公开符号

~~~text
  BuiltinNavigationModule, BuiltinNavigationManager, NavRepathBudget
  BUILTIN_NAVIGATION_MODULE_NAME
  module_descriptor()
  register_navigation_operation_handlers(
      &mut zircon_runtime::operation::RuntimeOperationService
  ) -> Result<(), zircon_runtime::operation::RuntimeOperationServiceError>
NavigationManager contract（位于 core::framework::navigation）
  load_nav_mesh(NavMeshAsset) -> Result<NavMeshHandle, NavigationError>
  load_navigation_settings(NavigationSettingsAsset) -> Result<(), NavigationError>
  find_path(NavPathQuery) -> Result<NavPathResult, NavigationError>
  find_path_with_filter(NavPathQuery, &NavQueryFilter) -> Result<NavPathResult, NavigationError>
  sample_position(NavSampleQuery) -> Result<Option<NavSampleHit>, NavigationError>
  raycast(NavRaycastQuery) -> Result<NavRaycastResult, NavigationError>
  stats() -> NavigationRuntimeStats
数据类型
  NavMeshAsset, NavMeshPolygonAsset, NavMeshTileAsset, NavMeshLinkAsset
  NavMeshBakeRequest, NavMeshBakeReport, NavMeshBakeDiagnostic
  NavPathQuery, NavPathResult, NavPathStatus, NavQueryFilter
  NavSampleQuery, NavSampleHit, NavRaycastQuery, NavRaycastResult
  NavigationSettingsAsset, NavigationError, NavigationErrorKind
  NavMeshHandle, NavAgentTickReport, NavMeshAgentDescriptor
~~~

`BuiltinNavigationModule`、`BuiltinNavigationManager`、`NavRepathBudget`、`module_descriptor` 和 `register_navigation_operation_handlers` 是 `zircon_runtime::navigation` 根导出；`NavigationManager` 及其余 contract/data 类型应从 `zircon_runtime::core::framework::navigation` 导入。

NavRepathBudget::new(max_queries_per_frame)、begin_frame()、try_consume() 为每帧重寻路建立硬上限；默认值是 32。BuiltinNavigationManager::new()/Default 创建线程安全 manager，tick_world_agents(&mut World, dt_seconds) 和 tick_world_agent(...) 写回 Scene transform。

~~~rust
use zircon_runtime::core::framework::navigation::{
    NavigationManager, NavMeshAsset, NavPathQuery,
};
use zircon_runtime::navigation::BuiltinNavigationManager;

fn query_quad() -> Result<(), Box<dyn std::error::Error>> {
    let manager = BuiltinNavigationManager::new();
    let mesh = manager.load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 10.0))?;
    let mut query = NavPathQuery::new([0.0, 0.0, 0.0], [1.0, 0.0, 1.0]);
    query.nav_mesh = Some(mesh);
    let path = manager.find_path(query)?;
    println!("path status: {:?}", path.status);
    Ok(())
}
~~~

空 NavMeshAsset 返回 NavigationError::MissingNavMesh；未指定或已卸载的 handle 也会产生 navigation error。bake_surface 固定返回 NavigationErrorKind::BackendFailure，find_path_with_filter 同样明确要求 navigation plugin。验证入口：[navigation runtime tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/navigation/runtime/tests.rs)、[framework navigation tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/core/framework/navigation/tests.rs)。

## script

script 是 VM 插件边界。它把宿主函数注册为 capability-protected exports，把 VM backend 与 package discovery 分开，并由 VmPluginManager/HotReloadCoordinator 管理 load、call、GC、反射和热重载。没有可用 VM backend 时，UnavailableVmBackend 会返回可诊断的 VmError，不会静默执行脚本。

可用条件：Cargo feature `script`（该 feature 会启用 `diagnostic-log`）。源码：[模块入口](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/script/mod.rs)、[backend registry](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/script/vm/backend/backend_registry.rs)、[plugin manager](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs)。

### 公开符号清单

~~~text
模块与注册入口
  ScriptModule, SCRIPT_MODULE_NAME, module_descriptor
  register_builtin_host_modules, register_gameplay_host_module
  register_bridge_host_module, builtin_host_capabilities
  builtin_host_module_descriptors, render_script_host_modules_markdown
  write_script_host_modules_markdown
backend
  VmBackend, VmBackendFamily, VmBackendRegistry, BuiltinVmBackendFamily
  UnavailableVmBackend, VmError
plugin 与生命周期
  VmPluginManager, HotReloadCoordinator, PluginHostDriver
  VmPluginInstance, VmPluginManifest, VmPluginPackage, VmPluginPackageSource
  VmPluginHostContext, VmPluginSlotRecord, VmPluginSlotState, PluginSlotId
  VmPluginSlotLifecycle, VmPluginManagementPolicy, VmPluginManagementPolicyError
  VmPluginManagementPolicyResult, VmPluginHotReloadPolicy
  VmPluginMemoryPolicy, VmPluginGarbageCollectionMode, VmPluginGarbageCollectionPolicy
  ZrVmExecutionMode, ZrVmPluginProjectSource
  discover_vm_plugin_package, discover_vm_plugin_package_with_limits
  discover_vm_plugin_packages, discover_vm_plugin_packages_with_limits
  VmPluginDiscoveryRequest, VmPluginDiscoveryLimits, DiscoveredVmPluginPackage
host capability 与调用表
  CapabilitySet, HostRegistry, HostHandle, HostCapabilityRecord, HostRegistryError
  HostExportFunction, HostExportCallback, HostExportModuleRecord, HostExportRegistry
  ScriptCallSiteId, ScriptCallSite, ScriptCallTable
  ScriptHostInterfaceMarkdownOptions
  ScriptBridgeCall, ScriptBridgeMethodDescriptor
反射、系统与 scene bridge
  VmReflectionCatalog, VmReflectionRegistrySnapshot, VmReflectionSchema
  VmReflectionSchemaInstaller, VmReflectionWorldAccess, VmReflectionWorldOperation
  VmReflectionError, VmScriptBehaviorBridge, VmBehaviorNodeRegistration
  VmRpcHandlerRegistration, VmEditorOperationRegistration, VmSystemRegistration
  VmSystemStage, ScriptSceneLifecyclePhase, ScriptSceneRuntimeSystem
  VmHostInterfaceRegistry, VmHostInterfaceError, VmInterfaceCaller
  VmCallbackHandle
GC 与对象
  VmGcBudget, VmGcDiagnostics, VmGcRootRegistry, VmGcRootToken
  VmGcRootRegistrationError, VmGcSlotStepReport, VmGcStepOutcome, VmGcStepReport
  VmObjectId, VmObjectRef, VmObjectRefError
状态迁移
  VmStateBlob, VmStateObject, VmStateFieldValue, VmStateSchema
  VmStateTypeIdentity, VmStateTypeSchema, VmStateMigrationError
  migrate_vm_state_blob
常量
  SCRIPT_MODULE_NAME, PLUGIN_HOST_DRIVER_NAME, VM_PLUGIN_RUNTIME_NAME
  VM_PLUGIN_MANAGER_NAME, VM_HOST_INTERFACE_MODULE
  BRIDGE_HOST_CAPABILITY, BRIDGE_HOST_MODULE
  SCRIPT_SCENE_FIXED_UPDATE_SYSTEM, SCRIPT_SCENE_UPDATE_SYSTEM
  SCRIPT_SCENE_RUNTIME_SYSTEM_SET, VM_GC_DIAGNOSTICS_HISTORY_CAPACITY
  VM_SYSTEM_CAPABILITY, VM_BT_NODE_CAPABILITY, VM_RPC_HANDLER_CAPABILITY
  VM_EDITOR_OPERATION_CAPABILITY, VM_REFLECTION_WORLD_EXTENSION_NAME
  VM_STATE_SCHEMA_VERSION_V3, DEFAULT_VM_GC_MAX_MICROS_PER_FRAME
utility
  script_float(f32) -> ScriptHostValue
test-support feature（仅测试，不是产品 API）
  ScriptRuntimeTestContext, with_script_runtime_test_context
~~~

backend registry 的调用形状是 VmBackendRegistry::new()、register_family(Arc<dyn VmBackendFamily>) -> String、resolve(selector) -> Result<Arc<dyn VmBackend>, VmError>、contains 和 names。family 负责把 builtin:mock/builtin:unavailable 或自定义 selector 解析到 VmBackend；backend 的 load_package(&VmPluginPackage, &VmPluginHostContext) -> Result<Box<dyn VmPluginInstance>, VmError> 是唯一装载边界。

~~~rust
use std::sync::Arc;
use zircon_runtime::script::{BuiltinVmBackendFamily, VmBackendRegistry};

fn select_vm() -> Result<String, zircon_runtime::script::VmError> {
    let registry = VmBackendRegistry::new();
    registry.register_family(Arc::new(BuiltinVmBackendFamily));
    let backend = registry.resolve("builtin:mock")?;
    println!("available selectors: {:?}", registry.names());
    Ok(backend.backend_name().to_owned())
}
~~~

宿主 export 使用 `HostExportRegistry::new(HostRegistry)`、`register_module(ScriptHostModuleDescriptor, impl IntoIterator<Item = HostExportFunction>) -> Result<HostHandle, VmError>`、`module`/`modules`、`script_call_table`、`call` 和 `call_with_capabilities`。`ScriptHostModuleDescriptor`、`ScriptHostCallFrame`、`ScriptHostValue` 和 `ScriptHostResult` 属于 `zircon_runtime::core::framework::script`，不是 `zircon_runtime::script` 根 re-export；`HostExportFunction::new(name, callback)` 的 callback 形状是 `for<'frame> Fn(&ScriptHostCallFrame<'frame>) -> ScriptHostResult + Send + Sync + 'static`。调用者必须提供满足模块声明的 `CapabilitySet`，否则返回 `VmError`。

CapabilitySet::with 会排序并去重 capability，contains 与 manifest 顺序无关。VmPluginManager::with_builtin_backends(host) 可建立带内建 backend 的 manager；随后可用 register_family、select_default_backend、discover_packages、load_package、hot_reload_slot、unload_slot、call_slot_export、gc_step 和 run_registered_systems 管理生命周期。`script_float(f32)` 将宿主侧 `f32` 明确提升为 `ScriptHostValue::Float(f64)`，用于避免脚本数值 ABI 的隐式转换。`with_script_runtime_test_context` 和 `ScriptRuntimeTestContext` 仅在 `test-support` feature 下导出，不是产品 API。

### 错误、状态和验证

VmError 覆盖未知 backend、package/manifest 无效、host capability、slot 状态、callback、GC 和执行失败。热重载应先读取 VmPluginSlotRecord.state，等待 draining slot 的 callback/GC 结束，再接受新 generation；不要持有旧 VmObjectRef 跨 unload。VmStateBlob 与 migrate_vm_state_blob 用 schema/type identity 搬迁脚本状态，迁移失败由 VmStateMigrationError 返回。

验证入口：[backend registry tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/script/vm/backend/backend_registry.rs)、[host/export tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/script/vm/tests/host_exports.rs)、[lifecycle tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/script/vm/tests/lifecycle_failures.rs)、[state migration tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/state_migration.rs)。

## resource

resource 是 zircon_runtime::core::resource 的 crate-root projection；它不重新定义资源协议。稳定 identity、marker、record 和 event DTO 来自 zircon_runtime_interface，registry、lease、snapshot、readiness/management generation 和 mutation 则由 zr_resource 实现。

可用条件：始终可用。源码：[runtime projection](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/core/resource/mod.rs)、[zr_resource root](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/crates/zr_resource/src/lib.rs)。

### 公开符号清单

~~~text
身份与协议
  AssetReference, AssetUuid, ResourceId, ResourceLocator, ResourceLocatorError
  ResourceScheme, STABLE_UUID_ALGORITHM_VERSION
  ResourceKind, ResourceMarker
  Resource, ResourceRuntimeInfo, RuntimeResourceState
  ResourceDiagnostic, ResourceDiagnosticSeverity, ResourceEvent, ResourceEventKind
  ResourceHandle, UntypedResourceHandle, ResourceRecord, ResourceState
typed marker
  AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker
  AnimationSkeletonMarker, AnimationStateMachineMarker, DataMarker, FontMarker
  MaterialGraphMarker, MaterialMarker, MeshMarker, ModelMarker, NavMeshMarker
  NavigationSettingsMarker, PhysicsMaterialMarker, PrefabMarker, SceneMarker
  ShaderMarker, SoundMarker, TerrainLayerStackMarker, TerrainMarker
  TextureMarker, TileMapMarker, TileSetMarker, UiLayoutMarker, UiStyleMarker, UiWidgetMarker
registry 与快照
  ResourceData, ResourceLease, ResourceRegistry, ResourceSnapshot
  ResourceManager, ResourceRegistryReadGuard, ResourceProjectionSnapshot
  ResourceRegistryError, ResourceResult
mutation
  ResourceMutationBatch, ResourceMutationReceipt
readiness / management generation
  ResourceReadinessState, ResourceReadinessGeneration
  ResourceReadinessGenerationDiagnostics, ResourceReadinessGenerationIdentity
  ResourceReadinessRow, ResourceReadinessRowIdentity
  ResourceManagementGeneration, ResourceManagementGenerationDiagnostics
  ResourceManagementGenerationIdentity, ResourceManagementKindSummary
  ResourceManagementPage, ResourceManagementQuery, ResourceManagementRow
  ResourceManagementRowIdentity, ResourceManagementScan, ResourceManagementSummary
events
  ResourceEventGap, ResourceEventReceiver, ResourceEventRecvError
  ResourceEventRecvTimeoutError, ResourceEventStreamDiagnostics
  ResourceEventTryRecvError
I/O helper
  resource::io::atomic_write, resource::io::atomic_write_new
  resource::io::ArtifactIdentityExhausted
~~~

ResourceId::from_stable_label(label) 生成可复现 identity；ResourceHandle<TMarker> 将 marker 与 id 绑定，避免把 model id 传给 texture API。ResourceRegistry/ResourceManager 负责注册、读取和投影；需要跨线程短时读取时使用 ResourceLease 或 ResourceSnapshot，不要把 registry 内部 guard 存入长期对象。

~~~rust
use zircon_runtime::core::resource::{ModelMarker, ResourceHandle, ResourceId};

fn stable_model_handle() -> ResourceHandle<ModelMarker> {
    ResourceHandle::new(ResourceId::from_stable_label("res://models/robot.glb"))
}
~~~

事件 receiver 的 try_recv/recv 可能返回 ResourceEventGap；出现 gap 时必须用新的 ResourceSnapshot 或 management/readiness generation 重建视图，而不是继续假设事件流完整。ResourceMutationBatch 提交后读取 ResourceMutationReceipt，并根据 ResourceRegistryError 处理冲突、未知 identity、状态和容量错误。

验证入口：[resource tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/crates/zr_resource/src/tests.rs)、[event stream tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/crates/zr_resource/src/event_stream.rs)、[management projection tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/crates/zr_resource/src/manager/management_projection/tests.rs)。

## engine_module

engine_module 是所有 runtime module 作者共享的声明边界。EngineModule 只描述模块身份和 ModuleDescriptor；真正的 driver/manager/plugin 对象由 descriptor 中的 ServiceFactory 延迟或立即创建。这样 core 可以先冻结依赖图，再按 InitLevel 和 StartupMode 激活。

可用条件：始终可用。源码：[入口](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/engine_module/mod.rs)、[EngineModule](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/engine_module/engine_module.rs)、[service contracts](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/engine_module/engine_service.rs)。

### 公开符号

~~~text
核心 module/runtime 类型（由 core re-export）
  CoreHandle, CoreRuntime, CoreWeak, DependencySpec
  DriverDescriptor, ManagerDescriptor, PluginDescriptor
  InitLevel, LifecycleState, ModuleContext, ModuleDependencySpec
  ModuleDescriptor, ModuleLifecycle, NoopModuleLifecycle
  PluginContext, PluginFactory, RegistryName, ServiceFactory
  ServiceKind, StartupMode
trait 与 contract
  EngineModule
  EngineService, EngineDriver, EngineManager, EnginePlugin
  DriverContract, ManagerContract, PluginContract
  driver_contract, manager_contract, plugin_contract
构造和命名函数
  module_context(module_name, CoreWeak) -> ModuleContext
  plugin_context(plugin_name, CoreWeak) -> PluginContext
  qualified_name(module, ServiceKind, service) -> RegistryName
  dependency_on(module, ServiceKind, service) -> DependencySpec
  factory(builder: Fn(&CoreWeak) -> Result<ServiceObject, CoreError>)
      -> ServiceFactory
  plugin_factory(builder: Fn(&PluginContext) -> Result<ServiceObject, CoreError>)
      -> PluginFactory
~~~

ModuleDescriptor::new(name, description) 之后可链式调用 with_init_level、with_module_dependency、with_lifecycle、with_driver、with_manager 和 with_plugin。descriptor 的 name 必须全局唯一；服务名应使用 qualified_name，依赖应使用 dependency_on，不要手写同形字符串。EngineService contract 可通过 owner_module、registry_name、service_kind、startup_mode 和 dependencies 读取冻结后的声明。

~~~rust
use zircon_runtime::engine_module::{EngineModule, InitLevel, ModuleDescriptor};

#[derive(Debug)]
struct GameplayModule;

impl EngineModule for GameplayModule {
    fn module_name(&self) -> &str {
        "gameplay"
    }

    fn module_description(&self) -> &str {
        "Project gameplay services"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(self.module_name(), self.module_description())
            .with_init_level(InitLevel::Services)
    }
}

fn register() -> Result<(), zircon_runtime::core::CoreError> {
    let runtime = zircon_runtime::core::CoreRuntime::new();
    runtime.register_module(GameplayModule.descriptor())?;
    runtime.activate_module("gameplay")?;
    Ok(())
}
~~~

factory 的闭包接收 `&CoreWeak`，必须显式 upgrade 并在 runtime 已关闭时返回 `CoreError::RuntimeUnavailable`；plugin_factory 接收 `&PluginContext`，可读取 plugin owner 和 package roots。两者都返回共享的 `ServiceObject`（完整路径 `zircon_runtime::core::runtime::ServiceObject`，即 `Arc<dyn Any + Send + Sync>`），不是普通同步函数指针。

### 生命周期、错误与验证

CoreRuntime::register_module 在重复名称、依赖缺失或 descriptor 冲突时返回 CoreError；activate_module 还可能报告依赖循环、service factory 失败和 ready timeout。ModuleLifecycle::cleanup/cleanup_until 必须释放 module-owned service，但不能在 deadline 后继续阻塞。验证入口：[engine module tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/engine_module/tests.rs)、[runtime registration tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/core/runtime/tests/registration/structure/module_layout.rs)。

## diagnostic_log

`diagnostic_log` 是进程级、线程安全的异步日志 sink。它负责筛选、console/file 输出、批量和有界 flush；业务代码只提交字符串，sink 未初始化或已 shutdown 时写入会被丢弃而不会 panic。该模块的 Cargo feature 名称是 `diagnostic-log`。

可用条件：Cargo feature `diagnostic-log`。源码：[入口](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/diagnostic_log/mod.rs)、[level/filter](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/diagnostic_log/level.rs)、[sink](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/diagnostic_log/sink.rs)。

### 公开符号

~~~text
级别、过滤和环境变量
  DiagnosticLogLevel, DiagnosticLogFilter, DiagnosticLogFilterConfig
  DiagnosticLogModuleFilter, DiagnosticLogLevelParseError
  DIAGNOSTIC_LOG_LEVEL_ENV, DIAGNOSTIC_LOG_FILTER_ENV
  DIAGNOSTIC_LOG_ENV, RUST_LOG_ENV
设置与位置
  DiagnosticLogSettings, LogSettings, DiagnosticLogSinkSettings
  DiagnosticLogLocation
  DEFAULT_DIAGNOSTIC_LOG_QUEUE_CAPACITY
  DEFAULT_DIAGNOSTIC_LOG_BATCH_RECORDS, DEFAULT_DIAGNOSTIC_LOG_BATCH_BYTES
  DEFAULT_DIAGNOSTIC_LOG_FLUSH_INTERVAL
  DEFAULT_DIAGNOSTIC_LOG_CRASH_FLUSH_TIMEOUT
  DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT
进程 sink
  DiagnosticLogSinkSnapshot
  initialize_process_log, initialize_process_log_with_filter
  initialize_process_log_with_config, initialize_process_log_with_location
  initialize_process_log_with_location_and_filter
  initialize_process_log_with_settings
  initialize_unity_process_log, initialize_unity_process_log_with_filter
  initialize_unity_process_log_with_config
  write_diagnostic_log, write_diagnostic_log_at
  write_debug_log, write_log, write_warn, write_error
  write_diagnostic_log_lazy, write_diagnostic_log_lazy_at
  write_debug_log_lazy, write_log_lazy, write_warn_lazy, write_error_lazy
  diagnostic_log_allows, diagnostic_log_allows_for_scope
  diagnostic_log_sink_snapshot, install_process_log_panic_flush
  flush_process_log, shutdown_process_log
diagnostic store 输出
  DiagnosticStoreLogSchedule, DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT
  format_diagnostic_store_snapshot, format_diagnostic_store_current_snapshot
  write_diagnostic_store_snapshot, write_diagnostic_store_current_snapshot
~~~

DiagnosticLogLevel 的顺序是 Verbose < Debug < Log < Warn < Error；DiagnosticLogFilter::Off 完全关闭，Minimum(level) 设置全局下限。DiagnosticLogFilterConfig::parse(value, default_filter) 支持 warn,module=debug 形式，allows 和 filter_for_scope 在写入前判断模块筛选。DiagnosticLogSettings::new(channel) 默认读取环境变量；unity_compatible 将 DiagnosticLogLocation 设为 UnityCompatibleFirst，并可继续链式配置 filter、console/file 和 sink limits。

~~~rust
use std::time::Duration;
use zircon_runtime::diagnostic_log::{
    flush_process_log, initialize_process_log_with_settings, write_debug_log,
    DiagnosticLogFilter, DiagnosticLogLevel, DiagnosticLogSettings,
};

fn start_logging() {
    let settings = DiagnosticLogSettings::new("gameplay")
        .with_filter(DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Debug))
        .with_console_enabled(true);
    let path = initialize_process_log_with_settings(settings);
    write_debug_log("gameplay.spawn", "spawn system ready");
    if !flush_process_log(Duration::from_secs(1)) {
        eprintln!("diagnostic log flush did not complete");
    }
    if let Some(path) = path {
        eprintln!("diagnostic file: {}", path.display());
    }
}
~~~

写入 API 没有 Result；需要在昂贵消息上使用 lazy 变体，避免被 filter 丢弃时构造字符串。initialize_* 返回 Option<PathBuf>，None 表示文件路径不可用（console 仍可能工作）；flush_process_log/shutdown_process_log 返回 false 表示超时或输出失败。DiagnosticStoreLogSchedule::disabled/repeating/tick 用于按周期把 core diagnostic snapshot 转成日志。

### 失败与验证

非法 level/filter 字符串返回 `DiagnosticLogLevelParseError`；sink 队列满时 critical enqueue 受有界 timeout 约束，不能假设无界写入。进程日志只有一个 active controller；已有 active sink 时，后续 `initialize_*` 请求保持当前 generation 和设置，不会替换正在运行的 sink。shutdown 后应重新初始化而不是复用旧状态。验证入口：[level tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/diagnostic_log/level/borrowed_parse_tests.rs)、[sink lifecycle](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/diagnostic_log/sink/tests/lifecycle.rs)、[backpressure](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/diagnostic_log/sink/tests/backpressure.rs)。

## dynamic_api

`dynamic_api` 是动态库唯一的 C ABI 边界，同时提供 Rust 侧链接会话和 shader prewarm 辅助。它的 API table 由 zircon_runtime_interface 冻结；不要把内部 session 函数或 Rust trait 当作 ABI。该模块的 Cargo feature 名称是 `dynamic-api`。

可用条件：Cargo feature `dynamic-api`。源码：[入口](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/dynamic_api/mod.rs)、[API export](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/dynamic_api/exports.rs)、[linked session](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/dynamic_api/session/linked_session.rs)、[shader prewarm](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/dynamic_api/shader_prewarm.rs)。

### 公开符号

~~~text
C ABI export
  zircon_runtime_get_api_v8(host: *const ZrHostApiV1) -> *const ZrRuntimeApiV8
Rust linked session
  create_linked_runtime_session(
      profile: &[u8],
      project_root: Option<&Path>,
      registrations: Vec<RuntimePluginRegistrationReport>
  ) -> Result<ZrRuntimeSessionHandle, RuntimeDynamicSessionError>
  RuntimeDynamicSessionError
  RuntimeProjectError
shader prewarm
  prewarm_shader_variants
  prewarm_shader_variants_with_execution_budget
  prewarm_shader_variants_with_wgpu_module_validation
  prewarm_shader_variants_with_wgpu_pipeline_validation
  prewarm_shader_variants_with_wgpu_module_and_pipeline_validation
  builtin_fallback_shader_prewarm_manifest
  builtin_standard_material_shader_prewarm_manifest
  builtin_standard_material_shader_prewarm_manifest_for_geometry
  builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor
  material_surface_shader_prewarm_template_source
  default_shader_variant_cache_root_for_project
  default_staged_shader_variant_cache_root_for_project
  ShaderPrewarmTemplateSource
~~~

`ShaderVariantPrewarmManifest`、`ShaderVariantPrewarmReport`、`ShaderVariantPrewarmExecutionBudget` 以及 geometry descriptor 类型属于 `zircon_runtime::core::framework::render` 的公开 re-export（内部实现模块 `shader` 不对外开放）；`dynamic_api` 根模块只 re-export 上述 prewarm 函数和 `ShaderPrewarmTemplateSource`。按该公开路径导入这些参数/返回类型，避免把它们误写成 `zircon_runtime::dynamic_api::*`。

zircon_runtime_get_api_v8 首先校验 ZrHostApiV1 的 ABI version、size 和指针形状；校验失败或 getter 内部 panic 时返回 null。成功返回的 ZrRuntimeApiV8 是静态、不可变 table，当前包含 create/destroy session、frame capture/tick、viewport surface/present、operation、world query/watch、plugin event 和 viewport pick 等函数指针。宿主必须先检查 abi_version 与 size_bytes，再按 Option<fn> 调用；table 内每个 FFI wrapper 都把 panic 转为 ZrStatusCode::Panic。

~~~rust
use zircon_runtime::dynamic_api::zircon_runtime_get_api_v8;
use zircon_runtime_interface::{
    ZrHostApiV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

fn get_table() -> Option<&'static zircon_runtime_interface::ZrRuntimeApiV8> {
    let host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
    let pointer = unsafe { zircon_runtime_get_api_v8(&host) };
    if pointer.is_null() {
        return None;
    }
    Some(unsafe { &*pointer })
}
~~~

create_linked_runtime_session 接受的 profile bytes 为 runtime（空 bytes 也按 runtime）、runtime-pipelined、editor、dev、minimal 或 headless。未知 profile 返回 RuntimeDynamicSessionError::UnknownProfile；项目根解析、模块发现、task graph、render bridge、asset/nav/script/UI startup 的每一步都有带 step/source 的变体，handle 空间耗尽返回 SessionHandleSpaceExhausted。项目根不是 UTF-8、Play scene 非 project-relative/格式不支持或资源不存在时，使用 RuntimeProjectError 的对应变体归因。

### shader prewarm

无验证的 prewarm_shader_variants(&ShaderVariantPrewarmManifest, cache_dir) 返回 ShaderVariantPrewarmReport，并把变体缓存写入给定目录。三个 WGPU 变体分别验证 shader module、render pipeline 或两者；prewarm_shader_variants_with_execution_budget 额外接受 ShaderVariantPrewarmExecutionBudget 和两个 bool，预算/离屏 backend 失败会记录在 report，而不是改变函数返回类型。ShaderPrewarmTemplateSource 暴露 wgsl_source、include content hashes 和 template revision，适合在写缓存前做 provenance 校验。

### 失败与验证

shader manifest 的非法 geometry source、template assembly、WGPU setup 或预算超限都进入 report 的 failure entries；只有 material_surface_shader_prewarm_template_source 直接以 Result<ShaderPrewarmTemplateSource, String> 报告未知 geometry/assembly 错误。缓存根函数只计算路径，不创建目录。验证入口：[API table tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/dynamic_api/tests/api_table.rs)、[session lifecycle](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs)、[session profiles](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/dynamic_api/tests/session_profiles.rs)、[shader prewarm tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/dynamic_api/shader_prewarm/tests.rs)。

## 版本与维护边界

Rust 公开面以对应 mod.rs 的 pub use 为准；私有 mod 中的 helper、pub(crate) 类型和 test-support 导出不构成产品 API。若源码新增/删除 root re-export、改变 feature gate、错误 enum 或 ABI table 字段，应同时更新本页并链接新的 focused test。动态 ABI 变更必须增加 API table 版本并协调 host；不能通过旧字段顺序或兼容别名偷偷扩展 V8。
