---
related_code:
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/frame_cadence.rs
  - zircon_app/src/entry/runtime_entry_app/frame_capture.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/polling/drain_budget.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_editor/src/core/asset/import_flow/mod.rs
  - zircon_editor/src/core/commands/palette.rs
  - zircon_editor/src/core/editor_event/retention.rs
  - zircon_editor/src/core/gateway/session/protocol.rs
  - zircon_editor/src/core/jobs/limits.rs
  - zircon_editor/src/core/jobs/pump.rs
  - zircon_editor/src/core/jobs/quota_settings.rs
  - zircon_editor/src/core/play/pending_edits/queue.rs
  - zircon_editor/src/core/play/process_backend/output.rs
  - zircon_editor/src/core/recovery/autosave.rs
  - zircon_editor/src/core/settings/defaults.rs
  - zircon_editor/src/core/settings/definition.rs
  - zircon_editor/src/core/settings/registry.rs
  - zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/pipeline/queue.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/save.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/host/layout_persistence.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/capture.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/redraw.rs
  - zircon_editor/src/ui/retained_host/ui_perf/counter_batch.rs
  - zircon_hub/src/process/editor_handshake/wait.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/resend.rs
  - zircon_plugins/net/runtime/src/service_types/diagnostics.rs
  - zircon_plugins/net/runtime/src/transport/reconnect.rs
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/texture_importer/runtime/src/container/dds.rs
  - zircon_plugins/texture_importer/runtime/src/container/ktx/mod.rs
  - zircon_plugins/texture_importer/runtime/src/container/support.rs
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_sdf/validate.rs
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap.rs
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap/decode.rs
  - zircon_runtime/src/asset/assets/texture/upload_support/dds.rs
  - zircon_runtime/src/asset/assets/texture/upload_support/ktx.rs
  - zircon_runtime/src/asset/mesh_sdf_cook/cook.rs
  - zircon_runtime/src/asset/mesh_sdf_cook/request.rs
  - zircon_runtime/src/asset/project/manager/artifact_access.rs
  - zircon_runtime/src/core/framework/foundation/config_manager.rs
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
  - zircon_runtime/src/core/runtime/events/subscriber.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/session/registry/frame_demand.rs
  - zircon_runtime/src/foundation/runtime/config_manager.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/compute_workload.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_construction.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_mip_streaming.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_compute_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/view_projection.rs
  - zircon_runtime/src/graphics/visibility/view_context/build_views.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs
  - zircon_runtime/src/scene/dynamic_scene/asset_reload/reports.rs
  - zircon_runtime/src/scene/dynamic_scene/spawn_task/loader.rs
  - zircon_runtime/src/scene/inspection/snapshot.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/text/model/style.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/optimize/zircon_runtime/09h2-exposure-color-bloom-dof-motion-blur-ssr-terminal-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/optimize/zircon_tooling/20-cargo-package-workspace-feature-dependency-target-graph-build-receipt-review.md
  - docs/plans/optimize/zircon_tooling/21-unsafe-rust-ffi-native-memory-thread-affinity-panic-unload-safety-governance-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/ConsoleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ConfigCacheIni.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Scalability.cpp
  - dev/godot/core/config/project_settings.cpp
  - dev/godot/core/config/engine.cpp
  - dev/bevy/crates/bevy_app/src/task_pool_plugin.rs
  - dev/bevy/crates/bevy_render/src/render_asset.rs
  - dev/bevy/crates/bevy_winit/src/winit_config.rs
  - dev/Fyrox/fyrox-impl/src/renderer/settings.rs
  - dev/Fyrox/editor/src/settings/graphics.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderPipeline/RenderPipelineGlobalSettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/Settings/ScalableSetting.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/Settings/ScalableSettingSchema.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/GlobalLightingQualitySettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/GlobalPostProcessingQualitySettings.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 22 · Magic Constant、Sentinel、Threshold、Timeout、Capacity、Budget 与 Policy 收敛差距

## 1. 结论

ZirconEngine并不是“所有数字都裸写在函数里”。当前源码已经大量使用命名常量、typed budget struct、checked arithmetic与饱和转换；Editor还具备`SettingsRegistry`的scope/schema/default/presentation骨架，Runtime也有`RenderProfileBundle`和持久化`ConfigManager`。这些基础应保留，不能用机械lint把矩阵维数、cube face数、format header、Win32 flag、hash prime、enum ordinal、ABI layout或shader workgroup一律搬到全局常量模块。

真正的工程缺口是**常量已命名，但policy没有形成唯一owner、可配置层、解析快照、消费证据与漂移门**。按tracked `zircon_*` Rust源码、排除明显test/generated/target路径并保守截断首个纯`#[cfg(test)]`后内容的词法盘点，共有11,758个production-like文件。7,224个命名`const/static`分散在1,848个文件；其中名称带timeout、interval、capacity、budget、limit、max/min、count、size、threshold、queue等policy信号的定义有1,791个，分散在800个文件。另有94行直接数值`Duration`构造、60行固定容量操作、504行`MAX`型值、8行`MIN`型值和348行十六进制字面量。

这些命中绝不是1,014项缺陷。逐cluster回读后，大量`MAX`是安全饱和、十六进制是格式或位运算、容量是短小固定输出、range是cubemap或packet bit width；它们属于definition-bound exemption或局部控制常量。需要重构的是：Plugin SDK与loader各自复制256 MiB ABI上限、Mesh SDF限制三份定义、IBL workgroup四份定义、DDS/KTX解析合同跨Runtime与Importer复制、App/Runtime/Editor复制60秒frame-demand clamp，以及产品API用`usize::MAX/u64::MAX/Duration::MAX`表达unlimited、missing、unknown generation或bypass admission。

本篇不重复已有图形、网络、资产、Editor workflow或ABI报告拥有的具体产品缺陷；只拥有跨域Constant/Policy控制面与放置纪律。**没有新增P0，登记36项P1和12项P2**。在同场景、同画质、同平台、同硬件和同失败语义的证据建立前，不能通过提高隐藏常量、关闭预算或用`MAX`绕过限制来宣称性能优于Unreal。

## 2. 审查边界与物理基线

### 2.1 Production-like词法盘点

| 信号 | 行数 | 文件数 | 解释边界 |
|---|---:|---:|---|
| tracked `zircon_*` Rust文件 | 11,758 | 11,758 | 排除明显test/generated/target路径；不是Cargo target可达性结论 |
| 全部命名`const/static` | 7,224 | 1,848 | 包含字符串、flags、schema、layout和算法常量 |
| policy-like命名定义 | 1,791 | 800 | 名称词法筛选；不是缺陷数 |
| 数值`Duration::from_*` | 94 | 58 | 包含命名默认值、struct default与少量直接call site |
| 数值`with_capacity/reserve/resize/truncate` | 60 | 30 | 多数是固定shape或小输出优化 |
| 数值range / `.take(n)` | 8 / 13 | 7 / 9 | cube face、packet width、diagnostic top-N等混合 |
| `MAX`型命中 | 504 | 280 | 饱和转换、all-bits mask、unlimited、missing sentinel混合 |
| `MIN`型命中 | 8 | 8 | 多数是`NonZero*::MIN`或边界表示 |
| 十六进制命中 | 348 | 85 | hash、format magic、bit packing、OS flag与颜色混合 |

policy-like定义按根crate分布为：`zircon_runtime` 875、`zircon_editor` 500、`zircon_plugins` 292、`zircon_runtime_interface` 72、`zircon_app` 29、`zircon_hub` 14、`zircon_runtime_host` 9。这个分布说明policy天然属于多个domain owner；它不支持建立一个跨仓库万能`constants.rs`。

计数方法有意保守。它会漏掉`#[cfg(all(test, ...))]`、macro展开和首个test block之后恢复的production item，也会把同一多行定义只记首行。数字用于建立可复核inventory上界与cluster，不用于证明某一行必须重构。实施M0必须改用Rust语法树/Cargo target closure生成最终manifest。

### 2.2 四级放置分类

| 分类 | 判断标准 | Zircon代表 | 目标位置 |
|---|---|---|---|
| Shared contract | 两个以上crate必须对同一wire/ABI/schema/format/semantic值一致 | native command 256 MiB上限、共享UI wire sentinel、同一artifact schema字段 | 最低共享owner或codegen schema；consumer不得复制 |
| Crate policy | 一个crate拥有默认值、override、clamp、telemetry和失败语义 | Editor job quota、Runtime asset reload budget、Net reconnect policy | owner crate的settings/config/profile模块 |
| Local control | 单一helper/algorithm内部控制，不是产品配置也不跨模块承诺 | 小Vec预分配、局部probe batch、private retry step | 最窄模块private const，必要时说明依据 |
| Definition-bound exemption | 数值就是协议、数学、格式或数据布局定义的一部分 | cube 6 faces、FNV prime、DDS/KTX header、half-float mask、Win32 flag | 紧贴definition；跨consumer时由共享parser/schema生成 |

同一个数字可因语义不同落在不同层。`6`作为cubemap face count是definition-bound；`6`作为UI展示上限是crate/local policy；`256 MiB`作为Plugin SDK wire admission是shared contract；`Vec::with_capacity(6)`只是局部shape优化。重构必须先分类，再移动。

### 2.3 已有owner与本篇不重复范围

| 已有报告 | 已有owner | 本篇新增责任 |
|---|---|---|
| Runtime03 / Editor17 | Runtime config、Editor settings、diagnostics与恢复 | 把分散policy接入统一definition/resolution/receipt，不重写其持久化事务 |
| Runtime04 | asset/resource/serialization预算与格式 | 共享格式常量、raw-limit bypass的跨consumer规则 |
| Runtime08E | Net config未消费、无界ingress与静默drop | `MAX`绕过和reconnect/poll policy进入统一resolved snapshot |
| Runtime09A-09H2 | GPU lifetime、workgroup、quality、固定预算与画质缺陷 | descriptor/shader/executor常量单源和scalability schema |
| Plugins01 / Runtime Interface01 | plugin ABI、foreign output与公开常量表 | shared ABI limit必须由同一schema生成，禁止SDK/loader双份定义 |
| Tooling10/20/21 | test、Cargo target、unsafe与BuildSet | Constant inventory/currentness/evidence进入同一BuildSet，不复制其他gate |

## 3. 必须保留的工程基础

### 3.1 命名常量和typed budget已广泛存在

Editor job pump、asset import、pending edits、event retention、play output和Runtime asset reload都已经用struct表达items/bytes/time，而不是只传裸整数。`JobEventPumpBudget`、`EditorEventRetentionBudget`、`PendingEditQueueLimits`和`AssetReloadLimits`是可迁移基础。问题是默认值与override来源没有共同schema，不是这些type本身应被删除。

### 3.2 Editor Settings已有scope/schema/presentation骨架

`SettingsRegistry`支持User/Project/Session scope、typed `SettingSchema`、default、presentation与resolve precedence；当前production注册点覆盖design tokens、keymap、MRU、locale、viewport snap和四项job quota。应扩展domain registration与generated inventory，不能另造第二套Editor preference store。

### 3.3 Runtime RenderProfile是少数完整typed消费链

App将`RenderProfileBundle`按`RENDER_PROFILE_CONFIG_KEY`写入Core，Runtime bridge反序列化后设置`RenderSubmissionConfig`。这条链已经具有typed value和真实consumer，后续应增加schema/version/source/resolution receipt，而不是降回字符串环境变量或散落常量。

### 3.4 Checked conversion与格式常量多数方向正确

大量`u64::MAX/u32::MAX`来自`try_from(...).unwrap_or(MAX)`、duration饱和、checked storage size失败表示和all-bits mask；DDS/KTX、FNV、half-float与Win32 flag也多为命名局部常量。它们不能按数字扫描结果机械替换。只有当值跨consumer、进入持久/ABI输出、代表隐藏unlimited或改变产品行为时才提升治理层级。

## 4. P1差距：Inventory、Owner、Schema 与 Resolution

### CONST-P1-001 · 没有canonical ConstantUseInventory

1,791个policy-like定义只能通过文本搜索发现，无法回答definition ID、owner、语义单位、适用target、override层、consumer、failure mode、evidence和source hash。Tooling生成`ConstantUseInventory`，将每项标记SharedContract、CratePolicy、LocalControl或DefinitionBound；未分类不允许跨crate提升。

### CONST-P1-002 · 常量没有唯一owner与placement reason

同名值出现在Runtime、Plugin、App、Editor和Interface时，reviewer只能凭路径猜测谁是authority。每项可配置policy声明domain owner；每项shared contract声明最低共享schema owner；每项exemption声明绑定的protocol/algorithm/layout。禁止以“大家都需要”作为放入全局crate的理由。

### CONST-P1-003 · Editor Settings覆盖面远小于实际Editor policy

production中只有6个`SettingDefinition::new`调用点，其中job quota通过一个路径注册四项；公开`editor.*`/`runtime.*` `_KEY`常量只有13个。与此同时Editor有500个policy-like命名定义。不是500项都应暴露给用户，但autosave、retention、job/output、watch/retry、UI performance和workflow budget至少应有明确的hidden engine policy或setting owner，而不是永远编译期固定。

### CONST-P1-004 · Runtime ConfigManager是无schema的`String -> Value`

`ConfigManager`只提供`set_value/get_value/flush`，没有key registry、type、range、unit、scope、restart、migration、secret、owner或unknown-key policy。当前production消费集中于layout与少量typed config；任意JSON可持久化而无法生成完整有效配置快照。建立`RuntimeConfigDefinitionRegistry`并保留typed serde API，raw value仅作迁移/诊断边界。

### CONST-P1-005 · 缺少统一precedence与来源证明

compile default、project setting、user setting、session override、CLI/env、platform profile、device capability和debug override没有统一顺序。Resolved value必须携`source_layer`、definition version、requested value、clamped value、reason和restart/apply状态；同名key不能由最后一次写入偶然获胜。

### CONST-P1-006 · hot apply、restart与generation语义未进入definition

帧节拍、线程池、GPU资源容量、pipeline workgroup、plugin ABI cap和Editor toast lifetime的变更代价完全不同。definition必须声明Live、NextFrame、NextScene、DeviceRecreate、ProcessRestart或BuildTime；apply成功生成新`PolicyGeneration`，失败保持last-good而不是半数consumer读取新值。

### CONST-P1-007 · 没有ResolvedPolicySnapshot

产品启动后无法导出“实际使用了哪些默认/override/clamp”。建立不可变snapshot，按BuildSet、target、adapter、profile、project/user/session generation冻结；Runtime、Editor、Hub和Tooling只消费snapshot或domain projection，不在执行路径再次解析字符串。

### CONST-P1-008 · 没有source-current duplicate/drift gate

当前没有检查shared值被复制、consumer常量与schema不一致、shader host workgroup漂移或默认值改动却未更新evidence。CI生成ConstantDelta：新增、移动、改值、扩大visibility、增加consumer、改变unit/default/limit都需owner review；definition-bound表格变化同时触发parser/shader/ABI parity。

## 5. P1差距：跨模块与跨crate重复合同

### CONST-P1-009 · Native command 256 MiB上限由SDK与loader双份定义

`zircon_plugins/plugin_sdk/src/native.rs`和Runtime `behavior_calls.rs`都定义`NATIVE_COMMAND_MAX_OUTPUT_BYTES_V4 = 256 * 1024 * 1024`并分别校验。它是host/plugin共同ABI admission合同，应由Plugin ABI schema/Interface生成到两端；版本升级需要capability negotiation，不能靠同名文本保持一致。

### CONST-P1-010 · Mesh SDF维度限制在validate/cook/request三份定义

`MIN_MESH_SDF_DIMENSION=4`与`MAX_MESH_SDF_DIMENSION=256`分别存在于asset validation、cook执行和import request解析，且一处使用`u64`、两处使用`u32`。建立`MeshSdfCookPolicy`与versioned recipe；request validation、cook与runtime artifact validation消费同一definition和unit conversion。

### CONST-P1-011 · IBL bake workgroup `[8,8,1]`复制四份

compute executor、graph plan、shader plan和WGPU dispatch各自声明`IBL_BAKE_WORKGROUP_SIZE`。这不是用户setting，但它是host dispatch、shader annotation与graph metadata必须一致的definition-bound contract。由shader recipe/codegen生成host constant与shader specialization，validation比较compiled reflection而不是四个手写常量。

### CONST-P1-012 · Shadow face count与camera阈值在plan/visibility重复

point light 6 faces在shadow plan、view projection和visibility build各自定义；near plane、minimum far plane和up-alignment threshold又在view projection与visibility复制。cube face数可保留为shared geometry definition；camera policy由ShadowViewPolicy拥有，两个consumer不得各算一套矩阵与work量。

### CONST-P1-013 · DDS/KTX格式常量跨Importer与Runtime复制

DDS 128/148-byte header、KTX2 80-byte header、24-byte level index、cubemap face count与多组flags同时存在于texture importer、external cubemap decoder和upload support。格式值本身是exemption，但parser有多个authority。抽出共享、fuzzed container descriptor/parser或生成schema；Runtime不能维护缩小版第二解析器。

### CONST-P1-014 · Render feature descriptor与executor各自复制GPU参数

compute workload descriptor声明HZB、exposure、LUT等workgroup/resource名，具体culler/post-process owner又定义相同参数。RenderFeatureDescriptor应引用compiled shader reflection/recipe ID；不允许descriptor、WGSL生成器和executor三方靠数字约定形成dispatch。

### CONST-P1-015 · 60秒runtime frame-demand clamp有多份authority

App `MAX_HOST_RUNTIME_FRAME_DELAY`、Editor `MAX_EDITOR_RUNTIME_FRAME_DELAY`、Editor window wake delay和Runtime `MAX_RUNTIME_FRAME_DEMAND_DELAY`均为60秒。它是Runtime ABI/host scheduling合同，应在Runtime Interface用明确unit/version发布，并允许host在能力范围内声明更短policy；各端只记录requested/clamped值。

### CONST-P1-016 · Frame capture staging重试在App与Editor复制

App runtime capture与Editor first-frame capture都在局部定义64次staging path尝试并复制相同算法。收敛到原子artifact staging helper，策略包括attempt budget、root、collision identity、cleanup和terminal diagnostic；不能只共享数字仍复制事务。

### CONST-P1-017 · Animation IK queue上限在Runtime与Plugin复制

Runtime animation manager与Animation plugin都定义每World 4,096条pending IK command，并实现相似拒绝。先确定唯一系统owner；若两条实现都保留，容量由共享capability/queue policy投影并分别记录producer、world generation和overflow outcome。

### CONST-P1-018 · Runtime与Interface复制UI默认值

text style与resolved style分别维护默认font size、tab size、font weight等值，IME/context又出现第三个font default。authoring default、wire fallback和resolved default必须区分；共同语义由versioned design/text style schema拥有，不同语义应改名并写placement reason，不能靠同名猜一致。

## 6. P1差距：Sentinel、Unlimited、Timeout 与 Budget

### CONST-P1-019 · `MAX`同时表示unlimited、missing、overflow与all-bits

同一代码库把`usize::MAX`用于unlimited budget、invalid index、unassigned shard、overflow observation和all entries，把`u64::MAX`用于generation unknown、area mask、missing payload、saturated metric。API无法从type区分语义。引入`Limit<T> { Bounded, Unlimited }`、`Option<Id>`、`GenerationHint`和typed mask；序列化sentinel仅留在明确versioned wire边界。

### CONST-P1-020 · Resource streamer默认以`u64::MAX`关闭预算

streamer construction与mip streaming多处把resident/upload max设为`u64::MAX`。这使“默认无约束”进入产品资源路径，且诊断无法区分未配置、无限、设备上限或暂时bypass。默认必须来自platform/quality memory profile并受adapter budget、resident reserve和overcommit policy约束。

### CONST-P1-021 · Scene与asset convenience API通过`MAX`绕过admission

scene reload、spawn task、prepared scene、project document和scene asset load的无后缀API把items/bytes/time/raw payload limit传为`usize/u64/Duration::MAX`。方便API不应等价于unbounded；改为消费ambient `SceneLoadPolicy`，只有offline trusted tool在显式capability下请求Unlimited并生成receipt。

### CONST-P1-022 · Network helper用`usize::MAX`绕过帧预算

Net diagnostics先`poll_worker_ingress(usize::MAX)`，Reliable UDP convenience resend也以`usize::MAX`作byte budget。具体无界队列/静默drop由Runtime08E拥有；本篇要求所有helper省略参数时仍消费resolved NetBudget，不允许“default method”绕开有界实现。

### CONST-P1-023 · UI与Editor用最大整数模拟invalid index

binding inspector把`None` selection改成`usize::MAX`传给mutation helper，command palette与notification projection也用最大值制造不存在的索引/limit。使用`Option<SelectedIndex>`、validated index和`VisibleLimit::All`；错误输入必须返回typed outcome，不依赖越界自然失败。

### CONST-P1-024 · GPU submission token以`u32::MAX`模拟missing

mesh draw build将缺失submission token编码为`u32::MAX`，随后多次相等判断并对低16位打包。改为`Option<SubmissionToken>`直到明确GPU packing边界；packing schema保留reserved value、range admission和decode parity，避免合法token增长后撞sentinel。

### CONST-P1-025 · Revision/generation hint以`u64::MAX`模拟unknown

scene inspection、world sync、asset reload排序与query cache分别用`generation != u64::MAX`或`unwrap_or(u64::MAX)`表达无hint/最新/未缓存。建立`GenerationQuery::{Exact, Latest, Any}`和`UninitializedGeneration`状态；排序fallback与协议sentinel不能共享一个裸整数。

### CONST-P1-026 · Export产品轮询仍直接`sleep(25ms)`

Export wizard执行child process时每轮poll后固定睡25ms。它没有wake handle、deadline、cancel latency budget、backoff或实际poll cost统计。接入ProcessSupervisor/event notification；fallback poll interval属于ExportProcessPolicy并在terminal receipt记录poll count、wait、cancel latency。

### CONST-P1-027 · retry/debounce/backoff各自实现且缺少共同outcome

asset refresh使用6次、50ms到2s退避，surface present使用8ms到250ms，Hub handshake为250ms/10s，module readiness与network accept又固定1ms。值可以各属domain，但策略形状应共享`RetryPolicy`语义：attempt/deadline/backoff/jitter/cancel/retryable class/last error；不能只返回最终失败字符串。

### CONST-P1-028 · App frame cadence是编译期固定产品策略

interactive约16.666667ms、headless 16ms、background 1s，unfocused game与mobile foreground直接复用interactive。当前没有显示器refresh、present mode、power mode、platform lifecycle、user frame cap或thermal policy输入。建立`FrameCadenceProfile`，分离simulation tick、render demand、present/vsync和background wake。

### CONST-P1-029 · Editor event retention表是硬编码产品数据

durable replay、frame local、latest state等类别的record/byte/age预算在`retention.rs`直接构造，跨度从128条/1 MiB/2秒到16,384条/64 MiB/24小时。该表需要stable category ID、settings/admin override、memory pressure降级、eviction reason和snapshot provenance；不是简单把数字改成pub const。

### CONST-P1-030 · Editor核心budget没有接入既有Settings authority

asset import 4,096/4 MiB/5分钟、job pump 64/1ms、pending edits 4,096/4 MiB/30分钟、play output 1,024/4 MiB/2ms与autosave 300秒都在owner内固定。默认值可保留在owner crate，但需要definition registration、project/user/session policy、clamp和diagnostic projection；UI只显示resolved值，不复制默认。

### CONST-P1-031 · 图形quality与memory budget没有统一scalability schema

专项报告已登记大量固定atlas/page/probe/ray/LUT/sample预算，本篇不重复数值缺陷。横向缺口是这些值没有由QualityTier + PlatformCapability + AdapterBudget + ViewFamily生成同代`RenderPolicySnapshot`；修改单一常量不能证明画质、显存、GPU时间与fallback仍闭合。

### CONST-P1-032 · diagnostics固定Top-5/Top-10限制不可追溯

profiling export与hotspot路径直接`.take(5/10)`，Hub/build summary也有固定行数截断。展示上限是合理local policy，但导出artifact必须记录total、emitted、truncated和policy ID；机器消费格式不能因UI top-N静默丢失完整evidence。

### CONST-P1-033 · 固定Vec capacity绑定隐式catalog cardinality

UI perf counter batch预分配52、platform diagnostics预分配29、Editor showcase catalog预分配70。预分配不是缺陷，但数字与字段/catalog增长脱节。优先由iterator `size_hint`、generated descriptor count或small-array shape推导；仅在benchmark证明必要时保留private capacity constant。

### CONST-P1-034 · 路径collision attempt是隐藏事务策略

UI asset local-copy最多尝试1,000个名字，frame capture最多64个staging名字。`exists()`后写入还存在TOCTOU语义，调高次数不解决原子占位、并发和crash cleanup。由ArtifactStagingTransaction执行create-new/reservation，attempt policy只作故障预算并记录collision evidence。

### CONST-P1-035 · diagnostics只报value，不报definition/source/generation

现有diagnostic line能显示queue capacity、batch bytes、render stats和platform capability，但无法证明值来自compile default、project override、device clamp还是fallback。每个policy observation包含DefinitionId、requested/resolved、source layer、generation、consumer和clamp/fallback reason，并关联BuildSet。

### CONST-P1-036 · 默认值没有workload-bound性能资格

2ms pump、64 events、4,096 queue、256 MiB output、8x8 workgroup或固定atlas budget都可能在某平台合理，但当前没有共同场景、规模、P50/P95/P99、overflow、memory pressure、power和quality结果。每个性能敏感policy绑定BenchmarkScenario与EvidenceReceipt；禁止通过设为Unlimited、增大容量或减少检查获得无语义对齐的“更快”。

## 7. P2长期能力

| ID | 能力 | 价值与边界 |
|---|---|---|
| CONST-P2-001 | 强类型unit与quantity | `Bytes/Items/Frames/Hertz/Duration/PixelExtent/WorkgroupSize`减少单位混用；不包装矩阵维数等显然局部值 |
| CONST-P2-002 | 开发CVar/console overlay | 借鉴Unreal CVar做受权限、flags、cheat/shipping gate与source记录的实时调参，不让console成为第二authority |
| CONST-P2-003 | Project/User/Platform/Device policy layer | generated schema统一层级、迁移、lock与override；domain仍拥有语义 |
| CONST-P2-004 | ScalableSetting schema | low/medium/high/ultra/cinematic及custom tier按schema ID解析，缺项/长度漂移在cook时失败 |
| CONST-P2-005 | Shader reflection constant parity | 从compiled shader反射workgroup/layout/specialization，host dispatch与descriptor自动验证 |
| CONST-P2-006 | Sentinel elimination lint | 对新增`MAX` equality、`unwrap_or(MAX)`、`Duration::MAX`参数做语义review，不禁止合法饱和和mask |
| CONST-P2-007 | Duplicate contract detector | AST/IR比较同名同值及同consumer schema，生成候选而非自动搬迁 |
| CONST-P2-008 | Policy migration/deprecation | key rename、unit变更、default变更和removed value有versioned migration与warning expiry |
| CONST-P2-009 | Runtime policy diff | 两个session/build/profile导出resolved diff，解释画质、性能、内存和行为差异 |
| CONST-P2-010 | Adaptive budget controller | 在明确上下界与稳定性证明内根据GPU/CPU/memory/power反馈调整，不用自适应掩盖容量不足 |
| CONST-P2-011 | Fleet/remote policy签名 | 长期支持受信远程override、rollout和rollback；不允许未签名配置改变ABI或安全上限 |
| CONST-P2-012 | Policy recommendation service | 基于benchmark/device class提出建议，用户/项目确认后生成可审计配置，不把启发式自动写成source truth |

## 8. 目标架构

```text
Rust AST + Cargo Target Closure + Shader/ABI/Format Schema
                         |
                         v
               ConstantUseInventory
      (owner / class / unit / consumers / source hash)
                         |
        +----------------+----------------+
        |                |                |
        v                v                v
 SharedContract    CratePolicy      DefinitionBound
 ABI/schema/codegen setting/config  format/math/layout
        |                |                |
        +----------------+----------------+
                         |
                         v
              Policy Definition Registry
    default / range / scope / apply / migration / evidence
                         |
                         v
 Project + User + Session + CLI + Platform + Device Layers
                         |
                         v
               ResolvedPolicySnapshot
  requested / resolved / source / clamp / generation / BuildSet
                         |
                         v
         Typed Runtime / Editor / Hub / Tool Consumers
                         |
                         v
      PolicyObservation + PerformanceEvidenceReceipt
                         |
                         v
          ValidationSet / Qualification / Release Gate
```

边界原则：

1. Shared contract只容纳必须跨crate一致的schema/ABI/format语义，不容纳所有默认值。
2. Crate policy由实际producer/consumer owner维护；Tooling只维护inventory、resolution protocol和gate。
3. Local control保持private，只有出现第二consumer、产品override或evidence需求时才提升。
4. Definition-bound值贴近协议、算法、shader或layout；跨语言时由同一schema生成，不从通用config读取。
5. `Unlimited`、`Latest`、`Missing`、`AllBits`和`Saturated`是不同语义，必须在type或wire schema中区分。
6. 修改default不等于优化；policy变化必须与功能、画质、失败语义和性能证据同代。

## 9. 参考引擎对照

### 9.1 Unreal：CVar、Config与Scalability是不同层

Unreal `Scalability.cpp`用`TAutoConsoleVariable`登记quality group、默认值、可读说明和flags，并通过Scalability ini load/save与console command管理；`FConfigCacheIni`又处理platform/config层。可借鉴的是definition metadata、flags、层级与runtime调参，不是复制全局单例或让每个内部布局值都变CVar。

### 9.2 Godot：ProjectSettings登记default、range、enum与restart语义

Godot `_GLOBAL_DEF`在首次注册时写default，并记录initial value、builtin order、basic/internal、restart-if-changed与property hint；Engine对physics tick、max steps、max FPS等再做typed validation和runtime apply。Zircon已有SettingSchema骨架，应补齐domain registration、override层和apply generation。

### 9.3 Bevy：typed Resource贴近consumer

Bevy `WinitSettings`将focused/unfocused update mode建模为Resource，`TaskPoolOptions`拥有线程分配policy，`RenderAssetBytesPerFrame`由主世界extract到render-world limiter。它没有把所有值塞入一个config registry，而是让typed policy贴近plugin/system消费。Zircon应采用同样的owner局部性，同时补上持久化、source provenance和企业级qualification。

### 9.4 Fyrox：可序列化/反射QualitySettings进入Editor与Renderer

Fyrox `QualitySettings`集中point/spot shadow、CSM、SSAO、HDR等质量选择，提供ultra/high/medium等profile；Editor `GraphicsSettings`直接包含并持久化它，Renderer消费同一type。Zircon需要更强的平台/设备/BuildSet身份，但不应让Editor再复制一套render常量。

### 9.5 Unity Graphics：Global Settings asset与ScalableSetting schema

Unity Graphics用`RenderPipelineGlobalSettings` ScriptableObject关联pipeline和项目级设置；HDRP `ScalableSetting<T>`携schema ID与分级值，lighting/post-process quality数组还有range metadata和Editor serializer。Zircon可借鉴versioned quality schema、asset owner和Editor/runtime共享数据，但必须避免数组长度/默认值成为新的隐式合同。

## 10. 重构里程碑

### M0 · Truth Freeze

- 用Rust AST、Cargo target closure和shader/ABI schema生成ConstantUseInventory；
- 固定本篇11,758文件/1,791 policy-like定义的文本baseline及误差说明；
- 将已有domain finding映射到ConstantUseId，不复制P0/P1 owner。

### M1 · Placement 与 Shared Contract硬切

- 为每项候选完成四级分类与owner；
- 先收敛native output cap、Mesh SDF、IBL workgroup、shadow、DDS/KTX、frame demand等明确重复合同；
- 删除consumer副本，禁止保留deprecated alias或双写同步期。

### M2 · Policy Definition 与 Resolution

- 扩展Editor Settings并建立RuntimeConfigDefinitionRegistry；
- 定义scope、unit、range、apply mode、migration、security和precedence；
- 生成ResolvedPolicySnapshot和PolicyGeneration。

### M3 · Sentinel 与 Unlimited类型化

- 将public/internal API中的unlimited、missing、latest、unknown generation迁移为enum/Option/newtype；
- wire/format reserved value只在encode/decode边界存在；
- scene/asset/network convenience API默认消费有界ambient policy。

### M4 · Runtime、Editor 与 Graphics接线

- 接入frame cadence、jobs/import/play/autosave/retention/retry/process supervision；
- 构建RenderPolicySnapshot，host/shader/descriptor从compiled recipe获取shared常量；
- memory pressure、device recreate和last-good generation形成明确状态机。

### M5 · Observation 与 Tooling Gate

- 输出definition/source/requested/resolved/clamp/generation/consumer；
- CI执行duplicate/sentinel/schema/shader parity和ConstantDelta owner review；
- policy artifact绑定BuildSet/ValidationSet，unknown/unconsumed definition失败。

### M6 · Qualification 与 Performance

- 为关键default建立同场景、同画质、同平台、同硬件benchmark矩阵；
- 验证overflow、deadline、cancel、memory pressure、device tier、hot apply与rollback；
- 只有EvidenceReceipt current且无语义降级时，才能比较或宣称优于Unreal。

## 11. 验收门

1. 最终Cargo target的每个policy-like definition都有稳定ID、owner、分类、unit和source hash。
2. SharedContract不存在consumer复制；SDK/loader、host/runtime、shader/CPU由同一schema/codegen生成。
3. CratePolicy保持在owner crate，LocalControl不因“统一”被提升到跨crate模块。
4. Format/layout/math/enum/OS flag等exemption有绑定reason，不进入runtime可配置层。
5. Editor可配置policy进入现有Settings scope/schema/persistence；不得出现第二套Editor store。
6. Runtime config key有type、range、scope、migration、apply mode和unknown-key policy。
7. ResolvedPolicySnapshot记录default/override/platform/device/clamp与BuildSet/PolicyGeneration。
8. live apply按NextFrame/Scene/Device/Process规则原子发布，失败保持last-good。
9. `Unlimited/Missing/Latest/AllBits/Saturated`在type层可区分，业务API不再猜`MAX`语义。
10. scene/asset/network的无参方便API仍有items/bytes/time/depth admission。
11. GPU workgroup、buffer stride与shader specialization通过compiled reflection parity。
12. DDS/KTX及其他共享格式只有一个parser/schema authority，并有malformed/fuzz corpus。
13. retry/poll policy记录attempt、deadline、jitter、cancel、last error与terminal outcome。
14. diagnostics top-N记录total/emitted/truncated/policy，机器artifact可请求完整数据。
15. policy diagnostics包含DefinitionId、source layer、requested/resolved、consumer与generation。
16. ConstantDelta能阻止未登记新增、shared duplicate、unit/default漂移和无owner visibility扩大。
17. 每项性能敏感default有场景、规模、P50/P95/P99、memory/quality/overflow evidence。
18. 调高limit、使用Unlimited或减少validation不能单独构成性能通过。
19. 已有domain P0/P1保持原报告owner；本篇控制面不得掩盖或降格具体功能缺陷。
20. 所有文档、manifest、snapshot和evidence引用当前source/build；source变化后自动要求requalification。

## 12. 本轮证据与限制

- 完成tracked `zircon_*` production-like Rust的命名常量、duration、capacity、range/top-N、MAX/MIN、hex、sleep/retry/timeout/budget词法盘点；
- 回读Runtime/App/Editor配置骨架、frame cadence、job/import/play/retention、scene/asset convenience API、network unbounded helper、Plugin ABI、Mesh SDF、IBL、shadow、DDS/KTX与resource streamer代表owner；
- 读取Unreal Console/Config/Scalability、Godot ProjectSettings/Engine、Bevy typed Resource、Fyrox QualitySettings与Unity Graphics Global/Scalable Settings参考；
- 明确排除tests/generated/vendor与`dev/`参考源码，不把普通0/1、矩阵维数、enum ordinal、ABI layout、format magic、hash prime、OS flag机械计为finding；
- 没有修改production、tests、manifest、workflow或reference engine source；
- 没有运行Cargo、产品进程、GPU capture或benchmark。既有Editor、Hub、WOC与plugin lock阻断未重复执行；
- 当前工作树包含其他Session在途源码变更，实施前必须重新生成AST/target-aware inventory，因此`source_recheck_required: true`。

本轮结论不是“把所有数字变成配置”。工程级收敛要求每个值处在正确层：共享合同单源生成、crate policy可解析可观测、局部控制保持局部、definition-bound常量紧贴定义。只有这样，性能调优才会改变一个可证明的产品策略，而不是在多个文件里追逐相同数字。
