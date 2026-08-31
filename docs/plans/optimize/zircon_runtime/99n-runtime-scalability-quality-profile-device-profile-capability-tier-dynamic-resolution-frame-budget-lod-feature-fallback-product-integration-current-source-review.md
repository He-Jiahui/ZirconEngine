---
title: Runtime Scalability、Quality Profile、Device Profile、Capability Tier、Dynamic Resolution、Frame Budget、LOD、Feature Fallback 与 Product Integration 当前源码复核
category: zircon_runtime
report_id: Runtime113
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
refreshes_report: Runtime65
related_code:
  - zircon_runtime/src/core/framework/render
  - zircon_runtime/src/graphics/runtime/render_framework
  - zircon_runtime/src/graphics/tests
  - zircon_plugins/hybrid_gi/runtime/src
  - zircon_plugins/virtual_geometry/runtime/src
  - zircon_editor/src/ui/retained_host/viewport
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/host/render_framework_boundary/mod.rs
  - zircon_app/src
tests:
  - zircon_runtime/src/core/framework/render/backend_types/tests.rs
  - zircon_runtime/src/core/framework/render/view_family.rs
  - zircon_runtime/src/core/framework/render/frame_profile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/frame_profiler.rs
  - zircon_runtime/src/graphics/tests
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/host/render_framework_boundary/mod.rs
  - zircon_editor/src/ui/retained_host/viewport/tests
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/gpu_timing_evidence.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/65-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-review.md
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
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
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
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Tests/Editor/ScalableSettingSchemaTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Tests/Editor/ScalableSettingTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Tests/Editor/ScalableSettingValueTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Tests/Editor/SerializedScalableSettingTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Tests/Editor/SerializedScalableSettingValueTests.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 99n · Runtime Scalability 与 Quality Authority 当前源码复核

## 1. 当前结论

Runtime65 的核心判断在当前源码上仍成立，而且本轮扩大冻结范围后没有发现可把它降级为“仅补几个字段”的证据。Zircon 已经具备可复用的局部构件：viewport quality 值、短锁 profile 编译/提交、capability summary、固定 frame/memory budget、global degrade ladder、延迟 GPU timing profile，以及带 generation、reason、history-reset 标志的动态分辨率控制器。但是这些构件仍未形成统一的产品级 quality authority。

当前生产链仍是：Editor 创建 viewport 时写入硬编码 profile；App 与 PBR viewer 不解析 project/device/user quality；camera 保存 `enabled + scale` 固定值；提交阶段把 camera scale 与 global ladder scale 取 `min`；一份全局 memory budget 和一份全局 ladder 服务所有 viewport；延迟 GPU timing 在 ladder 求值之后合并，只形成诊断，不驱动 controller。请求、能力解析、压力降级和最终生效值之间没有 typed receipt、transaction、generation fence 或 rollback 合同。

本轮账本保持 **0 P0、64 P1 Open、16 P2 Open、48 Gate Fail**。自 Runtime65 基线后，PBR viewer 已迁移到 `WgpuRenderFramework`，GPU timing evidence 也从单帧扩展为 warm-up 加 31 个连续样本的分布证据；这是产品路径与观测能力的真实进步。但 viewer 仍不调用 `set_quality_profile`，GPU sample 仍不进入 dynamic-resolution/budget controller，因此 RT65-P1-27、RT65-P1-33、RT65-P1-61 均不能标为 Partial 或 Closed。

本轮只做 review 和计划记录，没有修改 production、tests、Cargo 或参考源码，也没有运行 Cargo、真实窗口、多 GPU、多刷新率、VRAM/OS pressure、thermal/power、device loss、长时间 hysteresis 或竞争性 benchmark。当前不能宣称 Zircon 的质量系统达到或超过 Unreal。用户要求暂缓 tooling，本报告不安排脚本、生成器或现有工具的优化；未来工具迁 Rust 由独立计划拥有。

## 2. 当前源码冻结与可复现性

### 2.1 Zircon 扩展冻结

本轮用 owner 根目录与明确关键词选择器扩大物理冻结。各组会重叠；去重合计才是本报告的 canonical source set。

| 冻结组 | 选择规则 | 文件 | 行 | 非空行 | bytes | `#[test]` | `#[ignore]` |
|---|---|---:|---:|---:|---:|---:|---:|
| A · 中立 render contracts | `zircon_runtime/src/core/framework/render/**/*.rs` | 267 | 50,268 | 45,476 | 1,759,372 | 476 | 0 |
| B · RenderFramework authority | `zircon_runtime/src/graphics/runtime/render_framework/**/*.rs` | 137 | 20,119 | 18,484 | 771,046 | 194 | 0 |
| C · feature consumers | `zircon_runtime/src/graphics`、HGI、VG 中命中 quality/scalability/LOD/mip/dynamic-resolution/budget/fallback/resolution-scale/frame-time/GPU-timing 的 Rust 文件 | 410 | 155,714 | 145,365 | 5,844,844 | 1,415 | 18 |
| D · App products | `zircon_app/src/**/*.rs` | 191 | 32,669 | 29,859 | 1,230,287 | 547 | 0 |
| E · Editor viewport owner | retained viewport 全目录加两条外部集成测试 | 32 | 3,288 | 2,961 | 118,889 | 44 | 0 |
| F · focused render tests | `zircon_runtime/src/graphics/tests/**/*.rs` 加关键 inline-test owner | 142 | 53,089 | 49,587 | 1,922,070 | 508 | 0 |
| 去重合计 | canonical union | **1,054** | **266,922** | **246,759** | **9,864,458** | **2,771** | **18** |

当前 Zircon 冻结指纹是 SHA-256 `b8aee18e2b3fbfea4310d143795ca9fc821e98569d82dd45e39b4d229310f5e8`。算法沿用 Runtime65：仓库相对路径转 `/` 并排序去重，每个文件计算 lowercase SHA-256，以 `path|hash` 逐行编码，LF 连接且末尾不追加 LF，再对 UTF-8 payload 计算 SHA-256。关键词选择器为大小写不敏感的 `quality|scalability|LOD|mip|dynamic-resolution|budget|fallback|resolution-scale|frame-time|GPU-timing` 语义变体；A、B、D、E、F 是完整 owner 根，不依赖关键词命中。

本冻结是“逐文件纳入和机械扫描”的 currentness 证据，不把 1,054 个文件都虚构成同等深度的手工语义阅读。深度语义结论来自 profile、camera、view family、frame profiler、budget、capability、compile、viewport state、submission、Editor/App/viewer 以及对应测试 owner；feature consumer 大集合用于发现分散质量维度和防止遗漏。后续每个 feature 接入 M6 时仍必须在其独立报告内继续逐文件深挖。

### 2.2 五参考扩展冻结

Runtime65 的 30 个参考文件仍为 26,926 行、1,146,549 bytes，指纹仍是 `f0952bad9a60cc3b367243bb5929a36e1700cb9842285399b57f993f85237e75`，说明核心参考基线没有漂移。本轮增加 5 个 Unity HDRP scalable-setting Editor 测试：

| 参考 | 文件 | 行 | bytes | 直接测试声明 |
|---|---:|---:|---:|---:|
| Runtime65 核心参考 | 30 | 26,926 | 1,146,549 | 2 |
| Unity schema/value/serialized tests | 5 | 392 | 13,947 | 16 |
| 扩展合计 | **35** | **27,318** | **1,160,496** | **18** |

扩展参考指纹是 SHA-256 `395e6f04839b5c6e52ab9696763b35b28ba2c13e9468095037b3da357606a8de`，算法与 Zircon 集一致。新增测试直接验证 schema lookup/missing、level count/name、越界访问、level/override 解析、serialized access 与多对象 mixed-value；它们不是 Zircon 必须照搬的 C# API，但证明 schema、override 和 authoring projection 可以有可执行合同，而不应只依赖字符串名称和构造器习惯。

## 3. Runtime65 基线后的真实变化

| 变化 | 当前证据 | 对账本影响 |
|---|---|---|
| PBR viewer 迁入统一 framework | `scene.rs` 已由直接 `SceneRenderer` 改为创建 viewport、submit/present/capture，并从 framework 查询 timing/stats | 产品边界改善，但没有 production `set_quality_profile`；P1-61 保持 Open |
| GPU timing evidence 工程化 | viewer evidence v2 使用 5 个 warm-up 与 31 个连续 measured samples，校验 generation、timestamp period、pass coverage并报告 invalid/timeout | benchmark 观测改善，但未反馈 quality authority；P1-27、P1-46/64 保持 Open |
| App cadence 增加低功耗状态 | unfocused game 变为 100 ms，mobile/background 有显式 interval | 属于 host cadence，不是 refresh/device/quality budget resolver；P1-14 保持 Open |
| 核心 quality owner 未变化 | 从 `bea1acf91b...` 到当前 HEAD，quality、camera、frame budget、view-family controller、budget、capability、compile、profile setter、viewport record、Editor 默认 owner 均无相关提交变化 | 64 项 P1 和 16 项 P2 没有核心闭合证据 |
| 测试面扩大但仍偏 feature fixture | 当前冻结含 2,771 个 `#[test]` 声明；profile tests 证明 feature 开关与像素/graph 行为，controller tests 证明数学收敛 | 没有 catalog migration、device overlay、multi-viewport isolation、transaction rollback、device loss/thermal/soak 产品矩阵；P1-63 保持 Open |

“测试声明存在”不等于本轮运行通过。本报告没有执行 Cargo；测试数量只描述源码覆盖面。

## 4. 当前实现链与断裂位置

### 4.1 Profile 仍是无版本按值对象

`RenderQualityProfile` 只派生 `Clone/Debug/PartialEq/Eq`，身份仍是 `name: String`；字段是 pipeline override、feature booleans、global texture mip bias、anisotropy、half-resolution sigma、shader/TAA tier 和 Solari 设置。它没有 serde、qualified stable ID、schema version、catalog generation、source/provenance、target class、canonical hash、revision 或 migration。

`RenderFramework` 的公开写入口仍只有 `set_quality_profile(viewport, profile) -> Result`。内部做到了快照 capability/pipeline、锁外 compile、重新加锁验证并提交，这是应该保留的短锁基础；但没有 get/reset/preview/prepare/commit/rollback、取消、last-good receipt 或跨 viewport transaction。viewport 只保存 `Option<RenderQualityProfile>` 和通用 `generation`，stats 只保存一个全局 `last_quality_profile: Option<String>`；更新函数在 context profile 为 `None` 时不清空旧值。

### 4.2 Capability 仍把事实、可用性与 fallback 混在一起

`flagship_baseline_supported` 只检查 offscreen 与 graphics queue，随后 `virtual_geometry_supported` 和 `hybrid_global_illumination_supported` 都直接复用这一 baseline。profile validation 严格拒绝的主要是 anti-alias 和 Solari；`quality_profile_capability_validation_allows_advanced_features_to_degrade` 反而明确断言 VG/HGI 请求可通过后在 runtime plan 中降级。

compile options 会按 backend capability 与 provider availability 关闭高级 feature，这是必要的保护，但没有输出 requested/resolved delta、缺失 requirement、provider/device generation、视觉成本和用户可见 reason。当前也没有 `DeviceProfile`、hardware benchmark、capability tier、power/thermal overlay。因此同一个 `false` 可能表示用户关闭、设备不支持、provider 缺失或 runtime pressure 临时降级。

### 4.3 Budget 与动态分辨率没有闭环

`RenderFrameBudget::reference_1080p_mid()` 仍固定为 14,000 us 和固定 pass 表。`RenderMemoryBudget` 仍固定为 512 MiB transient texture、256 MiB transient buffer、64 MiB staging、1 GiB persistent texture；`warning_count` 只统计四个 pool 是否越线，不保存超额字节、趋势、heap、OS pressure 或 allocation failure。

`BudgetDegradeLadder` 仍是 0.85 scale、0.7 scale、global mip +1、关闭 `ssr`、`ssao`、`contact_shadow`、`bloom_high`，并使用 120 个预算内帧恢复。`bloom_high` 实际关闭整个 Bloom。`RenderFrameworkState` 仍在 viewport `HashMap` 旁只持有一份 `memory_budget` 和一份 `degrade_ladder`，所以任一 viewport 的压力会污染其他 viewport。

`FrameProfiler::write_profile` 先用 CPU/current profile 的 memory warning 推进 ladder，之后才 merge delayed GPU timer；合并后的 GPU over-budget 只增加 profile warning 和 stats。尽管现在可返回多个 `resolved_gpu_profiles`，它们仍未输入 ladder 或动态分辨率 controller。

`RenderDynamicResolutionController`、sample、decision、reason、scope 与 history-reset 标志都存在并有数学单测，但生产代码没有 controller 实例 owner、sample ingestion 或 decision consumer。提交阶段仍直接读取 camera `enabled + scale`，先与 global degrade scale 取 `min` 并回写 camera extract，再用固定 fraction 构造 view family。`resolve_for_viewport_with_dynamic_resolution_decision` 只在定义和单测中出现。

### 4.4 Product integration 仍只有 Editor 固定默认

Editor 的唯一 production caller 仍是 viewport lifecycle：create/recreate 后应用 `editor-viewport-default`，固定 VG=false、HGI=true，并从 `ZIRCON_EDITOR_HYBRID_GI_PROFILE` 的 `OnceLock` 选择局部 HGI preset。resize 通过 destroy/create 后重新套用同一默认，不能证明 project/user/device override 与 history generation 被继承。

`set_quality_profile` 在 App 下只出现在 `entry/tests/profile_bootstrap.rs` 测试；runtime preview 和 PBR viewer 没有 production caller。PBR viewer 已成为真实 framework consumer，但 framework viewport 的 quality 仍是 `None`。App 的 cadence 固定值处理 focus/mobile/background，不读取 monitor refresh、present mode、target frame rate 或质量目标。

### 4.5 精确零搜索

在 `zircon_runtime/src`、`zircon_editor/src`、`zircon_app/src` 和已纳入的 render plugin production Rust 源码上，本轮再次确认以下核心术语没有产品实现：`DeviceProfile`、`device_profile`、`HardwareBenchmark`、`hardware_benchmark`、`Scalability`、`scalability`、`capability_tier`、`thermal`、`battery`、`power_mode`、`target_frame_rate`。这些零结果不能证明未来类型必须采用相同名字，但可以证明当前没有等价 owner 链可供路由。

## 5. 五套参考实现的工程语义

| 参考 | 当前复核到的 owner/合同 | Zircon 必须吸收的工程语义 | 明确不照搬 |
|---|---|---|---|
| Unreal | `FQualityLevels` 完整维度/hash/benchmark result；`UGameUserSettings` validate/load/save/apply/dirty/confirm/revert；DeviceProfile parent/CVar/LOD hierarchy；dynamic-resolution begin/end owner和history reset | catalog/profile/device/user/benchmark 分层，apply transaction，qualified identity，ordered dynamic-resolution lifecycle | 不把 CVar 字符串变成 Zircon source of truth，不把类数量当性能或完整性 |
| Unity HDRP | schema ID + level table + per-use override；serialized tests覆盖缺失、越界与 mixed value；pipeline/frame settings持久化迁移；DR handler维护per-camera状态、user/system scaler及hardware/software fallback | schema化值表、可验证 override、per-camera/per-view state、迁移、双来源 scaler、fallback matrix | 不复制 C# serialization/Editor API 或固定三级/四级数量 |
| Godot | viewport 逐实例拥有 scaling mode/scale/sharpness/mip bias/anisotropy/LOD/MSAA/SSAA/TAA，setter 校验后提交 RenderingServer；ProjectSettings 提供 typed defaults/feature override | per-viewport effective state、即时校验传播、project source、feature override | 不复制 singleton/main-thread API 形状 |
| Fyrox | serializable/reflection `QualitySettings` 用具体 shadow/SSAO/scatter/FXAA/parallax/HDR/bloom 值构造 preset；Editor 通过 renderer settings 应用 | preset 必须解析为完整具体向量，Editor 使用同一 runtime contract，昂贵重配置显式化 | 不以较小 feature 集合作为 Zircon 上限 |
| Bevy | adapter/backend/power preference/required-disabled features/limits/memory hints/fallback adapter 显式协商；upscaling 是 per-view resource并参与 recovery | capability 必须来自真实 adapter/limits，required 与 fallback 分开，资源绑定 view/device generation | Bevy 没有 UE 式完整 scalability product，不据此删减 Zircon 产品层 |

共同点不是“都有 Low/Medium/High”，而是 source、request、capability、override、effective state、lifecycle 与 persistence 各有明确 owner，并能被测试。Zircon 可以采用更紧凑的数据布局、更低成本的采样与更强的并行化，但不能用 silent fallback、固定预算或 global state 制造表面性能优势。

## 6. P0 复核

当前仍无新增 P0。global ladder 的跨 viewport 污染、silent fallback 和固定 budget 会造成明显的产品错误与性能失控，但现有静态证据尚未证明合法公共调用必然导致崩溃、内存破坏、持久化数据损坏或安全边界突破。若后续运行证据证明 global pressure 能破坏关键产品最低质量、安全可读性或资源所有权，再升级严重度；当前保持 P1，避免用猜测抬高 P0。

## 7. P1 当前状态账本

| ID | 当前状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| RT65-P1-01 | Open | profile 身份仍是 `name: String` | qualified `QualityProfileId`、schema version、catalog generation |
| RT65-P1-02 | Open | profile 无 serde/migration | versioned schema、roundtrip、upgrade、downgrade rejection、last-good |
| RT65-P1-03 | Open | 没有 catalog owner | `QualityProfileCatalog` 管 preset/custom/source/hash |
| RT65-P1-04 | Open | preset 不是完整 typed vector | 每个 preset 解析所有注册维度，未映射即报错 |
| RT65-P1-05 | Open | profile 无 source/provenance | project/device/benchmark/user/session 逐字段来源 |
| RT65-P1-06 | Open | profile 无 revision/hash | canonical snapshot/hash/revision 与冲突检测 |
| RT65-P1-07 | Open | setter 只有一次性 Result | preview/prepare/compile/prewarm/commit/rollback/cancel |
| RT65-P1-08 | Open | 无 terminal receipt | requested/effective delta、reason、generation、cost receipt |
| RT65-P1-09 | Open | stats 仅全局名称且可能陈旧 | per-viewport qualified state，`None` 清除或重算 |
| RT65-P1-10 | Open | `DeviceProfile` 精确零命中 | parent hierarchy、selector、cycle/duplicate/unknown validation |
| RT65-P1-11 | Open | adapter/driver/display 无 profile mapping | `DeviceProfileResolver` 消费同代 RHI/display facts |
| RT65-P1-12 | Open | `HardwareBenchmark` 零命中 | workload/build/device/driver/clock/置信信息绑定结果 |
| RT65-P1-13 | Open | thermal/battery/power mode 零命中 | optional telemetry、Unknown policy、hysteresis、permission |
| RT65-P1-14 | Open | 14 ms 固定；App cadence 不是 refresh budget | 从 Runtime57/22 消费 display/refresh/present/latency goal |
| RT65-P1-15 | Open | App 无统一用户覆盖链 | 固定 project < device < benchmark < user < safety overlay |
| RT65-P1-16 | Open | device/display/power 变化无 quality transaction | 可取消 prepare/commit/rollback 与 generation fence |
| RT65-P1-17 | Open | flagship 只要求 offscreen+graphics queue | 每 feature 的 format/limit/queue/provider requirement set |
| RT65-P1-18 | Open | capability class 无决策证据 | `CapabilityTierReceipt` 与 accepted/rejected reason |
| RT65-P1-19 | Open | profile 主要只严格校验 AA/Solari | 覆盖所有 required feature、limit、budget、mutual exclusion |
| RT65-P1-20 | Open | VG/HGI 测试允许 silent degrade | requested/resolved delta、reason、visual/functional cost |
| RT65-P1-21 | Open | provider availability 未绑定代际 | receipt 绑定 provider/device/catalog generation |
| RT65-P1-22 | Open | fallback 是零散 if | required/preferred/fallback/forbidden policy 与 fail mode |
| RT65-P1-23 | Open | capability 与 pressure 都改 boolean | unsupported/degraded/user-disabled 分离 |
| RT65-P1-24 | Open | 关闭 feature 即被视为可接受 | fallback quality floor、visual oracle、product acceptance |
| RT65-P1-25 | Open | 14 ms 与 pass 表固定 | product/viewport/device/refresh/resolution budget profile |
| RT65-P1-26 | Open | CPU/GPU/present/memory 无统一归因 | typed bottleneck classification 与相应措施 |
| RT65-P1-27 | Open | GPU result 在 ladder 后 merge | resolved sample/confidence 先进入 controller，处理 timeout |
| RT65-P1-28 | Open | memory budget 为四个固定常量 | heap/budget/usage、OS pressure、resource domain |
| RT65-P1-29 | Open | warning 只计越线 pool 数 | pool、bytes over、trend、allocation failure、source |
| RT65-P1-30 | Open | 单帧 memory over 即推进 | bounded percentile/EWMA、warm-up、outlier policy |
| RT65-P1-31 | Open | budget 无 source/generation | versioned budget、display/device/source identity |
| RT65-P1-32 | Open | 只有 memory ladder 固定 120 帧恢复 | 统一 attack/release/cooldown/min residency |
| RT65-P1-33 | Open | controller 仅定义/导出/单测 | per-view-family production owner 与 delayed sample ingestion |
| RT65-P1-34 | Open | camera `enabled + scale` 是固定结果 | camera 只声明 min/max/intent，controller 发布 effective fraction |
| RT65-P1-35 | Open | 无 controller create/reset/retire owner | 随 viewport/view/device generation 生命周期管理 |
| RT65-P1-36 | Open | begin/sample/decide/submit/end 未接线 | 并发与延迟下固定 ordered events |
| RT65-P1-37 | Open | history-reset flag 无 production consumer | TAA/upscaler/exposure/history 原子 reset generation |
| RT65-P1-38 | Open | 只有 software fraction | hardware/software DR、upscaler 与 platform fallback matrix |
| RT65-P1-39 | Open | 无 CPU-bound protection | GPU idle 时禁止无效降分辨率，选择 CPU measures |
| RT65-P1-40 | Open | missing/late/device-loss sample 未产品化 | last-good/safe fraction、timeout、recovery rebuild |
| RT65-P1-41 | Open | state 仍只有一份 global ladder | per viewport/view-family/workload scope，显式 group 才共享 |
| RT65-P1-42 | Open | context 复制 global degrade snapshot | pressure 归因 owner/viewport，共享资源单独仲裁 |
| RT65-P1-43 | Open | authored 与 global scale 只取 `min` | typed user cap/controller fraction/safety cap composition |
| RT65-P1-44 | Open | global mip bias 加到所有 material/SSR | texture group/view/importance/residency 协调 |
| RT65-P1-45 | Open | ladder 使用 `ssr` 等字符串 | typed/versioned `FeatureScalabilityId` + owner/dependency |
| RT65-P1-46 | Open | `bloom_high` 实际禁用整个 Bloom | measure 名称与具体 tier/value 一致 |
| RT65-P1-47 | Open | 没有 product 最低质量锁 | competitive/readability/accessibility/minimum floor |
| RT65-P1-48 | Open | 120 帧后恢复不预热 | PSO/resource/history rebuild cost 与 prewarm transaction |
| RT65-P1-49 | Open | shader tier 未覆盖所有 feature，局部 Ultra 折 High | 每 tier 完整映射，未映射失败 |
| RT65-P1-50 | Open | shadow 仍由局部参数拥有质量 | registry 暴露 shadow dimensions/cost，算法仍归 feature owner |
| RT65-P1-51 | Open | reflection/GI 各有局部 preset/budget | 统一 resolution/rays/cadence/history/memory 维度 |
| RT65-P1-52 | Open | particle/volumetric 多为 enable boolean | spawn/update/voxel/slice/sample/async budget dimensions |
| RT65-P1-53 | Open | mip bias 不代表 geometry/terrain/decal LOD | view distance、mesh LOD、terrain density、decal/foliage budget |
| RT65-P1-54 | Open | persistent texture budget 被全局复用 | group/importance/visibility/lease/heap pressure arbitration |
| RT65-P1-55 | Open | quality change 无 dependency graph | requires/conflicts/invalidates/compile/resource dependencies |
| RT65-P1-56 | Open | 固定措施顺序无成本收益模型 | GPU/CPU/VRAM gain、visual cost、recent observed feedback |
| RT65-P1-57 | Open | Editor 默认仍 VG=false/HGI=true/固定预算 | project/device/user policy 与 effective/fallback projection |
| RT65-P1-58 | Open | HGI env 仍由 `OnceLock` 固化 | versioned setting source 与 transaction/diagnostics |
| RT65-P1-59 | Open | resize destroy/create 后重套默认 | qualified policy 继承与新 quality/history generation |
| RT65-P1-60 | Open | App/runtime preview 无 production caller | bootstrap 解析并应用 policy，失败显式 Unavailable |
| RT65-P1-61 | Open | viewer 已迁 framework 但不设 profile | 接同一 policy 或明确 test-only fixed qualified profile |
| RT65-P1-62 | Open | diagnostics 仍是全局 profile name | per-viewport ID/hash/generation/reason/budget/sample |
| RT65-P1-63 | Open | 无 multi-viewport/device/power/rollback/soak 产品矩阵 | 隔离、变化、恢复、history、长时 hysteresis tests |
| RT65-P1-64 | Open | timing evidence 改善但无同质量竞争门 | versioned scene/oracle 与 CPU/GPU/VRAM/percentile/visual gate |

## 8. P2 当前状态账本

| ID | 当前状态 | 差距 | 重构要求 |
|---|---|---|---|
| RT65-P2-01 | Open | `name: String` 同时承担显示与身份 | localized label、stable ID、debug path 分离 |
| RT65-P2-02 | Open | 默认值散落在构造器与产品 | catalog 声明 default/source/reason |
| RT65-P2-03 | Open | us/bytes/fraction/frame 使用裸数 | unit newtypes 与 checked conversion |
| RT65-P2-04 | Open | 0.85/0.7/120 缺适用来源 | policy asset、workload、校准证据 |
| RT65-P2-05 | Open | requested/enabled/effective/degraded 命名混用 | 统一状态词汇和 API |
| RT65-P2-06 | Open | debug 输出不可稳定 diff | canonical ordered snapshot 与 human diff |
| RT65-P2-07 | Open | capability summary 扁平 boolean 扩张 | requirement set 与 typed limit query |
| RT65-P2-08 | Open | budget warning 只有计数 | bounded structured event、dedupe、sampling |
| RT65-P2-09 | Open | reason 未来容易字符串化 | closed enum 加 versioned extension |
| RT65-P2-10 | Open | transition 无 trace span | resolve/compile/prewarm/commit/reset duration |
| RT65-P2-11 | Open | preset 边界没有单一文档源 | schema/catalog 成为未来 Rust tooling 输入；本轮不实现工具 |
| RT65-P2-12 | Open | test fixture 重复手工构造 profile | canonical test builder，禁止成为 production default |
| RT65-P2-13 | Open | benchmark 推荐不可解释 | limiting factor、confidence、source/build identity |
| RT65-P2-14 | Open | fallback diagnostics 无 machine/localization key | receipt machine code，产品层映射文本 |
| RT65-P2-15 | Open | transition audit 无过滤/retention | viewport/profile/device/reason 过滤与 bounded retention |
| RT65-P2-16 | Open | source/reference 快照会漂移 | 本篇双 fingerprint 变化即 recheck |

## 9. 目标 owner 与数据流

绑定架构计划优先于旧的宽泛分层：根包保持 `zircon_app`、`zircon_runtime`、`zircon_editor`；Runtime 内部收敛到 `core/{runtime,framework,manager,math,resource}`，不复活非网络 server package，也不留下 compatibility shim。

| 层 | 拥有内容 | 禁止内容 |
|---|---|---|
| `zircon_runtime::core::framework` | 中立 ID/schema/value/requirement/budget/sample/decision/receipt 合同 | backend、Editor、产品默认、global mutable singleton |
| `zircon_runtime::core::manager` | 稳定的 catalog/resolver/controller/transaction 查询与提交门面 | 复制具体 feature 算法或设备枚举 |
| `zircon_runtime` concrete runtime | device facts 消费、catalog、resolver、per-viewport state、frame controller、feature registry、compile/prewarm/commit | App/Editor 私有默认、CVar/string source of truth |
| `zircon_app` | product target、启动时 project/user policy 选择、Unavailable 处理、持久化 owner 接线 | 私有 quality 算法、直接改 runtime 内部状态 |
| `zircon_editor` | quality asset authoring、requested/effective/fallback 展示、preview transaction | 硬编码 runtime 默认或复制 resolver |

```text
Project Quality Asset -----+
DeviceProfile hierarchy ---+--> QualityProfileCatalog
Hardware benchmark --------+             |
User preferences ----------+             v
Platform/display/power ----+--> DeviceProfileResolver
RHI feature/limit facts ---+             |
                                          v
                                   CapabilityTierResolver
                                          |
                 requested vector + provenance + capability receipt
                                          v
Product/camera intent --> ProductQualityPolicy --> PerViewportQualityState
                                          |                  |
CPU/GPU/present/memory --> FrameBudgetController             |
                                          |                  |
                                          v                  v
                            DynamicResolutionCoordinator + FeatureScalabilityRegistry
                                          |
                           QualityTransitionTransaction
                    prepare -> compile/prewarm -> commit/rollback
                                          |
                              EffectiveQualityReceipt
                 requested/resolved/reason/generations/cost
                                          |
                     Render submission / diagnostics / Editor
```

必须保持六条不变量：

1. Device facts、project/user intent、capability resolution、runtime pressure 与 effective quality 是不同事实，任何层不得无 receipt 地覆盖上一层。
2. 每个 viewport/view family 有独立 quality generation、budget controller 与 history contract；共享 GPU 资源只能经显式 resource group 仲裁。
3. transition 在 prepare 阶段完成 capability/dependency validation、shader/PSO prewarm 与资源预算；commit 原子发布，失败保持 last-good。
4. 动态分辨率只消费绑定 frame/device/view identity 的延迟样本；camera 只声明范围和意图。
5. feature owner 保留算法实现，registry 只协调 typed dimensions、dependency、cost、fallback 和 invalidation。
6. Editor、App、runtime preview、viewer 调用同一 runtime authority，不能各自拥有固定默认或私有降级规则。

## 10. 分层实施里程碑

当前 MVP `00` 仍在进行，本报告只完成 review。实施必须等待 MVP gate 允许，并遵循依赖顺序，不能从 Editor 控件或高级 feature 映射倒序开工。

| 里程碑 | 目标 | 依赖 | 完成证据 |
|---|---|---|---|
| SQ-M0 · Current truth | 固化 current source/reference、owner、零搜索、旧基线 delta | Runtime113、Runtime09A/22/42/45/57 | 双 fingerprint、全账本、门禁状态可重建 |
| SQ-M1 · Schema/catalog | stable profile ID、schema、完整 vector、catalog、migration | Runtime24、45 | roundtrip/unknown/upgrade/hash/last-good tests |
| SQ-M2 · Device resolution | DeviceProfile hierarchy、真实 capability tier、benchmark、overlay | M1、Runtime09A/42/57 | multi-adapter/driver/display/power receipt matrix |
| SQ-M3 · Per-viewport authority | requested/effective state、transaction、receipt | M1-M2、Runtime09C/09H1 | isolation/rollback/prewarm/history generation |
| SQ-M4 · Budget feedback | refresh-aware CPU/GPU/present/memory budget 与统计窗 | M3、Runtime22/09A | delayed sample、CPU-bound、VRAM pressure、hysteresis |
| SQ-M5 · Dynamic resolution | 现有 controller 接入 per-view production owner | M4、Runtime09H1 | ordered events、timeout、resize/cut/device-loss pixel/timing |
| SQ-M6 · Feature registry | shadow/GI/reflection/post/particle/LOD/residency typed dimensions | M3-M5、各 feature owner | dependency/cost/floor/fallback/golden |
| SQ-M7 · Product integration | Editor/App/preview/viewer 统一解析、持久化、应用、展示 | M1-M6 | create/reopen/resize/override/device-change/error/recovery |
| SQ-M8 · Competitive gate | 同场景同 effective quality 同硬件视觉与性能资格 | M0-M7、O11/O14 | image oracle、CPU/GPU/VRAM/frame percentile/soak |

### 10.1 测试推进顺序

1. M1 先写 schema/catalog unit tests：合法 roundtrip、unknown field、upgrade、downgrade reject、canonical hash、last-good。
2. M2 写 resolver table/property tests：parent cycle、selector ambiguity、RHI generation mismatch、Unknown power state、overlay provenance。
3. M3 写 transaction concurrency tests：prepare cancel、compile failure、rollback、stale generation、两个 viewport 隔离。
4. M4-M5 写 deterministic controller tests：delayed/missing/outlier samples、CPU-bound、attack/release/cooldown、history reset ordering。
5. M6 写 feature contract/golden tests：typed ID、dependency、cost/floor、fallback image/numeric oracle。
6. M7-M8 才运行真实窗口、多个 refresh/device class、VRAM/OS pressure、device loss、soak 和竞争性 benchmark。

## 11. 当前门禁状态

| Gate | 状态 | 当前缺口 |
|---|---|---|
| SQ-G01 | Fail | 无 stable ID/schema/catalog generation/hash |
| SQ-G02 | Fail | preset 未解析完整 typed vector |
| SQ-G03 | Fail | 无 schema roundtrip/migration/last-good |
| SQ-G04 | Fail | 无 DeviceProfile hierarchy |
| SQ-G05 | Fail | 无同代 device facts receipt |
| SQ-G06 | Fail | 无 hardware benchmark owner |
| SQ-G07 | Fail | 无固定 overlay 顺序与 provenance |
| SQ-G08 | Fail | display/refresh 变化不重算固定 14 ms budget |
| SQ-G09 | Fail | 无 power/thermal Unknown policy |
| SQ-G10 | Fail | advanced feature requirements 过宽 |
| SQ-G11 | Fail | required/fallback 无 policy 与 receipt |
| SQ-G12 | Fail | unsupported/degraded/user-disabled 混淆 |
| SQ-G13 | Fail | setter 无 transaction/terminal receipt |
| SQ-G14 | Fail | 无跨 profile/pipeline/history last-good 证明 |
| SQ-G15 | Fail | receipt 不存在，无法绑定 generations |
| SQ-G16 | Fail | stats 可保留陈旧 profile name |
| SQ-G17 | Fail | budget 不按 product/viewport/refresh/device 解析 |
| SQ-G18 | Fail | CPU/GPU/present/memory 未分别归因 |
| SQ-G19 | Fail | GPU sample 在 ladder 后 merge |
| SQ-G20 | Fail | 无 bounded window/warm-up/outlier/cooldown controller |
| SQ-G21 | Fail | memory budget 不读取真实 heap/budget/usage |
| SQ-G22 | Fail | allocation failure/OS pressure 无高优先级路径 |
| SQ-G23 | Fail | controller/ladder 不是 per viewport |
| SQ-G24 | Fail | 无 shared resource group receipt |
| SQ-G25 | Fail | camera 固定 scale 仍是 effective source |
| SQ-G26 | Fail | ordered DR frame events 未接 production |
| SQ-G27 | Fail | history reset flag 无原子 consumer |
| SQ-G28 | Fail | 无 hardware/software/upscaler fallback matrix |
| SQ-G29 | Fail | 无 CPU-bound protection |
| SQ-G30 | Fail | timeout/device-loss/recovery 未产品化 |
| SQ-G31 | Fail | ladder 仍用裸字符串 feature ID |
| SQ-G32 | Fail | measures 无 CPU/GPU/VRAM/visual cost contract |
| SQ-G33 | Fail | `bloom_high` 与实际关闭整个 Bloom 不一致 |
| SQ-G34 | Fail | 无 product minimum/readability/accessibility floor |
| SQ-G35 | Fail | 主要 feature domains 未统一映射 |
| SQ-G36 | Fail | registry 尚不存在，owner 协调合同缺失 |
| SQ-G37 | Fail | mip measure 未与 residency/lease/heap 按组协调 |
| SQ-G38 | Fail | recovery 无 PSO/resource/history prewarm |
| SQ-G39 | Fail | Editor 仍硬编码 VG/HGI/default budget |
| SQ-G40 | Fail | recreate 不证明 qualified policy/history inheritance |
| SQ-G41 | Fail | App/preview/viewer 无 production quality caller |
| SQ-G42 | Fail | quality persistence 未接 Runtime45 transaction |
| SQ-G43 | Fail | diagnostics 非 per viewport 且字段不足 |
| SQ-G44 | Fail | 缺 multi-viewport/device/power/rollback/loss/soak matrix |
| SQ-G45 | Fail | 缺同 effective quality GPU golden |
| SQ-G46 | Fail | timing evidence 不等于完整竞争 benchmark receipt |
| SQ-G47 | Fail | 本篇提供 fingerprint，但尚无实现期自动 recheck owner |
| SQ-G48 | Fail | 只能在实现里程碑验证后关闭；本轮仅文档静态校验 |

## 12. 禁止的临时实现

- 不允许新增 `Low/Medium/High/Ultra` 枚举后继续由 feature 私有默认补齐未映射字段。
- 不允许把 module composition profile、shader tier、Editor preset 或 environment variable 冒充 DeviceProfile/quality authority。
- 不允许用 `pub use`、compatibility module、shim trait 或旧路径 wrapper 保留两套 profile owner。
- 不允许继续增加 global ladder、global mip bias、字符串 feature ID 或全产品固定 14 ms/固定 VRAM 阈值。
- 不允许把 unavailable GPU timing 当 0 ms、把 Unknown thermal 当 Normal、把 provider 缺失当用户关闭。
- 不允许先做 Editor quality 面板再回填 runtime transaction；UI 只能投影已稳定的 runtime 合同。
- 不允许用 feature-off 的高帧率与 Unreal 做性能比较；竞争门必须是同场景、同 effective quality、同硬件、同观测定义。
- 不允许在本计划夹带 Python/PowerShell tooling 优化；用户要求的未来 Rust 工具迁移由独立 owner 处理。

## 13. 状态与下一步

| 项目 | 状态 | 当前证据 |
|---|---|---|
| Runtime65 当前源码扩展冻结 | review_complete | 1,054 文件、266,922 行、9,864,458 bytes；SHA-256 `b8aee18e2b3fbfea4310d143795ca9fc821e98569d82dd45e39b4d229310f5e8` |
| 五参考扩展冻结 | review_complete | 35 文件、27,318 行、1,160,496 bytes；SHA-256 `395e6f04839b5c6e52ab9696763b35b28ba2c13e9468095037b3da357606a8de` |
| Severity/status refresh | review_complete | 0 P0 / 64 P1 Open / 16 P2 Open / 48 Gate Fail |
| Production、tests、Cargo 变更 | pending | 本篇 review-only；未执行 Cargo 或产品验证 |

Runtime113 的下一实施入口是 SQ-M1，而不是直接接 dynamic resolution 或重写 Editor 默认。开始 M1 前必须重新核对 MVP gate、Runtime24/45 的稳定身份与持久化合同、Runtime09A 的 device generation 事实，并先写失败测试。完成 M1-M3 前，现有 global ladder 只能作为已知临时底座冻结，不能继续扩展新字符串 step 或产品私有默认。
