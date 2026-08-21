---
title: Runtime Scalability、Quality Profile、Device Profile、Capability Tier、Dynamic Resolution、Frame Budget、LOD、Feature Fallback 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime65
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/core/framework/render/backend_types/quality.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/frame_profile.rs
  - zircon_runtime/src/core/framework/render/view_family.rs
  - zircon_runtime/src/graphics/runtime/render_framework/budget
  - zircon_runtime/src/graphics/runtime/render_framework/capability_summary
  - zircon_runtime/src/graphics/runtime/render_framework/capability_validation
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile
  - zircon_runtime/src/graphics/runtime/render_framework/frame_profiler.rs
  - zircon_runtime/src/graphics/runtime/render_framework/set_quality_profile
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record
  - zircon_editor/src/ui/retained_host/viewport/editor_viewport_render_defaults.rs
  - zircon_editor/src/ui/retained_host/viewport/viewport_lifecycle.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs
tests:
  - zircon_runtime/src/graphics/tests/project_render/render_quality.rs
  - zircon_runtime/src/graphics/tests/render_product_submit/profiles.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge/pipeline_profiles.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge/stats.rs
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/host/render_framework_boundary/mod.rs
  - zircon_editor/src/ui/retained_host/viewport/tests/fake_render_framework.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_runtime/09h2-exposure-color-bloom-dof-motion-blur-ssr-terminal-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/57-platform-host-window-registry-monitor-display-event-loop-application-lifecycle-surface-command-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Scalability.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Scalability.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/GameUserSettings.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameUserSettings.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/DeviceProfiles/DeviceProfile.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/DeviceProfiles/DeviceProfileManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/DynamicResolutionState.h
  - dev/UnrealEngine/Engine/Source/Runtime/SynthBenchmark/Public/SynthBenchmark.h
  - dev/bevy/crates/bevy_render/src/settings.rs
  - dev/bevy/crates/bevy_core_pipeline/src/upscaling/mod.rs
  - dev/bevy/crates/bevy_core_pipeline/src/upscaling/node.rs
  - dev/bevy/crates/bevy_window/src/window.rs
  - dev/Fyrox/fyrox-impl/src/renderer/settings.rs
  - dev/Fyrox/fyrox-impl/src/renderer/mod.rs
  - dev/Fyrox/editor/src/settings/graphics.rs
  - dev/godot/scene/main/viewport.h
  - dev/godot/scene/main/viewport.cpp
  - dev/godot/servers/rendering/rendering_server.h
  - dev/godot/servers/rendering/rendering_server.cpp
  - dev/godot/core/config/project_settings.h
  - dev/godot/core/config/project_settings.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/DynamicResolutionHandler.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/GlobalDynamicResolutionSettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/Settings/ScalableSetting.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/Settings/ScalableSettingSchema.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/Settings/ScalableSettingValue.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/Settings/FrameSettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/GlobalLightingQualitySettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/GlobalPostProcessingQualitySettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/HDRenderPipelineAsset.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 65 · Runtime Scalability、Quality Profile、Device Profile、Capability Tier、Dynamic Resolution、Frame Budget、LOD、Feature Fallback 与 Product Integration 工程化差距

## 1. 结论

Zircon 已经有一批值得保留的局部底座：`RenderQualityProfile`能够把主要feature开关、shader tier、TAA preset、mip bias和各向异性传入viewport；`set_quality_profile`遵循snapshot、锁外compile、重新加锁提交的短锁路径；capability summary、compile option裁剪、frame/memory profile、degrade ladder以及带generation/reason/history-reset语义的动态分辨率控制器也都不是空枚举。现有render product测试还能证明多个profile开关确实改变提交计划或像素结果。

但这些部件尚未构成产品级scalability系统。生产提交仍直接读取camera里作者写死的单一`scale`，`RenderDynamicResolutionController`及delayed GPU sample/decision只出现在定义和单元测试中；固定14 ms frame budget与固定内存阈值不来自display refresh、device、product或用户目标，且memory degrade在GPU timing合并前求值。全框架只有一份global degrade ladder，一个viewport超预算会把所有viewport一起降画质。profile又只是按值存入viewport的无版本结构，缺stable identity、schema、device profile、hardware benchmark、power/thermal overlay、requested/effective差异和fallback receipt。Editor只在viewport创建/重建时注入硬编码`editor-viewport-default`，默认请求Hybrid GI而关闭Virtual Geometry；App/runtime product没有统一应用或持久化路径。

本轮登记 **0项新增P0、64项P1、16项P2和48项验收门禁**。没有把无证据的视觉降级或性能失控虚构成崩溃/数据损坏级P0；global跨viewport污染、动态反馈未接线、silent capability fallback和产品无统一authority仍是必须在高级feature继续扩张前收敛的高优先级结构问题。目标架构是`QualityProfileCatalog + DeviceProfileResolver + CapabilityTierResolver + ProductQualityPolicy + FrameBudgetController + DynamicResolutionCoordinator + FeatureScalabilityRegistry + PerViewportQualityState + QualityTransitionTransaction + EffectiveQualityReceipt`。

本轮只做静态review与计划记录，没有修改production、tests、Cargo、ABI或参考源码，也没有运行Cargo、真实窗口、多GPU、不同refresh rate、VRAM pressure、thermal/power、device loss、长时帧抖动或竞争性基准，因此不能宣称已经达到或超过Unreal。用户已要求暂停tooling优化，本篇不新增脚本、生成器或tooling迁移任务。

## 2. 审查边界、规模与currentness

### 2.1 Zircon物理冻结

| 冻结组 | 文件 | 行 | bytes |
|---|---:|---:|---:|
| Public contracts、resolution、profile与budget定义 | 19 | 6,920 | 280,839 |
| Render-framework validation、state、profiling与fallback | 35 | 7,262 | 315,174 |
| Feature、LOD、residency与diagnostics consumer | 40 | 12,904 | 535,827 |
| Runtime、Editor与App product consumer | 3 | 1,133 | 49,493 |
| Focused external及inline-test-bearing文件 | 99 | 39,172 | 1,562,410 |
| 去重合计 | **196** | **67,391** | **2,743,743** |

Zircon冻结集fingerprint为SHA-256 `97ff36acb69e1398062f3e6fe3362208c9836cf594de49357baf25387347fa80`。算法与Runtime64一致：相对路径转`/`、排序去重，以`path|lowercase per-file SHA-256`逐行编码，LF连接且末尾不追加LF，再对UTF-8 payload计算SHA-256。

冻结时有19个入选working-tree路径带修改标记，主要包括PBR viewer `scene.rs`、Editor viewport测试、volumetric/anti-alias/camera-stack/image metadata/SSR/volume extract、WGPU construct、scene renderer/uniform和frame submission文件。其中PBR viewer正从直接`SceneRenderer`迁移到`WgpuRenderFramework`，但当前工作副本仍未设置quality profile；WGPU construct新增startup options/report与`from_renderer`，没有改变本篇global budget/default owner结论。本篇按当前working copy冻结，所有实现前必须对这19个路径和fingerprint变化重新审查。

### 2.2 参考物理冻结

| 参考 | 文件 | 行 | bytes |
|---|---:|---:|---:|
| Unreal Engine | 8 | 5,893 | 222,024 |
| Bevy | 4 | 2,111 | 81,278 |
| Fyrox | 3 | 1,915 | 72,100 |
| Godot | 6 | 14,088 | 592,149 |
| Unity Graphics / HDRP | 9 | 2,919 | 178,998 |
| 合计 | **30** | **26,926** | **1,146,549** |

参考集fingerprint为SHA-256 `f0952bad9a60cc3b367243bb5929a36e1700cb9842285399b57f993f85237e75`，采用与Zircon冻结集相同算法。参考用途按语义路由：Unreal提供scalability/device/user settings/hardware benchmark与dynamic-resolution owner；Unity HDRP提供schema化分层质量表、per-camera frame settings和动态分辨率实例；Godot提供per-viewport即时质量合同与ProjectSettings；Fyrox提供可持久化具体质量向量及renderer重配置；Bevy只提供显式adapter/features/limits/power协商和per-view upscaling资源，不外推为UE式完整scalability系统。

### 2.3 本轮拥有与明确不拥有

- Runtime65拥有统一quality profile/device profile/capability tier解析、frame-budget反馈、动态分辨率产品闭环、per-viewport有效质量状态、跨feature降级次序和requested/effective receipt。
- Runtime09A继续拥有adapter/device/queue/feature/limit的原始RHI事实、device loss与generation；本篇只消费这些事实，不复制backend discovery。
- Runtime09B、09C、09H1、09H2及各高级render feature报告继续拥有visibility、shader/pipeline、AA/upscaler/history、post-process和具体算法质量。本篇拥有它们如何被统一quality vector和fallback policy协调。
- Runtime22继续拥有clock、frame pacing、present cadence与时间域；本篇消费目标刷新率和CPU/GPU时序来制定budget，不另造display clock。
- Runtime24拥有通用stable identity/generation/owner epoch；本篇只规定profile、device resolution、viewport quality generation如何采用该合同。
- Runtime42拥有module catalog、composition profile与target resolution；本篇拥有render quality product policy，不能把module composition profile冒充device/scalability profile。
- Runtime45拥有通用preferences storage、scope、durability和multi-process冲突；本篇拥有quality schema、overlay order和apply receipt。
- Runtime57拥有monitor/display/window/surface生命周期；本篇消费refresh、surface extent/HDR/display变化并触发预算重算。
- Editor和App报告继续拥有设置UI、project/user配置、启动与产品生命周期；本篇要求它们只通过Runtime authority提交质量变更，不复制控件和bootstrap实现。
- 用户已暂停tooling优化；shader prewarm工具、脚本和未来Rust迁移不在本篇改造范围。

## 3. 当前实现的真实能力与断裂

### 3.1 Quality profile与提交

`RenderFeatureQualitySettings`已有clustered lighting、SSAO、temporal history、bloom、color grading、AA、reflection probe、baked lighting、particle、virtual geometry、hybrid GI、Solari、half-resolution transparency和async compute开关。`RenderQualityProfile`再携带pipeline、mip bias、anisotropy、half-resolution depth sigma、`ShaderQualityTier`、`TaaQualityPreset`和Solari settings；anisotropy也会正规化为1/2/4/8/16。

问题在authority而非字段是否存在。该结构只派生`Clone/Debug/PartialEq/Eq`，没有serde、stable ID、schema/version、source、target class、budget、revision或migration。`RenderFramework`公开面只有`set_quality_profile(viewport, profile)`，没有get/reset/preview/persist/transaction/change receipt。viewport按值保存`Option<RenderQualityProfile>`并递增viewport generation；统计只保留profile name，且上下文profile为`None`时不会清空`last_quality_profile`，可能继续展示陈旧名称。

### 3.2 Capability与fallback

`RenderCapabilitySummary`暴露了多项布尔能力和limit，这是可保留的事实面；但`flagship_baseline_supported`只要求offscreen与graphics queue，virtual geometry和hybrid GI的summary布尔值直接复用这一宽松baseline。profile validation只硬拒绝AA和Solari不满足，测试甚至明确允许请求virtual geometry/hybrid GI后由runtime plan降级。compile options能按provider/capability关闭高级feature、无fragment storage时关闭OIT、按limit决定HZB，但没有把requested、resolved、fallback reason、provider generation和用户可见影响形成持久回执。

### 3.3 Dynamic resolution与frame budget

camera资产里的`RenderDynamicResolutionSettings`只有`enabled + scale`，scale钳制在0.1到1.0，提交直接采用这一author-authored固定比例。`view_family.rs`已存在平方根反馈、min/max/target/max-step/hysteresis、scope、延迟GPU样本、generation/source/reason/history reset等中立类型，但生产搜索没有任何controller实例、sample ingestion或decision consumer；它只被定义、导出和单元测试。

`RenderFrameBudget::reference_1080p_mid`固定为14 ms并分配固定pass预算；`RenderMemoryBudget`固定为512 MiB transient texture、256 MiB transient buffer、64 MiB staging和1 GiB persistent texture。FrameProfiler先按当前CPU profile与memory warning推进degrade ladder，随后才合并延迟GPU timing；GPU over-budget只更新诊断warning，不驱动画质控制。pending GPU profile ring有4项上限是正向底座，但它尚未成为闭环controller。

### 3.4 Global degrade与产品接线

`BudgetDegradeLadder`依次执行0.85 scale、0.7 scale、全局mip +1、关闭`ssr`、`ssao`、`contact_shadow`、`bloom_high`，每个超内存帧前进一步，连续120个预算内帧才恢复。整个`RenderFrameworkState`只有一份memory budget和ladder，frame context对所有viewport复制同一snapshot；因此一个viewport的pressure会降级其他viewport。`bloom_high`实际上关闭整个Bloom，字符串step也没有typed feature owner、severity、scope或receipt。

Editor是唯一明确的production profile caller：创建或resize viewport时应用硬编码`editor-viewport-default`，固定关闭virtual geometry、开启hybrid GI，并使用32/64/16的固定高级预算；环境变量`ZIRCON_EDITOR_HYBRID_GI_PROFILE`通过`OnceLock`只解析一次。viewport resize会销毁重建后重新应用同一profile。App/runtime preview/PBR viewer没有统一project/user/device quality选择与持久化应用链。

### 3.5 零结果也是证据

在`zircon_runtime/src`、`zircon_editor/src`和`zircon_app/src`的Rust生产/测试全集上，对`DeviceProfile`、`device_profile`、`HardwareBenchmark`、`hardware_benchmark`、`Scalability`、`scalability`、`capability_tier`、`thermal`、`battery`、`power_mode`和`target_frame_rate`逐项精确搜索均为0命中。`ShaderQualityTier`确实进入shader variant和froxel quality，但Ultra在froxel映射中折叠为High；shadow、reflection、GI、particle、terrain、decal、LOD density等仍各自拥有不统一的局部quality类型。

## 4. 五套参考实现的语义差异

| 参考 | 已验证结构 | Zircon应吸收 | 不照搬 |
|---|---|---|---|
| Unreal | `FQualityLevels`覆盖resolution/view distance/AA/shadow/GI/reflection/post/texture/effects/foliage/shading/landscape，并有hash、单级映射、benchmark result；`UGameUserSettings`负责validate/load/save/apply/dirty/confirm/revert/hardware benchmark；DeviceProfile有parent/CVar/LOD层级；dynamic-resolution owner有ordered begin/end和history reset | 统一维度、设备层级、用户覆盖、benchmark、apply transaction、动态分辨率owner和可验证生命周期 | 不复制CVar字符串作为Zircon source truth，也不因类名数量宣称等价 |
| Unity HDRP | scalable setting有schema ID、level value和override；pipeline asset持久化platform/default frame/dynamic settings并迁移；frame settings逐camera；dynamic-resolution handler逐camera缓存实例、user/system scaler、hardware/software fallback和previous/current fraction | schema化quality table、per-camera override、迁移、双来源scaler、platform fallback与历史状态 | 不引入C#序列化约束或固定HDRP tier数量 |
| Godot | viewport逐实例拥有scaling mode/scale/sharpness/mip bias/anisotropy/LOD threshold/MSAA/SSAA/TAA等，setter校验并立即提交RenderingServer；ProjectSettings持久化typed default与feature override | per-viewport有效状态、即时验证/传播、project setting source与feature override | 不照搬main-thread singleton或RenderingServer API形状 |
| Fyrox | serializable/reflection `QualitySettings`用具体shadow/SSAO/scatter/FXAA/parallax/HDR/bloom值定义low到ultra；Editor持久化并通过renderer重配置 | preset必须解析为完整具体向量，Editor只提交同一runtime contract，昂贵重配置要显式 | 不把其较小feature面当成最终上限 |
| Bevy | startup settings显式协商backend、power preference、priority、required/disabled features、limits、memory hints、fallback adapter；实际adapter/device事实成为资源；upscaling资源逐view并在recovery清理 | capability输入必须来自真实adapter/limit，required与fallback分开，资源按view/device generation归属 | Bevy没有UE式统一scalability/device profile，不能据此删减Zircon产品层 |

共同规律是“请求的质量”与“设备实际能执行的质量”之间必须有可追踪解析层，而且runtime budget反馈、per-view override、用户持久化和设备变化都不能偷偷改写同一无版本结构。Zircon可以采用更紧凑的数据驱动布局和更低开销算法，但不能通过沉默降级、固定预算或global状态来制造表面性能优势。

## 5. P0审计

本轮没有新增P0。当前确认的问题能造成跨viewport画质污染、帧预算失控、silent fallback和产品配置不可追踪，但静态证据尚未证明合法公共调用会立即导致内存破坏、不可恢复数据损坏、稳定崩溃或安全边界突破。相关问题按P1纳入前置架构里程碑；后续若动态分辨率/history切换的真实GPU验证证明越界访问或device loss不可恢复，再由对应09A/09H1 owner升级，不在本篇预判。

## 6. P1差距与重构项

| ID | 差距 | 当前证据 | 需要重构的内容 |
|---|---|---|---|
| RT65-P1-01 | profile无stable identity | 只有自由字符串`name` | 引入`QualityProfileId`、namespace、catalog generation与不可混淆display name |
| RT65-P1-02 | profile无schema/version | 结构未serde且无version | 建立versioned schema、unknown-field策略、upgrade/downgrade与roundtrip |
| RT65-P1-03 | preset不是完整质量向量 | 大量feature只有bool，局部quality散落 | 以typed dimension registry解析为具体shadow/LOD/post/GI/particle数值 |
| RT65-P1-04 | profile无source/provenance | 无法区分project/device/user/runtime degrade | 记录base、overlay、override、controller及source revision |
| RT65-P1-05 | profile无target class | 无platform/product/display/workload适用条件 | 加入target predicate与明确匹配/拒绝规则 |
| RT65-P1-06 | profile无hash/等价判断 | 只能深比较结构 | 生成稳定canonical hash供cache、dirty和receipt使用 |
| RT65-P1-07 | set API无读取/重置 | trait仅`set_quality_profile` | 提供query、reset-to-source、preview与effective snapshot |
| RT65-P1-08 | apply无事务回执 | 提交成功只返回通用Result | 返回old/new generation、recompiled artifacts、history invalidation和fallback receipt |
| RT65-P1-09 | 无DeviceProfile模型 | 全源码精确搜索0命中 | 建立设备档案ID、parent chain、platform selector与data-driven overrides |
| RT65-P1-10 | 无设备档案继承解析 | Editor用硬编码默认 | 提供cycle/duplicate/unknown key校验和deterministic merge |
| RT65-P1-11 | 无HardwareBenchmark | 精确搜索0命中 | 建立可版本化CPU/GPU workload、result、confidence与推荐tier |
| RT65-P1-12 | 无adapter评分到profile映射 | capability只给布尔摘要 | 用vendor/device/driver/features/limits/memory/benchmark解析初始tier |
| RT65-P1-13 | 无电源/热状态overlay | thermal/battery/power_mode均0命中 | 设计可选platform telemetry输入、hysteresis和用户许可 |
| RT65-P1-14 | 无display/refresh预算overlay | 固定14 ms | 从Runtime57/22消费active display、refresh、present mode和latency goal |
| RT65-P1-15 | 无用户覆盖优先级 | App无统一设置应用链 | 明确project default < device < benchmark < user < session safety的overlay顺序 |
| RT65-P1-16 | 无设备档案热切/回滚 | device/display变化不触发quality transaction | 对adapter/display/power变化执行可取消prepare/commit/rollback |
| RT65-P1-17 | flagship baseline定义过宽 | 只要求offscreen+graphics queue | 每个advanced feature声明真实feature/limit/format/queue/provider requirements |
| RT65-P1-18 | capability class不可解释 | Default/Advanced/Experimental未给决策证据 | 生成`CapabilityTierReceipt`及每项accepted/rejected reason |
| RT65-P1-19 | profile validation覆盖不全 | 只硬校验AA与Solari | 校验所有required feature、limit、resource budget和mutual exclusion |
| RT65-P1-20 | silent downgrade无回执 | VG/HGI可请求后内部关闭 | 形成requested/resolved delta、reason、quality cost与用户可见诊断 |
| RT65-P1-21 | provider存在性未绑定代际 | compile options查询当前provider | receipt绑定provider/device/catalog generation，变化时强制重解析 |
| RT65-P1-22 | fallback无policy级别 | 所有降级由零散if决定 | 区分required/fallback/preferred/forbidden和fail-close/fail-soft |
| RT65-P1-23 | capability与budget混为feature开关 | 缺能力和压力都可能关闭同一feature | 分离unsupported、temporarily degraded、user disabled三类状态 |
| RT65-P1-24 | 无fallback质量等价门 | 关闭feature即视为可接受 | 为fallback声明视觉/功能差异、minimum quality floor与golden gate |
| RT65-P1-25 | frame budget固定1080p mid | default固定14 ms和固定pass表 | 建立按viewport/product/device/refresh/resolution解析的budget profile |
| RT65-P1-26 | budget无CPU/GPU瓶颈分类 | current profile与lagging GPU分离 | 对CPU render/game、GPU queue/pass、present wait分类并选择不同措施 |
| RT65-P1-27 | GPU timing不驱动degrade | ladder在延迟GPU merge前求值 | 以resolved GPU sample及confidence进入controller，处理Unavailable/Timeout |
| RT65-P1-28 | 内存预算固定且非heap感知 | 四个常量阈值 | 绑定adapter heap/budget/usage、OS pressure和resource domain |
| RT65-P1-29 | warning_count丢失严重度 | 只计四项是否越线 | 输出pool、bytes over、trend、allocation failure与pressure source |
| RT65-P1-30 | budget无帧窗统计 | 单帧超限即可推进 | 使用bounded percentile/EWMA/outlier policy与warm-up phase |
| RT65-P1-31 | 无budget change generation | runtime无法判断阈值何时变化 | 版本化budget/source/display generation并写入frame profile |
| RT65-P1-32 | 无controller cooldown/anti-oscillation统一策略 | memory ladder只有120帧恢复 | 统一attack/release/cooldown/min-residency并按措施成本配置 |
| RT65-P1-33 | dynamic controller未接production | 只有定义/导出/单测 | 建立per-view-family owner并在提交前消费delayed GPU sample |
| RT65-P1-34 | camera scale冒充动态分辨率 | 实际只读固定`enabled + scale` | 拆分author min/max/request与controller effective fraction |
| RT65-P1-35 | 无controller lifecycle owner | 没有实例创建/销毁路径 | 随viewport/view-family/device generation创建、reset、retire |
| RT65-P1-36 | 无ordered frame event接线 | Begin/End语义未进入产品 | 定义sample enqueue、decision publish、submission consume的严格顺序 |
| RT65-P1-37 | 无历史reset消费证明 | decision带flag但无人使用 | 与TAA/upscaler/exposure/history owner原子提交reset generation |
| RT65-P1-38 | 无software/hardware scaling策略 | 只形成spatial primary fraction | 明确hardware DR、software DR、upscaler和fallback兼容矩阵 |
| RT65-P1-39 | 无CPU-bound保护 | 所有反馈设计只围绕GPU ms | GPU idle且CPU超限时禁止无效降分辨率，改用CPU向措施 |
| RT65-P1-40 | 无多采样延迟/缺失策略 | sample enum存在但未产品化 | 按GPU query latency、timeout、device loss保持last-good或安全回退 |
| RT65-P1-41 | degrade ladder是global singleton | state只有一份ladder | 按viewport/view family/workload scope持有，显式共享时使用group ID |
| RT65-P1-42 | 一个viewport污染其他viewport | context复制global snapshot | pressure归因到resource owner与viewport，跨view共享资源单独仲裁 |
| RT65-P1-43 | global scale只取min | 与camera scale通过`min`组合 | 用typed composition解释user cap、controller fraction和safety cap |
| RT65-P1-44 | global mip bias跨场景污染 | ladder bias加到profile/SSR | 按texture group/view/material importance与streaming residency协调 |
| RT65-P1-45 | step用字符串feature ID | `ssr`等裸字符串 | 接入typed `FeatureScalabilityId`、owner、dependency和version |
| RT65-P1-46 | `bloom_high`语义错误 | 实际关闭整个Bloom | 措施名称必须对应具体tier/value，禁止名称与效果不一致 |
| RT65-P1-47 | degrade无用户质量下限 | 可一路关闭多项feature | 定义per-product minimum floor、competitive/readability/accessibility lock |
| RT65-P1-48 | recovery不考虑资源重建成本 | 固定120帧后逐级恢复 | 估算PSO/history/resource rebuild cost并预热后提交 |
| RT65-P1-49 | shader tier覆盖不完整 | 主要进入variant，froxel Ultra折叠High | 定义每个tier对所有注册feature维度的显式映射与未映射错误 |
| RT65-P1-50 | shadow quality未统一 | feature私有PCF/size/distance值 | 由feature registry提供shadow scalable dimensions和成本模型 |
| RT65-P1-51 | reflection/GI质量未统一 | 多个局部preset/provider budget | 统一resolution/rays/update cadence/history/memory维度但保留feature owner |
| RT65-P1-52 | particle/volumetric质量未统一 | profile多为enable bool | 映射spawn/update/voxel/slice/sample/async budget并支持CPU/GPU瓶颈措施 |
| RT65-P1-53 | geometry/LOD质量未统一 | mip bias不能代表mesh/terrain/decal LOD | 建立view distance、mesh LOD bias、terrain density、decal/foliage budget |
| RT65-P1-54 | texture mip与residency预算耦合粗糙 | persistent texture budget被全局复用 | 按texture group、importance、visibility、lease和heap pressure仲裁 |
| RT65-P1-55 | quality变更无依赖图 | feature开关零散裁剪 | registry声明requires/conflicts/invalidates/compile/resource dependencies |
| RT65-P1-56 | quality措施无成本/收益模型 | 固定顺序无法适应场景 | 记录预期GPU/CPU/VRAM收益、视觉成本和最近实测反馈 |
| RT65-P1-57 | Editor profile硬编码 | 固定名称、VG false、HGI true、固定budget | 从project/device/user policy解析并显示effective/fallback状态 |
| RT65-P1-58 | Editor环境变量缓存不可刷新 | `OnceLock`只解析一次 | 迁入versioned settings source，变更走transaction和diagnostics |
| RT65-P1-59 | resize通过销毁重建再套默认 | 丢失明确override/history语义 | viewport重建继承qualified policy并生成新quality/history generation |
| RT65-P1-60 | App/runtime preview无统一caller | 生产搜索只有Editor明确设置 | 在product bootstrap解析并应用quality policy，失败不能静默默认 |
| RT65-P1-61 | PBR viewer迁移后未设profile | 当前工作副本使用framework但无apply | viewer作为真实consumer接入同一policy或明确test-only fixed profile |
| RT65-P1-62 | diagnostics只存profile name | 全局`last_quality_profile`可陈旧 | 按viewport发布requested/effective ID/hash/generation/reason/budget |
| RT65-P1-63 | 缺多viewport/设备变化产品测试 | 现有测试多为单feature开关 | 增加隔离、display/device/power变更、rollback、history reset与soak矩阵 |
| RT65-P1-64 | 缺竞争性画质/性能资格 | 无同画质同硬件scalability证据 | 建立版本化场景、质量oracle、CPU/GPU/VRAM/frame percentile与视觉差异门 |

## 7. P2治理与可维护性项

| ID | 差距 | 建议 |
|---|---|---|
| RT65-P2-01 | `name: String`同时承担显示和身份 | 分离localized label、stable ID与debug path |
| RT65-P2-02 | quality枚举默认值散落 | 由catalog声明default及理由，构造器只解析 |
| RT65-P2-03 | 数字阈值缺单位类型 | 使用Milliseconds、Bytes、Fraction、Frames等newtype |
| RT65-P2-04 | 0.85/0.7/120等常量缺来源 | 移入policy asset并附适用workload和校准证据 |
| RT65-P2-05 | feature关闭路径命名不一致 | 统一requested/enabled/effective/degraded词汇 |
| RT65-P2-06 | profile debug输出不可稳定diff | 提供canonical ordered snapshot和human diff |
| RT65-P2-07 | capability summary布尔项增长扁平 | 按feature requirement set和limit query组织 |
| RT65-P2-08 | budget warning仅计数 | 提供bounded structured event与去重/采样 |
| RT65-P2-09 | 动态分辨率原因字符串化风险 | reason使用closed enum并保留versioned extension |
| RT65-P2-10 | quality transition缺trace span | 记录resolve/compile/prewarm/commit/history reset耗时 |
| RT65-P2-11 | preset边界缺文档生成源 | 从schema/catalog生成Editor字段与参考文档，tooling迁Rust后再实现生成器 |
| RT65-P2-12 | 测试fixture重复手工构造profile | 提供canonical test profile builder且禁止进入production default |
| RT65-P2-13 | benchmark结果缺可读解释 | 输出推荐tier的限制因素与置信区间 |
| RT65-P2-14 | fallback诊断缺本地化key | receipt携带machine code，由产品层映射文本 |
| RT65-P2-15 | profile变更缺审计过滤 | diagnostics按viewport/profile/device/reason过滤并设bounded retention |
| RT65-P2-16 | 参考快照可能漂移 | 保留本篇fingerprint、路径和applicability，变化后自动recheck |

## 8. 目标架构与数据流

```text
Project Quality Asset ----+
DeviceProfile hierarchy --+--> QualityProfileCatalog
Hardware benchmark -------+            |
User preferences ---------+            v
Platform/display/power ---+--> DeviceProfileResolver
RHI feature/limit facts --+            |
                                         v
                                  CapabilityTierResolver
                                         |
                       requested quality vector + receipt
                                         v
Product/camera override --> ProductQualityPolicy --> PerViewportQualityState
                                         |                 |
GPU/CPU/frame/memory ---> FrameBudgetController            |
                                         |                 |
                                         v                 v
                            DynamicResolutionCoordinator + FeatureScalabilityRegistry
                                         |
                           QualityTransitionTransaction
                        prepare -> compile/prewarm -> commit
                                         |
                             EffectiveQualityReceipt
                    requested/resolved/reason/generation/cost
                                         |
                     Render submission / diagnostics / Editor
```

关键不变量：

1. RHI capability、device profile、用户意图、runtime pressure和effective quality是五种不同事实，任何层都不能原地覆盖上一层而不留receipt。
2. 每个viewport/view family持有自己的quality generation、budget controller和history contract；共享GPU资源压力通过显式resource group仲裁，不能回退为global singleton。
3. quality transaction在prepare阶段完成capability validation、dependency resolution、shader/PSO预热和resource预算；commit要么原子发布profile/history generation，要么保持last-good。
4. 动态分辨率只消费有frame/device/view identity的延迟样本，Unavailable/TimedOut/device loss有明确策略；camera只声明范围和意图，不写入“动态结果”。
5. 每个feature保留自身算法owner，但必须向`FeatureScalabilityRegistry`声明typed维度、依赖、成本、收益、fallback和invalidations。
6. Editor、App、viewer和未来runtime player都调用同一authority；产品层只能选择、覆盖和展示，不能另建固定默认或私有降级规则。

## 9. 依赖顺序与实施里程碑

| 里程碑 | 目标 | 依赖 | 完成证据 |
|---|---|---|---|
| SQ-M0 · Truth freeze | 固化当前profile/capability/budget/dynamic-resolution/product caller与零搜索 | 当前报告、Runtime09A/22/42/45/57 | fingerprint、owner route、现有测试基线可重建 |
| SQ-M1 · Schema与catalog | 建立stable profile ID、schema、具体quality vector、catalog和migration | Runtime24、45 | roundtrip/migration/hash/unknown-field测试 |
| SQ-M2 · Device resolution | 建立DeviceProfile层级、真实RHI capability tier、benchmark和overlay | Runtime09A、42、57 | 多adapter/driver/display/power解析与fallback receipt |
| SQ-M3 · Per-viewport authority | 建立requested/effective state、transition transaction和receipt | M1-M2、Runtime09C/09H1 | 多viewport隔离、rollback、PSO预热、history generation测试 |
| SQ-M4 · Budget feedback | 建立refresh-aware CPU/GPU/memory budget与统计窗 | M3、Runtime22、09A | delayed GPU sample、CPU-bound、VRAM pressure与hysteresis测试 |
| SQ-M5 · Dynamic resolution | 把现有controller接入per-view-family生产路径 | M4、Runtime09H1 | ordered frame event、missing sample、resize/cut/device loss像素及时序测试 |
| SQ-M6 · Feature registry | 接入shadow/GI/reflection/post/particle/LOD/residency typed维度 | M3-M5、各feature报告 | dependency/cost/floor/fallback及同画质golden |
| SQ-M7 · Product integration | Editor/App/viewer统一解析、持久化、应用和展示 | M1-M6、Editor/App owners | create/reopen/resize/user override/device change/error/recovery闭环 |
| SQ-M8 · Competitive gate | 同场景同画质同硬件验证视觉与性能 | M0-M7、O11/O14 | image oracle、CPU/GPU/VRAM/frame percentile/soak完整证据 |

## 10. 验收门禁

| Gate | 验收内容 |
|---|---|
| SQ-G01 | profile有qualified stable ID、schema version、catalog generation和canonical hash |
| SQ-G02 | Low/Medium/High/Ultra及custom均解析为完整typed质量向量，无未映射维度 |
| SQ-G03 | schema支持roundtrip、unknown field、upgrade、downgrade拒绝和last-good恢复 |
| SQ-G04 | DeviceProfile parent chain检测cycle、duplicate、unknown key和不确定selector |
| SQ-G05 | adapter/device/driver/features/limits/memory事实来自Runtime09A同一device generation |
| SQ-G06 | hardware benchmark绑定workload/build/device/driver/clock状态与置信信息 |
| SQ-G07 | project/device/benchmark/user/session overlay顺序固定且产生逐字段provenance |
| SQ-G08 | display/refresh/present mode变化触发budget/profile重新解析，不沿用陈旧14 ms默认 |
| SQ-G09 | power/thermal输入不可用时有明确Unknown策略，不伪造Normal |
| SQ-G10 | capability requirement逐feature覆盖format/limit/queue/provider并可解释 |
| SQ-G11 | required feature不满足时fail-close；preferred fallback产生typed receipt |
| SQ-G12 | requested、resolved、unsupported、degraded和user-disabled状态不可混淆 |
| SQ-G13 | `set_quality_profile`支持preview/prepare/commit/rollback和terminal receipt |
| SQ-G14 | compile/prewarm失败保持last-good profile、pipeline、history和diagnostics一致 |
| SQ-G15 | effective receipt绑定viewport/device/provider/catalog/profile generation |
| SQ-G16 | profile置空会清除或重算stats，不保留陈旧`last_quality_profile` |
| SQ-G17 | frame budget按product/viewport/refresh/resolution/device解析并记录source |
| SQ-G18 | CPU、GPU、present wait和memory pressure分别归因且选择相应措施 |
| SQ-G19 | GPU delayed sample在ladder/controller求值前合并，Unavailable/TimedOut可测试 |
| SQ-G20 | budget controller使用bounded统计窗、warm-up、attack/release/cooldown和outlier策略 |
| SQ-G21 | memory budget读取真实heap/budget/usage并区分transient/staging/persistent pool |
| SQ-G22 | allocation failure和OS pressure可触发比普通over-budget更高优先级安全路径 |
| SQ-G23 | 每个viewport/view family拥有独立controller，两个viewport互不污染 |
| SQ-G24 | 显式shared resource group才允许跨view仲裁，且receipt列出受影响view |
| SQ-G25 | camera只声明DR范围/意图，effective fraction只能由controller generation发布 |
| SQ-G26 | dynamic-resolution begin/sample/decide/submit/end顺序在并发和延迟下固定 |
| SQ-G27 | resize、camera cut、upscaler change、large fraction jump原子触发history reset |
| SQ-G28 | hardware/software dynamic resolution与upscaler fallback兼容矩阵覆盖平台差异 |
| SQ-G29 | CPU-bound场景不会因无效降分辨率持续损失画质 |
| SQ-G30 | sample timeout/device loss维持last-good或安全值并在恢复后重建controller |
| SQ-G31 | feature scalability ID是typed/versioned owner key，不使用`ssr`等裸字符串 |
| SQ-G32 | 每个措施声明CPU/GPU/VRAM收益、视觉成本、依赖、invalidations和恢复成本 |
| SQ-G33 | `bloom_high`等名称与实际修改的tier/value完全一致 |
| SQ-G34 | product minimum quality floor、readability和accessibility lock不可被generic ladder越过 |
| SQ-G35 | shadow、reflection、GI、post、particle、volumetric、geometry、terrain、texture均有映射 |
| SQ-G36 | feature-specific owner仍拥有算法，registry只协调policy和effective values |
| SQ-G37 | texture mip措施与Runtime09D residency/lease/heap压力按group和visibility协调 |
| SQ-G38 | quality恢复先预热PSO/resource/history，不把恢复尖峰重新解释为降级信号 |
| SQ-G39 | Editor不再硬编码HGI/VG/固定budget，显示requested/effective/fallback差异 |
| SQ-G40 | Editor resize/recreate继承qualified policy并产生新viewport/history generation |
| SQ-G41 | App、runtime preview和PBR viewer均有明确quality policy caller或Unavailable状态 |
| SQ-G42 | project/user quality持久化遵循Runtime45 revision、atomic save和multi-process冲突策略 |
| SQ-G43 | diagnostics逐viewport输出ID/hash/generation/budget/sample/reason且bounded |
| SQ-G44 | 多viewport、adapter/display/power变化、rollback、device loss和长时hysteresis测试通过 |
| SQ-G45 | 同一场景同一effective质量在支持设备上通过GPU像素/数值golden |
| SQ-G46 | benchmark报告CPU/GPU/frame percentile/RSS/VRAM/I/O、置信区间和source/build身份 |
| SQ-G47 | 五套参考路径、Zircon冻结路径和fingerprint变化会把本报告标记recheck |
| SQ-G48 | `git diff --check`、Markdown frontmatter/path/link、finding计数与五份账本路由一致 |

## 11. 状态与产出记录

| 项目 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Zircon 196文件纵向冻结 | review_complete | 2026-08-20 | 67,391行、2,743,743 bytes；SHA-256 `97ff36acb69e1398062f3e6fe3362208c9836cf594de49357baf25387347fa80` |
| 五参考30文件语义对照 | review_complete | 2026-08-20 | 26,926行、1,146,549 bytes；SHA-256 `f0952bad9a60cc3b367243bb5929a36e1700cb9842285399b57f993f85237e75` |
| Severity与owner路由 | review_complete | 2026-08-20 | 0 P0 / 64 P1 / 16 P2；48 gates；共享父owner不重复计数 |
| Production、tests与Cargo变更 | pending | - | 本篇只review；MVP gate下未运行Cargo或产品验证 |
