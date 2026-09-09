---
related_code:
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/src/lib.rs
  - zircon_plugins/terrain/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
implementation_files:
  - zircon_plugins/ai/runtime/src
  - zircon_plugins/animation/runtime/src
  - zircon_plugins/navigation/runtime/src
  - zircon_plugins/net/runtime/src
  - zircon_plugins/physics/runtime/src
  - zircon_plugins/sound/runtime/src
  - zircon_plugins/particles/runtime/src
  - zircon_plugins/terrain/runtime/src
  - zircon_plugins/zr_vm_language/runtime/src
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_plugins/01
tests:
  - zircon_plugins/ai
  - zircon_plugins/animation
  - zircon_plugins/navigation
  - zircon_plugins/physics
doc_type: module-detail
title: 运行时插件族
status: source-audited
---

# 运行时插件族

本页按“包 -> 子系统 -> Rust 入口 -> 当前状态”说明主要运行时插件。示例 API 名均来自各 runtime crate 的 `lib.rs` 导出；实际使用前仍应检查 crate feature、目标模式和 capability。

## AI

`ai`（experimental）提供行为树、编译黑板、感知快照与 agent tick。行为树包含 selector/sequence/parallel、decorator、service、task、condition/observer abort、节点 catalog、编译和执行栈；黑板使用 schema 编译的稠密布局、store 与 observer；感知包含 source/receiver 组件、sight/hearing stimuli、扫描和适配层。`DefaultAiManager` 负责 schema、tree instance、参数、perception 和 tick LOD。

公开入口包括 `DefaultAiManager`、`AiBehaviorTickLod`、`AiModule`、`AiDriver`、`module_descriptor(_with_manager)`、`plugin_registration()`、`runtime_plugin()` 和 `runtime_plugin_descriptor()`。系统锚点包括行为 tick 与感知 tick，事件包括 agent tick、BT node result、behavior debug snapshot 与 hearing stimulus。

能力状态：感知为 complete；主 AI、行为树和黑板为 partial。`zr_vm_language` 的 `script.behavior.v1` 与 physics 的 `physics.query.v1` 都是可选依赖，缺失时对应集成功能不可用，但基础执行器仍可工作。

## Animation

`animation`（beta，主能力 partial）实现 clip 编译与采样、Hermite/通道插值、pose buffer/pool、层混合、骨骼目标表、动画图编译/求值、状态机与多 layer、transition interruption、IK、avatar mask 和 GPU skinning palette 双缓冲。运行时 pipeline 处理直接 clip、graph cache、状态机步进、参数应用、pose apply 和事件。

常用 Rust API：

```rust
use zircon_plugin_animation_runtime::{
    AnimationClipEvaluator, AnimationEvaluationPipeline, PoseBuffer,
    compile_animation_graph_runtime, compile_animation_state_machine_runtime,
    LookAtJob, TwoBoneIkJob, SkinningPalette,
};
```

还导出 `CompiledAnimationClip`、`CompiledAnimationGraph`、`CompiledAnimationStateMachine`、`TransitionDesc`、`AvatarMaskAsset` 以及 `register_runtime_system()`。`animation_graph` 和 `timeline_sequence` 是 editor authoring 包；timeline event track capability 仍 partial。

## Navigation

`navigation`（beta，partial）包含 NavMesh 烘焙、tile/dirty bake、路径查询、agent movement、动态 obstacle、modifier、off-mesh link/bridge、overlay frame 与 editor bake 工具。`navigation/native` 提供 Recast/Detour FFI、tile cache、crowd 和 fallback query；它是后端实现，不等同于通用 dist crate。

`DefaultNavigationManager` 是默认 driver，`SceneNavigationRuntimeHandle` 接入场景，模块同时注册 `NavigationManager`。公开 API 包括 `NavMeshBakeTaskHandle/State`、`NavMeshDirtyBounds/Report`、`navigation_component_descriptors()`、`navigation_plugin_options()` 和 `navigation_event_catalog()`。默认选项含 agent type、settings asset、debug gizmos 与 `recast` bake backend。

## Physics

`physics`（experimental）提供 rigid body/collider 运行时、固定步模拟、raycast/overlap/shape cast、trigger/contact、constraints 与 skeletal joints。后端层导出 `BuiltinPhysicsBackend` 和 `JoltPhysicsBackend`；Jolt 路径是否可用取决于相应构建/后端条件，不能仅因类型存在就假定宿主已启用。

公开类型涵盖 `DefaultPhysicsManager`、`PhysicsBackend`、`PhysicsStepContext/Stats`、`AxisConstraint`、`JointParams`、`JointSpring`、skeletal joint/ragdoll 相关描述，以及 runtime framework 的 physics query 类型。`integrate_builtin_physics_steps` 提供内建后端步进。配置键为 `physics.settings`，主系统在 fixed-update 域工作，并记录 step duration diagnostics。

主插件及 raycast、overlap、shape cast、constraints 等多项 capability 当前为 partial；将物理结果用于 gameplay 前应固定后端和 determinism 预期。

## Network

`net`（beta，partial）主包提供配置、TCP/UDP/HTTP/WebSocket 服务抽象、TLS 帮助函数、worker、运行时状态、ingress/egress 系统和诊断。`NetConfig`、`DefaultNetManager`、`NetDriver`、`NetRuntimeManager` 是主要入口。TLS API 包括 `rustls_client_config`、`rustls_server_config`、证书 pin 摘要与校验。

网络功能被拆成独立 feature runtime crate：

| Feature | 功能 | 主要 API |
|---|---|---|
| `net.http` | HTTP client/server、路由与安全策略 | `HyperReqwestHttpBackend`, `http_runtime_backend()` |
| `net.websocket` | client/listener、握手、frame、连接 | `TungsteniteWebSocketBackend`, `websocket_runtime_backend()` |
| `net.reliable_udp` | 分片、ACK/重传、恢复、顺序交付 | `ReliableUdpWirePacket`, `NetReliableUdpRuntimeManager` |
| `net.rpc` | handshake、channel、quota、dispatch/session | `NetRpcRuntimeManager`, `RpcChannelMessage` |
| `net.replication` | registry、interest、snapshot、budget、apply | `NetReplicationRuntimeManager`, `NetReplicationTable` |
| `net.content_download` | manifest、断点续传、bitmap、HTTP fetch、进度 | `NetContentDownloadRuntimeManager` |

feature 通过 `plugin_feature_registration()` 注册，必须先启用主 `runtime.plugin.net` 和 feature 自身 capability。RPC channel 常量区分 reliable ordered 与 unreliable；握手 magic/capability 是协议契约，不应随意改名。

## Sound

`sound`（beta，partial）是完整音频运行时而非单纯播放器。它包含设备/backend、voice 与 bus、DSP gain/filter/dynamics/reverb/meter/modulation、spatial listener/emitter、attenuation、HRTF/volume、动态事件目录与 ABI、场景系统、诊断和编辑器 authoring。

主要入口为 `SoundConfig`、`DefaultSoundManager`、`SoundDriver`、`sound_component_descriptors()`、`sound_options()`、`sound_event_catalogs()`、`runtime_plugin_descriptor()`。清单选项包括 backend、sample rate、channel count/layout 与 global volume。依赖 asset、scene；ray query 与 timeline sequence 为可选依赖。

子 feature 有 `timeline_animation_track` 与 `ray_traced_convolution_reverb`，各自包含 runtime/editor/dist。后者需要 ray tracing/query 路径，未协商相关能力时必须回退或禁用，不能静默承诺卷积结果。

## Particles

`particles`（experimental，partial）提供 emitter asset、module stack、CPU/GPU 模拟描述、spawn/initialize/update/output 阶段、粒子组件、service、sprite snapshot 与渲染互操作。公开 API 包括 `ParticleEmitterAsset`、`ParticleEmitterComponent`、`ParticleSystemComponent`、`ParticleSimulationError`、`ParticleSpriteSnapshot`、渲染数据/feature 注册与 manager/service 类型。

渲染包的 `vfx_graph` feature 依赖 particles 与 shader graph。基础 particles 可以独立存在；VFX graph 只是可视化图编译与 render feature 集成，不是粒子核心的替代品。

## Terrain 与 Tilemap 2D

`terrain`（beta，partial）目前是 heightfield runtime/editor 包，runtime 对外主要提供 capability、plugin declaration、manifest 和注册入口，核心表现仍偏契约骨架。`tilemap_2d` 同为 beta/partial，提供 2D tilemap runtime/editor 注册。二者可用于建立资产、组件和 authoring 工作流，但不应被描述为已有完整地形流送或完整 2D 渲染产品链。

## Texture

`texture`（stable，主能力 complete）提供纹理处理 manager/module/plugin 边界，并带 editor authoring 与 native dist。它与 `texture_importer` 不同：前者管理运行时纹理处理能力，后者负责把源图片/容器转成 Texture 资产。

## ZrVM Language

`zr_vm_language`（experimental，partial）是 ZrVM 项目语言后端，提供 package/instance、真实 backend、host modules、reflection host、extension host、call-site 编译表、参数布局、状态编码和行为/系统/RPC/editor-op host interface。它导出 `script.behavior.v1`，供 AI 等插件可选导入。

插件面向 client/server/editor host。脚本调用必须经过编译 call site 和参数布局，不要把 VM value 当作 Rust ABI 值直接跨边界。状态恢复需遵循现有 state encoding schema。

## 使用建议

运行时插件通过 `runtime_plugin()` 返回插件对象，通过 `plugin_registration()` 获取注册报告，通过 `runtime_plugin_descriptor()` 或 `package_manifest()` 做选择与检查。应用代码优先依赖 runtime framework trait（如 `NavigationManager`、physics query interface），只有需要具体实现扩展时才依赖插件 crate 的 concrete manager。
