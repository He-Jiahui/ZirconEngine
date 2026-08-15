# 全量审查覆盖账本

## 1. 用途

本账本记录“物理上读到了哪里”和证据等级，不记录里程碑验收结果。workspace总量数字是 2026-08-15 快照，最新UI切片更新于 2026-08-16，均包含测试代码；工作区存在大量其他会话修改，因此每个深审单元在写入修复计划前仍需重新取指纹并复核重叠文件。

证据等级使用主计划的 E0-E4 定义。`queued` 仅表示已建立扫描范围，不表示不存在问题。

## 2. Workspace 快照

| package/domain | Rust 文件 | Rust 行数 | 当前深审状态 |
|---|---:|---:|---|
| `zircon_app` | 190 | 28,885 | 局部 E3：bootstrap/session teardown及runtime entry window/input/surface/cadence host；其余 queued |
| `zircon_runtime` | 7,643 | 1,081,110 | 局部 E3：core/runtime/resource/scene/platform/systems、graphics到terminal composition及runtime UI architecture/tree/layout/input/accessibility；其余 queued |
| `zircon_runtime_interface` | 416 | 49,572 | 局部 E2：runtime event V1 input/window/lifecycle shape；其余 queued |
| `zircon_editor` | 5,711 | 563,950 | 局部 E3：process tree、Play process与export process output调用点；其余 queued |
| `zircon_plugins` | 2,817 | 221,355 | 局部 E3：plugin SDK、ZrVM real backend、first-party catalog与公共注册面；feature内部算法仍queued |
| `zircon_hub` | 130 | 38,964 | E0 |
| `zircon-engine-derive` | - | 870 | E0 |
| `cargo-zircon` | - | 4,310 | E0 |

这些数据只用于控制覆盖面，不能用于评价质量。后续每个大域必须把 production、tests、build scripts、features、examples/fixtures 和产品调用点分开统计。

## 3. `core::runtime` 物理范围

| 子域 | Rust 文件 | Rust 行数 | 本轮状态 |
|---|---:|---:|---|
| root facade | 11 | 829 | E2 |
| contexts | 3 | 20 | E2 |
| descriptors | 12 | 515 | E3 |
| state | 4 | 133 | E3 |
| handle/activation-registration-resolution | 当前 tranche | 约 8K | E3 |
| events | 6 | 771 | E3：delivery、queue、subscription、diagnostics 与行为/基准证据 |
| tasks | 22 | 4,408 | E3：pool、scheduler、handle、timer、bounded I/O 与生产调用点；存在其他会话修改 |
| diagnostics | 52 | 10,941 | E3：共享 store/snapshot/devtools、profiling、runtime collector、动态导出与 config 集成；render metric 语义另审；存在大量其他会话修改 |
| tests | 58 | 8,310 | lifecycle/registration/resolution E2，其余 queued |

Lifecycle/registry 轮次详细阅读覆盖：

- module/service descriptor、`LifecycleState`、`ModuleLifecycle`、`CoreHandle/CoreRuntime`；
- 单模块与批量 activation、rollback、ready polling、deactivation、blocked unload；
- module dependency sort、service dependency validation、registration、lazy resolution、generation；
- activation/registration/resolution 行为测试与结构守卫；
- `zircon_app` bootstrap、runtime library `Drop`、dynamic session teardown 的生产调用点；
- Bevy App/Plugin、Fyrox dynamic plugin、Godot module/GDExtension、Unreal ModuleManager 对应源码。

Event/task 轮次详细阅读覆盖：

- `EventBus` topic/subscriber snapshot、同 topic 串行 delivery、三种 queue policy、阻塞接收、drop/unsubscribe、poison 恢复和 hot-path diagnostics；
- event 行为测试、managed benchmark evidence，以及 `EngineEvent` 的 string/JSON framework surface；
- `TaskPool/TaskPools`、thread assignment、`JobScheduler/JobHandle`、process timer、bounded keyed I/O、asset/operation/graphics 生产调用点；
- dynamic session destroy 到 `cdylib` unload 的线程寿命闭环；
- Bevy task pool/typed messages、Godot worker pool/message queue、Unreal scheduler stop/restart 的对应源码。

Diagnostics/profiling/config 轮次详细阅读覆盖：

- `DiagnosticStore` descriptor/path、metadata、history/EMA/min/max、锁与 snapshot；ECS publish 和 runtime facade collector 的生产链；
- process-global profiler 的 capture control、scope/frame/counter、thread-local context、ring eviction、hotspot 分析、Perfetto/JSON 导出和 dynamic session ABI；
- `ConfigStore`、`DefaultConfigManager`、dirty generation/debounce worker、commit fence、atomic writer、flush/drop，以及 animation/editor/app 调用点；
- profiling 子树 29 个、diagnostics 整体 61 个 `#[test]`；未找到真实 benchmark harness；
- Bevy diagnostic store、Unreal CPU trace/counters、Godot performance/project settings、Unity Graphics profiling scope 的对应源码。

仍未覆盖：render/physics/animation 各 metric 的逐字段正确性、diagnostic log sink 的完整队列/轮转/crash flush，以及 event/task/config 与所有上层子系统的逐调用点迁移清单。它们进入 `09/10/11`、后续 logging 单元和所属上层计划；不能由 `03` 宣称整个 observability/config 系统完成。

## 4. Resource/Asset/Serialization 物理范围

| 子域 | Rust 文件/行数 | 本轮状态 |
|---|---:|---|
| `core/resource` | 57 / 约 9,381 | E3：authority、registry、mutation/commit、payload/lease、runtime slot、projection、event stream 与 durable I/O transaction |
| `asset` production | 377 / 约 54,106 | 局部 E3：identity/facade、project loading、worker/watch、importer registry/schema、artifact、reference、pack/install；格式算法另审 |
| `asset` tests | 174 / 约 33,789 | E2：866 个 `#[test]` 的分布与关键行为/结构守卫交叉核对，不以数量代替产品闭环 |
| `asset/pack` | 17 / 约 1,773 | E3：manifest、writer/reader、dedup、trim、delta、installer 与 hot-update production 调用链 |

Resource/asset 轮次详细阅读覆盖：

- `ResourceId/Kind/Record/Handle/Lease`、authority lock、原子 batch/revision、payload/runtime slot、management/readiness immutable generation 和有界 event stream；
- `ProjectAssetManager::ensure_resident/load_typed` 到 render/UI/streamer 等 production `load_*_asset` 调用点，以及专用 acquire consumer 的缺失；
- importer descriptor/immutable registry、schema migrator、worker single-flight/budget/cancel、watch coalescing/reconciliation/error publication；
- artifact v4 staging/zstd/content-addressed chunk/manifest/read path/cache key，以及物理 chunk 与 semantic streaming 的区别；
- resource dependency aggregate/cycle 测试、GUID/path/subasset resolver 与既有 stale-reference fixed 记录；
- `.zrpack` whole-memory writer/reader/delta/trim/install、发布恢复边界和 native hot-update 关联；
- Bevy handle/load state、Fyrox async loader/type UUID、Godot threaded load、Unreal streamable/IoDispatcher/AssetRegistry/Pak、Unity Graphics probe streaming 对应源码。

仍未覆盖：各 importer 格式规范与算法质量、meshopt/SDF/IBL、render residency/GPU upload 正确性、plugin ABI/trust 最终闭环。scene component/world mutation 语义已进入 `05`；其余分别进入 `07/09/10` 及 Host/Plugin 专篇，不能由 `04` 宣称完成。

## 5. Scene/ECS/World Lifecycle 物理范围

| 子域 | Rust 文件/行数 | 本轮状态 |
|---|---:|---|
| `scene` 全树 | 1,075 / 约 100,891 | 局部E3：identity、World/archetype/query/schedule、derived state、reflection/DynamicScene、LevelSystem、inspection与产品调用点 |
| `scene/world` | 76 / 约 19,944 | E3：identity、clone/serde、mutation、hierarchy/derived state、render extract、observer与generation |
| `scene/ecs` | 143 / 约 19,475 | E3：archetype table、sparse storage、query/change tick、system params/scheduler、commands、events/messages/observers |
| `scene/dynamic_scene` | 615 / 约 15,988 | 局部E3：capture/spawn/reload/document/session；archive组合API的每个方法行为未逐一重验 |
| `scene/inspection` | 14 / 约 3,431 | E3：artifact增量链、legacy snapshot/query与gateway消费者 |
| `scene` tests | 154 / 约 33,923 | E2：1,001个test属性分布；至少47个文件使用source `include_str!`，不能替代并发/规模/product验收 |

Scene/ECS轮次详细阅读覆盖：

- public `EntityId`、internal generational entity/registry、stable query order、archetype-owned columns、sparse locator、bundle/deferred transaction和compiled query plan；
- change tick、system param/access descriptor、schedule compile/runner/worker dispatch，确认Query/Res/Event等World参数仍无法worker执行；
- hierarchy mutation index、dirty flags、active/world transform/NodeCache、render extract和LevelSystem frame publication；
- DynamicScene root migration、per-component reflection metadata、capture/spawn/reload/session，以及Editor Play snapshot和runtime gateway query；
- inspection artifact的Arc复用/按实体字段失效与legacy full query差异；event active worklist、bounded message和single-frame unbounded events；
- runtime scene partition/level streaming owner缺失，以及Editor retained-host中的preview controls/feedback；
- Bevy entity/scheduler/dynamic world、Fyrox handle/graph、Godot Node/PackedScene/process group、Unreal UWorld/streaming/partition/Mass对应源码。

仍未覆盖：physics/audio/animation/navigation/script/network内部算法，renderer/RHI/GPU lifetime，editor undo/prefab/multi-user完整工作流，以及scene session 565文件中每个组合wrapper的独立行为。scene/ECS源码和相邻计划正在被其他会话大规模修改，本轮标记`recheck_required`；实现前必须重新取指纹并取得coordinator授权。

## 6. Platform/Input/Process Host 物理范围

| 子域 | Rust 文件/行数 | 本轮状态 |
|---|---:|---|
| `zircon_runtime/platform` | 52 / 约 7,343 | E3：module/driver/manager、capability matrix、target/features、preference persistence与cross-target tests |
| `zircon_runtime/input` | 31 / 约 4,005 | E3：event/state/reducer、frame buffer/recording、action evaluator、replay与tests |
| `core/framework/input` | 26 / 约 1,302 | E3：public event/snapshot/action/device/host-request contracts |
| `core/framework/window` | 14 / 约 765 | E2：window descriptor/lifecycle policy与tests |
| `zircon_app/runtime_entry_app` | 79 / 约 5,681 | E3：winit handler、single-window/surface、cadence、input producer、host request与gamepad |
| editor process owner | `core/process.rs` 604 + Play backend约1,133 + export support约513 | 局部E3：tree containment、spawn/cancel/reap、output budgets与分叉调用点 |

Platform/Input/Process轮次详细阅读覆盖：

- PlatformModule公开描述、真实preference-only driver、compiled capability matrix与EngineEntry backend安装路径；
- RuntimeEntryApp的window/surface/cadence/application handler，确认single window/fixed viewport及WindowId/DeviceId丢失；
- InputEvent/InputState/frame snapshot/action map/evaluator/recording/replay/host requests，及script/product消费者搜索；
- dynamic V1 event ABI、per-event session/input reducer链和2026-08-14/15 current-source性能/动作索引记录；
- editor Play Windows Job Object、Unix process group、local bounded output、compile/export/wizard进程调用点与Runtime11/Editor14/15 open failures；
- Bevy window mapping/lifecycle/accumulator、Fyrox graphics suspend、Godot window/device/application notification、Unreal input identity/async consumer/platform process对应源码。

仍未覆盖：RHI surface/device-loss内部实现、Editor retained-host完整多窗口产品工作流、plugin/remote input安全、各OS backend实机行为与graphics frame pacing。当前input/app/process源码存在其他Session活跃修改，06标记`recheck_required`；没有运行Cargo、真实设备、1k Hz input或1 GiB process output验收。

## 7. Script/Plugin Runtime 物理范围

| 子域 | Rust 文件/行数 | 本轮状态 |
|---|---:|---|
| `zircon_runtime/script` | 102 / 约 17,104 | E3：host exports/capability、package/discovery、instance/manager、hot reload/GC、reflection、gameplay host、scene system与tests |
| `zircon_runtime/plugin` | 613 / 约 51,837 | 局部E3：native ABI/loader/live host、package、catalog、extension/bridge/profile/export plan；feature contribution逐算法另审 |
| `zircon_runtime/dynamic_api` | 74 / 约 14,094 | 局部E3：startup scripts、linked plugins、runtime profile与Vampire产品测试入口 |
| `zircon_plugins` | 2,835 / 约 222,013 | 局部E3：SDK、ZrVM real backend、first-party catalog；其余物理盘点完成、算法queued |
| `zircon_editor/core/plugin` | 35 / 约 5,779 | 局部E3：admission/isolation/manager/publication/watcher/Play边界；完整authoring UX另审 |

Script/Plugin轮次详细阅读覆盖：

- capability-gated host exports、borrowed script values、VM package/manifest/discovery budgets、manager/slot/hot reload/state migration/reflection/GC和scene lifecycle；
- ZrVM real backend process-wide lock/raw owner、real-backend tests，以及Vampire/WoC startup script与dynamic session产品调用点；
- native ABI V2/V3/V4/epoch surface、library load、callback admission/generation pin、bounded command sink、state/callback budget与trust/isolation边界；
- package dependency、runtime catalog、extension registry、bridge、runtime profile、bootstrap/dynamic-session多authority和batch publication；
- 复核2026-08-15 native discovery/live host 88/88与catalog/extension/bridge/profile 135/135 current-source报告，并区分2026-07历史failure；
- Bevy Plugin lifecycle、Fyrox dynamic plugin/script callbacks、Godot ScriptLanguage/GDExtension、Unreal ModuleManager/PluginManager对应源码。

仍未覆盖：各physics/audio/animation/navigation/network/render feature plugin内部算法、graphics插件GPU正确性、Editor完整plugin authoring UX、真实签名链/恶意DLL/crash worker、跨平台ABI和产品规模性能。当前script/plugin/dynamic API/SDK/ZrVM/editor plugin源码有大量其他Session活跃修改，07标记`recheck_required`；没有运行Cargo、真实backend、fault/security或性能验收。

## 8. Physics Runtime 物理范围

| 子域 | Rust 文件/行数 | 本轮状态 |
|---|---:|---|
| `core/framework/physics` | 32 / 1,095 | E3：body/collider/joint/material/query/event/settings/manager公共合同 |
| `core/framework/scene/physics` | 11 / 423 | E3：scene mass/material/CCD/sleep/joint metadata与serde |
| physics runtime plugin | 77 / 12,366 | E3：manager、builtin/Jolt backend、query/contact/trigger/constraint、skeletal与runtime systems |
| physics editor plugin | 7 / 483 | E3：registration、debug DTO、ragdoll profile editor helper |
| physics native dist | 1 / 98 | E3：dynamic entry、distribution projection与Cargo feature传播 |
| physics plugin tests | 82个 `#[test]` | E2：合同/行为测试存在；无benchmark、property、Loom、sanitizer、soak或产品打包证据 |

Physics轮次详细阅读覆盖：

- 默认/runtime/dist/editor/animation Cargo feature传播和plugin manifest，确认普通发行路径没有启用`backend-jolt`，无feature默认`unconfigured/Disabled`；
- engine FixedUpdate调用链、Physics manager accumulator与产品`fixed_update_step_plan`，确认存在两个fixed clock authority；
- scene `node_records()`到owned `PhysicsWorldSyncState`、Jolt managed world、active body回写、LevelSystem replacement epoch/frame snapshot的完整生产链；
- builtin积分/近似query/contact/constraint与Jolt native world/layer/shape/mass/command/readback，确认Jolt query为空、event来自pairwise近似、constraint为step后投影；
- query filter/result allocation、collision layer/matrix、local shape transform/scale/multi-shape、material/mass、mesh asset registration/cook缺口；
- scene property/reflection校验、ragdoll profile/runtime/editor helper、animation pose feed和open physics overlay failure；
- Physics03历史milestone与current source交叉核对，确认ContactListener、native constraint、ragdoll产品入口和overlay provider曾出现API/test级false-green；
- Unreal BodyInstance/WorldCollision/Chaos solver、Godot PhysicsServer/direct state、Fyrox persistent PhysicsWorld/dirty sync与Bevy fixed clock对应源码。

仍未覆盖：Audio、Animation、Navigation、Network内部算法；Jolt真实native build、editor/app/export产品运行；跨平台、fault、soak、规模性能和确定性/rewind验证。`zircon_plugins/physics/runtime/src/plugin.rs`与`runtime_system.rs`存在其他Session修改，08A标记`source_recheck_required`。本轮未运行Cargo且未改production code，不能把静态审查记作Physics实现完成。

## 9. Audio Runtime 物理范围

| 子域 | Rust 文件/行数 | 本轮状态 |
|---|---:|---|
| `core/framework/audio` | 3 / 331 | E3：layout、channel 与 frame conversion contract |
| `core/framework/sound` | 28 / 2,114 | E3：manager、source/listener/volume、graph、timeline、event 与 config contract |
| sound runtime production | 229 / 10,971 | E3：Kira bridge、engine state、configuration、output、automation、timeline、events、package |
| sound runtime tests | 1,035 / 12,411 | E2：344个test属性；约132个文件使用`include_str!`，未发现benchmark/real-device/soak证据 |
| sound editor/dist/features | 28 / 2,186 | E3：registration、authoring/live-output DTO 与optional feature projection |
| audio importer families | 12 / 约1,695 | E3：WAV/Symphonia全量decode、descriptor authority与Opus diagnostic path |

Audio轮次详细阅读覆盖：

- Kira 0.12.2 manager、StaticSoundData、track/send graph compile/validation/transaction、active playback、Tween参数更新与output lifecycle，确认callback/audio thread归Kira而非Zircon manager mutex；
- scene component/plugin registration、module/manager bootstrap和app/editor/export产品调用点，确认没有RuntimeSceneSystem或`sound.spatial_update`，设备只由未进入production构造的Editor live-output controller启动；
- 全局SoundEngineState、source/listener/volume、playback completion、gameplay emission journal与AI perception consumer，确认journal按world有界但其余音频状态没有world/session生命周期；
- effect/3D/HRTF/occlusion/convolution/volume、timeline、automation、dynamic events与optional feature，确认Kira M1拒绝effect/advanced track state，旧owned-block DSP和空feature module没有production执行落点；
- SoundAsset、WAV/Symphonia/Opus importer、clip loading/Kira frame conversion，确认全量PCM、双份decoded residency、无streaming/cook/single-flight/eviction/voice virtualization；
- device catalog/configure/start/status、runtime snapshot和Editor meter/debug surface，确认无hotplug/recovery且callback/underrun/meter字段没有真实producer；
- Unreal AudioMixer/SoundWave、Godot AudioServer、Fyrox sound engine/streaming buffer与Bevy ECS audio/encoded asset对应源码。

仍未覆盖：Navigation、Network内部算法；真实Kira/CPAL设备、音频质量、跨平台、fault、soak、规模性能和Editor/App/Export产品运行。focused sound源码当前未显示工作区修改，但2026-07的M1与open send-routing记录缺少统一current-source结论，08B标记`source_recheck_required`。本轮未运行Cargo且未改production code，不能把静态审查记作Audio实现完成。

## 10. Animation Runtime 物理范围

| 子域 | Rust 文件/行数 | 本轮状态 |
|---|---:|---|
| `zircon_runtime/src/animation` production | 16 / 1,998 | E3：module/manager、clip event sampler、compiled sequence与scene property apply |
| `core/framework/animation` | 38 / 3,210 | E3：asset、player、graph/state machine、event、IK、GPU/readiness与manager contract |
| animation runtime production | 134 / 12,448 | E3：scene pipeline、compiled evaluator/cache、worker、IK、manager与plugin system |
| animation runtime integration tests | 26 / 5,330 | E2/E3：99个test属性，算法/world行为较强，产品/render/cook/scale证据不足 |
| animation editor/dist | 6 / 293 | E2：package、descriptor与inspector registration |
| animation graph package | 6 / 923 | E2：validation/registration，缺真实document/editor/runtime artifact compile |

Animation轮次详细阅读覆盖：

- plugin `animation.evaluate` scene system、typed projection、revision cache、direct clip worker、graph/state machine/layer/sequence、PosePool、pose apply、IK与physics pose bridge；
- replacement epoch、clip event admission/defer/cursor/bytes/count/span预算、deferred entity rollback与immutable pose/playback publication，确认这些是必须保留的current-source基础；
- core/plugin重复module/manager与dynamic session linked/unlinked routing，确认注册分支已避免同session双注册，但物理implementation authority仍重复；
- dynamic session frame demand与LevelSystem animation state，确认production没有播放状态写入continuous-frame位，只有event backlog可间接保持帧循环；
- core/plugin glTF importer与compiled target table，确认高优先级plugin输出animation placeholder，builtin非根bone leaf `target_id`与full skeleton path不一致，inverse bind JSON没有runtime/render consumer；
- raw clip/graph/state/sequence asset load、String/AoS pose、per-frame容器与deep equality、局部worker并行、masked blend、trigger、event sampling、逐骨scene transform写回及GPU skinning readiness/CPU clone假表面；
- animation editor与animation_graph package，确认声明的ZUI视图不存在、operation无handler，graph compile只返回output id，state-machine compile只返回计数；
- Unreal AnimInstance/AnimNode/AnimSequence/AnimSync、Godot Mixer/Player/Tree/Skeleton、Fyrox track/pose/machine与Bevy target/graph/transition对应源码。

仍未覆盖：Network内部算法；真实App/Editor reactive播放、glTF import-to-render、GPU skinning、跨平台、fault、soak、压缩质量和1k角色规模性能。animation pipeline/IK/manager存在其他Session大量current-source修改，08C标记`source_recheck_required`。本轮未运行Cargo且未改production code，不能把静态审查记作Animation实现完成。

## 11. Navigation Runtime 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| `core/framework/navigation` | 19 Rust / 1,645 | E3：asset、agent、surface、query、bake、settings、gizmo与manager contract |
| `zircon_runtime/src/navigation` | 15 Rust / 3,351 | E3：builtin module/manager、compiled fallback mesh、typed projection、avoidance、operation |
| navigation runtime plugin | 55 Rust / 8,110 | E3：scene system、Crowd与legacy agent、bake、obstacle、off-mesh、overlay |
| navigation native Rust bridge | 25 Rust / 3,716 | E3：Recast bake、Detour/Crowd/TileCache、asset FFI与Rust fallback query |
| navigation native C++ bridge | 10 C/C++/header / 3,394 | E3：自有C ABI bridge；vendored upstream另计 |
| navigation editor | 21 Rust / 3,308 | E3：registration、bake panel、operation command、PIE mirror、overlay/provider |
| navigation tests | 163个 `#[test]` | E2/E3：局部算法/bridge/world行为较强；无benchmark、scale、product、fault、soak证据 |

Navigation轮次详细阅读覆盖：

- dynamic session linked/unlinked module选择、默认navmesh TOML加载、plugin scene system和frame demand，确认builtin fallback没有agent tick/events且Navigation active work不会请求reactive frame；
- core builtin manager、plugin Recast/Crowd主路径和plugin legacy路径，确认obstacle/off-mesh会静默清Crowd并改变backend/avoidance语义；
- raw NavMeshAsset、manager state和native Detour bridge，确认state非world-scoped、asset校验/卸载不足，并且每query clone asset、重建dtNavMesh/dtNavMeshQuery后销毁；
- native failure到Rust fallback路由，确认unsupported/backend/no-path被Option混合，fallback每query重建高复杂度polygon graph；
- bake geometry、settings/profile、tiled/dirty/task流程，确认render/collider输入是单位顶面/圆盘近似、关键Recast参数不生效、tile结果被合并回raw asset且task无取消/容量/epoch；
- agent/Crowd projection、repath/debug与writeback，确认每帧node_records/JSON/容器分配、256硬容量、obstacle/link触发legacy以及Transform绕过character/physics；
- off-mesh traversal与AI MoveTo，确认内部直接插值Transform、无gameplay ticket，并且AI以event storage判断能力且没有request/outcome generation；
- Editor Bake/ZUI/operation、PIE overlay和plugin options，确认Bake handler固定失败、panel backend仅测试、16次同步poll、稳定navmesh每帧full mirror以及options只有manifest；
- Unreal NavigationSystem/Recast tile generator/world partition、Godot map iteration/query/generator、Fyrox scene navmesh和upstream Recast/Detour/Crowd对应源码。

仍未覆盖：Network内部算法；真实mesh import-to-bake、App/Editor reactive agent cadence、async Editor bake、world partition、跨平台native、fault、sanitizer、soak与1k/10k agent/100k polygon规模性能。navigation plugin/editor/operation当前存在其他Session未提交修改，08D标记`source_recheck_required`。本轮未运行Cargo且未改production code，不能把静态审查记作Navigation实现完成。

## 12. Network Runtime 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| `core/framework/net` | 18 Rust / 2,107 | E3：endpoint、transport/security、HTTP/WS、session/RPC、sync/reliable/download与manager contract |
| net core runtime | 49 Rust / 4,994 | E3：module/service、worker、TCP/UDP、HTTP/WS bridge、system、shutdown与diagnostics |
| HTTP / WebSocket features | 34 Rust / 2,286 | E3：client/server、TLS/pin、handshake、reader/writer、frame与feature composition |
| RPC / Replication features | 44 Rust / 3,431 | E3：session/handler/channel/quota与descriptor/delta/interest/schedule/apply |
| Reliable UDP / Content Download | 42 Rust / 2,835 | E3：wire/fragment/ACK/resend与manifest/range/hash/resume/progress |
| net editor / dist | 7 Rust / 519 | E3 registration/ABI，E1 product：ZUI/operation/runtime behavior缺失 |
| network tests | 126个test属性 | E2/E3：局部loopback与算法行为存在；无product composition、fuzz、scale、cross-platform、soak证据 |

Network轮次详细阅读覆盖：

- canonical `NetManager`与六个optional feature factory/consumer，确认HTTP/WS各建私有manager而非扩展canonical service，Content Download production依赖因此无法取得HTTP backend，RPC/Replication/Reliable UDP没有production consumer或transport接线；
- core worker、两个Tokio runtime、sync channel、service mutex和shutdown，确认所有TCP/UDP操作同步等待单一串行worker、timeout不取消、late success可产生orphan state、构造expect与shutdown join没有有界终态；
- runtime systems/diagnostics/config/options，确认diagnostics以`usize::MAX`绕过256预算、worker event静默drop、主event/WS frame无界、flush为空、frame index固定0且runtime mode/budgets只有manifest；
- endpoint/TCP/UDP状态与消息语义，确认无DNS/IPv6格式/socket policy/framing/readiness receive，TCP partial stream与裸ID/generation/error合同不足；
- HTTP/WS完整backend，确认逐请求client/TLS build、全body、无幂等retry/bounded server，local route可按path截获任意无显式端口URL，WebSocket pin未验证peer cert、入站无界且close不终止tasks；
- Reliable UDP logical/wire model，确认与socket断开、u64/u16 sequence和u16/u8 fragment冲突、MTU未扣header、assembly/ordered/outbound无界、wrap ACK/拥塞/pacing/security缺失；
- session/RPC，确认unused token、静态nonce直接作response、caller role外部传入、handler timeout不可抢占、queue/codec/transport/session cleanup不完整；
- replication/prediction，确认不扫描World、不发包、authority/strategy/field metadata未执行、每session full clone/sort、无baseline/ACK/relevancy/dormancy/input/time sync/reconciliation；
- Content Download与Editor/dist，确认同步全内存/insecure unsigned manifest/无持久cache原子安装，Editor引用缺失ZUI/default asset且operation无handler，dist只有stateless registration；
- Unreal NetDriver/Connection/Iris descriptor/filter/priority、Godot SceneMultiplayer/Replication/RPC/ENet/WebSocket及Bevy Remote executor边界；Fyrox无first-party multiplayer authority，Unity Graphics不用于反推network设计。

仍未覆盖：真实Cargo/App/Editor/dedicated server运行、双进程World/RPC/replication、公网/TLS/WSS、parser fuzz/sanitizer、跨平台、fault、24h soak与1/100/10k connection规模性能。`zircon_plugins/net/runtime/src/plugin.rs`和`runtime_system.rs`存在其他Session修改，08E标记`source_recheck_required`。本轮未运行Cargo且未改production code，不能把静态审查记作Network实现完成。

## 13. RHI / Render Graph / GPU Lifetime 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| `zr_rhi` | 15 Rust / 4,797 | E3：capability、descriptor、handle、command、native surface与UI contract |
| `zr_rhi_wgpu` | 61 Rust / 18,934 | E3：test-only deterministic RHI、validation、timer/statistics/readback与retained UI |
| `render_graph` | 16 Rust / 5,243 | E3：builder、compile、culling、lifetime、alias plan、dump/lint与49个tests |
| `graphics/backend` | 24 Rust / 3,131 | E3：真实WGPU adapter/device/surface、同步readback与IBL路径 |
| 产品接线 | targeted E3 | pass authoring/cache、framework submission、graph materialization、transient pool、scene submit/present与dynamic profile |

RHI/Graph轮次详细阅读覆盖：

- 复核RHI 76/76文件、23,731行、249个test属性，确认唯一`RenderDevice`实现现在只在`#[cfg(test)]`编译，真实RenderBackend直接拥有WGPU Instance/Adapter/Device/Queue，并且`create_command_list`泛型方法使trait非对象安全；
- 对照RHI command、capability、handle/fence和native surface，确认命令无法表达已声明的multi-draw/sparse/AS/queue能力，raw handle/fence无device generation，产品caps错误报告`supports_surface=false`且native target只有Win32；
- 追踪adapter/device协商、scene/UI surface、compiled scene submit、output writeback、resource/upload/readback和retained UI，确认默认Editor同步、可选容量1私有submit线程、普通presented frame静态至少2次submit、多个direct submit/poll与同步`wait_indefinitely`；
- 复核三槽`GpuReadbackQueue`、GPU timer/statistics、共享UI image和transient pool，确认局部对齐/缩容/parallel record可保留，但缺request/byte/age预算、submission ticket、device identity/loss终态，资源回池只依赖CPU frame而非GPU completion；
- 逐读Render Graph 16文件，确认handle只有usize且可接受外来同index handle、resource无version/subresource、manual WAW+latest-writer RAW、反向culling不消费resource version、logical alias和physical exact pool双authority、hash reservation与overflow语义不足；
- 追踪产品pass authoring、compiled cache和stats，确认全pass相邻依赖形成总链、Bloom按executor string特判、compile miss在framework state lock内、16-entry key含动态尺寸、每帧O(P²*A) store lint只取count；
- 对照Unreal RHI/RDG/D3D12 submission、Unity versioned RenderGraph/native pass compiler、Godot RenderingDeviceGraph/driver queue/barrier、Bevy centralized command buffers/device error/readback，以及Fyrox object-safe server/async readback。

仍未覆盖：material/shader/pipeline、texture/mesh streaming、lighting/shadow/post/temporal算法和runtime UI/text的逐文件质量；renderer/visibility/GPU Scene已进入09B。当前RHI/Graph/backend/UI文件存在大量其他Session修改，09A标记`source_recheck_required`。本轮未运行Cargo、真实GPU、Editor、device loss、RenderDoc、WPR、soak或规模benchmark，不能把静态审查记作RHI/RDG实现完成。

## 14. Renderer / Visibility / GPU Scene 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| `graphics/scene/gpu_scene` | 18 Rust / 4,733 | E3：slot、history、dirty upload、morph/skin/VG与产品sync |
| `graphics/visibility` | 62 Rust / 5,212 | E3：context、relevance、view、grid、plan、query与VG plan |
| `scene_renderer/mesh` | 113 Rust / 26,848 | E3：pending draw、cache、phase、indirect、replay与pipeline consumer |
| `scene_renderer/hzb` | 7 Rust / 1,660 | E3：culler、workspace、bind cache、WGSL与readback |
| `scene_renderer/core` | 99 Rust / 14,072 | targeted E3：构造、runtime prepare、compiled scene、submit与history |
| Virtual Geometry runtime plugin | 225 Rust / 36,875 | E3 production 197 / 14,523行；E2 tests 28 / 22,352行 |

Renderer/Visibility轮次详细阅读覆盖：

- 追踪extract -> visibility context -> pending draws -> VG/morph/light/GPUScene -> late visibility/cache -> phase indirect/HZB -> submit，确认稳定帧仍由viewport draw重建scene数据，完全不可见对象先支付material/deform/VG/GPU工作；
- 逐读GPUScene 18文件与产品sync，确认dirty upload、staging ring和history基础可保留，但每draw固定注册1个instance、全draw/register/retain/history扫描仍在，skin palette按实例保留多份固定大对象，morph/VG stable仍全payload比较；
- 逐读visibility 62文件，确认所谓static index是uniform grid/BTree结构，默认相机保守球需要64,000 cells并超过4,096预算后全扫，dirty Arc COW可复制整表，多shadow view结果最终并成bool/HashSet；
- 搜索visibility产品consumer，确认`gpu_instancing_candidates`、`visible_instances`、`draw_commands`、`instance_upload_plan`和`particle_upload_plan`没有进入graphics renderer，HGI输入被忽略并发布恒空结果；
- 复核GPU bounds与HZB WGSL，确认world translation/scale被写入primitive bounds后又被instance matrix二次变换，CPU frustum与GPU occlusion没有同一bounds truth；HZB只处理indirect子集、单lane遍历arg instances且不回写FrameVisibility truth；
- 复核mesh command/cache/indirect workspace，确认persistent buffer、multi-draw replay和异步readback是正向基础，但cache太晚且无产品retirement，CPU仍生成全部command/args，动态mesh与bind group可在per-draw路径创建；
- 复核Virtual Geometry plugin 225文件关键authority链，确认node/cluster层级遍历、page request、selection、所谓hardware raster record和VisBuffer entry主要由CPU构造，现有compute shader只扩展seed work item，没有真实raster pass写visibility attachment；
- 对照Unreal GPUScene/SceneVisibility/InstanceCulling/Nanite、Unity GPU Resident Drawer、Bevy GPU preprocess/meshlet，以及Godot/Fyrox CPU scene-cull基线。

仍未覆盖：material/shader compiler与permutation、texture/mesh streaming、lighting/shadow算法、post/temporal和runtime UI/text；这些进入09C以后单元。focused renderer和Virtual Geometry目录有大量其他Session修改，09B标记`source_recheck_required`。本轮未运行Cargo、Editor、真实GPU、RenderDoc/PIX、WPR、device loss、soak或规模benchmark，不能把静态审查记作GPU-driven或Virtual Geometry实现完成。

## 15. Material / Shader / Pipeline / PSO 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| asset shader/material | 22 Rust / 4,294 | E3：资产、依赖、readiness、zshader/material文档与management DTO |
| graphics material/shader/pipeline | 86 Rust / 20,493 | E3：shading model、template、variant、prewarm、async与driver cache；RDG细节引用09A |
| Mesh pipeline/cache | 28 Rust / 9,570 | E3：各pass descriptor、variant registry、source/disk/prewarm/fallback与直接consumer |
| core render shader/material | 69 Rust / 11,363 | E3：ABI、geometry、variant、prewarm、readiness、diagnostics与management/query surface |
| shader prewarm CLI | 21 Rust / 7,893 | E3：inventory、module dependency、manifest、WGPU validation与输出 |
| Material/Shader Graph与WGSL importer插件 | 16 Rust / 1,740 | E3 compiler/registration；E0/E1 operation/template/render execution |
| WGSL与横向PSO产品点 | 42 WGSL / 3,828；targeted E3 | 47处render pipeline创建分散44文件；生产`cache: None` 57处、`Some` 1处 |

Material/Shader/PSO轮次详细阅读覆盖：

- 复核242个聚焦Rust文件、55,353物理行、487个test属性与42个WGSL文件，追踪asset -> module/include -> material ABI -> permutation -> Mesh/non-Mesh PSO -> prewarm/driver cache；
- 确认Shader/source/variant/PSO/prepared material没有单一generation，Mesh两条私有compiler、serial prewarm和Vulkan driver cache具有不同key、budget、I/O、错误与shutdown；
- 确认disk key不包含source ID、template/Naga/WGPU expected version，lookup只校验schema/hash/canonical string，metadata不能阻止stale artifact命中；
- 横向扫描全runtime产品创建点，确认47处`create_render_pipeline`分散44文件、67处`create_shader_module`分散60文件，driver cache基本只进入Mesh局部；
- 确认异步编译默认关闭且只覆盖Base Mesh，miss/queue/worker failure用`SkipDraw`返回None；其余Mesh pass和post/UI/particle/lighting等仍各自同步创建；
- 确认Raw WGSL importer无条件标记Surface，而readiness不要求entry point/pipeline layout/surface ABI，可把语法合法但产品不兼容的module标为ready；
- 复核Material Editor与Shader Graph插件，确认前者有真实小型常量折叠compiler但命令无factory且ZUI/template缺失，后者维护第二套graph model、字符串顺序生成WGSL并注册noop executor；
- 确认template每variant全文replace/拼接/parse、plugin include按descriptor/token/catalog扫描、texture presence只统计未归一化、material management/IDE artifact与产品authority分离；
- 对照Unreal ShaderCompiler/JobCache/PSO cache/MaterialShader/VertexFactory、Bevy PipelineCache/ShaderCache、Godot ShaderRD version、Unity typed Shader Graph/variant stripping与Fyrox ShaderDefinition。

仍未覆盖：texture/mesh/material streaming和resident upload/eviction、lighting/shadow/post/temporal算法、runtime UI/text；这些进入09D以后单元。Material/Shader/Pipeline/Mesh cache/WGSL当前存在大量其他Session修改，09C标记`source_recheck_required`。本轮未运行Cargo、Editor、cook、真实GPU、PIX/RenderDoc、device loss或compile storm，不能把静态审查记作Shader/PSO实现完成。

## 16. Render Asset Streaming / Residency 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| graphics scene resources | 72 Rust / 11,222 | E3：prepared map、texture/mesh/model/material ensure、mip demand、GPU upload与accessor |
| texture importer | 39 Rust / 9,301 | E3：image/PSD/container、mipgen、BC5 transcode、settings与registration |
| artifact store/cache payload | 17 Rust / 5,030 | E3：manifest/chunk/residency、zstd+bincode read/write与typed payload |
| texture assets/upload support | 23 Rust / 5,733 | E3：descriptor/payload/container layout/readiness/cubemap/lightmap |
| mesh/model assets | 23 Rust / 2,361 | E3：CPU layout、conversion、bounds、SDF、usage和serialization |
| glTF/OBJ/model importers | 18 Rust / 4,860 | E3：parse、geometry/image/material/model/mesh subasset、VG/SDF cook与registration |
| legacy texture importer surface | 3 Rust / 290 | E3 descriptor；E0 product registration/implementation |
| ProjectAssetManager loading | 7 Rust / 639 | E3：ensure resident、clone load与lease acquire |
| render budget/submission/lease spot checks | 3 Rust / 725 | E3：state lock、hardcoded budget、lease drop/unload contract |
| 合计 | 205 Rust / 40,161 | 341个inline test属性；focused fingerprint `e5aab8e2e05c7de487b7a563b962d01a1099c72643d9d24fe7c25438b1781291` |

Render Asset Streaming轮次详细阅读覆盖：

- 追踪frame `ensure_scene_resources`到ProjectAssetManager、ArtifactStore和WGPU resource创建，确认render/framework state锁域内可同步执行完整chunk read、zstd/bincode、asset clone、Mesh转换、texture/buffer/bind-group创建；
- 逐读texture mip planner与physical rebuild，确认priority/tail/hysteresis/upload/resident budget和stale transition是可保留基础，但首次总是full resident，promotion/eviction重新读取完整asset并创建replacement texture，预算不含I/O/decode/staging/old+new/fence峰值；
- 复核artifact v4 chunk/manifest/cache，确认64 KiB content-addressed物理chunk有完整性与bounded cache价值，但单个bincode+zstd对象没有texture mip/layer/tile或mesh LOD/cluster/page semantic index，单subresource仍需全对象恢复；
- 复核ProjectAssetManager clone load、ResourceLease和texture/mesh asset usage，确认graphics绕过lease、prepared又保留CPU asset，usage字段没有驱动产品CPU bulk释放，renderer maps缺少unused/remove/retire闭环；
- 复核mip visibility输入，确认只覆盖主视图visible mesh material texture，使用translation与近似scale而非真实bounds/UV density，不覆盖secondary view、sprite/UI/lightmap/cookie/LUT/environment/probe等consumer；
- 复核GPU texture/mesh上传，确认compressed container可完整upload但不能physical mip streaming，Mesh一primitive一buffer且生成临时vertex与wire数据，没有arena/suballocation/partial update/LOD residency；
- 复核glTF/OBJ/model/texture importer，确认root/per-mesh Model/per-primitive Mesh重复geometry authority，glTF内嵌图片直接RGBA8绕过canonical texture cook，并有两套含糊texture importer surface；
- 对照Unreal RenderAssetUpdate/Texture2DStreamIn/StreamingManager、Bevy RenderAsset/mesh allocator、Godot RD texture/mesh storage、Fyrox async resource/temporary cache，以及Unity Graphics mip debug/VT consumer边界。

direct lighting/clustered light grid/shadow已进入09E；仍未覆盖environment/IBL/GI、advanced lighting、post/temporal与runtime UI/text的逐文件质量，这些进入09F以后单元。graphics/asset/importer范围存在其他Session修改，09D标记`source_recheck_required`。本轮未运行Cargo、Editor、cook、真实GPU、WPR/PIX/RenderDoc、device loss、OOM、fault、cross-platform、open-world traverse或soak，不能把静态审查记作streaming实现完成。

## 17. Direct Lighting / Clustered Light Grid / Shadow 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| light authoring / asset / world extract | 3 Rust / 458 | E3：组件、scene schema、方向约定、layer、mobility与snapshot producer |
| core light ABI / readiness / shadow settings | 5 Rust / 419 | E3：GPU layout、snapshot、family readiness与shadow public contract |
| light packing / CPU grid / GPU consumer | 4 Rust + 1 WGSL / 1,189 | E3：packing、z-bin、tile mask、upload、stats与shader traversal |
| shadow plan / atlas / cache / raster | 13 Rust + 1 WGSL / 4,475 | E3：allocation、cascade、slot、view、cache policy、draw replay与PCF |
| shadow visibility producer | 1 Rust / 456 | E3：directional/point/spot view创建与caster过滤 |
| direct-light shader consumers | 1 Rust + 6 WGSL / 2,407 | E3：deferred、fallback、generated PBR、GPU ABI与cookie交叉消费 |
| clustered graph / compute / post consumer | 12 Rust + 2 WGSL / 2,344 | E3：descriptor、declared access、host write、dispatch与zero blend |
| Contact Shadow plugin | 7 Rust + 1 WGSL / 1,502 | E3：runtime/editor registration、resource、dispatch、shader与WGPU tests |
| 合计 | 57 / 13,250 | 85个inline test属性；focused fingerprint `09cd7df6ede3608c9da977497bef549393eae7731c31a21648a7cc737f731865` |

Direct Lighting / Shadow轮次详细阅读覆盖：

- 追踪scene light component/asset -> World collect -> Render snapshot，确认四类light都没有authorable shadow字段且extract硬编码`shadow: None`，ambient的`affects_lightmapped_meshes`也在snapshot边界丢失；
- 复核`GpuLightData`、packing、GPU Scene与所有direct WGSL consumer，确认layer mask、shadow strength/normal bias没有最终consumer，RectLight size不参与面积光积分，三套以上shader重复非物理`(1-d/r)^2`衰减；
- 逐读CPU z-bin/tile mask builder和WGSL，确认camera-inside/near crossing会因light center `clip.w <= 0`丢灯，orthographic `ortho_size`重复乘0.5，每帧分配/全写/笛卡尔stats扫描且silent clamp/coarsen无overload truth；
- 追踪builtin ClusteredLighting descriptor到computed resources、compute shader和post params，确认真实grid由CPU host write，所谓cluster compute只写无最终权重的方向光tile颜色，却虚报四个storage write与AsyncCompute；
- 逐读shadow allocator/plan/cascade/slot/cache/raster/PCF，确认只选第一个方向光、固定near/default cascade、point-first logical slot、无gutter/point seam，`ShadowCache`无产品consumer且whole-atlas clear/all-slot redraw；
- 追踪visibility shadow view，确认每个disabled directional仍生成1个view、全部enabled directional固定生成4个，未使用planner allocation/rejection且caster不按light layer过滤；
- 逐读Contact Shadow runtime/editor plugin及post consumer，确认shader没有camera/world/light direction，只比较邻域非线性depth/HZB/normal.z，结果全局乘到已完成的scene color；
- 对照Unreal LightGrid/LightRendering/ShadowSetup/ShadowDepth/VSM，Bevy CPU/GPU cluster与photometric light，Godot volume-raster cluster，Fyrox传统shadow，以及Unity HDRP LightLoop/ContactShadow/LTC和URP CPU grid/atlas下限。

另抽查6个独立shadow product test文件、2,677行、18个test属性；它们主要手工注入render snapshot，不能覆盖scene authoring断链。仍未覆盖：真实Cargo/Editor/GPU、photometric migration、camera-inside/ortho/light-layer/rect golden、cluster overflow、shadow cache产品hit、device loss、atlas thrash、24h soak及同画质Unreal基准。当前范围存在其他Session修改，09E标记`source_recheck_required`；本轮未改production code，不能把静态审查记作Lighting/Shadow实现完成。

## 18. Environment / Sky / IBL / Reflection Probe 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| core environment ABI / CPU IBL / artifact | 28 Rust / 8,661 | E3：skybox、probe、recipe、projection、mip、PMREM、SH9/IEM、blob与upload artifact |
| asset import / cache / project scan | 12 Rust / 5,197 | E3：HDR decode、reuse判断、并行builder、source/derived/runtime cache与project scan入口 |
| submission hydration / compile | 3 Rust / 1,462 | E3：逐帧cache resolution、bounded hydration/pending bake与graph compile option |
| GPU environment / probe / realtime IBL | 50 Rust + 10 WGSL / 15,154 | E3：cubemap upload、BRDF LUT、probe资源、GPU bake/readback/writeback、realtime scheduler与shading |
| scene / editor / reflection-probe plugin | 17 Rust + 2 manifest / 2,465 | E3：World extract断点、viewport preview、capture trigger、六面捕获与注册 |
| 合计 | 122 / 32,939 | 273个inline test属性；focused fingerprint `2f6b75b65a6da938ee3c27843ef0752f1d0e77a2b2fc62d835b810527a29d9ba` |

Environment/Sky/IBL/Probe轮次详细阅读覆盖：

- 追踪scene component/asset/property/World -> `EnvironmentExtract`，确认普通项目没有Environment/Sky/Atmosphere/Cloud/ReflectionProbe authoring，World只把viewport `preview_skybox`转成默认gradient，产品测试在snapshot后手工覆盖environment；
- 逐读source cubemap、PMREM、SH9/IEM、canonical recipe、artifact blob/cache、HDR importer与project scan，确认并行CPU构建、version/hash和bounded runtime writeback可保留，但normal warm import先完整decode HDR，reuse判断再完整读取/解码staged artifacts；
- 追踪frame submission hydration，确认runtime cache miss在submission context同步`fs::read`/decode，frame extract携带完整Arc texel/upload payload而非09D resource handle；
- 逐读cubemap upload、BRDF LUT和IBL graph，确认prepared upload与pipeline cache是正向基础，但首个renderer仍做524,288次CPU BRDF积分，每pass重建整份command plan并创建小参数资源；
- 逐读realtime IBL scheduler/graph/recorder，确认双缓冲、last-ready、bounded compiled graph cache可保留，但首次一帧执行全部工作，`CaptureSky`与`CaptureCloud`调用相同gradient capture并覆盖相同faces；
- 追踪reflection probe prepare/upload/shader，确认pre-draw可同步ensure-resident，每新probe 48次texture write，每帧candidate分配/排序，fragment逐像素扫描最多64 probe，已上传layer mask无shader consumer；
- 逐读reflection-probe runtime/editor插件，确认trigger无Editor consumer，capture顺序clone/render/readback六面后CPU bake/落盘，结果只注册内存texture而不修改scene/catalog/cook，唯一真实capture测试为manual ignored；
- 对照Unreal SkyLight/ReflectionCapture/SkyAtmosphere/VolumetricCloud，Bevy atmosphere/light probe，Godot Environment/ReflectionProbe，Fyrox SkyBox，以及Unity HDRP SkyManager/PhysicallyBasedSky/HDProbe体系。

另抽查20个独立environment/IBL/probe test文件、8,301行、116个test属性；大量测试直接构造小cubemap、手工注入render extract或断言WGSL source string，不能覆盖scene/editor/cook闭环。本轮未运行Cargo、Editor、cook、真实GPU、大气/云golden、1k probes、device loss、VRAM pressure、24h soak或同画质Unreal基准。当前范围存在大量其他Session修改和未跟踪拆分文件，09F1标记`source_recheck_required`；本轮未改production code，不能把静态审查记作Environment/IBL/Probe实现完成。

## 19. Baked Lighting / Lightmap / Irradiance Volume / Offline Bake 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| core lightmap / probe ABI | 5 / 1,312 | E3：request/output/consume、generation、SH9 grid、environment attachment与volume selection |
| asset / mesh UV / glTF import | 5 / 1,344 | E3：RGBA16F array转换、UV1保存/缺省、TEXCOORD_1导入及fixture |
| offline bake / streamer / renderer / WGSL | 22 / 3,967 | E3：伪bake、resource prepare、GPU Scene slot、forward/deferred、volume bind与最终采样 |
| baked/volume plugins与Editor表面 | 19 / 1,932 | E3：manifest、capability、executor、Workbench command feedback与控件绑定 |
| 合计 | 51 / 8,555 | 63个inline test属性；focused fingerprint `4b3864bdc34debc0d4f1fa8269921a5e758123df2fdf84cf279be908a7c18463` |

Baked Lighting / Lightmap / Irradiance Volume轮次详细阅读覆盖：

- 追踪公开`offline_bake_frame`到Reflection Probe GPU admission，确认其只从DirectionalLight intensity和前N个mesh制造无`baked_cubemap`的球形probe metadata，不生成lightmap/probe grid/artifact，且唯一M4像素断言与真实consumer静态矛盾；
- 逐读Lightmap request/output/consume与Environment attachment，确认版本、slot/page、SH9和generation校验可保留，但scene snapshot payload无schema、content hash不复算且output无recipe/dependency/content identity，也可静默漏掉requested instance；
- 追踪mesh/glTF/UV路径，确认`TEXCOORD_1`可进入GPU shader，但没有自动unwrap、chart packing、overlap/padding/density验证，缺失UV1静默归零且没有bake diagnostic；
- 追踪scene/editor/plugin入口，确认普通项目没有baked settings、build data或IrradianceVolume component，Workbench只返回固定“87 lightmaps”字符串，Baked Lighting插件默认贡献noop pass而真实shader始终采样；
- 逐读atlas/probe resource prepare、GPU Scene和WGSL，确认atlas cache只按AssetId、probe grid只按generation，GPU上传的generation word无shader consumer，raw单mipRGBA16F无压缩/streaming且atlas线性采样无gutter保护；
- 追踪Forward/Deferred组合，确认direct/ambient/IBL无Static/Stationary/shadowmask排除，Deferred将baked diffuse写入GBuffer emissive并在SSS中归入retained；
- 逐读Irradiance Volume core、plugin、streamer和shader，确认core/plugin重复collect/select/write，整view只绑定一个包含任意mesh origin的最高priority volume，layer只看camera，所有volume完整ensure且失败被吞；
- 对照Unreal MapBuildData/GPULightmass/VLM、Bevy Lightmap/Irradiance Volume、Godot LightmapGI、Fyrox CPU lightmapper/uvgen，以及Unity PathTracing Lightmap/APV producer、artifact、streaming与诊断边界。

另抽查7个独立产品/integration/contract test文件、2,503行、21个test属性；它们主要反序列化或手写bake output并手工注入Environment，不能证明scene-to-bake-to-artifact-reload-to-pixel闭环。本轮未运行Cargo、Editor、真实baker/GPU、cook、device loss、large-world streaming或同画质Unreal基准。相关asset/environment/renderer文件存在其他Session修改，09F2标记`source_recheck_required`；本轮未改production code，不能把静态审查记作Baked Lighting实现完成。

## 20. Hybrid Global Illumination 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| Hybrid GI runtime production Rust/WGSL | 221 / 29,714 | E3：plugin/provider、scene representation、prepare、GPU resource/readback、Surface Cache、SDF、RC、trace/resolve |
| dedicated tests / fixtures / integration | 39 / 12,021 | E2：138个test属性；CPU oracle、source contract、WGPU readback与ignored PNG exporter |
| editor / dist / manifest | 9 / 317 | E3：capability、registration、native ABI与authoring template断链 |
| core/editor integration spot check | 26 / 2,109 | E3：settings、visibility stub、post-process双owner、stats与viewport default |
| HGI architecture / usage evidence docs | 2 / 3,254 | E2：历史进展、视觉证据和current-source能力声明交叉核对 |
| production focused fingerprint | 221 / 29,714 | 95个inline test属性；`86d113db0a5e67cf23f77b3747f15cd5674748bd8af87506738736d8b278aa5b` |

Hybrid GI轮次详细阅读覆盖：

- 追踪`RenderHybridGiExtract`、provider、四pass feature graph和三个handoff shader，确认产品GI固定压缩为8x8 trace tile、graph packet只容纳16个Surface Cache page/4个voxel clipmap/64个cell，再铺回全分辨率；
- 逐读card/Surface Cache/texture/depth producer，确认每mesh只有一张card，CPU固定采样中心UV、手写PBR得到一个RGBA8并填满64x64 tile，depth也由bounds公式产生单一8-bit值；
- 追踪visibility、screen probe与voxel fallback，确认visibility HGI计划/feedback硬编码为空，scene representation旁路自建one-card/one-probe状态，旧voxel每层只有4x4x4 cells；
- 逐读Radiance Cache CPU/GPU链，确认CPU先从单色Surface Cache/voxel产生RGB8，GPU trace只把常量写入最多32个4x4 tile，filter/mip后consume到当帧probe，但最终cache/irradiance/trace/slot/tile仍全量异步回读；
- 复核新增Mesh/Global SDF，确认真实Mesh SDF payload已被GPU build三线性采样，但Global SDF仍有4 clipmap/128 page/32 candidate等小容量，trace nearest采样且无lineage时制造蓝灰色；Hardware RT只有enum/mask，production明确禁用；
- 追踪plugin resolve到core post-process，确认core又逐像素嵌套遍历probe/trace region重建第二份splat GI与history，并与plugin 8x8 texture及baked ambient组合，存在多owner、双计和`pixels * probes * regions`复杂度；
- 逐读editor/dist/manifest和viewport defaults，确认注册的`authoring.zui`不存在、测试只验字符串，Editor默认强制启用实验性HGI并用硬编码预算改写0值；
- 抽查三张现有PNG，确认只覆盖简单三角形/方块/色块；扫描34处`test_device()`，至少10处无adapter静默return，2026-08-10 debug-view PNG当前缺失；
- 对照Unreal Lumen card capture/Surface Cache/screen probe/Radiance Cache/software与hardware tracing，Godot SDFGI，Unity APV residency/editor，以及Bevy/Fyrox Rust资源边界。

本轮没有运行Cargo、Editor、WGPU、RenderDoc、device loss、VRAM pressure、动态序列或同画质Unreal benchmark。HGI production存在大量modified/untracked拆分文件，本轮标记`source_recheck_required`；静态审查和历史PNG不能记作Hybrid GI实现完成。

## 21. Volumetric Fog / Froxel 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| 直接production与最终consumer focused set | 50 / 10,855 | E3：typed contract、scene抽取、froxel shader/executor、history、light packing、deferred/forward消费、plugin与Editor surface |
| dedicated froxel/plugin tests与fixture | 15 / 4,709 | E2：60个test属性；CPU oracle、WGPU readback、ignored PNG/RenderDoc exporter |
| Volumetric/Froxel artifacts | 20 / 28,250,439 bytes | E2：2个RDC、binding/resource stats、PNG和文本报告；当前窗光束产品gate失败 |
| production focused fingerprint | 50 / 10,855 | 32个inline test属性；`c0b98ade19f50bd6881021787f7a83e1e6cef95e25e40ce0998b477a635e42e2` |

Volumetric Fog / Froxel轮次详细阅读覆盖：

- 追踪scene asset、typed settings、Light参与标记、Post Process Volume、camera layer与World extract，确认Sphere和rotated Box都在抽取时退化为world AABB，blend distance、priority、rotation与原始shape不进入GPU；
- 逐读Media Inject、Light Scatter、Integrate和apply WGSL/executor，确认真实三段froxel链、Beer-Lambert积分、clustered light/shadow消费和forward/deferred/sky最终采样可保留；
- 复核grid/resource lifetime，确认Low/Medium/High固定160x90x48/64/96且不跟viewport、宽高比或dynamic resolution；alias后High temporal仍约31.64 MiB/视图，设置和Editor无预算真值；
- 复核介质注入，确认全局雾锚定绝对Y=0，每个froxel遍历所有uniform additive AABB local volumes，没有GPU culling/cap/overflow、shape SDF、fade、priority、texture/material、emissive或negative carve；
- 复核直接光散射，确认参与flag会让非volumetric cluster candidate进入遍历后早退，CPU使用`Vec::contains`，Rect Light忽略面积、cookie metadata未采样、ambient不接Sky/IBL/Baked/HGI；
- 逐读temporal reprojection，确认screen-pixel jitter直接加到froxel cell、首帧可抖动，history为nearest load、固定0.9且只按extinction差拒绝，没有motion/depth/normal/disocclusion/radiance clamp与mutation generation；
- 追踪OIT/transparent/SSS/transmission consumer，确认透明fragment在存入OIT前已应用camera-to-fragment雾，再与已雾化scene组合，缺少depth-segment介质合同；
- 读取Volumetric Fog runtime/editor plugin和Post Process workspace，确认只有能力登记和硬编码`PPV_CityGlobal`/queued feedback，没有可保存/撤销的Global/Local Fog authoring、gizmo、material、budget或debug；
- 抽查20个artifact，确认2026-07-11窗光束报告为`diagnostic_failed`、shaft brighter pixels为0且contrast为-0.000；RenderDoc只证明160x90x96资源/dispatch存在，不能关闭画质gate；
- 对照Unreal Volumetric Fog/volume material/local fog/light function、Unity HDRP LocalVolumetricFog/VBuffer、Godot FogVolume/material/gizmo、Bevy体积参数与Fyrox传统fallback边界。

本轮没有运行Cargo、Editor、WGPU或RenderDoc，没有重新生成旧失败artifact，也未执行camera cut、dynamic resolution、透明多层、动画density、大世界、stereo、VRAM pressure、device loss或同画质Unreal/HDRP benchmark。focused set中18个文件存在其他Session修改，09G1标记`source_recheck_required`；静态审查和资源存在不能记作Volumetric Fog产品完成。

## 22. Advanced Surface Lighting 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| production focused set | 93 / 15,053 | E3：scene/extract、material、compile routing、GPU resource/executor/shader、plugin runtime/editor |
| dedicated focused tests | 14 / 3,756 | E2：54个test属性，其中7个ignored exporter/capture |
| Advanced Surface artifacts | 10 / 25,461,676 bytes | E2：2个RDC、5个PNG、3个文本报告；仅证明简化场景路径 |
| production focused fingerprint | 93 / 15,053 | 58个inline test属性；`11f8ed352f54dd8504f495cb9ab9f8a0fa86a44f19367e39714eb3f0674b4618` |

Advanced Surface Lighting轮次详细阅读覆盖：

- 追踪MaterialAsset、World与`AdvancedLightingExtract`，确认普通World只写fog volume和volumetric light ID，Cookie/OIT/Planar赋值只存在于测试或手工extract；四组Editor plugin也只有capability字符串，无scene component、asset、inspector、gizmo、transaction、save/cook或runtime bridge；
- 逐读Cookie atlas plan/blit/resource与direct shader，确认固定1024 RGBA8、8x8/64个128像素cell，第65项静默截断，每个active frame整图清白并重绘全部ready texture，缺图仍发布metadata，无mip/gutter/HDR/IES/Area/Light Function与froxel compute消费；
- 逐读OIT store/resolve与transparent contributor，确认颜色用`pack4x8unorm`且resolve再次clamp，固定per-pixel capacity溢出直接丢片元、不可统计；默认4层1080p约71.19 MiB、4K约284.77 MiB，OIT只替换transparent mesh并手工纳入sprite，不接管particle、half-res或Transmission；
- 追踪Planar mirror camera、oblique clip、camera loop、filter和environment provider，确认全部probe共享单张1024 RGBA16F mip chain，而主视图按最小probe ID上传matrix/bounds，多probe时纹理/参数会错配；OnDemand mutation、visibility/importance/time slicing、per-probe generation和空间blend均缺失；
- 逐读SSS profile gather、deferred MRT、tile setup、indirect scatter和recombine，确认每有效像素R/G/B各64次、共192 candidate，重复depth/normal/world reconstruction；profile内嵌material且first-wins冲突可被折叠，无thickness/back-light/quality tier，Forward/MSAA会静默移除；
- 追踪Clearcoat/Anisotropy与Transmission，确认direct各向异性GGX和coat lobe真实存在，但anisotropic IBL缺失、coat F0固定；Transmission最多四次整屏RGBA16F copy，按命令数分组，shader只做normal.xy偏移，无Snell/depth/roughness mip，并对已雾化背景再次apply volumetric；
- 复核透明graph compile顺序，确认Transmission在OIT替换前从原透明pass复制出来，OIT/Transmission/particle/half-res/fog没有统一composition owner或depth-segment合同；
- 逐张读取10个artifact，确认OIT只覆盖640x360三平面LDR、Planar只覆盖单probe、SSS为差异极弱的平滑球、Cookie导出与Irradiance Volume混合、Advanced PBR只证明简化三球stage顺序；7个export/capture test为ignored；
- 对照Unreal Light Function Atlas/OIT/Planar/SSS/Translucency/Substrate，Unity HDRP Cookie Manager/HDProbe/Diffusion Profile/SSS/Refraction，Bevy HDR linked-list OIT与Transmission，Godot projector/SSS，以及Fyrox Rust renderer边界。

本轮没有运行Cargo、Editor、WGPU或RenderDoc，没有重导出当前源码artifact，也未执行HDR/overflow、particle/full-chain、双Planar probe、skin/back-light、rough glass、多层雾、4K/stereo/dynamic resolution、VRAM pressure、device loss或同画质Unreal/HDRP benchmark。focused set中26个文件存在其他Session修改，09G2标记`source_recheck_required`；静态审查和旧artifact不能记作Advanced Surface Lighting产品完成。

## 23. Temporal AA / Velocity / History / Upscaling 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| production focused set | 108 / 18,701 | E3：AA ABI、ViewFamily、history、submission、graph allocation、velocity/reactive producer、GPU executor与WGSL |
| dedicated focused tests | 9 / 4,382 | E2：55个test属性，其中1个ignored artifact exporter |
| 名称命中的历史artifact | 24 / 154,612 bytes | E1-E2：velocity、GI-MSAA及GI/volumetric temporal旁证；直接TAA/FXAA/SMAA/upscale artifact为0 |
| production focused fingerprint | 108 / 18,701 | 149个inline test属性；`7d2743c2adc128dc2c5d850cea728b8fe8250bb110d263bddc5b7c233cc5f292` |

Temporal AA / Velocity / History / Upscaling轮次详细阅读覆盖：

- 从requested AA追踪capability、effective mode、post-process stack、ViewFamily validation、compile options、resource descriptor、persistent history、previous state、velocity/reactive producer、GPU executor、WGSL、history flip、stats与artifact exporter；
- 确认`FrameHistoryValidationKey`比较完整camera、mesh transform/material、lighting、animation pose、post-process、particles与feature list，正常motion得到`FrameInputsChanged`并使TAA history失效，已有camera/object/particle velocity因而不能在连续变化帧积累；
- 确认ViewFamily虽在submission build中被计算并验证phase，却没有通过builder写入`FrameSubmissionContext`，production也没有getter consumer；resource descriptor仍按render/display标量二分；
- 确认动态scale下`TAA_OUTPUT`为render size、TAA双缓冲history为display size，单一render pass混绑两个extent，shader又按display coordinate直接load低分辨率scene/depth/velocity，无input/output transform或temporal reconstruction；
- 确认camera velocity executor把display target size用于render-size depth/velocity，scene uniform current matrix使用render region而previous helper使用frame viewport，subrect/DRS坐标合同不一致；
- 确认particle velocity executor与GPU路径存在，但production core particle descriptor只插color pass，带velocity descriptor仅来自test fixture；transparent/particle/sprite/compositor reactive/velocity覆盖不完整；
- 确认capability硬编码SMAA/CAS/DLSS关闭、MSAA最大1；graph可编译4x descriptor，但全部scene pipeline仍是single-sample且没有任何`resolve_target: Some`；
- 逐读TAA/FXAA/SMAA/upscale WGSL，确认TAA为nearest history、current-neighborhood depth delta、单color history与8个阈值，FXAA为5-tap edge blur，SMAA无area/search LUT与pattern/diagonal/corner，upscale只有bilinear sample；
- 对照Unreal TAA/TSR/Velocity/DRS、Unity STP与upscaler provider、Godot TAA/FSR2、Bevy SMAA/TAA/CAS及Fyrox FXAA 3.11实现。

本轮没有运行Cargo、Editor、WGPU或RenderDoc，没有重导出artifact，也未执行连续camera/object/skin/morph/particle运动、disocclusion、HDR、dynamic resolution、split viewport、4K/stereo/XR、device loss、GPU timing或同画质benchmark。focused set中33个文件存在其他Session修改或untracked状态，09H1标记`source_recheck_required`；静态审查不能记作时域/AA产品完成。

## 24. Exposure / Color / Bloom / DOF / Motion Blur / SSR / Terminal 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| production focused set | 239 / 25,878 | E3：authoring schema、Volume evaluation、pass graph、resource format、executor、WGSL、camera stack与terminal output |
| dedicated focused tests | 25 / 9,469 | E2：96个test属性，其中1个ignored artifact exporter |
| 名称命中的历史artifact | 67 / 48,982,850 bytes | E1-E2：26个exit、32个log、1个PNG、8个RDC；均早于当前源码指纹 |
| production focused fingerprint | 239 / 25,878 | 185个inline test属性；`cd83813b175061a628d312e382771831816d8d2da2c9af2eb9bd24492d7d4dc9` |

Exposure / Color / Bloom / DOF / Motion Blur / SSR / Terminal轮次详细阅读覆盖：

- 从scene camera/volume asset追踪project I/O、World ECS、shape/layer extraction、per-camera evaluation、camera-loop restore、effective settings、stack/graph、resource/pipeline/bind group、WGSL、history/readback、terminal/UI、stats、product test与artifact exporter；
- 确认启用tone/grading/LUT时只烘焙normalized 32³ LUT，Uber采样前把HDR scene color clamp到`[0,1]`；未启用时默认`TonemapOperator::None`又将HDR写入固定Rgba8 LDR；
- 确认公开`SrgbNonlinear/LinearExtended/Hdr10Pq` output enum没有production consumer，output-transfer shader只做整数坐标copy，无PQ/gamut/paper-white/metadata/display negotiation；
- 确认64-bin histogram与percentile/adaptation是真实compute，但delta time硬编码1/60秒，曝光复用09H1全局history invalidation，且没有metering mask/local exposure/physical calibration；
- 确认runtime有15个typed Volume descriptor，scene profile却只持久化volumetric fog/bloom/color grading/effect stack四类；Exposure、DOF、Motion Blur、SSR、LUT、Blur等无法完整save/reload/cook；
- 确认camera stack逐camera提交、独立history/output owner存在，但Overlay只改变camera descriptor，复用最初提取的base post-process settings；
- 确认Bloom为full-res 25-load 5x5且threshold产生超线性亮斑，DOF为heuristic CoC/full-res固定24 gather/12px硬上限，general Blur复用同一昂贵family；
- 确认Motion Blur有三级tile/neighbor max基础，但shutter字段混用fraction/degrees且final只读neighbor-max velocity，易跨surface拖影；
- 确认SSR有HZB/pyramid/refinement/temporal/specular-occlusion基础，但trace仍为最多128次线性等距步进，无tile classification/GGX/可靠denoise，scene composite又绕过material BRDF与probe/IBL fallback；
- 对照Unreal Eye Adaptation/Local Exposure/Combine LUT/Diaphragm DOF/Motion Blur/SSRT/Device Encoding、Unity HDRP PostProcessing、Godot SSR/DOF/luminance/tone、Bevy exposure/bloom/motion/tone以及Fyrox bloom/HDR。

本轮没有运行Cargo、Editor、WGPU或RenderDoc，没有重导出artifact，也未执行HDR monitor、exposure动态序列、Bloom firefly、foreground DOF、fast-thin motion、rough/offscreen SSR、camera stack、4K/stereo/XR、device loss或同画质benchmark。focused set中42个文件存在其他Session修改或untracked状态，09H2标记`source_recheck_required`。2026-07-05旧日志虽记录12个post-process product tests通过，唯一旧PNG却含大面积洋红/青色颗粒；2026-08-01 current-source exporter又在session coordinator超时，均不能证明当前颜色/效果产品完成。

## 25. Runtime UI Architecture / Tree / Layout / Input / Accessibility 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| `zircon_runtime/src/ui` architecture production | 370 / 70,644 | E3：module、tree、layout、surface、dispatch、binding、event、component、template/v2、accessibility、platform input |
| `zircon_runtime_interface/src/ui` architecture production | 196 / 17,732 | E3：tree/layout/focus/navigation/input/window/a11y/component/template/ECS DTO；text/render延后 |
| dedicated architecture tests | 372 / 89,208 | E2：1,482个test属性、1个ignored；关键行为交叉核对 |
| deferred UI text/render integration | 120 / 30,333 | queued：11B/11C |
| dynamic product bridge | 6 / 2,952 | E3：5 modified + `runtime_ui.rs` untracked，实施前source recheck |
| production architecture fingerprints | 370 + 196 | Runtime `414e14cd2dd03f73f10886b93d58d5d2aa26a4ebb2b2ef34e773f5316afe6e54`；Interface `6e3f27c235509e9699e9aa300b88a73ff484f6d55ada588ad9eea9af8b249359` |

Runtime UI architecture轮次详细阅读覆盖：

- 从project manifest `ui_roots`追踪prototype store、v2 surface build、dynamic session construction、multi-surface render/input/accessibility、host request drain和window/lifecycle event；
- 确认产品把`UiInputDispatchResult`压成bool，component/binding/clipboard/popup/tooltip/pointer lock/link/IME host request全部丢失，gameplay没有权威action/data-binding接口；
- 确认每surface `UiInputManager`从不tick、timestamp恒0，dynamic bridge没有调用已有window pump，DPI/raster scale/focus loss/application deactivate/close与UI脱节；
- 确认`UiRuntimeDriver`为空、`UiConfig.enabled`无消费者，module event manager只被Editor使用，真实surface owner位于未跟踪dynamic session文件，test manager又是`#[cfg(test)]`；
- 逐读incremental layout/Taffy bridge，确认局部subtree patch真实存在，但每container临时创建direct-leaf Taffy tree后丢弃，不是persistent graph；mixed backend和uniform grid只提供近似语义；
- 逐读virtual list/scroll/pool，确认全部child仍物化和measure/arrange，visible range只隐藏subtree；pool主要为Editor特定row bridge且无通用budget/eviction；
- 逐读Tree mutation与hit grid，确认public graph可绕过transaction，顺序insert为O(N²)，递归路径缺统一cycle/depth guard；hit grid没有finite/checked cell/bytes budget，可被巨大bounds触发overflow/OOM；
- 逐读focus/navigation/popup，确认每event全树候选收集/排序、多个公开boundary/restore/group契约未消费，modal/popup依赖MUI组件名和属性alias；
- 逐读accessibility extract/action/AccessKit converter，确认neutral snapshot/action基础真实，但产品只有JSON capture，无OS window adapter、incremental TreeUpdate/action callback/focus lifecycle，role/relation/live schema不足；
- 反查UI ECS和WorldSpaceSurface，确认前者只是昂贵diagnostic projection，后者只有catalog/capability/tests，无scene/camera/render/ray-hit产品consumer；
- 对照Bevy persistent UiSurface/Taffy schedule、Fyrox UserInterface dt/update、Unreal SlateApplication/invalidation/platform accessibility、Godot DisplayServer IME/AccessibilityServer。

本轮没有运行Cargo、Editor、产品、屏幕阅读器、WGPU、真实IME/触控/手写笔或性能采样。architecture范围有68个dirty production文件，dynamic bridge 6/6均modified/untracked；11A标记`source_recheck_required`。详细发现与7 P0、31 P1、8 P2见`zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md`。

## 26. 测试形态快照

`core::runtime` 的 activation/registration/resolution 三组测试共有 96 个 `#[test]`，同时出现 94 次 `include_str!` 和 641 次 `.contains(...)`。这说明测试数量不能直接代表行为置信度：大量断言锁定源文件文本、文件拆分和 1-5 元特化分支，而非并发状态机、停机闭环和活对象语义。

本轮已确认有行为覆盖：单线程四阶段 lifecycle、ready timeout、finish 失败 rollback、批量拓扑顺序/rollback、lazy resolution 并发、跨线程依赖环、factory 与 unload 竞争、已实例化外部依赖阻塞。

本轮未发现有效行为覆盖：同一模块 activate/activate 与 activate/deactivate 竞争、真实 `cleanup` 后 observer veto 回滚、单模块 activation 的 dependency closure、同 kind 服务反向拓扑、产品退出触发全量 cleanup、卸载后外部强引用的可撤销性。

## 27. 后续扫描队列

1. Runtime systems：Physics、Audio、Animation、Navigation与Network已完成首轮E3静态审查；script/plugin runtime已完成首轮公共控制面审查，feature内部算法随所属系统复核。
2. Graphics/UI：RHI、Render Graph、GPU lifetime、renderer/visibility/GPU Scene、material/shader/pipeline/PSO、render asset streaming/residency、direct lighting/clustered light grid/shadow、environment/sky/IBL/reflection probe、baked lighting/lightmap/irradiance volume/offline bake、Hybrid GI、Volumetric Fog/Froxel、advanced surface lighting、temporal AA/velocity/history/upscaling、exposure/color/bloom/DOF/motion blur/SSR/terminal composition，以及runtime UI architecture/tree/layout/input/accessibility已完成首轮E3静态审查；继续11B text/font与11C GPU UI renderer。
3. Host/ABI/Plugin：`zircon_app`、`zircon_runtime_interface`、dynamic library/session、plugin SDK剩余面。
4. Editor/Hub：authoring transaction、viewport/runtime bridge、content workflow、项目与引擎管理。
5. Tooling：workspace、derive/codegen、验证器、CI、打包与长期性能基线。
