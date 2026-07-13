---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: core-contract-reverse-dependencies
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
related_code:
  - zircon_runtime/src/core/framework/animation
  - zircon_runtime/src/core/framework/navigation
  - zircon_runtime/src/core/framework/physics
  - zircon_runtime/src/core/framework/bridge
  - zircon_runtime/src/core/framework/project
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/core/runtime/module_lifecycle_observer.rs
  - zircon_runtime/src/plugin
  - zircon_runtime/src/scene/runtime_extension
  - zircon_runtime/src/scene/runtime_hook
  - zircon_runtime/src/scene/navigation.rs
  - zircon_runtime/src/script/vm/host/bridge_host_module.rs
tests:
  - python tools/runtime_domain_dependency_audit.py --pretty --output docs/plans/zircon_runtime/frameworks/05/baselines/2026-07-13-runtime-domain-dependencies-production-only.json
  - python -m unittest tools.tests.test_frameworks_05_layer_direction
  - python -m unittest tools.tests.test_frameworks_03_contract_feature_boundary tools.tests.test_frameworks_03_domain_feature_matrix tools.tests.test_frameworks_03_profile_feature_presets tools.tests.test_frameworks_03_server_feature_boundary tools.tests.test_runtime_domain_dependency_audit
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked
  - cargo check -p zircon_runtime --lib --locked
  - cargo check -p zircon_app --lib --locked
  - cargo check -p zircon_editor --lib --locked
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime -p zircon_plugin_physics_runtime -p zircon_plugin_sdk --tests --locked
  - cargo test -p zircon_runtime --lib core_runtime_module_deactivation --no-default-features --features core-min --locked --jobs 1 -- --nocapture
resolved_at: 2026-07-13
---

# Frameworks05：core/contracts 反向依赖上层域与 facade

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：M0 / target layer dependency classification
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：Frameworks01 必须把 `core/runtime` 与 `core/framework` 物理切为 `zr_kernel` / `zr_contracts`；最低共享原因是 Frameworks05 尚未把 lower-layer contracts、runtime extension hooks 与 facade DTO 从上层 concrete owner 解耦。

## 失败现象与复现证据

Frameworks05 production-only baseline 当前为 2,151 references / 77 edges。按 Frameworks01 锁定的 layer 0–5 + facade 映射分类后：

- lower layer 反向依赖 upper layer：3 edges / 18 refs
  - `core→asset`：6
  - `core→graphics`：1
  - `core→scene`：11
- internal domain 反向依赖 facade：6 edges / 38 refs
  - `animation→plugin`：1
  - `asset→plugin`：4
  - `core→plugin`：15
  - `platform→builtin`：7
  - `scene→plugin`：9
  - `script→plugin`：2

关键 current owners：

- `core/framework/render/framework.rs` 直接使用 `graphics::RenderPipelineAsset`。
- animation/navigation/physics framework contracts 直接使用 asset DTO、`scene::World`、`scene::EntityId` 与 ECS `Resource`。
- `core/runtime` 的 runtime extension/hook state 直接使用 scene `World/SystemStage` 与 plugin lifecycle/registration concrete types。
- platform capability/config 曾直接使用 facade-owned `builtin::RuntimeTargetMode`；该组七条引用现已完成中立 owner 硬切并清零。
- asset/scene/script/animation 的可拆内部域直接使用 facade-owned plugin manifest、component descriptor 与 hook registration types。

因此 `zr_contracts` 目前无法作为纯底层 crate 编译，`zr_kernel` 也会被迫依赖 scene/plugin 上层；若直接开始 Frameworks01 M1，只能形成 Cargo 环、反向依赖或兼容 bridge，均违反计划。

## 当前修复进度（2026-07-13）

本移交已完成十八类最低共享层硬切，并通过编译与定向行为验收：

- framework animation event 已直接使用 canonical `core/framework/scene::EntityId`；shader IDE 记录已直接使用 `core/resource::ResourceLocator`，不再借用 asset `AssetUri` 投影。
- `SceneResource` 与 `SystemStage` 的唯一声明 owner 已硬切到 `core/framework/scene`；旧 `scene/ecs/resource/marker.rs` 和 `scene/ecs/system_stage.rs` 已删除，具体资源存储与调度执行仍归 scene。
- production-only foundation 复测先到 2,146 references / 77 edges；随后删除零调用者的 animation timeline 与 navigation gizmo concrete-asset 投影，把 dynamic component descriptor 唯一 owner 从 plugin facade 硬切到 `core/framework/scene`，并把完整 versioned navigation asset schema 从 asset 域硬切到 `core/framework/navigation/asset`。
- `RuntimeTargetMode` 已从 `builtin/runtime_modules/ids` 硬切到唯一中立 owner `core/framework/platform`。旧定义文件、builtin re-export 与 plugin SDK prelude 投影均已删除，Runtime/App/Editor/全体插件调用点直接使用新路径；不保留 alias 或兼容入口。
- 完整 `AnimationSkeleton/Clip/Sequence/Graph/StateMachine` versioned schema 与 `ZRANIM01` typed binary owner 已从 `asset/assets/animation` 硬切到 `core/framework/animation/asset`。旧目录、asset root/assets re-export 和所有 Runtime/Editor/plugin 旧导入均已删除；asset 域只保留 importer/cache/project integration。当前矩阵为 2,133 / 74；违规边为 `core→asset 0`、`core→graphics 1`、`core→scene 5`，所以 reverse-layer 18→6；facade-inbound 38→25（platform→builtin 7→0，scene→plugin 9→3），总违规 56→31。
- scene runtime hook executable protocol 与 stage plan 已从 plugin/core 硬切到 `scene/runtime_hook`，`WorldDriver` 成为唯一 callback/set owner；旧 plugin re-export、core hook state、CoreRuntime/CoreHandle install/query API 删除，core 只保留 data-only devtools snapshot。Plugin registry 继续验证/排序 scene-owned contribution，不再拥有 hook contract。当前矩阵为 2,139 / 72；`animation→plugin 1→0`、`scene→plugin 3→0`、`script→plugin 2→1`、`core→plugin 15→14`，facade-inbound 25→19，总违规 31→25。
- `InterfaceSlot`、bridge status/transition、diagnostics snapshot、`StrongBridge` 与 `BridgeInvocationTable` 已硬切到 `core/framework/bridge`；依赖 concrete `FrozenBridgeTable` generation/cache 的 `WeakBridge` 保留在 plugin owner。脚本 bridge host 现仅依赖中性 invocation contract 与 `ScriptBridgeMethodDescriptor`，旧 manifest binding/helper/re-export 全部删除，不留兼容入口。当前矩阵为 2,144 / 71；`script→plugin 1→0`，facade-inbound 19→18，总违规 25→24。
- `RenderFramework` 已删除只服务 concrete graphics authoring 的 `register_pipeline_asset(RenderPipelineAsset)`；该行为改为 graphics-owned `WgpuRenderFramework` inherent API。所有中性 Editor/Runtime test double 删除该上层方法，App 的重复 legacy fixture test 退役，plugin executor 集成改用模块启动时已注册的 forward pipeline 验证真实执行。当前矩阵为 2,144 / 70；`core→graphics 1→0`，reverse-layer 6→5，总违规 24→23。
- dependency audit 新增 `use crate::{ ... }` 单行/多行分组导入识别；TDD fixture 从错误的 0 refs 稳定 RED，再准确报告 core 1 + plugin 2。当前源码重算从旧扫描 2,142 / 70 校正为 2,213 / 75，并揭示 `asset/project/manifest/project_manifest.rs` 的第二条 asset→plugin；该精度变化不冒充架构回退或修复。
- asset native importer 通过 `NativeAssetImportCommandHost/Report/Status` 中性命令边界反转依赖，`LoadedNativePlugin` 在 plugin 域实现适配；package asset registry 改为只接收 package id + asset roots + filesystem root，删除 `PluginPackageManifest` 参数和全部旧 API。校正口径下这两刀共清除 native 1 + package manifest 2 三条 facade 引用，当前矩阵 2,213 / 75，asset→plugin=2、facade-inbound=16、总违规 21。
- `AnimationManager` 已删除直接接收 `scene::World` 的 `apply_sequence_to_world`；Runtime 与 Animation plugin 各自在 animation-owned sequence 执行层调用 shared apply helper，不再把 scene execution behavior 塞回中立 manager contract。当前矩阵 2,208 / 75，core→scene=4、reverse-layer=4、总违规 20。
- Core runtime 不再保存 concrete `RuntimePluginBridgeLifecycleState` 或暴露 provider event facade；它只保存 `Arc<dyn RuntimeModuleLifecycleObserver>`，Plugin lifecycle state 实现中性观察器，App 负责类型擦除安装。该刀把 core→plugin 14→4、总违规 20→10。
- `WorldRuntimeExtensionPlan` 与 executable registration storage 归 `scene::WorldDriver`；Plugin registry 只投影 type-erased scene plan，Core 的 `WorldRuntimeExtensionSet`、registry slot、World 参数及 `install_world_runtime_extensions` 全部删除。该刀清除 core→plugin 3 + core→scene 2，总违规 10→5。
- Core devtools plugin catalog 改为 data-only `RuntimeDevtoolsPluginCatalogEntry` 注入；`zircon_app` 从 builtin descriptor catalog 投影 rows，Core 不再调用 Plugin catalog。core→plugin 最后一条清零，总违规 5→4。
- `NavigationManager` / `PhysicsManager` 删除 concrete World execution；Navigation 由 scene-owned `SceneNavigationRuntimeHandle` 驱动，Physics runtime system 解析 plugin-owned concrete manager。core→scene 2→0，总违规 4→2。
- export profile/platform/packaging、project plugin selection 与 `RuntimeProfileId` 的唯一 owner 硬切到 `core/framework/project`；Plugin 旧文件、旧目录声明、root re-export 和 Runtime/App/Editor/插件工作区旧 imports 全部删除。asset→plugin 2→0，最终 current-source matrix 2,290 / 72、reverse-layer=0、facade-inbound=0、总违规 0。
- `tools/tests/test_frameworks_05_layer_direction.py` 完整 19/19 通过，其中十七个聚焦 owner/behavior boundary guards 与两个全移交守卫均为 GREEN。Frameworks03 全部 contract/profile/feature + dependency-audit 回归 41/41 通过。
- Windows 受管编译门全部通过：Runtime `core-min`、Runtime 默认功能、App、Editor，以及 Navigation Runtime / Physics Runtime / Plugin SDK 的 `--tests` 编译。`core-min` 定向模块停用行为测试 2/2 通过。

代码、静态依赖、工作区消费者编译与核心生命周期行为均已验收；本记录已按 failure-handoff 规则迁入来源子计划 `frameworks/01` 并回传 Frameworks01。默认功能 Runtime 定向 lib-test 的首次重编译还暴露了 Graphics SDF 测试私有导入这一无关既存阻断；该阻断不属于本移交，且未以兼容层绕过。

## 最低共享层根因

契约与生命周期 DTO 的 owner 尚未完成中立化：本应由 `core/framework`、`core/runtime` 或 `core/resource` 持有的 entity/world access contract、render pipeline descriptor、plugin lifecycle/registration DTO、runtime target mode 仍由 asset/graphics/scene/plugin/builtin concrete domains 定义。lower layer 为复用这些类型直接导入 upper/facade，破坏了未来 crate DAG。

## 架构修复验收

- `core/framework` 对 `asset`、`graphics`、`scene` 的 production direct references 全部为 0；共享 trait/DTO 必须拥有中立定义，不能通过 re-export 借用上层类型。
- `core/runtime` 对 `scene`、`plugin` 的 production direct references 全部为 0；kernel 只接收 framework/kernel-owned hook、lifecycle、world-access contracts。
- 可拆 internal domains 对 facade-owned `builtin/plugin/dynamic_api` 的 production direct references 全部为 0；必要 DTO 下沉到 kernel/contracts/resource，组装行为留在 facade。
- production-only dependency audit 的 reverse-layer 18 refs 与 facade-inbound 38 refs 均清零，并增加常驻 layer-direction guard，防止 M1 后重新引入。
- Frameworks03 contract purity tests、Frameworks05 dependency audit tests 与受管 Windows `core-min` check 通过；随后回到 Frameworks01 重新执行 M0 dependency classification，证明 `zr_kernel` / `zr_contracts` 可独立切出。

## 禁止临时方案

- 禁止 alias、compat module、旧路径 re-export、facade bridge、feature-only 回避或 test-only bypass。
- 禁止让 `zr_kernel` / `zr_contracts` 依赖 `zircon_runtime` facade 或任何 layer 2+ crate。
- 禁止复制同一 DTO 到 lower/upper 两个 owner，或用字符串/`Any` 特判绕开真实契约迁移。
- 禁止降低 dependency guard、把反向边加入永久 allowlist，或以当前单 crate 可编译冒充 crate DAG 合法。

## 修复结果与回传

- 根因：本应由 `core/framework`、`core/runtime` 或具体上层执行 owner 持有的 DTO、生命周期观察器和执行协议散落在 asset/graphics/scene/plugin/builtin facade，迫使 lower layers 反向导入 upper/facade。
- 架构修复：十八类 owner/behavior 边界一次性硬切到中立 contracts 或正确 concrete owner，删除旧声明、旧 re-export、兼容入口和 lower-layer World/graphics/plugin 执行依赖。
- 验证：production-only 机器基线为 2,290 refs / 72 edges，九组禁止边全部为 0；Frameworks05 19/19、Frameworks03 + audit 41/41；Runtime core-min/default、App、Editor、Navigation/Physics/SDK 编译全部通过；core-min 模块停用行为 2/2 通过。
- 回传：同一记录迁为 `frameworks/01/fixed-2026-07-13-core-contract-reverse-dependencies.md`。Frameworks01 可恢复 M0 crate-DAG 分类与后续物理提取；Frameworks05 其他 M4/M5 接缝仍按原计划独立执行。
