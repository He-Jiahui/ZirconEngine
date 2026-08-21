# 全量审查覆盖账本

## 1. 用途

本账本记录“物理上读到了哪里”和证据等级，不记录里程碑验收结果。workspace总量数字是 2026-08-15 快照，最新MVP control-plane切片更新于 2026-08-16，均包含测试代码；工作区存在大量其他会话修改，因此每个深审单元在写入修复计划前仍需重新取指纹并复核重叠文件。

证据等级使用主计划的 E0-E4 定义。`queued` 仅表示已建立扫描范围，不表示不存在问题。

## 2. Workspace 快照

| package/domain | Rust 文件 | Rust 行数 | 当前深审状态 |
|---|---:|---:|---|
| `zircon_app` | 190 | 28,885 | 局部 E3：product host/bootstrap/session teardown/runtime entry及PBR viewer/evidence链；其余 queued |
| `zircon_runtime` | 7,643 | 1,081,110 | 局部 E3：core/runtime/resource/scene/platform/systems、graphics到terminal composition、runtime UI architecture及text/font/shaping/layout/editing/IME；其余 queued |
| `zircon_runtime_interface` | 442 | 50,545 | 七轮纵向E3：生产源码已归入ABI、DTO、UI、diagnostic/plugin、host output、project/Hub protocol及contract certification；source drift需重检 |
| `zircon_editor` | 5,711 | 563,950 | 局部 E3：retained UI、document/scene/asset/Inspector/plugin/Play/command/jobs/notifications/logging及Settings/Preferences/locale/appearance纵向切片；其余 queued |
| `zircon_plugins` | 2,817 | 221,355 | 局部 E3：plugin SDK、ZrVM real backend、first-party catalog与公共注册面；feature内部算法仍queued |
| `zircon_hub` | 130 | 38,964 | 五轮纵向E3：backend/web/remote/control-plane/application-host；其余queued |
| `zircon-engine-derive` | - | 870 | E0 |
| `cargo-zircon` | - | 4,310 | E0 |

这些数据只用于控制覆盖面，不能用于评价质量。后续每个大域必须把 production、tests、build scripts、features、examples/fixtures 和产品调用点分开统计。

### 2.1 `zircon_runtime_interface` 七轮覆盖补记

截至2026-08-19，Interface 01-06已将当前production源码全部归入至少一份纵向报告；Interface07随后独立审查此前没有作为owner对象的契约测试与认证基础设施，而不是把测试数量算作实现完成。07的中央`src/tests`范围为35文件、13,563行、234项test、0 ignored；全crate识别401项test，分布于72个含测试Rust文件。加上9个跨crate生产者/消费者测试文件后，可确定性重建的冻结集合为81文件、25,568行、899,573 bytes、496项selected test与2项ignored，source fingerprint为`eea8fdb2c7e7d9042f381d8c8995f6d53675503137b6fc7f9937b9bc3a093f1e`；结构审计脚本和CI配置不混入该统计。

本轮确认的认证边界是：same-version Rust round-trip、本机layout、linked symbol与fake table测试均有局部价值，但没有required lane正向加载构建出的真实Runtime DLL并完成BuildSet skew、跨语言header/consumer、unload/reload、guard-page/fault、golden corpus和fuzz资格。18个参考文件共17,485行、613,618 bytes，fingerprint为`5028e6d929c5a0199c22039386b6cda8458e66be18488d087ecce14b5d2d33e5`；覆盖Godot生成接口与历史C extension兼容、Unreal BuildId/module manifest与test taxonomy、Bevy compile-fail、Fyrox Rust dylib边界以及Unity Graphics package API validation/serialization。详细差距和32项资格门见`zircon_runtime_interface/07-contract-certification-abi-layout-version-skew-cross-language-fuzz-test-architecture-review.md`。

07为review-only；未运行Cargo、真实DLL、C/C++、Miri/sanitizer/fuzz或性能。结构审计脚本在184.1秒超时且没有结果，不计作动态证据。选定Editor real-runtime ABI源码存在其他Session修改，因此七轮Interface报告仍必须在实施前重取source fingerprint。

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
| symbol-bearing current-source总语料 | 264 / 65,285 | E3：asset、Scene submit、neutral contract、plugin、graph、GPU、shader、Editor package与tests；fingerprint `64a037a6b3142818fddf2b53af64dbb12e21ec60e1b0a78ec6a896e5f22249aa` |
| production-like源码 | 196 / 45,254 | E3：产品数据链与294个内嵌test属性；fingerprint `cce29358bc29171c62204c2fd9117d3f3d82b62981d28eea25f3c846cf215923` |
| focused tests与support | 52 / 19,762 | E3：355个test属性，其中10个ignored；fingerprint `785e753ac6d9ecfbbf1d917f3614fd236cd723cf8d61a15f977cc15266b4711c` |
| 四组Editor feature packages | 16 / 269 | E3：只有manifest、capability、registration和descriptor壳；fingerprint `87674a393041cf5787e7135b5e04081b2b1341344438b111b504cbab97fa5eb1` |
| Advanced Surface artifacts | 10 / 25,461,676 bytes | E2：2个RDC、5个PNG、3个TXT；fingerprint `6486960b17d651bc6ee2051bb688fc6cfbb931d30aa3a6b78a27a4f438815792` |
| 五引擎参考切片 | 33 / 19,731 | E2-E3：Unreal、Unity HDRP、Bevy、Godot、Fyrox；fingerprint `c402aaa662740a8a0af0a3a563269c705c5270fc7bdad9a11180c183f59975a6` |

Advanced Surface Lighting轮次详细阅读覆盖：

- 追踪MaterialAsset、World与`AdvancedLightingExtract`，确认普通World只写fog volume和volumetric light ID，Cookie/OIT/Planar仍无Scene/project producer；同时纠正旧描述：高级PBR已按selected-camera layer解析最多4级material parent，SSS也会从生产mesh/material补profile，Planar camera loop已能派生reflection camera、提交capture并在成功后mark captured；四组Editor plugin仍只有capability/descriptor壳；
- 逐读Cookie atlas plan/blit/resource与direct shader，确认固定1024 RGBA8、8x8/64个128像素cell，第65项静默截断，每个active frame整图清白并重绘全部ready texture，缺图仍发布metadata，无mip/gutter/HDR/IES/Area/Light Function与froxel compute消费；
- 逐读OIT store/resolve与transparent contributor，确认颜色用`pack4x8unorm`且resolve再次clamp，固定per-pixel capacity溢出直接丢片元、不可统计；默认4层1080p约71.19 MiB、4K约284.77 MiB，OIT只替换transparent mesh并手工纳入sprite，不接管particle、half-res或Transmission；
- 追踪Planar mirror camera、oblique clip、camera loop、filter和environment provider，确认全部probe共享单张1024 RGBA16F mip chain，而主视图按最小probe ID上传matrix/bounds，多probe时纹理/参数会错配；OnDemand mutation、visibility/importance/time slicing、per-probe generation和空间blend均缺失；
- 逐读SSS profile gather、deferred MRT、tile setup、indirect scatter和recombine，确认每有效像素R/G/B各64次、共192 candidate，重复depth/normal/world reconstruction；profile内嵌material且first-wins冲突可被折叠，无thickness/back-light/quality tier，Forward/MSAA会静默移除；
- 追踪Clearcoat/Anisotropy与Transmission，确认direct各向异性GGX和coat lobe真实存在，但anisotropic IBL缺失、coat F0固定；Transmission最多四次整屏RGBA16F copy，按命令数分组，shader只做normal.xy偏移，无Snell/depth/roughness mip，并对已雾化背景再次apply volumetric；
- 复核透明graph compile与resource resolver，确认OIT/Transmission/particle/half-res/fog没有统一composition owner或depth-segment合同；新增独立P0：OIT executor读取Volumetric Integrated与Transmission Scene Color，但descriptor未声明这两个资源，resolver确定性返回None；
- 逐张读取10个artifact，确认OIT只覆盖640x360三平面LDR、Planar只覆盖单probe、SSS为差异极弱的平滑球、Cookie导出与Irradiance Volume混合、Advanced PBR只证明简化三球stage顺序；更宽语料共10个ignored attribute；
- 对照Unreal Light Function Atlas/OIT/Planar/SSS/Translucency/Substrate，Unity HDRP Cookie Manager/HDProbe/Diffusion Profile/SSS/Refraction，Bevy HDR linked-list OIT与Transmission，Godot projector/SSS，以及Fyrox Rust renderer边界。

旧09G2的12项P0全部保持开放，其中Scene producer与Planar lifecycle只记录局部进展、不误判关闭；Runtime100新增1项OIT graph-resource P0，另登记36项P1、8项P2与44项资格门。目标是`AdvancedLightingSceneCompiler + AdvancedLightingFeatureResolver + LightModulationAtlasService + TransparentCompositor + PlanarReflectionService + DiffusionProfileService + LayeredPbrService + TransmissionTracingService + AdvancedLightingResource/Diagnostics/Authoring`。

本轮没有运行Cargo、Editor、WGPU或RenderDoc，没有重导出当前源码artifact，也未执行HDR/overflow、particle/full-chain、双Planar probe、skin/back-light、rough glass、多层雾、4K/stereo/dynamic resolution、VRAM pressure、device loss或同画质Unreal/HDRP benchmark。相关current blob存在并发Session modified标记，报告保留`source_recheck_required`；静态审查和旧artifact不能记作Advanced Surface Lighting产品完成。详见`zircon_runtime/99a-runtime-advanced-surface-lighting-light-cookie-oit-planar-reflection-subsurface-scattering-clearcoat-anisotropy-transmission-product-integration-current-source-review.md`。

## 23. Temporal AA / Velocity / History / Dynamic Resolution / Upscaling / Reconstruction / Product Integration 物理范围

| 子域 | 文件 / 行 / bytes / test attributes / ignored | 本轮状态 |
|---|---:|---|
| current-source产品语料 | 193 / 45,449 / 1,713,638 / 344 / 0 | E3：AA ABI、ViewFamily、history、graph/resource、velocity/reactive、GPU executor/WGSL、Scene与Editor入口；fingerprint `cbd39439f719dbacae5b6adec7cfd1fd99e95f1e8a28e7a21326ef10a42f4081` |
| focused tests与support | 11 / 5,009 / 179,635 / 42 / 2 | E3：history、AA、particle velocity、reactive、motion blur与full-chain；fingerprint `790818a6b72a836ab3c13396aec5651218885a8c72cb3f716b5194e7713aa3f9` |
| 名称命中的历史artifact | 24 / 未归一 / 154,612 / 未归一 / 未归一 | E1-E2：velocity、GI-MSAA及GI/volumetric temporal旁证；直接TAA/FXAA/SMAA/upscale artifact为0；fingerprint `3dfbe697212347cedfaef1ebaa92297c63f59fe4d01408c34e6e98ea5e9464c8` |
| 五引擎参考切片 | 28 / 17,477 / 750,409 / 未归一 / 未归一 | Unreal、Unity Graphics、Godot、Bevy与Fyrox；fingerprint `f109744c7352ce0b180a6a51f386002c1dee561538d21a487bf3adba27353be1` |

Temporal AA / Velocity / History / Upscaling轮次详细阅读覆盖：

- 从requested AA追踪capability、effective mode、post-process stack、ViewFamily validation、compile options、resource descriptor、persistent history、previous state、velocity/reactive producer、GPU executor、WGSL、history flip、stats与artifact exporter；
- 确认`FrameHistoryValidationKey`比较完整camera、mesh transform/material、lighting、animation pose、post-process、particles与feature list，正常motion得到`FrameInputsChanged`并使TAA history失效，已有camera/object/particle velocity因而不能在连续变化帧积累；
- 确认ViewFamily虽在submission build中被计算并验证phase，却没有通过builder写入`FrameSubmissionContext`，production也没有getter consumer；resource descriptor仍按render/display标量二分；
- 确认动态scale下`TAA_OUTPUT`为render size、TAA双缓冲history为display size，单一render pass混绑两个extent，shader又按display coordinate直接load低分辨率scene/depth/velocity，无input/output transform或temporal reconstruction；
- 确认camera velocity executor把display target size用于render-size depth/velocity，scene uniform current matrix使用render region而previous helper使用frame viewport，subrect/DRS坐标合同不一致；
- 确认particle velocity executor与GPU路径存在，但production core particle descriptor只插color pass，带velocity descriptor仅来自test fixture；transparent/particle/sprite/compositor reactive/velocity覆盖不完整；
- 确认capability硬编码SMAA/CAS/DLSS关闭、MSAA最大1；graph可编译4x descriptor，但全部scene pipeline仍是single-sample且没有任何`resolve_target: Some`；
- 逐读TAA/FXAA/SMAA/upscale WGSL，确认TAA为nearest history、current-neighborhood depth delta、单color history与8个阈值，FXAA为5-tap edge blur，SMAA无area/search LUT与pattern/diagonal/corner，upscale只有bilinear sample；
- 确认`ViewportRecord`的camera history、motion、particle与多个temporal runtime map没有统一last-used/eviction/remove生命周期，temporal frame index又是viewport全局而非per-history generation；
- 确认Scene camera只持久化`msaa_samples`，World snapshot没有接入持久DRS，Editor没有AA/TAA/DRS/provider authoring或runtime diagnostics consumer；
- 对照Unreal TAA/TSR/Velocity/DRS、Unity STP与upscaler provider、Godot TAA/FSR2、Bevy SMAA/TAA/CAS及Fyrox FXAA 3.11实现。

旧09H1的10项P0全部保持开放并由其唯一计数；Runtime101另登记31项P1、10项P2与44项资格门。目标是`ViewFamilyService + ViewHistoryRegistry + MotionProducerRegistry + ReactiveCompositionService + TemporalReconstructionService + UpscalerProviderRegistry + DynamicResolutionService + SpatialAntiAliasService + MultisampleSurfaceService + TemporalDiagnostics/Authoring`。

本轮没有运行Cargo、Editor、WGPU或RenderDoc，没有重导出artifact，也未执行连续camera/object/skin/morph/particle运动、disocclusion、HDR、dynamic resolution、split viewport、4K/stereo/XR、device loss、GPU timing或同画质benchmark。报告绑定当前working-tree fingerprint并保留`source_recheck_required`；静态审查不能记作时域/AA产品完成。详见`zircon_runtime/99b-runtime-temporal-aa-velocity-history-dynamic-resolution-upscaling-reconstruction-product-integration-current-source-review.md`。

## 24. Exposure / Color / Tonemap / LUT / Bloom / DOF / Motion Blur / SSR / Output Transfer / Terminal Composition 物理范围

| 子域 | 文件 / 行 / 非空行 / bytes / test attributes / ignored | 本轮状态 |
|---|---:|---|
| current production review slice | 270 / 33,972 / 31,610 / 1,334,276 / 223 / 0 | E3主链与E2支持owner；fingerprint `2ecab8bc899a17a12711c9a8885c4c0fa213b457e6ebe6ed2bb27c3e71398b90` |
| dedicated relevant tests | 34 / 10,658 / 10,039 / 389,520 / 108 / 1 | 未执行；fingerprint `37f6cd44b7ee8b1e79668a798728d38c89bdce622007c5f35422a1961b36a929` |
| 旧artifact集合 | 67 / 未归一 / 未归一 / 48,982,850 / 未归一 / 未归一 | 58份旧日志、1 PNG、8 RDC；不绑定当前源码 |
| 五引擎参考切片 | 126 / 28,798 / 24,066 / 1,174,241 / 未归一 / 未归一 | Unreal、Unity HDRP、Godot、Bevy、Fyrox；fingerprint `25004219a1a78b7e3cd0ae76519a3c7fdff60e217e294aa6159ad50aa0374229` |

Runtime102详细阅读覆盖：

- 从Scene/Profile asset追踪project I/O、runtime registry、camera resolve、stack/graph、resource/pipeline、executor/WGSL、per-camera history、readback/stats、product tests/artifact与Editor workspace；
- 确认baked LUT仍在tone前clamp HDR，None仍写`Rgba8Unorm`，`Hdr10Pq/LinearExtended`未进入output shader、surface capability、gamut/EOTF、luminance或metadata；
- 确认Exposure已经按viewport-camera history handle隔离，撤销09H2“全局history串camera”描述；但delta仍固定`1/60`，没有metering mask、physical/pre-exposure、curve/local exposure和effect-specific lifecycle；
- 确认D3 LUT已真实导入、stream与绑定，撤销“3D LUT不可绑定”描述；但`.cube`忽略domain、拒绝1D shaper并量化到RGBA8，baked LUT固定32³，64³常量未进入质量计划；
- 确认Scene effect asset只保存Tonemap/Vignette/Grain/Dither/Chromatic/Fog，save/load会静默丢LUT、Blur、Motion Blur、DOF、SSR，Scene component又没有Exposure；
- 确认Editor Post Process workspace硬编码Bloom/Filmic/LUT/Exposure样例，Preview/Apply只返回canned queued字符串，没有Scene transaction、undo、asset picker或runtime mutation；
- 确认stack同时生成DOF/Motion/SSR Scene Composite/Blur与Uber，uber仍使用未mask参数重复执行；Bloom pass和uber又重复乘同一intensity；
- 确认Bloom仍为full-res 25-load，DOF为heuristic CoC/full-res固定gather，Motion Blur混用fraction/degrees且以neighbor-max作为中心方向；
- 确认SSR已有HZB、pyramid、refine、roughness与temporal基础，但reprojection仍读neighbor-max，history clamp基于scene RGB，visibility硬乘0.18/封顶0.35，最终无material F0/Fresnel/BRDF/fallback；
- 对照Unreal真实delta/local exposure/working-output device与scalable effects、Unity HDRP分层stage、Godot层级SSR、Bevy metering/mip Bloom/shutter单位以及Fyrox HDR/Bloom baseline。

旧09H2的8项P0均未完全关闭，其中P0-4只有per-camera history部分修复；Runtime102新增1项split/uber重复执行P0，另登记40项P1、10项P2与44项资格门。目标是`ColorPipelinePlan + ExposureService + PostProcessComponentRegistry + PostProcessExecutionPlanner + Scalable Effect Services + IndirectSpecularCompositor + OutputDeviceService + PostProcessAuthoringService`。

本轮没有运行Cargo、Editor、WGPU、RenderDoc或参考引擎，没有重导出artifact；源码声明的`20260801`两张PNG与JSON仍不存在。未执行HDR monitor、曝光动态序列、Bloom firefly、foreground DOF、fast-thin motion、rough/offscreen SSR、camera stack、4K/stereo/XR、device loss或同画质benchmark。详见`zircon_runtime/99c-runtime-exposure-color-tonemap-lut-bloom-dof-motion-blur-ssr-output-transfer-terminal-composition-product-integration-current-source-review.md`。

## 25. Runtime UI Architecture / Tree / Layout / Input / Accessibility 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| `zircon_runtime/src/ui` architecture production | 370 / 70,644 | E3：module、tree、layout、surface、dispatch、binding、event、component、template/v2、accessibility、platform input |
| `zircon_runtime_interface/src/ui` architecture production | 196 / 17,732 | E3：tree/layout/focus/navigation/input/window/a11y/component/template/ECS DTO；text/render延后 |
| dedicated architecture tests | 372 / 89,208 | E2：1,482个test属性、1个ignored；关键行为交叉核对 |
| deferred UI text/render integration | 120 / 30,333 | text已进入11B；GPU render仍queued到11C |
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

## 26. Runtime Text / Font / Shaping / Layout / Editing / IME 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| `zircon_runtime/src/text` 全部 | 284 / 62,681 | E3：font、shaping、layout、raster、SDF、atlas、artifact、cache、rich text与service |
| `zircon_runtime/src/text` production paths | 210 / 40,180 | fingerprint `7761821f150e57d0a1c288301f2988e99b2ef3b07c33333c0e70d61c08fa306e` |
| `zircon_runtime/src/ui/text` 全部 | 64 / 13,247 | E3：shared shaper、layout engine、viewport、geometry、grapheme与rich text |
| `zircon_runtime/src/ui/text` production paths | 33 / 8,307 | fingerprint `849ddf990ae0723f3f544f6c7dba5d84bafe2a4c9bba60c4a90cb78803f711e3` |
| text/IME interface focused set | 27 / 6,276 | E3：editable state、selection/caret/composition、IME event/host request、render/layout DTO |
| combined focused set | 375 / 82,204 | E2-E3：1,029个test attributes、10个ignored；未运行产品/平台验收 |

Runtime Text轮次详细阅读覆盖：

- 从font manifest/source/import追踪artifact cache payload、project file collection和runtime source resolution，确认import读取原始font bytes却不写入artifact，raw font又被排除为source auxiliary，runtime最终重新打开project source；
- 逐读shared font database、fallback、mutation、retired face和Cosmic thread-local cache，确认process-global可变真值、系统字体/源码树默认依赖、整库clone/粗失效和per-thread DB复制；
- 逐读Rustybuzz/Cosmic/vertical shaping、normalization、BiDi与hard-line split，确认真实Unicode/竖排基础可保留，但全backend失败会生成无font handle的synthetic glyph，64 KiB cap会制造假行并可能切断context/grapheme；
- 追踪glyph artifact、raster/SDF face resolution和UI consumer，确认synthetic glyph仍被标记requires-rasterization，layout/hit-test可成功而renderer没有权威face；
- 逐读UI layout session、measure cache、viewport和intrinsic path，确认fast path只覆盖plain/horizontal/nowrap/clip/non-editable，wrapped editor、rich、vertical、preedit仍全量layout，大输入intrinsic extent按源字节构造；
- 逐读editable reducer、grapheme、keyboard/clipboard、focus与IME，确认无undo/redo transaction，selection/composition只保证char boundary，上下键按source line、RTL左右键按逻辑顺序，IME geometry缺失时使用等宽启发式；
- 追踪secure field到render command和clipboard，确认原文仍被绘制/copy/cut，当前安全策略只禁用IME，既泄漏密码又阻断国际输入；
- 逐读rich cache/parser/HTML/BBCode/Markdown/table/list/decorator，确认cache为256项/8 MiB且局部嵌套有上限，但入口无input/token/node/time预算，未闭合marker可重复扫描，Markdown只是未版本化的极小子集且无structured diagnostics；
- 复核`docs/plans/zircon_runtime/text` 01–09与25个failure handoff，确认Text08“Ctrl+Z/Y已存在”等完成结论与current source冲突，需要按11B重新打开；
- 对照Unreal FontBulkData/FontCache/HarfBuzz、Godot TextServer/TextEdit/LineEdit、Bevy font loader/TextError/text edit和Fyrox FormattedText/TextBox；Unity Graphics仓不含权威主文本引擎实现，未据此推断闭源行为。

本轮没有运行Cargo、Editor、clean package、真实系统font fallback、WGPU、TSF/IMM32/IBus或性能采样。text production paths当前未出现在工作区修改列表，但跨域asset/UI/render consumer存在其他Session修改，11B仍标记`source_recheck_required`。详细3 P0、29 P1、8 P2及11个验收gate见`zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md`。

## 27. GPU UI Renderer / Atlas / SDF / Batch / Clip / Submit 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| combined focused set | 216 / 60,415 | E3：game scene UI、runtime render/icon、interface paint/batch/cache、RHI native surface与WGPU backend |
| production focused set | 162 / 43,925 | fingerprint `b628534653d8f2b332019ad31b0379d89fda0c278735868630daafb0bf87ba05` |
| focused tests | 54 / 16,490 | E2：495个test attributes、0 ignored；未运行GPU/product gate |
| product renderer authorities | 2 | game `ScreenSpaceUiRenderer` 与Editor `WgpuUiSurfaceRenderer`，未共享presentation/font/icon/cache/color contract |

GPU UI轮次详细阅读覆盖：

- 从arranged `(z_index, paint_order,node_id)`顺序追踪`UiRenderExtract`、每command paint projection、七组plan Vec、WGPU record和Glyphon/bitmap/SDF内部顺序，确认backend分类fanout破坏跨primitive painter order；
- 追踪runtime semantic icon producer、`UiVisualAssetRef::Icon`、`UiIconAtlasBuilder`和scene image分支，确认CPU atlas plan无raster/upload/production consumer，正常Icon被画成居中实心矩形；
- 逐读Editor retained present、damage scissor、pipeline blend和copy，确认DamagePatch使用Load在旧像素上premultiplied blend且不恢复背景，半透明重复更新与删除没有full-redraw等价保证；
- 逐读interface brush/batch/cache/clip/debug contract，确认rounded/gradient/vector/material、stencil、draw effect、`UiBatchPlan`和`UiRenderCachePlan`没有进入game产品提交；
- 逐读scene image、shape buffer、generation hash、Glyphon、native bitmap与SDF atlas，确认每帧plan/geometry/JSON generation/full payload hash、逐image draw、固定SDF产品预算及多text route顺序；
- 逐读RHI dependency batching、generation compiled cache、text、image registry和Editor draw-list producer，确认安全重排与bounded cache基础真实，但普通live full/damage明确无generation，稳定cache只覆盖特殊versioned路径；
- 逐读Editor icon atlas，确认process-global 64页/64 MiB owner与changed page即时sealed，跨frame动态发现会碎片化页面；
- 复核game sRGB final target与hex颜色、Editor non-sRGB UNORM surface、retained 4K全图copy，确认颜色空间、HDR/output和约31.64 MiB/次copy带宽缺口；
- 对照Unreal Slate batch/clip/atlas、Bevy UI render、Godot canvas、Fyrox draw/brush与Unity Graphics atlas；Unity Graphics仓不含UI Toolkit renderer，未推断闭源行为。

本轮没有运行Cargo、Editor、WGPU、RenderDoc、HDR或device-loss测试，也未执行同画质Unreal benchmark。focused production set未出现在当前工作区修改列表，但跨域Editor/test有其他Session修改，11C标记`source_recheck_required`。详细3 P0、30 P1、8 P2及四阶段重构gate见`zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md`。

## 28. Zircon App Product Host / Bootstrap / Dynamic Runtime / Shutdown 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| production focused set | 117 / 14,947 | E3：entry/bootstrap、runtime/editor runner、dynamic library/session、runtime app、plugin group与两个产品binary |
| focused tests | 56 / 6,986 | E2：396个test attributes、0 ignored；180次`include_str!`说明source guard占比过高 |
| combined focused set | 173 / 21,933 | fingerprint `743cb2c27f4ca99e3f325cf1a711d886cb840b4aae7b853abf094ba3bd0f5ed1` |
| crate-level integration | 3 files / 4 tests | 仅一个真实Editor authoring restart；无runtime/server/DLL/surface/shutdown产品矩阵 |

Zircon App product host轮次详细阅读覆盖：

- 从`EntryConfig`追踪module/plugin selection、`BuiltinEngineEntry`、Core activation、Editor/runtime/export/automation composition与owner返回；
- 确认`target-server`没有binary，`run_headless()`只bootstrap后返回；client内的headless/minimal仍创建Winit并固定16ms tick；
- 复核Cargo feature依赖，确认required `target-client`强制desktop default-platform、X11/Wayland、输入和dynamic DLL，Web/Android附加feature没有形成独立产品artifact；
- 逐读DLL path、V6 table validation、Editor gateway table、session create/destroy、owned buffer、operation、wake registry和teardown failure，确认library/frame/buffer owner基础真实，但host API为空、ABI size策略矛盾、surface只有全局bool、Drop硬编码viewport 1且多类foreign output无预算；
- 从Winit ApplicationHandler追踪window create/event/input/host request/cadence/surface native/fallback/resize/redraw/drop，确认WindowId/DeviceId被丢弃、Destroyed不清owner、suspend/exiting缺失、native surface失败无恢复、CPU fallback整帧readback；
- 追踪Play CLI到runtime V3 session和Editor process backend，确认`--play-report-pipe`仅生成带outlet标签的stdout文本，Editor不解析phase，runtime又在首帧前报告ready；
- 追踪binary crash flush、runner terminal merge和log shutdown，确认session destroy失败abort是合理最后防线，但没有process-wide quiesce/drain/deactivate/flush coordinator，log shutdown结果也被忽略；
- 对照Unreal GuardedMain/FEngineLoop、Bevy App runner/Winit state、Fyrox normal/headless executor和Godot setup/iteration/cleanup；Unity Graphics仓不含主循环源码，未推断其闭源lifecycle。

本轮没有运行Cargo、真实binary/DLL、signal、Android/Web、multiwindow、surface/device loss或同负载benchmark。focused production set成文前未出现在工作区修改列表；详细4 P0、27 P1、8 P2及14个验收gate见`zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md`。PBR viewer明确排除，由下一节独立拥有。

## 29. Zircon App PBR Viewer / Evidence / RenderDoc 物理范围

| 子域 | 文件/行数或体积 | 本轮状态 |
|---|---:|---|
| viewer production | 14 / 5,886 | E3：CLI、Winit lifecycle、后台load、generated project、scene/render/present、PNG/sidecar、GPU timing、RenderDoc |
| focused `app_tests.rs` | 1 / 689 | E2：29个测试；不创建真实EventLoop/window/SceneRenderer/RenderDoc session |
| viewer tests total | 126 test attributes / 0 ignored | E2：纯函数、临时文件和source contract为主；无真实GPU/native/product gate |
| immediate evidence tools | 7 / 4,487 | E3：profile contract/runner、ready/GPU/RenderDoc validator、summarizer、artifact receipt |
| tracked shader corpus | 182 / 624,277,393 bytes | E2：107 PNG、17 RDC；RDC占461,326,034 bytes，无LFS/current-source自动矩阵 |
| viewer production fingerprint | 14 / 5,886 | `37cfb1a281c6cd8fd95188591b164f91cb14d11b5f5aa981a3c35d60df24c325` |

PBR viewer/evidence轮次详细阅读覆盖：

- 从CLI/work path/RenderDoc preload追踪Winit ApplicationHandler、后台scene创建、Base PSO admission、camera/redraw和terminal exit，确认handler内致命错误只`event_loop.exit()`，`main`仍可返回退出码0；
- 逐读background task和scene Drop，确认scene内部有world/surface/renderer/Core依赖释放顺序，但spawn的JoinHandle被丢弃，无cancel/deadline/join，关闭期间不能确定性drain；
- 追踪generated project staging、manifest和ready predicate，确认staged rename基础可保留，但缓存只看六个文件存在，现有manifest不与recipe/content hash比对，崩溃残留无scavenger；
- 逐读offscreen/native render分支，确认传入screenshot时不会绑定native viewport surface；managed measured与RenderDoc run都请求screenshot，因此自动证据固定为CPU readback而非swapchain present；
- 追踪viewer composition，确认它直接创建最小CoreRuntime和environment-only preview的单镜面球工程，绕过ProductHost、动态DLL、插件目录、产品scene和完整deferred renderer；
- 逐读ready/GPU timing/RenderDoc validator，确认GPU generation/pass和RDC replay/SHA基础较强，但ready像素只要求两个颜色与一个非黑像素，没有golden、HDR线性、perceptual/local-global误差或平台容差；
- 追踪profile cold/warm/WPR/energy、binary/HDRI/source/artifact receipt，确认唯一run目录与hash基础真实，但artifact/report分散直接写入、无事务commit/terminal state，critical source只含14个viewer production文件中的6个；
- 统计历史shader corpus，确认182个tracked文件约595 MiB，其中17个RDC约440 MiB；没有内容寻址artifact store、retention/currentness index或current-source自动回归矩阵；
- 对照Unreal RenderCapture/ImageWriteQueue/Automation Screenshot，Unity Graphics LookDev/graphics tests，Bevy renderer-owned screenshot，Godot thread join与Fyrox task/executor lifecycle。

本轮没有运行Cargo、viewer、WGPU、WPR、RenderDoc、Editor或packaged runtime，也未重新生成artifact或执行同负载Unreal/HDRP benchmark。viewer与七个immediate tools成文前未出现在工作区修改列表；详细4 P0、26 P1、8 P2及16个验收gate见`zircon_app/02-pbr-viewer-tool-runtime-evidence-renderdoc-review.md`。

## 30. Runtime DLL ABI / FFI / Version / Handle / Foreign Ownership 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| interface ABI production | 17 / 3,204 | E3：runtime API table/carrier、buffer、handle、status、version、profiling；不是整个interface crate |
| runtime dynamic producer | 44 / 8,791 | E3：export/table/session/registry/event/surface/frame/operation/world/host/profile直接实现 |
| App runtime-library consumer | 9 / 3,226 | E3：load/table/session/frame/output/operation；成文期间有其他Session修改 |
| Editor gateway consumer | 12 / 1,030 | E3：session/library owner、owned buffer、world sync、plugin page |
| interface focused tests | 28 test attributes | E2：16 inline + 9 ABI safety + 3 runtime operation；未运行 |
| focused production fingerprints | 17 + 44 + 9 + 12 | interface `a7e00a6b...f9c0`；runtime `e36e2dee...a9a`；App `31354292...8563`；Editor `b8a1a0be...83b` |

Runtime DLL ABI轮次详细阅读覆盖：

- 从唯一导出`zircon_runtime_get_api_v6`追踪24字段/22个可选函数的V6 table、App exact version/size校验与函数可用性，确认hard cut没有Build Set ID、target/data model、feature/capability/schema fingerprint，同shape异语义构建会被误接受；
- 逐读host table与carrier版本，确认host支持判断只看version、不看size/callback，多数versioned struct没有size/reserved，capability只由空函数指针隐式表达；
- 追踪`ZrByteSlice::as_slice`到profile/project/startup/input/event/world/plugin和status diagnostics，确认null+nonzero被静默当空，非空slice在`isize::MAX`/预算检查前构造；
- 追踪producer `Vec`到`ZrOwnedByteBuffer`和App/Editor release，确认owning carrier为`Clone + Copy`，固定类别token和调用方可写pointer/len/capacity最终进入`Vec::from_raw_parts`，没有allocation registry或一次性release；
- 复核App在途`ForeignOutputState`，确认host/profile/operation/plugin已有结构、字节、条目和事后decode-time fuse基础可保留，但producer仍先分配，阻塞JSON解码也无法被事后时间检查抢占；
- 逐读frame capture、JSON输入输出和out parameter，确认任意`u32`维度可进入RGBA分配，多类payload没有producer-side byte/item/time/depth budget，frame buffer也缺stride/format/color space/alpha/origin/HDR metadata；
- 逐读17类扁平event carrier，确认wheel float复用key字段、window i32经f32中转、payload多义、有限值/unknown enum验证不一致，surface只表达Win32/native-none且产品路径固定default viewport 1；
- 追踪session registry open/closing/action/wake/Condvar与App/Editor library owner，确认in-flight drain和owner基础真实，但destroy没有deadline/cancellation，callback线程/重入/no-throw和DLL epoch合同不完整；
- 复核Rust layout/source-contract tests，确认当前192/96/48等断言只证明当前64-bit Rust布局，没有generated C header、compiled C/C++ consumer、target/skew矩阵、sanitizer/fuzz或OOM/hung-callback child-process gate；
- 对照Godot generated GDExtension interface、Unreal BuildId/module lifecycle、Fyrox动态Rust plugin风险声明与Bevy开发期dylib边界；Unity Graphics仓不含Player/native ABI权威源码，未推断闭源行为。

本轮没有运行Cargo、真实DLL/App/Editor、C/C++ consumer、sanitizer、fuzzer、跨构建skew、极端分辨率或callback hang测试。整个`zircon_runtime_interface`约442个Rust文件/50,544行，本轮只关闭stable runtime DLL C ABI切片；plugin ABI、serialization/project/hub、reflection/resource和UI/public DTO仍待审。runtime producer和App consumer成文期间持续有其他Session修改；App集合从初始3,014行/`88aed79f...6179`漂移到复取值，报告标记`source_recheck_required`。详细5 P0、30 P1、8 P2及M0-M3 gate见`zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md`。

## 31. 测试形态快照

`core::runtime` 的 activation/registration/resolution 三组测试共有 96 个 `#[test]`，同时出现 94 次 `include_str!` 和 641 次 `.contains(...)`。这说明测试数量不能直接代表行为置信度：大量断言锁定源文件文本、文件拆分和 1-5 元特化分支，而非并发状态机、停机闭环和活对象语义。

本轮已确认有行为覆盖：单线程四阶段 lifecycle、ready timeout、finish 失败 rollback、批量拓扑顺序/rollback、lazy resolution 并发、跨线程依赖环、factory 与 unload 竞争、已实例化外部依赖阻塞。

本轮未发现有效行为覆盖：同一模块 activate/activate 与 activate/deactivate 竞争、真实 `cleanup` 后 observer veto 回滚、单模块 activation 的 dependency closure、同 kind 服务反向拓扑、产品退出触发全量 cleanup、卸载后外部强引用的可撤销性。

## 32. Plugin SDK / Package / Catalog / Dist / Native Admission 物理范围

| 子域 | 文件/行数或数量 | 本轮状态 |
|---|---:|---|
| `plugin_sdk` 全部 / production | 21 / 5,676；18 / 4,842 | E3：declaration、runtime/editor builder、native ABI、dist macro；production fingerprint `9c5ccf5c...237d7` |
| SDK examples | 7 / 604 | E3：editor authoring与native dist；fingerprint `3b10f51a...6d770` |
| first-party dist crate | 39 / 4,248 | E3：79个test attributes；fingerprint `c1bf3303...13a64` |
| first-party runtime/editor catalog | 8 / 1,424 | E3：catalog wiring与manifest parity；fingerprint `51062a23...57344` |
| native/editor contribution fixture | 2 / 884 | E3：callback、state、panic、V2 debt与editor contribution；fingerprint `d4cb4890...497ad` |
| `cargo-zircon` plugin owners | 9 / 3,143 | E3：scaffold/sync/check/validate/artifact probe；fingerprint `4394f5bd...b21b` |
| `zircon_export/plugin_*.py` | 87 / 9,522 | E2-E3：standalone validate/build/package/hash/signing；fingerprint `b282b0d3...67c8` |
| host ABI/load admission focused set | 9 / 3,862 | E3：runtime ABI/loader、Editor/App selection；fingerprint `fa2ee015...c36` |
| repository manifest inventory | 39 plugin TOML / 140 Cargo TOML / 39 dist Cargo | E3：TOML identity、version、dependency和distribution结构统计 |

Plugin SDK/package轮次详细阅读覆盖：

- 从`declare_plugin!`追踪manifest sync、39份`plugin.toml`、source runtime/editor descriptor、first-party catalog与embedded native manifest，确认结构审计clean，但file manifest的SDK API 0.2.0与source descriptor默认0.1.0发生真实真值漂移；
- 逐读SDK native ABI与host mirror，确认`NativePluginStatic<T>`对任意`T`无条件`Sync`、owned byte carrier可复制且token可推导后`Vec::from_raw_parts`、borrowed slice可制造任意生命周期；
- 追踪entry macro和fixture host-ready callback，确认`on_host_ready` panic未经guard穿越non-unwind C ABI，可abort Editor/App；
- 从project selection追踪Editor/App discovery、compatibility、`Library::new`、descriptor/entry和registration filter，确认loader先执行全部editor candidate，之后才检查enabled/target selection；
- 逐个统计39个dist entry，确认全部`invoke_command: None`、`bridge_methods: []`、`on_host_ready: None`，只有glTF带state callback；对应第一方业务native projection均无system执行声明，dynamic artifact主要是metadata shell；
- 逐读`cargo-zircon` artifact probe和CI，确认manifest/catalog/dist build基础真实，但artifact validator只调用descriptor与查entry symbol，dist matrix仅在Ubuntu构建，不执行完整entry/behavior产品场景；
- 逐读`zircon_export plugin build/validate`、loadable hash/signing sidecar与export loader manifest，确认工具链能生成hash并调用外部signer，但runtime/editor/app loader不读取sidecar或`native_plugins.toml`；
- 复核package dependency、project selection和distribution schema，确认缺version/source/digest/interface version、project lock、per-platform artifact mapping与install/update/rollback service；
- 对照Unreal discovered/enabled/loading phase与plugin descriptor policy、Godot generated GDExtension C interface/init/reload、Bevy static plugin lifecycle及Fyrox dynamic Rust ABI风险声明；Unity Graphics仓不含权威native plugin manager，未推断闭源行为。

本轮实际运行`python tools/audit_plugin_structure.py --json`并得到clean结构结果；没有运行Cargo、真实DLL、Editor/App、signer、Windows/macOS发行、sanitizer、fuzzer或跨版本矩阵。focused contract paths成文前未出现在工作区修改列表；Hybrid GI四个在途算法文件不属于本轮合同集合。详细5 P0、30 P1、8 P2、M0-M3路线及18个验收门见`zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md`。

## 33. Editor Document / Transaction / Save / Autosave / Recovery 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| core editing production | 29 / 4,740 | E3：transaction/history/event/journal/routing；fingerprint `7a7f5663...46e8` |
| document lifecycle production | 4 / 959 | E3：project/scene lifecycle、ticket、retention；fingerprint `da8cae14...1cc7` |
| dirty/save clean production | 4 / 1,448 | E3：registry、batch、job adapter、external effect；fingerprint `d4d62e5b...4c60` |
| document toolkit production | 17 / 913 | E3：registry、save/close lease、autosave payload；fingerprint `8d5285dd...8b40` |
| recovery production | 17 / 2,901 | E3：autosave、catalog、restore flow、session guard；fingerprint `971797f0...3586` |
| product integration focused set | 18 / 5,254 | E3：tab/project/window close、foreground/autosave、UI asset/animation save、undo routing；fingerprint `417aa422...ac2a` |
| focused clean tests | 51 / 20,047 | E2：366个test attributes；未运行Cargo或产品/故障注入gate |

Editor document lifecycle轮次详细阅读覆盖：

- 从Dock/layout command追踪单tab关闭、toolkit close lease、session删除和DirtyRegistry unregister，确认dirty document没有决策就能被关闭；
- 从File菜单追踪`ProjectCloseRequested`、retained effect与`EditorManager::close_project()`，确认Close Project绕过main-window dirty prompt；
- 逐读main/floating window close prompt，确认dirty enumeration基础存在，但Save按钮明确未实现，`SaveReason::Close`没有产品caller；
- 从UI asset/animation save追踪serialize、源文件write、import/refresh/hydrate和mark clean，确认两条权威路径直接`fs::write`，且disk commit与projection failure没有复合终态；
- 逐读transaction scope/history/save token/fault recovery、DirtyRegistry与batch save，确认底层generation和原子admission基础真实，但UI asset仍有私有无界undo栈、animation无undo、全局Undo/Redo固定Global history；
- 追踪autosave admission、save mutex、capture、snapshot atomic write/rotation、completion与shutdown，确认队列有界但实际serialized payload无end-to-end byte budget，退出先取消autosave再直接close project；
- 追踪session guard live/residual、takeover、catalog与RestoreFlow，确认底层primitive只在测试可达，产品遇residual直接拒绝打开工程，restore只有plan没有executor；
- 对照Unreal batch save/checkout/autosaver/memory-bounded trans buffer、Godot per-history saved lifecycle与scene close gate、Fyrox Save/Save As/Save All和Yes/No/Cancel delayed action；Bevy与Unity Graphics参考树不提供完整Editor authority，未推断缺失源码。

本轮没有运行Cargo、Editor、真实window close、process-kill、disk fault、watcher race或跨版本journal replay。六个其他Session在途文件从clean fingerprint排除，实施前要求重读；详细5 P0、30 P1、8 P2、M0-M4路线及16个验收门见`zircon_editor/02-document-transaction-save-autosave-recovery-review.md`。

## 34. Editor Scene / Prefab / Selection / Mode / Gizmo / Picking 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| Scene Mode clean production | 16 / 915 | E3：registry/factory/stack/context、panic isolation与overlay builder；fingerprint `48eb401c...f784f4e` |
| Selection clean production | 5 / 282 | E3：Edit/Play双域、primary、多选mutation与generation；fingerprint `6015f9a2...5de0ebc` |
| Viewport clean production | 111 / 5,999 | E3：camera、handles、interaction、pointer/picking与render extract；fingerprint `de2952cd...ec49a35` |
| Scene document/product clean set | 9 / 2,144 | E3：lifecycle、route、installer、world replacement与gateway；fingerprint `24fcc12f...05257` |
| Prefab vertical clean set | 17 / 4,176 | E3：asset/project/cache、World I/O、Prefab Tools runtime/editor/dist与pane；fingerprint `00c8603e...91b1b` |
| focused combined evidence | 167 / 17,004 | 101个test attributes、0 ignored；未运行Cargo/Editor/GPU/大scene产品gate |

Editor Scene/Prefab轮次详细阅读覆盖：

- 从scene picker ticket追踪open/create、project document、installer、world replacement与lifecycle activation，确认ticket/path/rollback基础真实，但active scene identity在install时丢失，Save固定写default scene；
- 追踪Open Scene切换，确认installer会先清Global history、替换world和selection，未进入DirtyRegistry或统一Save/Discard/Cancel transition；
- 逐读scene command、Hierarchy intent与SelectionModel，确认create/delete/update/reflected field、rename/reparent和双域multi-selection可保留，但缺component topology、duplicate/copy/paste、per-document selection和下游完整多选；
- 从handle basis追踪move/rotate/scale preview与workbench transaction capture，确认子节点直接把local transform当world transform，preview先改world、释放时才补already-applied command；
- 从render packet追踪renderer-visible spatial query、precision candidate与runtime pointer dispatch，确认broad phase真实，但point picking仍是owner中心+半径代理，box selection全量扫描代理且pointer/viewport/camera identity硬编码；
- 逐读Scene Mode registry/stack/isolation和overlay provider，确认enter panic隔离、capability gate可保留，但非enter failure会丢失，context/输出缺完整authoring能力与time/item/byte budget；
- 从SceneEntityAsset/ProjectDocument/cache追踪World I/O，确认`prefab_instance`在load不消费、save固定写`None`；Prefab五个operation无factory、pane明确placeholder、runtime importer为DiagnosticOnly；
- 对照Unreal map save/mode/hit proxy、Godot per-edited-scene history与PackedScene ownership、Fyrox per-scene command/selection/mode和clipboard remap；Bevy Scene只作runtime primitive参考，Unity Graphics不含Editor Scene/Prefab权威源码。

本轮没有运行Cargo、Editor、真实GPU picking、父子变换、multiwindow、超大scene、Prefab round-trip或磁盘故障注入。interaction extract cache/test、hierarchy/inspection projection和project save相关在途文件从clean fingerprint排除；default-scene保存结论另以`HEAD`复核。详细3 P0、30 P1、8 P2、M0-M4路线及18个验收门见`zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md`。

## 35. Editor Asset Index / Import / Reimport / Catalog / Thumbnail / Reference Workflow 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| core asset orchestration clean production | 23 / 3,301 | E3：type registry、source authority、Editor index、import flow；fingerprint `608291d8...aeb91` |
| editor asset manager production | 44 / 4,168 | E3：catalog/source generation、reference graph、preview cache/scheduler、change stream；fingerprint `76a825ff...734e3` |
| product integration focused clean set | 15 / 2,900 | E3：project、scene、UI asset、animation、layout、model import、event refresh；fingerprint `0f7de5a1...51d3` |
| runtime registry/meta support focused set | 8 / 2,493 | E3：`.zmeta`、registry query、full/targeted scan/import；fingerprint `b29e679d...8fe9` |
| core index/import dedicated tests | 3 / 1,219 | E2：28个test attributes；未运行 |
| wider focused asset tests | 70 test attributes | E1-E2：type registry、index、flow与host契约；未运行Cargo/Editor/故障注入gate |

Editor Asset轮次详细阅读覆盖：

- 从Runtime `AssetRegistryIndex`、`.zmeta`、full/targeted scan追踪UUID/path/type/tag/importer/config/source digest/dependency/referencer，确认registry与sidecar基础真实，旧计划中“尚无metadata/index”的叙述已经过期；
- 逐读 `EditorAssetIndex` 与 `EditorAssetImportFlow`，确认generation key、同代合并、UUID串行、admission、panic/Drop清理可保留，但全仓production没有caller，成功测试也明确仍处于Stale；
- 从scene/project/UI asset/animation/layout/model入口追踪真实import/reimport，确认产品继续同步直调Runtime `AssetManager`，多个source已持久化路径吞掉导入错误，没有统一SavedSourceAwaitingRepair终态；
- 逐读 `DefaultEditorAssetManager` source sync/catalog generation，确认它是Asset Browser实际数据源，却与Runtime registry及未接入Editor index形成三套视图；增量路径仍复制完整source generation和两张map，并重建全部folder/details/reference projection；
- 从Ready asset projection追踪 `.zmeta` 与artifact load、handwritten `ImportedAsset` reference extraction，确认Editor重复构建Runtime已有的dependency/referencer graph，并用locator fallback掩盖部分GUID断裂；
- 逐读change mailbox、retained host drain/accumulator与refresh plan，确认512-key coalescing、256项/stream、600微秒与2毫秒drain预算真实，但昂贵catalog prepare仍同步发生在refresh caller线程；
- 从visible rows追踪preview scheduler/job/cache/currentness，确认64项admission、token/revision/Arc identity保护可保留，但provider只有SourceImage/placeholder，Operation不可执行，cache key缺provider/render版本，任务先非原子写最终PNG再校验且只更新内存preview state；
- 逐读AssetTypeRegistry、AssetImporterDescriptor与plugin materialization，确认type contribution校验基础存在，但importer descriptor无production consumer、无Runtime implementation/settings schema绑定，plugin catalog还允许批次部分成功；
- 对照Unreal Asset Registry/Import Data/Auto Reimport/Thumbnail Manager，Godot EditorFileSystem/EditorImportPlugin，Fyrox asset/preview，Bevy processor/meta和Unity Graphics具体reimport/importer consumer；不把Bevy或Unity Graphics缺失的完整Editor源码当作Zircon省略控制面的依据。

本轮没有运行Cargo、Editor、真实watcher storm、百万资产catalog、importer crash/hang、磁盘满、source control、GPU preview或跨版本reimport。四个dirty/save在途文件从clean fingerprint排除；Runtime importer ingest与project manager部分在途路径不用于格式算法结论。详细0个新增P0、30个P1、8个P2、M0-M4路线及20个验收门见`zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md`。

## 36. Editor Inspector / Reflection Property Authoring / Customization 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| Runtime reflection clean production | 46 / 5,544 | E3：schema/value/address、registry、built-in/dynamic adapters、read/write；fingerprint `ded07e2a...bdce5` |
| Editor Inspector model clean production | 24 / 6,587 | E3：extension、snapshot、binding、state、command/transaction integration；fingerprint `611355dc...f423` |
| retained product projection clean production | 94 / 19,164 | E3：builtin surface、pane payload/projection、control dispatch、workbench bridge；fingerprint `974ffd03...ff35` |
| focused Inspector tests | 13 / 2,860 | E2：46个test attributes；fingerprint `78db2465...b7fcb`；未运行 |

Editor Inspector轮次详细阅读覆盖：

- 从Selection mutation追踪`sync_selection_state`、snapshot、draft component event、Apply payload、binding batch、per-selection command capture与单Global transaction，确认事务原子性与多选集合基础可保留；
- 确认primary Translation/Scale先格式化到两位小数，任何Apply又无条件写回全部base fields，构成可复现的精度数据损失P0；
- 确认Apply会重新打包primary的Name/Parent/Translation和所有editable插件属性，state再把同一份Name/Parent/Translation/Scale/dynamic draft写入全部selected node；既有测试明确把整对象覆盖固化为成功语义，构成第二项P0；
- 从Runtime built-in reflection registration追到Editor snapshot，确认Camera、MeshRenderer、Mobility、Activation、RenderLayer、Light与RigidBody已有schema/adapter，却因产品只枚举dynamic components而不进入通用Inspector；
- 逐读ReflectFieldInfo/ReflectedValue/read-write request，确认default/range/enum/hint/documentation基础真实，但Editor只消费display/type/editable/visibility，且property address仅支持顶层字符串field name；
- 追踪FieldEditorContainer与CustomizationChain进入ContributionStore、capability filter、snapshot/payload/host data，确认field editor kind/asset markers和surface controller/data root/bindings在host前丢失，`Customization::build`没有production caller；
- 追踪三个Inspector presentation路径，确认它们在Scale、plugin component数量与控件选择上功能不等价；snapshot同步克隆dynamic JSON与reflected values、schema/field配对O(F²)、全量节点生成无预算/虚拟化；
- 对照Unreal property handle/multiple values/per-object/interactive transaction、Godot common multi-node schema与fieldwise mutation、Fyrox nested/collection/curve editor、Bevy ReflectPath/container和Unity Graphics typed SerializedProperty consumer；不把Bevy或Unity Graphics缺失的完整Editor核心当作Zircon省略控制面的依据。

本轮没有运行Cargo、真实Editor、多窗口、10k属性、多选异构component、插件卸载、undo crash recovery或reference engine benchmark。`scene_inspection_publication`、template component adapter和workbench reflection routes处于其他Session在途修改，已从clean fingerprint和稳定算法结论排除；实施前必须重读。详细2个P0、32个P1、9个P2、M0-M5路线及20个验收门见`zircon_editor/05-inspector-reflection-property-authoring-customization-review.md`。

## 37. Editor Plugin Manager / Enablement / Live Reload / Settings / Diagnostics 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| core plugin lifecycle/catalog | 35 / 6,318 | E3：manager、phase、snapshot、capability、materialization、isolation与panel source；fingerprint `dd12aa17...0153d6` |
| product Plugin Manager clean set | 59 / 6,195 | E3：status/enablement、action、live host、projection、pane data与retained conversion；fingerprint `27024dd1...2357f` |
| SDK/settings/manifest authoring | 8 / 1,757 | E3：serialized contribution、SDK builder、settings descriptor、extension merge与package metadata；fingerprint `7285c03e...a7d1f` |
| extension showcase clean set | 4 / 1,899 | E3：静态workspace、navigation/feedback/preview actions；fingerprint `66c6da5c...03d344` |
| focused plugin tests | 33 files / 145 test attributes | E2：测试源码已读；未运行Cargo、DLL、watcher或UI |

Editor Plugin UX轮次详细阅读覆盖：

- 从`EditorPluginManager`状态机、loading phase、generation snapshot和`EditorPluginPanelSource`追到真实pane，确认core有可保留的typed lifecycle authority，但产品面板完全改读另一份manifest/native字符串status report；
- 从Enable/Disable click追踪manifest load、同步native discovery、capability/lifecycle mutation、status publish和最后的atomic file save，确认save失败没有补偿或durable repair state，构成session/disk分裂P0；
- 从Unload/Hot Reload追踪真实`NativePluginHostHandle`与debug watcher，纠正旧计划“backend完全未连接”的过时判断，同时确认live outcome不发布新status Arc/generation，pane继续命中旧缓存形成false-green P0；
- 逐读feature action/projection/parser与dependency enablement，确认一行只可操作一个feature、qualified dotted package ID会误拆、provider缺失/歧义/cycle可留下部分selection，packaging/target又靠无约束轮转；
- 对照Runtime package manifest，确认version/description/category/maturity/platform/distribution/dependency/options等已有metadata未进入主产品详情，`PluginOptionManifest`没有Editor consumer；
- 从serialized contribution/SDK builder追到materializer和catalog generation merge，确认settings page在单次registry可注册，却在`build_editor_extensions`漏拷且全仓无产品consumer；
- 识别第二套静态Plugin Manager workspace：它硬编码插件、版本、依赖、更新/警告数量与queued反馈；clean证据只用于证明fixture投影，dirty template binding留待实施前重读；
- 对照Unreal search/category/pending restart/dependent disable/refresh、Godot effective-state rollback/recovery/create-edit、Fyrox reload owned-state cleanup；Bevy仅作lifecycle composition参考，Unity Graphics checkout仅作package metadata consumer参考。

本轮没有运行Cargo、真实Editor、native DLL装卸、artifact watcher、磁盘故障、manifest并发编辑、1000插件交互UI或跨版本恢复。详细2个P0、32个P1、8个P2、M0-M5路线及20个验收门见`zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md`。

## 38. Editor Play Session / Process Runtime / PIE / Game View / Live Edit / Recovery 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| Play core/process clean set | 33 / 4,419 | E3：controller、backend、snapshot、output、plugin activation、domain link、edit protection与process tree；fingerprint `a416ba78...da8428b` |
| Editor产品装配与状态 clean set | 18 / 4,736 | E3：command、menu enter/exit、project clear/replace、host tick/close、Game pane与startup wiring；fingerprint `cfab49de...e4f4bda3` |
| Runtime启动/报告 clean set | 5 / 2,704 | E3：Editor/Runtime entry、CLI、preview binary与Cargo target；fingerprint `abe4f8e0...f3bd47e4` |
| focused clean tests | 8 / 1,705 / 48 test attributes | E2：policy、queue、snapshot、process command、state、event stack与toolbar测试源码已读；fingerprint `3f56aa66...aa4aaff0` |

Editor Play轮次详细阅读覆盖：

- 从Enter Play追踪whole-world snapshot、`EditorState` checkpoint、native plugin activation、snapshot store、suspended process spawn与controller mode publish，确认产品真实安装Process backend，但启动同步占用UI且spawn成功就过早投影Playing；
- 从Runtime CLI追踪starting/start-failed/ready/terminal报告，确认`--play-report-pipe`只是stdout logical outlet，Editor没有parser，Ready又发生在event loop/首帧之前；
- 逐读process tree lease、bounded output pump、child stop/finish与backend poll，确认正常路径的树终止/日志预算基础可保留，同时定位`active.take()`后失败丢owner、terminal plugin恢复失败后空backend继续报Running的P0；
- 从Close Project command、retained side effect、`clear_project/replace_world`追到自动`exit_play_mode`，确认项目A checkpoint可在打开B后覆盖B authoring world，构成确定的跨项目数据破坏P0；
- 从产品startup确认唯一backend是外部`zircon_runtime`，启动时attach的gateway属于Editor persistent runtime session，不是spawn child；process backend永久不可attach，runtime consumers与WorldDomain Play抽象没有进入真实子进程；
- 从Game descriptor、pane projection、viewport image/capture caller追踪，确认Game虽是中心document却明确`is_viewport=false`，没有runtime画面、input/focus/resize或独立presentation；
- 从`PlayKind`、Building、`running_document`、`route_edit`和EditorState mutation gate追踪，确认Play/Simulate无行为差异、Building无产品入口、pending edit/router无production caller，真实行为只是硬禁scene编辑；
- 对照Unreal独立PIE world context、PIE/SIE切换、viewport与多client/server teardown，Godot多PID/remote debugger/embedded focus-size-timeout，Fyrox build/play profile与child loop，Bevy typed remote query/watch/mutation；Unity Graphics checkout不含Editor Play authority，未推断闭源实现。

本轮没有运行Cargo、真实Editor/runtime child、DLL、Game View、remote debug、跨平台termination fault、1 GiB scene或多实例benchmark。dirty的`core/play/tests.rs`、pending-decision tests和Runtime session ABI文件未作稳定证明；详细2个P0、32个P1、8个P2、M0-M6路线及20个验收门见`zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md`。

## 39. Hub Project / Engine / Build / Editor Launch / Persistence / Delivery 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| Project/config/shared registry clean set | 20 / 5,224 | E3：create/import/remove/delete、metadata、recent reconcile、config load/save与startup；fingerprint `43f58a7e...e56a8c4` |
| Engine/build/process clean set | 25 / 3,997 | E3：source registry/validation、build runner、job queue、Editor launch/handshake/focus与Tauri composition；fingerprint `385d3044...0b2e844` |
| Delivery clean set | 5 / 1,855 | E3：package、local install、receipt/download manifest与产品action；fingerprint `6cc3da48...1074836` |
| Cross-host Hub protocol clean set | 26 / 1,593 | E3：mailbox/session/focus/recent DTO与Editor publish/consume；fingerprint `25438ae4...16b9e3c6` |
| focused Hub contract tests | 7 / 3,228 / 42 test attributes | E2源码审查；多数为source-text contract，未运行；fingerprint `822cfb9d...c1907eb` |

Hub后端轮次详细阅读覆盖：

- 从Tauri composition追踪config load、recent manifest refresh、shared registry三方合并、source engine注册、catalog刷新和startup persist，确认窗口创建前存在全有或全无启动链、无限writer lease与离线recent tombstone复活；
- 从Create Project追踪linked-path拒绝、staging/backup/rename commit与kept-folder compensation，确认底座可保留，同时定位`persist_unchecked(None)`与零参数定义不匹配的静态编译P0；
- 从Delete Project追踪confirmation、Recycle Bin、Hub registry mutation和persist，确认没有探测active Editor session，且文件回收先于durable terminal commit，形成数据损失与错误重试P0；
- 从Open Editor追踪project selection、engine activation、session lease probe、focus signal、spawn、10秒mailbox和completion persist，确认Editor完整启动门真实存在，但Hub立即丢弃Child、无focus ack/heartbeat/terminal owner，且sibling Editor可绕过project engine binding；
- 从global background FIFO追踪Build/Package/Install/Open Editor，确认panic containment与FIFO基础真实，但queue无界、单lane、detached、不可取消，Build又用`Command::output()`无界缓冲完整日志；
- 从source engine registry/validation追踪path-FNV identity、workspace字符串检查、build exit-code success与8条字符串history，确认它不是versioned engine installer、resolver或verified build-set publisher；
- 从Package与Device Install追踪project tree copy、receipt hash和`file://`download manifest，确认当前交付只是本地源码复制，不含snapshot/cook/dependency closure/runtime binary/signing/真实device transport/atomic activation/rollback；
- 对照Unreal project-to-engine/InstalledPlatform/open compatibility，Godot threaded scan/missing-recovery/version-feature preflight，Fyrox Child ownership/try_wait/target-aware export；Bevy与Unity Graphics本地参考树不含可比Hub owner，未据此推断控制面可省略。

本轮managed Windows `cargo build -p zircon_hub --locked`实际执行并在`project_actions.rs:583`复现`E0061`，exit 101；测试因编译阻断未运行。没有运行Hub、Editor、回收站/磁盘/进程故障、跨进程竞争、设备、签名或规模benchmark。详细4个P0、36个P1、8个P2、M0-M6路线及20个验收门见`zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md`。

## 40. Hub Web Shell / Catalog / Settings / Team / Cloud / Accessibility / Performance 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| React web source clean set | 66 / 7,447 | E3：App状态流、全部页面与shared component、typed action、validator、theme/localization consumer；0个前端test文件；fingerprint `3e3b4baa...fc53c19` |
| Asset/Plugin/Learn/Team Rust clean set | 11 / 2,097 | E3：递归发现、scope刷新、Git投影、catalog DTO与coming-soon；26个test attributes；fingerprint `dee4b32a...42bd9ae` |
| Settings Rust clean set | 6 / 2,528 | E3：draft、browse/save/default、health与source checkout验证；27个test attributes；fingerprint `527d56a8...215f9f1` |
| View/action contract Rust clean set | 15 / 5,964 | E3：request decode、full snapshot DTO、localized projection与Tauri emit；59个test attributes；fingerprint `b32b9b0e...f5ae0ee` |
| Packaging/security contract clean set | 8 / 2,387 | E2-E3：Cargo/npm/TS/Vite/Tauri config、capabilities与lockfile；fingerprint `1ded19ed...47d24f` |

Hub UI/service-shell轮次详细阅读覆盖：

- 从React首帧追踪`loadHubState`、runtime validator、subscription、action sequencing、ErrorBoundary和Snackbar，确认typed Tauri action基础可保留，同时定位production任意后端/协议失败都会退化成写死Ready demo、浅层decoder、无revision full-snapshot广播和同tone通知不再重开的P1；
- 从source/project/settings变化追踪四类scoped view refresh，确认Assets/Learn/Plugins/Team在全局session锁内同步串行扫描，一个坏文件/manifest可阻止整次refresh和startup，限制又发生在无界遍历/全文读取之后；
- 逐读CatalogPage/HubList/HubTreeView，确认全部数组在client重建list/tree且无分页/虚拟化；同一Catalog component跨Assets/Plugins/Learn保留非法tab/query，tree default expansion也不会随route更新；
- 对照Editor Asset/Plugin报告确认Hub raw extension scanner和manifest scanner构成重复authority；普通project又固定获得Elysium封面、猜测platform和伪`1.8.2`；
- 从Settings输入追踪完整draft invoke、Rust patch、health、source validation、refresh和persist，确认没有draft revision/CAS，warn目录被计入100% Ready，save又在refresh/persist前改变内存config/engine；
- 从Team/Cloud/topbar追踪Git identity/author、package history和coming-soon，确认contributors被误称members、email自动投影、failed package history也可算ready，Update/Marketplace/Auth/Cloud/RBAC仍没有provider；
- 逐读custom window、responsive nav、tree/list/table/popover和Tauri config，确认undecorated窗口没有drag region、折叠nav丢accessible name、自定义tree/table缺keyboard contract且CSP为`null`；
- 对照Unreal Project Browser/Asset Registry/Slate accessibility、Godot cancellable scan与AccessibilityServer listbox语义、Fyrox async size/Child owner；Bevy与Unity Graphics不含可比商业Hub owner，未据此推断服务可省略。

本轮`npm run typecheck`实际exit 0。真实Tauri截图矩阵因当前binary不存在、Rust build仍被Hub 01的`E0061` P0阻断而未运行；没有使用Vite fallback截图冒充产品证据。详细0个新增P0、46个P1、8个P2、M0-M5路线及22个验收门见`zircon_hub/02-web-shell-catalog-settings-team-cloud-accessibility-performance-review.md`。

## 41. Tooling Workspace / Toolchain / CI / Validation / Developer Entrypoint 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| Workspace/toolchain/dependency | 5个核心Cargo/deny文件，root实际36包、plugin 139包 | E3：locked metadata、member ownership、package metadata和duplicate tree；fingerprint `5129c03c...27bb2e` |
| GitHub Actions | 3 workflows / 731行 | E3：触发器、toolchain、matrix、命令、cache、artifact与failure policy |
| Windows validator | 2文件 / 3,488行 / 104个Pester `It` | E3静态、E2行为：未运行完整Pester/Cargo矩阵 |
| Convention/feature/profile/domain guards | 10个核心脚本与聚焦测试 | E3：入口、解析、命令计划、失败传播与产品调用点 |
| Developer fast-build入口 | 8个脚本/包装/文档/测试 | E3：profile/feature映射、environment lease、CMD转发与文档命令 |
| tracked tooling inventory | 约338,551行、349个Python test模块、35个PowerShell Tests文件 | E1 inventory：其余工具不能据此视为已审 |

Tooling首轮详细阅读覆盖：

- 通过Cargo locked metadata确认root显式10个member、实际36个member，其中26个plugin package又归独立139包plugin workspace，形成双workspace/lock/profile上下文；
- 实际复现`zircon_plugins/Cargo.lock`失配，plugin workspace所有`--locked` CI门在编译前失败，记录为本轮P0；
- 读取全部三个workflow并追踪normal build/test、profile/domain、dependency、export policy和MVP F5链，确认正常完整workspace只在Linux，八平台export matrix没有安装SDK或构建任何target；
- 逐段读取1,311行Windows validator与2,177行Pester合同，确认managed physical target和hash evidence基础可保留，同时定位盘符硬编码、非DAG执行、破坏性cache clean、lane-local `CARGO_HOME`和artifact provenance缺失；
- inventory确认349个Python测试模块与35个PowerShell Tests文件大部分不进CI；聚焦运行62项合同测试，61通过、1项因`diagnostic_log_args`实现形状漂移失败；
- 执行domain audit得到19,557行、2,741条production reference和72条edge，确认其硬编码domain已漏`operation/runtime_diagnostics`、仍含不存在的`rhi`，且无allowed-direction/baseline/failing exit；
- 实际调用client快捷CMD并复现非法`client` profile；追踪interactive工具还请求多项不存在的plugin feature，README路径与profile词汇同步漂移；
- 对照Bevy workspace lint/MSRV/三平台/Miri/docs/action hardening，Fyrox真实template export，Godot typed SCons和多平台/sanitizer矩阵，Unreal BuildGraph/LowLevelTests/PGO/DDC以及Unity Graphics package validation/promotion链。

本轮没有修改manifest、lockfile、workflow、验证器或生产代码；没有运行完整workspace Cargo，因为plugin lock P0和既有Hub编译P0已确定阻断。详细1个P0、40个P1、8个P2、M0-M5路线及22个验收门见`zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md`。

## 42. Cargo Zircon Plugin Scaffold / Manifest / Native Probe 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| `cargo-zircon` production | 12文件 / 3,173行 | E3：CLI、check、diagnostic、sync/parser、scaffold/templates、static/native validate逐文件读取；fingerprint `c36994ba...a821` |
| `cargo-zircon` tests | 2 integration files / 1,155行，加2个inline test / 21 tests | E3源码；实际Cargo在production compile P0前失败，0 test executed |
| First-party catalog/App wiring | 7个核心manifest/source | E3：marker、registration、manifest inventory与feature链 |
| Python validator对照面 | 139 production/test files / 21,764行 | E1-E2：owner inventory加真实`--all`动态验证；算法留给export切片 |
| Plugin inventory | 39 root manifests / 42 declaration-containing files / 139 workspace members | E2：Python另发现2个feature extension，当前publish target共41 |

Cargo Zircon轮次详细阅读覆盖：

- 逐文件追踪手写CLI parser、workspace check、TOML validation、Rust declaration parser/sync、三类scaffold模板和in-process native artifact probe；
- 实际运行`cargo test -p cargo-zircon --locked`与真实`cargo run ... plugin check`，均在`check.rs:70`复现`Option<&Path>`传给`&Path`的`E0308`，记录当前compile P0；
- 对照生产runtime catalog确认静态manifest表已迁到`src/tests/generated_manifest.rs`，而generator/check及fixture仍要求`src/lib.rs` marker/include，编译修复后`plugin new/check`仍确定失败；
- 追踪scaffold的package文件和六个共享文件写入，确认没有writer lease、CAS、journal或atomic multi-file commit，失败rollback错误被全部忽略；
- 读取模板确认system/editor注册为空，importer只生成泛化Data descriptor，native behavior固定systems/events/bridge空且无unload/host-ready，不能把结构生成当行为完成；
- 追踪declaration parser确认字段固定顺序，`module_description`读取后丢弃，native projection除entry外全部吞token，Rust-to-manifest authority并不完整；
- 追踪native probe确认候选DLL在CLI进程内加载/执行，ABI projection无struct size和字符串长度，`CStr::from_ptr`无界读取，记录security/robustness P0；
- 实际运行Python发布validator，33.4秒exit 0，`target_count=41`、39 root package加2 feature extension，证明Rust/Python inventory与schema authority分裂；
- 对照Unreal版本化PluginDescriptor，Godot产品内plugin创建校验，Fyrox/Bevy真实lifecycle，以及Unity Graphics package validation/promotion链。

本轮没有修改Rust、Python、catalog、manifest、lockfile或CI。详细3个P0、36个P1、7个P2、M0-M5路线及22个验收门见`zircon_tooling/02-cargo-zircon-plugin-scaffold-manifest-validation-native-probe-review.md`。

## 43. Export Preset / Build / Cook / Pack / Platform Bundle / Release 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| `tools/zircon_export` production | 246 Python文件 / 42,172行 | E3：八阶段、preset、handoff、template、native/sign与104个report schema owner；scoped clean fingerprint `dcc07a38...3fbb1` |
| Python export tests | 201文件 / 71,091行 / 1,568个源码test methods | E2-E3：完整运行展开为1,642项，373.192秒，667 failures |
| Rust export build plan | 40文件 / 5,930行 | E3：profile/strategy、generated project、compile plan与desktop/mobile/browser host |
| zrpack/export bins | writer/reader/delta与pack/validate bins | E3：格式、内存模型、写入、delta、报告与校验链 |
| Editor export | core stages、wizard plan/execution/session defaults | E3：产品入口、默认输入、进程执行、report与bundle layout |
| shipped templates | Windows/Linux/macOS共3包 | E3：全部为placeholder host；真实PlatformBundle伪pack复现通过 |

Export轮次详细阅读覆盖：

- 逐字段追踪`.zpreset`解析与production consumer，确认target mode、entry scenes、keep/exclude、plugin subset、cook compression/binary assets和customized files赋值后无人消费；
- 从CLI Validate计划追到CompileHost和最终Report，合成复现production report得到4条必然schema诊断，确认两套CompileHost协议互斥；
- 从Editor retained action追到wizard options/core stage/layout，确认默认source manifest位于无人生成的新output路径，client又固定构建Hub/Editor/Runtime并以Hub为launcher；
- 逐读generated desktop/mobile/browser host，确认runtime owner在bootstrap函数返回时drop，lifecycle/input/viewport/resource callback为true stub，移动端与Web仍要求人工构建/复制Rust产物；
- 对shipped Windows template执行真实PlatformBundle，确认任意17字节伪pack与声明为placeholder的文本host可得到exit 0、fatal false和空diagnostics；
- 从Cook fallback的`res://`文本正则追到manifest绝对source路径，再追到zrpack整资产读取、一资产一chunk、全量内存writer/reader和整资产delta；
- 完整运行export Python suite，记录1,642项中667项失败，定位NativeDynamic共享Validate fixture缺`schema_version=2`和SourceTemplate诊断漂移等基线破坏；
- 对照Unreal BuildCookRun/CookOnTheFly/IoStore责任链、Godot template架构/PCK/签名提交、Bevy dependency-hashed AssetProcessor、Fyrox target-aware Android/WASM export；Unity Graphics仅用于package CI/promotion参考。

本轮没有修改生产Python/Rust/Editor、template、preset、pack格式或CI；没有执行真实跨平台SDK构建、签名、公证、安装、首帧、server health、10GB cook或patch中断。详细8个P0、46个P1、8个P2、M0-M7路线及24个验收门见`zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md`。

## 44. Reflection Derive / Script Host Macro / Schema Codegen 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| `zircon_reflect_derive` | 5个Rust文件 / 870行，加Cargo manifest | E3：container/field attribute、type/field expansion、name/slot读写与7个测试逐文件读取 |
| `zircon_runtime/reflection_macros` | 8个Rust文件 / 848行，加Cargo manifest | E3：script type、host function/module、inventory scanner与11个测试逐文件读取 |
| interface/runtime/consumer clean set | 合计89 tracked文件 / 11,472行 | E2-E3：DTO、registry、derived adapter、dynamic scene、内建component、script descriptor/migration/host module；fingerprint `b10d341b...a6205a3` |
| reference engines | Bevy reflect、Fyrox reflect、Unreal UHT、Godot ClassDB、Unity Graphics ShaderGenerator | E2-E3：typed IR、identity/hash、compatibility、diagnostic/codegen artifact责任对照 |

Reflection/codegen轮次详细阅读覆盖：

- 确认场景`ZrReflect`、脚本`ZirconScriptType/host function/module`与生产手工math descriptor形成三条authority，没有共享parser、schema IR、identity或validator；
- 从`infer_value_type_path`追到`DeclaredValueType::parse`，确认`Vec<T>`被生成成裸`List`，而consumer只接受`List<T>`；自由`value_type_path/value_kind`又可与真实converter矛盾；
- 逐读普通/VM registration验证，确认VM额外检查plugin identity、prefix和全部declared type，普通入口只检查字段名/type path非空及default，入口语义不一致；
- 确认两套derive都把enum生成成空field/空variant；生产`Mobility`手写虚拟`kind`读写补洞，但仍没有enum options、stable variant ID或unknown/deprecated策略；
- 从type path、dense slot、scene name-to-slot映射和VM state migration追踪身份边界，确认slot是generation内热路径ABI而非现有scene文档直接持久slot，同时定位type/field rename没有统一schema version/hash/alias/migration；
- 逐读host function调用adapter，确认只支持同步精确arity自由函数，不能自然返回typed `Result`，动态`ScriptHostValue`descriptor固定Null，float返回也不做finite验证；
- 对照Bevy renamed dependency/generic/enum/compile-fail、Fyrox UUID/property metadata、UHT definition hash/reload、Godot API/compatibility hash和Unity稳定失败式codegen，收敛为canonical `ReflectionSchemaIr`路线；
- 两个proc-macro包测试分别7/7和11/11通过；managed runtime consumer测试在执行前被当前326个无关compile errors阻断，未把该基线归因于reflection宏。

本轮没有修改Rust、Cargo、reflection schema、scene格式或script ABI。详细4个P0、42个P1、8个P2、M0-M6路线及24个验收门见`zircon_tooling/04-reflection-derive-script-host-macros-schema-codegen-review.md`。

## 45. WOC Content Codegen / Cargo Build Scripts / Generated Artifact / Incremental 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| WOC全tracked set | 1,967文件 / 73,610,362 bytes | E1 inventory；index-record fingerprint `2f18a4ce...fd3600` |
| WOC tools | 390 tracked文件，其中386个直接`.mjs` | E2全量分类：127 codegen、44 source extract、18 contract test、109 static guard、78 state/source check；类别可重叠 |
| WOC codegen | 127文件 / 21,799行 / 1,154,916 bytes | E2-E3：最大generator、source extraction、multi-output、writer/provenance与package入口纵向追踪 |
| generated ZrVM | 106文件 / 84,087行 / 3,367,241 bytes | E2-E3：56,242个`if`、解释执行配置、最大产物、生成器和产品import追踪 |
| native `woc_contract_codegen` | 8个Rust source/test文件，加manifest | E3：typed manifest、validation、fingerprint、Rust/ZrVM projection、CLI和测试逐文件读取 |
| Cargo crate-root build scripts | 7文件 / 1,019 physical lines | E3：target env、plugin/profile schema、native C++ source/link、OUT_DIR生成与rerun边界；fingerprint `d0ffc5b2...b827a` |

内容codegen/build.rs轮次详细阅读覆盖：

- 确认根`.gitignore`的`examples/*`隐藏WOC所有后续新增文件；tracked `package.json`当前引用两个只存在于本机ignored set的脚本，clean clone必然缺入口依赖；
- 按Git tracked set重算386个`.mjs`，只有188个被任一package script引用，198个无入口；默认generate/check分别只覆盖11/28个，`.github`没有WOC lane；
- 实际执行`npm run check`，80.945秒exit 1，在第7步复现typed contract实际157、测试硬编码148的漂移，后21步被`&&`短路；
- 量化130个`writeFileSync` writer、0个`renameSync`、64个multi-emit generator，追踪reference 8文档、JS/Rust双语言projection均无transaction/generation/journal；
- 量化291个`execFileSync`与仅20个TypeScript AST consumer，确认上游source parity依赖重复Git调用、regex/slice与多套expected count，没有统一typed IR和dependency graph；
- 识别106个generated ZrVM文件、84,087行和56,242个`if`，追踪ability/content generator把数据查找与rank/effect选择生成进解释执行源码，暴露紧凑只读table/string view能力缺口；
- 逐读`woc_contract_codegen`，保留typed schema、reserved ID、finite/length validation与manifest fingerprint优点，同时定位CLI孤立、双projection顺序直写与tool identity缺失；
- 区分7个真实crate-root build script和普通同名模块，确认App混用host/target cfg、Editor raw TOML复制plugin authority、Runtime profile复制清单/猜OUT_DIR布局、Navigation无序枚举与Physics未知target fallback；
- 对照Bevy dependency full hash/write-ahead transaction log、Unreal UHT typed export/body hash、Godot集中generated wrapper、Unity typed whole-file shader generation；未把Fyrox/Unity未提供的原子content pipeline能力写成参考事实。

本轮没有修改WOC、Rust、Cargo、生成物、ignore规则或CI。详细6个P0、48个P1、10个P2、M0-M6路线及22个验收门见`zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md`。

## 46. Session Coordinator / Control Plane / Lease / Validation / Artifact / Finalize / Supervision 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| Coordinator tracked set | 306文件 / 106,498 physical lines / 5,429,337 bytes | E2 inventory；所有owner chain E3；combined scope fingerprint `c68a32ca...a759` |
| 生产Python | 106文件 / 55,498行 / 2,267,339 bytes | E3：server、DB/migration、Session/WIP/lease、Cargo/validation、Git/artifact、workflow、control、sync、supervision逐文件读取 |
| Python tests | 102文件 / 43,943行 / 1,175 test methods | E2-E3；full discovery 15分钟超时，分组/聚焦验证发现当前snapshot合同失败与并行隔离问题 |
| Control Web | 60 source/script文件 / 3,998行 / 238,338 bytes | E3 contract/decoder/build/currentness；70 tests通过但checked-in dist与重建不一致 |
| Session Tray与安装入口 | 25 tracked文件 / 13,456 physical lines / 558,595 bytes | E3 Rust client/recovery/lifecycle、PowerShell Query/install边界；36 tests通过 |

Session Coordinator轮次详细阅读覆盖：

- 追踪legacy `/command`、control HTTP、router、runtime descriptor、Python client、Web和Tray，确认token固定为空、所有request被标为runtime authorized并映射maintainer，cookie/CSRF/elevation/role体系被入口整体旁路；
- 从validation submit/start/record_result追到Popen与ticket状态机，确认任意argv可执行、caller可写passed，terminal state没有worker attempt/process receipt；
- 从integration candidate seal/finalize追到compile ticket，只检查同Session passed而不绑定candidate blob/input tree/action；从milestone finalize确认validation在live worktree运行而commit使用shared index tree；
- 追踪shared index snapshot/overwrite/restore、cleanup reset和lock recovery，确认SQLite mutex不约束外部Git，长验证期间外部stage可能被旧snapshot抹掉；
- 追踪ordinary/benchmark两条process path，确认只有benchmark使用Job Object/creation identity，普通取消只terminate root，daemon重启后活PID会永久running且Popen/collector已丢失；
- 保留lease canonicalization/ancestor conflict、artifact filesystem identity reservation、bounded output tail、candidate temporary index、Tray recovery circuit等正向基线，并为它们定义收敛owner；
- 动态运行full Python discovery至904.040秒超时；聚焦snapshot 19项仍1 error，deferred 12项串行通过；Web check 70项和Tray 36项通过，但Web check覆盖dist后验证，真实产生22个新hashed文件；
- 确认`.github`无Coordinator Python/Web/PowerShell/Tray consumer，Hook Query又因installer硬编码schema 28对当前schema 65返回daemon incompatible；
- 对照Unreal UBT ActionGraph/history/hash cache、AutomationTool BuildGraph/TempStorage、Horde agent/lease/artifact/ACL，以及Bevy/Godot CI矩阵；未虚构Fyrox/Unity Graphics不存在的control-plane能力。

本轮没有修改Coordinator、Web、Tray、PowerShell、CI、数据库或Git index。详细8个P0、48个P1、10个P2、M0-M6路线和验收门见`zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`。

## 47. Performance Benchmark / Profile Capture / Symbol / Crash / Evidence Baseline 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| Cargo benchmark surface | 161个manifest；0个conventional bench target/framework | E2全量manifest扫描；代表ignored harness E3 |
| UI/PBR/RenderExtract capture | UI主脚本2,644行；PBR 883+935行；RenderExtract主脚本922行 | E3：path、source/build、process、collector、evidence gate、publication逐链读取 |
| Crash/log/symbol | diagnostic log 25个source/test文件、2个binary hook；全仓native crash/symbol owner搜索 | E3 panic/flush与build/export sidecar；确认无minidump/Crash Reporter/symbol service |
| performance plan | 562文件 / 40,057行 / 4,096,858 bytes | E2 inventory；主审计、pending/review ledger E3 |
| tracked test evidence | 1,213文件 / 862,902,570 bytes | E2 extension/size/catalog/LFS/retention；代表capture E3 |

Performance/tool evidence轮次详细阅读覆盖：

- 扫描161个Cargo manifest，确认没有`[[bench]]`、Criterion、Divan或Bencher；定位139个文件的190个ignored test，其中118处为performance/benchmark/manual/visual/GPU/profile证据；
- 逐读native plugin benchmark harness，保留source manifest、release profile、barrier和structured schema，确认其仍缺cross-process统计、趋势consumer与baseline gate；
- 逐读UI profile path/session/project/screenshot链，以read-only路径解析证明raw scenario可用`..`逃逸默认profile和tracked evidence根；确认3次measured run无跨运行聚合且evidence gate默认关闭；
- 交叉读取PBR provenance writer/capture/summarizer，确认前两者只接受schema 2 managed、后者只接受schema 1 local；21项summarizer测试通过但固化旧schema，属于协议false green；
- 逐读RenderExtract frozen input、dirty-byte fingerprint、exclusive lease、Job Object、timeout、no-overwrite evidence和hash report，保留为统一capture pipeline正向基线；
- 追踪panic hook到Editor/runtime preview，确认只flush旧日志并把panic_info交给默认stderr；全仓未找到native minidump、SEH/signal owner、thread stack、Crash Reporter、symbol store或自动symbolication；
- 量化`docs/plans/performance` 562文件与634个连续finding，确认accepted table为空且Rust ledger比current union分别落后155/248；
- 量化`docs/tests`约823MiB，其中37个RDC约632MiB、651个PNG约119MiB；确认无LFS/CAS、root catalog、retention/currentness与promotion合同；
- 对照Bevy Criterion bench crate、Unreal crash/minidump/reporter/symbol链、Godot Windows stack handler和Unity Graphics固定warmup/marker/performance test；未虚构Fyrox不存在的集中owner。

本轮没有修改或运行生产capture、GPU/WPR/Tracy/RenderDoc、crash handler、symbol upload、performance ledger或历史证据。详细4个P0、56个P1、10个P2、M0-M6路线及24个验收门见`zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md`。

## 48. Shared DDC / Build Cache / Remote Execution / Artifact Reuse 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| Asset artifact/CAS/currentness | artifact目录17文件，加scan/import恢复主链 | E3：action输入、manifest/chunk、publication、恢复、residency与disk GC |
| Shader disk/prewarm | 5个核心Rust owner、2个Python工具与合同测试 | E3：disk key、source identity、pair publication、report与fallback |
| UI/IBL persistent cache | UI store 374行、IBL store 189行 | E3：fingerprint、integrity、transaction、eviction与设备身份 |
| Cargo/CI cache | 3 workflows、fast-build、feature checker、Python/Coordinator environment | E3：cache activation、root、tool pin、reuse、metrics与CI consumer |
| Remote execution | 非参考源码owner搜索、Coordinator action/process对照 | E2 absence proof；E3 action/worker/CAS/sandbox/security目标边界 |
| selected combined scope | 45条Git index record / 14,450行 / 532,297 bytes | fingerprint `a4b2ce91...fb6459` |

Shared DDC轮次详细阅读覆盖：

- 保留asset schema 4 manifest、BLAKE3 64KiB chunk、先chunk后manifest、有界decode、内存residency，以及ProjectManager按source digest/importer/config/current artifact恢复的正向基础；确认当前本地恢复并非无条件陈旧命中；
- 确认`LibraryCacheKey`只有测试caller且使用64位`DefaultHasher`，asset manifest也不绑定source/importer/config/dependency/tool action；现有project locator与chunk CAS尚不能直接作为跨机器DDC；
- 逐行追踪shader disk key和prewarm worker，确认hash不含WGSL/template/Naga/WGPU，worker却按该hash去重并为第二个不同source记录written coverage，形成P0；
- 追踪shader私有fixed-temp writer与双文件发布，确认并发writer可竞争同一temp，rename失败时只要target存在就返回成功且不验证字节，mixed generation可被当作成功，形成P0；
- 读取UI persistent store与IBL runtime cache，确认前者直接`fs::write`并独立发布artifact/payload，后者单文件原子但rejected entry不隔离；两者均无全局disk quota/GC或remote policy；
- 读取3个workflow，确认共7处使用`Swatinem/rust-cache@v2`，不再把CI误报为完全冷构建；同时确认只有`dev-fast-build.ps1`显式设置`RUSTC_WRAPPER=sccache`，其余managed环境仅创建`SCCACHE_DIR`且会受ambient wrapper影响；
- 聚焦运行shader prewarm与Cargo environment共35项Python合同，全部通过；这些测试未覆盖shader key碰撞、并发pair publication或跨backend currentness；
- 非参考源码未发现REAPI/UBA/FASTBuild/Incredibuild或等价remote executor；定义先local Action Cache、再team DDC、最后authenticated hermetic remote worker的依赖顺序；
- 对照Unreal DDC key/policy/build definition/build worker、Bevy AssetProcessor、Godot importer/currentness与Fyrox ResourceManager；未把Fyrox runtime cache或Unity Graphics package源码虚构成共享DDC。

本轮没有修改生产cache、Cargo environment、workflow、Coordinator或remote service，也没有运行GPU、并发writer、网络backend或完整Cargo workspace。详细2个P0、52个P1、10个P2、M0-M6路线及24个验收门见`zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md`。

## 49. Release Channel / Artifact Repository / Install / Update / Rollback 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| CI与版本身份 | 3个workflow、0个Git tag、根/plugin/Hub/Tray版本声明 | E3 trigger/artifact/version authority；absence proof |
| Hub Engine/Build/Install | source engine、validation、build completion、device install/receipt、Tauri bundle | E3从source locator到staged/delivery状态纵向读取 |
| 签名与下载 | NativeDynamic signer/`.sig`、runtime net/content download | E3 producer/consumer、trust、TLS、resume与integrity |
| 事务与迁移seed | 3个local installer、MVP release probe、asset migration transaction | E3复用边界；不误判为产品updater |
| selected combined scope | 47条Git index record / 13,765行 / 539,986 bytes | fingerprint `7c4bb6f2...07cf` |

Release/provider轮次详细阅读覆盖：

- 确认仓库0个Git tag，3个workflow均无tag/release/publish trigger，唯一`upload-artifact`只保存7天MVP evidence；根/plugin/Hub/Tray的`0.1.0`没有统一authority或版本推进合同；
- 逐读Hub `SourceEngineInstall`、path ID、registry、validation与build completion，确认64位FNV路径身份、三个文件形状检查和进程exit 0会被投影成可变output中的staged payload；
- 确认Tauri只配置NSIS bundle，无updater plugin、endpoint、公钥或updater artifact，Hub文案明确remote update service在local v1未启用；
- 逐读device install/receipt，保留失败清理和逐文件SHA-256，确认它只向新目录递归复制、以本机path生成resource/download identity和`file://` URLs，不支持upgrade/slot/switch/health/rollback；
- 逐读NativeDynamic sign/notarize与plugin signature sidecar，确认外部命令audit完整但`.sig`只是未签名hash TOML，且runtime/editor/Hub没有consumer；
- 逐读content download manifest、Range/length/hash验证、partial bitmap/state与security policy，确认resume只在内存且HTTP固定使用允许非TLS的development policy；
- 保留asset migration的dry-run/durable journal/recovery和Coordinator installer的cutover/rollback作为未来InstallService种子，不复制其domain实现；
- 聚焦运行26项NativeDynamic signing测试全部通过，MVP staged project release probe合同通过；这些只证明局部执行/hash/handle-release合同，不证明发行信任或更新事务；
- 对照Unreal BuildPatchServices manifest/installer/verify-repair、Godot update discovery与versioned export templates、Bevy release content/version bump、Fyrox project dependency upgrade及Unity Graphics pack-validation-promotion链；没有虚构Godot/Fyrox/Unity Graphics不存在的完整二进制updater。

本轮没有修改production Hub、export、network、installer、workflow、version或安装目录。详细3个P0、54个P1、10个P2、M0-M6路线与24个验收门见`zircon_tooling/09-release-channel-artifact-repository-install-update-rollback-operations-review.md`。

## 50. Test Architecture / Partition / Selection / Isolation / Fixture / Flake / Results 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| 全仓测试源码 | 5,582文件 / 959,107行 / 35,916,766 bytes | E2全量inventory；domain/language/attribute/shape/规模统计 |
| Rust/Cargo | 21,211个`#[test]`、190 ignored、161 manifest、130 integration targets | E3 Hub可达性、Cargo metadata、默认命令、进程拓扑与ignored分类 |
| Tool Python/PowerShell | 659个Python test文件 / 4,295 methods；36个`*.Tests.ps1` | E2全量统计；CI consumer、两类PowerShell runner与代表suite E3 |
| Web与前端 | Coordinator custom Node runner、Hub package、Workbench preview | E3 scripts/dependency/workflow consumer与absence proof |
| selected combined scope | 42条Git index record / 23,176行 / 975,128 bytes | fingerprint `aba86c64...cd7d` |

Test architecture轮次详细阅读覆盖：

- 扫描排除`dev`、`docs/tests`和计划后的测试源码，确认5,582文件、959,107行；Rust共有21,211个test attribute，不能由数量推断默认可达性或发布可信度；
- 通过Hub manifest与Cargo metadata确认lib target `test=false`，而98个source保留61个test module和258项inline test；默认workspace test不会执行它们；
- 统计Hub外部39文件/270项测试中81次源码读取和189次`.contains()`，确认source-shape合同不能替代被关闭的行为测试；
- 量化Tool Python 659文件/4,295 methods，确认主CI只显式运行3个模块；聚焦执行这35项测试全部通过，但没有覆盖其余656文件；
- 识别36个PowerShell tests分成23个Pester和13个standalone，workflow没有统一发现/执行；Coordinator Web有custom check但CI不调用，Hub前端无test script；
- 通过36 package/184 target/130 integration target的Cargo metadata与环境变量/路径/端口/进程扫描，确认crate内局部mutex不能提供跨test-process资源隔离；
- 统计190个ignored tests、45个千行以上测试文件，以及大量source read/string contains断言；定义typed manual/capability/quarantine状态和Architecture/Behavior lane分离；
- 确认无property/fuzz framework/target、compile-fail harness、coverage pipeline、统一JUnit/result store、flake history/quarantine或source/build-bound ValidationSet；
- 对照Unreal Automation/Gauntlet分类与设备结果、Bevy集中CI tool、Godot Doctest元数据、Fyrox跨OS基线和Unity Graphics Wrench package validation；未把任一参考仓库描述为完整无缺的全引擎测试系统。

本轮没有修改test harness、Cargo manifest、workflow、Rust/Python/PowerShell/Node测试或证据。详细3个P0、52个P1、10个P2、M0-M6路线及24个验收门见`zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md`。

## 51. Serialization / Project / Resource / Reflection / World Sync / Public DTO 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| selected interface scope | 152 / 10,739 / 368,698 | E3：serialization、project、resource、reflect、world_sync；Hub/export/editor contribution/math为次级DTO |
| selected production | 132 / 8,068 / 274,928 | E3：排除dedicated test路径；包含4个inline test attributes |
| selected dedicated tests | 20 / 2,671 / 93,770 | E2静态：108个test attributes；动态测试未执行 |
| production schema adoption | 4个`VersionedSchema` / 0个generic binary caller | E3 caller/absence search：settings、dynamic scene、reflected JSON、ExportPreset均使用text |
| selected fingerprint | 152条Git index record | `cacb85cf...8202`；`lib.rs` modified与未跟踪`host_output`要求实施前复取 |

Runtime Interface DTO轮次详细阅读覆盖：

- 追踪text/binary write、canonical object/spool、header/load/migration与4个production schema，保留byte/node/depth/future-version/finite-float防线；确认generic binary没有production caller且写入会在final cap前物化多份完整表示；
- 确认所谓stable UUID使用Rust未承诺跨版本稳定的`DefaultHasher`，并被AssetUuid、ResourceId、locator reference及Editor/插件生产调用消费；当前无算法版本、golden vector、legacy redirect或collision catalog；
- 确认canonical object为每个value创建并长期持有一个open temp file，64 MiB只限制bytes而不限制entry/depth/handle/spool，temp又无attempt directory/journal/crash sweep；
- 逐读ResourceLocator/RelPath/project manifest/template/session lock，确认locator依赖宿主`Path::components`并参与stable ID，RelPath root保证只属词法，manifest绕过ProjectName、忽略library_version且default_scene仍为raw String；
- 逐读resource handle/kind/record/reference，确认typed handle wire丢失kind、record可构造矛盾provenance/state、两套AssetRef模型并存且UUID/locator可互相不一致；
- 逐读reflection type/field/value/address/read-write，确认validated constructor可被derived Deserialize绕过，registration存在重复/矛盾字段，缺stable field ID、central registry、revision/CAS/transaction/permission；
- 从WorldQuery追踪runtime whole-world inspection与FFI output，确认producer先构造反射JSON并collect全部rows，无page/cursor/snapshot/max bytes/deadline/cancel，Rows又不携带generation；
- 交叉读取Hub mailbox/recent project、ExportPreset/report、diagnostics/editor contribution/math，具体执行问题分别路由Hub01、Tooling03/04、Editor05而不重复登记；
- 对照Unreal CustomVersion/SoftObjectPath、Godot ResourceUID/binary format、Bevy AssetId/TypeRegistry、Fyrox Visitor/ResourceManager和Unity Graphics逐asset migration；没有把参考引擎局部实现描述为完整安全模板；
- 两次Windows Cargo聚焦测试均未进入test binary：共享target缺失serde artifact，独立target在编译中目录消失并导致dep-info/link缺失；源码library编译只出现4个既有UI warning，行为测试状态仍是未执行。

本轮没有修改production interface/runtime/editor/Hub/export代码或测试。详细4个P0、56个P1、10个P2、M0-M6路线和跨平台/兼容/资源故障gate见`zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md`。

## 52. Editor Command Registry / Keymap / Menu / Palette / Remote Automation 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| command core与commandlet | 21 / 5,078 / 183,126 | E3：descriptor、registry、factory、when、keymap、menu、palette与CLI；fingerprint `74d04199...c1c7ffb` |
| product routing与presentation bridge | 24 / 3,730 / 158,889 | E3：settings、eval snapshot、control/event/operation dispatch、retained menu/palette与workbench identity；fingerprint `1caac178...005f6ad3` |
| public plugin/control boundary | 4 / 779 / 28,144 | E3：shared contribution/control DTO、plugin SDK与Editor materializer；fingerprint `b86316bf...fb18653` |
| focused tests | 10 / 1,358 / 52,417 | E2：连同core inline tests共79个test attributes、0 ignored；fingerprint `ddf79850...4c701a` |
| selected combined scope | 59 / 10,945 / 422,576 | 当前工作树fingerprint `97c797b8...40dd6c6`；7个scoped文件在途，要求实施前复取 |

Editor Command轮次详细阅读覆盖：

- 从descriptor/registry/factory追到menu、palette、keymap和operation execution，保留canonical path、typed factory、immutable eval/palette snapshot、bounded query和完整chord复核等正向基础；
- 逐分支追踪`UiControlRequest`，确认CallAction检查remote-callable，而InvokeRoute/InvokeBinding没有；operation binding又以UiBinding执行并在现有行为测试中把direct control调用记录为RetainedHost，构成policy与provenance双旁路P0；
- 交叉读取Runtime Interface DTO、plugin SDK、Editor materializer和`EditorOperationPath`，确认前三者用二段ID作为成功fixture，宿主强制至少三段，公开合法值无法materialize；
- 继续追踪serialized Command，确认它只有ID/显示名，materializer注册Operation descriptor却没有executor/factory，修正ID后仍必然`MissingFactory`，构成独立P0；
- 逐读registry serde/admission、WhenClause和context projection，确认derived Deserialize绕过不变量、无owner/unregister/atomic batch、递归when无预算、remote默认allow且feature capability被误作caller policy；
- 从native keyboard event追到keymap resolution，确认同chord静默取字典序首项、disabled首项不fallback、未知override可截获输入、默认TOML与descriptor双authority，且产品没有Keymap Editor；
- 从built-in/extension menu追到retained legacy action，确认深层路径被压平、两种树算法并存、模型缺section/check/visibility等语义，legacy prefix还能绕过registry直接执行layout command；
- 从palette catalog/index/query/commit追到settings MRU，确认bounded heap/window基础真实，同时定位ASCII/Unicode归一化分裂、effective shortcut不刷新、无query cancel/budget及性能测试只打印p95不设门槛；
- 从commandlet metadata追到parser/action/report，确认CLI仍硬编码三个命令及flag，payload schema不参与执行，报告无version/request/progress/cancel，capability又由runner分支写死；
- 对照Unreal UICommandList/InputBindingManager/ToolMenus的action context、unmap、owner、section、profile，Godot Shortcut/command palette的多事件/add-remove/settings刷新，以及Fyrox CommandStack/key settings；Bevy仅作UI示例参考，Unity Graphics不含权威Editor command shell。

本轮没有修改production command/editor/plugin/interface代码，也没有运行Cargo、真实Editor、插件DLL、remote transport、键盘布局矩阵或万级命令benchmark。详细3个P0、44个P1、10个P2、M0-M6路线及24个验收门见`zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md`。

## 53. Editor Background Jobs / Admission / Scheduling / Cancellation / Progress / Shutdown 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| job core | 47 / 9,072 / 297,158 | E3：admission、reservation、queue、dependency、ticket、cancel、progress、observer、pump、shutdown与全部测试源码；fingerprint `dee19973...94fd0` |
| product integration与绕行边界 | 47 / 13,162 / 461,928 | E3：context、save/import/autosave、notification、asset refresh、export、welcome、viewport/profile、host close及自管thread owners；fingerprint `c5ab7625...f8bc71` |
| runtime scheduler boundary | 6 / 1,843 / 61,473 | E3：compute pool、scheduler、handle、diagnostics、pool ownership与report；fingerprint `af86c15d...d2ca7` |
| selected combined scope | 100 / 24,077 / 820,559 | 当前工作树fingerprint `222e262a...f69fc`；job core含108个test attributes、0 ignored，14个scoped文件在途 |

Editor Background Jobs轮次详细阅读覆盖：

- 保留pending 16,384项/64 MiB/5分钟admission、批量reservation Drop回滚、finite category quotas、三档公平slot、mutex/dependency、typed ticket、panic containment、progress generation、observer resync与64项/1 ms pump等正向基础；
- 从keyed admission追到promotion、progress cancel authority及welcome project probe，确认merge使用latest token运行却仍按首次token取消，现有合同测试期待latest token但本轮未执行，构成P0；
- 从`EditorJobSystem::shutdown`追到autosave service和retained app closeout，确认deadline只返回unfinished，产品随后仍拆卸project/settings/host state，没有join、隔离或late-commit barrier，构成P0；
- 逐读event queue与storm测试，确认progress可coalesce但Started/terminal lifecycle进入无界`VecDeque`，consumer暂停时pending admission不能限制长期内存，构成P0；
- 量化spec/limits/queue/runtime scheduler，确认4 KiB自报estimate只覆盖pending、category quota之和可超过worker width、priority不进入executor、IO/process/compute共用compute pool；
- 追踪dependency/cancel/result，确认`.after`只排序且失败不传播、terminal history固定256、cancel无reason/ack/quiescence、所有snapshot硬编码cancellable、ticket wait无deadline；
- 追踪progress/observer/pump/notification，确认primary按最小JobId、terminal outcome立即消失、production message-bus consumer缺席、pump只由retained tick驱动且无queue health metrics；
- 逐个登记save/import/autosave/preview/export/welcome/viewport/profile adapter及compile/play/live-watch线程，确认产品手工generation/ticket治理、foreground save同步wait、thread ownership source合同与真实owner不一致；
- 对照Unreal asset compiling/async work/notification、Bevy TaskPool、Godot WorkerThreadPool、Fyrox owner-bound completion和Unity Graphics typed jobs；按层取其原则，未虚构任何单一参考具备完整Editor authority；
- 聚焦Cargo测试编译617.2秒后被当前lib-test 239个既有错误/122个warning阻断，测试二进制和目标测试均未执行，不把源码合同写成动态结果。

本轮没有修改production job/runtime scheduler/product adapter/thread owner或测试。详细3个P0、48个P1、10个P2、M0-M6路线及24个验收门见`zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md`。

## 54. Editor Notification Center / Toast / Decision / History / Actions / Retention / Accessibility 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| notification core | 25 / 3,469 / 110,835 | E3：identity/source、toast/progress/decision model与center、presentation、service和38个test attributes；fingerprint `11deead3...0c7508` |
| product projection与接入 | 60 / 9,299 / 343,875 | E3：context、activity、Play adapter、toast producers、retained bridge/parser/cache/painter和Workbench assets；fingerprint `8f778590...6e7b83` |
| status-line绕行面 | 126 / 10,985 / 408,716 | E2 inventory、代表error链E3；317条生产匹配行，fingerprint `f2c866e9...e67836` |
| selected combined scope | 208 / 23,488 / 853,636 | 当前工作树fingerprint `b7f7e681...3b4bb2`；core 38个test attributes、0 ignored |

Editor Notification轮次详细阅读覆盖：

- 保留严格`NotificationId`、builtin/plugin source、有界toast/progress/decision center、单调expiry、decision ticket/incarnation/receipt cursor、job progress绑定、locale一致presentation和retained row cache等正向基础；
- 从toast publish追到`BTreeMap` snapshot和`.first()` current toast，确认queue按ID字典序而lifetime从publish开始，后来的短寿命error可在从未显示前过期；固定ID duplicate又被产品当成功吞掉，构成P0；
- 从Workbench asset和bridge追到native painter，确认普通toast/progress只令center `visibility=visible`，却保持closed/non-interactive且没有入口；live项消失后“history”同步清空，构成P0；
- 从raw pipe entry追到component parser，确认`severity=error`随后被`kind=toast`按tone再次解析成fallback info，所有toast severity在center row中降级，构成P0；
- 确认bridge只保留64项而asset `visible_limit=8`先截断，overflow仅统计超过64；decision options、progress、toast固定顺序又可挤掉critical row；
- 确认toast model没有action/dismiss，window没有action route，但painter和测试按宽度固定绘制英文`UNDO`与close mark；非decision center row选择也落入无`options`的dropdown路径成为no-op；
- 追踪decision default/cancel从core presentation到Play adapter丢失、options被扁平为独立row、无withdraw/owner revoke、每次history变化重置focus，以及`aria_modal=false`与强制modal矛盾；
- 量化126个生产status-line文件/317条匹配和17条toast publish相关生产匹配，确认Notification Service、Editor Log、status summary、provider diagnostic与plugin SDK尚未形成统一record/journal/delivery authority；
- 对照Unreal Slate Notification与Message Log、Godot Toaster/Editor Log、Unity Graphics provider/model diagnostics、Fyrox log/UI primitives和Bevy bounded diagnostic history，按层吸收原则且未虚构任何单一参考拥有完整目标架构；
- 同一工作树上一轮`zircon_editor --lib`已在617.2秒后被239个既有test-build错误阻断，本轮未重复该Cargo lane；现有测试源码反而明确固化closed center、无history和固定`UNDO`行为。

本轮没有修改production notification、retained host、UI asset、logging、jobs、plugin SDK或tests。详细3个P0、52个P1、10个P2、M0-M6路线及26个验收门见`zircon_editor/10-notification-center-toast-decision-history-actions-retention-accessibility-diagnostic-integration-review.md`。

## 55. Editor Logging / Diagnostic Journal / Output Console / Status Routing / Retention / Export 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| logging core | 13 / 1,889 / 59,995 | E3：config、entry/source/severity/jump、record/store、rolling file、service/filter/error及16个test attributes；fingerprint `4c94960e...a08aa932` |
| logging产品接入 | 63 / 12,694 / 464,755 | E3：builder、message sink、project/play/build/host producers、activity console、retained projection与两个Console资产；fingerprint `93c33665...f32ecab` |
| status/tracing绕行面 | 131 / 13,268 / 492,332 | E2 inventory、代表链E3：生产status/tracing调用集合；fingerprint `ca481c55...d1a49c` |
| selected combined scope | 192 / 23,899 / 873,142 | 当前工作树去重fingerprint `4225cab0...d37a0b`；core 16个test attributes、0 ignored |

Editor Logging轮次详细阅读覆盖：

- 保留entry/estimated-bytes双限内存store、clear后不复用的checked sequence、typed source/severity/jump、event backpressure/resync、reentrant sink测试及retained Console的filter/follow-tail/可见行裁剪基础；
- 从`emit`追到rolling append，确认每条record在全局emission mutex内执行目录/metadata/open/write/flush，解锁后producer仍同步drain唯一sink；慢盘或日志storm可阻塞UI/job/Play路径，构成P0；
- 从builder sink追到message bus registration，确认生产没有`EditorTopic::log()` subscriber，零subscriber dispatch却被映射为`Delivered`；Console依赖未来的全snapshot扫描而不是event invalidation，构成P0；
- 量化status/tracing绕行并追踪runtime diagnostics，确认数百处status写入、少量tracing、runtime snapshot、notification及Plugin/Import没有process-wide record authority；Plugin/Import枚举存在不等于产品adoption；
- 逐读rolling file格式和project lifecycle，确认无目录总bytes/age/file-count、session/process/build identity、跨进程协调、crash fence和版本parser；配置不probe I/O、project-open记录早于sink启用、append错误又被多数caller丢弃，构成P0；
- 追踪Console snapshot/filter/clear/jump，确认All与Info完全相同、Warning含Error、全量clone后静默截256行、index row identity、无search/copy/export/pause/timestamp，ScriptLocation不真正定位caret；
- 区分append-only event record、owner/generation provider diagnostic、status summary与notification delivery，避免用一个字符串store承载四种冲突生命周期；
- 对照Unreal OutputDeviceRedirector/Tokenized Message/Message Log/Output Log、Godot EditorLog、Bevy tracing、Fyrox listener和Unity Graphics provider diagnostics；只吸收各自可验证原则，不把局部源码描述成完整引擎上限；
- 同一工作树上一轮`zircon_editor --lib`已在617.2秒后被239个既有test-build错误/122个warning阻断，本轮未重复该Cargo lane，也未把16个静态test attributes写成动态通过。

本轮没有修改production logging、message bus、Console、status routing、runtime diagnostics、plugin SDK、child protocol或tests。详细3个P0、57个P1、12个P2、M0-M6路线及28个验收门见`zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md`。

## 56. Editor Settings / Preferences / Scope Persistence / Locale / Appearance 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| settings core与tests | 16 / 3,937 / 135,352 | E3：authority、registry、definition、snapshot、change log、store、startup、persistence、scope/page/defaults及34个test attributes；fingerprint `c41339da...17197e23` |
| i18n core、tests与bundles | 9 / 1,199 / 44,638 | E3：locale、catalog、service、sink/resync、macro/error、en/zh-CN bundle及10个test attributes；fingerprint `aa970691...1702e95` |
| 产品与appearance接入闭包 | 51 / 14,456 / 533,428 | E3：context、viewport persistence、plugin page materialization、notification translation、retained/V2 appearance和Preferences shell；fingerprint `03640bd8...91173fb` |
| Editor ZUI literal inventory | 248 / 39,353 / 2,498,794 | E2：3,201个可见/可访问文本属性，77个资产显式导入editor token资产；fingerprint `1b9887f2...5ed87922` |
| selected combined scope | 312 / 55,961 / 3,108,922 | 当前工作树去重fingerprint `ff3c73da...02e93a28`；core共44个test attributes、0 ignored |

Editor Settings轮次详细阅读覆盖：

- 保留validated dotted key、typed schema、User/Project/Session precedence、immutable ArcSwap snapshot、有界change log、atomic layer replace、versioned temp-write/flush/rename、project source binding和bounded keyed I/O ticket/fence基础；
- 从`SettingsAuthority::set/clear`追到snapshot、唯一hot subscriber和persistence service，确认内存修改先成功，持久化由caller可选；生产只有Scene Viewport三个Project snap值submit，User locale/design tokens/keymap/job quota无通用durable owner，构成P0；
- 追踪viewport submit/retry/project transition和host shutdown，确认failure不回滚或标dirty、ticket generation不等于worker实际序列化的live layer generation、不同key重复整文件写，shutdown又使用无deadline fence；
- 逐读store/document，确认`.toml`实际承载JSON envelope、0->1 migration只会拒绝、unknown plugin key使整个layer失败、read-to-string前无size cap，且无LKG/quarantine/external edit/CAS/cross-process merge；
- 逐读registry/page/plugin materializer，确认definition只可启动前注册，page与definition断开、无owner lease/unregister/query snapshot；`SettingsPageDescriptor`进入extension snapshot却没有生产consumer；
- 从`FloatingWindow::preferences()`追到ZUI和production caller absence，确认窗口只有General/Layout/Preferences静态文本，无打开route、setting控件、search、scope/origin、apply/reset/restart/error或accessibility workflow，构成P0；
- 逐读locale/catalog/service/builder sink和notification projection，保留captured-locale一致性、English fallback、generation拒旧、bounded queue/resync；同时确认生产无i18n topic consumer，普通零consumer可假Delivered；
- 量化两份bundle各54个相同key与248个ZUI资产3,201个文本属性，确认主shell不经localized identity/revision，locale hot-sync只覆盖小文本岛并产生混合语言，构成P0；
- 追踪design token setting到retained tick、V2 injection和host ArcSwap，确认真实热应用基础存在；同时登记无完整token validator、多个派生authority/generation、77/248 token import、未引用unreal-dark资产及17个runtime renderer独立default fallback；
- 对照Unreal Settings/DeveloperSettings/TextLocalizationManager、Godot EditorSettings/Dialog/TranslationServer/ThemeManager、Fyrox真实Settings plugin、Bevy Feathers theme resource和Unity Graphics provider discovery；按层吸收原则，不把任一局部实现描述为完整目标。

本轮没有修改production settings/i18n/Preferences/theme/runtime UI代码或测试。上一轮`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复相同lane；44个test attributes只作为静态inventory。详细3个P0、58个P1、12个P2、M0-M6路线及30个验收门见`zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md`。

## 57. Editor Layout Profile / Workspace State / Dock / Tab / Window Restore 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| layout/view/window registry/preset领域 | 78 / 5,032 / 169,585 | E3：model、command/apply/attach/detach/drop/normalize/restore、view policy/registry、window registry和page preset；fingerprint `507ff857...54723c4c` |
| project workspace与host persistence | 37 / 3,506 / 128,503 | E3：project envelope/store/save rollback、preset asset、host apply/capture/restore、builtin repair和window host；fingerprint `1c7b4ba9...114bb0ef` |
| retained drag/drop与native projection | 37 / 3,477 / 118,935 | E3：tab drag、layout callback、floating projection、native target/presenter/close和recompute；fingerprint `1f725ee9...3698c12a` |
| focused tests | 35 / 6,199 / 215,971 | E3静态阅读：113个test attributes、0 ignored；fingerprint `8b9cf8eb...0e47938` |
| selected combined scope | 187 / 18,214 / 632,994 | 当前工作树去重fingerprint `3e3fdd86...7572f5b9`；24个scoped source/tests文件在途 |

Editor Layout轮次详细阅读覆盖：

- 保留typed page/window/view/instance ID、递归document split tree、layout diff/generation、descriptor policy元数据、versioned workspace envelope、native close decision和presenter surface reuse基础；
- 从retained callback/binding追到manager apply与host registry，确认Move/Attach/CreateSplit先detach后验证，OpenView先创建instance后attach；错误不回滚并可产生orphan，构成P0；
- 从ResetToDefault追到document toolkit clear，确认layout偏好操作绕过dirty close decision并可静默丢失未保存会话，构成P0；
- 从project open追到`apply_project_workspace_state`及UI asset二阶段恢复，确认先清live workspace再逐项`?`恢复，第N项失败会留下旧状态已毁、新状态半成，构成P0；
- 逐读page preset capture/apply，确认它只保存drawer/整数size和粗粒度center split，恢复先合并全部tabs再构造right-deep空叶，且产品允许不存在page ID，构成P0；
- 逐读workspace/preset/config格式，确认外层format 1虽校验，内层layout version及preset version/name未消费；无bounded parse、完整validator/migration、unknown-plugin placeholder、LKG/quarantine，构成P0；
- 追踪`DockPolicy`、`PersistenceKeyPolicy`、drag kind、reflection和binding，确认metadata未形成所有入口一致执行的effective policy；duplicate ID/placement又可静默覆盖；
- 追踪project-local workspace、named preset与Runtime ConfigManager，确认用户/项目profile混层、preset名称碰撞/raw write/import后置、async durability和错误静默fallback；scene/workspace补偿也不是crash-atomic generation commit；
- 从Workbench floating frame追到WindowHostManager、native target和presenter，确认没有monitor/work-area/DPI/window state，未观察到OS move/resize回写authoritative layout，backend sync失败也可留下部分topology；
- 对照Unreal TabManager/LayoutService、Godot DockManager/WindowWrapper、Fyrox recursive dock config、Bevy logical/physical window abstraction和Unity Graphics局部EditorWindow migration；明确Bevy/Unity Graphics checkout不提供完整Editor shell依据。

本轮没有修改production layout/workspace/preset/native window代码或tests。上一轮`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复相同lane；113个test attributes只作为静态inventory。详细5个P0、58个P1、12个P2、M0-M7路线及32个验收门见`zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md`。

## 58. Editor Animation Sequence / Graph / State Machine / Timeline / Curve / Preview / Compiler Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| session/event/binding/host authoring核心 | 31 / 5,147 / 188,207 | E3：open/restore/mutate/dirty/save及三类document逐分支；fingerprint `197af8bc...e4fae73d` |
| 产品projection、ZUI、showcase与preview control | 52 / 8,814 / 380,881 | E3代表链、E2全量inventory；fingerprint `99d88d06...01fe5b06` |
| runtime asset schema与glTF导入接点 | 20 / 2,098 / 67,952 | E3：binary/schema/reference与skeleton/clip derivation；fingerprint `ca9c0a02...68d188b4` |
| focused tests | 17 / 3,909 / 141,474 | E3静态阅读：58个test attributes、0 ignored；fingerprint `ee4b2275...7fc8693f` |
| selected combined scope | 118 / 19,681 / 767,992 | 本轮取证时去重fingerprint `6e5d6e57...16b2175`；成文时13个animation/timeline/weight source/tests文件在途 |

Editor Animation轮次详细阅读覆盖：

- 保留`ZRANIM01` magic/version/kind、payload fallback、typed channel/Graph/State Machine schema、document dirty/autosave接点、glTF skeleton/clip derivation和timeline/heatmap paint primitive；
- 从Asset Browser `OpenAsset`追到type registry和toolkit，确认production只为UI资产注册toolkit，animation打开测试先手工注入toolkit，默认产品无法打开Sequence/Graph/State Machine，构成P0；
- 从ZUI按钮追到binding/event/session，确认Scrub固定frame 0、Add Node发送unsupported `State`，timeline/canvas又为空slot；高级animation workspace多数只写queued字符串，构成P0；
- 从undo policy追到event execution与session dirty registration，确认animation被声明delegated却直接原地修改、无transaction ID/undo，且后置dirty注册失败可留下已改source，构成P0；
- 对照runtime binary和asset schema，确认envelope有版本与kind校验，但Editor/runtime没有共享semantic compiler；ID/reference/pin/type/cycle/condition/layer/property binding无统一验证，构成P0；
- 搜索Sequence playback与Blend Space transport consumer，确认前者只改session标签、后者只改control text/checked和固定0/3秒时间，没有preview world/runtime evaluator，构成P0；
- 逐读Sequence key/track/range/rebind，登记Scalar+Step默认、last-key复制、float EPSILON identity、无typed value/curve/tangent/multi-select/event/root motion和timebase合同；
- 逐读Graph node/connect/parameter，登记只新增Output/Blend、无port/type/cycle、Additive只写base、string猜parameter type、无canvas metadata/plugin node/compiler/debug map；
- 逐读State Machine，登记只新增GraphRef、单pair transition、固定30 FPS、无exit/interruption/layer/condition group/Blend Space authoring；
- 对照Unreal Persona/Animation Blueprint、Godot Animation Track Editor、Fyrox Animation/ABSM command和Bevy runtime graph分层；明确Unity Graphics checkout不含核心Animation Editor，不能作为完整authoring依据。

本轮没有修改production animation/editor/runtime代码或tests。上一轮`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复相同lane；58个focused test attributes只作为静态inventory。详细5个P0、60个P1、12个P2、M0-M7路线及32个验收门见`zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md`。

## 59. Editor Material / Shader Graph / Instance / VFX / Particle / Preview / Compiler Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Editor projection、registry、operation dispatch与Workbench/ZUI产品面 | 12 / 3,917 / 183,586 | E3代表链、E2全量inventory；fingerprint `07128120...f67fe92e` |
| Material Editor、WGSL importer及runtime material/shader/graph合同 | 24 / 3,414 / 125,764 | E3：registration、schema、validator/compiler、import与projection；fingerprint `be9e4e79...843f2f3e` |
| Particles editor/runtime/package | 57 / 8,733 / 306,329 | Editor E3、runtime接点E2；fingerprint `121a10a8...fd48254b` |
| Rendering Shader Graph / VFX Graph features | 16 / 592 / 19,711 | E3：schema/compiler/feature/executor/editor registration；fingerprint `9f88f84a...0794a0b8` |
| selected combined scope | 109 / 16,656 / 635,390 | 本轮取证时去重fingerprint `520c48ec...44c96c5`；67个test attributes、0 ignored，范围内无在途source |

Editor Material/VFX轮次详细阅读覆盖：

- 保留丰富`MaterialAsset`/`ShaderAsset`合同、Naga WGSL parse/validate、typed Material/RendererData projection及Particles CPU/GPU runtime基础；
- 从builtin asset registry和default linked catalog追到plugin registration，确认Material/MaterialGraph/Shader无builtin toolkit、默认catalog不包含Material/Particles/Shader/VFX，且Material插件引用的graph ZUI/default template物理缺失，构成P0；
- 从authoring contribution batch追到operation dispatch，确认Material/Particles只注册descriptor而没有factory，Graph descriptor/palette无产品consumer，Particles菜单disabled且ZUI为空，构成P0；
- 从Workbench action追到module/extension feedback，确认Material/VFX/Shader/Particle compile/simulate/preview只改固定status/output字符串，不调用compiler/job/runtime，构成P0；
- 对照runtime authoring graph、graphics shader assets和rendering Shader Graph/VFX Graph，确认三套schema分裂，importer/compiler规则冲突，generated WGSL未复验且Shader/VFX executor no-op，构成P0；
- 搜索Material projection与Particles asset/importer consumer，确认projection仅tests使用，`particles.system`无runtime importer，Material/VFX/Particle也没有transactional document、durable save、derived artifact和runtime一致preview，构成P0；
- 逐读Material validator/compiler，登记pin/type/cardinality/topology缺口、palette float与vec4语义冲突、只计算base color、texture arithmetic拒绝及其他Material字段固定default；
- 逐读WGSL importer、Shader Graph generator和VFX Graph compiler，登记reflection/dependency/target/permutation/source-map缺口、任意ID拼identifier、未定义texture helper、fixed pass/no-op executor；
- 逐读Particles editor/runtime边界，区分真实CPU/GPU simulation/render基础与descriptor/template/Space-only Editor shell，避免把runtime基础误判为零或把Editor声明误判为产品完成；
- 对照Unreal Material Editor/Niagara、Godot Shader/Visual Shader/Particles、Fyrox Material/Particle、Unity ShaderGraph/VFXGraph和Bevy runtime material/render resource分层。

本轮没有修改production Material/Shader/VFX/Particle/editor/runtime代码或tests。上一轮`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复相同lane；67个test attributes只作为静态inventory。详细5个P0、60个P1、12个P2、M0-M7路线及32个验收门见`zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md`。

## 60. Editor Terrain / Landscape / Foliage / Scatter / World Partition / Level Streaming Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Terrain plugin editor/runtime/dist/package | 16 / 856 / 31,660 | E3逐文件：registration、import plan、diagnostic importer、dist ABI及8个test attributes；fingerprint `5d2d9c02...c30a0d11` |
| Editor World Building Workbench、route、binding与feedback | 14 / 4,821 / 243,025 | E3代表action全链、E2完整ID inventory；fingerprint `f2385462...e1d72a8` |
| Runtime Terrain asset/import/artifact/scene/level接点 | 16 / 5,396 / 196,621 | E3资产链与consumer absence；fingerprint `487b6310...0f125ab` |
| Catalog、authoring batch、plugin materialization与operation dispatch | 9 / 2,216 / 83,245 | E3默认装配、toolkit与MissingFactory路径；fingerprint `0707a4ef...7e99bca9` |
| selected combined scope | 55 / 13,289 / 554,551 | 当前工作树去重fingerprint `fade7a6d...a5307da`；44个test attributes、0 ignored，1个import排序在途source |

Editor World Building轮次详细阅读覆盖：

- 保留Terrain/LayerStack asset、TOML importer、artifact/cache/facade/load、SceneTerrain引用及Terrain editor/runtime/dist package基础，避免把真实资产接点误判为零；
- 从default linked catalog、builtin toolkit、plugin registration追到resource resolution，确认三份`plugins://terrain/...`目标资源均缺失且默认产品装配不闭合，构成P0；
- 从authoring batch追到operation dispatch和scene-mode handoff，确认Import/Create/Open/Sculpt只注册descriptor、无factory，Terrain mode仍无input/transaction/overlay owner，构成P0；
- 逐读Editor import plan与runtime plugin，确认前者不读bytes，后者明确使用backend-not-installed诊断导入器且无renderer/service/height query，构成P0；
- 从Terrain asset/import/artifact/load追到graphics/scene consumer全局搜索，确认serialization存在但执行consumer为零；LevelSystem也只有整World Loaded/Unloaded，无cell/streaming/data layer/HLOD authority；
- 从四份ZUI追到binding、reference action和feedback，确认Terrain/Foliage/Scatter/Level Streaming的64/96 cells、64K/84K instances、HLOD和warning均为固定字符串，不来自job/runtime，构成P0；
- 搜索Foliage/Scatter资产、规则、compiler、instance artifact和runtime system，确认生产domain不存在；旧Terrain/Vegetation计划TV-M1至M4仍未落地；
- 对照Unreal LandscapeEditor/FoliageEdit/WorldPartitionEditor、Unreal Runtime Landscape、Fyrox terrain interaction/commands/runtime、Godot height map/MultiMesh与Unity Graphics TerrainLit；明确后两者不是完整World Building Editor基准。

本轮没有修改production Terrain/World Building/editor/runtime代码或tests。上一轮`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复相同lane；44个test attributes只作为静态inventory。详细5个P0、60个P1、12个P2、M0-M8路线及32个验收门见`zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md`。

## 61. Editor Sound / Audio Clip / Mixer / Spatial / Acoustic / Timeline Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Sound editor/package | 16 / 1,755 / 63,878 | E3逐文件：plugin、33 operations、五份ZUI、live-output controller/DTO及6个test attributes；fingerprint `0e7b89f7...44549db` |
| Timeline audio与ray-traced convolution可选feature | 24 / 1,064 / 39,851 | E3逐文件：editor/runtime/dist/capability/manifest及10个test attributes；fingerprint `d0b20b8c...1b0f7f4` |
| Sound runtime生产实现，不含`src/tests` | 232 / 11,754 / 389,282 | E2交叉复核Runtime08B，E3复核scene registration、output telemetry与Editor依赖接口；fingerprint `0dd69693...ab3e2be` |
| Sound asset与三组audio importer | 22 / 2,400 / 84,995 | E3资产/decode/import authority及23个test attributes；fingerprint `720cd9b2...7862814` |
| Runtime core Sound公共合同 | 28 / 2,142 / 65,840 | E2完整inventory，E3复核三类scene component与Editor依赖；fingerprint `008a130b...e3c238` |
| Editor共享catalog/toolkit/operation/asset-open/Sequencer接点 | 6 / 1,418 / 60,272 | E3默认装配、No-toolkit、MissingFactory和Audio Theme静态行；fingerprint `60572a82...16e0b2` |
| selected combined scope | 328 / 20,533 / 704,118 | 当前工作树去重fingerprint `c9a49221...8ace0e`；86个test attributes、0 ignored，范围内无在途文件 |

Editor Sound Authoring轮次详细阅读覆盖：

- 保留Runtime Sound typed services、Kira/CPAL backend、output lifecycle、严格WAV解析、Symphonia codec导入及真实live-output controller，不把已有基础误判为零；
- 从builtin Sound type、default linked editor catalog和`OpenAsset`追到toolkit选择，确认默认产品没有Sound asset editor或只读fallback，构成P0；
- 从33个operation追到authoring batch与operation dispatch，确认全部只有descriptor、没有factory，三个可见output route也无法到达controller，构成P0；
- 逐项读取五份ZUI，确认Mixer、三类component drawer与Acoustic Debug合计29个业务`Space`，只有Refresh/Start/Stop三个event且无executor，构成P0；
- 从live-output controller追到production caller与runtime telemetry写入点，确认controller只在测试构造，callback counters/xrun没有生产更新，构成P0；
- 从SoundAsset/importer追到Editor入口，确认全量PCM DTO、无provenance/settings/loop/marker/waveform/stream policy，三组importer还存在重复authority；
- 从runtime plugin/component descriptor追到scene consumer，确认没有RuntimeSceneSystem或AudioSource/Listener/Volume同步；Editor也没有transaction/audition/save acknowledgement，构成P0；
- 逐读两个optional feature，确认Timeline audio与ray-traced convolution的Editor仅descriptor/capability、Runtime仅空module；Sequencer Audio Theme/Ready为固定展示；
- 对照Unreal AudioEditor/MetaSoundEditor、Fyrox audio panel/preview、Godot waveform/import/bus/gizmo与Bevy runtime audio；明确本地Unity Graphics不含Audio Editor，不能作音频完成度基准。

本轮没有修改production Sound/editor/runtime代码或tests。上一轮`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复相同lane；86个test attributes只作为静态inventory。详细5个P0、60个P1、12个P2、M0-M8路线及32个验收门见`zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md`。

## 62. Editor Physics Material / RigidBody / Collider / Joint / Collision Cook / Ragdoll / Debug Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Physics editor/package | 13 / 722 / 25,310 | E3逐文件：四份ZUI、registration、overlay/generator及4个test attributes；fingerprint `9a080c1b...364ab6` |
| Physics runtime生产实现，不含tests | 66 / 8,516 / 299,179 | E2交叉复核Runtime08A，E3复核Editor所需scene/ragdoll/debug/cook接点；fingerprint `b2d6c8f3...827b3a` |
| Collision Proxy/Physics Collision Workbench链 | 7 / 2,180 / 123,251 | E3读取38 routes、binding/navigation/feedback与固定数据；fingerprint `a04eba99...f4f9f` |
| Core physics与scene physics合同 | 41 / 988 / 28,436 | E2完整inventory，E3复核material/filter/shape/joint；fingerprint `61c02c06...5f194` |
| Asset/scene physics纵向接点 | 49 / 14,397 / 563,334 | E2 inventory、E3 PhysicsMaterial/RigidBody/Collider/Joint关键链；fingerprint `366c59b6...20e2e` |
| Editor共享catalog/toolkit/operation/viewport extension接点 | 6 / 1,928 / 72,881 | E3默认装配、No-toolkit、operation与overlay provider；fingerprint `b885681a...1c737` |
| selected combined scope | 182 / 28,731 / 1,112,391 | 当前工作树去重fingerprint `49c08d6d...84cfb`；124个test attributes、0 ignored，1个import排序在途source |

Editor Physics Authoring轮次详细阅读覆盖：

- 保留RuntimeSceneSystem、Jolt/builtin backend、world sync、Physics Material asset/import/artifact、overlay builder和Ragdoll generator，避免把真实基础误判为零；
- 从builtin Physics Material、default linked catalog和plugin registration追到`OpenAsset`，确认默认asset/plugin authoring入口不闭合，构成P0；
- 逐读四份plugin ZUI，确认Authoring/Debug/Diagnostics/Ragdoll由11个业务`Space`承载且0 event/controller，构成P0；
- 从Ragdoll creation template/operation追到event，确认“Generate”只OpenView，generator/from_toml/spawn helper没有product asset链，构成P0；
- 从debug toggle追到viewport extension registry，确认只OpenView、无Physics-owned provider，既有open failure仍成立，构成P0；
- 逐项追踪两份Workbench 38 routes，确认Bake/Test/Simulate/Validate只写固定queued/计数，`register_mesh_asset`无production caller，构成P0；
- 对照Unreal PhysicsAssetEditor/StaticMeshEditor/ChaosEditor与Godot shape/joint/physical-bone gizmo/UndoRedo；明确Fyrox、Bevy与Unity Graphics checkout不提供等价Physics Editor基准。

本轮没有修改production Physics/editor/runtime代码或tests。上一轮`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复相同lane；124个test attributes只作为静态inventory。详细5个P0、60个P1、12个P2、M0-M8路线及32个验收门见`zircon_editor/18-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-authoring-review.md`。

## 63. Editor Navigation Settings / NavMesh / Agent / Area / Surface / Modifier / Obstacle / Off-mesh Link / Bake / Query / Debug Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Navigation editor/package | 34 / 3,838 / 134,903 | E3逐文件：registration、operation command、panel、mirror/provider、11份ZUI及focused tests；fingerprint `d881eedb...739502` |
| Runtime operation/scene/plugin生产桥，不含tests | 59 / 8,273 / 287,822 | E3复核Editor所需operation、bake、scene、event、manager接点；fingerprint `7a8527ff...529081` |
| Native Recast/Detour bridge与vendor | 91 / 35,032 / 1,087,347 | E2按ABI、build/query/crowd/tile-cache职责复核；fingerprint `2536c84e...3d6f1` |
| Core Navigation与Asset合同 | 30 / 4,737 / 170,064 | E3公共DTO、settings、asset/import/artifact/load纵向接点；fingerprint `5876a582...e4f1` |
| Editor assembly/toolkit/provider共享接点 | 7 / 1,453 / 52,989 | E3默认target装配、asset open与extension/provider合同；fingerprint `9a8f9fd6...87c03` |
| 静态Navmesh AI Workbench链 | 7 / 2,644 / 141,758 | E3读取ZUI、binding、navigation、preview action和feedback；fingerprint `3ba21b54...429d` |
| selected combined scope | 228 / 55,977 / 1,874,883 | 当前工作树去重fingerprint `d37f25c5...1f98c`；97个test attributes、0 ignored，1个import排序在途source |

Editor Navigation Authoring轮次详细阅读覆盖：

- 保留默认`target-editor-host`/first-party catalog装配、Recast/Detour、Runtime manager、typed DTO、reader-count debug capture、可撤销snapshot command与PIE mirror/provider，避免把真实基础误判为零；
- 从Bake ZUI/panel/factory/command追到Runtime operation service/handler，确认selected payload已补齐，但两类Bake `prepare`固定失败且focused test仍期待成功，构成P0；
- 逐读11份plugin ZUI，确认Surfaces、Agents/Areas、两类asset、五类drawer和Debug viewport合计11个业务`Space`，Bake列表/诊断/progress也没有生产provider，构成P0；
- 从panel/controller到全仓caller确认其只有测试构造；asset toolkit只能打开空view，没有transaction、job、staging artifact、save/reload或generation owner，构成P0；
- 从runtime overlay event、PIE mirror、provider追到command/filter，确认producer/provider真实，但toggle只有schema、checkbox无event、编辑态无source且full-frame复制，构成P0；
- 逐项追踪Navmesh AI Workbench route、binding、navigation和feedback，确认NavMesh_Main、18 tiles、96 polys、4 agents等全为固定文本，不调用Navigation domain，构成P0；
- 对照Unreal NavigationSystem/Rendering/TestingActor、Fyrox NavMesh interaction/selection/commands和Godot 2D/3D Navigation editor；明确Bevy无游戏世界NavMesh Editor、本地Unity Graphics不拥有Navigation authoring，不能作为降低基线的依据。

本轮没有修改production Navigation/editor/runtime代码或tests。上一轮`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复相同lane；97个focused test attributes只作为静态inventory。详细5个P0、60个P1、12个P2、M0-M8路线及32个验收门见`zircon_editor/19-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-authoring-review.md`。

## 64. Runtime AI Behavior Tree / Blackboard / Perception 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Runtime core AI公共合同 | 9 / 1,026 / 30,659 | E3逐文件：tree、Blackboard、Perception、tick/report/snapshot、manager、ID与error contract |
| AI runtime生产实现，不含tests | 44 / 7,979 / 273,401 | E3逐文件：compiler/catalog/executor、Blackboard、manager、Perception、LOD、plugin registration与system |
| AI runtime tests | 20 / 6,771 / 238,656 | E3 inventory与代表断言复核；98个test attributes只证明局部算法形状，不作为产品闭环证据 |
| AI native dist | 2 / 115 / 4,087 | E3 ABI descriptor、E1业务行为；native projection的systems/events/extensions均为空 |
| generated AI manifest | 1 / 122 / 4,495 | E3 capability/dependency/component/event/module/dist声明；与真实产品能力交叉核对 |
| selected combined scope | 76 / 16,013 / 551,298 | 当前工作树fingerprint `8ef9320e...f30971`；100个test attributes、0 ignored，范围内无在途source |

Runtime AI轮次详细阅读覆盖：

- 保留版本化Behavior Tree DTO、严格拓扑校验、dense preorder program、18项typed node identity、owner lease/revoke gate、schema-compiled Blackboard、changed-slot observer、有界hearing入口、分帧Perception cursor和弱Physics/Script provider合同，避免把已有基础误判为零；
- 从`.btree.toml` compiler、manager registration和scene component追到全仓production caller，确认没有asset import/cook/load consumer、Brain/Agent/BehaviorTree/Blackboard component或普通scene启动链；
- 逐节点复核标准语义，确认`SetBlackboard`、`EmitEvent`和`UpdateBlackboardDistance`只读取`result`/`service_result`，无名称承诺的写入、事件或距离服务副作用；
- 逐读executor abort路径，确认TimeLimit、Parallel终结和owner revoke不取消全部latent leaf，递归执行又没有node/time/depth budget，同一agent并发tick可因remove/execute/reinsert分叉并丢写；
- 从MoveTo追到Navigation event/property contract，确认它扫描历史report、无request/generation/cancel ID，成功/失败/abort后的“清除”实际把当前位置写成新destination；PlayAnimation和ScriptTask同样没有真实terminal lifecycle；
- 逐读Blackboard schema/layout/store/DTO边界，确认公开contract仍由字符串type/key和完整Vec snapshot驱动，缺default/inheritance、object/asset/nav值域、entity generation、migration、save/replication/write authority；
- 从Perception scene scan、pair cursor、Physics bridge和stimulus retention追到输出，确认每帧重建receiver/source并轮询全局R×S，固定256 pair不保证最大延迟，Physics缺失/error又默认可见；
- 复核Behavior LOD和debug发布，确认active camera会改变真实AI tick频率，dedicated server退回全部Full，系统在无Editor reader时仍clone全catalog、Blackboard和Perception状态；
- 对照Unreal BehaviorTree/Blackboard/AIPerception/AISense_Sight/EQS/StateTree、Fyrox轻量behavior、Godot navigation agent及Bevy task pool；明确Godot/Bevy/Fyrox轻量设施和Unity Graphics不提供降低工程AI基线的理由。

本轮没有修改production AI/editor/runtime代码或tests，也没有运行动态测试。上一轮`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮不重复无新增证据的同一lane；详细20个P1、5个P2、M0-M8路线及32个验收门见`zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md`。

## 65. Runtime Gameplay Ability / Effect / Attribute / Tag / Cue / Prediction 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| 通用`gameplay_host`及focused tests | 16 / 2,875 / 105,895 | E3逐文件：descriptor、callback、world/entity/component/combat/navigation/transition及14个test attributes |
| host注册与公开导出 | 3 / 668 / 28,710 | E3复核builtin module注册、capability清单、public export与生产grant caller absence |
| runtime asset/scene absence anchors | 3 / 529 / 19,070 | E3完整导出表，确认无Tag/Attribute/Effect/Ability/Cue asset或scene owner |
| runtime plugin assembly absence anchors | 3 / 629 / 23,948 | E3复核RuntimePluginId、first-party runtime catalog与Cargo依赖；8个test attributes、1个无关ignored |
| selected combined scope | 25 / 4,701 / 177,623 | 当前工作树fingerprint `0094f455...1c141`；22个test attributes、1个无关ignored，范围内无在途source |

Runtime Gameplay轮次详细阅读覆盖：

- 搜索production source并逐项核对asset、scene、RuntimePluginId和first-party catalog，确认没有GameplayAbility/Effect/Tag/AbilitySpec/PredictionKey或等价domain、artifact、plugin和world owner；
- 逐读 `zr.zircon.gameplay`，确认它是input/scene/transform/component/spawn/despawn/navigation的通用脚本宿主，不是Ability System；保留versioned descriptor、typed host values、函数级capability、有限JSON错误和generation-backed scene transition基础；
- 追踪damage/heal实现，确认其clone完整`script.bindings` JSON、修改第一个enabled binding的字符串`hp`，归零时直接despawn，heal又信任caller提供的`max_hp`；
- 追踪capability descriptor与grant caller，确认`builtin_host_capabilities()`列出四项gameplay能力但没有证据表明自动赋给所有脚本；准确登记为一个`gameplay.entity`覆盖任意裸u64实体多类危险操作的粒度/authority缺陷；
- 从Workbench公开Ability/Effect/Tags、Server Initiated和predicted activation反查runtime command/provider，确认全部缺失，形成跨层产品真实性P0；
- 对照Unreal AbilitySystemComponent、GameplayAbility、GameplayEffect、GameplayPrediction及GameplayTags manager/container/settings，登记Tag/Attribute/Effect/Ability/Cue、asset/scene/world、authority/prediction/replication的完整重构链；
- 搜索Fyrox、Bevy、Godot与本地Unity Graphics，确认没有同级first-party Ability System参考，不用其缺席降低Unreal级目标。

本轮没有修改production runtime/editor代码或tests，也没有运行动态测试。上一轮`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮不重复无新增证据的同一lane；详细5个P0、30个P1、5个P2、M0-M8路线及32个验收门见`zircon_runtime/08g-gameplay-ability-effect-attribute-tag-cue-prediction-runtime-review.md`。

## 66. Editor AI Behavior Tree / Blackboard / Perception / EQS / Debug Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| AI editor package | 10 / 1,654 / 58,169 | E3逐文件：Cargo、Behavior Tree/Perception ZUI、capability/ID、plugin、mirror、overlay与tests |
| default Editor assembly | 6 / 490 / 17,285 | E3复核first-party editor catalog feature/dependency/registration、App Cargo与产品投影；7个test attributes |
| shared authoring/runtime-event/dispatch | 20 / 5,363 / 196,160 | E2完整inventory，E3复核graph descriptor、operation failure、provider admission与event session/pump链 |
| static AI Workbench | 8 / 3,232 / 140,084 | E3读取两份ZUI、binding/navigation、preview action、固定feedback与bottom panel |
| runtime debug producer/contract | 5 / 865 / 34,509 | E3复核snapshot、tick event转换、producer、manifest与focused test |
| selected combined scope | 49 / 11,604 / 446,207 | 当前工作树fingerprint `33a8b8dc...0bead`；26个test attributes、0 ignored，1个纯import排序在途source |

Editor AI Authoring轮次详细阅读覆盖：

- 保留typed runtime event、play-session/delivery sequence防护、World-qualified mirror、owner-aware extension/provider admission、runtime节点目录与finite-input overlay extract，避免把已有基础误判为零；
- 逐读AI Editor crate并对照current Editor API，确认`overlay.rs`使用已删除的Viewport Tool Mode API、`plugin.rs`通过sibling module导入private consumer常量，源码静态编译不兼容，构成P0；
- 从Project plugin selection追到first-party runtime/editor catalog和App投影，确认runtime有AI分支而Editor没有AI feature/dependency/registration，9个插件测试又全部绕过真实catalog，构成P0；
- 从5个operation descriptor追到共享dispatch，确认Import/Open/Validate/Compile/Toggle均无factory并会返回`MissingFactory`；Graph descriptor/palette也没有产品UI consumer，构成P0；
- 逐读Behavior Tree/Perception ZUI、control ID、mirror与overlay controller，确认两个业务`Space`、0 event、0 overlay provider registration，mirror/controller均无产品consumer，构成P0；
- 从runtime tick report追到BT node event，确认每agent tick至多从单`active_node`合成一条最终状态，不是active path、search/abort/task/service/subtree lifecycle trace；无reader时runtime仍构造全量snapshot；
- 逐读默认Workbench两份AI ZUI、binding/navigation/preview action与feedback，确认`BT_Enemy`、`AI_Guard_01`、视锥/距离/时间及Validate/Simulate/Compile/Diff反馈全部固定，形成第二套假authority，构成P0；
- 对照Unreal BehaviorTreeEditor/Blackboard/AIGraph/Diff/Debugger、EnvironmentQueryEditor/Profiler、GameplayDebugger和StateTree Editor；Fyrox、Godot、Bevy及Unity Graphics本地源码没有同级first-party AI authoring，不作为降低基线的理由。

本轮没有修改production AI/editor/runtime代码或tests，也没有运行动态测试。上一轮`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法到达AI产品行为的相同lane；详细5个P0、60个P1、12个P2、M0-M8路线及32个验收门见`zircon_editor/20-ai-behavior-tree-blackboard-perception-eqs-debug-authoring-review.md`。

## 67. Editor Gameplay Ability / Effect / Attribute / Tag / Cue / Debug Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Gameplay Workbench surfaces | 7 / 1,913 / 117,321 | E3逐项：Ability/Effect/Tags三份ZUI、top toolbar、workspace composition与generated bottom panel |
| route/binding/field edit/feedback/bottom panel | 11 / 3,288 / 137,034 | E3逐分支：所有Gameplay action从event到最终UI mutation；1个test attribute |
| focused Workbench tests | 5 / 2,373 / 85,949 | E3静态阅读：24个test attributes，覆盖模块选择、字段输入、固定反馈与reference projection |
| runtime通用gameplay host | 16 / 2,875 / 105,895 | E3交叉复核Runtime08G：通用host存在但Ability domain不存在；14个test attributes |
| runtime asset与first-party Editor catalog absence anchors | 7 / 768 / 27,493 | E3完整导出/分支表：无Gameplay资产、scene owner或Editor provider；6个test attributes |
| selected combined scope | 46 / 11,217 / 473,692 | 当前工作树fingerprint `8fb4d5c9...74f9c156`；45个test attributes、0 ignored，1个纯import排序在途source |

Editor Gameplay Authoring轮次详细阅读覆盖：

- 逐项统计三份ZUI，确认Ability/Effect/Tags合计771行、90个control、68个event/route；Effect在top toolbar默认checked/selected，是首屏产品信号而非隐藏fixture；
- 从字段Change/Submit追到`module_field_edit`，确认只改retained control的`value/value_text`并refresh，没有document、revision、transaction、dirty、undo、save或validation；
- 从Save/Compile/Diff/Simulate/Effect Apply/Ability Playtest/Tags Add/Rename追到`module_command_feedback`，确认全部直接写固定success/queued/pending字符串；focused tests又把这些文案固化为绿色断言；
- 逐读generated bottom panel及四个handler，确认Effect Attribute Delta/Validation/Compile Log、Ability Compile/Event/Simulation、Tags Reference/Migration/Compile九类view只有静态row、selection、mode和label，没有job/compiler/runtime provider；
- 反查first-party Editor catalog、runtime asset/scene与Runtime08G，确认没有Gameplay Editor plugin、五类asset、Ability System、prediction或replication endpoint；
- 对照Unreal GameplayAbilitiesEditor的factory/graph/schema/audit/Effect details/execution customization，以及GameplayTagsEditor的picker/add/rename/cleanup/settings/reference migration，建立transactional authoring与runtime truth目标；
- 明确Fyrox、Bevy、Godot和本地Unity Graphics没有同级first-party Ability Editor，不用缺失项降低Unreal级产品基线。

本轮没有修改production Editor/runtime代码或tests，也没有运行动态测试。上一轮`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法到达Gameplay domain的相同lane；详细5个P0、60个P1、12个P2、M0-M8路线及32个验收门见`zircon_editor/21-gameplay-ability-effect-attribute-tag-cue-debug-authoring-review.md`。

## 68. Editor Render Pipeline / Frame Capture / Lighting Bake / Reflection Probe / Post Process / Debug Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Rendering根Editor、15个feature Editor与plugin manifest | 67 / 1,676 / 54,434 | E3逐文件：16个descriptor、manifest、Reflection Probe capture helper及3个test attributes |
| Render/Lighting Bake/Post Process/generated bottom surfaces | 4 / 1,223 / 78,047 | E3逐control：pipeline/frame/resource、bake/UV/probe与profile/volume全部可见值和route |
| route/binding/field edit/feedback/bottom panel | 19 / 6,515 / 285,195 | E3逐分支：所有Rendering action到最终UI mutation；2个test attributes |
| focused Workbench tests | 5 / 2,373 / 85,949 | E3静态阅读：24个test attributes，覆盖module/extension导航、字段输入、固定feedback与projection |
| first-party Editor catalog | 4 / 239 / 8,423 | E3完整分支：默认只装配Navigation和Neural，无Rendering；6个test attributes |
| selected combined scope | 99 / 12,026 / 512,048 | 当前工作树fingerprint `2db19ed8...aa09fc0`；35个test attributes、0 ignored，2个纯import排序在途source |

Editor Rendering Authoring轮次详细阅读覆盖：

- 保留Runtime现有Render Graph materialization/execution/resource/alias/coverage/profile/stage report、frame/GPU timing、viewport capture和Reflection Probe capture/register helper，明确缺失的是Editor产品桥而非运行时基础为零；
- 逐读`rendering/plugin.toml`、根Editor和15个feature Editor package，确认manifest发布`stable`/`complete`与16个Editor module，但源码只有16处`with_capability`，其余asset/surface/operation/menu/graph/viewport/authoring contribution均为0；
- 从Project selection追到first-party runtime/editor catalog，确认runtime装配Rendering而Editor只装配Navigation/Neural，manifest承诺的Editor模块不进入默认产品；
- 逐control核对Render Workbench，确认Frame 1234、MainPipeline.rp、SceneColor/BloomInput 1.84 ms、R11G11B10_FLOAT Read和Windows DX12/GPU 6.24 ms均为静态文本，Compile/Save/Preview只写固定feedback；
- 逐control核对Lighting Bake与Post Process extension，确认87 assets/4 warnings、12 volumes、6 texels、02:30，以及Global Stack/Cinematic/Bloom 0.65/Filmic +0.4/LUT/EV warning均为静态样例，Bake/Apply没有job、document、artifact或runtime ack；
- 逐读Reflection Probe Editor helper及tests，确认request/placement序列化和runtime capture/register调用真实存在，但全仓产品caller、operation factory、catalog owner、job与selection transaction均缺失；
- 对照Unreal RenderDoc backend capture与GPU Lightmass world subsystem/capability流程、Godot LightmapGI真实bake/error/progress/cancel，以及Unity Rendering Debugger/VolumeProfile serialized authoring，建立generation、artifact与failure-first产品门。

本轮没有修改production Editor/runtime代码或tests，也没有运行动态测试。上一轮`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法到达Rendering产品行为的相同lane；详细5个P0、60个P1、12个P2、M0-M8路线及32个验收门见`zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md`。

## 69. Editor UI Asset / HUD / Widget / Binding / Theme / Icon / Accessibility / Menu Flow / Font Atlas Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| UI Asset核心model/session/editor算法 | 95 / 25,399 / 889,169 | E3逐文件：binding、preview、source、tree、style、theme、undo与V2 projection；27个test attributes |
| Host/session/product wiring | 141 / 10,887 / 399,663 | E3逐分支：open/edit/save/watcher/import hydration/retained actions/detail events/pane projection；35个test attributes |
| UI插件、asset registry与first-party catalog | 22 / 1,777 / 63,945 | E3完整分支：manifest、descriptor、资源URI、create operation、catalog与tests；22个test attributes |
| 真实Editor与七份Workbench UI surfaces | 10 / 2,681 / 149,798 | E3逐control：真实shell/action bar、HUD和UI Asset/Binding/Icon/A11y/Menu/Font extensions |
| Workbench route、feedback与binding | 12 / 4,814 / 210,306 | E3逐分支：UI diagnostics navigation、field mutation与固定feedback；2个test attributes |
| Runtime V2/a11y/icon/font anchors | 19 / 4,884 / 167,498 | E2/E3 focused handoff：验证Editor投影与11A/11B/11C合同，不重复runtime owner |
| focused Editor tests | 50 / 20,270 / 698,888 | E3静态阅读：300个test attributes，覆盖editing/host/retained/UI/registry/Workbench |
| selected combined scope | 349 / 70,712 / 2,579,267 | 当前工作树fingerprint `9e2c26ec...94f0c126`；386个test attributes、0 ignored，40个在途文件 |

Editor UI Authoring轮次详细阅读覆盖：

- 保留真实UI Asset Editor的typed session、V1/V2 real-surface preview、slot-aware palette/tree、component/reference导航、style cascade/theme refactor、binding CRUD、source outline、watcher/conflict/autosave/recovery，明确问题不是整套Editor均为占位；
- 追踪V2到legacy再回写的完整链，确认`repeat: None`、node `slots`清空、不可达node丢失、`ThemeTokens -> Style`和全量pretty-print共同构成一次视觉编辑即可触发的语义损坏；
- 从Ctrl+S/Keep Local一路追到`save_to_canonical_source`、`mark_saved`、`fs::write`与被吞掉的`import_asset`结果，确认写失败会假clean，普通Save也不检查external conflict/base revision；
- 逐读widget/theme promote、undo/redo stack和external effect执行，确认session/stack先推进，跨文件write/remove/import后执行且无统一rollback，构成可复现的部分提交风险；
- 核对`ui_asset_authoring`所有descriptor URI与create operation，确认四份ZUI资源物理缺失、三个operation无factory、默认first-party Editor catalog不装配该插件；
- 逐项审查Designer/Preview/Inspector/Binding，确认缺delete/clipboard/zoom/guide/snap等基本工作流，locale不替换真实文字，Preview Interact只生成binding DTO且无产品caller，binding suggestion由`save`字符串启发式驱动；
- 逐control核对HUD及六个UI extension，确认WBP_Inventory/42 widgets、Health.Value、icon-warning/312、Gameplay_HUD/9 issues、Screen_Start/64、Inter UI/4096 glyphs等均为fixture，field/action只修改control或固定feedback；
- 对照Unreal UMG Factory/Compiler/Designer/Hierarchy/Palette/Preview/Navigation、Godot Control/Theme Editor和Fyrox UI scene/clipboard/command/interaction，建立lossless document、transaction、真实preview和authoring产品门；本地Unity Graphics不含完整UI Builder/TextCore源码，未据此猜测闭源行为。

本轮没有修改production Editor/runtime代码或tests，也没有运行动态测试。上一轮`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法到达UI产品行为的相同lane；详细5个P0、60个P1、12个P2、M0-M9路线及32个验收门见`zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md`。

## 70. Editor Data Table / Structured Data / Schema / Import / Validation / SaveGame / Slot / Migration / Platform / Cloud Storage Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Editor Workbench、registry与routes | 21 / 6,205 / 295,470 | E3逐分支：两份ZUI、入口、field mutation、feedback/navigation/template binding、asset registry/reference analysis；7个test attributes |
| Data importer、model与artifact | 18 / 4,239 / 158,621 | E3逐文件：builtin/optional importer、DataAsset、cache payload、artifact store与load facade；14个test attributes |
| Save/archive/platform anchors | 13 / 3,212 / 116,395 | E2/E3 focused handoff：World snapshot、level、archive artifact/writer/slot/manifest及preference atomic file；10个test attributes |
| focused tests | 5 / 862 / 30,219 | E3静态阅读：21个test attributes，覆盖builtin Data、artifact、session archive/single-slot与preference quota/failure |
| selected combined scope | 57 / 14,518 / 600,705 | 当前工作树fingerprint `a83a2e18...17e53d6`；52个test attributes、0 ignored，3个在途文件 |

Editor Data/Save Authoring轮次详细阅读覆盖：

- 逐control与route核对Data Table Workspace，确认`DT_Items`、`Schema_Item`、Potion/Sword/Armor/Debug Item、128 rows/2 warnings/512 refs和version 12均为fixture，field/action只改control或固定feedback；
- 逐control与route核对Save Data Workspace，确认AutoSave_01/Manual_03/Cloud_02、PlayerState/Inventory/QuestLog/DebugSlot、SaveData v4、LZ4及Cloud queued均无storage/runtime authority；
- 追踪`ResourceKind::Data`到builtin registry与reference analysis，确认只有placeholder thumbnail、无factory/toolkit，且`ImportedAsset::Data`明确产出空references；
- 逐读`DataAsset`、builtin importer和optional data plugin，确认只有Text/TOML/JSON/YAML/XML原文与canonical JSON，无row schema/typed runtime consumer，且TOML/JSON存在双importer authority；
- 追踪source load、`source_text`、parser DOM/value和artifact payload，确认没有source/depth/node/alias/CPU/allocation预算；XML递归投影丢mixed-content顺序、namespace、comment/PI并有深树栈风险；
- 复核Runtime05的约565文件DynamicScene archive证据，确认artifact/writer/slot/retention与atomic path是真实底座，但无产品caller，`World::clone`/serde硬编码builtin component map会遗漏plugin/typed component，不能直接改名SaveGame；
- 复核platform preference atomic file，确认stage/commit/fsync及typed capacity/permission错误可复用，但其key-value/backend语义不等于player/profile/slot/platform/cloud服务；
- 对照Unreal DataTable factory/editor/runtime typed row合同与SaveGame platform/async slot API，以及Godot resource/user I/O、Fyrox Visitor和Bevy scene/reflect基础，明确serialization foundation与DataTable/SaveGame product的层级差异；本地Unity Graphics无同级源码，未推测闭源行为。

本轮没有修改production Editor/runtime/plugin代码或tests，也没有运行动态测试。上一轮`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达DataTable/SaveGame产品行为的相同lane；详细5个P0、60个P1、12个P2、M0-M9路线及32个验收门见`zircon_editor/24-data-table-structured-data-schema-import-validation-save-game-slot-migration-platform-cloud-storage-authoring-review.md`。

## 71. Editor Runtime Diagnostics / Performance Timeline / Console / Telemetry / Observability Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Workbench diagnostics routes | 15 / 4,400 / 229,944 | E3逐control/route：Console Diagnostics、Runtime Diagnostics、Performance与Telemetry四张静态产品；0个test attributes |
| Editor真实产品diagnostics | 27 / 3,764 / 133,596 | E3逐文件：builtin pane、gateway、Console journal、UI reflector、presentation与profile command；21个test attributes |
| Runtime diagnostics/profile/interface | 62 / 14,242 / 472,878 | E2/E3纵向：collector/store、33个render stats叶文件、profile recorder/export、dynamic ABI/FFI与world watch；82个test attributes |
| `runtime_diagnostics` plugin | 9 / 436 / 16,128 | E3逐文件：manifest、registration、descriptor、surface与resources；4个test attributes |
| focused tests | 10 / 2,376 / 91,009 | E3静态阅读：32个test attributes，覆盖snapshot ABI、recorder retention/export、builtin pane与profile projection |
| selected combined scope | 123 / 25,218 / 943,555 | 当前工作树fingerprint `856cab60...e6a5bb`；139个test attributes、0 ignored，8个在途文件 |

Editor Observability轮次详细阅读覆盖：

- 逐control与route核对四张Workbench，确认固定session/frame/actor/event、Profiler_Editor、DAU/MAU/Crash Rate、Export/Record/Capture成功反馈均无产品authority；
- 追踪builtin Runtime Diagnostics到collector与presentation，确认只观察Editor宿主Core/render/physics/animation和UI reflector，既不消费动态子Runtime完整diagnostics response，也不展示本地metric series；
- 逐字段复核Editor与子Runtime profile recorder、merge、control和export，确认独立`Instant`时钟不可比较，数组拼接不是时间线，双source控制没有transaction/timeout/compensation且默认输出可能同名覆盖；
- 逐读33个render stats叶文件并统计453个`record_count`、23个`record_bytes`、10个`record_microseconds`、133个`record_bool`与648类`render.*`路径，确认可见pane在presentation重算中同步采集、深复制与FFI，且store总series/path/tag cardinality无界；
- 精确搜索Telemetry production依赖、provider、schema、ingest、query、consent、redaction、auth、tenant与retention，确认仓内只有静态Dashboard和示例protocol，不存在可宣称运营数据的产品链；
- 追踪`runtime_diagnostics`插件manifest与registration，确认其引用的`plugins://runtime_diagnostics/editor/authoring.zui`不存在，并与builtin重复占有`editor.runtime_diagnostics`；
- 对照Unreal TraceServices/Insights analysis session与timing tracks、Godot debugger session/performance profiler、Bevy Diagnostic注册/历史平滑与Remote JSON-RPC、Fyrox stats window，以及Unity Graphics DebugManager/FrameTiming，建立source-qualified session、clock calibration、registered metrics、analysis provider和bounded capture门。

本轮没有修改production Editor/runtime/plugin代码或tests，也没有运行动态测试。上一轮`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达observability产品行为的相同lane；详细6个P0、60个P1、12个P2、M0-M9路线及32个验收门见`zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md`。

## 72. Editor Multiplayer Lobby / Matchmaking / Online Services / Replication / Network Emulation / PIE Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Workbench Lobby/Matchmaking、route、binding、field与feedback | 14 / 3,886 / 211,030 | E3逐control/route：两份ZUI、40个binding及最终selection/string/fixed feedback mutation；1个test attribute |
| Net Editor、first-party catalog与App装配 | 15 / 1,896 / 74,553 | E3逐文件：2 view、6 operation、3 inspector、asset/graph palette、5个缺失资源与默认catalog断点；1个test attribute |
| Runtime session、RPC与Replication anchors | 46 / 3,799 / 125,326 | E2/E3 owner handoff：typed合同、manager/factory、schedule/apply与World/Reflection/transport断点 |
| Play topology bridge | 10 / 2,011 / 68,765 | E3逐分支：request、controller、单child backend、process args与生命周期；23个test attributes |
| focused tests | 28 / 4,863 / 172,863 | E3静态阅读：93个test attributes，覆盖Workbench/catalog/descriptor、RPC/Replication局部manager和单backend Play |
| selected combined scope | 113 / 16,455 / 652,537 | 当前工作树fingerprint `55e857ec...cb498d`；118个test attributes、0 ignored，3个在途文件 |

Editor Multiplayer Authoring轮次详细阅读覆盖：

- 逐control、event、route与feedback核对Lobby/Matchmaking两份Workbench，确认`Lobby_Default`、8 slots/4 players/crossplay warning、Ranked/6 queues/128 players/42-62 ms均为fixture，field只改control字符串，Simulate/Validate只写固定结果；
- 精确搜索Online Service、Identity、Party、Lobby、Matchmaking Ticket、Allocation、Backfill、Playlist、Crossplay及EOS/Steam/PlayFab/GDK/PSN等production domain/dependency，确认仓内没有在线产品owner，现有NetManager仅覆盖低层transport/service；
- 追踪Net Editor到first-party catalog、App Cargo、resource resolver和operation invocation，确认Runtime Net可被选中而Editor Net不装配，5个ZUI/TOML资源物理缺失、6个operation无factory并会返回`MissingFactory`；
- 追踪Replication Schema descriptor到Runtime manager，确认没有canonical source/parser/compiler/artifact/install bridge，runtime dense index仅进程内排序、descriptor使用String/raw bytes，manager未连接World/Reflection/transport；
- 逐读RPC与Replication focused tests，确认它们对direction/schema/quota、delta/interest/budget/schedule/late join有局部价值，但均直接构造孤立内存manager，不能证明Editor资产到多人游戏的闭环；
- 追踪Editor07 Play request/controller/backend/CLI，确认只有单child且不携server/client role、client count、port/account/join plan/network profile，当前Simulate不会启动server + N clients或应用per-link网络仿真；
- 对照Unreal Online Session生命周期、Play多人拓扑/网络仿真、Replication Graph/Iris/Network Prediction Insights，以及Godot Replication Editor/Network Profiler/SceneMultiplayer，建立provider、document、artifact、test session与observation五层owner；本地Bevy/Fyrox无同级first-party在线服务，Unity Graphics也不含可审查Netcode源码，未用缺失参考降低标准。

本轮没有修改production Editor/runtime/plugin代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达Multiplayer产品行为的相同lane；详细5个P0、60个P1、12个P2、M0-M9路线及32个验收门见`zircon_editor/26-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-review.md`。

## 73. Editor Project Operations / Source Control / Changelist / Diff / Automation Report / Submit Gates / Health Dashboard 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| 五张Production Workbench、route、binding、field与feedback | 14 / 5,140 / 262,920 | E3逐control/route：五张ZUI、100个binding和最终selection/string/fixed feedback mutation；1个test attribute |
| 真实Project Overview与projection closure | 9 / 3,071 / 114,394 | E3逐文件：真实project/catalog字段、pane projection与focused tests；10个test attributes |
| authoring commandlet、App composition与retained-host automation | 14 / 6,041 / 216,796 | E3逐分支：typed request/path、single project authority、callback/journal、Save/Undo/Redo、report与failure；66个test attributes |
| manifest与跨owner handoff anchors | 26 / 5,011 / 181,730 | E2/E3：Editor02/04/06/08-11与Tooling03/09/10边界；31个test attributes、1 ignored |
| selected combined scope | 63 / 19,263 / 775,840 | 初始61文件fingerprint `e22fb06d...2e83670`，另含2个逐hash retained-host文件；108个test attributes、1 ignored，7个在途文件 |

Editor Project Operations轮次详细阅读覆盖：

- 逐control、route、binding与feedback核对Source Control，确认`CL_2048`、18 files、2 conflicts、6 checks、Alice/Bob/Chen和Validate/Submit queued均为fixture，且generic field只改control字符串；
- 精确扫描Cargo manifest与tracked production source，确认没有Git/libgit2/gix、Perforce/P4或SVN provider，也没有repository/workspace/revision/file-state/diff/changelist operation/receipt合同；Project Manifest也不保存repository或submit policy；
- 逐control核对Automation Report，确认642 tests、7 failed、3 flakes、Worker_03/09/11与Screenshot diff不消费真实结果，Validate/Publish只写固定反馈；
- 追踪真实`authoring-automation`到App composition、retained-host callback、transaction、Save/Undo/Redo、journal和scene snapshot，确认它是应保留的窄域产品自动化基础，但缺TestPlan/Attempt/worker/deadline/artifact等全局控制面合同；
- 对照Tooling10，明确Automation UI只消费`TestPlanManifest`、`TestAttemptReceipt`、`TestCaseResult`、`TestArtifactManifest`和`ValidationSet`，不另建第二套schema；
- 对比内置与extension Project Overview，确认前者真实投影project/catalog元数据，后者固定NebulaGame/M3/Healthy/72%/jobs并允许直接编辑Health，形成双authority；
- 复核Build Export与Plugin Manager extension，确认同一preview机制固定62% cook/CDN和18 installed/3 updates，真实owner分别属于Tooling03与Editor06；
- 对照Unreal SourceControl provider/state/operation/changelist、Automation Controller report/result/artifact和Godot typed VCS/diff/stage/commit，建立provider-neutral VCS、带provenance/freshness的`ProjectOperationsSnapshot`、source-bound `SubmissionCandidate`与可恢复Submit/Publish事务。

本轮没有修改production Editor/runtime/app/tooling代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达Project Operations产品行为的相同lane；详细5个P0、60个P1、12个P2、M0-M9路线及32个验收门见`zircon_editor/27-project-operations-source-control-changelist-diff-automation-report-submit-gates-health-dashboard-review.md`。

## 74. Editor Spawn Rules / Encounter / Population / World State / Scenario / Quest Flag / Authority / Simulation Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Spawn Rules / World State Workbench闭环 | 8 / 2,544 / 139,493 | E3逐control/route/binding/field/feedback：两份ZUI、40个binding、6对field route与fixed simulation/validation；2个test attributes |
| ECS / DynamicScene结构与生成底座 | 44 / 7,584 / 293,070 | E2/E3纵向：stable identity、commands、transaction、preflight、async task、asset reload与remap；34个test attributes、1 ignored |
| script gameplay host | 5 / 1,380 / 53,031 | E3逐函数：`gameplay.entity` capability、裸entity Int、spawn/model/despawn与dynamic component写入；10个test attributes |
| focused tests | 4 / 2,164 / 91,381 | E3静态阅读：identity、spawn transaction、asset reload与script spawn；60个test attributes、1 ignored |
| selected combined scope | 60 / 12,995 / 546,933 | 当前工作树fingerprint `07edfdf2...6b08375`；85个test attributes、1 ignored，4个在途文件 |

Editor Gameplay Spawn/World State Authoring轮次详细阅读覆盖：

- 逐control、event、route、binding和feedback核对Spawn Rules Workspace，确认`SpawnRules_Enemy`、`Zone_A`、`Condition_Night`、18 rules/12 zones/1 conflict、Seed 2026、Server/Client Preview/Offline及96 spawns均为fixture，field只改control字符串；
- 同样核对World State Workspace，确认`Scenario_NightRaid`、`Alarm.Active`、Weather/AI/Quest、84 keys/6 layers/1 conflict、Authority Server及42 events没有schema/store/scenario/runtime authority；
- 对production source执行精确域搜索，确认`SpawnRule`/`WorldStateKey`只在Editor route/binding出现，`SpawnPoint`、`SpawnVolume`、`SpawnTable`、`SpawnSet`、`Respawn`、`DespawnPolicy`、`ScenarioAsset`和`ScenarioState`均无生产实现；
- 逐读ECS/World/DynamicScene identity、commands、transaction、preflight、spawn task、asset reload及tests，确认stable entity和原子结构变更是真实底座，但`spawn_into`只返回`EntityRemap`，没有instance/source/owner/authority/lifecycle/whole-instance receipt；
- 逐读script gameplay host descriptor与lifecycle实现，确认一个`gameplay.entity` capability覆盖裸实体ID的transform/component/combat/spawn/despawn，`spawn_empty`/`spawn_model`直接写active level而`despawn`直接remove；
- 对照Unreal FActorSpawnParameters/deferred spawn/MassSpawner/GameState/DataLayer、Godot MultiplayerSpawner/PackedScene/Node，以及Bevy Commands/SceneSpawner/typed State transition，建立definition/artifact/instance与typed state transaction分层；本地Fyrox和Unity Graphics无同级Gameplay产品源码，未据此降低目标或推测闭源行为；
- 明确Runtime05保留ECS/DynamicScene底层，Runtime08G拥有script authority，Runtime08E/Editor26拥有network，Editor24拥有save authoring，Editor16拥有partition；Editor28只拥有transactional authoring、diagnostics和simulation UX。

本轮没有修改production Editor/runtime/plugin代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达Spawn/World State产品行为的相同lane；详细5个P0、60个P1、12个P2、M0-M9路线及32个验收门见`zircon_editor/28-spawn-rules-encounter-population-world-state-scenario-quest-flag-authority-simulation-authoring-review.md`。

## 75. Editor Input Action / Mapping Context / Binding / Trigger / Modifier / Device / User / Rebinding / Accessibility Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Action/context/binding公共合同 | 11 / 987 / 27,824 | E3逐字段：action/map/state/manager、button/gamepad/frame、module config/descriptor；0 tests |
| evaluator/runtime安装底座 | 6 / 1,079 / 33,144 | E3逐分支：compiled generation、workspace、consumed/axis index与manager；1 test、0 ignored |
| Editor authoring与keymap复用锚点 | 13 / 2,031 / 67,393 | E2/E3：ResourceKind、asset registry、first-party catalog、Editor keymap/settings；16 tests |
| product consumers与focused tests | 7 / 1,980 / 77,103 | E3：script raw key、action mapping/axis/manager/boundary tests；37 tests |
| selected combined scope | 37 / 6,077 / 205,464 | 当前工作树fingerprint `b95b135f...e6d172b9`；54个test attributes、0 ignored、0个在途文件 |

Editor Input Action Authoring轮次详细阅读覆盖：

- 逐字段核对`InputAction`、Context、Binding、Map和State，确认它们只有string identity、button chord与单`f32`axis，没有stable ID、value type、trigger/modifier/composite、schema version或diagnostics；
- 逐分支阅读generation/workspace/evaluator，确认compiled binding range、context/axis/consumed index和10K规模是真实可保留基础，同时发现missing context自动enabled、unknown-action binding无消费者和duplicate helper静默first-wins；
- 追踪`InputConfig`到module descriptor及production callers，确认默认disabled空map是唯一production构造，手工configured descriptor只在Input模块与测试出现，没有project/asset/cook/runtime install链；
- 追踪`GamepadId`从gilrs连接槽到`InputButton::Gamepad`/`InputAxisBinding` serde，确认authored-shaped binding保存临时runtime instance；全仓没有InputUser/LocalPlayer device assignment、profile或rebind receipt；
- 追踪script Gameplay到`gameplay.key_pressed`，确认其resolve raw InputManager snapshot并绕过Action Manager、context、consume、profile、device/user和accessibility；
- 核对Runtime interface 26类ResourceKind、Editor builtin registry和first-party catalog，确认无Input Action/Mapping Context asset、factory、toolkit、surface或provider；
- 复核Editor08 command keymap，确认typed override/settings/conflict/signature index可复用，但其operation/scope/storage不等于shipping Gameplay map；
- 对照Unreal Enhanced Input Action/Context/subsystem/user mapping/Input Editor、Godot InputMap与event configuration dialog，以及Bevy raw input基础，建立source/artifact/per-user runtime/profile/rebind分层；本地Fyrox与Unity Graphics无同级命中，未推测外部行为。

本轮没有修改production Editor/runtime/interface/plugin代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达Input authoring产品行为的相同lane；详细5个P0、60个P1、12个P2、M0-M9路线及32个验收门见`zircon_editor/29-input-action-mapping-context-binding-trigger-modifier-device-user-rebinding-accessibility-authoring-review.md`。

## 76. Editor Camera Asset / Component / Rig / Controller / Director / Blend / Shake / Cinematic Cut / Preview Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Scene asset/component/world/project IO | 13 / 3,797 / 154,905 | E3逐字段与转换：source、component、reflection、active camera、render extract和focused tests；19个test attributes |
| Render view/stack/order/history | 10 / 2,774 / 93,763 | E3逐分支：descriptor、Base/Overlay校验、history key和velocity cut heuristic；25个test attributes，3个在途文件 |
| controller/dynamic runtime/script/AI | 24 / 3,142 / 116,192 | E3逐输入与写入：free/orbit/pan、动态session事件、script follow和AI LOD；11个test attributes，2个在途文件 |
| Editor viewport/Sequencer/catalog anchors | 20 / 4,188 / 141,021 | E2/E3：transient editor camera、create command、static sequence rows和ResourceKind；12个test attributes，1个在途文件 |
| selected combined scope | 67 / 13,901 / 505,881 | 当前工作树fingerprint `3c91d2ae...1350fd00`；67个test attributes、0 ignored、6个在途文件 |

Editor Camera Authoring轮次详细阅读覆盖：

- 逐字段核对`SceneCameraAsset`与`CameraComponent`，确认前者已有15个真实source字段，而后者14字段中11个被reflection skip，Editor实际只能创作FOV、near和far；
- 追踪Scene source/component到`CameraRenderDescriptor`与stack validator，确认render层已有Base/Overlay、stack、clear-depth和独立masks合同，但source层没有对应字段，extraction固定Base/empty stack/default clear-depth并混用RenderLayer；
- 逐文件阅读free/orbit/pan controller，确认typed input/settings/state/output和局部数学可保留，但Orbit只有Editor/dynamic消费者，Free/Pan没有shipping gameplay owner；
- 追踪dynamic session construction/events，确认所有session无条件构造camera controller，UI未消费的right/middle/wheel既提交通用InputEvent，又直接orbit/pan/zoom active camera transform；
- 追踪world active camera、project IO、script `camera_follow`和AI Behavior LOD，确认未持久化global singleton同时承担render默认、玩法写入和AI observer语义，且invalid set静默忽略；
- 精确搜索Rig/Director/Blend/Shake/SpringArm/Cine/Mode和camera preview/pilot/frustum/safe-frame，确认production产品均缺失，`CameraRig`只存在于测试/展示fixture字符串；
- 追踪Sequencer Camera/Camera Cut rows和Temporal velocity，确认前者只有静态control/route，后者以far-plane 20%位移、60度旋转、15度FOV等阈值猜`CameraCutOrInvalid`，没有typed cut/history epoch；
- 对照Unreal Camera/PlayerCameraManager/SpringArm/CineCamera/MovieScene/GameplayCameras、Godot Camera3D preview、Bevy/Fyrox camera endpoint和Unity Graphics per-camera stack/history，建立endpoint/source/compiler/director/view/editor分层，并明确不推测未收录的Unity Cinemachine。

本轮没有修改production Editor/runtime/interface/plugin代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达Camera产品行为的相同lane；详细5个P0、60个P1、12个P2、M0-M9路线及32个验收门见`zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md`。

## 77. Editor Script Source / Code Editor / Build / Compiler / Hot Reload / Debugger / Visual Script / Class / Component Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Editor orchestrator/log/command/catalog | 11 / 2,656 / 85,877 | E3：5个script-build文件、diagnostic projection、typed jump最终动作、唯一Console命令和Editor catalog；31个test attributes |
| Runtime interface | 2 / 291 / 7,643 | E3：`ScriptDiagnostic`与完整`ResourceKind`逐字段；1个test attribute |
| Runtime project/scene/package/reload | 14 / 4,627 / 174,354 | E2/E3：startup load、scene binding、bounded discovery、slot manager、state migration和hot-reload rollback；27个test attributes，1个在途文件 |
| App/ZrVM provider/catalog | 10 / 991 / 38,093 | E3：feature graph、provider catalog、plugin capability、real backend compile/session与instance state；1个在途文件 |
| WOC product evidence | 5 / 1,276 / 78,914 | E2/E3：project/package/project manifest、entry module与README truthfulness边界 |
| selected combined scope | 42 / 9,841 / 384,881 | 当前工作树fingerprint `dda20ad4...a786398c`；59个test attributes、0 ignored、2个在途文件 |

Editor Script/Visual Script Authoring轮次详细阅读覆盖：

- 逐文件核对`ScriptBuildOrchestrator`、request与diagnostics sink，确认debounce、first-event cap、path count/byte预算和single pending基础真实存在，但generation直接等于request ID、失败会删除更新source事实，且除自身/测试外无production caller；
- 追踪command与jump最终动作，确认唯一可见命令只是Script Build Console filter，`ScriptLocation`被降为通用`OpenAsset(path)`，line/column只写status line而不定位document；
- 追踪App feature与first-party catalogs，确认`woc`/`vampire` required ZrVM selection没有进入默认Client/Editor profile，Editor catalog也没有Script/ZrVM provider；
- 追踪Runtime startup、package discovery、real backend和Rust binding，确认startup同步discover/wait，load时直接compile+start session，真实compiler只返回计数或字符串错误，没有生产`ScriptDiagnostic`；
- 逐分支阅读VM manager与HotReloadCoordinator，确认slot generation、state snapshot/schema migration、reflection prepare/commit和失败rollback可保留，但`hot_reload_discovered_slot()`没有非测试production caller；
- 逐字段核对Scene binding/project IO/runtime projection，确认`package/module + JSON map`整体编码进动态`script.bindings`，没有Class/Component/Field stable schema、typed overrides或迁移；
- 统计WOC package为817个`.zr`、246,765物理行、354个`.zrp`和37个已跟踪局部`.zro`，并依据README保留“partial/inventory/not playable”边界，不把规模或golden当产品完成度；
- 对照Unreal Blueprint/Compilation Manager/Live Coding、Godot ScriptEditor/ScriptLanguage/debugger和Fyrox Script Inspector/build queue；同时核对外部ZrVM当前LSP/debug实现，建立复用语言服务而非重写简化parser的路线；Bevy无同级authoring产品、Unity Graphics不含脚本权威源码，未据此降低目标或推测外部行为。

本轮没有修改production Editor/runtime/interface/plugin/App代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达Script产品行为的相同lane；详细5个P0、60个P1、12个P2、M0-M9路线及32个验收门见`zircon_editor/31-script-source-code-editor-build-compiler-hot-reload-debugger-visual-script-class-component-authoring-review.md`。兄弟仓库选取的6个ZrVM adapter文件另有2个在途文件，实施前必须冻结revision/API并重算双方fingerprint。

## 78. Editor Model / Mesh / Skeleton / Geometry Import / LOD / Collision / Retarget / Preview Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Editor catalog/import/preview/workspaces | 18 / 3,154 / 173,163 | E3逐control/callback/route：Quick Import、toolkit/thumbnail、animation derivation、Retarget与Collision Proxy；2个test attributes |
| Runtime Model/Mesh/import | 26 / 6,066 / 207,724 | E3逐字段/分支：Model/Mesh schema、validation、glTF/OBJ、meshopt、SDF/VG request与registry selection；11个test attributes，4个在途文件 |
| Runtime animation/scene/render/physics handoff | 6 / 1,525 / 56,253 | E3：Skeleton、PhysicsMesh、Scene/World LOD与skinning palette完整路径 |
| Plugin/product assembly | 14 / 3,207 / 111,088 | E3：split glTF/OBJ、STL/PLY/DXF、diagnostic-only formats、VG Editor与feature catalog；7个test attributes，1个在途文件 |
| Focused tests | 8 / 2,739 / 90,313 | E3静态阅读：glTF labels/channels、Mesh validation、OBJ、Scene mesh binding、plugin import与registration；38个test attributes |
| selected combined scope | 72 / 16,691 / 638,541 | 当前工作树fingerprint `54a74f35...a91bfdb`；58个test attributes、0 ignored、5个在途文件 |

Editor Geometry Authoring轮次详细阅读覆盖：

- 逐control/callback追踪Asset Browser Quick Import，确认path TextField没有event、callback没有production invoke caller、默认空值使按钮无法获得标准输入；即便外部注入路径，也只允许OBJ/glTF/GLB并把import与当前Scene插入耦合；
- 追踪Core/split glTF descriptor、registry排序和product features，确认Core schema 2能力更完整，而schema 1 plugin以priority 120在compiled/enabled/available时覆盖核心10；默认Client/Editor又未编译base catalog，形成profile-dependent语义而非单一authority；
- 逐字段核对Model/Mesh与overview，确认core glTF把root primitive变成空inline+Mesh reference，但overview/descriptor/resource management不resolve引用，可产生0顶点/空bounds伪事实；
- 追踪Skin/Skeleton/imported Scene到renderer palette，确认Mesh只保存一个IBM vector、同Mesh多Skinfirst-wins、Skeleton无stable bone ID/结构验证、Scene不安装Skeleton/player，且生产skinning完全不消费imported IBM；
- 核对format family，确认STL/PLY/DXF有真实解析器，FBX/DAE/3DS/USD为明确DiagnosticOnly；Quick Import却不接前三者，OBJ两套实现也未形成MTL/Material完整链；
- 追踪Scene LOD、PhysicsMesh、Mesh SDF/VG和workspaces，确认LOD只读并按entity origin距离选择，无reduction/crossfade；无Render Mesh到PhysicsMesh cooker，Collision Proxy与Retarget均为fixed feedback；
- 核对AssetType registry/thumbnail/toolkit和Virtual Geometry Editor包，确认Model/Mesh/Skeleton只有placeholder、无toolkit，而VG contribution引用实际不存在的`authoring.zui`；
- 对照Unreal Static/Skeletal Mesh Editor、Interchange与IK Retargeter，Godot ImporterMesh/Mesh/Skeleton/BoneMap/Retarget，Fyrox FBX/glTF，Bevy typed runtime data，以及Unity Graphics GPU LOD/crossfade，建立source/recipe/normalized asset/derived artifact/toolkit分层并限制各参考的适用范围。

本轮没有修改production Editor/runtime/plugin/App代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达Geometry产品行为的相同lane；详细5个P0、60个P1、12个P2、M0-M9路线及32个验收门见`zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md`。5个在途文件均非本轮产生，实施前必须重算72文件fingerprint并复核provider选择与Model/Mesh payload终态。

## 79. Editor Localization / String Table / Culture / Translation Import-Export / Fallback / Pseudo-localization / Preview Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Editor shell i18n | 18 / 4,345 / 157,604 | E3逐字段/分支：bundle validation、locale normalization、settings同步、event backpressure、notification/settings presentation；30个test attributes |
| Runtime UI localization | 15 / 1,504 / 52,378 | E3逐值路径：reference/report/collector/catalog、component validation、package manifest与renderer string resolution |
| UI Asset Editor locale/report/preview | 10 / 1,351 / 48,773 | E3逐action/call chain：固定选项、diagnostic-only resolver、preview compile与host projection；4个test attributes，2个在途文件 |
| Runtime text/culture | 7 / 1,425 / 48,448 | E3逐字段：system locale、font culture matching、Cosmic font-system locale cache与render language；4个test attributes，1个在途文件 |
| Asset/resource anchors | 4 / 976 / 34,785 | E2/E3：ResourceKind、builtin registry、template document与resource/cook接缝；2个test attributes |
| Focused tests | 4 / 1,301 / 39,670 | E3静态阅读：localized ref collection/package、UI report/locale action与render-shaped fixtures；30个test attributes，2个在途文件 |
| selected combined scope | 58 / 10,902 / 381,658 | 当前工作树fingerprint `e7b2942f...3e21871`；70个test attributes、0 ignored、5个在途文件 |

Editor Localization Authoring轮次详细阅读覆盖：

- 逐字段/分支核对Editor catalog、locale与service，确认English/zh-CN各54 key完全对齐、English fallback、settings热切换、bounded event/resync和notification locale snapshot是真实可保留基础，同时确认parser只是近似BCP 47且format仍是循环replace；
- 追踪production consumers与252份production ZUI，确认Settings/notifications/Play decision使用i18n，但ZUI中production `text_key`为0、252个`display_name`均为literal，locale message没有Retained Host production subscriber形成完整UI重建；
- 逐值路径阅读`UiLocalizedTextRef`、collector、component validation、compiled attributes与renderer，确认localized table靠fabricated empty String通过schema后仍是TOML table，而renderer只接受`Value::as_str()`，不会解析translation或reference fallback；
- 追踪dependency manifest到全仓consumer，确认localization dependency只被生成/序列化和测试，没有package/cook/runtime reader；collector literal规则又只覆盖`.text/.label/.title`；
- 逐action追踪UI Asset locale preview，确认固定`authoring-fallback/en-US/zh-CN`只刷新key存在性诊断与report列表，preview compile没有locale/catalog参数，catalog只存key set且production无注册caller；
- 追踪Runtime system locale、composite font、Cosmic font cache、ResourceKind、project/export/cook和script host，确认font/shaping culture基础真实存在，但无String Table/Target/Archive、game culture authority、lookup service、script API与shipping culture/chunk；
- 核对tools Localization Workbench/Preview，确认97% coverage、PO export ready、missing/overflow均为design fixture而非production document/controller/job/artifact；
- 对照Unreal Localization Target/Dashboard/String Table/Gather/Import/Export/LocRes/chunk，Godot Translation/Domain/Server/PO/CSV/pseudo preview；同时限定Fyrox/Bevy/Unity Graphics本地源码的非产品参考范围，建立identity/source/archive/artifact/runtime/editor分层。

本轮没有修改production Editor/runtime/interface代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达Localization产品行为的相同lane；详细5个P0、60个P1、12个P2、M0-M9路线及32个验收门见`zircon_editor/33-localization-string-table-culture-translation-import-export-fallback-pseudo-localization-preview-authoring-review.md`。5个在途文件均非本轮产生，实施前必须重算58文件fingerprint并复核preview/catalog/font终态。

## 80. Editor Sprite / Atlas / TileSet / TileMap / Canvas 2D / Animation / Collision / Preview Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Asset/schema/import | 13 / 2,413 / 82,454 | E3逐字段/分支：TileSet/TileMap、SpriteAtlas、ImportedAsset、marker/load、TOML ingest；16个test attributes |
| Scene/render/performance | 27 / 4,660 / 172,951 | E3逐值路径：Sprite component、Scene IO、extract/phase、CPU tessellation、batch、WGPU submit、visibility与Tilemap feature slot；10个test attributes，1个在途文件 |
| Editor/atlas/scene UX | 18 / 2,239 / 78,007 | E3逐调用/控制路径：atlas pack/write/resolve/cache、asset type registry、NodeKind和菜单映射；16个test attributes |
| Tilemap plugin/product assembly | 22 / 1,357 / 49,977 | E3逐descriptor/provider：manifest、runtime/editor registration、operation、resource、first-party catalogs和App feature；14个test attributes，1个在途文件 |
| Focused tests | 16 / 3,383 / 124,399 | E3静态阅读：authoring schema、scene reference、sprite render、asset registry和plugin descriptor；78个test attributes |
| selected combined scope | 94 / 13,837 / 500,887 | 当前工作树fingerprint `687fa008...ed80f47e3`；126个test attributes、0 ignored、2个在途文件 |

Editor 2D Authoring轮次详细阅读覆盖：

- 逐字段追踪`Sprite2dComponent`、render snapshot、phase queue、CPU slice expansion、adjacent-texture batch与WGPU submit，确认Material不进入shader/PSO/batch，所有stage共用alpha-blend/no-depth-write管线，且每batch每帧创建buffer和render pass；
- 追踪Scene source到World load/save，确认TileMap reference在load中被忽略、save固定为`None`，而Sprite2D连Scene字段都不存在，构成确定性内容丢失；
- 逐字段核对SpriteAtlas校验、packer、artifact writer和Retained Host resolver，确认严格pixel/UV校验和确定性pack可保留，但packer没有production caller、只写Editor UI cache且不是原子多artifact发布；
- 核对ResourceKind/ImportedAsset/marker/load与TileSet/TileMap schema，确认typed asset接缝真实存在，但单图/单TileSet/dense numeric cell/string collider只有layer-length validation，不能作为长期工程合同；
- 追踪Tilemap runtime/editor plugin、first-party catalogs与App feature，确认runtime只有dynamic descriptor和DiagnosticOnly Tiled importer，Editor五个operation无factory、两份ZUI不存在，默认产品也未装配provider；
- 精确搜索Flipbook/AnimatedSprite/Canvas2D/Light2D/SpriteMask/2D physics与Editor create/inspect/pick，确认production产品缺失，generic 3D subsystem没有消费TileSet collider或TileMap数据；
- 对照Unreal Paper2D Sprite/Atlas/Flipbook/TileSet/TileMap与toolkits、Godot TileSet/TileMapLayer/tiles editor、Fyrox tilemap runtime/editor、Bevy chunk renderer，并以Unity Graphics 2D visual tests限定render验收范围；
- 建立`source + recipe -> compiler -> generation-qualified artifact -> Scene component -> chunk/instance renderer`分层，以及transactional Sprite/Atlas/TileSet/TileMap document、collision/nav/occlusion cook、Tiled reimport和32项资格门。

本轮没有修改production Editor/runtime/interface/plugin/App代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达2D产品行为的相同lane；详细5个P0、60个P1、12个P2、M0-M9路线及32个验收门见`zircon_editor/34-sprite-atlas-tileset-tilemap-canvas-2d-animation-collision-preview-authoring-review.md`。2个在途文件均非本轮产生，实施前必须重算94文件fingerprint并复核App provider与Sprite phase extract终态。

## 81. Editor Texture / Image / Cubemap / RenderTarget / Sampler / Compression / Streaming / Preview Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Texture model/import/IBL artifact | 53 / 11,640 / 412,144 | E3逐字段/分支：descriptor、payload、image decode、DDS/KTX/ASTC、cube/array、external source cubemap、PMREM、artifact key/cache；94个test attributes |
| GPU upload/streaming/output target | 19 / 5,094 / 181,988 | E3逐调用/资源生命周期：upload plan、sampler cache、ensure、residency调度、texture rebuild、camera target与writeback；54个test attributes |
| Texture/importer plugins | 66 / 10,710 / 362,657 | E3逐manifest/provider/importer：texture shell、主texture_importer、重复asset_importers owner、mipgen与BC5；183个test attributes |
| Editor preview/product assembly | 28 / 3,739 / 127,314 | E3逐provider/控制路径：type registry、preview generation/cache/scheduler、first-party catalogs与App feature；18个test attributes |
| Focused contract tests | 16 / 4,758 / 166,640 | E3静态阅读：upload readiness、cube/source cubemap/IBL staging、camera texture target与registry；76个test attributes，1个ignored |
| selected combined scope | 182 / 35,941 / 1,250,743 | 当前工作树fingerprint `8594ca82...1fc72ceb`；425个test attributes、1个ignored、4个在途文件 |

Editor Texture Authoring轮次详细阅读覆盖：

- 逐字段核对`TextureAsset`、descriptor、metadata与upload readiness，确认typed usage/mip/color/sampler和严格RGBA8/container shape/byte校验可保留，同时确认单一Texture identity、字符串format、重复extent字段及任意settings覆盖无法证明payload不变量；
- 追踪builtin与plugin普通图片导入，确认HDR/EXR均先`to_rgba8()`；HDR usage会在metadata validation拒绝RGBA8，而float format重标又被upload readiness拒绝，形成普通HDR导入/上传确定性断裂；
- 追踪mip/compression实际值路径，确认builtin不执行offline mipgen，plugin虽执行mip/normal流程但Kaiser fallback存在静态arity断点；自产encoder只有BC5，BC7/BC4/BC6H等默认目标只是metadata而非actual artifact；
- 逐header/subresource阅读DDS/KTX/KTX2/ASTC upload plan，确认container ingestion、设备能力门和source-only cubemap拒绝真实存在，但没有source-to-platform encoder、tool/version/RDO key、bulk mip artifact与mobile/web cook；
- 单独追踪Environment IBL，确认RGBA32F decode、external source cubemap、source mip、GGX PMREM、SH9/IEM、parallel executor、algorithm/versioned BLAKE3 key与atomic source/derived publication是必须保留的工程化基础，不能与普通`.zcube` RGBA8路径混淆；
- 逐调用追踪`ensure_texture`与mip streaming，确认首次全链上传、render resource路径同步load/rebuild、compressed禁用、主视图mesh/material-only demand及scale近似；SVT只有metadata且被普通streaming排除；
- 逐字段核对Sampler和OutputTarget，确认sampler缺compare/border/LOD合同，Camera以普通Texture handle引用target，OutputTarget又仅接受单层单mipRGBA8/sample count 1并缺少pool/resize/history/readback政策；
- 追踪Editor registry、preview cache/scheduler、Texture插件与两套importer包，确认raw source `image::open()`会拉伸thumbnail且不能覆盖container/cube/array/volume，Texture Editor引用不存在ZUI、无factory/controller/default装配，重复importer owner中一套不注册实现；
- 对照Unreal Texture/DDC/streaming/toolkit、Godot texture/layered import和editor、Bevy HDR/EXR/sampler/cache、Fyrox typed texture/import options，以及Unity Graphics RenderGraph TextureDesc/atlas/mip debug，并严格限制各参考源码适用范围。

本轮没有修改production Runtime/Editor/interface/plugin/App代码或tests，也没有修改lockfile。定向`zircon_plugin_texture_importer_runtime --locked --offline`检查在编译前被既有`zircon_plugins/Cargo.lock`漂移阻断；此前`zircon_editor --lib`测试编译仍被239个既有错误/122个warning阻断。详细5个P0、60个P1、12个P2、M0-M9路线及32个验收门见`zircon_editor/35-texture-image-cubemap-render-target-sampler-compression-streaming-preview-authoring-review.md`。4个在途文件均非本轮产生，实施前必须重算182文件fingerprint并复核import validation、App feature与image metadata终态。

## 82. Editor Video / MediaSource / Player / Track / Clock / MediaTexture / Playback / Capture / Recording Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Resource/UI identity与产品装配 | 14 / 2,439 / 90,903 | E3逐枚举/映射/manifest：ResourceKind、ImportedAsset、UI Media推断与Data降格、Editor type registry、first-party catalogs与App feature；2个test attributes |
| Runtime time与Sound接入 | 22 / 1,559 / 56,349 | E3逐字段/调用：frame/real/virtual/fixed time、Sound source DTO、external block store、voice adapter、clip controls与timeline advance；1个test attribute |
| Render capture、readback与ABI | 18 / 4,365 / 174,949 | E3逐资源生命周期：CapturedFrame/HDR、RenderFramework capture、Viewport mailbox、3-slot GPU readback、ABI owned output和App PNG publication；27个test attributes |
| Editor gateway/capture产品面 | 13 / 3,347 / 145,006 | E3逐command/ownership/job：gateway wrapper、静态workbench反馈、template binding、profiling artifact admission/export；10个test attributes |
| Focused contract tests | 6 / 2,319 / 88,551 | E3静态阅读：readback、render debugger/visual export、gateway demand/ownership与viewport editing；57个test attributes，1个ignored |
| selected combined scope | 73 / 14,029 / 555,758 | 当前工作树fingerprint `34e57da8...d9628f4`；97个test attributes、1个ignored、5个在途文件 |

Editor Media Authoring轮次详细阅读覆盖：

- 精确检索ResourceKind、ImportedAsset、import/provider/catalog/App feature与Cargo backend，确认没有MediaSource/Player/Track/Texture/Clock/Decoder/Writer身份或常见媒体backend依赖；
- 追踪UI template resource inference与resolver，确认`.mp4/.webm/.mov`等会被识别为`UiResourceKind::Media`，随后统一降为`ResourceKind::Data`，形成公共模板字符串支持但无播放语义的truthfulness断裂；
- 逐字段追踪Sound External source提交、state与voice同步，确认PCM block只按handle覆盖保存，缺PTS/FIFO/backpressure/EOS，且实际Kira voice adapter明确返回unsupported；
- 核对FrameClock、real/virtual/fixed clocks与Sound timeline，确认simulation timing基础真实，但没有media epoch、audio device master、PTS correlation、seek flush、drift或A/V sync；
- 追踪`CapturedFrame`/`CapturedHdrFrame`、3-slot `GpuReadbackQueue`、Viewport generation mailbox和capture report/profile，确认异步单帧readback基础可保留，但无timestamped video sample、YUV/color/HDR/zero-copy/MediaTexture产品合同；
- 核对`ZrRuntimeFrameV1`、Editor gateway foreign ownership与App PNG writer，确认跨ABI只传width/height/generation/RGBA bytes，单帧PNG虽原子发布但没有continuous session、audio、encoder、mux、pacing/finalize；
- 追踪Workbench Capture Frame与profiling artifact export，确认production高层无capture caller，命令固定返回frame 1234，profile job虽先admission再物化单图但最终路径直写且不是Recorder；
- 对照Unreal Media player/controls/tracks/samples/clock/assets/provider插件、Godot VideoStream/Player/MovieWriter和Unity Graphics逐camera RenderGraph capture hook，同时明确Bevy缺失与Fyrox FBX record不能作为成熟媒体参考；
- 建立`MediaSource -> Provider -> Session -> Track/Sample/Clock -> AudioSink/MediaTexture`与`CaptureSource -> Recorder -> Encoder/Muxer -> durable artifact`双链、5项P0、60项P1、12项P2、M0-M9及32项资格门。

本轮没有修改production Runtime/Editor/interface/plugin/App代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达Media产品行为的相同lane；详细内容见`zircon_editor/36-video-media-source-player-track-clock-media-texture-playback-capture-recording-authoring-review.md`。5个在途文件均非本轮产生，实施前必须重算73文件fingerprint并复核App feature、gateway foreign output和Capture Frame binding终态。

## 83. Editor Volume / Zone / Trigger / Region / Gameplay-Audio-PostProcess Environment Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Scene、Post Process与共享shape候选 | 16 / 6,412 / 239,567 | E3逐字段/调用/测试：Scene持久化、Collider property access、volume registry/extract/evaluator、unsupported shape与camera mask；35个test attributes |
| Physics trigger与LevelSystem事件 | 18 / 2,938 / 103,915 | E3逐pair/event/lifecycle：Builtin/Jolt、BTree pair diff、Enter/Stay/Exit、manager/runtime event surface；24个test attributes |
| Sound Volume runtime与Editor贡献 | 18 / 1,254 / 48,126 | E3逐descriptor/service/effect/drawer：validation、storage、source-position influence、strongest resolver、空drawer和无Scene bridge；0个test attributes |
| Navigation modifier/area volume | 10 / 1,466 / 46,840 | E3逐bake path：descriptor、hierarchy inheritance、Empty-node AABB、point classification、diagnostics与overlay；16个test attributes |
| Volume/Post Process Editor产品面 | 9 / 4,319 / 172,309 | E3逐模板/route/feedback/Inspector projection：固定业务数据、固定queued、字段绑定和dynamic component fallback；6个test attributes |
| selected combined scope | 71 / 16,389 / 610,757 | 当前工作树fingerprint `ba7dfd9d...2d0a80c`；81个test attributes、0 ignored、7个在途文件 |

Editor Spatial Region Authoring轮次详细阅读覆盖：

- 逐字段追踪Scene PostProcessVolume/Collider持久化、volume extract/evaluator/registry与camera mask，确认typed连续blend基础真实，同时确认local volume缺Collider或使用Capsule/Cylinder/Convex/Mesh/HeightField/Compound时component仍存在而extract为空，且Inspector/cook/runtime没有产品诊断；
- 逐pair追踪Builtin/Jolt trigger、BTree previous/current diff与LevelSystem snapshot，确认Enter/Stay/Exit和deterministic方向基础可保留，但event缺pair generation/subshape/sequence/exit cause/current overlap，production没有Damage/Checkpoint/Streaming consumer；
- 逐descriptor/service/DSP追踪SoundVolume，确认validation、stable tie与gain/low-pass/reverb/convolution真实执行，同时确认Box只有world center/extents、无rotation，只按source position选strongest volume，且全仓无Scene create/update/remove bridge；
- 逐bake path追踪NavMeshModifier与area volume，确认dynamic descriptor和area修改真实存在，同时确认Empty node scale被当无旋转AABB、冲突取第一个、area按source node position而非triangle coverage判定；
- 逐control/route/feedback追踪通用Volume与Post Process Workbench，确认`VOL_DamageZone`、Damage/Reverb/Checkpoint/Streaming、DPS、24 volumes/12 overlaps/1 warning及queued结果均为静态第二authority，field commit不触达Scene transaction或runtime receipt；
- 精确检索PostProcess Scene property access、Sound `update_volume/remove_volume`生产caller与Gameplay region类型，确认typed PostProcess无法从现有Inspector编辑、AudioVolume drawer为空、DamageZone/CheckpointVolume/StreamingGate没有生产domain；
- 对照Unreal共享AVolume加域专用Trigger/Physics/Audio/PostProcess/NavModifier、Godot Area3D shape-pair/override mode、Unity Graphics Volume/Profile/Stack/Manager、Fyrox sensor intersection，并限定Bevy typed event的非空间产品边界；
- 建立`SpatialRegionSource -> CompiledRegionGeometry -> generation-qualified RegionInstance/Index -> typed domain adapters -> Editor document/Inspector/gizmo/diagnostics`分层、5项P0、60项P1、12项P2、M0-M9及32项资格门。

本轮没有修改production Runtime/Editor/interface/plugin/App代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达Region产品行为的相同lane；详细内容见`zircon_editor/37-volume-zone-trigger-region-gameplay-audio-post-process-environment-authoring-review.md`。7个在途文件均非本轮产生，实施前必须重算71文件fingerprint并复核PostProcess registry/extract与Workbench binding终态。

## 84. Editor Weather / Climate / Time-of-Day / Wind / Precipitation / Cloud / Atmosphere / Environment Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Weather Editor产品面 | 11 / 4,835 / 236,083 | E3逐模板/route/feedback/binding/catalog：固定Storm、Regions、Layers、timeline、warnings、presets和queued反馈；11个test attributes |
| Scene环境authoring与持久化 | 16 / 4,097 / 152,217 | E3逐asset/component/extract/project I/O：Camera/Light/PostProcess可持久化，环境只由`preview_skybox`布尔值映射固定skybox；15个test attributes |
| Sky、Cloud、Fog、IBL与lighting底座 | 24 / 6,986 / 248,673 | E3逐extract/graph/scheduler/recorder/shader：procedural gradient sky、volumetric fog与IBL发布基础；`CaptureCloud`重复sky capture；73个test attributes、1 ignored |
| Runtime clocks与time service | 10 / 966 / 29,806 | E3逐clock/frame/fixed-step/timer：real/virtual/fixed时间基础真实，无calendar/geography/celestial authority；9个test attributes |
| CPU/GPU particles与Editor贡献 | 24 / 4,748 / 168,762 | E3逐asset/simulation/planner/shader/authoring：sprite particle基础真实，GPU缺CPU external force parity且无wind/precipitation/surface coupling；24个test attributes |
| 跨域plugin/material/sound/terrain/catalog核查 | 6 / 772 / 28,746 | E3精确存在性检索：无Weather/Climate plugin、资产owner、wetness/snow/rain/wind字段或Audio/Terrain/Material产品adapter；1个test attribute |
| selected combined scope | 91 / 22,404 / 864,287 | 当前工作树fingerprint `7c2378ae...b779ce`；133个test attributes、1 ignored、8个在途文件 |

Editor Weather / Climate / Environment Authoring轮次详细阅读覆盖：

- 逐control/route/feedback追踪Weather Workbench，确认`Weather_Storm`、`Region_Mountains`、`Layer_Clouds`、Cloud Build/Rain Burst/Wind Gust/Lightning timeline、8 layers/5 regions/2 warnings与build/preview结果全是静态第二authority；
- 逐Scene asset/component/project I/O/render extract追踪环境来源，确认Camera、DirectionalLight与PostProcess持久化真实，但没有Environment/Climate/Weather Scene owner，`build_environment_extract`只把`preview_skybox`布尔值映射为固定default或disabled；
- 逐字段追踪`ProceduralSkyParams`与DirectionalLight，确认gradient sky加sun disk可运行，但两者没有celestial authority、calendar/geography/astronomy、light/sky synchronization或物理大气参数；
- 逐pass追踪Realtime IBL graph、scheduler、recorder、double buffer、stale/retry/publish和time slice，确认发布基础可保留，同时确认`CaptureCloud`和`CaptureSky`调用同一gradient shader写同一source cubemap mip，Cloud命名没有云渲染语义；
- 逐clock追踪real/virtual/fixed/frame/task timer，确认duration、pause、scale、fixed step真实，但无world date/time、timezone、latitude/longitude、sun/moon/season或网络确定性Weather tick；
- 逐asset/simulator/GPU planner追踪Particle，确认shape/rate/lifetime/velocity/gravity/drag和CPU optional physics可用，同时GPU不编码CPU `external_force`，也没有wind field、precipitation camera volume、surface impact、wetness或snow accumulation合同；
- 精确检索Runtime/Editor plugin catalog、Terrain、Sound、Material与Graphics字段，确认没有Weather/Climate插件或typed跨域owner，`weather.Component.CloudLayer`与Wind命中仅为Runtime registry测试fixture；
- 对照Unreal SkyAtmosphere/VolumetricCloud/DirectionalLight atmosphere sun/cloud shadow/WindDirectionalSource、Unity HDRP VisualEnvironment/PhysicallyBasedSky/VolumetricClouds/GlobalWind、Godot Environment/WorldEnvironment/FogVolume、Bevy Atmosphere和Fyrox SkyBox持久化边界；
- 建立`Climate/Celestial/Weather/Region Source -> deterministic Compiler/Artifact -> generation-qualified Weather Snapshot -> typed Light/Sky/Cloud/Fog/IBL/Particle/Audio/Surface adapters -> transactional Editor toolkit`分层、5项P0、70项P1、12项P2、M0-M11及32项资格门。

本轮没有修改production Runtime/Editor/interface/plugin/App代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达Weather产品行为的相同lane；详细内容见`zircon_editor/38-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-authoring-review.md`。8个在途文件均非本轮产生，实施前必须重算91文件fingerprint并复核environment extract、IBL recorder与Workbench binding终态。

## 85. Editor Spline / Path / Road / River / Decal / Brush / Geometry Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Decal package、catalog与pipeline truth | 16 / 1,862 / 64,514 | E3逐descriptor/manifest/registration/pass placement：projector descriptor无consumer，executor为空，feature disabled-by-default且未进入第一方catalog；8个test attributes |
| Scene asset、reflection与DynamicScene边界 | 23 / 3,720 / 139,955 | E3逐static Scene/project I/O/World registration/DynamicScene capture：动态组件底座真实，项目Scene没有plugin component authoring bridge；3个test attributes |
| Material domain与render contract | 14 / 4,575 / 180,200 | E2/E3逐material schema/domain DTO/Workbench route/executor：UI提供Decal，runtime enum和`.zmaterial`没有该domain；8个test attributes |
| Editor Scene tool与plugin authoring底座 | 36 / 4,954 / 162,295 | E3逐SceneMode/selection/overlay/plugin hook及Terrain/Particle对照：基础可扩展，Decal没有任何contribution；31个test attributes |
| mock surfaces与项目专用Road绕行 | 13 / 19,759 / 893,735 | E2/E3逐WOC extract/codegen/script query、Vampire mesh和静态Workbench/preview：14条道路被生成成索引分支并O(total segments)扫描；0个test attributes |
| 既有非空间curve碎片 | 7 / 1,532 / 52,383 | E3逐glTF/Hermite/Sound automation：只有时间/value curve，无空间identity、arc length、frame、bounds/index；5个test attributes |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics参考 | 36 / 25,382 / 1,003,701 | E2/E3按Spline/SplineMesh/PathFollow/Water/Road/Decal/Editor visualizer/CSG职责路由；12个test attributes |
| selected combined scope | 145 / 61,784 / 2,496,783 | 当前工作树fingerprint `6af26787...14524351`；67个test attributes、0 ignored、2个在途文件 |

Editor Spline / Path / Road / River / Decal / Brush / Geometry Authoring轮次详细阅读覆盖：

- 精确检索production ResourceKind、component、asset、plugin、Scene与Editor入口，确认没有空间Spline/PathFollow/Road/River/WaterBody/Geometry Brush/CSG产品；`CubicSpline`和Hermite命中仅为动画插值，Sound curve仅为标量自动化；
- 逐descriptor/registration/pass/executor追踪`rendering.decals`，确认插件只注册字符串property、PostProcess pass和永远`Ok(())`的空executor，mode/opacity/normal blend/atlas region无实例、extract、shader、resource、culling、batch或GPU consumer；
- 逐Editor plugin hook和Material route追踪Decal authoring，确认声明的drawer ID从未注册，主Editor无projector visualizer/operation/transaction，Material Workbench提供`decal`但runtime `MaterialDomain`与`.zmaterial`没有该domain；
- 逐static Scene、plugin `apply_to_world`、World dynamic component与DynamicScene v2 capture/migration追踪，确认reflection/JSON/generation底座真实，但项目Scene不能创建/保存plugin component，Decal也没有typed default、migration或render consumer；
- 逐WOC道路extract、JSON contract、codegen与VM查询追踪，确认14条二维折线有source-order/digest验证，但被生成为大量X/Z索引分支，植被每次距离查询遍历所有道路/segment，且没有mesh/UV/material/collision/nav/traffic/terrain/partition产物；
- 核对Vampire示例的重复Road mesh、Terrain/Foliage的Brush/River静态label及独立preview工具的Decal页面，确认它们分别是普通mesh和设计fixture，不是engine product authority；
- 逐SceneMode registry/factory/stack、selection、overlay provider与Terrain/Particles authoring contribution核对可复用Editor底座，确认当前context仍缺document transaction、typed sub-selection、async compiler与stable spline pick identity；
- 对照Unreal Spline/SplineMesh/Landscape/Water/Decal/Brush、Godot Curve3D/PathFollow3D/Decal、Fyrox Decal、Bevy cubic spline/clustered-forward decal，以及Unity HDRP DecalProjector/System/Editor，严格区分数学、runtime renderer与Editor产品边界；
- 建立`SpatialSplineSource -> deterministic CompiledSplineArtifact -> immutable runtime query -> typed Path/Road/River/Geometry consumers`和独立`DecalMaterial/Projector -> extract/cull/batch/render/editor`双链、5项P0、70项P1、12项P2、M0-M11及32项资格门。

本轮没有修改production Runtime/Editor/interface/plugin/App代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达Spline/Decal产品行为的相同lane；详细内容见`zircon_editor/39-spline-path-road-river-decal-brush-geometry-authoring-review.md`。2个在途文件均非本轮产生，实施前必须重算145文件fingerprint并复核Material preview actions与world-building binding终态。

## 86. Editor Procedural Content Generation / Rule Graph / Biome / World Generation Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Scatter Editor false surface | 10 / 4,303 / 220,696 | E3逐Workbench route/template/action：固定`SC_Forest`、18 rules、64K instances、1 conflict和queued反馈；2个Rust test attributes、2个在途文件 |
| Terrain package | 16 / 856 / 31,660 | E3逐manifest/descriptor/importer/operations：importer明确DiagnosticOnly且返回backend-not-installed，operation只有descriptor无executor；8个test attributes |
| Terrain asset与Scene边界 | 19 / 4,965 / 195,148 | E3逐asset/schema/project Scene/component/runtime consumer：平面`width * height`高度数组与layer列表，没有chunk/LOD/edit layer/stream/cook/runtime terrain产品；17个test attributes |
| WOC source projection与生成脚本 | 38 / 8,904 / 348,107 | E3逐extract/codegen/build script/VM query/check script：有fixed source、sentinel/digest、known vector和局部cell碰撞重演，但没有engine PCG产品authority |
| pinned WOC source | 7 / 4,792 / 185,602 | E2逐terrain/noise/decoration/collision投影源：保留项目专用确定性语义，不能替代通用graph、artifact、resource ownership或Editor toolkit |
| Unreal PCG参考 | 22 / 15,342 / 567,906 | E2/E3按Graph/Compiler/Executor/Cache/Partition/Managed Resource/Editor职责路由；本仓`PCGBiomeCore`为空模块，不作为完成证据 |
| Godot/Fyrox/Bevy/Unity Graphics参考 | 20 / 17,893 / 750,393 | E2/E3按Noise/MultiMesh、Terrain edit/undo/quadtree、asset event/render batching和GPU instance backend路由；30个test attributes |
| selected combined scope | 132 / 57,055 / 2,299,512 | 当前工作树fingerprint `503bfb6c...28cba84`；57个test attributes、0 ignored、2个在途文件 |

Editor Procedural Content Generation / Rule Graph / Biome / World Generation Authoring轮次详细阅读覆盖：

- 精确检索production asset/type/node/pin/data/compiler/executor/cache/request/partition/managed generated resource/editor product，确认没有engine PCG产品；`WorldGeneration`仅是Scene world revision counter，不能计为世界生成系统；
- 逐Scatter Workbench route/template/binding/action追踪，确认UI由固定行、固定统计和固定冲突构成，没有graph canvas/model、backend job、generation receipt、inspection或cancel/retry产品闭环；
- 逐Terrain package manifest、diagnostic-only importer、operation descriptor、TerrainAsset与Scene reference追踪，确认import明确断路、operations没有executor、资产模型是平面高度数组且没有runtime consumer、chunk/LOD/edit layer/stream/cook receipt；
- 逐WOC固定source commit、sentinel/digest、hash/noise/terrain known vector、decoration lattice与collision grid追踪，确认项目脚本确有可复现codegen和邻域candidate重演基础；`npm run check:m3-terrain`通过，耗时11.5秒；
- 核对WOC输出边界，确认它没有engine graph/type/compiler/cache、stable generated identity、provenance/regeneration diff、managed render instances、terrain artifact、partition/cook receipt或通用Editor workflow；
- 对照Unreal PCG Graph/Compiler/Executor/Cache/Partition/Managed Resource/Editor determinism-diff-profile-log，Godot Noise/MultiMesh，Fyrox Terrain edit/undo/quadtree，Bevy asset event/render batching及Unity Graphics GPU instance backend，严格区分通用PCG产品、地形编辑、实例渲染和项目专用内容脚本；
- 建立`PcgGraphSource -> typed compiler artifact -> deterministic GenerationRequest/cache/partition -> immutable typed data -> managed generated resource -> typed adapters`、Biome/WorldRecipe与transactional Editor toolkit，记录5项P0、70项P1、12项P2、M0-M11及32项资格门。

本轮没有修改production Runtime/Editor/interface/plugin/App代码或tests。`npm run check:m3-terrain`在11.5秒内通过；此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达PCG产品行为的相同lane。详细内容见`zircon_editor/40-procedural-content-generation-rule-graph-biome-world-generation-authoring-review.md`。2个在途文件均非本轮产生，实施前必须重算132文件fingerprint并复核preview actions与world-building binding终态。

## 87. Editor Level Variant / Data Layer / Level Instance / World Outliner Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Level Variant false surface | 10 / 13,025 / 726,046 | E3逐ZUI/route/navigation/feedback/preview设计入口：固定`Vehicle_Showcase`、18 overrides、2 conflicts，Preview/Apply只写queued文字；1个test attribute、1个在途文件 |
| Hierarchy与World Outliner基础 | 115 / 11,942 / 415,917 | E3逐Runtime inspection/artifact、message/delta、retained projection、pointer、paint、filter、rename、drag/reparent及focused tests；86个test attributes、1个在途文件 |
| Scene、Prefab与Level Instance边界 | 26 / 4,372 / 176,712 | E3逐ResourceKind、Scene DTO/document/artifact/World IO与完整prefab_tools package；12个test attributes，确认World load-save会把`prefab_instance`写成`None` |
| Unreal参考 | 60 / 28,078 / 1,028,423 | E2/E3按Data Layer authority、Level Instance lifecycle/edit、Variant capture/apply及Scene Outliner item/hierarchy/mode/column/filter职责路由 |
| Godot/Fyrox/Bevy/Unity Graphics参考 | 21 / 17,666 / 627,687 | E2/E3按Scene owner/instance/editable children、WorldViewer provider/undo、ScenePatch依赖生命周期及rendering-layer非等价边界路由；2个test attributes |
| selected combined scope | 232 / 75,083 / 2,974,785 | 当前工作树fingerprint `d710ebd9...9ae4b71`；101个test attributes、0 ignored、2个在途文件 |

Editor Level Variant / Data Layer / Level Instance / World Outliner Authoring轮次详细阅读覆盖：

- 精确检索ResourceKind、asset、component、manager、subsystem、Editor controller与operation，确认Level Variant只存在静态Workbench，Data Layer和Level Instance没有生产产品类型；`ResourceKind::Data`、`render_layer_mask`、`active_in_hierarchy`和runtime parent均不是等价实现；
- 逐Level Variant ZUI、binding、navigation、preview action和feedback追踪，确认set/variant/binding/capture/override/conflict全是固定字符串，Preview/Apply没有asset、reflection address、executor、transaction、preflight、rollback或runtime artifact；
- 逐Runtime inspection/artifact、Editor message/publication、retained fragment/filter/pointer/renderer和transaction追踪，保留stable row/subtree hash/generation、稀疏patch、selection revision、5K deep filter、F2 rename、多选reparent和10K clipped paint真实基础；
- 逐最终pane DTO、template virtual rows、topology delta、expand/context-menu全链追踪，确认最终只保留`id/name/depth/selected`、没有typed item/mode/column/filter/folder/layer/instance状态，tree node无expand/collapse，右键菜单不执行command，retained node仍随总item数增长；
- 逐Scene entity/project document/artifact/World IO与prefab_tools package追踪，确认override以字符串path + JSON定位、importer为diagnostic-only、五个operation无executor、helper只清空Vec，且`World::from_scene_asset`不消费prefab而`to_scene_asset`固定输出`None`，形成P0数据损失；
- 对照Unreal Data Layer asset/instance/manager/editor columns、Level Instance interface/subsystem/edit lifecycle、Variant Manager typed capture/apply和Scene Outliner扩展接口，Godot owner/instance、Fyrox provider/undo、Bevy ScenePatch lifecycle及Unity rendering layer，严格拆开四个产品owner；
- 建立`VariantSetAsset + ApplyService`、`DataLayerAsset + WorldState`、`LevelInstanceSource/Record + Subsystem/EditSession`、`WorldOutlinerModel + typed item/mode/column/filter`架构，记录5项P0、70项P1、12项P2、M0-M11及32项资格门。

本轮没有修改production Runtime/Editor/interface/plugin代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达四域产品行为的相同lane；详细内容见`zircon_editor/41-level-variant-data-layer-level-instance-world-outliner-authoring-review.md`。2个在途文件均非本轮产生，实施前必须重算232文件fingerprint并复核scene inspection publication与world-building binding终态。

## 88. Editor Scene Snapshot / World Diff / Merge / Restore / Conflict Resolution Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Editor authoring、Play、recovery与false Diff surface | 62 / 14,280 / 509,814 | E3逐command/transaction、Play snapshot/store、recovery plan、UI document replay与Workbench route/binding/feedback；72个test attributes、2个在途文件 |
| Runtime Session Archive | 565 / 10,510 / 360,657 | E3逐slot capture/diff/apply/restore、archive manifest/index/retention/selection/merge、artifact/path/writer/atomic commit；7个test attributes |
| Runtime DynamicScene、inspection与identity | 57 / 9,686 / 336,520 | E3逐capture eligibility、payload/validation/remap/spawn transaction、live inspection delta和reflection schema；76个test attributes |
| Runtime focused tests | 23 / 6,659 / 245,815 | E3逐roundtrip/path atomicity/retention/slot merge/whole-level restore/exact-match diff；133个test attributes、1个在途文件 |
| Unreal Level Snapshots参考 | 106 / 15,941 / 599,400 | E2/E3按snapshot asset、object/property/reference/hash/diff、selection/filter、custom serializer、restorability/listener与Results UI路由 |
| Godot/Fyrox/Bevy/Unity Graphics参考 | 18 / 17,741 / 613,845 | E2/E3按PackedScene identity/instance、reversible command、ScenePatch dependency resolve和render object-ID非等价边界路由；51个test标记命中 |
| selected combined scope | 831 / 74,817 / 2,666,051 | 当前工作树fingerprint `93bb5e91...62a5dab`；339个test attributes、0 ignored、3个在途文件 |

Editor Scene Snapshot / World Diff / Merge / Restore / Conflict Resolution Authoring轮次详细阅读覆盖：

- 逐DynamicScene capture/registration/adapter/field、document/validation/remap/spawn transaction追踪，确认版本化反射序列化和preflight基础真实，但未注册、无adapter和不可序列化状态被静默跳过，没有source revision、catalog fingerprint或coverage report；
- 逐Runtime Session Archive的565文件slot/facade/artifact/path/writer链追踪，确认canonical payload、seal、manifest、retention、bounded I/O和path CAS可复用，但diff只有完整equality与数量，merge只处理重复slot ID，apply为additive spawn，restore为整World replacement；
- 逐Workbench Diff route/binding/navigation/action/feedback追踪，确认Scene分支只输出`Scene diff prepared`与固定preview compared文字，没有provider、snapshot source、change artifact或executor，形成P0虚假能力面；
- 逐Editor intent/command/transaction、Play Snapshot、autosave RestoreFlow和UI document replay追踪，保留RAII transaction、selection/dirty/history/journal及临时进程输入基础，同时确认没有bulk restore、comparison executor或semantic scene diff；
- 逐WorldInspection hierarchy/field delta追踪，确认它可用于同一live generation的稀疏UI刷新，但不是跨snapshot、schema-stable、可持久化的change authority；
- 对照Unreal `ULevelSnapshot`、World/Actor data、hash-first diff、typed selection map、custom serializer/restorability/listener/filter和Results UI，Godot PackedScene、Fyrox command、Bevy resolved ScenePatch及Unity HDRP object-ID，严格区分scene serialization、render identity、slot container和authoring snapshot产品；
- 建立`SceneSnapshotAsset -> SceneChangeSet -> base/ours/theirs SceneMergeResult -> staged SceneRestorePlan -> one Editor bulk transaction -> receipt`架构，记录5项P0、70项P1、12项P2、M0-M11及32项资格门。

本轮没有修改production Runtime/Editor/interface/plugin/App/Hub代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达Scene Snapshot产品行为的相同lane；详细内容见`zircon_editor/42-scene-snapshot-world-diff-merge-restore-conflict-resolution-authoring-review.md`。3个在途文件均非本轮产生，实施前必须重算831文件fingerprint并复核autosave adapter、Workbench preview action和DynamicScene spawn transaction测试终态。

## 89. Editor Multi-User / Collaborative Editing / Session Replication / Locks / Presence / Transaction Conflict Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Zircon Editor transaction/document/session foundation | 59 / 11,308 / 383,541 | E3逐command/history/scope/journal/event sink、document identity、session guard与control route；52个test attributes |
| Zircon focused transaction/message tests | 25 / 4,575 / 156,052 | E3逐commit/undo/redo/failure recovery、journal roundtrip、paging及bus backpressure；111个test attributes |
| Zircon Hub Team与reserved collaboration | 10 / 5,067 / 180,966 | E3逐local Git projection、Team page、Coming Soon与UI contract；19个test attributes |
| Unreal Concert / Multi-User参考 | 251 / 37,330 / 1,456,512 | E2/E3按session/transport/workspace/activity DB、transaction/package/lock/presence/property authority及UI职责路由 |
| Godot/Fyrox/Bevy/Unity Graphics参考 | 16 / 9,374 / 335,043 | E2/E3确认local undo/command、remote runtime control与local serialized authoring均不等价协同；17个test attributes |
| selected combined scope | 361 / 67,654 / 2,512,114 | 当前工作树fingerprint `43b379c...fd08eb`；199个test attributes、0 ignored、0个在途文件 |

Editor Multi-User / Collaborative Editing / Session Replication / Locks / Presence / Transaction Conflict Authoring轮次详细阅读覆盖：

- 精确检索production Editor/Runtime/App/Plugin/Interface/Hub，确认没有collaborative session、participant、presence、durable activity、server sequencer、distributed authority、remote transaction replay或reconnect/resync owner；Hub Team只投影本地Git身份与contributors，remote collaboration诚实disabled；
- 逐Editor transaction engine、history、scope、command、journal、event sink与focused tests追踪，保留RAII transaction、rollback、selection/dirty/save token及versioned payload真实基础，同时确认`participants`是document集合、`try_merge`是本地coalescing、`TransactionId`/lineage/event均为单进程语义；
- 逐`TransactionJournal` encode/decode和四类Scene command追踪，确认production没有decoder/replay registry，Update/Reflected Field apply没有current-before CAS，raw NodeId/path-derived DocumentId也不具备跨client稳定性，直接广播会形成silent last-writer-wins；
- 逐project session guard的Windows mutex、Unix flock、persisted PID/instance/heartbeat追踪，确认它是必须保留的physical checkout安全底线；协同必须使用独立workspace/sandbox/overlay，不能通过共享root启用；
- 逐Editor08 control边界复核，确认`InvokeBinding/InvokeRoute`已知remote gate/provenance旁路是协同前置阻断，远端命令必须使用principal-safe deny-by-default typed gateway；
- 对照Unreal Concert session/endpoint/transport、workspace/activity DB、transaction/package/lock/presence和property authority，Godot/Fyrox local undo，Bevy JSON-RPC remote mutation及Unity Graphics local Volume Editor，严格区分协同产品、remote control、gameplay networking和source control；
- 建立`authenticated admission -> initial sync -> server-ordered durable activity -> typed codec/precondition -> local transaction projection`、package/property authority、transient presence、checkpoint/reconnect/conflict/workspace/save产品链，记录5项P0、70项P1、12项P2、M0-M11及32项资格门。

本轮没有修改production Editor/Runtime/interface/Hub/plugin/App代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达协同产品行为的相同lane；详细内容见`zircon_editor/43-multi-user-collaborative-editing-session-replication-locks-presence-transaction-conflict-authoring-review.md`。实施前必须重算361文件fingerprint，再运行真实2+ Editor process、server crash、message fault、partition/reconnect、compatibility/security与scale lanes。

## 90. Editor Archetype / Class Defaults / Instance Override / Property Propagation / Reset-to-Default Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Zircon asset/persistence | 9 / 3,267 / 126,377 | E3逐Prefab DTO、Scene DTO/cache/World IO与focused asset tests；18个test attributes，确认World load-save会擦除`prefab_instance` |
| Zircon reflect/default schema | 41 / 4,415 / 144,266 | E3逐Runtime与Interface reflection/default metadata；10个test attributes，确认只有单层optional default |
| Zircon Editor inspector/surfaces | 14 / 4,106 / 163,735 | E3逐Inspector DTO/command/snapshot、Material局部override、Prefab Workbench与SVG；15个test attributes |
| Zircon prefab plugin/support | 17 / 1,250 / 47,192 | E3逐完整package、registration/helper/importer/resources与support batch；12个test attributes |
| Zircon ECS archetype | 12 / 1,945 / 61,882 | E3逐signature/index/table/row/change tick，确认仅为ECS storage layout；10个test attributes |
| Unreal reference | 17 / 3,984 / 146,572 | E2/E3按CDO/archetype、component override/cache、reset policy及Level Instance diff职责路由 |
| Godot/Fyrox/Bevy/Unity Graphics参考 | 26 / 25,323 / 933,220 | E2/E3按default precedence、inheritable modified bit、resolved ScenePatch和Volume override/reset职责路由；41个test attributes |
| selected combined scope | 136 / 44,290 / 1,623,244 | 当前工作树fingerprint `075f2037...699614`；106个test attributes、0 ignored、0个在途文件 |

Editor Archetype / Class Defaults / Instance Override / Property Propagation / Reset-to-Default Authoring轮次详细阅读覆盖：

- 逐Prefab asset、Scene entity/cache/World IO与tests追踪，确认DTO/cache携带`prefab_instance`，但World load完全不读、save固定写`None`；该P0由Editor 41拥有canonical修复责任，本专题将其设为所有default/override动作的前置gate；
- 逐完整`prefab_tools` package、Editor support batch和声明资源追踪，确认五个operation只有descriptor无factory/executor，三个`.zui`/`.toml`资源不存在，runtime importer明确diagnostic-only且overrides property不可序列化；
- 逐92行authoring helper追踪，确认apply只返回去重DTO后清空override、revert只清空vector、break只返回transform与override，没有source写入、effective parent恢复、subtree物化、reference remap、transaction或rollback；
- 逐230行Prefab Workbench、19条route和feedback callback追踪，确认固定`PF_Chest`、`Chest_04`、18 children、6 overrides和2 warnings没有provider/backend，Apply/Validate只产生queued文字；两个override/revert图标也没有production consumer；
- 逐Reflection、Inspector snapshot/change command和Material局部projection追踪，确认single optional `default_value`没有origin/layer/revision，generic Inspector不投影default/override/reset，Set Reflected Field又没有current-before CAS；
- 逐ECS archetype的12文件signature/table/change-tick实现追踪，明确它是运行时component storage locality，不是对象prototype、CDO、Prefab source或class default authority；
- 对照Unreal CDO/archetype、Inheritable Component、Instance Data Cache、Property reset和Level Instance diff，Godot native/script/scene precedence，Fyrox modified/revert，Bevy resolved layered ScenePatch及Unity Graphics Volume override/reset，严格限制参考边界；
- 建立`DefaultValueAuthority -> versioned Prefab/Class source -> stable typed override -> dependency-indexed propagation/rebase -> transactional reset/apply/revert/break -> resolved runtime artifact`产品链，记录5项P0、72项P1、12项P2、M0-M11及32项资格门。

本轮没有修改production Runtime/Editor/interface/plugin/App代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达Default/Prefab产品行为的相同lane；详细内容见`zircon_editor/44-archetype-class-defaults-instance-override-property-propagation-reset-to-default-authoring-review.md`。实施前必须重算136文件fingerprint并先通过Scene无损roundtrip、stable identity、typed migration和transaction fault gates。

## 91. Editor Cinematic Sequencer / Shot / Track / Binding / Take Recorder / Movie Render Queue Authoring 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Zircon静态Sequencer surface | 7 / 12,870 / 704,599 | E3逐ZUI、route、binding、navigation、preview design与feedback；0个tests，确认固定`SEQ_Intro`、shot/key统计及queued反馈 |
| Zircon timeline plugin/contracts | 13 / 2,330 / 81,315 | E3逐完整package、manifest/dist、resource、descriptor、helper和tests；15个test attributes，确认无executor且event capability错配 |
| Zircon runtime sequence/event/playback | 18 / 3,065 / 107,378 | E3逐asset、target、compiler、cache、sample、apply与capability；17个test attributes |
| Zircon generic Animation Editor | 25 / 3,419 / 133,463 | E3逐session、sequence/graph、transport、host lifecycle/save/sync；27个test attributes，2个在途文件 |
| Zircon camera/capture/output substrate | 11 / 2,922 / 111,078 | E3逐capture DTO/mailbox/readback/dynamic API、camera stack和PNG evidence；38个test attributes，2个在途文件 |
| Unreal reference | 26 / 12,093 / 449,556 | E2/E3按MovieScene source/binding/section/hierarchy/player、Sequencer、Take Recorder与Movie Render Pipeline职责路由 |
| Godot reference | 12 / 23,338 / 855,551 | E2/E3按typed animation tracks、player backup/restore和MovieWriter路由 |
| Fyrox reference | 11 / 7,396 / 289,488 | E2/E3按UUID track/signal、command editor、ruler/curve/preview路由；8个test attributes |
| Bevy reference | 3 / 2,882 / 110,234 | E2/E3按AnimationTargetId、clip/event/graph/player路由；10个test attributes |
| Unity Graphics reference | 5 / 1,426 / 65,968 | E2/E3仅按camera capture callback、AOV buffer/completion路由，不外推Timeline/Recorder |
| selected combined scope | 131 / 71,741 / 2,908,630 | 当前工作树fingerprint `ac54c4e4...ad72f5f`；115个test attributes、0 ignored、5个在途文件 |

Editor Cinematic Sequencer / Shot / Track / Binding / Take Recorder / Movie Render Queue Authoring轮次详细阅读覆盖：

- 逐230行Sequencer Workbench、20项production action、navigation/allowlist/template binding和feedback callback追踪，确认固定`SEQ_Intro`、Camera/Audio/Event rows、12 shots、428 keys、Preview/Validate queued文字没有document provider、controller、compiler、job或receipt；
- 逐`AnimationSequenceAsset`、target/compiler/cache/sample/apply与scene player追踪，保留path-free typed property writer底座，同时确认它没有电影track/section/hierarchy/binding/time/restore合同，cache不含per-instance context且apply结果被忽略；
- 逐`timeline_sequence`完整package及Editor support batch追踪，确认五个operation仅为descriptor、声明ZUI物理缺失、dist无command/bridge，`event_marker`没有sequence持久化/runtime evaluator，依赖能力实际只服务Animation Clip事件；
- 逐keyframe helper追踪，确认它以collection index为identity并在全序列validate前原地修改/排序，可能返回`Err`但对象已变化，不能接入transaction/undo；
- 逐Runtime capture/mailbox/dynamic API、Editor poll与PBR PNG writer追踪，确认当前仅有单帧RGBA evidence，缺format/color/timecode/shot/sample/pass、fixed-step、AOV、checkpoint和durable movie artifact；
- 对照Unreal MovieScene/Sequencer/Take Recorder/Movie Pipeline，Godot typed Animation/MovieWriter，Fyrox UUID command editor，Bevy AnimationTargetId和Unity Graphics capture/AOV，严格区分通用动画、单帧capture与电影产品；
- 建立`versioned cinematic source -> hierarchy/binding/section compiler -> deterministic evaluation/pre-animated restore -> provider-backed Sequencer -> Take session -> Queue/Job/Shot -> bounded output/artifact`产品链，记录5项P0、72项P1、12项P2、M0-M11及32项资格门。

本轮没有修改production Runtime/Editor/interface/plugin/App代码或tests，也没有运行动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复无法抵达Cinematic产品行为的相同lane；详细内容见`zircon_editor/45-cinematic-sequencer-shot-track-binding-take-recorder-movie-render-queue-authoring-review.md`。5个在途文件均非本轮产生，实施前必须重算131文件fingerprint并复核Animation session、template binding、camera stack和dynamic frame API终态。

## 92. Hub Marketplace / Account Auth / Organization / Cloud Repository Provider 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Hub remote surfaces/contracts | 32 / 9,667 / 338,557 | E3逐TopBar/User Menu、Catalog/Cloud/Team、fallback、IPC/types、coming-soon、local catalog/Git/delivery及静态contracts；84个test attributes |
| Zircon package/download substrate | 19 / 2,458 / 80,781 | E3逐本地package/install/receipt、plugin manifest/package/signature producer与content download manager；9个test attributes |
| Unreal reference | 14 / 5,541 / 239,991 | E2/E3按Auth、Commerce/Entitlement、User/Title File、Portal service与BuildPatch职责路由 |
| Godot reference | 2 / 2,786 / 97,161 | E3按Asset Library query/version/license/download/SHA/install/offline/proxy路由 |
| Fyrox reference | 4 / 2,279 / 78,308 | E2/E3按local project manager、Cargo dependency与upgrade路由 |
| Bevy reference | 3 / 6,355 / 183,475 | E2只确认Cargo workspace/package dependency，不外推Marketplace/Auth/Cloud |
| Unity Graphics reference | 4 / 228 / 12,832 | E2只确认render package manifest/version/dependency，不外推Unity Package Manager/账号/云服务 |
| selected combined scope | 78 / 29,314 / 1,031,105 | 当前工作树fingerprint `a34523ef...c2fcc40`；93个test attributes、0 ignored、0个在途文件 |

Hub Marketplace / Account Auth / Organization / Cloud Repository Provider轮次详细阅读覆盖：

- exact dependency/term scan确认Hub没有HTTP client、OAuth/OIDC/JWT、token、keyring、TLS/certificate、entitlement、audit或WebSocket实现；Marketplace/Auth/Cloud/Invite/Permission均为disabled coming-soon，保留其真实性基线；
- 逐TopBar/User Menu/Team/local Git追踪，确认Git `user.name`被显示为用户头像与“我的账户”，Account跳Team，而所谓members只是首个仓库最近200条提交聚合的8个authors；要求Local Profile、Git Identity、Online Account与Organization Member硬分域；
- 逐local plugin catalog追踪，确认它只递归读取项目/引擎`plugin.toml`，缺publisher/license/version dependency/digest/signature/entitlement/install state，宽松schema与单项错误还会中断全catalog；
- 逐本地package/device install/receipt与content download追踪，保留owned-directory失败清理、逐文件SHA与Range/chunk hash底座，同时确认不存在signed metadata、durable resume、dependency transaction、atomic activation/update/rollback；
- 逐Cloud页与action history追踪，确认当前仅为本地package/install目录视图，没有snapshot/revision/CAS/blob/upload/download/conflict/encryption/quota/recovery，不能改名为sync；
- 对照Unreal Auth/Commerce/UserFile/BuildPatch与Godot Asset Library，严格限制Fyrox/Bevy/Unity Graphics本地package声明参考边界，不虚构闭源Launcher/Package Manager或远程服务；
- 建立`Auth/Credential -> Organization/RBAC/Audit -> signed Marketplace/Entitlement -> shared Package Service`与`Project Snapshot -> CAS blob/head -> conflict -> staged apply/recovery`两条产品链，记录5项P0、72项P1、12项P2、M0-M11及32项资格门。

本轮没有修改production Hub/Runtime/Plugin/Tooling/Editor代码或tests，也没有连接远程服务或运行动态测试。clean Hub source仍存在`persist_unchecked(&mut self)`与`persist_unchecked(None)`参数不匹配，Hub01已复现managed Cargo `E0061`阻断，本轮未重复同一失败lane；详细内容见`zircon_hub/03-marketplace-account-auth-organization-cloud-repository-provider-review.md`。实施前必须重算78文件fingerprint，并先恢复Hub build、封闭native plugin pre-admission code execution及建立secure credential owner。

## 93. Runtime Interface UI Authoring / Accessibility / Input / Diagnostic / Status 公共合同物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Interface UI完整子树 | 232 / 25,568 / 804,699 | E3逐authoring/compiled/tree/layout/dispatch/focus/accessibility/reflection/control/debug与serde边界；642个公开声明、35个test attributes、0 ignored |
| Interface完整Rust source背景 | 442 / 55,571 / 1,815,527 | E2/E3核对UI占比、runtime API/operation交叉合同；943个公开声明、397个test attributes、1 ignored |
| Runtime纵向consumer | dynamic accessibility、UI loaders/compiler/tree/dispatch与operation service | E3逐request -> producer -> JSON/allocation -> action dispatch及source -> artifact -> runtime路径 |
| Runtime Host与Editor纵向consumer | shared foreign output、v2 projection/save、reflection/control/debug reflector | E3确认七类host budget已收敛、accessibility未纳入，并追踪remote/property/action与有损authoring往返 |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics参考 | 21个定向入口 | E2/E3按authoring/runtime分层、generational identity、central input reply、diagnostic identity、migration和debug lifecycle路由 |
| selected UI source fingerprint | `7eecef1b...cc2707` | 15个interface在途source/test文件；实施前必须重取指纹并复核API/operation/accessibility终态 |

Runtime Interface UI Authoring / Accessibility / Input / Diagnostic / Status轮次详细阅读覆盖：

- 逐legacy `UiTemplateDocument`、v1 `UiAssetDocument`、v2 `UiV2AssetDocument`、两份Runtime loader与Editor `v2_projection`追踪，确认三套活跃authoring authority和`repeat`/slots/focus/navigation/picking/accessibility/widget/unknown syntax有损往返；
- 逐UI 232文件公开/serde面统计，确认642个公开类型、80个`serde(default)`文件、0个`deny_unknown_fields`、43个`usize`文件，以及source/compiled/runtime/debug/transport/implementation混在稳定interface crate；
- 逐dynamic accessibility request/capture/encode/allocation/action链追踪，确认`generation_hint`未读取、完整快照无producer budget/page/delta、App/Editor无host consumer、action无snapshot generation，node ID采用未校验的16/48 bit packing；
- 逐reflection/control/subscription和Runtime/Editor consumer追踪，确认raw path/JSON/optional result/boolean remote gate没有request identity、principal capability、revision/CAS、receipt、sequence/ack/overflow/resync；
- 逐45套Diagnostic/Status/Report/Result与operation submit/poll/harvest追踪，确认code/severity/span/correlation/budget碎片化，Cancelled/Expired终态没有cancel API且harvest outcome无法表达；
- 复核`zircon_runtime_host::foreign_output`当前实现，保留App/Editor共享七类consumer budget、metrics与session fuse进展，同时确认accessibility kind、producer admission、typed metrics和可抢占decode尚未闭合；
- 对照Unreal Slate/UMG accessible object、FReply与generated widget class，Godot typed input/resource/error，Fyrox UiMessage/generational Handle，Bevy AccessKit/ECS/diagnostic，以及Unity Graphics migration/debug lifecycle，并严格限制各参考边界；
- 建立`lossless source -> bounded compiler/migration -> immutable package -> generational runtime -> bounded snapshot/action/diagnostic/side-effect receipt`链，登记3项P0、72项P1、12项P2、M0-M11及32项资格门。

本轮没有修改production Interface/Runtime/Host/Editor代码或tests，也没有运行新的动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误/122个warning阻断，本轮未重复相同不可达lane；详细内容见`zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`。

## 94. Runtime Interface Profiling / Plugin Event / Script Diagnostic / Manifest 收口物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| selected interface closing scope | 8 / 1,998 / 74,510 | E3逐profiling、manifest、plugin ABI/diagnostic/event、script diagnostic、dynamic event DTO与lib exports；10个test attributes、0 ignored |
| Runtime profiling/diagnostics | recorder/control/export/dynamic response | E3确认任意`usize`容量、完整snapshot、调用方路径、固定文件覆盖与producer/consumer budget不对称 |
| Runtime native plugin adapter | V3/V4 registration、bridge、stub callbacks | E3确认system/component/bridge有实现，spawn/asset/event明确Unsupported，而diagnostics/metric返回Ok丢弃 |
| Runtime plugin-event mirror | bounded scene queue、dynamic encoder、foreign output | E3确认64/128 KiB page、16K/64 MiB queue、sequence/backlog基础及overflow continuity缺口 |
| App/Editor/Host consumers | profile merge、gateway、bounded pump、shared output policy | E3确认不可比较clock数组拼接、retention无source identity，以及16 MiB/256 KiB consumer cap |
| selected interface fingerprint | `7e030683...df65005` | 3个selected在途文件；实施前重取fingerprint并复核profiling/event/lib终态 |

Runtime Interface Profiling / Plugin Event / Script Diagnostic / Manifest轮次详细阅读覆盖：

- 逐`ProfileCaptureConfig`、control、recorder、snapshot、export与dynamic response追踪，确认capacity无hard max、NaN可穿透、全量deep clone/JSON发生在host cap之前；
- 逐`output_root/session_id`到`create_dir_all/fs::write`追踪，确认absolute/relative root可控且sanitizer保留`.`，`..`可越出root并覆盖六个固定文件；
- 逐Profile/RuntimeDiagnostics/Hotspot/UiScenarioHotspot公共字段和Editor merge追踪，确认缺capture/process/thread/clock/build/completeness，retention vector无source identity，canonical Cube假设进入稳定DTO；
- 逐V3/V4 native host table与实际adapter追踪，确认部分function pointer广告不可用服务，diagnostics/metric尤其返回Ok但无sink，system invoke status又被丢弃；
- 逐module descriptor、event type、component schema、state snapshot和callback request/result追踪，确认手写ABI缺subtable独立版本、codec、registry identity、clock/deadline/correlation与state artifact合同；
- 逐scene event mirror queue、dynamic page、App/Editor shared policy和bounded pump追踪，保留双预算、sequence/backlog进展，同时确认overflow丢事件后没有dropped range/ResyncRequired；
- 逐ScriptDiagnostic与RegistrationDiagnostic追踪，确认severity/code/location仍是孤立小协议，未接共享span/build/correlation/privacy/budget/truncation envelope；
- 对照Unreal Trace/Plugin descriptors、Godot Performance/GDExtension schema、Bevy DiagnosticPath/plugin lifecycle、Fyrox dynamic plugin/stats和Unity Graphics profiling/debug lifecycle，建立Observation/Artifact/generated ABI/Event continuity链；
- 登记3项P0、60项P1、12项P2、M0-M10及32项资格门，selected scope fingerprint为`7e03068352be1072caa74d77a9e0ae0b593238defe5be1ce0d3eb5e63df65005`。

本轮没有修改production Interface/Runtime/Host/Editor/Plugin代码或tests，也没有运行新的动态测试。此前`zircon_editor --lib`编译阻断仍有效；详细内容见`zircon_runtime_interface/04-profiling-plugin-event-script-diagnostic-manifest-crate-ownership-consolidation-review.md`。

## 95. 跨报告 Owner / Schema / ABI / P0 收敛物理范围

| 子域 | 数量 | 本轮状态 |
|---|---:|---|
| 七分类直接专项报告 | 102篇 / 49,060行 / 4,498,274 bytes | E3逐报告计数、metadata、owner与依赖归并 |
| 专项finding | 454 P0 / 4,470 P1 / 949 P2 | 保留原报告唯一owner，不复制全局finding |
| frontmatter路径 | 4,303专项 / 4,329含总表 | 首轮发现6条source owner漂移并修正文档路径，复跑确认0 missing/0 duplicate |
| 标准状态元数据 | 94篇 | 86篇recheck true、8篇false；8篇旧格式待M0迁移 |
| 全局owner与依赖 | O00-O15 / C0-C12 / L0-L5 | 已建立report-to-owner全量路由、manifest schema和40项资格门 |

本轮不是新的浅层源码搜索，而是对既有102次纵向审查做闭包检查：按“报告自述优先、明确无新增P0计零、缺失时唯一ID/表格fallback”重建severity账本，避免Hub02或Runtime03-08F重复继承上游P0；逐frontmatter验证4,303条Zircon/plan/reference路径，定位Editor message/project、Workbench callback和Runtime Interface text共6条owner漂移；逐报告映射O00-O15最低共享owner，并建立BuildSet、Schema、Artifact、ABI、Runtime Generation、Evidence六类不可混淆identity。

详细内容见`01-cross-report-owner-schema-abi-p0-consolidation-review.md`。本轮只修正文档路径、索引和coverage，不修改production或tests，也不重复已知不可达动态lane。Editor仍受617.2秒后239个既有test-build错误/122个warning阻断；Hub仍受已复现`persist_unchecked(None)` E0061阻断。实施顺序必须先L0 truth/build reachability，再走identity/kernel/security、data/artifact/transaction、runtime systems/graphics/UI、authoring、delivery/evidence。

## 96. Neural Model / ONNX / CPU-GPU Inference / Post Process / Editor 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Neural package全量 | 41 / 7,016 / 239,770 | E3逐runtime/editor/post-process/dist与manifest；fingerprint `df857c43...9facc1` |
| Rust source | 36 / 6,886 / 235,676 | 39个test attributes、0 ignored、6个`unsafe`出现 |
| 产品装配 | App Editor Host feature + runtime/editor catalogs | E3追踪默认编译、project selection、registration与capability投影 |
| 包外业务consumer | 非Neural production source | 0个`NnModelAsset`/`NnGraphExecutor`/`NnPostProcessSettings`consumer |
| 参考 | Unreal NNE/NNEEditor/BasicCPU/RDG/NNEDenoiser；Godot/Bevy/Fyrox asset；Unity Graphics Upscaler | 严格区分通用Neural runtime、asset分层和render provider，不外推缺失参考域 |

本轮逐41个文件确认`.znn`格式、ONNX子集转换、CPU reference interpreter和GPU descriptor/WGSL planner均有可保留实现，同时追踪出产品链在包边界终止：`NeuralRuntimePlugin::register()`为空，Post Process测试固定全部runtime extension为空，native dist只有registration manifest；Editor Host默认可注册ONNX importer并生成`neural.model`，仓内却没有Runtime asset loader、model service或包外consumer。

进一步逐format/validate/parser/converter/EditCommand/CLI/CPU/GPU路径确认：Import和undo直接覆盖目标且完整旧bytes进入history；ONNX parser无producer budget、signed dimension直接`as u32`、忽略opset/domain/external data；`.znn`在证明op table记录数前按不可信count预分配并无界复制weights；model validation不验证topology/producer/kind/arity/shape/alias，CPU/GPU又有不一致的晚期contract，GPU Reshape直接别名且elementwise只按output count读取。

对照Unreal的source model -> runtime/target cook/cache -> Model -> ModelInstance -> shape prepare -> CPU/GPU/RDG binding/enqueue，以及NNEDenoiser的IO mapping/history/resource/tiling/view extension，本轮建立bounded source frontend、validated IR、versioned artifact、provider/model instance、真实Render Graph执行和Post Process产品链。登记5项P0、60项P1、12项P2、M0-M11及32项资格门；详细见`zircon_plugins/02-neural-model-onnx-inference-post-process-editor-product-integration-review.md`。

本轮没有修改production或tests，也没有运行新的动态测试；公共Editor test build既有239 errors/122 warnings阻断仍有效，未重复相同不可达lane。

## 97. Desktop Export / Native Window Hosting / Source-Dist Provider 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Desktop Export package全量 | 16 / 1,563 / 65,713 | E3逐manifest/Cargo/source/dist/tests/ZUI/profile；fingerprint `2912aa92...1940e3` |
| Native Window Hosting package全量 | 9 / 476 / 17,299 | E3逐manifest/Cargo/source/dist/tests；包内0个ZUI；fingerprint `99d287d6...c68d6f` |
| 两包Rust | 13 / 1,535 / 62,633 | 11个test attributes、0 ignored |
| 产品装配 | App first-party Editor catalog、native contribution materializer、builtin export/window owners | E3逐selection -> registration -> extension/capability -> view/operation/provider链 |
| 合并fingerprint | `227b7732...45cab5` | 两包成文时source clean；后续源码变更仍需重取 |

本轮确认两个包都不是空目录，但package与产品行为没有闭合。App first-party Editor catalog只链接Navigation和Neural，因此两包source `plugin_registration()`没有默认产品调用者；两份native projection又都声明`extensions: []`，dist behavior没有command/event/bridge/host-ready回调，native loader只能materialize空贡献。CI standalone lane只check/build cdylib，不验证安装、选择、贡献物化、资源、operation、disable或unload。

Desktop Export真正可运行的八阶段plan、process-tree cancellation、Editor job、panel session和retained projection位于`zircon_editor` core，builtin shell又无条件创建该view。插件另注册七个无event/factory的operation、没有业务consumer的`build.export_profile`/`ExportProfileController`和三份没有包外controller的report template；默认`windows-release` profile还固定为debug。Native Window真正的winit/presenter/lifecycle同样位于core，`editor.extension.native_window_hosting`在未安装package时也由optional subsystem默认启用；插件只重复注册core Workbench/Prefab view，并引用物理不存在的`plugins://native_window_hosting/editor/authoring.zui`。

对照Unreal Project Launcher/Slate、Godot ExportPreset/EditorExportPlatform/WindowWrapper、Bevy WinitPlugin、Fyrox Build Tools/Window Settings和Unity Graphics BuildProcessors，本轮建立host service capability、package feature capability与requested capability三分、canonical source/dist contribution bundle、PackageContentManifest、typed behavior provider和enable/disable/update事务。登记4项P0、48项P1、10项P2、M0-M5及32项资格门；详细见`zircon_plugins/03-desktop-export-native-window-source-dist-provider-integration-review.md`。

本轮没有修改production、tests、manifest、ZUI或CI，也没有重复已知不可达的Editor动态lane；239 errors/122 warnings既有阻断仍有效。

## 98. Rendering Umbrella / Feature Bundles / Solari / Native Provider 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Rendering物理目录 | 173 / 9,037 LF行 / 395,923 | 151个tracked文件、15对runtime/editor feature与24个物理shader cache artifact；fingerprint `d64fa967...c657d` |
| Solari物理目录 | 7 / 469 / 17,053 | source unavailable provider与native dist逐文件；fingerprint `7f2c4295...a246a` |
| 两域Rust | 118 / 8,021 / 289,261 | 68个test attributes、3 ignored |
| 产品装配 | runtime/editor catalogs、App Editor composition、generated export、native replay | E3逐manifest -> selection -> feature status -> contribution/provider -> product链 |
| 合并fingerprint | `049b747d...ef3794` | tracked source clean；22个ignored cache文件仅纳入物理快照 |

本轮确认Runtime core中已有多项可保留真实图形实现，但Rendering package没有把它们一致交付给产品。普通Editor只收集umbrella registration，不收集15个feature registrations；feature resolver仍从optional metadata构造定义、报告available并发布capability，extension merge找不到具体registration时静默跳过。Generated export则会生成并提交feature provider source，同一project selection的preview与export因此可能拥有不同RenderFeature/executor/component/shading-model inventory。

Rendering root标为stable/complete，默认post process、SSAO、reflection probes和baked lighting；其中post process与baked lighting executor明确为空，reflection probes无pass/executor。Solari source路径诚实返回Unavailable，但native manifest声明的`runtime.render.solari_provider` extension不会被live-host replay消费：零systems直接返回空成功report。production catalog已有15项、旧tests仍断言9项，static/SDK为0.2.0而programmatic default仍为0.1.0，形成多份当前identity authority。

对照Unreal Renderer/Plugin Manager/Shader Job Cache、Bevy RenderPlugin、Godot Renderer RD/cache、Fyrox Plugin lifecycle和Unity Graphics build/global settings，本轮建立Resolved Product Plugin Graph、RenderFeatureProviderBundle、source/export/native canonical contribution、typed native provider replay及cache root隔离路线。登记5项P0、56项P1、12项P2、M0-M4及10项产品资格门；详细见`zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md`。

本轮没有修改production、tests、manifest、cache或CI，也没有重复已知不可达Editor动态lane；239 errors/122 warnings阻断与3个ignored GPU/产品测试边界仍有效。

## 99. Shader WGSL / Family Importer / Compiler Artifact / Native Dist 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| 新WGSL package | 7 / 590 / 20,618 | manifest/runtime/dist逐文件；7个test attributes、0 ignored；fingerprint `f44ed27a...d37a1c` |
| 旧Shader family package | 7 / 879 / 30,243 | manifest/runtime/dist逐文件；9个test attributes、0 ignored；fingerprint `5d4f7869...7347c` |
| 合并范围 | 14 / 1,469 / 50,861 | 8个Rust、1,278行Rust；fingerprint `80fe164a...00b4cf` |
| 产品链 | static/linked/builtin catalog、App、core registry/frontend、ShaderAsset/readiness、native replay | E3逐selection -> handler -> IR/asset -> artifact/PSO链 |

本轮确认shader frontend存在core、新WGSL package与旧family三份owner。Static inventory同时收录两包，actual first-party provider catalog却没有依赖或registration分支；默认core直接注册GLSL/SPIR-V Function handler，只为WGSL注册缺包诊断。旧family又故意不发布WGSL capability，却注册要求该capability的WGSL Function handler；registry不执行required capability admission，所以仍自报Available并可运行。

两份native dist均只有零systems的`runtime.asset.importer.shader` extension声明，没有source/settings/outcome bridge，当前replay会返回空成功。source frontend虽真实调用Naga，但所有WGSL/GLSL/SPIR-V/compute固定写Surface，reflection/resources/pipeline layout/import graph为空；readiness不要求entry point或layout。`Capabilities::all()`与generic GLSL静默回退Vertex又使target不支持或stage猜错的shader在导入后晚期失败。

对照Unreal ShaderCompilerCore/Preprocessor/Job Cache、Bevy Shader/ShaderCache、Godot ShaderLanguage/Compiler、Fyrox Shader resource和Unity Graphics stripping/report，本轮建立唯一frontend owner、SourceGraph/IR/reflection/target artifact、bounded compiler operation、native compile ABI与Editor/cook/runtime分层路线。登记5项P0、54项P1、12项P2、M0-M5及12项资格门；详细见`zircon_plugins/05-shader-wgsl-family-importer-compiler-artifact-native-product-integration-review.md`。

本轮没有修改production、tests、manifest或CI，也没有声明Cargo/GPU动态通过；16个包内test attributes不能覆盖默认产品不可达、capability admission、target artifact和像素行为。

## 100. WOC Product Role / ZrVM Transaction / World State / Client-Server Integration 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| WOC全物理目录 | 2,416 / - / 98,353,987 | E1全量inventory；1,967 tracked、449 ignored、0 normal-untracked |
| native workspace | 8 members / 约46,406行Rust / 1,505,575 bytes | E3逐manifest、production、role entry、transaction/protocol/projection/parity；513 test attributes |
| ZrVM source | 817 `.zr` / 246,765 / 9,978,430 | E2全量量化、E3入口/import/state纵向；149 tracked+29 ignored module从main可达 |
| `world/state.zr` | 1 / 68,730 / 3,298,740 | E3 writer/decoder/fixedTick owner；538 imports、1,691顶层函数 |
| Client/Server/Bot/Headless | 118 physical files | E3确认四个main均为8行identity reporter，library models没有产品host caller |
| 参考引擎 | Unreal Launch/GameInstance/NetDriver；Bevy App/Fixed；Godot Main/SceneTree/Multiplayer；Fyrox Executor/Plugin；Unity Graphics pipeline | 按host/schedule/network/plugin/render submit职责路由，不外推本地参考缺失域 |

本轮确认WOC有大量可保留typed protocol、projection validation、fixed-tick wrapper和golden provenance基础，但它们没有组成可运行产品：

- 根`examples/*`隐藏449个文件；tracked入口graph依赖29个ignored本地Zr module，clean clone不可重建；
- native `cargo test --workspace`在132.6秒后因`woc_protocol` 6个compile error失败，513个测试没有开始；验证产生的`Cargo.lock`改动已恢复；
- Client/Server/Bot/Headless四个binary只调用`identity_report_json()`并退出；没有窗口、loop、scene、render/UI/audio/input、network、service、persistence或shutdown owner；
- 仓内没有production `WocProjectVm`实现，`woc_runtime`也没有Zircon runtime/interface/plugin SDK registration，project selection没有materialize真实provider；
- native protocol、Zr schema、current writer和reader分别为WOS83/WOS113/118/117，默认writer输出会被同文件decoder拒绝；
- `fixed_tick(&mut self)`可先修改VM，wrapper之后才做budget/output/projection校验，接口没有rollback token/snapshot restore；Zr saveState又不对应真实world state；
- 每tick全量clone/encode最多64MiB state并解析最多16MiB JSON presentation；budget只检查VM自报usage，不能interrupt挂死或谎报guest；
- Server/Client没有transport/replication/persistence，Auth effect把password/token放进Clone+Debug String；54-golden double-run又两次返回`expected.clone()`，未执行real VM。

对照参考引擎，本轮建立clean-clone/BuildSet、engine-owned ProductHost、ZrVM admission、qualified world schema、candidate/commit/rollback、paged/delta state、bounded binary projection、authoritative server、retained client与real parity runner路线。登记9项P0、66项P1、14项P2、M0-M5及18项WOC产品资格门；详细见`zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md`。

本轮没有修改WOC production、test、manifest、asset或generated artifact；只新增review文档并更新索引/coverage/全局owner总账。Tooling05的WOC npm 157/148失败仍为独立codegen阻断，没有重复执行未变化lane。

## 101. WOC ZrVM Package Kernel / World State / Fixed Schedule / Serialization Runtime 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| WOC Zr source | 817 `.zr` / 246,765 / 9,978,430 | E2全量inventory；本篇聚焦package kernel、world/state、schedule和codec |
| `world/state.zr` | 1 / 68,730 / 3,298,740 | E3拆分production/test、field/import/function/loop/lookup、codec和fixed root |
| `kernel` | 13 / 3,004 / 93,814 | E3逐文件确认world/clock/entity/RNG与实际产品authority分叉 |
| fixed schedule | 51个直接阶段 / 35个直接全实体扫描阶段 | 41个`entityIds.length`命中、56个while、20个线性lookup，未计下层调用 |
| state codec | encode 2,200行 / decode 2,485行 | 548/546 primitive调用、92/217 version分支；单实体244个静态写入点 |
| tests in source | state尾部22,598行/204 tests；369个`*test_main.zr` | E3确认production module、deep self-test和test artifact未物理分离 |
| 参考引擎 | Unreal Tick/Mass；Bevy Schedule/ECS；Godot process groups；Fyrox Engine update；Unity Graphics RenderGraph | 按scheduler/storage/structural barrier/graph declaration边界路由，不把Graphics外推为Unity world runtime |

本轮确认WOC有可保留的generation identity、严格command sequence、integer authoritative time、state-owned RNG cursor、bounded collection和source-pinned generated catalog基础，但它们仍被单一world god authority包裹：

- `WorldState`有534个公开字段，其中325个是entity列、158个是offline字段；顶部直接导入109个模块，整文件有538个import表达式、1,691个顶层函数和728个`.indexOf(`；
- `fixedTick`完整decode state、复制command columns、串行调用51阶段、完整encode/digest/snapshot；没有system access、DAG、parallel executor、dirty frontier、partition、deadline或per-system telemetry；
- `partyEntityIndexById`是从0开始的O(N)扫描；全文件168个文本命中包含1处定义，即167个调用点。单实体编码固定部分下界约1,449 bytes，100,000 entity decoder cap与64MiB state envelope不自洽；
- protocol reader/writer和state writer在input/field/finish/snapshot间多次逐byte复制，`readF64LeAt`为8-byte读取复制整个payload并线性跳offset；
- 主entity rows没有通用remove/generation reuse；pet replacement只标记dead/inert，退役行永久进入325列、snapshot和后续扫描；
- `decodeState`在读完bytes后seed discovery、重算deed、补NPC、normalize title，普通decode不是纯snapshot restore；
- `kernel/entity`的despawn实现是私有AoS/线性查找测试孤岛，`kernel/world`只有常量/test，clock和RNG又形成独立authority；
- `world/state`最后22,598行是204个测试函数，package lifecycle self-test还导入23模块、调用19个contract test和完整world self-test。

对照参考引擎，本轮建立唯一WocPackageKernel、generation location + component chunk storage、structural command barrier、compiled fixed schedule、typed command outcome、paged/COW snapshot、纯decoder + migration DAG、qualified RNG substream、test artifact分离和规模/soak资格链。登记5项P0、60项P1、14项P2、M0-M5及14项runtime资格门；详细见`zircon_runtime/12-woc-zrvm-package-kernel-world-state-schedule-serialization-runtime-review.md`。

本轮没有修改WOC production、tests、manifest、generated artifact或lockfile，也没有重复native/npm已知失败lane。App03继续唯一拥有产品host、真实VM adapter和跨边界transaction；Tooling05继续唯一拥有codegen mixed-generation；本篇只拥有Zr内部world/storage/schedule/codec。

## 102. WOC Combat / Casting / Effect / Aura / Damage / Threat / Death Runtime 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| `src/combat`全量 | 197 `.zr` / 21,357 / 815,747 | E2物理inventory、E3逐owner/dispatch/state/parity纵向 |
| 非`*_test_main`模块 | 109 / 20,616 / - | 135 classes、1,467公开字段、120 throw、282 Array构造、277 while |
| 产品root可达combat | 54 / 10,198 / 383,731 | 从`src/main.zr`静态import BFS得到 |
| 不可达非test-main combat | 55 / 10,418 / 400,924 | 含damage/death/threat/aura/effect/Boss/affix候选owner |
| ability projection | 308 known / 117 M4 projected / 21带M4 scenario | 191 not projected不外推为全WOC必然不存在 |
| parity evidence | 54 manifest entries / 0个当前`woc_owner`存在 | golden可作oracle，不能记当前pass |
| 参考引擎 | Unreal GameplayAbilities；Bevy Schedule/Archetype/Fixed；Godot process/multiplayer；Fyrox lifecycle；Unity Graphics RenderGraph | 按ability lifecycle、prepared graph、schedule和extension边界路由 |

本轮确认combat目录已经保存大量可迁移职业、公式、effect、control、Boss和source-pinned contract知识，但产品authority与这些模块分裂：

- 109个非test-main模块只有54个从产品根可达；`ability_admission`、`damage_state`、`death_state`、`threat_state`、`effect_sequence_state`、`aura_state`、Nythraxis、Drowned Litany和mob affix等55个模块断线；
- `WorldState`的519字段中至少202个字段名呈现cast/projectile/dot/hot/absorb/aura/threat/power/crit/haste语义，生产段还有77个直接HP赋值点；该名称计数只是保守审计信号，不是正式component分类；
- `applySupportedCastSlotCommand` 616行/117能力条件，`applySupportedCastCommand` 780行/116 payload能力匹配，completion 219行/45分支，projectile landing 257行/33分支，四份手写dispatch无法证明同一能力生命周期一致；
- M4 generic projection仅117/308，117项中只有21项带M4 scenario；unknown/unsupported路径在throw、silent return和部分实现间没有typed outcome；
- 通用damage/death/threat/aura owner不可达，heal/auto attack/numeric effect主要经临时对象copy-in/copy-out，命中到死亡没有唯一atomic CombatTransaction或ordered journal；
- player death代码明确只实现single-player subset并推迟完整aura/pet/revenge/multiplayer retargeting，mob pursuit也推迟damage/threat/flee/leash/cast/boss mechanics；
- threat使用flattened CSR数组，插入/clear会移动全局rows并反复更新后续offset；RNG又由module cursor或手工random arrays混用，draw order成为隐式ABI；
- 99个非test-main模块把`contractTest`混在生产文件，test tail约4,808行/23.32%；88个test main没有形成当前combat aggregate执行证据；
- `parity_scenarios.json`的54个`woc_owner`全部不存在；native workspace仍在`woc_protocol` 6个compile errors处停止，测试没有开始。

对照参考引擎，本轮建立CombatBuildSet、prepared CombatProgram、Activation/ActiveEffect handle、typed target/admission/commit/cancel/end、atomic combat write-set与journal、统一damage/heal/aura/threat/death/control owner、Encounter extension、qualified RNG/time、逐能力coverage和可执行parity/performance资格链。登记6项P0、62项P1、15项P2、M0-M4及16项runtime资格门；详细见`zircon_runtime/13-woc-combat-casting-effect-aura-damage-threat-death-runtime-review.md`。

本轮没有修改WOC production、tests、manifest、generated artifact或lockfile，也没有重复native/npm已知失败lane。Runtime08G继续拥有通用Ability System，Runtime12拥有world storage/schedule/codec，App03拥有VM/host transaction；本篇只拥有WOC combat语义、产品接线与资格。

## 103. WOC Progression / Inventory / Item / Economy / Crafting / Quest / Talent Runtime 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| `src/progression`全量 | 101 `.zr` / 12,849 / 486,872 | E2物理inventory、E3逐owner/state/transaction/parity纵向 |
| 非`*_test_main`模块 | 55 / 12,082 / 457,689 | 67 classes、573公开字段、466 public functions、157 Array构造 |
| 普通fixed-tick可达progression | 15 / 2,505 / 98,583 | 以`world/state.zr`生产段import为根递归 |
| 普通fixed-tick不可达 | 40 / 9,577 / 359,106 | 含item ledger、bank/trade/market、quest/craft/talent核心候选owner |
| product M5 content | 82 items / 2 quests / 1 mob / 6 NPC / 3 talent options / 3 abilities / 2 specs | bounded projection，不代表current-head完整内容 |
| current-head item-level contract | 580 records / 395 source items / 331有item level | 不外推为580项都已inventory-ready |
| progression test/parity evidence | 46 test mains / 45 manifests / 0 checked-in binary dirs / 16相关owner缺失 | golden可作oracle，不能记当前pass |
| 参考引擎 | Unreal AssetManager/SaveGame/FastArray/Tags；Bevy Handle/Change/Schedule；Godot Resource/Multiplayer；Fyrox Visitor；Unity Graphics RenderGraph | 按identity、persistence、delta、transaction和schedule机制路由 |

本轮把“模块物理存在”“整个package因self-test可达”和“普通fixed-tick产品执行”分开，确认progression目录保留了大量可迁移current-head规则，但产品仍是有界scalar projection：

- 55个非test-main模块只有15个进入普通执行图；`inventory_instance_ledger`、bank、trade、market、通用quest/crafting transaction/gather/enchant/salvage、loot distribution、talent commit/migration、XP与deed等40个模块断线；
- WorldState 519个公开字段中至少100个名称带progression/economy信号；该计数只是保守名称审计，不是正式component taxonomy；
- 产品inventory只有flattened offsets/item codes/counts/manual slots，删除stack会搬移全局数组并更新全部后续entity offsets；实例payload明确因ZrVM structured persistence缺失而排除；
- 不可达ledger虽保存signer、rolled stats/masterwork、enchant、binding和charges，却没有进入WorldState codec、equipment、loot、bank、trade或market；
- `craft_item_state`只识别两份recipe，`ritual_vestments`因需要signed rolled instance明确unsupported，产品成功路径只剩minor healing potion；
- bank/trade/market/quest等module明确是未导入产品的source-contract fixture或局部平行数组，不能作为durable service、原子transaction或可重连在线能力；
- 产品quest只保留boars/wolves两组标量，并选择第一个player entity；talent提交只执行部分effect，完整candidate commit与world commit模块不可达；
- generated modifier/catalog使用数千到上万条if分支，modifier在多个战斗路径重复构造/线性扫描，未按allocation revision编译缓存；
- 46个test main对应45份manifest且没有当前checked-in binary目录，`stat_core_rules_test_main`无manifest；16个相关parity `woc_owner`全部缺失；
- native workspace仍在`woc_protocol` 6个既有compile errors处停止，测试未开始，本轮未重复无变化失败lane。

对照参考引擎，本轮建立ProgressionBuildSet、qualified ItemDefinition/ItemInstance/Container identity、ContainerStore、atomic ProgressionTransaction与receipt/outbox、durable vendor/bank/trade/market/loot/craft、typed Quest/XP/Talent runtime、prepared catalog和可执行save-reload/parity/performance资格链。登记6项P0、64项P1、15项P2、M0-M5及16项runtime资格门；详细见`zircon_runtime/14-woc-progression-inventory-item-economy-crafting-quest-talent-runtime-review.md`。

本轮没有修改WOC production、tests、manifest、generated artifact或lockfile，也没有重复native/npm已知失败lane。Runtime12拥有world storage/schedule/codec，Runtime13拥有combat credit，App03拥有VM/host outer transaction，Tooling05拥有generated artifact；本篇只拥有WOC progression identity、transaction、产品接线与资格。

## 104. WOC Social Identity / Party / Raid / Chat / Duel / Arena / Matchmaking / Minigame Runtime 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| `src/social`全量 | 60 `.zr` / 8,425 / 305,975 | E2物理inventory、E3逐owner/reducer/identity/parity纵向 |
| 非`*_test_main`模块 | 31 / 7,699 / 277,741 | 26 public classes、338 public vars、312 public functions |
| 产品可达social | 10 / 1,737 / 63,448 | Card Duel八模块、chat wire、target marker |
| 产品不可达 | 21 / 5,962 / 214,293 | 完整chat/party/arena/duel/DF/Fiesta/Yumi/Vale Cup候选owner |
| WorldState social投影 | 36名称信号字段 / 168个`partyEntityIndexById`生产调用点 | raw entity-row authority，无account/presence owner |
| social capability差异 | SocialGraph 18 / Dungeon Finder 9 / Vale Cup 6 / DuelArena 6 | 仅Arena queue/leave和Card Duel部分接入；其余未实现或无投影 |
| social test/parity evidence | 29 test mains / 29 manifests / 0匹配checked-in artifacts / 10相关owner缺失 | static guard与fixture不能记产品pass |
| 参考引擎 | Unreal Party/Auth/Social/Presence/Lobbies/Chat；Godot Multiplayer；Bevy Entity/Message/Event；Fyrox Handle；Unity Graphics RenderGraph | 以identity、lifecycle、event、visibility、owner依赖路由 |

本轮把“领域文件存在”“协议/client intent存在”和“产品root可执行”分开，确认social目录保存了大量可迁移规则，但当前产品能力面和authority严重错位：

- 31个非test-main模块只有10个进入产品图，21个断线模块约占源码行数77.4%；party/chat/arena实际仍由WorldState平行数组和手写reducer拥有；
- Catalog与native client可构造18个SocialGraph、9个Dungeon Finder、6个Vale Cup及6个Duel/Arena命令；World只实现arena queue/leave，duel三命令、arena augment和前三组能力落入未实现；
- Chat除`/ready`/`/readycheck`外的非空合法消息会先消耗token，随后state-neutral并推进command sequence，没有delivery/rejection receipt；
- Party invitation、leader、member、queue和duel都使用raw entity ID；没有account principal、session、presence、connection generation、disconnect/reconnect或cross-world transfer；
- Party shared facts按entity row复制，ready-check与validation含嵌套扫描，marker命令恢复/提交全数组；arena只有admission row，fixed tick没有matcher drain；
- Card Duel确实进入command/fixed-tick图并保留queue/match/hand/snapshot/RNG基础，但coordinator `join`需要三参数、service只传两参数，typed command result又被World丢弃；
- Card Duel每个command batch和每个fixed tick无条件snapshot decode、双向player/entity同步、update、encode，queue pairing连续两次`removeAt(0)`；
- 29份manifest没有当前匹配的checked-in artifact，十个social parity `woc_owner`路径不存在；native/npm仍停在既有失败lane，本轮未重复无变化命令。

对照参考引擎，本轮建立OnlinePrincipalRegistry、PresenceRuntime、SocialGraphRuntime、PartyRuntime、ChatRuntime、MatchmakingRuntime、CompetitiveMatchRuntime、CardDuelRuntime、SocialEventOutbox、recipient-filtered projection和可执行privacy/fault/load/parity资格链。登记6项P0、66项P1、16项P2、M0-M5及16项runtime资格门；详细见`zircon_runtime/15-woc-social-identity-party-raid-chat-duel-arena-matchmaking-minigame-runtime-review.md`。

本轮没有修改WOC production、tests、manifest、generated artifact或lockfile，也没有重复native/npm已知失败lane。Runtime08E拥有transport/replication，Runtime12拥有world storage/schedule/codec，Runtime13/14拥有combat/reward transaction，App03拥有VM/host outer publish；本篇只拥有social identity、presence、party/chat/social graph、matchmaking/minigame产品接线与资格。

## 105. WOC Instance / Dungeon / Delve / Pet / Companion / Lockout / Reset / Collision Runtime 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| `src/instances`全量 | 103 `.zr` / 10,993 / 410,346 | E2物理inventory、E3逐owner/reducer/lifecycle/parity纵向 |
| 非`*_test_main`模块 | 57 / 9,939 / 359,339 | 14 classes、264 public vars、532 public functions |
| `main.zr`静态closure | 5 / 484 / 18,063 | 含仅被package contract test消费的`heroic_dungeon_tuning` |
| 普通WorldState产品可达 | 4 / 350 / 12,740 | Emberkin ranged、pet target/follow、Delve companion content规则叶 |
| 普通产品不可达 | 53 / 9,589 / 346,599 | 完整Dungeon/Delve/lockpick/reset/PetState候选owner |
| 相关`src/world`非test-main | 17 / 16,450 / 428,372 | 产品只接4文件/4,317行；Delve collision/layout/LOS等13文件断线 |
| capability差异 | Pet 11 / Dungeons 4 / Delves 10 | 4 pet、2 dungeon、8 delve命令无reducer；另有pet no-op/forced-false |
| test/parity evidence | 46 test mains / 39有manifest / 38 M7 binary dirs缺失 / 11 owner缺失 | missing/not-run不能记pass |
| 参考引擎 | Unreal World/GameMode/PlayerState/NetDriver；Godot Multiplayer；Bevy World/Entity；Fyrox Scene/Pool；Unity Graphics RenderGraph | 以world scope、identity、travel、spawn/despawn、resource lifetime路由 |

本轮把“规则投影存在”“进入package closure”“进入普通产品authority”“对外Supported”四层拆开，确认实例目录保存了有价值的规则素材，但当前产品不是工程级instance runtime：

- 普通产品路径只到达4/57个非test-main模块；`DelveState`、`DungeonState`、`DungeonResetState`、`PetState`、lockpick session、Drowned Litany和M7 matrix均不可达；
- WorldState没有InstanceId、allocator、claim、membership、admission lease、isolated clock/RNG/schedule、placement、transfer、reconnect或shutdown owner；
- standard dungeon/arena/Yumi通过X/Z坐标带选择静态layout，standard band未知X回退crypt，active Delve/far-east因布局未移植抛错；
- current-head Catalog公开11个Pet、4个Dungeon、10个Delve命令，产品缺14个reducer；七个pet route又仅服务offline Emberkin，其中四个是no-op或强制false；
- Delve产品只做shop buy/companion upgrade，Dungeon只做difficulty/heroic vendor；不存在可进入、运行、退出、重连和恢复的run；
- `delve_state`以77字段和八组整数event数组混合run/layout/boss/lockpick/reward/test，`dungeon_reset_state`只在内存平行数组中替换claim/lock，不具备crash atomicity；
- pet fixed tick执行pet×entity全扫描与直线open-ground移动，没有persistent PetId、stable/spellbook、navigation、despawn和save/load生命周期；
- 46个test main中七个无manifest，38个M7 package没有binary目录，11个current-head parity owner路径不存在，实例文件又混用两个source commit且95个文件无同等provenance。

对照参考引擎，本轮建立InstanceDefinitionRegistry、InstanceAllocator、InstanceWorldRuntime、InstanceClaimStore、InstanceTransferRuntime、DungeonRuntime、DelveRuntime、EncounterRuntime、PetRuntime、CompanionRuntime、InstanceRewardTransaction、recipient-filtered projection和产品root parity/fault/load资格链。登记6项P0、68项P1、16项P2、M0-M6及16项runtime资格门；详细见`zircon_runtime/16-woc-instance-dungeon-delve-pet-companion-lockout-reset-collision-runtime-review.md`。

本轮没有修改WOC production、tests、manifest、generated artifact或lockfile，也没有重复native/npm已知失败lane。Runtime05/08D/08E/12/13/14/15分别拥有world lifecycle、navigation、network、codec、combat、transaction和principal/party；本篇只拥有instance allocation/claim/transfer、dungeon/delve/pet/companion产品接线与资格。

## 106. WOC World / Terrain / Collision / Locomotion / Spawn / Spatial / Targeting Runtime 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| `src/world`全量 | 208 `.zr` / 102,546 / 4,245,497 | E2物理inventory、E3逐owner/reachability/hot-path/evidence纵向 |
| 非test-main | 67 / 100,755 / - | Runtime12拥有68,730行`state.zr`；Runtime16拥有17个instance helper |
| 本篇非重复主体 | 49 / 15,575 / 455,226 | 30 classes、179 public vars、397 public functions、2,916个`if` |
| main静态closure可达 | 35 / 13,080 / - | terrain/collision/player-mob motion/target/lifecycle产品路径 |
| 不可达candidate | 14 / 2,495 / - | Pathfind、SpatialGrid、Interaction、Corpse、WorldBoss、Roster/Placement等 |
| collision热路径 | 170 static colliders / 3 passes / 0.2 movement step | axis-return调用内部最坏约1,020检查；caller取X/Z约2,040/段 |
| generated content | collision 4,210行/1,389 if；terrain 3,178行/808 if | 无definition/partition/cooked tile/generation/residency |
| world evidence | 141 test mains / 5无manifest；74 world-entry packages / 64 binary dirs缺失 | 六个current parity owner缺失；66/67非test-main无source commit |
| 参考引擎 | Unreal World/Partition/Collision/Nav；Godot World3D/PhysicsSpace/NavAgent；Bevy/Fyrox World/Graph/Pool/Navmesh；Unity Graphics RenderGraph | 按world generation、partition、query、navigation、resource lifetime路由 |

本轮将world物理目录、产品静态闭包、断线candidate和通用Runtime owner分开，确认WOC已有可迁移的terrain/collision/spawn规则，但没有形成工程级world runtime：

- 49个非重复模块中14个不在产品闭包；`SpatialGridState`、`PathfindState`、`InteractionSelection`、corpse rights、world boss participation和future roster/materializer均只有测试或断线consumer；
- `world_collision_router`每0.2单位采样，开放世界每个坐标最多三遍扫170 collider；内部先求X/Z，caller又按axis重跑整条pair结果，最坏约2,040次collider检查/采样段；
- 16-yard `collision_grid`查询时仍遍历全部170 static collider，只用于safe-position，不在player/mob主移动route；
- terrain/content是3,178行标量accessor与即时height/gradient/ground组合，没有WorldDefinition、tile/chunk、partition、streaming、LOD、cook/install generation或跨collision/nav/render同代关系；
- `PathfindState`每query重建1-yard局部网格和六列scratch，64-cell上限或无路时退回直线；产品mob/pet只做direct step/七向slide fan；
- aggro、target、pet/mob query仍按实体数组全扫，target command分配多列candidate并做insertion sort；断线SpatialGrid不能计入产品性能；
- spawn候选源码明确自称scalar projection/future materializer，缺SpawnId、atomic materialization、despawn/respawn、cell lifecycle和rollback；
- 141个world test main中五个无manifest，74个world-entry package有64个binary目录缺失，六个current parity owner不存在，只有`terrain_noise`携两个冲突source revision。

对照参考引擎，本轮建立WorldDefinitionRegistry、WorldPartitionRuntime、TerrainRuntime、StaticCollisionRuntime、MovementRuntime、SpatialQueryRuntime、SpawnRuntime、InteractionRuntime、WorldEncounterRuntime、recipient-filtered projection和产品root evidence路线。登记6项P0、68项P1、16项P2、M0-M6及16项runtime资格门；详细见`zircon_runtime/17-woc-world-terrain-collision-locomotion-spawn-spatial-targeting-runtime-review.md`。

本轮没有修改WOC production、tests、manifest、generated artifact或lockfile，也没有重复native/npm已知失败lane。Runtime05/08A/08D/08F/12/13/14/15/16分别拥有通用world、physics、navigation、AI、codec、combat、loot、principal和instance边界；本篇只拥有WOC world definition/partition消费、terrain/collision/movement/spatial/spawn/interaction产品接线与资格。

## 107. WOC Generated Content / Catalog / BuildSet / Install / Query Runtime 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| `src/generated`全量 | 102 `.zr` / 81,093 / 3,362,552 | E2物理inventory、E3逐artifact/reachability/query/provenance纵向 |
| generated非test-main | 101 / 81,064 / 3,361,294 | 105 public vars、1,098 public functions、3,382 throws、约58,229个非测试`if` |
| `src/content`全量/非测试 | 10 / 616 / 22,698；非测试3 / 376 / 13,218 | 三个非测试facade均不在产品closure |
| 本篇非测试主体 | 104 / 81,440 / 3,374,512 | 46 generated可达；58个content/generated候选不可达 |
| generated分类 | catalog 7；contract 78；milestone 14；test 1；wire/meta 2 | 仅46/102进入`main.zr`静态closure |
| 七大catalog | 48,466行 / 1,916,502 bytes / 35,044 if / 2,754 throws | ability/effect/talent/modifier/proc/content热查询未prepare/index |
| active投影 | 308 known abilities、117 M4 abilities、162 talent options、189 modifiers、55 procs、82 M5 items | 无aggregate ContentBuildSet和cross-catalog support closure |
| provenance/schema | 92/102携同一source commit；21文件有局部64-char digest | `stateSchema()`不包含catalog digests、tool/target/dependency identity |
| content evidence | 7 test mains / 3无manifest；5相关binary dirs缺失 | M4/M5 expected SHA与21/14计数被当前117/82静态推翻 |
| 参考引擎 | Unreal DataTable/DataRegistry/AssetManager；Godot Resource/UID/Loader；Bevy Assets/Handle/Server；Fyrox ResourceManager；Unity Graphics resource registry | 按typed identity、load state、generation、registry与lifetime路由 |

本轮将generator producer职责与runtime consumer职责分开，确认WOC生成内容已有真实规模和可迁移数据，但没有形成工程级内容运行时：

- 104个非测试主体中只有46个generated文件进入产品closure；三个content facade与55个generated candidate断线，WorldState则绕过facade直接import具体M3/M4/M5/M8/current模块；
- 81K行generated源码把数据、key/field/rank/effect选择和错误策略编成约58K个条件分支，`talent_modifier_catalog`与`m4_ability_effects`都超过13K行；
- product同时消费308项known ability与117项M4 definition/effect、162项talent option、189项modifier、55项proc和M3/M5/M8分期projection，但差集没有Implemented/Unsupported/owner/test状态；
- 21份局部catalog/schema digest没有进入`main.stateSchema()`，该schema只组合protocol contract、command catalog/payload、WOS113与20/60 Hz；
- generated没有typed immutable table/index、generation-qualified handle、ContentSnapshot、load/prepare/activate/retire、hot reload/rollback或save/network/replay compatibility；
- `talent_modifier_state`按189 entries与字符串字段线性访问，proc路径逐次构造definition/response；真实catalog规模下的VM parse/startup/query/allocation成本没有资格证据；
- M4 test仍期待旧SHA和21项而当前为新SHA/117项，M5 test期待旧SHA/14 items而当前为新SHA/82 items；三test无manifest、五binary目录缺失；
- native/npm仍停在已记录的既有失败lane，本轮未重复无变化命令，也没有把未执行的后续check计为pass。

对照参考引擎，本轮建立ContentSchemaRegistry、ContentBuildSetRegistry、ContentArtifactLoader、ContentValidationRuntime、ContentRegistry、ContentQueryRuntime、ContentCompatibilityRegistry、ContentActivationCoordinator、ContentReferenceResolver、ContentProjectionRuntime与ContentEvidenceRunner。登记6项P0、68项P1、16项P2、M0-M6及16项runtime资格门；详细见`zircon_runtime/18-woc-generated-content-catalog-buildset-install-query-runtime-review.md`。

本轮没有修改WOC production、tests、manifest、generated artifact或lockfile。Tooling05拥有generator graph/atomic publication，Runtime04拥有通用artifact IO/residency，Runtime12–17拥有world与业务语义；本篇只拥有runtime内容BuildSet、安装代际、typed query、compatibility、activation与qualification。

## 108. WOC Command Protocol / Payload Codec / Admission / Movement / Outcome Runtime 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Zr `src/protocol`非test-main | 4 / 2,643 / 109,929 | `binary`/`commands`/`command_payloads`产品可达；57行movement helper仅测试可达 |
| native `woc_protocol/src` | 39 / 11,713 / 378,644 | 29 tracked、10 required source被`.gitignore`忽略；当前物理树仍6 compile errors |
| native integration tests | 4 / 3,432 / 113,411 | 76 test attributes；compile阻断导致0运行 |
| command catalog | 165连续ID / 156 client-send / 9 dispatch-only | 21 facets；35命令无facet；script known只判断`id < 165` |
| typed payload contract | 157 commands / 61 kinds / 97 fixed / 60 variable / 47 empty | 8个dispatch-only ID未映射；descriptor lookup线性扫描157项 |
| protocol artifacts | 主bin有2个protocol `.zro` | `command_payloads.zro`、`movement_input.zro`与movement test binary缺失 |
| wire/budget | 44-byte native frame header；128 MiB frame、64 MiB state、4,096 commands、65,536 movement | 多轮frame/state/payload/parallel-array copy，无aggregate work资格 |
| 参考引擎 | Unreal Iris NetSerializer/DataStream；Godot marshalls/PacketPeer/MultiplayerAPI；Bevy Message；Fyrox Visitor；Unity Graphics generation registry | 按versioned codec、validation、peer authority、delivery、generation与lifetime路由 |

本轮从client intent一直追到VM/WorldState/outcome，确认协议拥有不少可保留的length、canonical、finite与typed payload检查，但入口仍然分裂：

- client mapper调用`validate_command_payload`与`require_client_send`，而public raw `Command`、`Command::decode_payload`、FixedTick decode和host transaction都可绕过command-specific validator；
- Zr binary只验证known ID与全局64 KiB，WorldState再查kind/min/max/fixed length并在68K行类中手工解析、校验actor sequence和直接mutation；
- native frame只携core fingerprint，script command identity又截断到16 hex字符；catalog/payload/WOS83-WOS113/ContentBuildSet不能证明同代；
- command失败只有throw/whole candidate rollback或`RejectedCommand { index, reason }`，没有per-command accepted/rejected/duplicate/stale/deferred receipt；
- movement relay把sequence作为ACK高水位却应用所有valid packet，较旧packet仍覆盖flags/facing；key无world/session/connection且没有teardown API；
- outer frame/FixedTick/Zr reader会重复复制，Zr f64读取按字段重建reader并以最坏约1,074次循环求幂；合法最大输入没有CPU/allocation/latency/VM instruction证据；
- default npm check先因148/157旧计数冲突停止，native compile失败，movement artifact缺失；现有test数量和source-shape检查不能记为产品pass。

对照参考引擎，本轮建立ProtocolSchemaRegistry、CommandCodecRegistry、CommandBatchDecoder、CommandAdmissionRuntime、CommandDispatchRegistry、CommandOutcomeJournal、MovementInputRuntime与ProtocolEvidenceRunner。登记6项P0、68项P1、16项P2、M0-M6及16项runtime资格门；详细见`zircon_runtime/19-woc-command-protocol-payload-codec-admission-movement-outcome-runtime-review.md`。

本轮没有修改WOC production、tests、manifest、generated artifact或lockfile，也没有重跑未变化的失败lane。Runtime08E拥有transport/security，Runtime12拥有world/schedule，Runtime13–17拥有domain reducer，Runtime18拥有ContentBuildSet，App03拥有product host，Tooling05/10拥有generator/test runner；本篇只拥有command schema/codec/admission/dispatch/outcome与movement input transport state。

## 109. WOC Oracle / Trace / Golden / Differential Replay / Evidence 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| native `woc_parity` production | 6 / 2,070 / 66,792 | canonical、RNG、golden、WTR decoder均有实现；无产品caller |
| native parity tests | 3 / 351 / 11,311 | 12个test；double-run actual是`expected.clone()`，workspace compile阻断导致0执行 |
| Zr parity | 14 / 2,812 / 110,327 | 四条M3 trace+wire fixture；只有4个dump binary，4个test binary均缺失 |
| current-head catalog | 54场景 / 54 reference owner / 54 WOC owner | reference文件54/54存在；WOC owner 0/54存在；materialize删除owner字段 |
| reference golden | 54 JSON / 2,538,832 bytes / 104,070行 | 793 frames、296 full frames、251 coverage refs；静态物化/hash通过 |
| trace dictionary | 1,070唯一symbol | 完整SHA-256只截取60 bit进入WTR fingerprint；三端codegen静态通过 |
| reference actual probe | 1次`entity_roster`动态运行 | 固定commit manifest与working-tree import分叉，首帧/状态/digest漂移后失败 |
| 参考引擎 | Unreal Automation/Replay/Trace；Bevy Runner/Stepping；Godot Profiler；Fyrox Visitor；Unity Graphics audits | 按result/artifact、checkpoint/replay、session/schema、真实产物观察路由 |

本轮确认参考资产自洽、kernel fixture稳定和产品parity是三类不同证据：

- `current_head_parity_materialize --check`确认固定commit的54份golden bytes，`trace_symbol_codegen --check`确认1,070项字典输出；二者均不能证明Zircon actual；
- native `GoldenSuite`、`compare_double_run`和`decode_vm_trace`除crate自身tests外无consumer，唯一double-run closure直接clone expected；
- source catalog为54项声明的`scripts/woc_game/tests/parity/*.zr`全部不存在，native投影又删除`woc_owner`，无法执行owner完整性门；
- lifecycle/locomotion/roster trace各只调用一次kernel contract，其余由1,039-1,756个数字字面量写出golden；targeting只把少量metric填入固定模板；
- reference materializer从`git show <commit>`读取，probe直接import当前参考工作树；HEAD相同但dirty source仍导致实际probe失败，oracle执行不具hermetic identity；
- materialize/probe/WTR encode/verify不在默认package check，native/npm又有既有上游阻断；没有54/54 source/build/schema-bound ValidationSet或release admission。

本轮建立WocParityCatalogCompiler、WocReferenceOracleAdapter、WocActualTraceRunner、WocTraceSchemaRegistry、WocDifferentialEngine、WocGoldenRepository/PromotionCoordinator与WocParityTestAdapter。登记6项P0、68项P1、16项P2、M0-M6及18项资格门；详细见`zircon_tooling/11-woc-parity-oracle-trace-golden-differential-replay-evidence-review.md`。

本轮没有修改WOC production、tests、manifest、golden、generated artifact或参考仓源码；只执行两条静态check和一次写入前即失败的临时reference probe。App03拥有真实ProductHost，Runtime13-19拥有业务authority，Tooling05/10/09分别拥有codegen、TestPlan与release；本篇只拥有oracle/actual/differential/golden/replay evidence链。

## 110. WOC Native Client Window / Input / Shell / UI / Presentation Frame 产品集成物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| client production | 61 Rust / 11,635 / 361,156 | 逐文件E3；binary不引用library，composition没有window/settings/device/backend |
| client tests | 47 Rust / 11,398 / 376,405 | 355个test；无binary/window/GPU/network/async/race/fuzz/benchmark，workspace compile阻断导致0执行 |
| input | 20 / 4,010 / 127,569 | 151 intents、61 key actions、gamepad/touch helper；无OS device adapter/action context |
| preferences | 10 / 2,007 / 58,457 | 43 numeric + 41 bool；29/84 application route为空，其余symbolic effect无consumer |
| presentation | 10 / 412 / 12,534 | fixed accumulator/timeline骨架；无render submit/present/network jitter/pacing host |
| shell | 13 / 3,765 / 118,360 | auth/realm/character/offline纯状态；无secure transport/request generation |
| windows | 5 / 1,199 / 36,875 | inventory/quest/settings view model；无retained tree/focus/layout/paint |
| product callers | 关键window/settings/keybind/gamepad/welcome/graphics-budget owner外部命中0 | host effect和settings application都没有native production consumer |
| 参考引擎 | Unreal Launch/GameEngine/Slate/UserSettings；Bevy Winit/Input/Render；Fyrox Executor/Engine/Renderer；Godot Main/Display/Input/Audio；Unity Graphics DynamicResolution/RenderGraph | 按真实runner、device lifecycle、apply/confirm、present与backend owner路由 |

本轮把大量pure model与native client产品能力分开，确认以下硬缺口：

- `main.rs`只有8行，调用`identity_report_json`后退出；61个production source不在binary调用闭包，manifest也没有window/GPU/UI/audio/network/async backend；
- `WocClientSession`只组合shell/HUD/command mapper/frame driver，`ClientWindowController`、settings/keybind/gamepad、welcome/inventory/graphics budget均未进入composition；
- frame driver在authority成功后才push timeline，push失败仍扣accumulator、清commands、推进movement sequence并返回error，形成部分提交；
- auth/realm/character effect没有operation/session generation、deadline/cancel/TLS endpoint identity，认证effect的Debug还会包含password/token/2FA/recovery code；
- 84项setting有29项application为空，其余只返回无人消费的Audio/Renderer/Fullscreen/CSS等symbolic route；preference read/write又吞掉错误、pending和durability语义；
- inventory view每次重建HashMap并clone item/visible/cells，stale click存在ABA，unknown content静默不显示；window model没有focus/z-order/modal/DPI/accessibility；
- authored 355 tests没有启动binary或真实OS/GPU/network设备，native workspace仍停在既有6个protocol compile errors，不能计client acceptance。

本轮建立ClientProductHost、ClientFrameTransaction、InputActionMap/InputDeviceEvent、OnlineOperationEnvelope、ClientSettingsDocument/ApplyReceipt、WindowWorkspaceSnapshot、RenderExtractionSnapshot与PresentReceipt路线。登记5项P0、88项P1、16项P2、M0-M6及20项资格门；详细见`zircon_app/04-woc-native-client-window-input-shell-ui-presentation-frame-product-integration-review.md`。

本轮只新增review与索引，没有修改WOC production、tests、manifest、asset、generated artifact或lockfile，也没有重跑未变化的native/npm失败lane。App03继续拥有四角色ProductHost/真实VM/outer transaction，Runtime11a-c拥有UI/Text/GPU UI，Runtime19拥有protocol/movement；本篇只拥有native client入口、window/input/settings/shell安全、presentation/present loop。

## 111. WOC Native Server / Bot / Headless Service Tick / Replication / Persistence / Operations 产品集成物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| 三角色production Rust | 5 / 286 / 9,827 | server 3文件、bot/headless各1；三个binary均只打印identity |
| 三角色manifest | 3 / 28 / 632 | server只依赖protocol/runtime；bot/headless只依赖runtime |
| server tests | 1 / 201 / 6,854 | 6个同步test；全部使用`RecordingVm`，workspace compile阻断导致0运行 |
| Cargo target graph | server lib+bin+integration test；bot/headless各1 bin | server binary不引用同package library；driver无产品caller |
| fixed scheduler | 259行；20 Hz、3项非零budget、catch-up cap | 局部canonical/validate/backlog骨架可保留，无host clock/service loop |
| service integration | network/async/persistence/replication/health/admin/signal/metrics有效实现命中0 | 无principal/session/world、journal、commit publication或shutdown owner |
| bot/headless | 各8行main | 无policy/observation/action/episode或snapshot/step/result workload |
| 参考引擎 | Unreal Launch/NetDriver/World；Bevy ScheduleRunner/Fixed；Fyrox headless Executor；Godot Main/SceneTree/Multiplayer；Unity Graphics RenderGraph capability boundary | 按持续runner、fixed phase、dispatch/flush/shutdown、resource/capability lifetime路由 |

本轮在App03四角色总边界之下继续追踪service owner，确认fixed driver即使接上socket也不能成为工程级authority：

- `advance()`在VM transaction前`mem::take`并清空queue/dedupe set；fault batch不回队，只留在无journal/receipt/replay API的`last_failed_input`；
- fault后的下一次`advance()`会以空batch触发`SessionNotRunning`并覆盖原diagnostic，输入的拒绝、重放或提交结果均无法恢复；
- enqueue public API接收无principal、connection generation、world/shard与actor lease的裸command/movement；pending dedupe跨tick即消失；
- success只替换内存`CommittedSnapshot`，没有把command outcome、replication ACK、event journal、checkpoint与durability绑定同一commit generation；
- queue只按item计数，constructor可按任意`usize`立即预分配；aggregate bytes、decode/sort/clone work、per-principal share和process memory不在producer admission前限制；
- Bot/Headless与Server统一映射`server_runtime`却没有独立runner/capability，不能证明remote agent、local deterministic simulation或offscreen job语义；
- 没有binary/process、network/storage/fault/recovery/race/fuzz/benchmark/soak/packaged artifact证据；既有native compile blocker未因source无变化而重复执行。

本轮建立ServerProductHost、ServerClockScheduler、ServerIngressCoordinator、AuthorityCommitCoordinator、ReplicationCoordinator、ServerPersistenceCoordinator、ServerOperationsHost、BotProductHost与HeadlessSimulationHost路线。登记4项P0、72项P1、16项P2、M0-M6及20项资格门；详细见`zircon_app/05-woc-native-server-bot-headless-service-tick-replication-persistence-operations-product-integration-review.md`。

本轮只新增review与索引，没有修改WOC production、tests、manifest、lockfile或generated artifact。App03继续拥有四角色ProductHost/真实VM/outer transaction，Runtime08E拥有transport/security，Runtime12-18拥有world/content authority，Runtime19拥有protocol/admission/outcome；本篇只拥有native server service、clock/queue/fault recovery、publication/durability orchestration、operations与bot/headless runner。

## 112. WOC Package Root / World API Facet Registry / Snapshot / Command Publication 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| package root source | `main.zr` 160/5,212；`identity.zr` 7/106 | lifecycle进入WorldState；normal产品不import/register World API |
| World API production | 1文件 / 174 / 5,702 | 只有Card helper；catalog公开合同0/5匹配 |
| World API test | 1 test main / 79 / 3,414 | 唯一consumer；`.zrp`声明target但binary目录缺失 |
| current-head catalog | 28 facet / 248 member / 2,436行 / 85,354 bytes | 181 method、67 data；186 simulation、56 service、6 presentation |
| owner materialization | 28个唯一`woc_owner` | 1存在、27缺失；不能以空stub推进完成率 |
| Card public contract | 1 data + 4 method | command ID 90..93已有；facet facade、snapshot与outcome binding缺失 |
| Rust validator | 699行 / 20,851 bytes | 只读name/kind/ownership；忽略signature/source owner/WOC owner/member_count |
| inventory tests | 205行 / 7,255 bytes / 7 tests | 验证reference identity/count；不执行owner/implementation/artifact |
| root `bin` | 8文件 / 1,382,792 bytes | manifest登记6 module；18个artifact路径仅6存在，漏World API与已用module |
| 参考引擎 | Unreal UWorld/WorldSubsystem；Bevy World/Commands；Fyrox plugin context；Godot ClassDB/SceneTree/MultiplayerAPI；Unity Graphics RenderGraph lifetime | 按per-world identity、registry、deferred command、capability context与generation lifetime路由 |

本轮把reference reconstruction inventory、World API schema、运行时facet capability与产品artifact readiness分开，确认：

- catalog内部计数与ownership自洽，但28个facet owner只有Card一个物理存在；Rust validator不检查路径、签名、实现、导出或BuildSet；
- 唯一Card owner导出DTO与projection helper，没有实现`cardMinigameInfo`和四个catalog method，根产品也没有注册它；
- Card command已经在protocol/payload/reducer链出现，缺口是typed facade、principal-scoped snapshot、generation与outcome publication，不是重复实现规则；
- `infoFromService`接受任意pid、mutable service和外部opponentName，可读取任意player的私有hand，且getter序列没有commit generation fence；
- `buildView`把simulation DTO与presentation混在同owner，并把所有非waiting card标为playable，无法表达turn/generation/admission/outcome；
- 根manifest使用本机绝对路径，引用12个不存在的zri/AOT C文件，漏`protocol/command_payloads`、`kernel/rng`和World API；Card test artifact也缺失；
- reference test通过只能证明pinned JSON未漂移，不能记为任何facet可用或产品parity。

本轮建立WorldApiSchemaRegistry、per-world WorldApiRegistry、PrincipalScopedWorldReadSnapshot、WorldCommandPort、WorldApiPublication与WorldApiArtifactReceipt路线。登记5项P0、68项P1、16项P2、M0-M6及20项资格门；详细见`zircon_runtime/20-woc-package-root-world-api-facet-registry-snapshot-command-publication-review.md`。

本轮没有修改WOC production、tests、manifest、generated artifact或reference数据，也没有重跑未变化的native/npm失败lane。Runtime15拥有Card Duel规则，Runtime19拥有protocol/admission/outcome，Runtime12拥有package/world，App03拥有VM/host，Tooling05拥有通用生成；本篇只拥有facet registry、snapshot/privacy、typed facade、publication generation与API artifact readiness。

## 113. First-Party Plugin Source / Editor / Runtime / Dist Catalog / Profile / Capability Closure 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| plugin workspace | 139 package / 139 member / 162 target | `cargo metadata --no-deps`成功；98 lib、41 cdylib、20 test、2 custom-build、1 bin |
| generated package manifests | 39 / 3,230行 / 104,490 bytes | 54 runtime、42 editor、39 native module row；结构审计0 violation |
| runtime provider source | 53 entry / 4,822行 / 177,985 bytes | 30个package声明runtime；通用catalog只链接14个，缺16个 |
| editor provider source | 40 entry / 4,413行 / 166,644 bytes | 25个package声明editor；通用catalog只链接Navigation/Neural，缺23个 |
| dist projection | 39 `lib.rs` / 4,248行 / 162,941 bytes / 79 tests | ABI/行为壳归Plugins01；本篇只核对声明、source、catalog、profile与packaging closure |
| runtime profiles | Client2D/3D、Editor、Dev要求Sound/Rendering | 对应App feature不链接base runtime catalog；profile required closure不自洽 |
| selection resolution | runtime/editor catalog均返回裸`Vec<RegistrationReport>` | invalid/missing provider直接`continue`，required/optional outcome丢失 |
| capability status | 35 partial / 3 complete | status未绑定compiled provider、product caller、artifact、generation或EvidenceSet |
| 参考引擎 | Unreal PluginManager/ModuleDescriptor；Godot extension manager/loader；Bevy PluginGroup；Fyrox static/dylib plugin；Unity Graphics package/asmdef | 按required configure、显式lifecycle、确定性group、包装形态与compiled closure路由 |

本轮把manifest schema完整、compiled provider、产品路由、profile feature closure、包装形态与qualified capability分开，确认：

- 39份manifest与139包workspace结构真实且结构审计绿色，但该审计不比较30/25个声明package与14/2个产品catalog路由；
- runtime缺口覆盖Physics、Terrain、Prefab、Tilemap与十类importer等16包；这些包有真实registration或业务入口，却无法由通用App source catalog返回；
- editor缺口覆盖AI、Physics、Sound、Rendering、Terrain、UI Asset等23包；真实toolkit、command、view或overlay不会由默认Editor catalog materialize；
- Client2D/3D、Editor与Dev profile将Sound/Rendering列为required，App target features却不含`first-party-runtime-plugins`；现有绿色bootstrap test只在手工额外feature下编译；
- 两个catalog对invalid ID或missing compiled provider直接跳过，返回值不保留selection index、required、packaging、missing reason或逐项terminal outcome；
- generated export function-pointer provider是可保留的独立链，但手写App catalog、generated export和native discovery没有共享resolution receipt与行为资格矩阵；
- `complete`和`classified-and-clear`只证明声明或结构，不能证明标准产品可启动provider、跨包装同语义或达到性能/表现目标。

本轮建立ProviderBuildMatrixCompiler、ProductPluginResolver、PluginResolutionReceipt、PluginRegistrationTransaction与CapabilityQualification路线。登记5项P0、72项P1、16项P2、M0-M6及20项资格门；详细见`zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md`。

新鲜只读验证为`cargo metadata --manifest-path zircon_plugins/Cargo.toml --no-deps --format-version 1`与`python tools/audit_plugin_structure.py --json`，二者均成功且lockfile未变化。本轮没有运行compile/test/startup，也没有修改production、tests、Cargo manifest、lockfile、generated manifest或dist artifact；既有Editor编译阻断未重复执行。

## 114. Vampire Roguelite Example Product / Asset / Script / Gameplay / Evidence 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| tracked sample | 173文件 / 8,880,831 bytes / 约11,458文本行 | 有真实综合场景，但根ignore规则使新增required source默认不入库 |
| project/meta | 1 project manifest / 52 `.zmeta` | meta均可解析且UUID唯一；本机cache不能替代source/artifact closure |
| scene | 110 entity / 90 mesh / 14 point light / 9 script binding | 只有player+3 enemy binding启用；无Boss、collision、physics或audio |
| models | 24 tracked GLB / 7 ignored model TOML | GLB容器有效；scene直接引用4份未tracked model source，clean clone断裂 |
| script package | 192行source + tracked `main.zro` | `onFixedUpdate`为空；CLI manifest含绝对路径并引用不存在zri/AOT文件 |
| gameplay data | balance + behavior tree | 无production consumer；移动、DPS、contact damage按variable update次数变化 |
| product tests | asset/import/extract + 10 real-VM behavior test | importer使用test-only fixture；10个行为测试全部ignore且新owner未接手 |
| visual/perf evidence | 34 PNG + 20文本 | 18文本为空、3组重复图、无sidecar；README缺2张accepted图且性能口径漂移 |
| 参考引擎 | Unreal Lyra target/experience/game feature；Bevy FixedUpdate/state；Fyrox game plugin；Godot project export pack；Unity Graphics image test runner | 按产品profile、fixed simulation、relocatable package和source-bound oracle路由 |

本轮把“当前开发机能看到完整场景”与“clean clone可构建发布产品”分开，确认：

- 根`.gitignore`忽略`examples/vampire/*`，7份项目自有模型TOML及对应meta只在ignored local state；scene直接引用其中4份，主import test也先要求它们存在；
- 当前108个唯一`res://`引用在本机都存在，但依赖578个ignored local文件中的source/cache/registry，不能成为仓库闭包证据；
- project required Rendering/glTF/Navigation/ZrVM，README却要求手工拼五组Cargo feature；asset测试用`register_first_wave_plugin_fixture_importers_for_test()`绕过产品catalog；
- 唯一脚本把movement/attack/contact damage放在`onUpdate`，玩家每update固定移动1单位，伤害没有cooldown；`balance.toml`和behavior tree无人消费；
- scene没有Boss、spawn/wave、XP/level/upgrade、win、durable save、完整Retry、collision/physics/audio；6个enemy binding只有3个启用；
- tracked `.zr_cli_manifest`写入本机绝对路径并登记两个不存在产物；`main.zro`没有compiler/ABI/BuildSet/target receipt；
- 10个real-ZrVM gameplay/menu/HUD/performance test全部ignore，目标plugin owner无等价Vampire测试；主import test又对当前27行WGSL断言20个不存在marker；
- README称latest accepted的ground/game-over图不在仓库，34张PNG无结构化sidecar且跨代重复；单帧60.87 FPS声明与后续30.89/33.98 FPS正式样本不可比较。

本轮建立VampireProductProfile、VampireSourceManifest、VampireRunCoordinator、VampireSimulationConfig、VampireWorldDirector、VampireScriptPackageBuilder与VampireEvidenceHarness路线。登记5项P0、80项P1、16项P2、M0-M6及20项资格门；详细见`zircon_app/06-vampire-roguelite-example-project-asset-script-gameplay-evidence-product-integration-review.md`。

本轮只新增review与索引，没有修改Vampire production、tests、manifest、asset、script artifact、cache或截图，也没有把既有workspace编译阻断重复执行。Runtime04/07/08F/08G继续拥有通用asset/ZrVM/AI/Gameplay，Plugins06拥有provider catalog/profile，Tooling03/07/10拥有打包、性能证据与测试基础设施；本篇只拥有Vampire产品样例闭包、玩法真值、工件与验收消费。

## 115. Renderable Empty Project Template / Create / Import / Render / Export / Evidence 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| template source | 17文件 / 217行 / 4,867 bytes | 唯一启用模板；Runtime Interface编译期逐项内嵌 |
| project/settings/scene | 3 TOML / 1,485 bytes | camera/sun/cube最小场景；project不声明plugin/provider/BuildSet |
| source/meta | OBJ/material/WGSL/zshader各1，`.zmeta` 3 | reference存在且UUID一致；meta为dirty seed、空digest/importer、mtime/version 0 |
| template pack | 7文件 / 237行 / 7,811 bytes | 有exact source/embedded集合、RelPath和TOML name rewrite；无version/digest/compatibility |
| Editor create owner | 18文件 / 3,064行 / 104,784 bytes | 同parent staging/rename、open与rollback基础较完整 |
| Hub create owner | 13文件 / 2,659行 / 85,917 bytes | 复制/rename后即成功；不open、scan/import或验证Project Ready |
| consumers/tests | 101个非reference consumer / 28个直接test文件 | F2检查changed pixel/draw/light/zero fallback；Hub、升级、installed export与跨平台仍缺 |
| export/evidence | Windows release preset + F1-F5 workflow | 默认暴露未qualified export；CI仅Windows、artifact保留7天且当前Editor compile被239错阻断 |
| 参考引擎 | Unreal template descriptor/targets/plugins；Bevy required features；Fyrox generator/upgrade；Godot project renderer/export；Unity Graphics package manifest/image test | 按versioned package、显式capability、统一create operation、upgrade和source-bound evidence路由 |

本轮把“17个bytes可被复制并在测试binary中渲染”与“用户得到可识别、可升级、可发布的项目产品”分开，确认：

- `ProjectTemplateId::RenderableEmpty`没有schema/version/content digest/engine compatibility；修改模板后仍沿用同一ID，生成项目也不记录template、creator build或migration baseline；
- project manifest没有plugin selection、renderer/backend、required importer或BuildSet，标准产品是否可渲染取决于host碰巧编入哪些feature；
- Editor在publish后open项目并能回滚post-commit failure，Hub维护另一份创建事务并在复制完成后直接返回success，两者不共享Project Ready定义；
- scene没有tracked `.zmeta`，三份asset meta只是首次import seed；当前pack renderer不验证scene dependency、provider、artifact或export compatibility；
- Windows release preset随每个项目生成，但Tooling03已证明通用export链仍可接受无效pack/placeholder host；模板必须在资格通过前隐藏或标记不可发布；
- F2的WGPU像素、draw/light/material oracle和F4/F5 source fingerprint值得保留，但Hub create、native present、installed game、Linux/macOS、template migration与durable accepted evidence未覆盖。

本轮建立TemplatePackageDescriptor、TemplateContentManifest、ProjectCreationReceipt、共享CreateProjectOperation、ProjectReadyQualification与TemplateUpgradeGraph路线。登记4项P0、72项P1、16项P2、M0-M6及20项资格门；详细见`zircon_app/07-renderable-empty-project-template-create-import-render-export-evidence-product-integration-review.md`。

本轮只新增review与索引，没有修改template、production、tests、workflow或artifact，也没有重复运行source条件未变化的Editor失败lane。Runtime04、Interface02、Editor02/04、Hub01、Plugins06、Tooling03/10继续拥有通用asset/schema/transaction/provider/export/test基础；本篇只拥有template package identity、跨创建面成功语义、默认project capability/export truth和template级资格消费。

## 116. Top-Level Acceptance Archive / Serialization Fixture Provenance / Currentness / Migration 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| acceptance archive | 100 Markdown / 5,938行 / 530,914 bytes | policy已定义为历史归档；仍混有current/accepted语义与direct test consumer |
| metadata | 41 frontmatter / 39 Date / 25 header status | 59无frontmatter、61无Date、75无header status；状态值不是有限schema |
| structured sections | Scope 50 / tooling 33 / results 28 / decision 31 | 0 source/build identity、0 machine result link |
| result text | Accepted63 / passed91 / failed77 / in-progress33 / pending30 | token互相重叠，不能生成current qualification |
| environment | 29文件 / 372个Windows absolute path | 主要为历史target/temp目录；不可移植且无environment receipt |
| frontmatter lists | 664 item | 586 path、59 command/note、17 identifier、2 glob；8个`user:`被YAML解析为mapping |
| path drift | 46次 / 32 unique / 14文件 | 11次output record迁到`_archive`未更新，35次source/test/session删除或移动 |
| inbound links | 187次 / 82 source / 68 target name | 7个target不存在；现存61篇有引用、39篇无入站 |
| direct executable consumer | 2 | Python与Runtime test读取历史Markdown字符串作为current结构门 |
| serialization fixtures | 6文件 / 50行 / 1,283 bytes | 全部可解析且有真实consumer；无writer provenance、digest或corpus manifest |
| 参考引擎 | Unreal Automation Info/ExecutionInfo/Controller；Bevy/Fyrox CI；Godot doctest runner；Unity Graphics UTR/Yamato artifact | 按test definition、run receipt、artifact、archive与immutable fixture分型 |

本轮把“历史叙述”“当前测试结果”和“可执行兼容性输入”分开，确认：

- repository skill已明确禁止继续在`tests/acceptance`创建per-feature文档，编号计划的`状态与产出记录`才是canonical evidence；100篇旧记录尚无机器archive catalog、disposition或canonical row迁移；
- 没有一篇绑定source fingerprint/commit/BuildSet，也没有一篇链接机器result receipt；492个Cargo、53个Python和78个PowerShell命令标记只是自然语言转录；
- 两个required test owner直接要求archive文件存在并包含指定owner/中文叙述，改历史Markdown即可改变测试结果，必须迁到实际module/owner/behavior contract；
- 82个文件仍引用archive路径，7个目标从未存在或已删除；frontmatter又有46次精确path drift，archive迁移没有原子link/disposition工具；
- 四个project manifest fixture被Interface/Runtime/Hub共享，两个scene fixture被真实migration test消费，全部TOML/JSON解析成功、无BOM且有终止LF；
- 6个样本没有原始writer/build/platform、schema代际解释和immutable digest，当前测试只能证明当前fixture与当前reader自洽，不能证明真实历史发行兼容。

本轮建立AcceptanceArchiveManifest、TestInfo/RunReceipt/EvidenceSet单向投影与SerializationFixtureCorpus路线。登记3项P0、60项P1、14项P2、M0-M6及20项资格门；详细见`zircon_tooling/12-top-level-acceptance-archive-serialization-fixture-provenance-currentness-migration-review.md`。

本轮只新增review与索引，没有修改production、tests、fixture、workflow或100篇历史正文，也没有重跑source条件未变化的Editor/WOC失败lane。Tooling10继续拥有通用TestPlan/result，Tooling07拥有性能/画质证据，Runtime04/05与Interface02拥有实际serialization schema；本篇只拥有archive硬切/迁移与顶层fixture历史来源/不可变性。

## 117. Runtime Host Foreign Output Safe API / Ownership / Admission / Budget / Fuse / Observability 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| host production | 11 Rust / 1,066行 / 36,797 bytes | ownership/decode/error/item/kind/metrics/policy/state全量逐文件E3 |
| host tests | 1 Rust / 489行 / 17,379 bytes / 9 tests | normal/reject/release/depth/time/concurrency/perf/policy；无soundness/unload/admission race |
| App direct consumer | 5文件 / 1,928行 / 70,479 bytes | session借用保持DLL；frame校验/释放接fuse；status与raw carrier仍穿过safe host API |
| Editor direct consumer | 7文件 / 934行 / 32,952 bytes | `Arc` owner保持provider；owned frame wrapper重复；empty frame语义与App分叉 |
| Interface carrier/policy | 12文件 / 2,230行 / 76,919 bytes | raw output/status可由Safe Rust伪造；accessibility limit/API已发布 |
| Runtime producer | 3文件 / 1,100行 / 38,562 bytes | allocation ID/session census基础可保留；accessibility producer无人消费 |
| output coverage | 6 JSON policy / 7 metric kind | frame混入session_protocol；accessibility/status无shared kind/policy闭环 |
| 参考引擎 | Unreal SharedBuffer/ModuleManager；Godot generated extension allocator；Fyrox dylib owner；Bevy reader/task pool；Unity Graphics resource scope | 按owner-lifetime、module unload、generated contract、controlled execution与structured observability路由 |

本轮确认shared consumer policy是可保留基础，但其“Safe host-side”公共合同仍不成立：

- `decode_json`与`ensure_call_succeeded`是安全方法，却能解引用安全调用方自行构造的`ZrOwnedResultV2`/`ZrStatus`裸指针；null/len/allocation shape不能证明地址有效；
- `RuntimeOwnedOutputReleaser::new`安全且类型为`Copy`，内部却调用unsafe function pointer，不持有DLL/provider、session lease或generation；
- App/Editor局部owner通常保持当前主路径DLL存活，但不能修复公共crate soundness；raw adoption必须集中为unsafe入口，safe层只接收不可伪造RAII owner；
- fuse检查与FFI调用之间没有admission token，另一线程熔断后已通过检查的线程仍能进入provider；现有gate只阻止结果记为accepted；
- Interface/Runtime已发布并生产accessibility tree及16MiB/65,536项limit，App/Editor/host却没有consumer/kind/policy；frame又缺专属指标，Editor允许App拒绝的empty frame；
- deadline每4KiB read检查且在完整decode/validation后复核，item count发生在完整DOM/typed value构造后；consumer cap不能替代既有Interface producer P0；
- 9个test没有compile-fail/Miri、guard-page child process、实际DLL unload、真实registry纵向、admission race/fuzz/32-bit/C consumer；M2.4的8/8和Critical0记录已不具当前性。

本轮建立RuntimeProviderLease、RuntimeSessionLease、ForeignCallLease、OwnedRuntimePayload、Open/Fusing/Fused/Closing/Destroyed状态机和declarative OutputPolicyRegistry路线。登记3项P0、72项P1、16项P2、M0-M5及14项资格门；详细见`zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md`。

本轮只新增review与索引，没有修改或运行production/tests，也没有复用历史M2.4结果冒充fresh validation。Interface01/03/04继续canonical拥有ABI/build identity、producer budget/accessibility generation/profile exhaustion；本文只拥有shared host safe abstraction、provider/session owner、call admission/fuse和consumer output policy闭环。

## 118. Repository Codex Skill / Hook / Structural Audit Control Plane 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| tracked `.codex` | 243文件 / 34,889行 / 1,559,270 bytes | E3分簇；inventory fingerprint `b5f40120...cf178` |
| permissions/hooks | config + hooks.json + 2 Python entry | repo授予never/danger-full-access；Cargo guard与Session sync均fail-open |
| skill inventory | 11顶层目录 / 53 SKILL / 21 agent YAML | catalog漏6顶层、只写10 summary；12 nested SKILL无frontmatter |
| executable control code | 111 scripts / 24,632行 / 1,006,207 bytes | 96 Python、12 PowerShell、3 Shell；语法检查全部通过 |
| runtime audit subsystem | 91文件 / 15,552行 / 643,380 bytes | aggregate + boundary/inventory/renderer；source anchor与exact count密集 |
| aggregate execution | Runtime 114.6s / Editor 8.6s | 两者报告blocked/debt仍exit 0；Runtime JSON 673,171 chars |
| external audit tests | 19 modules / 50 tests / 180.597s | 38 pass / 12 fail；外层182.1s timeout，CI未选择 |
| convention truth | 两份169行Markdown | hash不同；Runtime error、foreign-output ABI、Cargo policy三项MUST分叉 |
| reference engines | Unreal BuildGraph/Automation；Bevy CI tool；Fyrox matrix；Godot runner；Unity Graphics Yamato/UTR | 按typed graph/result、失败退出、环境生命周期和artifact路由 |

本轮把“代理说明”“安全权限”“执行hook”“结构规则”“测试结果”分开，确认：

- tracked repo config自行选择`approval_policy = never`和`danger-full-access`，权限authority没有留在用户/组织/运行环境；
- PreToolUse guard只接受Bash payload并用command regex判断，非Bash、alias/wrapper、payload/内部异常均可放行；五类Session sync也吞掉任意异常返回0；
- agent读取的`.codex` development convention与CI读取的docs convention已在`KernelError/CoreError`、旧foreign buffer/当前owned result ABI、九root lease/shared target三处冲突；
- Runtime aggregate输出多个blocked/debt/missing/mismatch但固定exit0，Editor在30项debt时同样exit0；50项外部测试当前12项失败且不在required CI；
- 语法层面96 Python、12 PowerShell、3 Shell、21 YAML与1 JSON全部可解析，说明下一步应修语义、真源、执行门和currentness，而非推倒全部脚本。

本轮建立Repo Control Plane Manifest、外部Security Policy/Session Grant、typed Hook health/admission、SourceSnapshot/Rule DAG/FindingSet与SARIF/JUnit/artifact路线。登记6项P0、72项P1、16项P2、M0-M5及16项资格门；详细见`zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md`。

本轮只新增review与索引，没有修改`.codex` config、hook、skill、audit script、tests或CI。Tooling01/06/10继续拥有Cargo/Coordinator/Test Service通用实现；本篇只拥有repo-local trust boundary、skill/rule/hook registry、aggregate gate/currentness与required CI接线。

## 119. Editor Workbench DesignSpec / Screenshot / Visual Evidence / Prototype 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| preview tool source | 16文件 / 15,483行 / 692,040 bytes | E3逐文件；10个JS/MJS语法通过 |
| renderer/manifest | `design.js` 9,183行；270项design | 192个editor-page只有id/output/kind，无capability owner/lifecycle/product gate |
| visual corpus | 271 PNG + 2 metadata / 26,370,084 bytes | 直接tracked；candidate与自认证hash同操作写入，无approved/diff/promotion |
| current verifier | 15.8s / exit 1 | clean tracked `view-descriptors.json`与evidence digest漂移；required CI 0 invocation |
| browser structure | 270页 / 72,540 DOM nodes | 140种结构、最大同构组34；14,775 visual controls、1,805 native controls、0 explicit a11y annotation |
| evidence dependencies | 46 source entries | 31 icons + 4 fixtures未被固定design capture请求；legacy app三件套反而未绑定 |
| capture environment | Playwright 1.62.1 + system Edge channel | 未记录Edge/OS/font/DPR/locale/theme/GPU identity；固定250ms wait |
| reference engines | Unreal Functional Screenshot/Comparison；Unity Graphics Test/Yamato；Bevy showcase/Pixel Eagle；Fyrox/Godot真实Editor preview/capture | 按真实产品capture、环境key、incoming/approved/diff、review/promotion与artifact lane对照 |

本轮确认工具的双向manifest检查、LF canonical hash、完整导出才写evidence、unknown selection fail-close和双reference byte identity可以保留，但必须进入正式Visual Evidence Service。当前失败不是简单遗漏重导出：固定DesignSpec与legacy fixture prototype共享了错误的dependency graph；普通fixture迁移会让全部271张设计图RED，prototype代码变化却不使其证据失效。

本轮建立DesignSpec Registry、Prototype Scenario、真实Editor Visual Product Test三产品分离，以及VisualCaptureReceipt/ComparisonReport/ApprovalReceipt、nonce worker、staged candidate、immutable approved baseline和required visual lane路线。登记6项P0、40项P1、12项P2、M0-M6及16项资格门；详细见`zircon_tooling/14-editor-workbench-design-spec-screenshot-export-visual-evidence-prototype-governance-review.md`。

本轮只新增review与索引，没有运行会覆盖271张PNG/evidence的`design:export`，没有修改fixture、web prototype、production Editor、tests或CI。Editor各domain报告继续拥有真实authoring功能，Tooling10继续拥有Test Service；本篇只拥有设计证据真实性与产品映射。

## 120. MVP BuildSet / Product Process / Acceptance Evidence / Resource Baseline Control Plane 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| `tools/mvp` | 33文件 / 14,789行 / 675,385 bytes | E3逐文件；31个PowerShell AST 0 error，2个JSON合法但无schema version |
| direct tests | 17文件 / 约9,012行 | 14-file aggregate 84 total / 81 pass / 3 fail；一个script suite单跑Pester TotalCount 0 |
| source fingerprint | 2,373 untracked files / 1,680,960,238 bytes | 静止顺序复算一致但分别13.228s/11.318s；不是immutable snapshot且owner不受控 |
| build/product receipt | 4个固定Windows product | 依赖repo-local Codex validator；不绑定实际toolchain/target/linker/SDK/runtime deps/symbol/SBOM |
| staging/process | 1个固定RenderableEmpty happy path | Stage无Job Object、继承parent env、unbounded stdout/stderr、按CIM path/PID清理 |
| visual/product oracle | self-reported marker + PNG | 只拒绝空图/少于100个非背景像素；无独立observer、approved baseline或semantic diff |
| resource baseline | 9个plan scenarios / 0 tracked observation producer | reporter可将结构匹配的caller JSON直接标为`measured` |
| required Windows workflow | 1 lane | F0-F5 Cargo/product path存在，但0个MVP/resource/render-extract focused control-plane suite调用 |
| reference engines | Unreal TargetReceipt/BuildGraph/Gauntlet；Unity Graphics Tests；Bevy showcase；Godot CLI/test；Fyrox build tools | 按BuildSet、ProductReceipt、Scenario/Device/Run、Observation、Evidence/Promotion拆分对照 |

本轮确认Windows no-follow native handles、volume/file/creation identity、ancestor lease、staging snapshot/projection/tree manifest、partial tree验证和no-overwrite rename是值得保留的强基础；RenderExtract的Job/frozen input也可作为统一runner backend。缺口在其上层信任链：活动工作树采样不能证明四个artifact来自同一source bytes，stager事后记录当前rustc不能证明build toolchain，产品自报marker/PNG不能独立判定行为，caller JSON形状更不能证明真实性能观测。

本轮建立BuildSetId、ToolchainSetId、ProductReceiptId、ScenarioId、DeviceProfileId、RunId、ObservationId、EvidenceSetId与PromotionId路线。登记6项P0、60项P1、14项P2、M0-M5及分层资格门；详细见`zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md`。

本轮只新增review与索引，没有修改production/test/workflow，也没有重跑已知被239个editor compile errors和未完成F5前置阻断的完整product acceptance。现有F5与resource evidence保持unqualified/blocked，不能由局部历史artifact升级为current。

## 121. Capability Truth / Placeholder / No-op / Fallback / Degraded / Qualification 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| capability/status类声明 | 256声明 / 162文件 | 多个局部状态机存在，但没有统一Qualification类型 |
| `todo!` / `unimplemented!` | 3处 | 全部位于Sound Editor测试替身，production owner为0 |
| TODO/FIXME/XXX/HACK | 30处 | 28处vendored Recast，另两处为测试字符串/错误文本，无actionable production TODO |
| placeholder/stub候选 | 原始4,337行 / 831文件；过滤后924行 / 252文件 | 只能作为consumer语义定位，未批量定性为缺陷 |
| noop候选 | 原始360行 / 125文件；过滤后108行 / 51文件 | 混合合理Null Object、测试替身和产品协议风险 |
| fallback/degraded候选 | 原始7,182行 / 1,327文件；过滤后4,046行 / 761文件 | 缺统一严重度、预算、last-good与qualification聚合 |
| unsupported候选 | 原始1,918行 / 620文件；过滤后1,231行 / 395文件 | 必须区分硬件事实、目标限制、临时缺失与恢复政策 |
| 重点控制链 | Runtime plugin/profile/lifecycle/render、Editor capability/Play、rendering/particles plugin、Hub status | manifest/config/membership可越级生成Complete/Available/Success |
| reference engines | Unreal module/modular feature/RHI、Bevy App/Plugin/Render、Godot RenderingDevice、Fyrox Plugin、Unity Graphics RenderGraph | 声明、load、device feature、execution与product qualification必须分层 |

本轮确认生产源码的主要风险并不是显式`todo!`，而是已能返回成功的临时语义。`CapabilityStatus`/`PluginMaturity`来自package自述，availability projection只验证target、maturity与provider membership；`ModuleLifecycle`默认ready true，render registry和Play公开构造又允许成功型no-op。单个机制可以合法存在，但当前没有SemanticEffect、ProviderInstance、ExecutionObservation、FallbackEvent与current QualificationReceipt阻止其成为“功能完成”证明。

本轮也保留正向基础：availability已经分类target/maturity/provider原因，particles使用typed optional status，Editor产品安装真实ProcessPlayBackend/NativePluginBridgeActivation，mesh no-op会被真实executor覆盖。登记3项P0、52项P1、12项P2及M0-M6；本篇只拥有跨域状态/资格协议，render、Play、lifecycle、plugin与Hub实现仍回到canonical domain报告。详见`zircon_tooling/16-capability-truth-placeholder-noop-fallback-degraded-qualification-control-plane-review.md`。

本轮只新增review与索引，没有修改production/test/manifest/CI，也没有重复运行已知被Editor、Hub和WOC编译漂移阻断的动态lane。静态关键词命中不是缺陷计数，后续实施必须按具体caller、generation、输入域、fallback impact和产品consumer复核。

## 122. Repository Content / SourceSet / Ignore / Generated / Vendor / License / SourceArchive 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| 全部tracked tree | 27,432 path / 1,335,024,765 checkout bytes | E2只读枚举；Git commit固定字节但未分类Source/Generated/Vendor/Fixture/Evidence |
| tracked-but-ignored | 2,849 files / 93,877,865 bytes | WOC 1,967、Vampire 172、`.codex` 242、docs evidence 461、plugins 7 |
| source enumerator | MVP + Coordinator 2条 | 均用`git ls-files --others --exclude-standard`，ignored新输入不进入fingerprint |
| Cargo metadata | 162 manifests / 159 packages | 159都有license；0 repository/readme；仅2个显式`publish=false` |
| Node metadata | 5 private packages / 4 lockfiles | 0 license/repository/engines/packageManager；docs prototype无lock |
| root license | `MIT OR Apache-2.0`声明 / 1份MIT文本 / 0份Apache文本 | 声明与可分发文本不闭合，本轮不作法律解释 |
| local notices | WOC、Vampire、navigation | WOC声明的`source-LICENSE.txt`本机存在但ignored/untracked；无根级/产品级NoticeGraph |
| checked-in generated/cache | `.rustc_info.json`、contact-shadow cache pair、Coordinator 29-file web dist、tray dist | 无统一content class、generator receipt或source archive policy |
| path portability | 27,432 paths | 0 case-fold collision、0 Windows非法segment、0特殊Git mode、0当前绝对路径>=240 |
| reference engines | Bevy、Godot、Fyrox、Unity Graphics、Unreal artifact/third-party roots | 对照attributes、dual-license/credits、copyright/thirdparty、package notice与receipt分层 |

本轮确认根`.gitignore`同时承担local output、source membership、evidence retention和asset selection，导致已跟踪内容靠历史force-add存活，同目录新增required source/license/fixture却能逃逸普通变更视图与当前fingerprint。Git tree还混合1.148 GB docs/evidence、本机rustc缓存、shader cache、generated web bundle与vendor source，没有RepositoryContentManifest或profile化SourceArchive。

许可侧只记录机器事实：Cargo声明dual-license但Git tree只有MIT文本；WOC `LICENSES.md`声称materialized的source license没有进入Git tree；`cargo deny`不验证最终产品notice/asset/native vendor闭包。报告建立ContentClass、Frozen SourceSet、Generated/Vendor Receipt、NoticeGraph、SourceArchiveReceipt与Package publication policy；登记3项P0、54项P1、12项P2及M0-M5。详见`zircon_tooling/17-repository-content-source-set-ignore-generated-vendor-license-distribution-review.md`。

本轮只新增review与索引，没有修改`.gitignore`、license、cache、source、test或workflow，没有生成1.3 GB archive或执行package/release。Cargo workspace、WOC codegen、Coordinator、evidence、release、Codex和BuildSet仍由既有canonical报告拥有。

## 123. Executable Target / Entrypoint / CLI / Process Receipt / Qualification 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| Rust executable target | 18 | 8 Product + 10 Tool/Generator/ControlPlane；未发现第19个产品入口 |
| Cargo声明 | 12 explicit `[[bin]]` / 6 auto-discovered | 仅`zircon_app`、`zircon_hub`、session tray关闭`autobins` |
| feature guard | 7 target | App 3项，Runtime工具4项；其余目标没有统一variant/profile声明 |
| product entry | WOC 4 + App 3 + Hub 1 | WOC只打印identity后退出；实际产品缺陷由App03-05拥有 |
| tool/control entry | 10 | codegen、cargo-zircon、tray、ONNX convert与Runtime 6个工具；内部语义由既有owner拥有 |
| MVP target projection | 2 executable + 2 runtime DLL variant | 私有PowerShell schema只覆盖editor/runtime，不是仓库target catalog |
| CLI/process contract | JSON、自由文本、Rust `Result`、panic、exit 1/2/3/4 | 无统一protocol version、ExitDomain、Ready handshake或terminal receipt |
| direct entry tests | App inline、cargo-zircon/WOC/tool Python/Rust tests | 无18项catalog-driven matrix，部分产品仅测helper或内部library |
| reference engines | Unreal TargetDescriptor/Rules/Receipt；Godot mode routing；Bevy example showcase；Fyrox CommandDescriptor/BuildProfile | 按target identity、artifact、launch、observation与qualification分层对照 |

本轮确认薄入口和library委托可以保留：editor/runtime经`EntryRunner`统一host边界，PBR viewer有独立event loop/capture设置，Hub与tray把`main`委托给library，多数Runtime工具也拆分args/run模块。缺口是这些局部机制没有汇聚成canonical ExecutableTargetManifest；Cargo package/bin/path、MVP logical id、artifact filename、Hub启动路径和测试选择器仍各自拥有字符串映射。

报告建立`ExecutableTargetDescriptor -> TargetArtifactReceipt -> LaunchContract -> ProcessInstanceReceipt/HealthObservation -> TargetQualificationReceipt`链，并区分long-running Product/Service与one-shot Tool/Generator状态机。登记2项P0、50项P1、12项P2、M0-M6及12项资格门；spawn、窗口出现、日志关键字、identity JSON或exit 0都只能作为observation，不能单独生成Qualified。详见`zircon_tooling/18-executable-target-entrypoint-cli-process-receipt-product-qualification-review.md`。

本轮只新增review与索引，没有修改Cargo manifest、entrypoint、production、tests或workflow，也没有重跑已知被Editor/Hub/WOC源码漂移阻断的动态lane。WOC、Hub、ONNX、export、prewarm、font与reflection的内部实现继续由App/Hub/Plugins/Runtime/既有Tooling报告拥有。

## 124. Script Entrypoint / Interpreter / Command Registry / CLI / Operation Receipt 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| tracked script-like文件 | 2,210 | 85 PS1、21 PSM1、1,170 Python、3 Shell、383 JS、423 MJS、41 TS、84 TSX；全部Git mode 100644 |
| PowerShell入口 | 49个非测试PS1 | 全部有参数块；常用strict mode，但没有`#requires -Version`或统一exit/output domain |
| Python入口 | 23个直接入口 | 20个argparse、18个JSON、23个terminal exit；仓库没有tracked Python dependency/version lock |
| Node命令 | 5 package / 323 script | WOC占304；4个执行型项目有lockfile，0个package声明`engines`/`packageManager` |
| GitHub workflow | 3 workflow / 33个`run` block | shell自由文本没有CommandId、typed input/output或统一receipt |
| 本机wrapper | 11个`tools/dev-*.cmd` | ignored/untracked，clean clone不存在；source membership继续由Tooling01/17拥有 |
| mutation入口 | build/export/install/cleanup/Coordinator/MVP/evidence/hook | 路径或alias本身兼作权限，没有统一MutationScope、admission、transaction与rollback |
| reference engines | Unreal AutomationTool/BuildGraph；Godot SCons；Bevy typed CI tool；Fyrox CommandDescriptor/BuildProfile；Unity Graphics Wrench | 对照command catalog、typed invocation、interpreter/tool identity、step observation与terminal receipt分层 |

本轮保留PowerShell参数块/strict mode、Python argparse、安装脚本ShouldProcess/DryRun、WOC generate/check配对、Node lockfile及`zircon_export`/Coordinator的入口-domain拆分；缺口不是语言选择，而是调用权限、interpreter/tool dependency、输入输出、退出域和操作结果仍由各文件自行定义。

报告建立`ScriptEntrypointDescriptor -> InterpreterSetReceipt -> CommandInvocation -> ScriptStepObservation -> ScriptOperationReceipt`链，登记2项P0、54项P1、12项P2及M0-M5。具体export、WOC codegen、Coordinator、MVP、Workbench、Codex hook、test和Rust executable语义仍回到既有canonical owner；本篇只拥有跨语言命令控制面。详见`zircon_tooling/19-script-interpreter-entrypoint-command-registry-cli-operation-receipt-review.md`。

本轮只新增review与索引，没有修改脚本、workflow、package、lock、production或test，也没有重复运行已知被Editor、Hub和WOC编译漂移阻断的动态lane。

## 125. Zr Language / Parser / Type System / SemIR / Bytecode / Package Loader / VM Runtime 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| Zircon script runtime | 102个Rust文件 / 约17,307行 / 653,951 bytes | host/package/hot-reload基础真实；语言实现由外部backend提供 |
| external parser/core/AOT/LSP | 971个C/H/Rust文件 / 约497K行 | 完整工程基础，不是stub；当前独立repo dirty且未进入Zircon SourceSet |
| external tests | 8,040行CMake / 128个CTest / 10个executable | 上游CI真实存在；Zircon required CI不构建或运行real backend |
| tracked language资产 | 709 `.zr` / 38 `.zro` / 277 `.zrp` / 0 `.zri` | 无唯一ProductScriptBuildSet；两份manifest含绝对路径和14个缺失output |
| `.zro` IO | host-native writer + recursive reader | signature/version/width/endian/count/verifier admission不闭合 |
| type/codegen/object/GC | conversion、constant ref、prototype、PIC | 临时语义跨层，可静默错编译或掩盖cache corruption |
| reference engines | Godot GDScript、Unreal VerseCompiler/VerseVM、Bevy Reflect、Fyrox Script | 对照phase/cache invalidation、CFG verifier、type identity和typed lifecycle |

本轮保留Zr parser/SemIR/AOT/LSP分层、dependency lock、compile-time budgets、atomic comptime cache、AOT descriptor validation、Rust callback unwind/root和大量测试。硬阻断是real backend由未固定兄弟仓库与本机DLL提供，Cargo/required CI不能重建同一toolchain；`.zro` reader读取却不验证关键header与长度域；incremental key遗漏compiler/options/target/ABI/dependency输入且逐文件覆盖final artifact；type checker、codegen、object conversion和GC inline cache又没有共同schema。

报告建立`ZrToolchainSourceReceipt -> ZrToolchainBuildReceipt -> ProjectManifestV2/DependencyLock -> Typed SemIR -> ZroContainerV2/Verifier -> ScriptBuildReceipt -> RuntimeGeneration/ExecutionBudget -> ScriptExecutionReceipt`链，登记4项P0、60项P1、14项P2及M0-M7。Runtime07、Plugins01、Editor31、Tooling05/10/11/17和App03继续拥有各自package/ABI/UI/tooling/product边界；本篇只拥有语言核心、compiler artifact、bytecode trust和VM execution governance。详见`zircon_runtime/21-zr-language-parser-type-system-semir-bytecode-package-loader-vm-runtime-review.md`。

本轮只新增review与索引，没有修改Zircon或外部`zr_vm` production/test/build源码，没有迁移或生成脚本artifact，也没有在dirty external source上重跑CMake/CTest/real backend。外部commit、dirty state、toolchain identity和artifact inventory必须在实施前重检。

## 126. Cargo Package / Workspace / Feature / Dependency / Target Graph 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| Cargo manifest | 162份 / 2,914行 / 88,076 bytes | 形成159个package和338个target object，尚无canonical package catalog |
| target kind | 18 bin / 113 plain lib / 42 cdylib / 2 proc-macro / 156 integration test / 7 custom build | target identity、role、artifact与资格分别由manifest、脚本和测试字符串拥有 |
| workspace graph | root 37 package、plugin静态139 member、WOC 8、tray 1 | root/plugin重叠26个package，membership与发布边界未形成单一真源 |
| lock graph | 4份 / 21,832行 / 524,791 bytes | common package在root-plugin/root-WOC/root-tray分别有105/13/77组版本集合差异 |
| dependency graph | 638次直接依赖：607 normal / 24 dev / 6 build / 1 target-specific | 342 workspace、150 path、146 registry；没有统一source/trust/version policy receipt |
| feature graph | 53 package / 208 feature / 67 optional dependency | optional依赖均被feature绑定，但产品角色约束只验证请求值，不验证Cargo解析closure |
| Editor product role | `target-editor-host`解析同时包含`target-client` | `zircon_editor -> zircon_runtime(default)`重新启用Runtime默认Client，构成1项P0 |
| package metadata | 全部0.1.0 / edition 2021 / MIT OR Apache-2.0 | 全部缺`rust-version`、repository、readme；该治理缺口由既有Tooling01/17继续拥有 |
| reference engines | Unreal RulesAssembly/TargetReceipt；Godot SCons；Bevy package graph；Fyrox profile；Unity package/asmdef | 对照module/target rules、resolved dependency closure、receipt与build profile分层 |

本轮确认多数局部做法可以保留：workspace dependency在根、plugin和WOC已有局部authority；所有67个optional dependency都进入feature；仓内没有Git dependency；Bevy式的`default-features = false`与target-specific dependency已有少量正确示例。新增硬缺口是Editor产品角色并非exact-one：命令请求Editor host时，Cargo最终图仍因`zircon_editor`依赖Runtime默认feature而包含Client。现有profile测试比较手写数组，无法发现这类transitive resolution污染。

报告建立`RepositoryContentManifest -> CargoPackageCatalog/DependencyPolicy/FeatureConstraint -> ProductRole -> canonical Cargo resolution -> ResolvedPackageGraphReceipt -> Target/BuildScript/GraphDiff Receipt -> BuildSet`链，登记1项P0、48项P1、12项P2及M0-M6。Tooling01/05/10/17/18、Plugins06和Runtime21继续拥有toolchain、build script、test、source archive、executable、provider profile与外部Zr source等既有边界；本篇只拥有Cargo package、workspace、feature、dependency、target graph及其resolved receipt。详见`zircon_tooling/20-cargo-package-workspace-feature-dependency-target-graph-build-receipt-review.md`。

本轮只新增review与索引，没有修改manifest、lock、production、test或workflow。动态证据仅使用只读`cargo tree --locked`确认Editor最终feature closure；没有重跑已知被plugin lock、Editor、Hub或WOC源码漂移阻断的lane。

## 127. Unsafe Rust / FFI / Native Memory / Thread / Panic / Unload Safety 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| tracked Rust | 17,263文件 | 234文件出现1,656行unsafe相关文本；macro展开不在此数内 |
| production-like unsafe block | 约810 block / 140文件 | 只按路径排除明显tests，是风险inventory而非语法/soundness证明 |
| unsafe function/export | 131个unsafe fn候选 / 231个unsafe extern声明或定义 | 73个实际production public/restricted-public unsafe函数中65个近邻无`# Safety` |
| C layout/raw pointer | 212处`repr(C)` / 663行裸指针使用 | Runtime DLL、plugin、gateway、ECS、native backend与platform API分散拥有 |
| unsafe Send/Sync | 18项，其中13项production-like | Thread affinity、move/share、drop/unload proof没有统一manifest |
| safety policy | 4个WOC crate `forbid(unsafe_code)` / 72行可识别Safety说明 | 其余155个package无deny-by-default和workspace unsafe lint |
| native C/C++ | Navigation 34 `.cpp` / 23 `.h`，总子树91文件 | Rust token门看不到bridge/vendor、compiler flag、exception与sanitizer风险 |
| dynamic library | Runtime DLL / native plugin / RenderDoc / `cargo-zircon` probe | 四条lease/trust/call/unload路径没有共同NativeModuleGeneration |
| reference engines | Bevy UnsafeWorldCell；Unreal ModuleManager；Godot GDExtension；Unity CoreUnsafeUtils；Fyrox pool反例 | 对照alias witness、pre-unload、init level、unsafe utility locality和非规范性参考 |

本轮保留必要的低层基础：App持有DLL直至session destroy，Runtime export与部分plugin call已有panic guard，foreign output已有预算/release/fuse，ZrVM owner明确process lock和drop order，ECS dense storage与query conflict是性能真实实现。问题不是unsafe数量本身，而是没有canonical UnsafeUnitId、hazard/owner、Safety proof、thread/module generation、unload quiescence和source-bound evidence。

Plugins01、Runtime Interface01/05、Runtime05、Runtime08A/08D、Runtime21及Tooling01/10继续拥有具体soundness、ABI、ECS、Jolt/Recast/ZrVM和sanitizer finding；本篇不重复其P0。本篇建立`UnsafeUnitManifest -> ForeignCallScope/ThreadCapability/EcsAccessWitness -> NativeModuleLease/Generation -> UnsafeEvidenceReceipt -> BuildSet/ValidationSet`链，登记0项P0、28项P1、10项P2及M0-M6。详见`zircon_tooling/21-unsafe-rust-ffi-native-memory-thread-affinity-panic-unload-safety-governance-review.md`。

本轮只新增review与索引，没有修改Rust/C++ production、manifest、workflow或tests，也没有运行Cargo、Miri、sanitizer、fuzz、native build、DLL unload或性能测试。Runtime与Editor source仍有其他Session在途修改，实施前必须重取unsafe inventory fingerprint。

## 128. Magic Constant / Sentinel / Threshold / Timeout / Capacity / Budget / Policy Convergence 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| production-like Rust inventory | 11,758文件 | 排除明显tests/generated/vendor/target并保守截断纯`cfg(test)`尾部；不是AST完整性证明 |
| 命名常量 | 7,224个const/static / 1,848文件 | 命名不等于拥有schema、单位、override、provenance或qualification |
| policy-like命名定义 | 1,791个 / 800文件 | Runtime 875、Editor 500、Plugins 292、Runtime Interface 72、App 29、Hub 14、Runtime Host 9 |
| literal signal | 94处numeric Duration / 58文件；60处fixed capacity / 30文件；21处range或take / 16文件 | 只作为owner复核入口，不能机械等同finding |
| Editor/Runtime设置authority | 6处SettingDefinition构造；13个公开Editor/Runtime setting key；Runtime ConfigManager为String到JSON | UI设置、runtime policy、project config与产品资格没有同一typed resolution链 |
| 重复shared contract | IBL 8x8x1、Mesh SDF 4..256、DDS 128/148、KTX2 80/24/6、point shadow 6、IK 4096、capture 64、frame demand 60秒 | 同一协议跨文件/类型重复，修改不能保证consumer同代收敛 |
| sentinel/MAX | usize/u64/Duration MAX及u32::MAX分散使用 | unlimited、invalid、missing、generation、no-budget与admission bypass语义混叠 |
| reference engines | Unreal CVar/Config/Scalability；Godot ProjectSettings；Bevy Resource；Fyrox QualitySettings；Unity GlobalSettings/ScalableSetting | 对照typed owner、分层override、metadata/restart、consumer-local resource和scalability schema |

本轮保留可用基础：大量局部常量已有明确命名；definition-bound格式值可以留在格式decoder附近；App到Runtime的typed RenderProfile链证明仓库已有可复用的强类型方向。问题不是“数字必须全部搬到中央文件”，而是shared contract、crate policy、local helper与definition-bound exemption四类尚无机器可识别的owner/placement，设置override与最终运行值也没有可追溯快照。

Runtime03、Runtime graphics/asset/system、Editor12及各具体domain报告继续拥有配置、格式、GPU、产品和性能finding；本篇不复制其P0。本篇建立`ConstantUseInventory -> SharedContract/CratePolicy/LocalHelper/DefinitionBoundExemption -> PolicyDefinition Registry -> project/user/session/CLI/platform/device layers -> ResolvedPolicySnapshot -> typed consumer -> PolicyObservation/PerformanceEvidenceReceipt -> Validation/Release`链，登记0项P0、36项P1、12项P2及M0-M6。详见`zircon_tooling/22-magic-constant-sentinel-threshold-timeout-capacity-budget-policy-convergence-review.md`。

本轮只新增review与索引，没有修改production、tests、settings、shader、format schema或manifest，也没有运行Cargo、GPU、Editor、网络、soak或benchmark。Runtime与Editor source仍有其他Session在途修改，实施前必须以AST/Cargo/shader/ABI schema重取inventory并逐consumer确认语义。

## 129. Failure Contract / Panic / Unwind / Error Propagation / Poison Recovery / Result Observability 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| production-like Rust scope | 11,498文件 / 约1,023,340行前缀 | 排除明显tests/benches/examples/fixtures/generated/vendor/target并截断纯`cfg(test)`尾部；父模块`cfg(test)`仍可能使少量无标记文件进入inventory |
| typed failure foundation | 451个error type定义候选 / 408处`derive(Error)` / 74处`catch_unwind` | 仓库已有typed error与unwind guard基础，但没有统一FailureDomain/ErrorCode/BoundaryAdapter契约 |
| fatal/invariant site | 24处`panic!` / 190处`unreachable!` / 69处`unwrap` / 950处`expect` / 243处assert族 | invariant、输入错误、资源缺失、worker失败和程序员错误使用同类终止原语，缺少机器可判定的fatality与product影响 |
| silent loss/default | 804处`.ok()` / 600处`let _ =` / 1,061处`unwrap_or_default` | 包含日志关闭、发布回滚、线程join、process wait、schedule/send等结果丢弃；数量是复核入口而非逐项finding |
| poison policy | 421处poison后`into_inner`继续 / 91处poison `expect`终止 | 同类共享状态在Editor、Runtime与Plugins采用相反恢复语义，且没有corruption、quarantine或degraded-state证据 |
| weak error boundary | 948个`Result<_, String>`候选 / 328文件；其中576个public-shaped / 251文件 | 字符串失败跨crate、任务、产品和FFI边界传播，无法稳定驱动重试、用户动作、遥测聚合或兼容迁移 |
| process terminal action | 2处`abort` / 2处`exit` / 2处task abort handle | Runtime动态库失败teardown的process abort具有soundness理由；其余仍需显式TerminationReason与receipt |
| reference engines | Unreal AssertionMacros/ValueOrError；Godot error macros/logger；Bevy ErrorContext/RenderErrorPolicy；Fyrox VisitError；Unity RenderGraph/UnifiedRayTracing error code | 对照recoverable/fatal分层、typed context、render-policy、serialization error与稳定错误码，而不是照搬单一宏体系 |

本轮保留已有正确方向：operation service对snapshot/prepare/apply panic做捕获并转成typed状态，动态runtime export与native plugin call存在unwind guard，World同时提供strict和checked资源访问，动态runtime失败teardown在无法证明worker/callback静默时选择abort具有soundness依据。问题不在于工程中绝对不能panic，而在于没有统一登记failure domain、fatality、boundary、recovery owner、observability与产品健康状态，导致同一故障在不同crate中可能panic、吞掉、默认化或继续使用poisoned state。

报告建立`FailureSiteInventory -> FailureDomainDefinition/ErrorCode Registry -> BoundaryAdapter -> FailureEvent -> RecoveryDecision -> Operation/Task/Product Outcome -> ProductHealthState/QualificationReceipt`链，登记0项P0、40项P1、12项P2及M0-M6。Runtime、Editor、Plugins、App、Hub与各具体domain报告继续拥有实际业务失败和产品恢复finding；本篇只拥有跨crate failure vocabulary、panic/unwind、poison、结果消费、边界映射与可观测性治理。详见`zircon_tooling/23-failure-contract-panic-unwind-error-propagation-poison-recovery-result-observability-review.md`。

本轮只新增review与索引，没有修改production、tests、manifest或workflow，也没有运行Cargo、产品进程、故障注入、soak或crash recovery验证。Runtime与Editor source仍有其他Session在途修改，且静态词法inventory不能替代AST/CFG/call-graph，因此实施前必须重取fingerprint并逐boundary确认语义。

## 130. Concurrency / Locking / Atomic Ordering / Blocking / Thread Lifecycle / Backpressure / Deadlock 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| production-like Rust scope | 11,485文件 / 约1,009,937行前缀 | 排除明显tests/benches/examples/fixtures/generated/vendor/target并截断纯`cfg(test)`尾部；不是AST/CFG或真实产品线程拓扑 |
| lock surface | Mutex 890/234文件；RwLock 98/20；Condvar 49/17；MutexGuard 289/130 | 锁已广泛使用，但没有canonical LockGraph、rank、reentry或wait/hold evidence |
| atomic surface | 551/114文件；Relaxed 250/65；Acquire 101/40；Release 40/25；AcqRel 46/21 | 独立counter可保留Relaxed；复合发布协议缺AtomicInvariant与model test |
| task/thread/runtime | JoinHandle 24/12；raw spawn 4/2；Builder 8/8；Rayon 14/5；Tokio runtime 18/16；block_on 21/13 | process同时拥有公共pool、独占runtime与subsystem worker，但没有resolved topology总预算 |
| channel/backpressure | explicit bounded 8/7；unbounded 12/10；blocking send 37/27；blocking recv 18/15 | imported短名会漏计；one-shot、latest/coalesced和ongoing stream必须按cardinality/bytes/age区分 |
| confirmed examples | asset subscriber持锁wake且使用unbounded stream；inspection cache六把RwLock分裂发布；network latency双Relaxed atomic | 三项均有具体source/caller状态证据，不是由词法数量直接推导 |
| concurrency validation | 有多线程回归；直接Cargo/CI/tool配置无Loom/Shuttle/Miri/TSan | OS调度重复测试不能证明所有interleaving、弱内存序或无死锁 |
| reference engines | Unreal limiter/heartbeat；Bevy pools/ECS executor；Godot worker pool；Fyrox task pool；Unity Graphics jobs | 对照slot/priority/timeout/lifetime/hang、dependency/access和JobHandle数据分区，不照搬表面API |

本轮保留已有正确方向：Runtime TaskPools与JobHandle、文本raster entry+byte budget、diagnostic sink、pipelined render一槽反馈队列、Editor play output及多处锁外subscriber snapshot。问题不是“Mutex/Relaxed/unbounded API全部错误”，而是缺少产品级execution domain、线程拓扑、锁序、atomic发布、channel cardinality、blocking和shutdown truth，导致正确局部机制无法约束其他owner。

报告建立`ConcurrencySiteInventory -> ExecutionDomainDefinition/ThreadOwnerManifest -> ProductThreadTopologyBudget -> LockGraph/AtomicInvariant/ChannelPolicy -> TaskScope/AdmissionTicket/ScheduledTask -> Completion/Failure/ShutdownReceipt -> ConcurrencyEvidenceReceipt -> ProductQualification`链，登记0项P0、40项P1、12项P2及M0-M6。Runtime02、Runtime04/05/11、Graphics、Editor09、Hub01、App02、Plugin08E与Tooling21/23继续拥有具体线程、资源、UI、GPU、网络、unsafe和failure finding；本篇不重复其P0。详见`zircon_tooling/24-concurrency-locking-atomic-ordering-blocking-thread-lifecycle-backpressure-deadlock-review.md`。

本轮只新增review与索引，没有修改production、tests、manifest或workflow，也没有运行Cargo、产品进程、GPU、network、model checking、TSan、stress或soak。当前source dirty且既有Editor/Hub/WOC/plugin验证阻断未变化；实施前必须按BuildSet重取AST/Cargo inventory、采集actual topology/contention并执行E4/E5资格。

## 131. Memory Allocation Domain / Budget / OOM / Pressure / Fragmentation / Pooling / Cache Residency 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| production-like Rust scope | 11,716文件 / 约1,032,404行前缀 | 排除明显tests/benches/examples/fixtures/generated/vendor/target并截断纯`cfg(test)`尾部；不能观察monomorphization、allocator/driver/third-party内部 |
| container/owner allocation signal | Vec capacity 491/339文件；Box 193/95；Arc 803/363；collect Vec 1,115/629；clone 10,382/2,371 | 只作为热路径、lifetime与slack复核入口，不能机械等同allocation或缺陷 |
| fallible/low-level signal | reserve 26/18；try_reserve 6/5；raw alloc 2/2；OutOfMemory 1/1 | try_reserve多项是自定义byte budget；真实fallible heap admission集中在少数plugin边界 |
| existing local foundations | ECS heap estimate；4 MiB inline command arena；shaped cache capacity估算；asset/Editor/text/IO/foreign byte budget；RHI/render stats | 可保留，但没有统一MemoryDomain、产品总预算、pressure/reclamation与snapshot |
| confirmed truth gaps | artifact eviction不计external Arc lease；inline arena clear保留峰值capacity；UiSurfaceNodePool无entry/byte/age cap | cache-owned、logical、actual live/resident语义不一致，pool可把峰值变成长驻 |
| allocator/OOM | production无global allocator/alloc-error hook；ECS raw growth调用handle_alloc_error | 不要求机械替换allocator，但allocator选择、fatal OOM和bulk pre-admission未绑定BuildSet/FailureDomain |
| observability | Windows WorkingSet/PeakWorkingSet、Coordinator RSS soak、少数allocation-count test | RSS不含完整GPU/child/tag归因；局部tests未形成全产品required hot-path gate |
| reference engines | Unreal FMemory/PlatformMemory/LLM/slack；Godot Memory/PagedAllocator；Bevy BlobArray/Table；Fyrox Pool；Unity RenderGraph pool/BuddyAllocator | 对照tag、memory class、scope、try allocate、trim与lifetime证据，不照搬allocator/API表面 |

本轮保留正确方向：ECS连续列与heap estimate、command arena、shaped-text保守估算、bounded keyed IO、asset/Editor cache和foreign-output admission，以及Animation/Sound/Editor/UI局部allocation tests。问题不是标准Vec/Arc或system allocator本身，而是大量分配、pool和cache没有进入同一domain、budget、pressure、failure与产品证据链。

报告建立`AllocationSiteInventory -> MemoryDomain/Tag Registry -> ProductMemoryBudget -> MemoryReservation/ExpansionPlan/AllocationScope -> Pool/Cache/SharedPayloadLease/Foreign-GPU ownership -> MemoryPressure Coordinator -> Reclaim/Degrade/Cancel/Reject/Fatal -> MemorySnapshot/ShutdownReceipt -> ProductQualification`链，登记0项P0、40项P1、12项P2及M0-M5。Runtime asset/ECS/graphics/UI、Editor、Runtime Interface05与Tooling07/15/21/22/23/24继续拥有具体功能、unsafe、policy、failure和concurrency finding；本篇不重复其P0。详见`zircon_tooling/25-memory-allocation-domain-budget-oom-pressure-fragmentation-pooling-cache-residency-observability-review.md`。

本轮只新增review与索引，没有修改production、tests、manifest或workflow，也没有运行Cargo、产品进程、heap profiler、allocator benchmark、OOM injection、GPU capture、stress或soak。实施前必须按BuildSet重取AST/MIR/Cargo inventory，并采集allocation trace、heap/RSS/VRAM snapshot、pressure/failure与fragmentation E4/E5证据。

## 132. Time / Clock Domain / Fixed Step / Determinism / RNG / Replay / Scheduling 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| production-like Rust scope | 12,170文件 / 约1,170,797行前缀 | 排除明显tests/benches/examples/fixtures/generated/vendor/target并截断纯`cfg(test)`尾部；只用于路由，不能机械计缺陷 |
| clock signals | `Instant::now` 385/125文件；SystemTime/UNIX 188/50；Duration constructor 331/103 | monotonic、UTC、profiling、simulation、input等domain没有统一taxonomy或注入边界 |
| time consumers | `delta_seconds` 176/43文件；`frame_index` 1,140/111；`RuntimeTimeAdvance` 17/9 | 当前深读确认普通Update使用raw real delta，Virtual pause/scale/clamp未控制产品游戏逻辑 |
| fixed transaction | Real -> Virtual -> Fixed accumulator -> `drain_steps` -> WorldDriver loop | Fixed在执行前预推进全部steps；子步共享最终clock，失败没有逐step commit/rollback truth |
| lifecycle/cadence | first tick从CoreRuntime构造计时；background cadence 1秒；resume无rebase | 加载、遮挡、暂停/恢复可形成discontinuity，当前只有Fixed受到Virtual最大delta保护 |
| schedule基础 | stable order/system ID/step rank、topological plan、worker deterministic key merge | 可保留；必须绑定BuildSet/tick context，simulation side effect进入commit journal |
| RNG | AI DefaultHasher；Particle CPU自定义u64 RNG/GPU hash；WOC Mulberry | 没有共享algorithm ID、master seed、stream key、draw counter、snapshot或migration |
| replay | 通用Runtime没有manifest/input journal/checkpoint/seek/divergence | WOC parity由Tooling11继续拥有；本轮建立引擎级clock/RNG/state replay合同 |
| currentness | revision `ae2be3d...`；dynamic session profile在途 | 报告`source_recheck_required: true`，未吸收其他session修改 |
| reference engines | Bevy Time/manual strategy；Unreal FApp/Tick/RandomStream/Demo；Godot timer sync/PCG；Fyrox loop；Unity Graphics history | 对照逐域clock、逐step advance、seed/state、checkpoint/scrub和render temporal consumer，不照搬单一API |

本轮保留Real/Virtual/Fixed三时钟、Virtual pause/speed/max delta、Fixed overstep、稳定schedule与worker merge、ordered ECS容器及局部显式seed。确认两个产品级P0：`WorldDriver`把raw real delta传入Update/PostUpdate，使Animation、script和游戏逻辑绕过Virtual；`drain_steps`在schedule前提交全部Fixed elapsed/frame，令各子步身份相同且失败状态虚报。

报告建立`ClockSource/ClockDomain Registry -> FrameTimeSnapshot -> WorldTimeController -> begin/commit SimulationTick -> Tick Effect Journal -> RandomStream Registry -> Canonical State Digest/Input Journal/Checkpoint -> Replay/Seek/Divergence Receipt -> Product Qualification`链，登记2项P0、40项P1、12项P2及M0-M6。Runtime08A/08C/08F、Runtime09H1、Runtime11A、Editor07与Tooling11继续拥有Physics、Animation、AI、render history、UI timer、Play控制和WOC parity局部finding；本篇不重复其计数。详见`zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md`。

本轮只新增review与索引，没有修改production、tests、manifest或workflow，也没有运行Cargo或产品进程。既有Editor/Hub/WOC/plugin动态阻断未变化且不能抵达时间语义；实施前必须先添加Virtual bypass与逐Fixed-step failure两组可失败回归，再按current BuildSet执行hitch/suspend/pause/scale/rate mutation、double-run、cross-thread-count和cross-platform资格。

## 133. Coordinate Space / Unit / Precision / Transform / Numeric Robustness / Large World 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| production-like Rust scope | 11,936文件 | 排除显式tests目录、`tests.rs`和examples；只用于路由，不是AST/CFG、类型流或缺陷计数 |
| shared math discipline | `Real` 2,544/339文件；`Vec3` 2,792/408；direct glam仅runtime 1/1、interface 10/1，其他家族0 | 单一math seam是真实基础，应保留并升级为versioned precision/space schema |
| mixed precision surface | raw `[f32;3]` 206/84文件；`f64` 949/255 | 大量mesh/GPU f32合理；reflection、animation、sound和world/render bounds确认阻断“只改alias”的迁移假设 |
| numeric checks | `is_finite` 1,582/544；inverse 28/19；normalize 311/156；EPSILON 634/255 | 已有大量局部防御，但没有统一unit/scale/condition/failure policy，不能机械按关键词修复 |
| transform correctness | scene拒绝nonfinite、zero-length quat和exact-zero scale | 非单位quat、near-zero scale、affine/shear分解、compose condition与fallible inverse/look-at/projection仍未闭合 |
| coordinate/unit contract | RH、+Y up、-Z forward可从代码推导；49个unit命中均为局部语义 | 没有versioned CoordinateSystem、World Unit、typed Position/Direction/Normal或import conversion receipt |
| large-world source truth | 11个origin/relative关键词命中逐条分类后为popup/temporal命名；没有world owner实现 | CPU absolute f32、GPU absolute current/previous matrix和absolute camera仍是产品路径；无cell/origin generation/rebase |
| persistence/ABI | Scene/asset/plugin/wire没有precision/coordinate/unit identity | precision profile必须进入schema、BuildSet、cache、snapshot、network/replay和migration admission |
| reference engines | Unreal LWC/tile+offset/double-float/translated view；Unity camera-relative；Godot real_t；Bevy Dir3；Fyrox UnitQuaternion | 组合吸收CPU range、GPU relative、build profile和typed invariant，不把任一引擎表面API当完整方案 |
| currentness | revision `ae2be3d...`；核心math/transform/render边界clean，邻接scene tests/schedule与Editor在途 | 报告`source_recheck_required: true`；实施前重取owner fingerprint并按profile执行E4/E5 |

本轮保留共享math alias、render precision seam、Local/World transform分层、finite写入校验、iterative hierarchy与animation quaternion normalize等正确基础。确认现有架构文档的“future f64主要只改alias/helper”不成立：reflection、animation、sound、bounds、persistence、plugin和BuildSet均有硬编码或缺失schema；`to_render_scalar`未来还缺range/post-cast finite检查。

报告建立`Precision/Coordinate/Unit/Numeric Schema -> typed WorldPosition/Direction/Normal/ValidatedTransform -> WorldOrigin generation/rebase transaction -> subsystem adapters -> per-view current/previous relative extract -> checked narrowing receipt -> GPU/temporal/product qualification`链，登记0项P0、40项P1、12项P2及M0-M6。Runtime05/08A/08B/08D/09B/09H1/17、Editor03与Tooling22/23继续拥有hierarchy、backend、graphics history、WOC world、gizmo、constant placement和failure局部finding；本篇不重复其P0。详见`zircon_runtime/23-coordinate-space-unit-precision-transform-numeric-robustness-large-world-review.md`。

本轮只新增review与索引，没有修改production、tests、manifest或workflow，也没有运行Cargo、GPU或产品进程。既有Editor/Hub/WOC/plugin阻断没有变化；实施必须先写invalid quaternion/scale/look-at/projection/inverse与precision schema negative tests，再按world range、hierarchy depth、backend、多view、rebase和current/previous frame做资格验证。

## 134. Stable Identity / Handle / Generation / Owner Epoch / Stale Reference / Exhaustion 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| production-like Rust scope | 11,936文件 | 排除显式tests目录、`tests.rs`和examples；只用于identity路由，不是AST/type-flow或缺陷计数 |
| public identity surface | 167个公开identity声明/127文件；74个数值tuple wrapper/45文件 | newtype基础广泛存在，但多数未机器声明Persistent/Live/Scoped/Sequence/Revision分类 |
| generation/allocator signal | generation/revision/epoch/token约3,308处/562文件；atomic fetch候选32处/29文件 | 信号多不等于缺陷；确认owner间存在checked、wrap、saturate、retire、overwrite和panic分裂 |
| positive foundations | UUID resource ID、ECS index+generation、HostRegistry、native context retirement、Render Graph builder generation、font snapshot、VM root lease | 保留并提升为共享schema/allocator/owner conformance，不推倒重建 |
| persistent identity | stable UUID由DefaultHasher生成；Scene load使用`max(EntityId)+1` | 算法无持久版本/迁移；MAX文档可panic或回绕到0，必须在admission阶段拒绝 |
| live owner qualification | InternalEntity无World owner；RHI handle无Device epoch；ABI token依赖外部session参数 | 代际slot只阻止同owner常见stale，不能解决cross-world/device/session alias |
| exhaustion source truth | particle在MAX后复用并覆盖live handle；message checked add后expect；多owner普通/饱和/atomic递增 | 合法末端只能是拒绝、slot retirement或受控owner epoch rollover |
| persistence boundary | 多个ephemeral handle可serde，缺stable-to-live remap和identity manifest | serializer默认deny ephemeral，load transaction建立stable remap后原子publish |
| reference engines | Unreal WeakObjectPtr/ObjectHandle；Bevy Entity；Fyrox pool Handle；Godot RID owner/ObjectID | 组合吸收live/persistent分层、owner validation和typed pool，不复制单一位布局 |
| currentness | revision `ae2be3d...`；identity核心路径clean，邻接Scene tests/Editor等在途 | 报告`source_recheck_required: true`；实施前重取owner fingerprint并执行小位宽model test |

本轮保留资源typed UUID、ECS内部代际slot、HostRegistry显式耗尽错误、native context slot retirement、Render Graph builder generation、字体database generation与root lease。确认缺失的是引擎级identity taxonomy、OwnerKey/Epoch、codec eligibility、allocator/exhaustion contract与统一stale/wrong-owner错误，而不是“所有handle都需要改成UUID”。

报告建立`Identity Schema Registry -> StableIdAlgorithm v2/migration -> OwnerKey/Epoch + Slot/Generation -> checked allocator/retirement -> stable-to-live remap -> ephemeral serialization deny -> ABI/error mapping -> cross-owner/near-exhaustion/soak qualification`链，登记0项P0、40项P1、12项P2及M0-M6。Runtime05、Runtime08B/08D/08E/08F、Runtime09A、Interface01/02、Editor03与Tooling22/23继续拥有各自world/subsystem/ABI/persistence/failure局部finding；本篇不重复其计数。详见`zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md`。

本轮只新增review与索引，没有修改production、tests、manifest或workflow，也没有运行Cargo或产品进程。实施必须先固定stable UUID测试向量与MAX/0/duplicate Scene load negative test，再以可缩小位宽的allocator state machine覆盖reuse、wrap、wrong-owner、teardown与restart。

## 135. Filesystem / Path / URI / VFS / Mount / Watch / Sandbox / Atomic I/O 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| production-like Rust scope | 11,937文件 | 排除显式tests目录、`tests.rs`和examples；只用于path/filesystem路由，不是AST/type-flow或缺陷计数 |
| path signals | `PathBuf` 1,834处/393文件；`Path` 2,234处/561文件 | OS path使用广泛，但没有logical/physical/display/persistent/file-identity公共taxonomy |
| filesystem/open candidates | 777处/203文件 | 不代表203文件全部违规；确认缺backend-only allowlist、bypass owner和机器可检查调用层 |
| path operations | canonicalize 49处/24文件；read_dir 45处/34文件；strip_prefix 260处/156文件；starts_with 406处/198文件 | containment与投影规则跨Project、transaction、Hub、Editor等owner重复实现 |
| positive project path foundation | operation/display path、junction/SUBST/symlink与未创建尾段解析、Windows不稳定相对形态拒绝 | 保留并抽取共享physical identity owner，不退回lexical normalize |
| positive write/watch foundation | atomic/durable transaction、journal/recovery、bounded ingress/pending、reconciliation与generation publication | 保留后半段，在secure open与watch mapping前端补齐合同 |
| false service surface | `ResourceIo`只有定义/re-export且无implementation/consumer；`AssetIoDriver`为空却immediate注册 | 不能以trait/descriptor存在宣称异步I/O完成；需真实provider、queue和consumer |
| source/mount architecture | fixed `Res/Library/Package/Builtin/Memory` enum；无VFS/Mount Registry/source instance | 建capability、priority、generation、collision、lease、unmount/quiescence与provider routing |
| URI/encoding | locator使用host Path parser；source/watch URI和AssetManager path边界使用lossy string | Interface02拥有DTO parser；本篇拥有portable grammar、OS codec、conversion receipt和mount mapping |
| runtime asset path | root/prefix/`.`/`..`被静默丢弃；invalid asset root环境配置panic | strict fallible relative path与typed startup error，禁止重绑和panic |
| secure containment | symlink/reparse/canonical containment已存在；check与open分离且hard link未纳入 | 需要root-handle-relative no-follow open/create与opened file identity policy |
| watcher mapping | 只有Rename Both形成rename；mapping失败/异常路径数静默返回空 | mapping不确定必须进入已有reconciliation链并携mount/source generation |
| reference engines | Unreal IPlatformFile/package mount；Bevy AssetSource；Fyrox ResourceIo；Godot File/Dir Access | 组合吸收provider、source、mount与访问域分层，不复制单一全局VFS类 |
| currentness | revision `ae2be3d...`；10个关键path/VFS/watch/Hub文件clean，邻接区域在途 | 报告`source_recheck_required: true`；实施前重取fingerprint并运行跨平台path/link/rename corpus |

本轮保留ProjectPaths物理身份、project/package root admission、safe scanner、atomic/durable transaction、bounded watcher、generation publication和artifact read validation。确认缺失的是引擎级Path Schema、Physical Identity、FileSystem Provider、Source/Mount Registry、secure opened capability、watch mapping outcome、direct filesystem治理与产品资格矩阵，而不是“所有`std::fs`都必须删除”。

报告建立`Path/URI schema -> shared physical identity -> Local FileSystem provider -> Source/Mount Registry -> secure root capability -> durable writer -> watch outcome/reconciliation -> I/O scheduler/error/telemetry -> product qualification`链，登记0项P0、40项P1、12项P2及M0-M6。Runtime04、Interface02、Plugins01、Hub01、Editor02与Tooling03/08/09继续拥有artifact/DTO/plugin/Hub/document/distribution局部finding；本篇不重复其计数。详见`zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md`。

本轮只新增review与索引，没有修改production、tests、manifest或workflow，也没有运行Cargo或产品进程。既有Editor/Hub/WOC/plugin阻断没有变化；实施必须先固定portable URI、non-UTF8、case/Unicode、UNC/SUBST/junction/symlink/hardlink、rename loss、mount change与power-loss fixture，再逐ProductRole准入provider。

## 136. Security / Principal / Credential / Trust / Capability / Cryptography / Supply Chain / Audit 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| production-like SourceSet | 12,542文件 / 约1,475,370行 | 8个产品/代码family；用于security owner路由，不是漏洞或实现计数 |
| identity/permission signals | 3,070处 / 656文件 | capability、permission、principal与auth词汇广泛，但没有共同Principal/SecurityContext/Decision schema |
| trust/signature signals | 1,434处 / 271文件 | 包含类型/图形signature、owner revocation和声明文本；SHA/hash不能外推为publisher trust |
| secret/credential signals | 58处 / 14文件 | Tray有局部SecretString，TLS key、HTTP header/body、Hub/signing路径没有统一sensitive metadata与lease |
| transport security | 1,623处 / 24文件 | Rustls/root/pin基础可保留；HTTP pin关闭标准验证，WebSocket pin无peer验证 |
| native execution | 10处 / 6文件 | Runtime/App/RenderDoc动态加载点；plugin loader在digest/signature/trust前执行`Library::new` |
| package/release trust | hash旁车 + external signer audit | `.sig`无signature bytes/key/publisher chain且无runtime consumer，不能投影Signed/Trusted |
| Hub/Web | Tauri最小window permission；remote功能disabled | 保留fail-close；CSP为null，remote内容启用前需origin/navigation/download/sanitization/IPC隔离 |
| local tool auth | Coordinator opaque session/CSRF/loopback/role/audit；Tray Debug redaction | 可复用decision/audit形状，但不得升级成本地外的账号、plugin或game principal authority |
| CI dependency governance | root/plugin `cargo-deny` advisories/bans/licenses/sources | 正向基础；仍缺完整product closure、action commit pin、SBOM/provenance与SecurityQualification消费 |
| reference engines | Unreal crypto/key/signed pak/TLS/auth；Godot Crypto/TLS/encrypted file | 吸收typed provider、验证顺序与独立owner；Bevy/Fyrox lifecycle及Unity Graphics缺失不构成安全完成度豁免 |
| currentness | revision `ae2be3d...`；31个关键文件clean | fingerprint `d9954d...2639c`；邻接源码在途，实施前重取dependency、decision path与attack corpus |

本轮保留Script host-call admission、Rustls、Hub remote fail-close、Coordinator browser auth/audit、Tray redacted Debug与CI cargo-deny。确认缺失的是跨产品Security Control Plane、Principal/SecurityContext、CredentialLease、typed CapabilityDecision、TrustReceipt、CryptoPolicy、SensitiveField与Audit/Qualification schema，而不是“完全没有安全代码”。

报告建立`Principal + Source + Resource + Action -> SecurityContext -> policy generation -> credential/trust decision -> capability lease -> domain adapter -> effect/audit receipt -> BuildSet-bound SecurityQualification`链，登记0项P0、40项P1、12项P2及M0-M6。Plugins01、Runtime07/08E/25、Hub03与Tooling03/06/09/13/16/17/21/23继续拥有native/script/network/filesystem/Hub/release/repo/unsafe/failure局部finding；本篇只拥有O15公共合同和总体验收。详见`zircon_tooling/26-security-principal-credential-trust-capability-cryptography-supply-chain-audit-review.md`。

本轮只新增review与索引，没有修改production、tests、manifest或workflow，也没有运行Cargo、产品或攻击测试。实施前必须先禁用shipping invalid-cert pin路径、保持WebSocket pin/Hub remote为Unavailable、给`.sig`去除密码学完成度含义，再固定wrong-principal/scope/pin/signature、expired/revoked、malicious plugin/archive、secret canary、TOCTOU与sandbox fault矩阵。

## 137. Version Domain / Schema Compatibility / Support Window / Migration / Deprecation / Upgrade-Downgrade 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| production-like SourceSet | 13,093文件 / 约2,054,144行 | 9个family；排除显式generated/vendor/dist/fixture/test/bench，用于owner路由而非缺陷计数 |
| version fields | 1,505处 / 498文件 | Schema/format/protocol/ABI/artifact/policy/engine compatibility广泛存在，但没有共同VersionDomain与typed version vector |
| compatibility signals | 564处 / 136文件 | bool/string/error各自判定；没有CompatibilityDecision、reason code、policy generation或support window |
| migration signals | 1,029处 / 212文件 | 包含真实migration、upgrade词汇与普通转换；确认serde/asset/UI/VM/DB形成互不相认的执行岛 |
| deprecation/legacy | 精确deprecated/obsolete 20处/14文件；`#[deprecated]` 0；legacy 735处/123文件 | 没有deprecated-since/replacement/removal/usage budget，兼容分支缺退出治理 |
| common contract exact search | 0 | `VersionDomain/CompatibilityDecision/MigrationReceipt/SupportWindow/DeprecationRegistry`均不存在 |
| Cargo identity | 160份manifest：157 workspace继承、3 explicit、0 missing | package version一致是正向基础，但不能替代EngineBuild/CompatibleWithBuild/BuildSet |
| plugin compatibility | 39份plugin manifest、41个distribution记录 | 全部固定ABI 3与`>=0.1,<0.2`，没有target/profile/behavior/schema/BuildSet验证矩阵 |
| positive serialization foundation | SchemaId+version envelope、future reject、linear chain validation、typed error | 保留；Interface02继续拥有wire/unknown-field/reader-writer实现，本篇只拥有公共版本域与资格 |
| positive migration foundation | Asset DryRun/Apply、preflight/formal reader、journal/rollback/recovery/fault injection；Coordinator 65步DB migration | 保留并适配公共Plan/Receipt，不建立吞并所有domain的单一迁移器 |
| concrete split | native/runtime/tooling三套兼容解析；Project单一format；workspace exact-current；settings拒绝legacy | 收敛decision与支持窗口，局部owner继续实现各自adapter与事务 |
| reference engines | Unreal CustomVersion/Package SavedBy+CompatibleWith；Godot resource format gate；Unity migration step/tests；Bevy migration guide | 组合吸收版本域、producer/compatible双身份、幂等步骤与breaking-change治理，不复制历史包袱 |
| currentness | revision `ae2be3d...`；31个关键文件clean | fingerprint `4ba51804...2df42`；报告`source_recheck_required: true`，实施前重取版本catalog与兼容矩阵 |

本轮保留shared serialization、asset migration transaction、UI migration report、VM state type schema、workspace version inheritance与Coordinator数据库migration。确认缺失的是跨产品Version Domain Registry、VersionVector、Reader/Writer SupportWindow、typed CompatibilityDecision、MigrationCatalog/Plan/Receipt、Upgrade Coordinator与Deprecation Registry，而不是“完全没有版本字段或迁移代码”。

报告建立`VersionDomain Registry -> VersionVector/BuildIdentity -> SupportWindow -> CompatibilityDecision -> MigrationCatalog/Plan -> domain executor -> MigrationReceipt -> deprecation/qualification`链，登记0项P0、40项P1、12项P2及M0-M6。Interface01/02、Runtime04/05/07/24、Editor13/24、Plugins01与Tooling04/09/12/20继续拥有局部ABI、wire、asset/scene、Editor、plugin、release、fixture与Cargo finding；本篇只拥有O03公共合同和整体验收。详见`zircon_tooling/27-version-domain-schema-compatibility-support-window-migration-deprecation-upgrade-downgrade-review.md`。

本轮只新增review与索引，没有修改production、tests、manifest或workflow，也没有运行Cargo或产品。实施必须先冻结VersionFieldInventory、EngineBuild/CompatibleWithBuild、plugin compatibility golden corpus和N-2/N-1/current/future reader-writer矩阵，再接入effectful migration。

## 138. LiveOps / Feature Flag / Remote Config / Segment / Experiment / Patch / DLC / Crash Control Plane 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| Workbench产品宣称 | 8张DesignSpec + 8张PNG | Feature Flags、Remote Config、Telemetry Query、Patch Planner、DLC Catalog、Crash Symbolication、Player Segment与Experiment Console均以生产产品形态展示 |
| production exact owner | 0 | 对应ID只出现在preview renderer、verifier和文档；Editor、Runtime、Plugin、Hub没有capability、document、command/operation、provider或runtime consumer |
| renderer effect surface | 0 | `design.js`没有event listener、fetch/XHR/WebSocket、storage、timer或signature路径；固定值不能成为live data、操作成功或发布证据 |
| 固定业务状态 | `Live v42`、12 flags、42K users、3 experiments、184 crashes、92% resolved | 必须先降级Prototype/Unavailable；静态截图和自签candidate不能越级成为LiveOps authority |
| 本地可保留基础 | settings/profile、plugin package、download/export、diagnostic/crash局部机制 | 它们分别由Editor12、Plugins01、Tooling03/07/09/14/26/27等owner治理，不能改名拼装成LiveOps完成度 |
| Unreal对照 | OnlineHotfix/Update、InstallBundle、GameFeatures、AssetManager、Analytics与CrashReport分owner | 吸收non-executable content、typed lifecycle/result、dependency/install/mount/activate、primary asset/chunk与analytics/crash边界；不复制其历史API形状 |
| Godot/Bevy/Fyrox对照 | PCK add/remove/encrypt；AssetSource/AssetServer/Remote registry；ResourceManager/hot reload/event | 支持package/source/resource lifecycle与更新事件的分层判断；均不自动证明完整LiveOps平台 |
| Unity Graphics边界 | analytics enabled gate/event metadata/build stripping | 本地镜像只支持图形包analytics边界对照，不能外推Unity完整Editor/Services/LiveOps能力 |
| currentness | revision `ae2be3d...`；26个Zircon选取文件clean | fingerprint `b36f6ae...45da7`；53个Zircon/reference选取文件共40,378行，实施前重取consumer、policy与delivery图 |

本轮确认缺失的是LiveOps Authoring/Control Plane与Runtime Policy Contract：signed immutable policy snapshot、deterministic evaluation、review/publish/rollout、consent-qualified segment/experiment，以及对BuildSet、entitlement、install、telemetry和crash canonical owner的只读编排。Patch/DLC/Crash不能在Editor46重新实现第二套build、package、install、telemetry或symbol authority。

报告登记6项P0、72项P1、12项P2及36个验收门，按`policy identity/schema -> signed immutable snapshot -> deterministic evaluator -> review/publish -> staged rollout -> consent-qualified experiment -> canonical delivery/crash orchestration -> source/build-bound evidence`收敛。详见`zircon_editor/46-liveops-feature-flag-remote-config-segmentation-experiment-patch-dlc-crash-control-plane-authoring-review.md`。

本轮只新增review与索引，没有修改production、tests、manifest或workflow，也没有执行联网LiveOps、patch、DLC或crash服务。无签名policy、provider、consumer与qualification时，八张入口必须保持不可用。

## 139. Documentation / API Reference / Plan Currentness / Link / Source Trace / Knowledge Publication 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| tracked docs tree | 5,329个条目，5,328个存在，1,148,609,716 bytes | 文档素材规模很大；缺ContentClass、owner、currentness、publication与retention统一合同 |
| plan tree | 2,084个tracked条目，2,083个存在；2,079个Markdown中2,078个存在 | `docs/plans/index.md`只路由5组；绝大多数plan没有机器可判review/implementation/recheck状态 |
| structured path gate | 2,649篇frontmatter文档、78,359 path、692 violation、242篇受影响 | 667 missing、22 repository escape、3 absolute；required validator当前exit 1，必须修复而非降门 |
| ordinary Markdown links | 2,477个local candidate；61个唯一missing target、64次、27篇 | regex词法下界，不含完整AST/anchor；现有validator不覆盖普通link graph |
| Cargo publication identity | 162份manifest、159个package | readme/documentation/repository/homepage/docs.rs metadata全部0 |
| Rust API lexical lower bound | 12,485个`.rs`；23,636个public item中1,445个前邻`///`，约6.1% | 不是语义coverage；足以确认PublicApiSet、missing-doc policy、rustdoc/doctest publication缺失 |
| entrypoint/workflow | 根README与`docs/index.md`不存在；3份workflow无docs lane | 需要角色入口、generated index、versioned DocumentationBuild与release promotion |
| temporal/currentness | 写入Tooling28前的3,240篇Markdown中2,641篇命中date/current/fresh/pass之一 | archive/plan合法命中；问题是class/claim/evidence未分离，不能机械删词 |
| positive foundation | frontmatter source mapping、`check_conventions.py` resolver/JSON、编号计划状态字段 | 保留并扩展为DocumentId、ContentClass、Source/Link/Symbol Graph与PublicationReceipt |
| reference engines | Unreal entry/source/API archive/locales；Bevy rustdoc workflow；Godot class generator；Fyrox crate docs；Unity package docs | 组合吸收入口、生成、版本、TOC、example和发布边界，不外推线上质量或性能 |
| currentness | revision `ae2be3d...`；29个关键文本输入clean | fingerprint `cd37e226...1d46`；本轮仅执行docs validator，不修改production/CI/manifest |

本轮确认缺失的是工程级Knowledge Publication控制面，而不是“没有Markdown”。规范reference、guide、plan、EvidenceReceipt、archive、DesignPrototype与Generated API必须有不同currentness和发布政策；自由文本`current/passed/fresh`不能继续替代source/build-bound EvidenceSet。

报告建立`DocumentSource/PublicApiSet/ExampleManifest/EvidenceReceipt -> ContentClass/DocumentSchema -> Source/Link/Symbol Graph -> Currentness/Visibility -> DocumentationBuild -> validators -> immutable artifact/PublicationReceipt -> versioned publish/search/rollback/retire`链，登记1项P0、56项P1、12项P2及36个验收门。Tooling01/12/13/17/20/27继续拥有CI、archive、audit、SourceSet、Cargo graph和Version Domain；本篇只拥有文档内容与发布控制面。详见`zircon_tooling/28-documentation-api-reference-plan-currentness-link-source-trace-knowledge-publication-review.md`。

本轮只新增review与索引。692项structured path violation、61个lexical missing target下界和API publication缺口尚未修复；实施前先冻结typed baseline并阻止新增，不能删除validator、批量删frontmatter或把required门改成warning。

## 140. Rust Module Boundary / Root Entry / Large File / Declaration-Behavior / Folder Topology 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| tracked Rust source | `zircon_*`、`examples/`、`tools/`下17,263个tracked `.rs` | 必须由Cargo metadata与auxiliary workspace registry生成SourceSet，不能用crate名前缀猜scope |
| manual production-like lower bound | 排除明确test/fixture/bench/generated后11,958个、1,315,926行 | 词法候选，不是同量缺陷；正式门需要Rust AST/cfg/target分类 |
| physical budget | `>=800` 87、`>=900` 32、`>=1000` 13、`>=1100` 5 | 13个超限文件逐读确认有手写行为或内联测试边界，违反GEN-S4 |
| inline test ownership | 13个超限文件中9个含大型`mod tests` | physical debt与production behavior分别报告；先拆TestOwner不能冒充功能/性能修复 |
| root/binding candidates | 24个`lib.rs/main.rs/mod.rs/binding.rs`不少于300行 | contact-shadow/codegen/profiling/static-index/scaffold/Editor binding有行为；render mod与GPU binding是反例/审查项 |
| folder topology | 163个目录至少12个direct manual Rust文件；最大81/60/47/42 | 用domain/fanout/co-change审查，不机械按文件数判失败 |
| Runtime audit | 11个hotspot、3个owner debt、`migration-debt-present`、exit 0 | 漏examples/plugin workspace/tools；SourceSet与required exit未闭合 |
| Editor audit | 30项debt、`migration-debt-present`、exit 0 | 28个test budget、1个UI root、1个duplicate tree；不检查root/declaration behavior |
| target control plane | SourceUnitClassifier、FileRole、ModuleBoundaryGraph、DomainOwner、BoundaryWaiver、Refactor/Structure Receipt | 结构Finding与功能/性能Evidence分开，不允许`partN`机械切块 |
| reference engines | Bevy/Fyrox facade；Godot/Unreal declaration-implementation；Unity cohesive subsystem | 组合吸收owner/cohesion/visibility/waiver，不复制文件尺寸或据此宣称性能 |
| currentness | revision `ae2be3d...`；55个输入fingerprint `9303e0a...916860` | `source_cubemap.rs`取证时dirty；实施前重取SourceSet、AST与相邻修改 |

本轮确认缺失的是全仓Rust Source Architecture Gate，而不是“没有拆过文件”。现有Runtime/Editor/Runtime15门的范围、阈值、test/generated分类与exit语义不一致，无法共同证明GEN-S3/S4。行数只触发review，root/binding/declaration/behavior职责必须由语法图和DomainOwner判断。

报告登记0项P0、48项P1、12项P2及36个验收门，按`Cargo-resolved SourceSet -> FileRole -> Rust syntax/module/public-symbol graph -> DomainOwner/policy/waiver -> required FindingSet -> RefactorTransaction -> Structure/Behavior/Performance receipts`收敛。Tooling13/17/20继续拥有runner、repository SourceSet与Cargo graph；各App/Runtime/Editor/Plugin报告继续拥有拆分后的功能与性能。详见`zircon_tooling/29-rust-module-boundary-root-entry-large-file-declaration-behavior-folder-topology-review.md`。

本轮只新增review与索引，没有修改Rust、Cargo、tests、CI或审计器。13个超限、87个800行预警和30项Editor debt仍未修复；不得上调阈值、把红finding改warning、机械切文件或在dirty `source_cubemap.rs`上覆盖相邻修改。

## 141. Cross-Language Source Architecture / Entry / Service / Schema / Generated-Test / Folder Topology 物理与语义范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| tracked non-Rust source | 排除`dev/docs/vendor/target/node_modules`后2,684个、684,736行、29,022,345 bytes | 必须投影language/package/build target/FileRole，不能由临时扩展名脚本长期充当SourceSet |
| manual lower bound | 1,496个manual non-test/non-generated、383,212行 | 词法保守下界；297个`>=300`、50个`>=800`、23个`>=1000`只触发owner审查 |
| generated/test roles | 158 generated、其余1,030 test/fixture候选；30个generated Zr位于非`generated/`目录 | 需要GeneratedSourceCatalog/Receipt和parser-aware TestOwner，不能只靠路径猜测 |
| Zr world root | `world/state.zr` 68,730行、538 imports、约1,430函数候选/232 public函数 | world/schema/default/command/codec/offline rule/selfTest混装，必须按Runtime12-20功能owner hard cut |
| Python control plane | Coordinator 62个root Python；7个核心service class跨度1,662至2,971行 | 已有子包/测试可保留，但application/command/transaction/recovery/projection边界未穿透 |
| PowerShell/JS tooling | 5个大型PowerShell入口AST可解析；WOC tools 368个direct MJS；prototype JS/CSS 9,183/3,620行 | root command变薄、generator按domain分包、prototype冻结为证据，不成为production authority |
| Web schema | Hub types 776行/47 interfaces，fallback data 681行；Coordinator validation 553行/44函数 | API schema/validator/page/demo fixture需分owner；局部strict/check存在但根CI未执行 |
| Native/shader | C ABI header 329行/23 structs，query cpp 886行；post-process WGSL 910行/53函数/7 entrypoints | 建立ABI layout catalog与ShaderSourceGraph；按domain/pass拆分且保留ABI/binding/performance |
| required quality matrix | 根CI无完整Node、Python quality、PowerShell analyzer、Zircon-owned C++ static analysis、WGSL全变体或Zr结构门 | 用typed FindingSet/receipt接入required runner；typed skip不能冒充执行成功 |
| reference engines | Unreal AutomationTool/Build.cs；Unity asmdef/RenderGraph；Godot SCsub/Python；Bevy/Fyrox shader owner | 组合吸收module/task/assembly/pass边界；不复制大文件尺寸或据此宣称性能 |
| currentness | revision `ae2be3d...`；100个输入fingerprint `95f0a0c...05f65` | `validate-matrix.ps1`取证时dirty、18篇输入报告既存untracked；实施前重取SourceSet/AST/build graph |

本轮确认缺失的是按语言校正的Cross-Language Source Architecture控制面，而不是“所有非Rust文件都要缩短”。migration history、shader library、CSS和schema declaration允许经cohesion/performance证明的waiver；巨型application/service、混域package root和内嵌test不能用相同理由豁免。

报告登记0项P0、64项P1、16项P2及44个验收门，按`Repository SourceSet -> CrossLanguageSourceUnit/FileRole -> parser/build graph -> owner/policy/waiver -> required FindingSet -> Generated/Command/Shader receipts -> RefactorTransaction -> behavior/performance evidence`收敛。Tooling29继续拥有Rust专项；Tooling05/06/17/19/21/27与App/Runtime/Hub报告继续拥有各自功能语义。详见`zircon_tooling/30-cross-language-source-module-boundary-large-file-entry-generated-test-folder-topology-review.md`。

本轮只新增review与索引，没有修改Zr、Python、PowerShell、JS/TS/CSS、C++、WGSL、tests、CI、package或manifest。23个千行热点、30个目录外generated leaf和跨语言required-matrix缺口仍未修复；不得用wrapper、compat路径、机械切块或更高阈值掩盖聚合owner。

## 142. Declarative Project / Asset / UI / Scene / Manifest / Schema / Generated Artifact 物理权威范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| focused declarative/artifact set | 715个文件、11,809,045 bytes | 317 ZUI、277 ZRP、38 ZRO、58 ZMeta、14 ZMaterial、6 ZShader、1 ZPreset、4 ZrAnim；必须先按Format Catalog区分source、derived、package、cache与generated evidence |
| ZUI source documents | 317个均可解析；295个production V2（205 component、83 view、6 style、1 theme tokens） | 317个ID当前唯一、1,107个production引用当前可解析是正向基础；默认V2、重复loader、重复ID静默择胜与产品cache currentness仍未闭合 |
| project/scene/settings | active examples仍有V1 project、template为V2；4个scene均无显式schema version；template `settings.toml`实际为严格JSON | 需要Project/Scene/Settings各自canonical reader-writer、stable reference、migration与真实文件类型，不得让扩展名和内容语法分裂 |
| scene identity/reference | Eastbrook scene 4,466行、268 entities；262个project reference为零GUID | parser把未知字段压入扁平rest并手工映射known reference；缺schema-owned identity/reference graph及重命名、搬迁、合并资格 |
| import metadata | 58个ZMeta全部为V7、553 entries且严格拒绝未知字段；52个root digest只有16字符，1个64字符，5个缺失/空，3个mtime为0 | schema严格是可保留基础；digest算法/宽度、timestamp语义、source/importer/settings/dependency provenance仍需规范化 |
| material/shader/animation | 14个ZMaterial和6个ZShader均显式V2；4个ZrAnim为binary | source reader仍把缺失version默认成当前版本；binary需要magic/version/endian/checksum/bounds与reader-writer corpus，不得将当前样本成功外推为兼容合同 |
| ZRP/ZRO/compiler artifacts | 276个ZRP为四字段JSON，1个WOC ZRP为两行TOML；38个ZRO形成28个hash/6组重复、1,922,113 duplicate bytes | ZRO tracked但ignored、两份CLI manifest含绝对`E:\Git`路径、外部`zr_vm` source dirty且manifest max version与loader/tests不一致；必须切断隐式外部/本机权威 |
| generated Tauri schema | 8个schema文件、4个unique hash；desktop/windows四份相同，ACL两份相同，capabilities不同 | 用generator identity、input digest、toolchain、consumer与GeneratedArtifactReceipt治理复制，不以checked-in重复文件充当currentness |
| structured syntax check | 316个`.json`均可解析；241个`.toml`中仅Editor settings刻意为JSON envelope | 语法通过只证明局部parse；扩展/format id/schema/version/semantic validation必须同源并进入required gate |
| reference engines | Unreal AssetRegistry/DDC；Godot text resource UID；Bevy AssetMeta/ProcessedInfo；Fyrox Visitor；Unity `.asset`/`.meta`/package manifest | 组合吸收稳定身份、import metadata、dependency hash、transaction与版本边界；不复制内部格式或据此宣称性能领先 |
| currentness | revision `ae2be3d...`；98个输入路径0 missing/0 duplicate；fingerprint `d5bfa1e4...3c10e9` | 外部`E:\Git\zr_vm`为dirty source；实施前须重取Format Catalog、reader/writer graph、artifact hash与BuildSet |

本轮确认缺失的是declarative format与generated artifact的物理权威控制面，而不是“格式完全不可用”。ZUI V2、ZMeta V7严格解析、ZMaterial/ZShader显式版本、UI compiled package和局部staging validator均为可保留基础；问题在于它们没有被同一Format Catalog、BuildSet、identity/reference graph与reader-writer-migration资格链约束。

报告登记0项P0、52项P1、12项P2及40个验收门，按`RepositoryContentManifest -> FormatCatalog/ArtifactRole -> canonical reader-writer -> DocumentId/ReferenceGraph -> Import/Derived/Package Receipt -> Version/Support/Migration matrix -> BuildSet/currentness -> required compatibility corpus`收敛。Runtime04/05、Editor04/23、App07与Tooling03/05/08/17/27继续拥有各自格式语义、authoring workflow、构建分发与版本治理；本报告只拥有跨格式物理资格。详见`zircon_tooling/31-declarative-project-asset-ui-scene-manifest-schema-generated-artifact-physical-authority-review.md`。

本轮只新增review与索引，没有修改asset/project/scene/UI source、loader、writer、compiler、cache、generated schema、tests、CI或外部Zr VM。混合语法、默认版本、重复blob、绝对路径和隐式外部authority均仍未修复；不得通过改扩展名、忽略parse失败、复制generated文件或把当前版本设为默认来掩盖兼容与provenance缺口。

## 143. Hot Path / Algorithmic Complexity / Data Movement / Batching / Cache Locality 性能治理范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| production-like Rust source | 11,993文件、1,340,621行、46,909,799 bytes | 全仓性能治理不能由Runtime07的少量固定source list代表 |
| lexical candidate inventory | clone 12,790、collect 4,452、sort 578、format 6,438、Vec构造9,021、map构造1,513 | 只发现candidate；不含cadence/payload/optimizer/workload，禁止自动定性或机械改写 |
| blocking/shared-state candidates | filesystem I/O 631、lock 1,100、blocking 231 | 必须由owner区分cold/maintenance/error与frame/audio/UI hot path |
| positive instrumentation | profile frame/scope/counter已有19/368/375处；reserve/with_capacity 1,190处 | 保留现有观测与容量基础，通过HotPath/Metric ID接入资格，不重写所有宏/容器 |
| Runtime07 historical dynamic evidence | 双次Vampire 30.8944/33.9832 FPS、116 draws、偏差9.521868%；trace与ECS/extract基线接受 | 是有效历史BuildSet/workload证据，不外推当前dirty source、新plugin或其他产品规模 |
| Runtime07 current structure audit | 40/46 source、90/91 test owner、219 anchors中35 missing、11 large-file hotspots/3 debt、7 risks | 红项混合asset submodule误报、scheduler/extract snippet漂移与animation metric真实handoff缺口；必须typed分型 |
| world projection | `node_records()`每次分配、全实体投影并排序；frame/tick语义spot-read命中Physics/Nav/AI等 | cold serialization/inspection API可保留；frame consumer迁移typed borrowed/incremental projection |
| local positive cost contracts | QueryState/extract cache、scheduler batch bytes、asset worker wall/copy/queue、animation/sound/native/UI allocation tests | 迁移为catalog关联的Workload/CostContract/Observation，不把局部绿色冒充全产品资格 |
| reference engines | Unreal Array/MemStack/ParallelFor、Bevy ECS Table、Godot LocalVector/RID、Fyrox Pool/Sparse/TaskPool、Unity RenderGraph pool/compiler | 组合吸收容量、scratch、dense storage、generation、batch、compile-plan与reuse约束；不复制实现或声称整机领先 |
| currentness | revision `ae2be3d...`；77个输入0 missing/0 duplicate；fingerprint `6364601e...d4e7` | 当前worktree有相邻dirty source；实施前重取SourceSet、typed audit、workload和BuildSet |

本轮确认缺失的是跨仓HotPath成本与资格控制面，不是“没有任何profile/性能实现”。Runtime07的动态基线、QueryState/extract cache、worker batching、counter export和局部零分配测试均需保留；问题在于它们没有通过同一HotPath/Metric Catalog、WorkloadScale、CostContract、DirtySet、Batch/Scratch/Cache、DataProduct lineage与QualificationReceipt覆盖后续owner迁移和新增系统。

报告登记0项P0、48项P1、12项P2及40个验收门，按`Cargo-resolved SourceSet -> HotPath/Metric Catalog -> WorkloadScale/CostContract -> Mutation/DirtySet -> Batch/Scratch/Cache/DataProduct -> Observation -> Complexity/Benchmark Evidence -> ProductQualification`收敛。Tooling07/24/25继续拥有benchmark evidence、concurrency与memory；Runtime05/08A/08D/08E/08F、Editor25与Plugins04继续拥有具体算法和metric handoff。详见`zircon_tooling/32-hot-path-catalog-algorithmic-complexity-data-movement-batching-cache-locality-performance-governance-review.md`。

本轮只新增review与索引，没有修改Rust、Cargo、tests、workflow、Runtime07计划或审计器。当前7条Runtime07 risk和既有领域hot path仍未修复；不得搬回父文件满足字符串、把关键词count变finding、用历史FPS冒充当前资格，或在缺FeatureParityReceipt时宣称优于Unreal。

## 144. Reference Engine Source Corpus / Snapshot / Provenance / Citation / Applicability / Comparison Currentness 范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| `dev/` root authority | 21个目录、2个普通文件；主仓`.gitignore`整体排除`/dev/`且tracked path为0 | 本机reference universe不能由Zircon commit恢复；应跟踪小型manifest/receipt，不提交76 GB源码 |
| five core checkout footprint | 328,251文件、76,229,954,431 bytes | 含Git object、LFS、generated docs、test asset和archive，不等于source规模或质量排名 |
| existing report citations | 写入前142篇报告、2,099次引用、1,596 unique locator、当前0 missing | 广泛离线引用是正向基础；路径存在/五家数量不证明claim正确 |
| citation granularity | 1,472文件、124目录；明确文件275,849,543 bytes | directory只能作SearchScope；229 MB Unreal API tgz需要archive member identity |
| reference file fingerprint | path + per-file digest聚合`d1ba462c...203ad` | 可冻结当前file locator set；不含目录递归内容，不能冒充完整CorpusId |
| snapshot fields | 142篇报告中structured corpus/snapshot/revision/commit/citation/claim字段均为0 | report无法机器绑定reference generation、symbol和差异主张 |
| validator coverage | `check_conventions.py`只检查implementation/related/tests | `plan_sources/reference_engines`删除、case、kind、digest和snapshot drift不进required gate |
| Unreal identity | 无`.git`；Build.version为6.0.0/UE5且CL/CompatibleCL均为0 | 需要non-Git tree manifest/build identity/access receipt；不能声称唯一current官方快照 |
| open-source Git state | Bevy/Fyrox/Godot完整clean；Graphics clean但shallow/grafted、rev-list count 1 | commit基础可保留；mirror/upstream、shallow/LFS/completeness与atomic acquisition仍缺policy |
| version semantics | Bevy/Fyrox/Godot nearest tag分别与0.20-dev/2.0-rc.1/4.8-dev source version不同 | commit、tag、package/source version、branch/build必须分字段 |
| license/access | Unreal EULA pointer、Bevy MIT/Apache、Fyrox/Godot MIT、Graphics Companion License | 本篇只建access/use metadata；许可/derived-source/notice由Tooling17与安全owner治理 |
| currentness | revision `ae2be3d...`；30个输入0 missing/0 duplicate；fingerprint `fb6c20d1...9e356` | worktree与reference roots可漂移；实施前重取Zircon SourceSet、snapshot、citation与access状态 |

本轮确认缺失的是reference review的可复现证据链，不是“报告没有看其他引擎”。现有routing skill、1,596个离线locator、四个clean Git checkout与五家组合对照均为可保留基础；问题在于没有`ReferenceCorpusManifest -> SnapshotReceipt -> ResolvedCitationSet -> ComparisonClaim -> Applicability/TranslationDecision -> ReviewReceipt -> Currentness`。

报告登记0项P0、48项P1、12项P2及40个验收门。Tooling17继续拥有Reference内容类别、license/notice和分发；Tooling28拥有Document/SourceGraph与publication currentness；Tooling07/32拥有benchmark、workload、FeatureParity与性能ComparisonReceipt；各Runtime/Editor/Plugin/App报告拥有领域实现。详见`zircon_tooling/33-reference-engine-source-corpus-snapshot-provenance-citation-applicability-comparison-currentness-review.md`。

本轮只新增review与索引，没有修改`dev/`、production、test、manifest、workflow、validator或既有reference path。三处`dev/fyrox` case错误、124个目录locator、non-Git Unreal identity、Graphics shallow/LFS与全部legacy claim edge仍未修复；不得直接pull覆盖快照、把整个`dev/`提交进主仓、按引用数量评分，或用source precedent宣称性能优于当前Unreal。

## 145. Global State Scope / Singleton / Static Registry / Initialization / Reset / Multi-instance Isolation 范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| production-like Rust candidate | 11,849文件、1,316,739物理行、45,742,170 bytes | 明确排除tests/benches/fixtures、典型测试文件与native dynamic fixture；仍不是cfg-resolved BuildSet |
| static declaration | 253处/171文件；Runtime116、Editor85、Plugins41、App6、Interface3、Hub2 | 混合immutable descriptor、ID sequence、metric、cache、registry和service，不能机械视为缺陷 |
| initialization primitives | OnceLock 94/81文件、LazyLock 3；static Mutex/RwLock/ArcSwap 38/2/1 | 一次初始化和同步安全不证明project/world/window作用域正确或可重启 |
| thread/process state | 39个thread-local宏/38文件、64个atomic static、production-like static mut为0 | 保留`static mut=0`；thread identity不能替代业务owner/generation |
| instance owner positive base | CoreRuntimeInner实例持有module/service/event/config/clock/state；module shutdown逆activation order | 作为EngineRuntime scope根保留并补RuntimeInstanceId、root terminal shutdown和receipt |
| lifecycle positive base | diagnostic log generation/session lease先unpublish再join；wake registry token由Drop注销 | 推广StateLease/quiescence/unload模式，不建立巨型global manager |
| process default seam | CoreRuntime默认克隆首个PROCESS_TASK_POOLS；PROCESS_TIMER永久OnceLock clone阻断last-owner Drop | 共享政策必须进入CoreRuntimeBuilder；timer需可关闭generation和DLL unload join |
| editor/project isolation | GUI持有实例plugin manager，commandlet另用process mutable shared manager；project snapshot无ProjectSessionId | builtin catalog可共享immutable，lifecycle/selection/registration必须归Host/CommandletSession |
| window/document isolation | UI Asset Editor全进程共用一个document/surface/size NodeProjectionSession | mutex只避免race；surface需降到Host/Window/Document scope并绑定theme/font/template generation |
| appearance/font scope | process design-token RwLock、paint-theme ArcSwap/DPI、shared mutable font database | generation通知变化但不标owner；Project font、Host theme、Window DPI必须分域 |
| retry/cache identity | `OnceLock<Option<EditorUiHostRuntime>>`永久缓存瞬态失败；template key仅path/mtime/len；path revision表永久按path | 增加retry generation、content/compiler/build/scope identity和owner retirement |
| currentness | revision `25e09a23...d404`；58个frontmatter路径0 missing/0 duplicate；43个source/test/reference输入808,312 bytes，fingerprint `bb80393a...eb3b9` | 邻近源码仍dirty且动态isolation未运行；实施前重取AST/BuildSet和真实多实例场景 |

本轮确认缺失的是产品级`GlobalStateInventory -> StateScopeDefinition -> Owner/Instance/Generation -> InitializationDAG -> StateLease -> Reset/Shutdown/Unload -> IsolationScenario -> StateScopeReceipt`，不是“所有OnceLock都错误”。不可变ABI/descriptor、content-addressed cache与明确process integration可以保留；可变状态必须证明最窄owner、世代、终止和跨实例隔离。

报告登记0项P0、48项P1、12项P2及40个验收门。Tooling24继续拥有lock/atomic/thread lifecycle，Tooling25拥有cache bytes/pressure，Tooling10拥有test isolation；Runtime05/11B、Editor12、Interface05和各plugin owner保留具体行为。详见`zircon_tooling/34-global-state-scope-singleton-service-locator-static-registry-cache-initialization-reset-multi-instance-isolation-review.md`。

本轮只新增review与索引，没有修改Rust、tests、Cargo、workflow或validator。现有process default、plugin双authority、projection/font/theme/cache scope仍未修复；不得把进程退出、mutex无race、generation计数或测试串行化当作多Project/World/Window及DLL unload资格。

## 146. Ownership Graph / Shared-Weak-Borrow-Lease / Callback-Subscription / RAII-Cycle-Detach-Leak Isolation 范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| production-like Rust physical candidate | 11,877文件、1,199,398物理行、45,771,850 bytes | 覆盖七个产品/代码根并排除常见test/bench/fixture；仍不是cfg-resolved BuildSet |
| shared/weak owner signal | 2,819处/823文件；212处weak signal/64文件 | Arc/Rc/Weak只作候选，不能用数量判定正确性或泄漏 |
| callback/subscription signal | 113个callback-owner/52文件；108个subscription/observer/55文件 | 宏展开与C ABI function pointer不在词法下界内 |
| terminal primitives | 141个Drop impl/123文件；19个detach/forget/ManuallyDrop/14文件 | Drop存在不证明quiescence；detach/ownership transfer需逐项路由 |
| positive event ownership | Core EventBus weak state + subscription Drop；asset stream weak mailbox + prune | 可作为普通订阅与死亡receiver清理的conformance基础 |
| positive dynamic ownership | native library-generation callback lease、WeakBridge、VM root lease、runtime mirror reclaim | operation强pin与长期weak observation可并存，无需机械改成全Weak |
| naked subscription seams | UiEventManager死亡sender；state hook只增不减；World/EventStore裸observer | 缺token、owner、generation、auto revoke与quiescent close |
| UI/Editor seams | pointer/navigation/router无remove；lifecycle bridge裸subscriber；listener旧snapshot写detached inbox | 需要node/document/plugin owner与in-flight/closing epoch合同 |
| plugin/network seams | HTTP route/sound executor可撤销；RPC validator/handler无owner/revoke；sound ABI callback无library lease字段 | 保留局部正例并统一PluginModuleId/LibraryGeneration |
| constructible strong cycle | ResourceManager拥有payload，ResourceLease release closure又强持完整manager | 类型合同允许manager-payload-lease-manager环；当前未证明shipping payload已触发 |
| currentness | revision `25e09a23...d404`；59个source/test/reference输入886,906 bytes，fingerprint `61c56764...b0adb` | 工作树邻近Editor源码dirty，动态heap/reload/leak场景未运行，实施前必须重检 |

本轮确认缺失的是`OwnershipEdgeInventory -> Owner/Owned/Borrowed/Observed -> Strong/Weak/LeasePolicy -> SubscriptionToken -> CallbackCapturePolicy -> CycleAnalysis -> Close/Drop/Unload -> LeakCensus -> OwnershipReceipt`，不是“Rust的Arc本身有问题”。EventBus、asset stream、native lease、WeakBridge、VM root与runtime mirror reclaim都应保留并标准化；裸ID、永久callback、无owner ABI指针和反向强持authority必须按具体生命周期硬切。

报告登记0项P0、48项P1、12项P2及40个验收门。Tooling21继续拥有unsafe/FFI soundness，Tooling24拥有线程/lock/channel/backpressure，Tooling25拥有resident bytes/pressure，Runtime24拥有identity/exhaustion；Runtime/Editor/Plugin/Interface原报告保留具体行为。本篇只拥有ownership edge、token/capture、cycle、terminal census与qualification。详见`zircon_tooling/35-ownership-graph-shared-weak-borrow-lease-callback-subscription-raii-cycle-detach-leak-isolation-review.md`。

本轮只新增review与索引，没有修改production、tests、Cargo、workflow或validator。现有死亡sender、state hook、observer、UI route、Editor subscriber、RPC callback与ResourceLease强环仍未修复；不得把Rust内存安全、手动unsubscribe、进程退出或strong_count当作clean unload/leak-free证明。

## 147. Type Erasure / Dynamic Dispatch / Any-Downcast / Trait Object / Reflection Type Identity / VTable Generation 范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| discovery inventory | Any 225行/85文件；TypeId 551/122；downcast 140/66；shared dyn 1,147/478 | 词法命中只作candidate，panic payload、enum variant、owner-local slot和测试不是自动finding |
| Core service | ServiceObject为`Arc<dyn Any>`；descriptor/handle无expected type contract；consumer最后downcast | index/generation/kind/admission可保留，但类型错误必须前移到registration/preflight并输出expected/actual |
| resource/ECS | resource按kind后downcast；Rust component按TypeId；dynamic component按String | ECS table column的TypeId/layout/callback是正例；durable/plugin identity需stable contract/schema/owner generation |
| erased World projection | MessageStore按TypeId存Any queue；Clone返回default、Eq恒true | active-channel优化可保留；clone/equality/serialize discard需显式WorldProjectionPolicy和receipt |
| reflection | full path字符串registry、短名歧义和catalog generation；derive使用`module_path!()` | 短名与VM owner校验可保留；缺stable TypeKey、schema version/hash、alias/migration与per-type retirement |
| plugin bridge | frozen InterfaceSlot、奇偶generation、WeakBridge；PluginInterface只有字符串ID | 需要InterfaceContract/method schema和typed mismatch；不能把wrong type压成NotEnabled |
| VM backend | family BTreeMap、register silent replace、resolve遍历且吞Err、contains调用resolve | 需要owner token/revoke、duplicate reject、NotMine/Failed分离与frozen selector map |
| render dispatch | compiled pipeline按registry generation缓存ID存在校验；execute每pass再查BTreeMap | 保留validation generation；compile必须绑定ExecutorSlot/call target/policy并失效旧plan |
| Editor composition | thread-local String cache，K generation downcast；metadata M未进key | same ID/K different M可复用错误model；需typed K/M/owner key和Absent/TypeMismatch分离 |
| RPC/provider | RPC descriptor/schema/validator/handler与graphics provider/collector按字符串注册 | payload reflection request是基础；silent overwrite、owner generation、schema contract与事务发布未闭合 |
| reference translation | Unreal UClass/Cast；Bevy TypeRegistry/ECS；Fyrox Type UUID；Godot ClassDB/Variant；Unity typed RenderGraph | 吸收双层identity、metadata+call target、typed error与compile-time binding，不照搬GC/global DB/C# object model |
| currentness | HEAD `25e09a23...d404`；60个source/test/reference输入1,071,391 bytes；fingerprint `34f97bde...60bf` | 邻近Runtime/Editor源码dirty，未运行reload/migration/microbenchmark，实施前必须重检 |

本轮确认缺失的是`ErasureInventory -> StableTypeIdentity -> TypeRegistry -> RegistrationValidation -> DowncastBoundary -> DispatchPlan -> Owner/Generation -> Unload/SchemaMigration -> DispatchCost -> TypeErasureQualificationReceipt`，不是“Any或trait object本身错误”。ECS erased column、asset importer、service admission、bridge slot与render validation cache都应保留并标准化；string-only contract、late downcast、silent replacement、错误折叠与每pass动态lookup必须按边界硬切。

报告登记0项P0、48项P1、12项P2及40个验收门。Tooling21继续拥有unsafe/FFI/DLL soundness，Tooling27拥有通用version/migration，Tooling32拥有hot-path cost，Tooling35拥有owner/lease，Runtime04/05/21和Editor09保留具体domain语义。本篇只拥有stable type contract、typed registration、downcast boundary、compiled dispatch与type-erasure qualification。详见`zircon_tooling/36-type-erasure-dynamic-dispatch-any-downcast-trait-object-reflection-type-identity-vtable-generation-performance-review.md`。

本轮只新增review、metadata与索引，没有修改production、tests、Cargo、workflow或validator。现有service/reflection/bridge/VM/render/RPC/Editor cache缺口仍未修复；不得把TypeId、字符串ID、downcast成功、registry generation或trait object可扩展性当作跨build身份、schema兼容、unload safety或性能资格。

## 148. Transaction Atomicity / Prepare-Commit-Publish / Rollback-Compensation / Idempotency / Crash Recovery 范围

| 子域 | 本轮证据 | 结论 |
|---|---:|---|
| discovery inventory | 13,438个production-like候选文件；transaction 900行/172文件；rollback 163/40；commit 1,737/437；publish 730/240；preflight 239/61；compensation 2/2；idempotency 55/15；atomic-write信号222/65 | 词法命中只作candidate；数据库transaction、Editor undo、GPU submit和deferred queue不能因同名自动获得同一原子性语义 |
| durable resource baseline | versioned intent journal、owner lock、staging/backup digest、per-file state、durable commit point、directory sync、rollback/recovery与crash/fault测试 | 是当前最完整正例；应抽象共享schema与receipt，不能退化为临时文件加rename |
| asset migration/project generation | 复用resource durable transaction；migration与生成器在preflight后提交 | 复用方向正确；仍需显式OperationId、input/output fingerprint与终态receipt供上层消费 |
| scene/ECS | dynamic scene与bundle先验证后内存发布；SceneDocumentRoute依次发布source/catalog/authoring world后再激活；PreparedSceneCreation使用hard-link/delete | 前两者可保留为preflighted infallible publication；route与prepared file链缺补偿、durable journal和restart recovery |
| Editor history/scene route | transaction engine在apply/revert失败时恢复；document route跨source/catalog/world/activation | undo/redo恢复是正例；跨authority scene publication仍可能留下partial state |
| archive/Hub文件与目录 | Session archive只`flush`、process-local revision、backup/temp rename；Hub create使用PID/counter ID、无journal/fsync/restart scan | 进程崩溃后可能出现unknown outcome或遗留backup/temp；需要stable OperationId、durable intent、recovery scan与terminal receipt |
| lifecycle/reload | Core module batch activation有preflight与逆序cleanup；UI reload先evict/invalidate/apply theme再做可失败surface dirty；runtime Running先于observer通知 | batch cleanup可保留；reload与notification需明确commit point、observer failure disposition和generation publication |
| domain compensation | native/VM hot reload有内存rollback；ragdoll忽略remove rollback结果；sound mixer只记录首个rollback error | 需要统一CompensationResult、residual effect集合、operator action与跨进程recovery，不得用首个字符串错误掩盖剩余状态 |
| retry/idempotency | RPC/HTTP retry缺operation key、dedup/effect/timeout contract；coordinator有request fingerprint、terminal replay和offline spool | coordinator是局部正例；所有会产生effect的retry必须区分never-applied/applied/unknown并由server-side dedup证明 |
| coordinator external effects | SQLite WAL与DB transaction可保护内部记录，但不能原子覆盖Git、文件、子进程和通知；accepted->interrupted不证明effect outcome | 需要outbox/effect ledger、reconciliation与operator-visible disposition，不能把DB commit当作跨系统commit |
| render/ECS qualification | GPU submit不可回滚；ECS deferred command batch可部分执行 | 分别标记commit-only与best-effort；如需原子批必须另建preflight/staging或显式补偿，不得重命名伪装 |
| reference translation | Unreal TransBuffer/ScopedTransaction/SavePackage2；Bevy CommandQueue；Fyrox editor command；Godot UndoRedo；Unity RenderGraph | 吸收transaction scope、command history、prepare/compile、durable save与explicit terminal语义，不照搬各引擎object model或假定render graph可rollback |
| currentness | HEAD `25e09a23...d404`；85个source/test/reference输入1,586,278 bytes；fingerprint `dd44319b...19e4` | 邻近Runtime/Editor/Hub/Coordinator源码dirty，既有动态验证阻塞已记录，实施前必须按BuildSet重检 |

本轮确认缺失的是`OperationInventory -> OperationId/Intent -> Preflight -> Staging -> CommitPoint -> Publication -> Compensation/Rollback -> Recovery/Reconciliation -> Idempotency/Dedup -> OperationReceipt -> TransactionQualification`，不是“所有mutation都必须可rollback”。Durable resource journal、scene/bundle preflight、Editor transaction恢复、module batch cleanup与Coordinator request journal应保留并标准化；partial、unknown、best-effort和commit-only必须成为显式Disposition，不能被成功布尔值或重试循环抹平。

报告登记0项P0、48项P1、12项P2及40个验收门。Runtime04/05/25继续拥有resource/scene/filesystem语义，Editor02/03/06拥有document/scene/plugin workflow，Tooling06/09/19/23/24/27拥有Coordinator/release/operation/failure/concurrency/version合同，各domain保留具体补偿行为。本篇只拥有跨域operation identity、commit point、disposition、recovery/dedup和transaction qualification。详见`zircon_tooling/37-transaction-atomicity-prepare-commit-publish-rollback-compensation-idempotency-crash-recovery-review.md`。

本轮只新增review、metadata与索引，没有修改production、tests、Cargo、workflow或validator。现有Scene route、archive、Hub create、reload、observer、RPC retry、外部effect与domain rollback缺口仍未修复；不得把rename、database transaction、undo stack、retry、deferred queue或GPU submit自动当作端到端原子提交。

## 149. Particle / VFX System / Emitter / CPU-GPU Simulation / Rendering / Scalability / Determinism 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| `zircon_plugins/particles`全量 | 57文件 / 8,733 / 306,329 | E3逐asset/manager/CPU/GPU/render/test调用链 |
| `rendering/features/vfx_graph` | 8文件 / 295 / 10,153 | 五节点graph、固定workload、两个no-op executor |
| runtime provider | 3文件 / 49 / 1,316 | GPU readback到manager feedback的窄桥 |
| core particle renderer | 21文件 / 1,193 / 43,626 | CPU billboard/velocity、每批buffer、color-only shader |
| Scene/script/product | 657行Scene extract + gameplay host + Vampire script | 产品走dynamic JSON sprite旁路，不消费particle plugin |
| 本轮冻结输入 | 140 / 43,229 / 2,010,138 | 113 Zircon/source/test/product + 27 reference；SHA-256 `6554789a...d31f` |
| 参考实现 | Unreal Niagara 8；Fyrox 4；Godot 4；Unity VFX Graph 6；Bevy render architecture 5 | Bevy当前无第一方particle runtime，不虚构功能对标 |

本轮把“类型/测试存在”“renderer可执行”“普通产品可达”和“可声明Complete”分开，确认当前粒子实现有真实底座，但没有形成唯一工程级runtime：

- particle package诚实标为experimental/Partial且默认不启用；SoA/free-list、局部确定seed、GPU ping-pong/alive/indirect、renderer prepare/readback和CPU previous-state/velocity应保留；
- 全仓生产搜索没有`ParticlesManager::tick` scheduler，`ParticleSystemComponent`也未进入Scene/ECS load/save/instantiate/remove；
- Vampire通过`gameplay.set_particle_sprites`写`render.particle_sprites` JSON，Scene直接生成colored quads，绕过asset、simulation、material与plugin lifecycle；
- GPU asset在manager内仍建立并推进CPU fallback，renderer-owned GPU owner又从同一manager克隆实例执行独立simulation，形成双backend authority；
- manager仍用单一mutex串行tick/rewind/snapshot；当前工作树已将snapshot payload改为共享`Arc<[T]>`并把diagnostic改成有界分页队列，但handle在`u64::MAX`饱和后复用并可覆盖live实例；
- `looped`没有consumer，CPU大dt spawn全部推进完整dt；无fixed substep、checkpoint/replay、world budget、LOD或scalability；
- physics collision只是damping且`bounce`未消费，animation binding字段未求值；缺provider或target时多处只warning/静默`Ok`；
- GPU把所有playing实例聚合为单一asset/backend，拓扑变化会重建全局资源；1,048,576 slot先到先得，双state约208 MiB且无budget；
- 多实例各自的dt会编码到emitter参数并由WGSL独立读取，旧“`max_dt`污染全部emitter”判断由Runtime103撤回；pause/stop/reset、aggregate重建和稳定GPU state合同仍未关闭；
- CPU renderer每sprite扩6 vertex并为depth/overlay/velocity逐批新建buffer；CPU/GPU shader都只输出颜色，不消费material/texture/UV/rotation/soft particle，GPU无velocity/history；
- 独立VFX Graph compiler只检查SpawnRate/material等存在性，固定声明`[1,1,1]` dispatch，simulation/transparent executor均直接`Ok(())`；
- GPU tests在adapter请求失败时`.ok()?`返回None，没有typed skip receipt；也没有普通product scene完成load/tick/render/reload/save/reopen与规模/故障资格。

对照Unreal world simulation/scalability/multi-renderer、Godot fixed FPS/one-shot/bounds/trail、Fyrox Scene/serialization/material和Unity compiled attribute/context program，本轮建立`ParticleSourceAsset -> ParticleSemanticCompiler -> CompiledParticleProgram -> ParticleWorldRuntime -> authoritative CPU或GPU executor -> immutable ParticleRenderPacket -> renderer/history -> qualification`。Bevy只提供MainWorld/RenderWorld extract、RenderAsset prepare/re-extract、GPU buffer/readback的架构证据。

报告登记0项P0、60项P1、12项P2、M0-M5及40项runtime资格门。Editor15继续拥有authoring document/graph/compiler UI/preview workflow；Runtime04/05/09/22-24拥有通用asset/world/GPU/material/history/time/identity合同；本篇只拥有Particle/VFX simulation、backend parity、renderer family、scalability与产品接入。详见`zircon_runtime/26-particle-vfx-system-emitter-cpu-gpu-simulation-rendering-scalability-determinism-product-integration-review.md`。

本轮没有修改production、tests、Cargo、manifest、workflow或lockfile，也没有重复已知plugin locked metadata失败lane。由于package仍为experimental/Partial且默认不启用，本篇不新增P0；任何profile提升为Complete/required/default enabled前必须先通过40项资格门。

## 150. Screen-Space Ambient Occlusion / GTAO / Denoise / Temporal / Depth-Normal Integration / Scalability 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| SSAO plugin与rendering umbrella | 10 / 1,058 / 37,131 | runtime descriptor、默认feature、Editor descriptor、capability与重复shader authority达到E3 |
| Runtime pipeline/resource/shader/test选择集 | 52 / 14,166 / 553,205 | Forward/Deferred input producer、current/history、DRS、compute、composition、SSR consumer与产品oracle达到E3 |
| 参考实现 | 27 / 11,105 / 450,990 | Fyrox 3、Bevy 5、Godot 7、Unity HDRP 8、Unreal 4 |
| 本轮冻结输入 | 89 / 26,329 / 1,041,326 | 62 Zircon + 27 reference；SHA-256 `16c546c2...c60108`；Zircon选择集0 dirty |

本轮从默认feature装配、pipeline compile、资源producer/binding、shader、history copy、lighting consumer一直追到产品测试，确认已有真实generic compute、shared HZB、AO current/history与Render Graph access底座，但当前产品路径存在三项P0：

- `ssao.wgsl`没有view-space position reconstruction、projection/world radius、thickness/falloff或far-depth guard，只比较8个相邻像素的raw device depth，并按world-space `normal.z`固定压暗；它是分辨率/朝向相关的edge darkener，不是几何AO；
- 默认Forward+没有Deferred Geometry或normal attachment writer，Mesh pass只写depth/scene color，却仍将retained `normal_view`作为`GBUFFER_NORMAL`绑定给SSAO，首次值无资格、pipeline切换后还可能stale；
- terminal `post_process.wgsl`把AO平方后乘到完整lit scene color，使direct diffuse/specular、emissive和sky/background一起变暗，SSR又把同一标量当specular occlusion。

进一步确认当前只支持full-resolution、固定8 taps与四个硬编码tuning值；没有edge-aware spatial denoise、motion reprojection、depth/normal rejection、disocclusion、neighborhood clamp、quality tier或project/camera/volume settings。AO params使用display `target.size`，但depth/normal/HZB/current AO按`target.render_size`分配；degrade ladder先切到0.85/0.7 scale才稍后关闭SSAO，因此错误坐标域是正常降级状态。AO history又按output size分配、只复制render-size左上区域，并继承全局history bool。

现有`ssao_quality_profile_darkens_scene_when_enabled`在默认Forward+首帧只断言平均luma下降超过5，同时检查pass/HZB/materialization已执行；它没有验证contact shape、open surface、orientation invariance、emissive/direct preservation、motion/history或DRS，因而会奖励上述错误normal输入和全局压暗。报告将这条真实产品lane保留，但要求改造为正反例组合oracle。

对照Fyrox的view-space reconstruction/normal transform/32样本/noise/half-resolution、Bevy的VBAO/depth preprocess/horizon integration/edge-aware denoise、Godot的adaptive gather/quality/blur/interleave、Unity HDRP的spatial-temporal-denoise/reprojection/rejection/upsample与Unreal的method/pass/quality/async资格，本轮建立`CompiledAoProfile -> qualified depth/normal/motion/render rect -> GTAO/VBAO evaluate -> spatial denoise -> temporal reproject/reject/clamp -> bilateral upsample -> indirect diffuse/specular-occlusion consumers -> per-view history/evidence`。

报告登记3项P0、48项P1、12项P2、M0-M5及40项runtime资格门。Runtime09A/09B/09C/09H1/09H2/23继续拥有通用RHI、HZB、shader/PSO、history/velocity、terminal/SSR与projection合同；Plugins04拥有package/capability truth，Editor22拥有通用authoring framework。本篇只拥有AO算法、输入资格、AO专属denoise/history、lighting composition、quality/scalability与产品oracle。详见`zircon_runtime/27-screen-space-ambient-occlusion-gtao-denoise-temporal-depth-normal-scalability-product-integration-review.md`。

本轮只新增review、metadata与索引，没有修改production、tests、Cargo、manifest、workflow或lockfile。三个P0关闭前，当前SSAO不得默认启用或以“画面更暗/pass执行”宣称Complete。

## 151. Hardware Ray Tracing / BLAS-TLAS / Ray Query-Pipeline / SBT / Denoising / Scalability 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Zircon production选择集 | 53 / 8,418 / 277,321 | RHI、capability/profile、builtin/plugin policy、Solari、Hybrid GI consumer与Sound命名边界达到E3 |
| Zircon test/product选择集 | 8 / 1,763 / 63,174 | RHI capability、pipeline slot、provider状态、Solari product与Hybrid GI route tests达到E3 |
| 参考实现 | 55 / 38,138 / 1,516,185 | Bevy Solari 19、Unreal 10、Unity Graphics 18、Godot 5、Fyrox 3 |
| 本轮冻结输入 | 116 / 48,319 / 1,856,680 | 61 Zircon + 55 reference；SHA-256 `bb811f12...3f67a4`；所选Zircon语义diff为0 |

本轮从backend negotiation、framework capability、feature/profile compile、RHI resource/pipeline/command一直追到scene/mesh/material、BLAS/TLAS、query/pipeline/SBT、Solari/Hybrid GI/Sound consumer、denoise/history、Editor与产品测试，确认当前没有硬件光追实现：

- `AccelerationStructureCaps`只有supported、inline、pipeline、max-instance四字段，framework projection还丢失max-instance；WGPU无条件disabled，仓库没有第二个可执行图形backend；
- `RenderDevice`没有BLAS/TLAS handle、descriptor、create/destroy/size query，CommandList没有build/update/compact/dispatch rays，PipelineKind与ShaderStage也没有ray pipeline/stages，bind resource没有AS类型；
- builtin `ray_tracing`与plugin `ray_tracing_policy`形成两套零workload/零pass authority；后者又同时要求Inline Query与Ray Pipeline，不能表达query-only或pipeline-only设备；
- Solari provider诚实返回Unavailable，产品测试只验证NotRequested/Missing/Unavailable，并通过`override_capabilities_for_tests`手工注入没有RHI实现的AS/Inline bool；
- Hybrid GI把HardwareRayTracing编码为`1 << 3`，实际WGSL只有surface cache、Global SDF与voxel binding；Sound的RayTraced只接收外部impulse-response samples，二者均不能证明graphics hardware rays executed。

参考侧，Bevy `bevy_solari`已有mesh `BLAS_INPUT`、BLAS增量build/按帧compaction、TLAS/material/light/previous-transform binding、Realtime ReSTIR/world cache与验证path tracer；Unreal把BLAS视为有build priority、LOD/streaming/residency/dynamic update的长期资源，并有完整RHI scene/pipeline/SBT；Unity UnifiedRayTracing以同一接口支持Hardware与Compute software BVH，再由HDRP管理RTAS、light cluster、temporal/denoiser/fallback。Godot/Fyrox没有实时硬件RT时保持诚实缺席，只提供真实compute/CPU tracing与offline lightmap路径。

报告登记0项P0、56项P1、14项P2与42项资格门。0 P0源于当前Experimental/Partial/Unavailable与WGPU fail-close，不表示功能接近完成；任何UI/profile/receipt把零pass、bitmask、test bool或外部声学samples宣称为Hardware RT Ready/Executed时立即升级P0。目标链为`RtCapabilityProfile -> RtSceneCompiler -> BLAS/TLAS stores -> Ray Query/Pipeline+SBT -> effect gateway -> denoiser/history -> scalability/evidence`。详见`zircon_runtime/28-hardware-ray-tracing-blas-tlas-ray-query-pipeline-sbt-denoising-scalability-product-integration-review.md`。

本轮只新增review、metadata与索引，没有修改production、tests、Cargo、manifest、workflow或lockfile，也没有运行无法增加证据强度的普通raster产品测试。

## 152. Terrain / Landscape / Heightfield / Quadtree LOD / Material Layer / Foliage / World Partition / Physics / Navigation / Scalability 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Zircon production选择集 | 43 / 9,529 / 351,833 | terrain asset/plugin、scene/world、render feature/extract、physics、navigation与product wiring达到E3 |
| Zircon test/product选择集 | 10 / 4,938 / 170,484 | terrain/plugin/asset/scene、Vampire与WOC settings证据达到E3 |
| 参考实现 | 60 / 55,181 / 2,190,598 | Unreal Landscape/Foliage/World Partition/VHM、Fyrox terrain、Bevy generic render、Godot height map/MultiMesh/Jolt与Unity Graphics TerrainLit/GPUDriven/TerrainToMesh |
| 本轮冻结输入 | 113 / 69,648 / 2,712,915 | 53 Zircon + 60 reference；SHA-256 `83ddf41b...9c5f21`；所选Zircon语义diff为0 |

本轮从terrain source/asset/importer、scene document与World roundtrip，一直追到render extract/feature、heightfield physics、navigation geometry、foliage、world partition、scalability和Vampire产品证据，确认当前没有可执行的Terrain runtime：

- `TerrainAsset`把完整height samples内联为`Vec<Real>`，只有弱dimension校验；没有source/artifact/live instance分层、稳定chunk/patch/layer identity、版本化grid/layer schema或target build receipt；
- scene文档虽然有`SceneTerrainAsset`，`SceneNode`/`NodeRecord`却没有terrain component，`World::from_scene_asset`不消费该引用，`World::to_scene_asset`还固定写`terrain: None`；
- `BuiltinRenderFeature::Terrain`只属于descriptor-only advanced slot，拥有extract section名称但没有phase/pass，geometry extract也只有mesh/light；没有height/normal/weight/hole GPU artifact、patch topology、quadtree/clipmap/CDLOD、crack control、visibility、material layer或pass资格矩阵；
- Jolt backend把height field展开为`2*(w-1)*(h-1)`个三角形，builtin physics对height field的AABB/raycast/contact返回无结果；navigation又把render source简化为固定quad，并跳过asset-backed collider geometry；
- production没有foliage prototype/scatter/cluster、World Partition manifest/grid/cell、HLOD或多预算streaming owner。Vampire把“Baked Jungle Terrain”同时绑定普通mesh和terrain引用，现有测试只证明TOML可解析且普通mesh能出像素；WOC的`foliageDensity`也没有引擎consumer。

参考侧，Unreal把Landscape、Foliage、Virtual Heightfield Mesh与World Partition分成长期资源和streaming owner；Fyrox terrain已有chunk/quadtree/CDLOD、layers与query/raycast；Godot以真实HeightMapShape3D/Jolt native height field、MultiMesh和navigation integration提供可执行基础。Bevy没有内建terrain，因此本轮只采用其RenderAsset/GPU preprocessing/meshlet架构，不把generic机制冒充terrain；Unity Graphics仓只作为TerrainLit、GPU resident drawing和TerrainToMesh/path tracing参考，不宣称其中包含Unity核心Terrain authoring。

报告登记0项P0、62项P1、14项P2与44项资格门。0 P0源于Editor16已拥有backend无consumer、Workbench伪结果和partition authority缺失等产品阻断，不表示功能接近完成；本篇建立`TerrainSourceAsset -> TerrainBuildArtifact -> TerrainRuntimeInstance -> Render/Physics/Nav/Foliage/Partition adapters -> typed receipt`，并将算法、代际、预算和产品oracle归于Runtime29。详见`zircon_runtime/29-terrain-landscape-heightfield-quadtree-lod-material-layer-foliage-world-partition-physics-navigation-scalability-product-integration-review.md`。

本轮只新增review、metadata与索引，没有修改production、tests、Cargo、manifest、workflow或lockfile。真实Terrain provider、scene roundtrip和至少一个render/physics/nav consumer关闭前，不得把asset引用、descriptor、普通mesh fallback或静态batch称为Terrain Ready/Executed。

## 153. Water / Ocean-Lake-River Surface / Wave-FFT / Shallow Water / Underwater / Buoyancy / Query / Scalability 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Zircon production | 46 / 10,300 / 382,137 | E3：resource/scene、frame extract、PBR transmission、SSR/Planar/OIT、Physics force、Navigation与first-party catalog |
| Zircon focused tests | 14 / 4,193 / 145,312 | E3静态阅读：通用render/physics/nav tests、WOC water elemental材质与虚构`ocean.query.v1`bridge fixture |
| Unreal/Unity HDRP/Godot/Bevy/Fyrox参考 | 84 / 40,586 / 1,660,441 | E2/E3：Water Body/query/quadtree/wave/buoyancy、HDRP FFT/CPU search/underwater及通用refraction/SSR边界 |
| 冻结合计 | 144 / 55,079 / 2,187,890 | SHA-256 `c8478ec84d2888516a64863c31699108b1f5e1c6c146e483150105db7ac430d8` |

production精确词边界搜索确认没有Water/Ocean/River/Buoyancy/Underwater/Caustic/Gerstner领域owner；`ResourceKind`、Scene asset/node、`BuiltinRenderFeature`、first-party runtime catalog和rendering feature catalog均无Water。现有Standard PBR支持transmission/thickness/IOR/attenuation，但默认IOR为1.5；screen-space transmission只按normal.xy、`(ior-1)`、thickness和固定0.02偏移LOD0 scene color。SSR、Planar Reflection、OIT、ApplyForce/Impulse和Recast/Detour均为真实通用底座，不是Water Body、wave/query或buoyancy闭环。

测试中的`ocean.query.v1`只用于plugin bridge slot/generation/unload，WOC `water_elemental.glb`只验证普通glTF材质能导入IOR 1.333与volume transmission参数；两者均无production provider。Physics没有Water volume/submersion/buoyancy/current/drag，Navigation没有swim/water locomotion layer。Editor39继续拥有River/Spline source/compiler与flow artifact，Editor38拥有WindField/Weather generation，Runtime30从accepted artifact/generation边界消费。

参考侧，Unreal Water有Water Body/Zone、CPU query、Water Mesh/Quadtree、Gerstner、Buoyancy、baked shallow water、HLOD与terrain/nav integration；Unity HDRP分离FFT/simulation、CPU search、current/deformation/foam、水线、underwater与tile/tessellation。Godot只有通用refraction/SSR，Bevy只有ExtendedMaterial water shader示例，Fyrox本轮只找到通用Fresnel；后三者未被误记为引擎级Water系统。

报告登记0项P0、62项P1、14项P2与44项资格门。0 P0源于Editor39、Runtime09G2和Runtime08A已拥有相关产品真实性、transmission/compositor和buoyancy/field边界，不表示Water接近完成；本篇建立`WaterBodySource -> WaterBuildArtifact -> WaterRuntimeInstance -> Surface/Wave/Query/Physics/Nav/Audio/VFX adapters -> typed receipt`。详见`zircon_runtime/30-water-ocean-lake-river-surface-wave-fft-shallow-water-rendering-underwater-buoyancy-query-scalability-product-integration-review.md`。

本轮只新增review、metadata与索引，没有修改production、tests、Cargo、manifest、workflow或lockfile。真实Water provider、Scene roundtrip、surface pixels、CPU query、underwater与buoyancy consumer关闭前，不得把材质参数、静态蓝色mesh、SSR/Planar/OIT或ApplyForce称为Water Ready/Executed。

## 154. Cloth / Fabric / Soft Body / Garment / Simulation / Collision / Deformation / Rendering / Wind / LOD / Scalability 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Zircon production | 70 / 10,207 / 393,979 | E3：resource/scene、Mesh/Skin/Morph、Animation pose、Physics/Jolt bridge、frame extract、GPU mesh/velocity、PBR material与feature catalog |
| Zircon focused tests | 9 / 3,290 / 121,743 | E3静态阅读：Mesh/Morph、render mesh cache、Jolt rigid contract、Ragdoll/profile tests；45个test/ignore属性均未覆盖Cloth产品链 |
| Unreal/Godot/Jolt/Bevy/Unity HDRP参考 | 42 / 11,166 / 554,016 | E2/E3：Chaos Cloth资产/runtime映射与LOD、Godot/Jolt soft body、Bevy skin/Morph current-previous buffer、HDRP Fabric光学模型 |
| 主冻结语料 | 121 / 24,663 / 1,069,738 | SHA-256 `ce2fbebacb7814a4b3dac120cad7c7bc5ba965f167ae574304244a42b6337c94` |
| 补充GPU mesh upload | 2 / 107 / 3,860 | SHA-256 `d8f902d5881d26539339a10e8de5ad955bdb5cf5978ec957e2dc5adb33cf12e7`；主/补充合计123个显式输入 |

production精确标识搜索确认没有Cloth/SoftBody/Garment/Fabric领域类型；`ResourceKind`、`SceneEntityAsset`、Scene component、`RenderFrameExtract`、`BuiltinRenderFeature`、Physics capability和Jolt bridge都没有布料资源、软体实例、solver/provider、模拟输出或渲染feature。Physics collider与native bridge仅覆盖刚体shape/body/constraint/query/event，固定步默认60 Hz、全局最多4个substep，也没有Cloth独立iteration、collision或预算合同。

Mesh四权重skinning、Morph position/normal/tangent/color、current/previous palette与velocity是可保留基础，Ragdoll也能在bone transform与rigid body之间回写；但它们没有sim/render topology、seam/fabric/weight map、render-to-sim mapping或每LOD状态。GPU mesh upload只创建`VERTEX`/`INDEX` buffer，没有storage/copy-dst、fence或generation-qualified动态形变发布；Standard PBR虽有anisotropy/transmission/SSS，也没有Fabric sheen/fuzz/Charlie等明确光学模型。

参考侧，Unreal明确分离2D/3D simulation topology、render topology、seam/fabric/maps、barycentric render-to-sim mapping、per-LOD proxy/task、teleport/collision/cache；Godot/Jolt的最小soft-body链仍有typed object/backend/vertex/render update。Bevy仅提供skin/Morph current-previous buffer基础，Unity HDRP Fabric只提供光学材质，二者均未被误记为Cloth solver；Fyrox本轮没有找到可归属的Cloth/SoftBody production模块，作为负面参考记录。

报告登记0项P0、64项P1、14项P2与36项资格门。0 P0源于Runtime08A已拥有“softbody/cloth无owner”占位结论、Runtime09H1已拥有previous deformation边界，且当前catalog没有宣称Cloth Ready，不表示功能接近完成；本篇建立`ClothSourceAsset -> ClothBuildArtifact(sim/render/mapping/LOD) -> ClothRuntimeInstance -> Solver Provider -> qualified DeformationOutput -> Render/Physics/Animation/Wind adapters -> typed receipt`。详见`zircon_runtime/31-cloth-fabric-soft-body-garment-simulation-collision-deformation-rendering-wind-lod-scalability-product-integration-review.md`。

本轮只新增review、metadata与索引，没有修改production、tests、Cargo、manifest、workflow或lockfile。typed Cloth资源、source compiler、solver、collision、dynamic deformation output、Fabric pixels与产品场景证据关闭前，不得把skinned mesh、Morph、Ragdoll、double-sided material或wind vertex offset称为Cloth Ready/Executed。

## 155. Hair / Groom / Fur / Strand Source / Binding / Simulation / Rendering / Lighting / Shadow / LOD / Streaming / Scalability 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Zircon production | 63 / 12,296 / 446,067 | E3：resource/scene、Mesh/Skin/Morph、importer、frame extract、GPU mesh、OIT、shadow/velocity、PBR/shading model、visibility与streamer |
| Zircon focused tests | 13 / 3,934 / 136,621 | E3静态阅读：glTF triangle/channel、Mesh Morph、generic transparency、skinned velocity、ordinary shadow与standard shading；51个test/ignore属性均未覆盖Hair产品链 |
| Unreal HairStrands / Unity HDRP Hair参考 | 72 / 51,958 / 2,153,090 | E2/E3：source/group/attribute、binding/interpolation、guide sim/cache、strands/cards/meshes、cluster/LOD/streaming、visibility/deep shadow/transmittance、Hair BSDF与RT |
| 冻结合计 | 148 / 68,188 / 2,735,778 | SHA-256 `8e13c82d4e455f2ebf6f158f6ce3707e5f63c82f94b9d166a35bf9348b016032` |

production精确标识搜索确认`Hair/Groom/Fur/Strand/Alembic/Fiber/Follicle/Melanin/Marschner/Kajiya/WindField`均为零命中；`ResourceKind`、`SceneEntityAsset`、`RenderFrameExtract`、`BuiltinRenderFeature`和importer capability没有Hair资源、binding、instance、deformer/cache、render feature或material model。glTF ingest只接受Triangles，optional FBX/DAE/3DS/USD model backend为DiagnosticOnly，也没有Alembic `.abc`或curve Groom payload。

现有四权重skin、Morph、current/previous palette、static GPU mesh、generic OIT、ordinary shadow/velocity和anisotropic GGX是真实通用基础，但不能表达group/strand/point schema、root binding、guide interpolation、representation resource或Hair光学。OIT默认平均每像素4层、精确排序最多8层，shader固定最多32层并在capacity溢出时直接丢fragment；8-bit packed color与tail merge不能替代coverage/node visibility、deep opacity/transmittance或strand composition。

参考侧，Unreal分离HairDescription、GroomAsset/Binding/Cache、rest/deformed/current/previous resources、guide simulation、cluster/LOD/streaming，以及visibility、composition、deep shadow、environment、voxel、velocity、cards与ray tracing；Unity HDRP Hair提供melanin/absorption、cuticle、longitudinal/radial roughness、R/TT/TRT和dual/multiple scattering光学参考。当前Bevy/Godot/Fyrox目标production路径的文件名精确搜索为零，作为本地镜像范围内的负证据，不被外推或当作降级许可。

报告登记0项P0、72项P1、16项P2与40项资格门。0 P0源于当前catalog没有Hair Ready声明，且Runtime09F3已拥有GI hair classification边界，不表示功能接近完成；本篇建立`HairSourceAsset -> HairBuildArtifact(groups/guides/strands/cards/meshes/binding/LOD) -> HairRuntimeInstance -> Deformation/Cache Provider -> representation resources -> Visibility/Lighting/Shadow/RT adapters -> typed receipt`。详见`zircon_runtime/32-hair-groom-fur-strand-source-binding-simulation-rendering-lighting-shadow-lod-streaming-scalability-product-integration-review.md`。

本轮只新增review、metadata与索引，没有修改production、tests、Cargo、manifest、workflow或lockfile。typed Hair source/binding、compiler、deformer/simulation/cache、strand visibility、Hair BSDF、deep shadow、representation LOD和真实产品证据关闭前，不得把alpha mesh、anisotropy、Morph、particle trail、shell/fins、cards-only demo或generic OIT称为Hair Ready/Executed。

## 156. Destruction / Fracture / Geometry Collection / Clustering / Damage Field / Simulation / Rendering / Cache / Scalability 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Zircon production | 136 / 15,479 / 541,011 | E3：resource/mesh/scene、physics framework与Jolt/builtin backend、constraint/manager、frame extract、GPU mesh、visibility与WOC可破坏墙规则 |
| Zircon focused tests | 17 / 4,285 / 150,553 | E3静态阅读：普通mesh、body/collider/joint、query/contact/trigger、frame extract与generic mesh/visibility；84个test/ignore属性均未覆盖破坏产品链 |
| Unreal GeometryCollection/Chaos/Fracture/Field/Cache参考 | 96 / 46,511 / 1,794,924 | E2/E3：typed collection、hierarchy/connection/collision facade、fracture/cook、cluster proxy、damage/strain/field、piece rendering、event与cache |
| 冻结合计 | 249 / 66,275 / 2,486,488 | SHA-256 `230c11c6324a7d2c294b415a39d92a8fc14245e488e915ccc65c4795e8a65392` |

production精确标识搜索确认`GeometryCollection/Voronoi/DamageThreshold/Destructible/Fracture/Shatter/BreakingEvent/ExternalClusterStrain/InternalClusterStrain`均为零命中；`ResourceKind`、`SceneEntityAsset`、`PhysicsWorldSyncState`、`PhysicsBackend`、`RenderFrameExtract`与`BuiltinRenderFeature`没有破坏资源、piece/cluster identity、damage/field、breaking event、逐片transform或render feature。

当前Scene每实体只有一份mesh/body/collider/joint；physics backend只创建单body/shape/constraint并按entity写回，contact event又只有两entity、point和normal。GPU mesh只有ordinary vertex/index buffer，mesh snapshot/draw以单entity transform为单位，没有piece bone map、cluster hierarchy、active mask、dynamic bounds或current/previous piece palette。WOC `destructible_wall`仅按health开关3.2坐标碰撞半径，技能/职业里的destruction/fracture/shatter字符串同样不构成引擎实现。

参考侧，Unreal以Managed GeometryCollection保存vertices/faces/geometry/transform/material与依赖重映射，FractureEngine编译Voronoi/plane/slice/brick/mesh cutter、interior surface、hierarchy/connection和collision，Chaos proxy消费damage/field/strain并原子产生cluster break；render data/scene proxy维护piece map和current/previous transform buffer，ChaosCaching记录逐particle transform与breaking/collision/trailing事件。Bevy/Fyrox/Godot当前本地镜像没有对应production模块，Unity Graphics命中的Voronoi只是Shader Graph/VFX噪声，均不作降级许可。

报告登记0项P0、72项P1、16项P2与40项资格门。0 P0源于Runtime08A已拥有“destruction无owner”的占位结论且catalog未宣称Ready，不表示功能接近完成；本篇建立`DestructionSourceAsset -> FractureBuildArtifact(pieces/interiors/collision/hierarchy/graph/LOD) -> DestructionRuntimeInstance -> ClusteredRigid Provider -> Damage/Field ingress -> Break/Removal/Piece output -> Render/Nav/Audio/VFX/Network/Cache adapters -> typed receipt`。详见`zircon_runtime/33-destruction-fracture-geometry-collection-clustering-damage-field-simulation-rendering-cache-scalability-product-integration-review.md`。

本轮只新增review、metadata与索引，没有修改production、tests、Cargo、manifest、workflow或lockfile。typed source/artifact、stable piece identity、deterministic fracture compiler、clustered physics、damage/field、piece rendering/cache和真实产品证据关闭前，不得把换模型、隐藏墙、普通joint或随机spawn刚体称为Destruction Ready/Executed。

## 157. Vegetation / Tree / Foliage / Grass / Species / Instancing / Wind / Billboard / Impostor / LOD / Streaming / Scalability 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Zircon production | 212 / 28,667 / 1,027,774 | E3：mesh/model/material/import、Scene LOD、frame extract/static batch、feature slots、GPU Scene/visibility、shader templates、Terrain runtime |
| Zircon focused tests | 15 / 4,354 / 157,165 | E3静态阅读：ordinary asset/extract/GPU Scene/draw/shader/visibility/terrain/Vampire contract；无species、cluster、wind、impostor或leaf-shading测试 |
| Vampire产品证据 | 11 / 3,354 / 110,253 | E3：grass premerged model、broadleaf/fern model、opaque double-sided material、forest shader/README与相关scene/asset evidence |
| Unreal/Unity/Godot/Bevy/Fyrox参考 | 104 / 44,476 / 1,722,355 | E2/E3：FoliageType/HISM/InstanceCulling/SpeedTree wind、GPUDriven wind/SpeedTree/grass、MultiMesh、generic batching/meshlet与renderer负证据 |
| 冻结合计 | 342 / 80,851 / 3,017,547 | SHA-256 `f568baa7a537597749d612388e2f591d13905a5e796787cd95a49f4a8b66ac3f` |

production精确标识搜索确认`VegetationSpecies/VegetationAsset/FoliageAsset/FoliagePrototype/FoliageRuntimeCluster/GrassAsset/SpeedTree/Impostor/WindDirectionalSource/WindField/FoliageShading`均为零命中。`TreeAsset`的11个候选未形成species资源；`Tree/Billboard/Terrain/MeshLod`只是advanced feature descriptor/extract槽，没有pass/executor或产品提交链。

当前`GeometryExtract::static_batches`没有graphics consumer；mesh产品路径对每个pending draw固定`gpu_scene.register(..., 1)`并只写一个176-byte实例，LOD只按entity原点距离选择`min_distance`。shader模板没有vertex deformation/WPO hook，wind只存在于测试字符串。Vampire grass是预合并ordinary mesh，材质为opaque double-sided，forest shader只有fragment细节，README与测试只证明DTO/资产而不证明draw reduction、wind、billboard或impostor。

参考侧，Unreal Foliage分离species policy与instance storage，HISM/InstanceCulling维护cluster tree、mutation、visible compaction与indirect work，SpeedTree wind覆盖global/branch/leaf/frond/gust/LOD/time；Unity Graphics另有GPUDriven wind current/history、SpeedTree7/8与WavingGrass pass。Godot MultiMesh和Bevy batching/meshlet仅作通用批实例底座，Fyrox本地树没有专用vegetation模块，不作降级许可。

报告登记0项P0、72项P1、16项P2与40项资格门。Terrain29、Editor16、Runtime09B/09C/09D/28及App06继续拥有placement/authoring/generic GPU Scene/material/residency/RT/产品证据；本篇建立`VegetationSpeciesSource -> VegetationBuildArtifact -> Prototype -> Cell/Cluster/Instance Set -> Wind/Interaction State -> GPU LOD/Streaming -> Raster/Shadow/Velocity/GI/RT adapters -> typed receipt`。详见`zircon_runtime/34-vegetation-tree-foliage-grass-species-instancing-wind-animation-billboard-impostor-lod-streaming-scalability-product-integration-review.md`。

本轮只新增review、metadata与索引，没有修改production、tests、Cargo、manifest、workflow、lockfile或Vampire资产。species/compiler、instance-set/cluster、representation LOD、wind history、thin-leaf shading、cross-system adapter和真实产品证据关闭前，不得把Tree枚举、static-batch DTO、预合并草mesh或double-sided PBR称为Vegetation Ready/Executed。

## 158. Decal Projector / Material Domain / DBuffer-GBuffer-Forward / Receiver / Culling-Batching / Atlas-Streaming / Temporal-RT / Scalability 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Zircon production | 193 / 29,236 / 1,085,634 | E3：Decal runtime/editor package、plugin manifest、component schema、Scene/DynamicScene、extract、feature/pipeline、material/shader、deferred/visibility/execution context |
| Zircon focused tests | 24 / 8,109 / 314,912 | E3静态阅读：173个test/ignore属性覆盖generic extract/pipeline/deferred/context/shader/DynamicScene；Decal仅另有1个registration test |
| 产品/控制面证据 | 4 / 1,430 / 61,866 | E3：Material Workbench false domain、App source assertion、plugin README与template binding；examples/templates无独立Decal单词命中 |
| Unreal/Unity/Godot/Bevy/Fyrox参考 | 65 / 25,760 / 1,030,358 | E2/E3：component/proxy/stage/DBuffer/mesh/mobile/RT，HDRP/URP projector-system-chunk-atlas-technique，RID/node与forward/clustered paths |
| 冻结合计 | 286 / 64,535 / 2,492,770 | SHA-256 `2c472f87c0cc2648b6952eac4790500d1ab8dc9254127dba39d84a17c0bb7b35` |

当前`rendering.decals`插件定义Deferred/ScreenSpace mode和四字段projector descriptor，注册PostProcess pass后由`noop_render_executor`直接返回`Ok(())`。projector struct/package外无consumer，Scene静态schema无plugin payload，`RenderFrameExtract`无Decal snapshot，GPU context无prepared Decal work；`MaterialDomain`没有Decal，atlas_region只是无人消费String。

pass只读depth/color并原位写color，normal/ORM/emissive没有attachment；两个mode无分支，opacity/normal_blend/atlas_region均未读取。pipeline把Decals插在baked lighting/reflection probes/bloom之后，不具备pre-lighting DBuffer/GBuffer语义。测试只验证registration/descriptor/quality-gate枚举，不执行GPU命令或像素oracle；examples/templates没有真实caller。

参考侧，Unreal把material blend descriptor编译为DBuffer/GBuffer/mobile/emissive/AO stage、target/write mask/blend/raster状态，并由generation-qualified proxy和per-view visibility list提交；Unity HDRP/URP有projector、instance manager/chunk、cull/batch/atlas以及DBuffer/screen-space/GBuffer technique。Godot/Fyrox较小实现也有持久Scene node、资源、bounds和真实shader；Bevy clustered/forward Decal贯通extract/prepare/bind/shader并显式报告bindless平台限制。

报告登记0个新P0、72个runtime子P1、16个P2与40项资格门。Editor39保留5项P0和P1-50..60父要求，Plugins04保留package/capability装配真相；Runtime35只拥有`DecalMaterialArtifact + ProjectorInstance -> RenderSnapshot -> per-view stage work -> DBuffer/GBuffer/Forward/RT adapters -> receipts`的可执行分解。详见`zircon_runtime/35-decal-projector-material-domain-dbuffer-gbuffer-forward-receiver-culling-batching-atlas-streaming-temporal-rt-scalability-product-integration-review.md`。

本轮只新增review、metadata与索引，没有修改production、tests、Cargo、manifest、workflow或产品资产。shared domain/schema、Scene instance、extract、projection/cull/batch、真实executor、DBuffer/forward/temporal/RT和产品证据关闭前，不得把Decal枚举、pass token、Material下拉或成功registration称为Decal Ready/Executed。

## 159. Weather / Climate / Celestial / Time-of-Day / Wind / Precipitation / Cloud / Atmosphere / Surface State / Determinism / Network / Save / Scalability 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Zircon production | 135 / 28,313 / 992,696 | E3：time、Scene asset/component、environment/frame extract、IBL、fog、particle、terrain、sound及首方catalog纵向链 |
| Zircon focused tests | 43 / 11,559 / 422,964 | E3静态阅读：230个test/tokio::test属性和1个ignore；Weather/Climate命中均为generic loader/registry/interface fixture |
| 产品/控制面证据 | 12 / 8,653 / 269,676 | E3：静态Weather Workbench/bindings、SDK示例窗口、WOC天气画质开关与Rain/Lightning技能合同 |
| Unreal/Unity/Godot/Bevy/Fyrox参考 | 38 / 22,260 / 1,042,502 | E2/E3：Atmosphere/Cloud/Wind组件与DaySequence、HDRP Volume/Sky/Cloud/history、Environment资源、ECS extract和SkyBox持久化 |
| 冻结合计 | 228 / 70,785 / 2,727,838 | SHA-256 `cdf27abd370ed3f7dca546f1464f89a22de56eb8b8e282679d1cb549b0800e3f` |

production没有Weather/Climate package、source asset、compiler、artifact、World service、transition graph、region resolver、snapshot、query ABI或first-party catalog项。Scene asset/component没有Environment/Atmosphere/Cloud/Weather字段；`build_environment_extract`只把viewport `preview_skybox`布尔值映射为disabled/procedural default，EnvironmentExtract也只有skybox/probes/baked lighting/probe grid。

通用基础不能替代产品闭环：Runtime time只有real/virtual/fixed duration，working diff新增virtual_delta也没有calendar/celestial语义；程序天空sun与DirectionalLight分裂。Particle CPU只累加asset gravity和external_force，GPU只上传gravity；Terrain/Sound/Material对weather/wetness/puddle/snow/humidity/precipitation/temperature/wind独立词均零命中。

参考侧，Unreal把Atmosphere/Cloud/Wind建成持久组件并有scene proxy，DaySequence提供day length、cycle、preview、modifier volume和replicated playback；Unity HDRP以typed VolumeParameter、per-camera sky context/hash、cloud maps/wind/history/shadow组织渲染。Godot、Bevy、Fyrox提供资源持久化、ECS extract和fail-visible capability下限，但五家都不能直接替代Zircon所需deterministic climate/region/network/save authority。

报告登记0个新P0、72个runtime子P1、16个P2与40项资格门。Editor38保留5项P0和P1-01..70父要求；Runtime36只拥有`WeatherProgramArtifact -> WorldWeatherService -> WeatherFrameSnapshot -> typed domain adapters -> receipts`的可执行分解。详见`zircon_runtime/36-weather-climate-celestial-time-of-day-wind-precipitation-cloud-atmosphere-surface-state-determinism-network-save-scalability-product-integration-review.md`。

本轮只新增review、metadata与索引，没有修改production、tests、Cargo、manifest、workflow或产品资产。source/compiler/authority、deterministic transition/region、network/save、domain adapters和产品资格关闭前，不得把静态Workbench、gradient sun、fog density、Rain particle、SDK Weather或fixture名称称为Weather Ready。

## 160. Camera Endpoint / Director / Rig / Controller / Blend / Shake / Cinematic Cut / History / Multi-View / Network / Save / Scalability 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Runtime production与直接consumer | 77 / 17,079 / 641,147 | E3：Scene asset/component、World active camera、controller/script/AI、frame extract、camera loop、viewport history与首方catalog纵向链 |
| 聚焦测试 | 29 / 7,361 / 262,533 | E3静态阅读：101个test/tokio::test属性、0 ignore；覆盖asset roundtrip、controller数学、target/stack/order和局部render history |
| 产品/作者控制面 | 9 / 3,161 / 147,796 | E3：PBR viewer、Editor viewport controller、Sequencer静态面与Editor30父报告 |
| Unreal/Unity/Godot/Bevy/Fyrox参考 | 17 / 11,171 / 467,226 | E2/E3：Camera Component/Manager/SpringArm/Cine/Cut、Gameplay Cameras rig/evaluator/blend/shake/collision、projection与per-camera render data |
| 冻结合计 | 132 / 38,772 / 1,518,702 | SHA-256 `c97f89c409a078c1c515d4f86e9aed73a2a6dc01d68426ff2c409dce22b6d280` |

Zircon已有真实Scene-to-render Camera链：`SceneCameraAsset`保存pipeline、projection、FOV/ortho、clip、surface/texture/headless target、viewport、order、active、HDR、exposure、clear、MSAA和post-process；World extract生成`CameraRenderDescriptor`，camera loop按target/order执行Base/Overlay，viewport record也保存per-camera history。

但Scene没有持久active camera身份，`CameraComponent`的14个字段只有FOV/near/far进入reflection；render type/stack/clear-depth、独立culling/volume mask、dynamic resolution与projection override只存在于DTO。`Perspective`又同时是合法设置和“无override”哨兵，source/property入口也没有统一finite、range与near/far交叉校验。

玩法链只有单一`world.active_camera`：dynamic session固定Orbit controller，脚本可覆盖任意entity transform，AI LOD读取同一全局camera。production没有Rig/Lens/Shake source与compiler、per-player/per-viewport Director、possession/view target、collision、blend、modifier、shake或camera cut event；split-screen、XR、network/save也没有owner。

`ViewportCameraHistoryKey`不包含World/source/director generation、core pipeline、lens/projection epoch；ViewportRecord至少七个keyed map没有retirement/prune。Velocity只能用运动阈值推测`CameraCutOrInvalid`，既不能识别同位置硬切，也可能误判快速连续运动。

报告登记0个新P0、72个runtime子P1、16个P2与40项资格门。Editor30保留5项P0和P1-01..60父要求；Runtime37只拥有`Camera source/artifact -> Director/evaluator -> ViewResult/cut epoch -> render/audio/AI/network adapters -> bounded receipts`的可执行分解。详见`zircon_runtime/37-camera-endpoint-director-rig-controller-blend-shake-cinematic-cut-history-multiview-network-save-scalability-product-integration-review.md`。

本轮只新增review、metadata与索引，没有修改production、tests、Cargo、manifest、workflow或产品资产。Director ownership、compiled program、cut/history retirement、multi-view/network/save和真实产品证据关闭前，不得把可渲染Camera DTO、Orbit控制器或Sequencer静态页面称为Camera System Ready。

## 161. Gameplay Framework / Game Instance / World Context / Level / Game Mode / Game State / Local Player / Controller / Pawn / Possession / Spawn / Travel / Network / Save / Scalability 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Runtime production与直接consumer | 61 / 13,195 / 485,700 | E3：Scene/Level/World、dynamic session、gameplay host、Input、Net、runtime interface与App host纵向链 |
| 聚焦测试 | 24 / 6,398 / 232,453 | E3静态阅读：127个test/tokio::test属性；4个Vampire real-ZrVM测试被显式ignore |
| 产品/父计划控制面 | 19 / 9,393 / 590,847 | E3：Vampire、WOC及Runtime05/08E/37、App01/03/06、Editor07/28父owner |
| Unreal/Lyra/Bevy/Godot/Fyrox/Unity Graphics参考 | 25 / 24,578 / 992,933 | E2/E3：instance/world/rule/player/possession/travel/scene lifecycle与render边界 |
| 冻结合计 | 129 / 53,564 / 2,301,933 | SHA-256 `76af666bf94bb21b2debf112a8ea5dc6500b42335fa9fb266105ebe430630a19` |

Zircon已有可保留的World/ECS generation、Level wrapper、project load/save、dynamic session、Input manager、network handshake/replication DTO和bounded host-request ABI。它们可以作为Gameplay Framework底层，但当前产品拓扑仍是一个session直接持有一个Level、一个全局InputManager和一个固定Orbit camera controller。

Game Instance、World Context、Experience/Game Mode、Game State、Local Player、Player State、Controller、Pawn与Possession没有Runtime产品owner。相关概念与Travel词的联合检索只有18个命中，全部来自Editor静态展示文案或script migration测试。Vampire用`role="player"`字符串、全局WASD和硬编码camera entity `1`实现玩法，不能证明typed player lifecycle。

`request_scene_transition`只把`ReplaceActive`请求写进World单槽resource；host ABI只有IME、rumble与cursor，dynamic session也只收集这三类请求，全产品没有transition consumer、执行状态或terminal result。LevelManager同时只有插入/查询/load/save，没有enumerate/active/unload/remove/travel；`Unloaded`与字符串subsystem注册只在测试写入，registry会随load持续增长。

参考侧，Unreal把GameInstance/WorldContext、权威GameMode、复制GameState/PlayerState、Controller-Pawn possession和Level Streaming状态机分owner；Lyra把Experience做成可组合、异步激活的产品定义。Bevy、Godot与Fyrox分别提供App/SubApp和state enter/exit、current-scene change/unload与node lifecycle、SceneContainer及plugin init/deinit。Unity Graphics本地镜像只证明scene load时render-pipeline选择边界，不支持完整Unity Gameplay Framework结论。

报告登记0个新P0、72个runtime子P1、16个P2与40项资格门。Runtime05/08E/37、App01/03/06和Editor07/28继续拥有World/Network/Camera/Product/PIE/Spawn父问题；Runtime38只拥有`GameInstance -> WorldContext -> Experience/GameRule -> Player/Controller/Pawn/Possession -> Travel/Network/Save adapters -> receipts`的可执行分解。详见`zircon_runtime/38-gameplay-framework-game-instance-world-context-level-game-mode-game-state-local-player-controller-pawn-possession-spawn-travel-network-save-scalability-product-integration-review.md`。

本轮只新增review、metadata与索引，没有修改production、tests、Cargo、manifest、workflow或产品资产。scene-transition consumer、Level retirement、player ownership、authority、travel rollback和产品闭环关闭前，不得把单Level脚本演示称为Gameplay Framework Ready。

## 162. Prefab / Archetype / Prototype / Class Default / Instance Override Runtime 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Runtime Prefab、asset与Dynamic Scene | 60 / 12,827 / 463,752 | E3：DTO、cache、importer/registry/module wiring、World IO、spawn/reload transaction纵向链 |
| `prefab_tools` package | 15 / 908 / 33,207 | E3：manifest、runtime/editor/dist registration和helper；无instancer |
| 聚焦测试 | 8 / 2,843 / 106,354 | E3静态阅读：50个test属性、0 ignored；覆盖DTO/cache/registry/Dynamic Scene结构，无Prefab E2E |
| 产品/父计划控制面 | 18 / 8,490 / 678,992 | E3：Vampire、WOC与Asset/World/Editor/Plugin父owner；产品资产零Prefab |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics参考 | 26 / 17,253 / 649,378 | E2/E3：archetype/reinstance、SceneState、inheritance/remap、resolved patch与override stack |
| 冻结合计 | 127 / 42,321 / 1,931,683 | SHA-256 `63617f8adc10077fb354239974d3805232a422835e6aee98be1ad250d87d07a7` |

Zircon已有Prefab DTO、project/cache serialization、内建`.prefab.toml`解析、typed asset load，以及带World/schema/component/change-tick generation检查、隔离preflight、bounded staging和无失败publication的Dynamic Scene事务。这些可以承接compiler/artifact与实例化底座，但生产代码没有任何Prefab instantiation consumer或Instance Registry。

内建导入器与`prefab_tools`插件以相同`.prefab.toml` suffix和默认priority 0争用authority：前者泛型解析成功，后者只返回backend未安装；registry把冲突视为fatal duplicate，一条active registry合并路径却用`let _`吞掉注册错误。插件还宣称instancing service而实际只注册component descriptor和诊断导入器，capability、catalog与行为不一致。

Dynamic Scene reload监听的是`SceneAsset`并再次spawn，临时`EntityRemap`不保存source/revision/instance identity、旧entity set或反向索引；它不会回收旧实例，也不做old base/new base/local override三方rebase。更严重的Scene World roundtrip数据损失继续由Editor44/41父owner承载：load忽略`prefab_instance`，save固定写`None`。

报告登记0个新P0、72个runtime子P1、16个P2与40项资格门。Editor44的5个P0及Editor41、Runtime04/05/24/08E、Plugins01继续拥有authoring/persistence/asset/world/identity/network/package父问题；Runtime39只拥有`unique import authority -> resolved artifact -> Instance Registry -> transactional instantiate -> provenance/rebase/update -> streaming/network/save -> receipts`。详见`zircon_runtime/39-prefab-archetype-prototype-class-default-instance-override-runtime-instantiation-propagation-hot-reload-network-save-scalability-product-integration-review.md`。

本轮只新增review与索引，没有修改production、tests、Cargo、manifest、workflow或产品资产。唯一importer、无损codec、instancer、stable provenance、replace/rebase、network/save和真实产品E2E关闭前，不得把Prefab DTO、component descriptor或再次spawn称为Prefab Runtime Ready。

## 163. SaveGame / Checkpoint / Slot / Participant / Migration / Platform / Cloud Runtime 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Dynamic Scene Session archive | 565 / 10,510 / 360,657 | E3完整文件级扫描：slot/artifact/manifest/retention/path API/writer，无SaveGame身份、外层迁移或durable commit |
| Runtime/platform/schema底座 | 96 / 16,839 / 552,054 | E3：Dynamic Scene capture/restore、preferences、resource transaction与versioned serialization |
| Native plugin与Zr VM热重载状态桥 | 8 / 3,547 / 130,201 | E3：schema、save/restore callback、skip/error与真实VM backend |
| focused external tests | 23 / 7,853 / 288,375 | E3静态阅读：151个test属性/宏、0 ignored；无SaveGame产品E2E |
| WOC/Vampire产品证据 | 18 / 74,667 / 3,585,056 | E3：WOC world writer/transaction/client preferences及Vampire fixture；无slot/storage/cloud闭环 |
| 父计划控制面 | 13 / 5,268 / 551,637 | E2：Editor24、Runtime05/12/24/38、App03/05/06、Interface02、Plugins01唯一owner |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics参考 | 20 / 12,055 / 513,894 | E2/E3：platform SaveGame、resource IO、Visitor、scene/reflect与序列化容器边界 |
| 冻结合计 | 743 / 130,739 / 5,981,874 | SHA-256 `483035db8ac9fa636bacdecef1ba749949e8874c7a9aec8ea8b660574c244137` |

Zircon已有565文件的Session archive、Dynamic Scene隔离preflight/commit、Runtime Interface versioned migration、platform preferences mutation/durability状态，以及core resource IO的fsync/journal/recovery。这些是真实且可保留的底座，但仓内没有SaveGame service、player/profile/platform-user identity、participant registry、schema catalog、cloud provider或任何产品caller。

Session archive当前外层固定version 1且无migration chain，artifact以canonical JSON保留最多512 MiB完整bytes；atomic writer只`BufWriter::flush()`，没有文件/parent directory `sync_all`，stale revision依赖进程内HashMap，`.tmp/.bak`无startup recovery。`load_or_empty_from_path`在36个文件出现41次并把NotFound映射为空archive；这会混淆首次创建、路径错误、用户切换与数据丢失。

Dynamic Scene capture接收serializable字段，restore却只写`serializable && editable`字段，因而存在成功capture/load但静默丢字段的确定风险。Native plugin与VM状态只服务best-effort hot reload；WOC `saveState`不代表world且WOS113/118分裂，Vampire保存常量并忽略restore输入。玩家SaveGame、server checkpoint、network snapshot与hot-reload snapshot必须建立不同identity/authority/lifetime，不能共用裸slot字符串或best-effort恢复。

报告登记0个新P0、72个runtime子P1、16个P2与40项资格门。Editor24 P0-3/P0-4继续拥有SaveGame服务缺失和Dynamic Scene误复用根问题；App03 P0-5/P0-6、Runtime12及App06继续拥有WOC schema/transaction/kernel与Vampire产品fixture。Runtime40只拥有`service identity -> participant registry -> consistent capture/restore -> versioned envelope/migration -> durable platform/cloud/server storage -> receipts/qualification`。详见`zircon_runtime/40-save-game-checkpoint-slot-participant-capture-serialization-migration-platform-cloud-async-network-scalability-product-integration-review.md`。

本轮只新增review与索引，没有修改production、tests、Cargo、manifest、workflow或产品资产。无损roundtrip、断电恢复、跨进程CAS、历史迁移、cloud conflict、真实Vampire/WOC跨进程恢复和规模资格关闭前，不得把Session archive、hot-reload blob或fixed-tick snapshot称为SaveGame Ready。

## 164. Operation Service / Registry / Admission / Prepare / Apply / Progress / Cancel / Shutdown 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `zircon_runtime::operation`完整目录 | 11 / 2,639 / 95,277 | E3逐文件：23个test属性、0 ignored；registry/admission/task/completion/maintenance/harvest全覆盖 |
| ABI、dynamic session与Editor gateway | 16 / 4,310 / 151,672 | E3：V7 operation函数表、bounded JSON、session tick/wake/frame demand和三类gateway |
| navigation真实producer/consumer | 12 / 2,083 / 71,958 | E3：唯一handler、两个runtime driver、Editor command与编译漂移focused test |
| 父计划与唯一owner | 10 / 4,809 / 459,343 | E2：Runtime02/08D/24、Editor09/19、Interface01/05、Tooling37与全局P0总账 |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics参考 | 13 / 4,988 / 173,842 | E2/E3：cancel/finish/shutdown、task group progress、scope、owner completion、process/GPU drain |
| 冻结合计 | 62 / 18,829 / 952,092 | SHA-256 `015542c0edfdf11b31978b61a8e0dde011c3e42f70949686a49fbf5481286eef` |

Zircon已有session级handler registry、task/retained-byte双容量、decode前reservation API、owner snapshot、worker prepare、owner apply、panic隔离、deadline/cancel、terminal TTL和两阶段harvest。这些基础应保留，但请求没有stable identity、principal、priority、deadline或idempotency，registry没有descriptor/schema/owner lease，调度从HashMap无序选取，progress又固定为0/1。

取消和deadline只停止结果发布，running prepare没有token且继续占用worker；snapshot文档声称immutable却能调用`world_mut()`，apply可先产生副作用再返回Failed而没有effect disposition或receipt。submit不会请求runtime wake，session frame demand也不观察Operation状态；reactive host可能接受任务却不再推进。dynamic ABI只提供submit/poll/harvest，in-process gateway则直接报告capability missing。

唯一生产consumer是navigation。Bake Scene/Surface的prepare固定失败，Editor command只同步yield/poll 16次，plugin focused test仍调用旧`poll(context, handle)`并期待Bake成功。Editor19 P0-1/P0-3、Runtime02 P0-1、Interface01 P0-05与App01 P0-3继续拥有根阻断；Runtime41不重复计P0，只拥有descriptor/identity、确定性调度、合作取消、可信progress/effect receipt、wake/result delivery与Operation领域shutdown。

报告登记0个新P0、48个runtime子P1、12个P2与40项资格门。详见`zircon_runtime/41-operation-service-handler-registry-admission-prepare-apply-progress-cancel-deadline-harvest-retention-shutdown-product-integration-review.md`。本轮没有修改production、tests、Cargo或ABI；在真实navigation Bake和第二个重型consumer、reactive wake、in-flight cancel、result paging、session/plugin drain以及并发/scale资格关闭前，不得把当前轮询器称为工程级Operation Control Plane。

## 165. Builtin Runtime Module / Catalog / Profile / Target / Feature / Extension Assembly 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `zircon_runtime/src/builtin`完整目录 | 30 / 3,283 / 127,500 | E3逐文件检查公开入口、ID、manifest、availability、profile/target assembly、extension flatten、load report与测试 |
| Profile schema、availability、catalog merge与extension registry | 26 / 3,919 / 139,039 | E3反查生成源、capability消费、provider membership、正式merge/order/finalize路径 |
| App与dynamic session真实consumer | 6 / 1,248 / 44,384 | E3核对重复composition、final graph、fatal handling与lifecycle observer安装 |
| 父报告与唯一owner | 7 / 3,032 / 336,615 | E2核对P0归属、生命周期、脚本、产品host和first-party catalog边界 |
| Unreal、Bevy、Godot、Fyrox、Unity Graphics参考 | 22 / 16,511 / 613,477 | E2/E3核对module phase、target policy、plugin lifecycle、package/assembly依赖和reload边界 |
| 冻结合计 | 91 / 27,993 / 1,261,015 | SHA-256 `17ba34f9403b619cb65d77da5828c68d4b96f5675750b8c34bd5fdd9881230ed` |

Zircon已有6个typed Profile、12个内建模块、target候选、内建依赖闭包、插件availability分类、Core拓扑排序和结构化load report。这些底座可以保留，但`runtime-feature-presets.toml`、builtin assembly、`RuntimePluginCatalog`与`zircon_app` PluginGroup各自决定一部分composition，最终没有共享不可变plan、generation、BuildSet或commit receipt。

最直接的正确性断路是registration过滤：plugin registration只检查自身默认selection，不检查本次project manifest，因而未选择或禁用插件的importer/render extension仍可进入模块；feature registration又只按feature id匹配，同一feature的多个provider会全部拍平注入，而正式catalog只选一个provider。Builtin路径也绕过catalog的extension merge/order/conflict/finalize，App随后再次构建catalog和module group，load report因此不是最终产品图。

Profile的`required_capabilities`没有生产consumer；`#[cfg]`会直接删除Graphics/Script枚举variant和生成成员，使同一Profile与序列化schema随BuildSet改变；target快捷入口不选择Profile，Server固定保留Input却排除Script，EditorModule又在Profile图之外追加。Plugins06、Plugins01、App01、Runtime01/07/21继续拥有required provider、native trust、产品host/lifecycle和script/plugin generation的既有P0；Runtime42不重复计P0。

报告登记0个新P0、52个runtime子P1、14个P2与42项资格门。详见`zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md`。本轮没有修改production、tests、Cargo、ABI或产品资产；在单一`RuntimeCompositionPlan`、effective selection、唯一provider、capability admission、稳定schema、transactional activation/rollback、final graph receipt与真实Client/Editor/Server矩阵关闭前，不得把当前builtin assembly称为工程级Composition Compiler。

## 166. Dynamic Runtime Session / Registry / FFI / Frame / Event / UI / World Sync / Shader Prewarm 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `zircon_runtime/src/dynamic_api`完整实现与测试 | 76 / 17,955 / 652,225 | E3逐文件检查export、session registry、FFI、frame/event/UI/world/operation、bounded JSON、shader prewarm和测试 |
| `zircon_runtime_interface/src/runtime_api`完整V7合同 | 11 / 2,251 / 68,849 | E3反查session/event/viewport/frame demand/host request/plugin event/operation固定布局与能力表达 |
| App动态库宿主与shader prewarm真实consumer | 5 / 3,089 / 114,990 | E3核对API装载、session包装、foreign output释放及CLI/cache调用链 |
| 父报告与唯一owner | 18 / 7,851 / 779,967 | E2核对P0归属、模块生命周期、任务、world、render、UI、identity、operation、composition和产品边界 |
| Unreal、Bevy、Godot、Fyrox、Unity Graphics参考 | 18 / 43,341 / 1,614,620 | E2/E3核对engine/world context、plugin phase、runner/sub-app、input/UI、PSO batch和render graph cache |
| 冻结合计 | 128 / 74,487 / 3,230,651 | SHA-256 `cdac75859748f3addbe8e09fd43ed0e23214617f52472b501ac4f184c0071850` |

Dynamic API已经具备V7固定函数表和panic边界、session action quiescence、foreign allocation owner校验、bounded JSON、plugin/world分页、frame extract cache/诊断以及真实WGPU shader module validation。这些是可保留底座，但当前session/allocation仍由进程全局裸`u64` registry管理，所有tick、render、query、UI和operation在单个session mutex内执行；destroy可无限等待callback/action，内部module drain却使用零时长预算，bootstrap补偿失败还可能终止整个宿主进程。

Profile和project startup形成第二套composition truth。Minimal/Headless仍映射ClientRuntime并注入脚本，linked registration可在缺project selection时自动启用，scene/navmesh/startup scripts绕过统一VFS/cook/transaction，`play_report_pipe`校验后被丢弃。无render bridge或pipelined frame未完成时返回成功黑帧；仅支持default viewport和Win32 surface，extract命中仍深clone且失效key不完整。

输入缺timestamp/device/user/window generation，touch退化为单cursor；host request分组排空会重排因果顺序。Runtime UI同步扫描全部UI资产、忽略alias冲突、以manifest顺序和16/48位mask制造identity，并在无UI时返回伪accessibility tree；engine内还硬编码Vampire HUD/menu/文案和component写入。Plugin event无wake，world invalidation无remaining且逐项`remove(0)`；shader prewarm固定六pass、literal版本、每次新建WGPU backend并可把template错误静默变成空manifest/fallback source。

报告登记0个新P0、64个runtime子P1、16个P2与42项资格门。Interface01/03/05、Runtime01/02/05/07/09A/09C/11A/22/24/41/42、App01/06与Plugins05继续拥有ABI、lifecycle、task/world/plugin/render/UI/time/identity/operation/composition/host/product/shader既有P0；Runtime43不重复计数。详见`zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md`。

## 167. Process Diagnostic Log Router / Filter / Record / Queue / Sink / Durability / Rotation / Crash / Multi-Session 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `zircon_runtime/src/diagnostic_log`完整实现与测试 | 31 / 3,885 / 126,766 | E3逐文件检查filter、record、queue、worker、metrics、file、panic、flush、shutdown及性能harness |
| Runtime/App/plugin真实集成面 | 18 / 7,023 / 263,499 | E3核对module descriptor、动态session lease、二进制入口、退出码、Tracy与native host log |
| Editor独立logging owner | 13 / 1,889 / 59,980 | E2核对journal、rolling file、output console与process log重复authority |
| 父计划、failure与跨owner报告 | 6 / 2,257 / 272,482 | E2确认唯一owner、开放阻断和不得重复计数的P0 |
| Unreal、Bevy、Godot、Fyrox参考 | 11 / 3,878 / 130,016 | E2/E3核对redirector/fence/panic device、tracing layer、rotation/composite与最低实现基线 |
| 冻结合计 | 79 / 18,932 / 852,743 | SHA-256 `0a5ebd1e47eb9a2ab144e693897ccb4f060e49747dbbc2ee332413fc027830f5` |

Process diagnostic log已有compiled filter、lazy callsite、有界count queue、batch worker、基础metrics、flush/shutdown命令、panic hook及dynamic session lease。这些底座可以保留，但`OnceLock`按链接映像存在，使App静态Runtime与动态Runtime DLL在同一进程各自建立sink；模块descriptor不拥有真实service lifecycle，首次初始化永久决定配置，session也没有project/world/frame/generation身份。

Record只有level/scope/message，时间戳在worker输出时才生成；没有结构化字段、callsite、sequence、thread/span/session或隐私策略。队列不限制owned bytes，Warn/Error满队列时执行无时限阻塞send，却只保证进入RAM。Console/file在单worker串行写，sink error/panic无隔离监督；flush与数据争抢同一满队列，文件又没有rotation、retention、quota、exclusive identity、manifest或可声明的sync等级。

Panic hook在panic payload写出前flush且只覆盖Rust panic；Editor另有rolling file，`tracing`主要只接可选Tracy，plugin `host_log`多数为空，diagnostic snapshot还会逐series喷入同一队列。报告登记0个新P0、52个runtime子P1、14个P2与36项资格门，并继承仍为open的Runtime07 synchronous-sink failure；Runtime03/07、Runtime43、App01、Editor11/25、Interface01/03/04和Plugins01继续拥有既有父合同。详见`zircon_runtime/44-process-diagnostic-log-router-filter-record-queue-sink-durability-rotation-crash-multi-session-product-integration-review.md`。

## 168. Preference / Settings / Scope / Storage / Overlay / Bounded I/O / Generation / Fence / Durability / Migration / Multi-Process 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Framework preference contract | 5 / 493 / 12,593 | E3逐文件检查key、backend kind、error及snapshot/mutation/fence中立合同 |
| Bounded keyed I/O | 12 / 2,431 / 78,763 | E3逐文件检查admission、generation、coalescing、fence、deadline、cancel、shutdown、panic和测试 |
| Platform backend、lifecycle与App注入 | 20 / 4,902 / 173,873 | E3核对overlay、atomic file、backend replacement、module cleanup、capability、默认root与descriptor factory |
| Editor与WOC真实consumer | 46 / 9,660 / 310,822 | E2/E3核对第二settings authority及settings/keybind/gamepad/offline/inventory生产调用链 |
| 开放failure与父报告 | 7 / 1,811 / 202,778 | E2确认三个failure保持open并划分Editor12、Runtime03/25/40边界 |
| Unreal、Godot、Fyrox参考 | 23 / 21,127 / 787,210 | E2/E3核对config context/hierarchy、dirty/validate/apply/confirm/migrate、metadata/change tracking和Rust editor组织 |
| 冻结合计 | 113 / 40,424 / 1,566,039 | SHA-256 `0ab3e6f7ee694d007190303cd1f9ca795fefcc7b3cc7a806363f1c2156ed31bf` |

Preference链已有中立storage、read-your-write overlay、entry/byte双容量、bounded cold read、generation coalescing、fence prerequisite、panic/deadline/shutdown metrics与shared atomic writer底座。但backend primitive成功会直接把mutation标为Durable，而HostProvided可能尚未flush，module cleanup也没有final flush fence；WOC的refresh与submission harvest只有测试caller，Pending在生产模型里固定成default。

全局单active lane使一个永久hung backend冻结所有key/fence/shutdown，cleanup超时后guard drop仍可无界等待。App默认root固定为`ZirconEngine/preferences`且key无product/project/channel/profile/account owner；跨进程又没有revision/CAS/lock/watch，整份WOC JSON可last-writer-wins丢更新。成功value/tombstone无淘汰路径、默认cold-read预算只容纳一个最大读、Editor还有第二套settings document/lane authority。

报告登记5项P0、58项P1、14项P2与40项资格门。Framework05两项与Runtime11 failure保持open；Editor12、Runtime03、Runtime25与Runtime40继续拥有UI/config/filesystem/SaveGame父边界。详见`zircon_runtime/45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md`。本轮没有修改production、tests、Cargo或ABI；在真实durability receipt、hung-I/O bounded teardown、product/principal isolation、multi-process conflict和WOC/Editor产品hard cut完成前，不得把当前Preference称为工程级持久化服务。

## 169. Engine Module / Service Contract / Context / Factory / Descriptor Snapshot / Composition / Lifecycle 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `zircon_runtime/src/engine_module` facade与测试 | 8 / 509 / 14,323 | E3逐文件检查author接口、descriptor、service contract、identity helper、prelude投影与合同测试 |
| Core descriptor/context/lifecycle与注册、解析、激活、状态consumer | 24 / 3,402 / 122,927 | E3核对collect/freeze/register/activate/lazy resolve、slot状态、owner/kind/dependency校验与devtools snapshot |
| Runtime builtin assembly、load report与App composition/bootstrap consumer | 11 / 2,373 / 81,344 | E3逐代跟踪profile cache、group replace/finalize、entry snapshot和Platform host factory绑定 |
| 26个生产`EngineModule` owner反查 | 26 / 2,358 / 85,016 | 横切集合，与前三行有重叠；逐owner核对name/description/descriptor/lifecycle/factory，确认Asset description真实漂移 |
| VM plugin context与package host consumer | 4 / 1,593 / 54,941 | E3核对Core PluginDescriptor factory与VM package context的身份、root、generation和权限域混用 |
| 父计划与开放failure | 5 / 1,780 / 164,038 | E2确认descriptor regeneration failure继续保持open，并修正其未覆盖Runtime选择阶段首次generation的范围 |
| Unreal、Bevy、Godot、Fyrox参考 | 16 / 15,310 / 563,961 | E2/E3核对module manager/status/load phase、plugin staged lifecycle、init level/reload与registration/runtime context分域 |
| 去重冻结合计 | 93 / 26,955 / 1,073,201 | SHA-256 `a9f0c987934bc148cfc7e2abd2ef26e9a6afbf362ed7d9d62f9a32e2d8daac83`；production owner横切行不重复计入合计 |

Engine Module链已有Core依赖图冻结、ModuleLifecycle build/ready/finish/cleanup、module activation panic隔离、service owner/kind/dependency校验和运行态snapshot底座。但author trait与descriptor重复拥有identity/description，Runtime selection缓存的descriptor被丢弃，App会再次resolve、replace后清空cache、finalize再生成，bootstrap又按字符串patch Platform factory。Lazy factory在slot进入`Initializing`后直接调用closure，panic会越过reset/notify；`EngineService`及marker没有生产consumer且可表达矛盾metadata，Core与VM的PluginContext也混用了不同service/package身份与filesystem authority。

## 170. Editor Runtime Gateway / Session / Event Consumer / World Sync / Reconnect / Shutdown 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Gateway、runtime event consumer、World Sync核心 | 34 / 5,860 / 196,476 | E3逐文件检查ArcSwap generation、session owner、subscription、bounded pump、pending tail、watch map与world projection |
| 聚焦测试 | 15 / 4,043 / 137,591 | E3静态阅读106个test attributes、2个managed ignored lane；覆盖预算、公平、错误与顺序替换，缺并发replacement/host shutdown矩阵 |
| Editor Host与Retained Host产品consumer | 11 / 2,911 / 120,264 | E3核对startup、extension registration、active tick、hierarchy refresh、menu exit、Drop和diagnostics |
| App RuntimeSession与Editor composition | 6 / 1,998 / 77,467 | E3确认Arc owner、foreign output、gateway转移、session destroy与动态库寿命，不错误宣称UAF |
| Runtime Interface与producer | 8 / 1,470 / 54,802 | E3核对V7、plugin page metadata、world invalidation分页及prepare/commit/rollback |
| 父计划与开放failure | 9 / 1,252 / 122,975 | E2复核已落地的ArcSwap、bounded pump、indexed watch与registration atomicity，并纠正hierarchy caller旧结论 |
| Unreal、Godot、Fyrox、Bevy参考 | 15 / 9,953 / 347,606 | E2/E3按session/instance identity、bridge teardown、disconnect、20ms pump预算、plugin lifecycle与无界observer局限路由 |
| 去重冻结合计 | 98 / 27,487 / 1,057,181 | SHA-256 `bfbf925c215138c729b69dfe3a27745ad82d977ceaff1ca4954da4a081905fb3` |

Editor gateway单次调用已能固定Arc snapshot，SessionGateway也由`Arc<RuntimeSession>`保持动态库和session存活；新P0是多步协议没有固定同一origin identity。World Sync先读取generation再由第二个snapshot drain，runtime event active entry又不保存subscription创建时的gateway generation/session owner，replacement时复用的opaque token可能跨项目/PIE投影或取消。World page没有cursor/final/gap合同，bus dispatch结果被忽略后仍增加published并推进watermark；plugin begin/consume/end callback没有panic fault domain；Controller与Retained Host Drop又不能证明consumer、watch、output、gateway和RuntimeSession按终态顺序退休。

## 171. Editor Message Bus / Topic / Subscription / Inbox / Retention / Admission / Dispatch / Request / Dirty Projection / Shutdown 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `core/editor_message`实现 | 35 / 2,935 / 93,365 | E3逐文件检查topic/payload、subscription、inbox、retention、count/byte budget、dispatch plan/commit、request与report |
| 聚焦测试 | 13 / 1,642 / 59,366 | E3静态阅读bus/retention/byte budget/request/dirty集成测试，并对照生产caller识别测试断言与真实零subscriber路径分叉 |
| 直接产品producer/consumer链 | 18 / 6,796 / 253,338 | E3反向核对Host、Retained Host、Plugin、Job、Transaction、Play、Logging、I18n、World Sync与Scene inspection |
| 设计、父计划与开放failure | 8 / 1,408 / 123,947 | E2复核既有fanout/backpressure、job reservation、subscriber result与Editor owner边界，不把计划状态当实现完成 |
| Unreal、Bevy、Godot、Fyrox、Unity Graphics参考 | 17 / 6,515 / 219,669 | E2/E3按subscription lifecycle、owner-bound unregister、queue OOM/flush guard、typed single-consumer与scoped provider/fixed buffer适用性路由 |
| 去重冻结合计 | 91 / 19,296 / 749,685 | SHA-256 `3b7e94575b94bc84a45e0b99e3681b89eeafaddd371d2edac711e47c16c4a7da` |

Message Bus已有共享payload、per-subscriber inbox、三类retention、双容量预算和lossless预检基础；新P0位于产品投影边界：`publish_view_invalidation()`发布合法`view.invalidated`并依赖bus标记dirty，但bus仅在`delivered`非空时标记，而生产没有该topic subscriber，导致refresh路径可返回空dirty并跳过UI更新。其余关键缺口包括拒绝后消息所有权丢失、unregister静默销毁lossless pending、无subscription generation/RAII/ack/resync/shutdown、Latest key缺topic/scope、plugin二次无界pending及多个topic没有production consumer。

## 172. Editor Event Runtime / Envelope / Listener Registry / Journal / Replay / Snapshot / Dirty / Lifecycle 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `core/editor_event`完整生产模块 | 36 / 2,667 / 85,420 | E3逐文件检查event schema、stamp、journal、retention、listener registry/route/page/control与replay |
| `tests/editor_event`聚焦测试 | 30 / 8,109 / 293,454 | E3 inventory 138个test attributes并深读retention/listener/registry/replay/retained/project/trace主链 |
| 直接产品caller与F5消费链 | 13 / 3,970 / 148,082 | E3追踪pointer callback、host dispatch/effect/undo、Context owner、retained automation、App composition/report与MVP assertion |
| 模块文档、failure、性能与owner计划 | 13 / 3,466 / 350,278 | E2/E3复核forward repair、开放验证、F5边界及跨报告唯一owner |
| Unreal、Bevy、Godot、Fyrox、Unity Graphics参考 | 15 / 16,847 / 566,660 | E2/E3按transaction、delegate/signal lifetime、reader cursor、queue admission、command/message分层与provider scope路由 |
| 去重冻结合计 | 107 / 35,059 / 1,443,894 | SHA-256 `af663ee796aa7f191f48c880135e3af3c2d6ab9f9384c29c9b966fd2934692fd` |

Editor Event已有sequence/revision分离、共享`Arc` record、三类有界retention、latest index、listener cursor、锁外route及lag/drop诊断基础；五项P0位于产品证据、重放与热路径边界：F5用global journal长度差切片，retention/coalesce后可越界或误归因；随后把RetainedHost记录改写为CLI provenance；raw replay重新执行输入、失败和外部副作用；pointer move同步支付完整审计链；revision在执行前推进而成为attempt counter。其余关键缺口包括envelope无qualified scope/schema/provenance、journal ordering合同矛盾、listener无generation/page bytes/gap/resync/可信ack/shutdown、delivery DTO遗漏事件语义，以及listener/replay无production consumer。本轮只写review与重构计划，未运行Cargo、F5或性能捕获，也未修改production与tests。

## 173. Editor Extension / Contribution Store / Registry / Toolkit / Provider / Snapshot / Reload / Lifecycle 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `core/extension`完整模块 | 30 / 5,267 / 170,756 | E3逐文件检查ContributionStore、batch、snapshot、lifecycle、Inspector/field editor、document toolkit与同目录测试 |
| extension descriptor合同 | 6 / 1,959 / 61,491 | E3检查registry、authoring descriptor、template、view、overlay和capability字段 |
| `core/plugin`完整模块 | 35 / 6,323 / 227,022 | E3检查catalog、manager phase/state、materialization、registration、isolation、SDK与manager tests |
| scene mode / overlay runtime | 23 / 3,015 / 97,662 | E3检查registry、prepare/install、active instance、callback isolation、capability与viewport消费 |
| Workbench/Host产品消费闭包 | 31 / 5,999 / 227,905 | E3反查startup、snapshot、Inspector/template/asset/toolkit、dirty与UI投影 |
| 聚焦产品测试 | 8 / 4,124 / 150,688 | E3检查plugin registration、overlay lifecycle、toolkit、SDK、validation与Workbench projection |
| App composition/startup | 5 / 2,416 / 90,553 | E3检查first-party/native registration来源、RunConfig和RetainedHost直接安装 |
| 当前计划与owner文档 | 12 / 3,168 / 305,918 | E2/E3复核历史failure、性能审查及Editor02/05/06/08/12和Plugins01去重 |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics参考 | 14 / 16,849 / 638,005 | E2/E3按owner unregister、module/plugin phase、toolkit lifecycle和provider discovery/priority路由 |
| 去重冻结合计 | 164 / 49,120 / 1,970,000 | SHA-256 `a265ff46731682f428a5fe264cae3bf093fec0f3db160c1ab591fceb38bf87ea` |

Extension链已有immutable contribution snapshot、ticket revoke、有界change journal、scene/overlay panic boundary、context checkpoint和toolkit save/close lease等基础。五项P0位于产品激活、撤销、能力与callback边界：Plugin Manager每代构建的`active_extensions`没有production reconciler，而RetainedHost startup直接永久安装另一套Workbench状态；项目native Active registration也不挂载。产品无统一revoke/quiesce，scene/overlay/importer capability规则不一致，Inspector/field editor/pane source callback缺统一隔离且部分在shell锁内执行，DocumentToolkitRegistry还在mutex内调用trait/Drop并与DirtyRegistry分裂提交。

报告建立manager desired set到mounted set的唯一ExtensionReconciler、qualified owner-generation mount lease、跨registry prepare/commit/rollback/revoke、compiled effective capability、callback supervisor和toolkit纯数据snapshot，登记5项P0、60项P1、15项P2与40项资格门。本轮只写review与重构计划，未修改production/tests，未运行Cargo、GUI、reload或性能捕获。详见`zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md`。

## 174. Hub Command / Action / Message Delivery / Task / History / ViewModel / Localization 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Hub control-plane生产与直接source | 45 / 12,624 / 480,641 | E3逐文件检查action request/dispatch、queue/worker、task projection、history/persistence、message schema/localization、ViewModel与OS action边界；134个selected unit tests |
| Hub integration contracts | 39 / 19,261 / 708,770 | E3分类270个integration tests；仅`project_management_contract.rs`链接产品crate，38个文件以source/doc/TSX string guard为主，只有9个测试直接执行production business type |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics参考 | 14 / 8,695 / 309,066 | E2/E3按cancelable async notification、real work progress、message log、project preflight/migration、multi-task registry、typed message cursor和provider适用性路由；2个reference tests |
| 去重冻结合计 | 98 / 40,580 / 1,498,477 | SHA-256 `fd1e66a7f681c3b24634b211d621c8cbb8e56670ea87e2bdd1c6087f7f01dd30` |

Hub action链已有closed action ID、structured message ID、atomic config writer、process executor、bounded visible history与locale projection等可保留基础；五项P0位于权限、目标代际、任务生命周期、事务与敏感信息边界：任意absolute output path可从WebView进入系统shell，queued action不冻结target且执行会改写全局选择，无界进程内队列和单一TaskStatus不能取消/恢复/证明终态，外部effect与history/config不是可恢复commit，raw command/log又被未分级持久化和投影。

报告建立versioned CommandEnvelope/ActionDescriptor、immutable TargetLease、durable TaskRegistry/worker supervisor、EffectLedger/OperationReceipt、redacted audit history、typed MessageCatalog与generation-consistent read model，登记5项P0、60项P1、15项P2与40项资格门。本轮只写review与重构计划，未修改production/tests，未运行Cargo、Tauri、OS explorer或性能捕获。详见`zircon_hub/04-command-action-message-delivery-task-history-view-model-localization-product-integration-review.md`。

## 175. Editor Project Startup / Open / Create Authority / Hub Handshake / Session Guard / Focus / Recent / Recovery 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Editor authority/session | 62 / 11,677 / 406,451 | E3逐文件检查ProjectAuthority、SessionGuard、Hub link、recovery、EditorManager activation/rollback/close；126个selected tests |
| Editor product tests | 63 / 6,661 / 252,116 | E3分类startup/session/Welcome/recent/retained-host contracts；74个selected tests，明确区分source-shape与产品执行证据 |
| App composition | 12 / 2,942 / 111,503 | E3检查manifest到runtime/editor/native plugin选择、bootstrap顺序与handshake传递；55个selected tests |
| Hub counterpart | 10 / 1,511 / 53,396 | E3检查editor launch、focus publish、ready polling与action projection；17个selected tests |
| Interface/runtime contract | 42 / 3,171 / 107,304 | E3检查hub protocol、session lock、manifest summary、runtime project open与durable transaction底座；37个selected tests |
| 去重冻结合计 | 189 / 25,962 / 930,770 | 309个selected Rust `#[test]`、0 ignored；SHA-256 `722174e750c1db79e7b728e6ba923e72167557f800fac4de1a3d432f818ddae7` |
| Unreal、Godot、Fyrox、Bevy参考；Unity Graphics不适用 | 13 / 29,487 / 1,065,419 | E2/E3按compatibility/migration/recovery mode、process switch/child owner、runner lifecycle与0 applicable Unity Project Manager source路由；SHA-256 `b8e41fcf56e80abde2ff74c644b7016a141a139ba76c8cd55c9b0b3c25465bf8` |

项目链已有staged create、canonical open、SessionGuard、Hub launch/focus mailbox和activation rollback等可保留基础；五项P0位于准入、回滚、ready状态、focus交付和兼容恢复边界：项目派生plugin在exclusive admission/compatibility前加载，runtime rollback关闭失败后仍释放guard，Claimed/Activating被Hub误当Ready，一次性startup watcher不会随Welcome后Open或项目切换重绑且无ack，直接Editor open又丢弃Engine/BuildSet/migration/Safe Mode决策。

报告建立versioned ProjectLaunchIntent、data-only ProjectPreflight、Claimed/Activating/Ready/Closing admission lifecycle、reversible activation effect ledger、generation-qualified focus inbox/ack、first-present ready receipt、bounded recent projection与Safe/Recovery Mode，登记5项P0、60项P1、15项P2与40项资格门。本轮只写review与重构计划，未修改production/tests，未运行Cargo、双进程Hub+Editor、真实窗口、GPU first-present或性能捕获。详见`zircon_editor/51-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-product-integration-review.md`。

## 176. Runtime Interface Project Manifest / Session Lock / Hub Protocol / Focus / Recent 跨进程合同物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Interface DTO / codec | 43 / 1,806 / 63,065 | E3逐文件检查manifest summary、project name/rel path、session lock、mailbox/focus/recent/token/schema；19个selected tests |
| Editor direct consumers | 29 / 3,725 / 135,662 | E3追踪SessionGuard、Hub link、project probe/recent、activation/startup/retained host；22个selected tests |
| Hub / App direct consumers | 16 / 3,558 / 122,423 | E3追踪spawn、active probe、focus publish、handshake wait、recent store、CLI；40个selected tests |
| Runtime full manifest | 6 / 309 / 10,799 | E3对照完整manifest authority、migration/load/save/validation |
| 去重冻结合计 | 94 / 9,398 / 331,949 | 81个selected Rust `#[test]`、0 ignored；SHA-256 `88fd24544ca3fc9029ed258d0e14166c9a9f668f08057914960b239068a806cc` |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics参考 | 18 / 12,678 / 447,122 | E2/E3按descriptor compatibility/conversion/recovery、cancelable lock、child manager、compositional Ready与package/asset migration路由；SHA-256 `10e80eb62d797b0090c412edb50992c443c2bf2bf9d108e6c6b02bc3198e2923` |

协议已有future manifest拒绝、canonical UUID token、strict marker、OS lease、原子publish和Editor terminal outcome等可保留底座；但summary partial probe、lease Active、mailbox Ready、focus publish和recent write没有共同的ProjectId/BuildSet/OperationId/AdmissionEpoch/SessionGeneration，消费方会把“局部文件成立”解释为“项目可打开或Editor可交互”。

报告建立data-only compatibility probe、phase/generation-qualified admission record、自绑定LaunchRequest/StartupReceipt、有sequence/ack的Focus协议及不阻塞project open的RecentProjectOperation投影，登记0项新P0、56项P1、14项P2和36项资格门；Editor51五项项目生命周期P0、Interface02 identity/schema、Hub01 child owner与Editor02 heartbeat/recovery保持原owner。本轮只写review与重构计划，未修改production/tests，也未运行Cargo、真实双进程、kill/PID reuse、GUI first-present或性能捕获。详见`zircon_runtime_interface/06-project-manifest-session-lock-hub-protocol-recent-project-cross-process-contract-product-integration-review.md`。

## 177. Runtime Interface Contract Certification / ABI Layout / Version Skew / Cross-Language / Fuzz 测试物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Interface 中央契约测试 | 35 / 13,563 / 499,228 | E3逐文件分类234项test、2项should-panic、0 ignored，核对layout、serde、source-shape与行为断言 |
| Interface全部含测试源码 | 72 / 21,831 / 766,910 | 401项test、1 ignored；覆盖中央、inline/domain与1个package-root parser integration文件 |
| producer/host/consumer test | 9 / 3,737 / 132,663 | 95项test、1 ignored；核对Runtime linked API、Host foreign output、App loader/session、Editor gateway/real-ABI |
| Zircon冻结合计 | 81 / 25,568 / 899,573 | 496项selected test、2 ignored；fingerprint `eea8fdb2c7e7d9042f381d8c8995f6d53675503137b6fc7f9937b9bc3a093f1e` |
| Unreal、Godot、Bevy、Fyrox、Unity Graphics参考 | 18 / 17,485 / 613,618 | E2/E3核对BuildVersion/ModuleManifest、生成式extension ABI、历史C compatibility、compile-fail、Rust dylib边界与package API/serialization validation；SHA-256 `5028e6d929c5a0199c22039386b6cda8458e66be18488d087ecce14b5d2d33e5` |

Interface已有V7本机布局、null/misalignment拒绝、opaque allocation释放、malformed/migration serialization和多类DTO行为测试；但required资格没有构建并正向加载真实Runtime DLL，没有生成C/C++ header与独立consumer，没有BuildSet/version-skew矩阵、历史golden corpus、全公开contract覆盖映射、property/fuzz或fault-isolated ABI lane。现有唯一package-root integration文件只验证UI binding parser，不能替代二进制或跨语言认证。

报告登记1项P0、48项P1、12项P2与32项资格门，并建立InterfaceSpec生成Rust/C carrier、header/schema/layout/symbol/test inventory及C1-C5资格层。本轮只写review与重构计划；未修改production/tests，未运行Cargo、真实DLL、C/C++、Miri、sanitizer、fuzz或性能测试；`audit_runtime_structure.py --json`在184.1秒后超时且无输出，不计作证据。详见`zircon_runtime_interface/07-contract-certification-abi-layout-version-skew-cross-language-fuzz-test-architecture-review.md`。

## 178. Hub Application Host / Bootstrap / Window / IPC / Close / Shutdown / Crash Recovery 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Native host与build metadata | 9 / 332 / 9,663 | E3逐文件检查入口、builder、managed state、window event、Tauri config/capability与依赖面 |
| Worker、process与task status | 10 / 4,974 / 186,430 | E3追踪background action、focus refresh、Editor `Child`、handshake与host exit交点；44个selected tests |
| Persistence与shared registry | 3 / 1,347 / 46,754 | E3检查startup load、window/runtime config、atomic writer与跨进程registry边界；12个selected tests |
| React bootstrap、window与IPC | 6 / 637 / 21,536 | E3检查initial load、event subscription、native controls、fallback与unmount cleanup |
| Focused lifecycle contract tests | 4 / 1,698 / 61,135 | E2分类21个source/doc字符串合同；0个Tauri runtime harness、0个真实window/exit test |
| Zircon冻结合计 | 32 / 8,988 / 325,518 | 77个selected test、0 ignored；fingerprint `2c9a6aa3bd69ec86e250abbb046ea7370d080e69fe3b4a02e32fbfa765b416ca` |
| Unreal、Godot、Fyrox、Bevy参考；Unity Graphics不适用 | 12 / 20,983 / 756,598 | E3核对受管进程取消/join、Project Manager close/recovery、有序cleanup、owned Child、typed app exit与TaskPool shutdown；26个reference tests，fingerprint `8a5f99b1689c2f8382984375e79a69fdeff54f9c308f5faf22779470706951cc` |

Hub已经有可启动的Tauri宿主、Rust-owned state、focus refresh、后台action worker、原子config替换和Editor handshake；但应用生命周期仍由偶然drop决定。WebView直接调用`appWindow.close()`，Rust只处理`Focused(true)`，没有`CloseRequested`/`ExitRequested`、stop-admission、close decision、quiesce、checkpoint或terminal receipt；两个`thread::spawn`丢弃`JoinHandle`，启动后的Editor `Child`也不再由host持有。`HubWindowState`只有schema/default/test使用，没有production restore/save。

React initial snapshot与event subscription分开建立，事件没有host/session/window/generation/sequence，listener失败没有retry/resync；native load或schema validation失败还能被`fallbackShellState`吞成可操作shell。报告建立`BootMachine`、`InstanceCoordinator`、`WindowSession`、generation-qualified IPC handshake、`LifecycleCoordinator`、shutdown participant/receipt、crash marker/recovery mode和真实Tauri/OS qualification，登记0项新增P0、40项P1、12项P2与32项资格门。Hub01继续拥有single-instance store、process/Child supervisor，Hub04继续拥有durable operation/effect/shutdown P0，不重复累计。本轮只写review与重构计划，未修改production/tests，未运行Cargo、Tauri真窗口、第二实例、close/crash、DPI/多显示器、OS shutdown或性能测试。详见`zircon_hub/05-application-host-bootstrap-window-ipc-close-shutdown-crash-recovery-review.md`。

## 179. First-Party Asset Importer Source / Dependency / Subasset / Artifact / Sandbox / Product Integration 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Legacy `asset_importers`非shader family | 34 / 2,935 / 109,871 | E3逐文件核对audio/data/model/texture declaration、runtime实现、测试与生成manifest |
| 新glTF、OBJ、Texture、Audio、Opus、UI Document包 | 81 / 14,584 / 539,670 | E3逐文件核对parser、subasset、container、mip/transcode、codec、UI reference与package/runtime/dist声明 |
| Runtime内建importer、registry、ProjectAssetManager、catalog与App caller | 23 / 6,799 / 175,158 | E3追踪builtin/plugin选择、priority、merge、真实产品装配与重复实现边界 |
| Zircon冻结合计 | 138 / 24,318 / 824,699 | 直接插件根115文件；fingerprint `ef93fc480de91549c85b7a999c108dbafb7abd67d8558d14cd3c9b3b6a47fc26` |
| Unreal、Godot、Bevy、Fyrox、Unity Graphics参考 | 25 / 16,172 / 645,781 | E3核对source hash/多源、reader snapshot/dependency/labeled asset、平台variant、scene/audio/texture import、external resource与ScriptedImporter subobject；fingerprint `8875a71310992fca2c2e89e74ae931c3ff441f0ba98293a49c573c9fcfa82e6d` |

Texture container已有DDS/KTX1/KTX2/ASTC header、range、alignment、metadata与部分supercompression负向校验，Runtime内建glTF也能生成Texture、Material、Mesh、Scene、AnimationClip、AnimationSkeleton、Skin与inverse-bind subasset；registry具有generation、重复matcher拒绝与确定性排序。这些是本轮确认应保留的工程底座。

但当前tracked Texture mip kernel存在参数错位的源码级编译阻断；first-party catalog链接的stable glTF v1/priority120会遮蔽更完整的内建v2/priority10，并把animation降为`DataAsset` placeholder。glTF/OBJ geometry缺少统一index admission，plugin glTF重新读主路径与任意relative URI且无immutable snapshot、canonical containment或source dependency graph，KTX2解压上限又直接由输入声明的expected length决定。OBJ丢弃MTL/material，Texture尚无完整target cook matrix，长音频全量解码驻留，UI reference不进入dependency，legacy/new provider仍双轨。

报告登记5项P0、72项P1、16项P2与20项资格门，建立`ImportRequest`、`SourceBroker`、`ImportProduct`、`DerivedArtifactRecipe`和`ImporterQualificationReceipt`，并要求按M0-M5关闭compile、安全、唯一provider、stable subasset、platform artifact、legacy cutover及conformance/fuzz/determinism/product E2E。本轮只写review与重构计划；未修改production/tests，未运行Cargo、真实Editor导入、cook/load/render/playback或性能测试。详见`zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md`。

## 180. First-Party Editor Authoring Extension / Document / Operation / Toolkit / Runtime Contract 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Material Editor、Animation Graph、Timeline Sequence | 30 / 2,922 / 110,492 | E3逐文件核对graph/timeline asset、validator/compiler/helper、operation descriptor、source/native registration与测试 |
| UI Asset Authoring、Runtime Diagnostics | 18 / 960 / 36,227 | E3核对内建authority冲突、伪create命令、缺失document与diagnostic data provider |
| Prefab Tools、Tilemap 2D、Terrain、Texture | 64 / 3,289 / 119,863 | E3核对editor/runtime/dist、component/importer/helper、catalog与产品consumer断点 |
| 九个package合计 | 112 / 7,171 / 266,582 | 77个test、0 ignored/should_panic；fingerprint `3bc7523221ac52beba9f102f581d38cedf6e357e5d7d93c3b1103400f1a7478e` |
| 共享catalog/host/caller | 30 / 8,346 / 318,723 | E3追踪editor_support、first-party catalog、App组合、command/extension/native registration与document resolver；fingerprint `23e0189dec161ab7ca0b18a8097dd6f250a201f82950058f94cb7d7f3a286af2` |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics参考 | 19 / 13,115 / 476,470 | E3核对AssetEditorToolkit、EditorPlugin、可逆commands、executable plugin/loader与ShaderGraph真实artifact；fingerprint `7b378fa7ded4fea7a9ec7ce010db950ba6c3997bbda83c8a78eb05bfb810adf1` |

九包共有42个command descriptor：9个surface open、30个没有event/factory的领域操作、3个只发送OpenView的UI“创建”命令；没有package注册operation factory。20条`plugins://` URI对应资源全部缺失，source registration又不绑定plugin root。first-party editor catalog对九包0链接，runtime catalog对Prefab/Tilemap/Terrain/Texture只链接Texture；native serialized materializer也无法表达template/toolkit/factory/compiler/preview/document provider。

Material compiler只处理base color，Animation Graph compiler只返回output source字符串，Timeline move失败可残留原地mutation；Prefab apply/revert/break、Tilemap paint、Terrain import plan与Texture尺寸摘要均未进入document transaction、artifact或Runtime consumer。UI Asset和Runtime Diagnostics还与内建view/command authority冲突，而isolated package registration test在空registry中看不到这些失败。

报告登记0项新增P0、72项P1、18项P2与32项资格门；硬阻塞继续由Plugins01/06、Editor14/15/16/23/25/34/35/44/45/50及Runtime29/39/42拥有。本文建立`EditorExtensionPackageContract`、`ExtensionMountReceipt`、`ExecutableOperationBinding`、`AuthoringDocumentProvider`及source/native parity路线。本轮只写review与重构计划；未修改production/tests，未运行Cargo、真实Editor、NativeDynamic、save/cook/play或性能测试。详见`zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md`。

## 181. First-Party Particle / VFX Source / Runtime / Editor / Dist / Catalog 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `zircon_plugins/particles`全包 | 57 / 8,733 / 306,329 | E3逐文件核对manifest、asset/component/service、CPU simulation、GPU planner/backend/owner/shader、render feature/executor、editor/dist与资源 |
| package tests | 41 | runtime 38、editor 1、dist 2；0 ignored；主要覆盖局部算法、metadata与可选adapter GPU路径 |
| package fingerprint | `04a0024e1d515eb2e721f8ddd7f8f717c6283c15a96f76383912933c3434ffb6` | path不区分大小写排序，按`path|file_sha256`的LF串重算 |
| 产品装配与caller | runtime 1 / editor 0 | first-party runtime catalog链接provider，editor catalog不链接；无普通world scheduler/component lifecycle caller |
| Unreal、Fyrox、Godot、Unity Graphics、Bevy参考 | 20个关键文件 | E3核对Niagara world/scalability/GPU/renderer、Scene原生粒子、fixed FPS/trail、compiled VFX data及RenderWorld资产生命周期 |

包内已有CPU SoA/free list、局部确定性RNG、shape/burst/curve/sprite extract，以及真实WGSL、双buffer compute、compact、indirect draw和异步readback底座；manifest也诚实标为experimental/partial/default-off。纵向产品链仍断裂：runtime catalog已链接而editor catalog未链接，12个领域operation无event/factory且菜单全disabled，三个ZUI只有`Space`占位，CPU sprite TOML没有create/import/save consumer；NativeDynamic dist只返回metadata，无法投影source manager/render/editor行为。

运行时又存在三条事实源：manager对GPU资产推进CPU fallback，renderer owner独立推进GPU aggregate，Scene/script则以dynamic JSON直接写最终sprites。全仓生产搜索没有普通scheduler调用`ParticlesManager::tick`、typed component instantiate/remove或GPU feedback回写；三个Render Graph compute executor只校验metadata，真实compute在runtime prepare中提前录制。GPU aggregate还会因实例增删/暂停/asset变化重建并重置其他系统，并按顺序分配固定slot上限；Runtime103已确认每emitter dt独立编码，撤回旧全局max-dt污染判断。CPU/GPU字段、material/texture与history语义仍不等价。

报告登记0项新增P0、48项P1、12项P2与32项资格门；Runtime26、Runtime05/09A/09B/09C、Editor15/50及Plugins01/06继续拥有最高优先级问题。本篇只拥有单包manifest/source/editor/runtime/dist/catalog纵向交付合同。本轮只写review与重构计划，未修改production/tests，未运行Cargo、GPU、Editor、NativeDynamic、save/reopen或产品场景。详见`zircon_plugins/09-first-party-particle-vfx-source-runtime-editor-dist-catalog-simulation-render-product-integration-review.md`。

## 182. First-Party Network Source / Runtime / Editor / Dist / Catalog / Transport / RPC / Replication 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `zircon_plugins/net`全包 | 186 / 14,377 / 495,870 | E3逐文件核对manifest、base runtime、HTTP/WS/RUDP/RPC/Replication/Download六feature、Editor、dist及全部测试 |
| production / test files | 127 / 59 | 120项test attribute、0 ignored；主要覆盖单进程loopback、局部算法、锁中毒/回调重入和descriptor |
| package fingerprint | `85a58658dd3afeac16dad9fea085acaeaa47be480aa32aa42e69caa075a4f777` | tracked path排序，按小写`path|file_sha256`的LF串、无末尾LF重算 |
| 产品装配 | root runtime 1 / feature 0 / editor 0 | 普通first-party source catalog只链接root；generated export存在独立feature registration路径 |
| Editor / Native | 0/5资源、0/6 operation可执行、dist metadata-only | 缺resource、factory、document/compiler和source/native behavior parity |
| Unreal、Godot、Bevy、Fyrox、Unity Graphics参考 | 15个关键文件 | E3核对world/connection/replication、PIE topology、installer、SceneMultiplayer/WS/HTTP、remote transport分层及参考缺失边界 |

包内已有真实TCP/UDP socket、HTTP client/server、WebSocket client/server和可保留的RUDP/RPC/Replication/Download局部算法；poison recovery与回调重入测试也覆盖了一批真实失败边界。纵向产品链仍断裂：普通runtime catalog只装配无HTTP/WS backend的root manager，六个feature provider均未进入产品；HTTP/WS factory创建私有manager，RUDP/RPC/Replication再创建各自内存authority，Content Download生产factory解析canonical manager却拿不到私有HTTP backend，测试则用`cfg(test)`注入绕过。

Net Editor不在first-party editor catalog，4个ZUI与1个TOML资源不存在，6个operation无factory，toolkit/graph没有document/save/undo/compiler/runtime artifact；NativeDynamic dist只返回descriptor/registration metadata。base runtime又是process级同步manager，使用串行worker和双Tokio runtime；ingress/event/backpressure、TLS/WSS、RUDP wire、认证handshake、RPC transport、World replication、disk installer及产品观测均未闭合。

报告登记0项新增P0、48项P1、12项P2与32项资格门；Runtime08E、Editor26/07/25及Plugins01/06继续拥有最高优先级问题。本篇只拥有Net单包manifest/source/feature/editor/dist/catalog纵向交付合同。本轮只写review与重构计划，未修改production/tests，未运行Cargo、真实网络、TLS、Editor、NativeDynamic、多客户端、soak或性能测试。详见`zircon_plugins/10-first-party-network-source-runtime-editor-dist-catalog-transport-rpc-replication-product-integration-review.md`。

## 183. First-Party Sound Source / Runtime / Editor / Dist / Catalog / Mixer / Spatial / Reverb / Timeline 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| zircon_plugins/sound 全包 | 1,307 / 23,903 / 907,000 | E3逐文件核对plugin.toml、base runtime/editor/dist、ray/timeline两个feature及全部测试路径 |
| production / exact test-path files | 269 / 1,038 | 362项test attribute；大量structure/manifest碎片与局部Mock/Kira行为测试并存 |
| package fingerprint | 0a1d327265a1b5c134aa52f4f5b7e3b9a12bbb79adbf14e5f38b1f4ba86b624b | tracked path排序，按小写path与file SHA-256的LF串、无末尾LF重算 |
| 产品装配 | ordinary runtime 0 / editor 0；catalog root 1 / optional 0 | 默认App与Editor Host不链接Sound；generated export存在独立feature路径 |
| Editor / Native / features | 0/33 operation可执行、五份ZUI主要为Space、dist/feature metadata-only | 缺document/factory/audition/telemetry、真实feature provider和包装形态parity |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics参考 | 30个关键文件 | E3核对audio/render thread、device recovery、mixer/submix/effect、streaming/spatial、Editor产品和package边界 |

包内已有真实Kira 0.12.2 playback、typed source/listener/volume/mixer/effect/automation/timeline/acoustic合同、图校验、动态事件ABI与局部空间/声学算法。产品链仍断裂：普通App与Editor Host都不会安装Sound provider，runtime catalog只链接root且不链接ray/timeline feature，editor catalog对Sound为0；manager factory固定使用默认配置，dist和两个optional feature又只发布descriptor/capability。

真实Kira render path只闭合有限PCM播放，source_environment的attenuation、doppler、volume、occlusion、HRTF与convolution没有生产调用者，engine/dsp和filter只在测试配置下编译。graph admission接受effect/send后compiler拒绝，music_sfx与spatial_room preset因此不可应用；Kira send routing仍有3项已登记红门。Editor的33个command没有factory，五份surface主要为Space，live controller只有测试fake。

报告登记0项新增P0、48项P1、12项P2与32项资格门；Runtime08B、Editor17、Plugins01/06/07及既有Kira failure继续拥有最高优先级问题。本篇只拥有Sound单包manifest/source/feature/editor/dist/catalog纵向交付合同。本轮只写review与重构计划，未修改production/tests，未运行Cargo、真实声卡、Editor、NativeDynamic、soak或性能测试。详见zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md。

## 184. First-Party Physics Source / Runtime / Editor / Dist / Catalog / Simulation / Collision / Joint / Ragdoll 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| zircon_plugins/physics 全包 | 93 / 13,207 / 462,852 | E3逐文件核对plugin.toml、runtime/editor/dist、builtin/Jolt、manager、constraint、skeletal和全部测试路径 |
| runtime / editor / dist | 78 / 12,383 / 433,244；12 / 635 / 22,371；2 / 115 / 4,298 | 82项test attribute；Jolt测试、registration/DTO与descriptor smoke并存 |
| package fingerprint | ebcef0796246eb827b0d5f7a9f2bcfc2cbabfe73bd992ad93109ae7fb04015c7 | tracked path排序，按小写path与file SHA-256的LF串、无末尾LF重算 |
| 产品装配 | ordinary runtime 0 / editor 0；generated source 1 / native behavior 0 | 普通App/catalog不链接Physics；export默认Disabled；dist为stateless metadata shell |
| Solver / Query / Event / Constraint | builtin approximation；Jolt body partial | Jolt query空实现、两层filter、近似pair event、Rust projection constraint、无collision cook |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics参考 | 20个关键文件/目录 | E3核对solver/query/asset/editor/test、service/direct state、persistent world/dirty sync、fixed clock与参考缺失边界 |

包内已有中立Physics合同、generation handle、命令预算、world replacement epoch、builtin参考后端、可选Jolt body lifecycle、scene sync和ragdoll profile基础。纵向产品链仍断裂：普通App、first-party runtime catalog和editor catalog都不链接Physics；generated source export显式链接registration却没有`backend-jolt`，NativeDynamic dist则只有registration metadata且明确把world/query留给source runtime。

显式启用Jolt也没有形成完整solver事实。产品fixed system每次调用只推进一步scheduler delta，公开manager accumulator是另一套未接入时钟；每tick扫描全Scene并深拷贝同步状态。Jolt native filter只有moving/non-moving，query trait为空，contact/trigger重走builtin近似pair scan，constraint不创建native对象而是在native step后做Rust投影。builtin没有collision impulse、friction、stack、island、sleep或CCD，却可报告Ready。

Physics Editor四份ZUI以11个业务Space占位，command没有domain factory/controller；overlay只生成DTO，toggle只open view，开放provider failure仍未关闭；ragdoll create同样不生成或保存资产，Workbench还保留固定演示binding。报告登记0项新增P0、48项P1、12项P2与32项资格门；Runtime08A、Editor18、Plugins01/06及开放overlay failure继续拥有最高优先级问题。本篇只拥有Physics单包manifest/source/editor/dist/catalog纵向交付合同。本轮只写review与重构计划，未修改production/tests，未运行Cargo、Jolt、App、Editor、NativeDynamic、soak或性能测试。详见`zircon_plugins/12-first-party-physics-source-runtime-editor-dist-catalog-simulation-collision-joint-ragdoll-product-integration-review.md`。

## 185. First-Party Animation Source / Runtime / Editor / Dist / Catalog / Skeleton / Clip / Pose / Graph / State Machine / IK / Skinning 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `zircon_plugins/animation`全包 | 170 / 18,172 / 634,036 | E3逐文件核对plugin.toml、runtime/editor/dist、manager/evaluation/state-machine/IK/skinning和全部测试路径 |
| runtime / editor / dist | 161 / 17,794 / 620,461；6 / 208 / 7,507；2 / 115 / 4,123 | 38个test-bearing path、129项test attribute；算法、registration与descriptor smoke并存 |
| package fingerprint | `9fb8c3491df494af7b883afef0c2836a4910a595ef2ee60a91566a24b7877f34` | tracked path排序，按小写path、空格与file SHA-256的LF串、无末尾LF重算 |
| core fallback / authoring依赖 | Runtime fallback 17 / 2,202 / 76,471；Graph 10 / 1,015 / 38,720；Timeline 10 / 864 / 32,492 | 两套同名module/manager互斥装配；Graph/Timeline与主Editor共8份ZUI缺失 |
| 产品装配 | ordinary Client/Editor为core fallback；显式source plugin可替代；native behavior 0 | editor catalog不链接Animation；dist为stateless metadata shell |
| Import / Evaluate / Deform | glTF Animation placeholder；compiled与legacy并存；GPU palette无consumer | generic skin Data、同步load、字符串pose/physics bridge、无renderer deformation闭环 |
| Unreal、Bevy、Fyrox、Godot、Unity Graphics参考 | 17个关键文件/目录 | E3核对artifact/parallel/editor、stable target/graph、pose/ABSM、phase/transaction/render skeleton与GPU product boundary |

包内已有compiled clip evaluator、strict key validation、target table、PosePool、graph/state-machine program、layer/mask、blend space、two-bone/look-at IK、事件背压、ECS QueryState projection和replacement epoch基础。纵向产品链仍断裂：普通Client/Editor Host不链接plugin provider而运行core fallback；两套`animation.runtime` manager/module可独立漂移，插件内部legacy sampler与compiled evaluator又没有硬切为唯一owner。

首方glTF importer仍把每条animation产出为明确的Data placeholder，skin与inverse bind也没有typed Animation artifact。帧循环同步load多类资产、克隆参数与字符串骨骼名；direct worker忽略schedule失败后阻塞等待，失败可panic。pose在world-transform之后按bone name写回Scene，physics再收到字符串全pose，Animation GPU palette没有renderer consumer；CPU matrix test不能证明GPU skinning。

Animation Editor四份ZUI、Animation Graph三份ZUI和Timeline一份ZUI均不存在，editor catalog没有Animation；Graph/Timeline operation只注册descriptor，没有产品handler/compiler。NativeDynamic dist则声明evaluation留在source runtime，并以stateless、空command/event、无state/lifecycle/bridge导出metadata。报告登记0项新增P0、48项P1、12项P2与32项资格门；Runtime08C、Editor14、Plugins01/06/07/08、Runtime42与三份开放failure继续拥有最高优先级问题。本轮只写review与重构计划，未修改production/tests，未运行Cargo、App、Editor、GPU、NativeDynamic、soak或性能测试。详见`zircon_plugins/13-first-party-animation-source-runtime-editor-dist-catalog-skeleton-clip-pose-graph-state-machine-ik-skinning-product-integration-review.md`。

## 186. First-Party Navigation Source / Native / Runtime / Editor / Dist / Catalog / Recast / Detour / Crowd / TileCache / Query / Bake 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| 三个canonical root合计 | 217 / 52,108 / 1,671,981 | E3逐文件核对Navigation plugin、core fallback、framework、native、editor、dist及全部测试路径 |
| Zircon-owned / vendored Recast | 166 / 24,480 / 834,631；51 / 27,628 / 837,350 | vendor只含Recast/Detour/Crowd/TileCache，无RecastDemo；算法存在与产品集成成熟度分开核算 |
| package fingerprint | `3f9544fc4771cb9da09706c2575ddf55f465c3fbb4d533baa53271b3f038ba07` | 217个tracked path排序，按小写path、空格与file SHA-256的LF串、无末尾LF重算 |
| test inventory | 169项test attribute | plugin runtime 66、native 37、editor 29、core fallback 26、framework 9、dist 2；无property/fuzz/benchmark/soak入口 |
| 产品装配 | Client builtin fallback；Editor source Recast；Server无Navigation；NativeDynamic behavior 0 | 同project不同target没有唯一provider/backend/artifact receipt |
| Asset / Query / TileCache | raw triangle DTO/TOML；per-query重建`dtNavMesh`；单个硬编码cache layer | 无prepared Detour tile artifact、query pool、streaming chunk、provenance或platform cook |
| Bake / Agent / Editor | 假几何、settings未应用、全World JSON扫描、crowd组合fallback、operation拒绝Bake、11份ZUI占位 | 五份开放failure保持open；本轮不以source drift静态关闭 |
| Recast、Unreal、Godot、Fyrox参考 | 18个关键文件/目录 | E3核对算法边界、dirty/tile lifecycle、server iteration、Editor command；Bevy/Unity Graphics为负向适用边界 |

Navigation包内已有真实Recast、Detour、DetourCrowd、DetourTileCache bridge、typed组件/查询、off-mesh traversal、dirty tile plan、overlay frame与局部generation。纵向产品链仍分裂：ordinary Client安装core Rust fallback，Editor Host才source-link Recast provider，Server未启用Navigation；NativeDynamic只导出metadata。两套同名`navigation.runtime`实现没有共同provider、artifact schema、backend generation或qualification receipt。

当前`NavMeshAsset`保存raw vertices/indices/polygon DTO而不是Detour tile blob；每次find/sample/raycast会重建并销毁native navmesh/query，TileCache又把全asset变成一个`0/0/0`layer并硬编码walkability与4 tile上限。Bake不读取真实render mesh，collider被简化为顶面/圆盘/AABB，TriangleMesh与HeightField跳过，空输入生成synthetic quad；agent/profile settings多数只进入hash/warning，`output_asset`没有writer或原子发布。

Runtime每帧全World扫描和动态JSON解析；任意obstacle/off-mesh link会清空全部crowd转legacy逐agent寻路和O(A² + A×O) avoidance，再直接写Transform。Editor虽有BakePanel模型和11份ZUI，product operation却明确拒绝Bake，多数ZUI为业务`Space`，overlay复制全量三角形。报告登记0项新增P0、48项P1、12项P2与32项资格门；Runtime08D、Editor19、Plugins01/06/07、Runtime42及五份开放failure继续拥有最高优先级问题。本轮只写review与重构计划，未修改production/tests，未运行Cargo、C++、App、Editor、NativeDynamic、soak或性能测试。详见`zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md`。

## 187. First-Party AI Source / Runtime / Editor / Dist / Catalog / Behavior Tree / Blackboard / Perception / EQS 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| canonical AI | 86 / 17,667 / 609,467 | E3逐文件核对plugin manifest、runtime、editor、dist、framework/bridge及全部局部测试路径 |
| plugin runtime / editor / dist | 64 / 14,750 / 512,057；10 / 1,654 / 58,169；2 / 115 / 4,087 | 109项canonical test attribute；runtime 98、editor 9、dist 2 |
| selected product vertical | 113 / 23,528 / 824,401 | 加入App target、runtime/editor catalog、builtin rows、Editor产品资源、Vampire asset/scene与gameplay test，共137项test attribute |
| package fingerprint | `ecba84bc9e896e4c3633cd3b35e4d1780767e8b3c0c0c2b5fc0a3e05724d6f1b` | 113个selected tracked path排序，按小写path、空格与file SHA-256的LF串、无末尾LF重算 |
| 产品装配 | Client / Editor / Server默认provider均未闭合；NativeDynamic behavior 0 | runtime catalog仅在显式feature下route AI，editor catalog无AI；dist为stateless metadata shell |
| Asset / Scene | generic Data TOML schema漂移；scene字符串；脚本镜像 | 无AI importer/compiler/artifact/production loader/manager registration产品链 |
| Runtime / Editor | dense tree与typed Blackboard可保留；global manager、递归无预算、全扫描、无EQS；ZUI无产品provider | 5项Editor P0继续由Editor20拥有，本篇0新增P0 |
| Unreal、Fyrox、Bevy、Godot、Unity Graphics参考 | 17个关键文件/目录 | Unreal主参考行为树/Blackboard/Perception/EQS/Editor；Fyrox为紧凑树基线；其余为负向适用边界 |

AI包内已有dense compiled tree、标准节点目录、typed Blackboard、observer、owner-aware registration/revocation、bounded hearing ingress、typed perception组件、runtime mirror与局部debug基础。纵向产品链仍断裂：普通Client不启用首方runtime plugin，Editor Host既不链接AI runtime/editor也没有editor catalog provider，Server同样没有AI provider；manifest声明的target与effective capability不一致。NativeDynamic只有descriptor和registration metadata，明确让runtime behavior留在embedded source module。

Vampire唯一作者化树由generic TOML importer标为Data，字段与当前descriptor不兼容；场景只写`behavior_tree`字符串，README说明脚本镜像决策，仓内没有production caller把asset编译、注册并绑定agent。manager仍是全局`Arc<Mutex<_>>`，没有Brain/Agent/Tree/Blackboard scene lifecycle；compiler/executor递归且无深度、节点、时间和bytes预算，每tick又无条件构造全量debug snapshot。Perception全World扫描并按receiver×source计算，缺Physics时Sight刷新为可见；EQS没有生产类型、service或scheduler。

报告登记0项新增P0、48项P1、12项P2与32项资格门；Runtime08F、Editor20、Plugins01/06/07、Runtime42等canonical owner继续拥有最高优先级问题。本篇只拥有AI单包从manifest、source/runtime/editor/dist/catalog、App target到示例asset的纵向交付合同。本轮只写review与重构计划，未修改production/tests，未运行Cargo、App、Editor、NativeDynamic、soak或性能测试。详见`zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md`。

## 188. First-Party Zr VM Language Source / Runtime / Dist / Catalog / Reflection Callsite / Host Interface / GC / Hot Reload 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `zircon_plugins/zr_vm_language` | 40 / 4,573 / 163,822 | E3逐文件核对manifest、runtime、real backend、callsite、reflection、host interface、dist与41项test attribute |
| Runtime VM | 101 / 18,834 / 650,843 | 中立backend、host、package、reflection、GC、manager、reload、scene systems与155项test attribute |
| App/catalog/runtime integration | 24 / 5,676 / 207,726 | feature、runtime/editor catalog、builtin rows、dynamic session和11个ignore |
| docs与Vampire | 21 / 1,923 / 267,926 | 14份VM文档、6份文本产品输入、1份`.zro`；Vampire 10个owner test仍ignore |
| selected product vertical | 186 / 31,006 / 1,290,317 | 185个文本、1个binary artifact、305项test attribute；fingerprint `ad5cf7653809c0fcebca7298d73f7460da8d3530eba7f6fa2ee7d8666aa5d7e5` |
| external ZrVM | commit `8a843bdd...c084c8` / 54项working-tree变化 | Runtime21拥有语言/compiler/bytecode语义；本篇只记录插件消费边界，实施前必须clean recheck |
| 产品装配 | Client/Editor默认provider缺失；dist无真实backend | manifest target/carrier与effective capability不一致，缺失到load时才暴露 |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics参考 | 17个关键文件/目录 | 线程/GC/bytecode/debug、ScriptLanguage、stable type identity；Unity Graphics为负向适用边界 |

Zr VM插件已有具体backend owner、预编译dense callsite、catalog generation guard、capability-gated extension、显式native drop order、GC时间预算和hot-reload state migration底座。但真实backend依赖外部dirty源码与本机DLL，Runtime装载时同步编译，所有package/world共享进程mutex；值边界仅覆盖标量/string/bytes/handle，Array统一按bytes逐元素转换，state和reflection仍走JSON。NativeDynamic只有registration metadata，没有invoke/state/bridge/lifecycle执行能力。

四份父计划failure继续open：确定性bulk/cross-platform、ScriptCallTable hard-cut、VM hotpath和Vampire测试owner。报告登记0项新增P0、48项P1、12项P2与32项资格门；Runtime07/21、Editor31、Plugins01/06、Interface04/05/07和App06继续拥有最高优先级问题。本轮只写review与重构计划，未修改production/tests，未运行Cargo、真实ZrVM、App、Editor、NativeDynamic、soak或性能测试。详见`zircon_plugins/16-first-party-zr-vm-language-source-runtime-dist-catalog-reflection-callsite-host-interface-gc-hot-reload-product-integration-review.md`。

## 189. First-Party Virtual Geometry Source / Runtime / Editor / Dist / Catalog / Asset Cook / Cluster / Page Streaming / Culling / Raster 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `zircon_plugins/virtual_geometry` | 237 / 37,539 / 1,390,191 | E3逐路径核对manifest、runtime、editor、dist、两份WGSL、test sources与214项test attribute |
| runtime production | 198 / 14,810 / 539,741 | typed residency/slot/hierarchy/cluster、CPU traversal、GPU resources/readback、pass DTO；五个公共executor均为validation-only |
| runtime tests | 30 / 22,391 / 838,478 | 5份已接线test source与其他tests；9份15,289行/88 tests被promotion guard明确保持未接线 |
| editor / dist | 6 / 154 / 5,303；2 / 116 / 4,224 | editor catalog无VG、`authoring.zui`缺失；NativeDynamic无invoke/state/bridge/lifecycle |
| package fingerprint | `9cecec2bfccd27e76d3d0a66f1314532cb9326e1e3f5294c95db60fba89c9760` | 237个tracked path排序，按小写path、空格与file SHA-256的LF串、无末尾LF重算 |
| Product path | 默认profile/viewport/PBR viewer关闭；ordinary indexed mesh + payload substitution | `joints.x/y`偷渡vertex ordinal，payload缺失时静默回退原mesh |
| Cook / streaming | `ZVG0`只存hierarchy与triangle range | 必须回读完整原mesh；无自包含压缩page、async store、byte/VRAM budget或几何byte uploader |
| Unreal、Bevy、Unity Graphics、Godot、Fyrox参考 | 22个关键文件/目录 | Nanite与Bevy meshlet为正向产品链；其余限定为GPU-resident/ordinary LOD负向适用边界 |

Virtual Geometry包内部代码规模较大，但产品链仍断裂。public prepare/cull/feedback/visbuffer/debug executor只验证context；私有`VirtualGeometryGpuResources`没有production构造点；所谓hardware raster和visbuffer只写CPU records，WGSL只复制seed/更新page table。Cook artifact不含可独立解码几何，runtime按triangle range从原mesh展开顶点，真实产品仍是ordinary indexed mesh加实验性storage attribute substitution。

报告登记0项新增P0、48项P1、12项P2与32项资格门；Runtime09B/09C/09D、Runtime04、Editor22/32、Plugins01/04/06继续拥有最高优先级问题。两份Plugins13 failure与两份Runtime04 failure保持open。本轮只写review与重构计划，未修改production/tests，未运行Cargo、GPU、Editor、NativeDynamic、跨平台、soak或性能测试。详见`zircon_plugins/17-first-party-virtual-geometry-source-runtime-editor-dist-catalog-asset-cook-cluster-page-streaming-culling-raster-product-integration-review.md`。

## 190. Runtime Picking / Pointer / Ray / Hit / Hover / Drag-Drop / Event / Backend 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `core/framework/picking` production | 23 / 2,069 / 61,154 | 逐文件读取ray、hit、backend、hover、event、pipeline与pointer location；无独立test owner |
| `src/tests/picking` | 6 / 889 / 29,809 | 22项test、0 ignored；逐文件读取，未执行Cargo |
| focused total | 29 / 2,958 / 90,963 | fingerprint `377c7f27b41bf31a6d7c061ac3feee9ab793c492d6468dbf5b65ba110252b1c3` |
| Editor产品调用链 | viewport pointer dispatch、picking resolver、scene picking state与测试 | 完整pipeline/event/backend无production caller；Editor只消费手工`PointerHits`的route/debug，`runtime_input`无消费者 |
| Unreal、Bevy、Godot、Fyrox、Unity Graphics参考 | 22个关键文件/目录 | 对照renderer hit proxy、normalized pointer target、多viewport/capture、physics query、Editor picking pass与non-jittered matrix |

Runtime Picking已经有RayMap、确定性hit sort、hover diff与局部事件状态机底座，但同一pointer跨viewport会在hover/event层合并，fixed `HitTarget`与raw `u64`没有generation/owner资格，backend仍是同步、不可失败、ray-only合同。ray构建忽略projection override与viewport/DPI合同；事件层缺capture、阈值、click时间/次数、target失效和分层执行，disabled/retirement/failure语义也会静默清空或不可区分。

报告登记0项新增P0、48项P1、12项P2与36项资格门；Editor03、Runtime12/23/24、Render04/09A与PERF-MVP-332继续拥有既有父问题。本篇只拥有Picking纵向composition、qualified frame/view identity、backend protocol、interaction state与产品资格合同。本轮只写review与重构计划，未修改production/tests，未运行Cargo、Editor、GPU、soak或性能测试。详见`zircon_runtime/47-runtime-picking-pointer-ray-hit-hover-drag-drop-event-backend-product-integration-review.md`。

## 191. Runtime-wide State / NextState / Transition / Hook / History / Schedule / Scope 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `core/framework/state` production | 12 / 519 / 14,817 | 逐文件读取typed state、next state、machine、registry、hook index、dispatch与event；0项内嵌test |
| runtime facade / owner wiring | 5 / 861 / 30,031 | 完整读取CoreHandle/CoreRuntime facade、CoreRuntimeInner owner、core/prelude re-export；3项内嵌test |
| dedicated state tests | 2 / 310 / 10,143 | 9项test、0 ignored；完整读取，未执行Cargo |
| focused total | 19 / 1,690 / 54,991 | 12项test、0 ignored；fingerprint `70a97bb241ae70381c39ab1e098e9cc786abe80af45fe404f473a8dcb7ac6b15` |
| 产品与文档调用链 | Runtime/App prelude、poison guard、module doc、App/Editor/Plugin/Hub/Interface/examples全仓搜索 | 非测试production caller为0；current dedicated test与module doc均要求已不存在的旧direct-match实现 |
| Bevy、Unreal、Fyrox、Godot、Unity Graphics参考 | 19个关键文件/目录 | 对照application state schedule/message/scope、StateTree request/event/instance、animation graph lifecycle；Unity Graphics作为无通用state authority的负向边界 |

Runtime-wide State已经有typed current/next、identity suppression、按state/state-pair哈希桶定位hook、确定exit/transition/enter顺序和锁外dispatch底座，但它仍是手工驱动的进程内callback registry。产品caller为0，Runtime03九阶段表没有state apply point；单`NextState`静默last-writer-wins，request无producer/priority/sequence/receipt，`insert_state`覆盖时伪造初始化，history永久增长且查询全量clone，hook无token/owner generation/quiescence，poison又被静默视为健康。`TypeId + Any + type_name`也不能承担持久、脚本、DLL或热重载身份。

报告登记0项新增P0、48项P1、12项P2与36项资格门；Runtime01/02/03/05/07/22/24/38/41/46、Tooling35与PERF-MVP-320继续拥有父问题。Runtime48只拥有通用state service的scope/descriptor、request admission/resolver、transition receipt、schedule接线、bounded journal、hook lifecycle与产品资格；不把application state扩张成Animation/AI/Gameplay StateTree。本轮只写review与索引，未修改production/tests，未运行Cargo、并发、plugin unload、soak或性能测试。详见`zircon_runtime/48-runtime-wide-state-next-state-transition-hook-history-schedule-scope-product-integration-review.md`。

## 192. First-Party Texture Source / Importer / Runtime / Editor / Dist / Catalog / Image / Cubemap / Array / Volume / Compression / Streaming 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `zircon_plugins/texture` | 16 / 614 / 20,925 | E3逐文件核对stable manifest、summary runtime、editor contribution、metadata-only dist与7项test attribute |
| `zircon_plugins/texture_importer` | 43 / 9,575 / 323,386 | E3逐文件核对六类importer、DDS/KTX/ASTC、mipgen、BC5、dist与172项test attribute |
| 旧 `asset_importers/texture` | 7 / 521 / 18,346 | experimental descriptor-only duplicate；没有FunctionAssetImporter |
| runtime/editor provider catalogs | 10 / 1,489 / 53,302 | runtime只路由`texture`，主importer与Texture editor均无provider dispatch |
| Runtime Texture/builtin ingest | 33 / 7,654 / 264,732 | RGBA8/container asset、typed upload readiness、builtin image decode与52项test attribute |
| App feature/source assertion | 4 / 535 / 19,289 | default target不启用base provider；assertion只证明App不直接fanout |
| selected product vertical | 113 / 20,388 / 699,980 | 257项test attribute；fingerprint `e8885212283d766a9cf350c7a65c49eb7405d298f24acd61c818a96da587d6b9` |
| Unreal、Unity Graphics、Bevy、Fyrox、Godot参考 | 18个关键文件 | 对照DDC/streaming/editor、RenderGraph Texture/atlas、float loader、typed image/import options与测试 |

Texture已有可保留的DDS/KTX1/KTX2/ASTC parser、BC5、typed upload readiness与RGBA32F IBL路径，但当前产品事实断裂。主importer的tracked Kaiser mip代码不可编译；image/PSD和builtin路径把HDR/EXR量化为RGBA8，settings与现有测试允许format/payload矛盾；cube/array通过直接filesystem read绕过source snapshot、VFS、dependency graph与containment。其余compression target、BasisLZ、runtime mip和D3/Volume尚未形成真实platform artifact。

主importer虽在builtin catalog中列为stable，却不在first-party provider catalog依赖/feature/dispatch和Runtime profile中；默认Client/Editor target又不启用提供`texture` shell的base feature。Texture editor只注册缺失的`authoring.zui`，两个dist均不执行业务。报告登记0项新增P0、48项P1、12项P2与32项资格门；Editor35、Plugins07/08/06、Runtime04/09D与Plugins01继续拥有五项最高优先级问题。本轮只写review与重构计划，未修改production/tests，未运行Cargo、GPU、Editor、NativeDynamic、跨平台、soak或性能测试。详见`zircon_plugins/18-first-party-texture-source-importer-runtime-editor-dist-catalog-image-cubemap-array-volume-compression-streaming-product-integration-review.md`。

## 193. First-Party Hybrid GI Source / Runtime / Editor / Dist / Catalog / Scene Representation / Surface Cache / Global SDF / Radiance Cache / Probe Trace / Denoise 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| tracked package 总体 | 270 / 43,848 / 1,579,121 | 250 Rust、16 WGSL、4 TOML；E3逐文件分类与调用链核对 |
| runtime production Rust/WGSL | 221 / 29,806 / 1,099,143 | provider、scene representation、GPU resources、readback、四个public executor与16份shader |
| runtime tests/fixtures | 39 / 12,241 / 468,626 | 237项test attribute、20项ignored；21项adapter缺失时直接返回成功 |
| package/editor/dist | 9 / 317 / 10,843 | manifest、ID-only Editor贡献、缺失ZUI与metadata-only native carrier |
| ignored shader cache | 42 files / 171,469 bytes | `.zircon-cache`和`.zircon/cache`双路径；21份meta与21份`.wgsl.zst`，不计source evidence |
| selected production fingerprint | 221 files | `f137fd4efad828df2989aa9c138f7c7ae0f0eb46c5929b4a10b91ee09dd0871d` |
| Unreal、Unity Graphics、Godot、Bevy、Fyrox参考 | 24个关键文件 | 对照Lumen Scene/Surface Cache/Probe/Radiance/HRT、APV/SSGI、SDFGI/VoxelGI、light probe与deferred renderer |

Hybrid GI已经有typed provider、四段Render Graph、depth/HZB handoff、Mesh SDF/Global SDF page table、probe trace、temporal resolve、debug source view和readback ring，可保留这些合同、测试向量与部分kernel。但当前viewport被固定压成`8 x 8` trace tile；scene packet只容纳16个Surface Cache page、4个`4 x 4 x 4` voxel clipmap和64个cell；场景表示是每mesh一个球形card、每card一个所谓screen probe；Radiance Cache最多32个slot，每个probe仅为`4 x 4` RGBA8 tile，trace stage只是把packed radiance复制到内部`2 x 2` texel。生产路径没有BLAS/TLAS、RayQuery或ray pipeline执行。

Surface Cache没有捕获真实surface：card来自transform translation/scale，材质统一采样中心UV，page是均匀RGBA8/depth sample，CPU voxel light使用手写常数。主collector在一个全局mutex内clone scene/light/中立DTO并完成projection、residency、dispatch和readback enqueue；每帧又拆出大量buffer/slot readback，未完成项会阻塞整帧并产生FIFO队首阻塞。Editor只注册ID且引用缺失的`authoring.zui`，manifest/default profile、source/native carrier和产品启用事实也相互冲突。报告登记0项新增P0、56项P1、14项P2与36项资格门；Runtime09F3继续拥有14项算法P0，Runtime09A-09F2/09H1/28、Editor22与Plugins01/04/06/08继续拥有共享硬阻塞。本轮只写review与重构计划，未修改production/tests，未运行Cargo、GPU、Editor、NativeDynamic、像素、跨平台、soak或性能测试。详见`zircon_plugins/19-first-party-hybrid-gi-source-runtime-editor-dist-catalog-scene-representation-surface-cache-global-sdf-radiance-cache-probe-trace-denoise-product-integration-review.md`。

## 194. Runtime Debug Gizmo / Command Buffer / Retained Asset / Extract / View Filter / Budget / Render 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `core/framework/gizmos` production | 6 / 798 / 20,360 | 逐文件读取buffer、command、config、extract、retained与module export；0项内嵌test |
| dedicated gizmo tests | 1 / 207 / 6,933 | 10项test、0 ignored；逐测试读取，未执行Cargo |
| overlay DTO与WGPU line提交 | 7个focused文件 | 读取overlay DTO、prepare/record、line vertex、pipeline、shader与逐帧GPU buffer创建 |
| 产品producer与stats | 4个focused文件 | Editor camera/light、Runtime Navigation、Navigation/AI plugin、Virtual Geometry与dynamic extract stats反查 |
| focused total | 18 / 2,715 / 82,535 | 12项test；fingerprint `bac42e7125ae5e46246b12d50730f8988c2de1bc7a3377dd7748ab964891ae27` |
| Unreal、Bevy、Godot、Fyrox、Unity Graphics参考 | 18个关键文件 | 对照DebugDrawService/LineBatch、typed config/retained store、Editor gizmo plugin、debug renderer与volume/debug display policy |

Runtime Debug Gizmo已有九类typed command、稳定记录顺序、CPU shape tessellation与最终WGPU line提交，可保留这些typed math和最小reference路径。但`GizmoBuffer`、`GizmoAsset`、`RetainedGizmo`、extract与append的production caller均为0；Editor、Navigation、AI与Virtual Geometry已各自直写`SceneGizmoOverlayExtract`。group、line width、depth bias、render layer与screen scale没有进入overlay/pipeline/shader，`selected`也不渲染；retained复制commands却丢失config，且没有asset handle、generation、owner lease、TTL或remove receipt。

Axis只变换origin，Sphere/Circle不缩放radius，AABB只变换min/max后重建，在旋转、负缩放与非均匀缩放下均不可靠。最终renderer固定LineList/LessEqual/zero bias，逐帧创建buffer并逐icon提交，也没有qualified producer/view identity、finite validation、bounds/culling/LOD、预算、诊断、capture或产品设置。报告登记0项新增P0、56项P1、14项P2与36项资格门；PERF-MVP-333、Runtime09A/09B/23/24/47、Editor03、Runtime08D/08F与Plugins14/15继续拥有父问题。本轮只写review与索引，未修改production/tests，未运行Cargo、Editor、WGPU、RenderDoc、soak或性能测试。详见`zircon_runtime/49-runtime-debug-gizmo-command-buffer-retained-extract-filter-budget-render-product-integration-review.md`。

## 195. Runtime Manager Resolver / Named Service / Handle / Generation / Lifecycle 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `core/manager` production | 5 / 435 / 14,235 | 逐文件读取resolver、service wrapper、canonical names、tests与module export；3项内嵌test |
| descriptor、registry、handle与state | 9个focused文件 | 读取identity、generation、admission、in-flight、lazy factory、解析和错误合同 |
| Runtime产品caller | 15个focused文件 | 读取Foundation/Asset/Input/Scene/Graphics/Platform/Animation/Navigation及dynamic session/runtime loop |
| Editor与首方插件caller | 8个focused文件 | 读取retained host、viewport及AI/Net/Physics/Sound模块注册 |
| focused total | 37 / 7,351 / 269,212 | 29项test；fingerprint `7edf502c02b69e06a8761840cc7bb6f5b9400f91271052a732ac1ae35cac7849` |
| Unreal、Bevy、Godot、Fyrox、Unity Graphics参考 | 14个关键文件 | 对照Subsystem scope/collection、scheduler resource access、singleton catalog、plugin context与typed render context slot |

Runtime Manager Resolver已有弱Core resolver、typed handle、stale generation拒绝和底层`ServiceCallGuard`底座；API也真实进入Runtime、Editor、App与首方插件。但manager facade在identity校验后只clone裸`Arc<T>`，调用不进入in-flight drain，已解析对象可越过deactivation/generation失效，原生trait object还可能越过DLL卸载边界。AI/Net/Physics/Sound/Animation/Scene等同时注册raw concrete manager与wrapped interface service，形成两套contract与生命周期节点。

`ManagerDescriptor`没有type/scope/provider/ABI/affinity/access/unload policy，handle三个identity字段公开，捕获时不验证T。句柄index也不直接寻址，解析仍走`RegistryName`、全局`Mutex<HashMap>`、downcast与Arc clone；Graphics resource streamer、dynamic session、render loop和Editor viewport已在产品热路径重复使用。报告登记0项新增P0、56项P1、14项P2与36项资格门；生命周期/原生卸载硬阻塞继续由Runtime01、Interface01、Plugins01、App01负责，dense slot性能主问题继续由PERF-MVP-628负责。本轮只写review与索引，未修改production/tests，未运行Cargo、DLL unload、Editor、WGPU、soak或性能测试。详见`zircon_runtime/50-runtime-manager-resolver-named-service-handle-generation-lifecycle-product-integration-review.md`。

## 196. Runtime Asset Registry / Index / Persistence / Rebuild / Incremental / Query 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `asset/registry` production | 16 / 1,861 / 64,031 | 逐文件读取entry、index、query、persistence、rebuild、incremental、targeted、inspection与extractor；3项内嵌test |
| dedicated registry tests | 6 / 890 / 30,186 | 19项query、persistence、incremental、extractor与scan safety test |
| ProjectManager与catalog generation | 11个focused文件 | 读取open、full/targeted generation、projected inventory、durable transaction、publication与registry access |
| AssetManager产品链 | 5个focused文件 | 读取open/reimport/watch、resource同步、generation event与diagnostic投影 |
| Runtime/Editor消费者 | 5个focused文件 | 读取Runtime UI、Scene reference、Editor index与project sync |
| 产品集成tests | 4个focused文件 | 读取full/targeted/catalog/watch generation回归 |
| focused total | 47 / 11,574 / 419,973 | 74项test；fingerprint `d7a1f02ecd673972dabda38fa98f6d8935bcaf1d9e8c63342314e2d6074c6597` |
| Unreal、Bevy、Godot、Fyrox、Unity Graphics参考 | 16个关键文件 | 对照indexed state/category edge/async gather、typed load state/source、UID/import currentness、reflection graph与source/artifact/custom dependency consumer contract |

Runtime Asset Registry已有UUID/path/source/referencer反查、JSON版本与corrupt rebuild、duplicate检测、targeted replacement、project inspection和不可变catalog generation。产品full generation还通过ProjectedMetaInventory把sidecar变化留在candidate，并把`.zmeta`、artifact和registry纳入同一durable journal；这条产品路径不能被误写成逐文件裸写。

但duplicate GUID自动remint没有reference closure、redirect/tombstone或migration receipt，独立rebuild还会在registry persist前逐sidecar保存。ProjectManager与ProjectAssetManager则先安装candidate/resource/watcher并发布generation event，随后才把`RecoveryDeferred`转成错误，形成caller失败与live状态成功并存。registry v1不绑定exact type/schema/artifact/source/project/BuildSet/generation/currentness；扫描缺确定性root/case/orphan/budget policy，query全表scan并反复分配排序，public registry incremental helper只有tests caller。报告登记2项P0、60项P1、16项P2与40项资格门；Runtime04/24/25、Editor04、Tooling37与PERF-MVP-556继续拥有共享父问题。本轮只写review与索引，未修改production/tests，未运行Cargo、项目导入、Editor、watch、fault injection、soak或性能测试。详见`zircon_runtime/51-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-review.md`。

## 197. Editor Builtin View / Window Descriptor Catalog / Content Provider / Capability / Template / Localization 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| `ui/host/builtin_views/**` | 29 / 838 / 33,867 | 逐文件读取全部Activity View/Window定义、聚合与capability side table；38个目录内定义 |
| 外部内建定义与产品路径 | 2个定义点 + 13个owner文件/目录 | 补齐UI Asset、Welcome，反查registry/open/snapshot/pane/layout/menu/reflection/domain session |
| 聚焦测试 | 5 / 1,305 / 47,857 | 25个test、0 ignore；metadata、host placement、instance policy与capability toggle |
| 唯一ZUI资源 | 16 / 2,771 / 135,439 | 16/16路径存在；fingerprint `bf408f1cf43a67cf8a928eb81ceefefb9bbb2cc88868fd060dc5885bbe45b468` |
| 聚焦目录合计 | 29 / 838 / 33,867 | fingerprint `5b1d206bd6fb5be9b3767672942eaa9a93db6fb585c4d62a7baf377e22fe003e` |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics参考 | 14 / 11,421 / 415,202 | spawn/can-spawn、真实Control/tool owner、plugin lifecycle与feature-conditioned provider |

40个内建定义由22个Activity View与18个Activity Window组成，14项声明pane template、8项声明activity-window template。集中目录、capability snapshot、placement和UI Asset/Animation真实session是可保留底座；但19项不在content-kind映射中，17个所谓functional panel/window会落入Placeholder。10个panel只有default design stack caller，`editor.scene_game_window`没有目录外production caller，Prefab/Material/Animation/Diagnostics四个window只有command/menu caller，UI Asset与Asset Browser window也只多一个Welcome入口；`open_descriptor()`仍在没有provider/template/session body时返回成功，snapshot又把Placeholder标成`placeholder: false`。Prefab虽被映射，pane正文仍明确声明asset-specific tooling是placeholder。

报告登记1项P0、40项P1、12项P2与32项资格门，要求先把假Available入口降级，再建立kind-specific typed definition、compiled immutable catalog、provider-bound open transaction、复合capability、template/localization/icon link、alias/persistence migration与40项资格矩阵。Editor03/14/15/23/25继续拥有domain工具，Editor50拥有extension lifecycle，Editor13拥有layout restore/migration；本轮只写review与索引，未修改production/tests，未运行Cargo、GUI、ZUI编译、reload、restore、pixel/accessibility或性能测试。详见`zircon_editor/52-editor-builtin-view-window-descriptor-catalog-content-provider-capability-template-localization-product-integration-review.md`。

## 198. Editor Interactive Tool Scheduler / Resource Lease / Input Capture / Scene Mode / Modal / Extension Lifecycle 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| 聚焦Zircon源码 | 20 / 4,910 / 170,594 | scheduler、context service、event、Scene Mode与Viewport产品链逐文件审查；43个test、0 ignore |
| `core/tools/**` | 4 / 1,336 / 41,588 | canonical set、single/set queue、promotion、release_all、ToolId与14项unit test |
| 产品caller反查 | 全量`zircon_editor/src` | 仅builder构造service；0个真实acquire/release caller、0个`editor.tool` subscriber |
| 聚焦源码合计 | 20 / 4,910 / 170,594 | working-tree fingerprint `c77effa7d049d8d16f5b49174332c4dce48b31206934223aaddd09cf4026644c` |
| Unreal、Fyrox、Godot、Bevy、Unity Graphics参考 | 19 / 9,893 / 369,905 | tool instance、input capture/focus、terminal disposition、plugin removal、阶段排序与状态恢复 |

现有scheduler是可保留的局部骨架，但尚未成为产品authority：SceneMode、Viewport、Gizmo、Modal、Export与extension unload全部绕过租约，三个exclusive resource只有tests使用。算法还允许同一ToolId以不同集合覆盖`active_sets`并遗留幽灵holder；service在解锁后逐条publish，能让并发Acquire/Release以错误因果顺序进入bus，同时忽略delivery failure。报告登记3项P0、48项P1、12项P2与36项资格门；Editor03/09/48/50继续拥有Scene/Gizmo业务、job/export operation、message bus与extension lifecycle父问题。本轮只写review与索引，未修改production/tests，未运行Cargo、GUI、并发模型、reload、focus-loss或性能测试。详见`zircon_editor/53-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-product-integration-review.md`。

## 199. Editor Workbench Shell AutoLayout / Constraint Language / Responsive Tier / Region Binding / Geometry Authority 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| 聚焦Zircon源码与资产 | 69 / 10,694 / 372,587 | 44个autolayout文件、产品recompute/template/render链、共享constraint合同；89个test、0 ignore |
| `autolayout/**` | 44 / 4,466 / 当前工作树 | skeleton、asset、binding、constraint、tier、geometry和fallback逐文件检查 |
| 产品caller反查 | 全量`zircon_editor/src`与资产引用 | 0个`WorkbenchShellRegionsAsset`产品loader caller、0个CSS-like product caller |
| 聚焦源码合计 | 69 / 10,694 / 372,587 | working-tree fingerprint `68b9521019c7a2e8a82baaf69ae9051bc365386227821a572b945c96afffdae9` |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics参考 | 20 / 16,083 / 589,880 | splitter/dock真实tree、responsive arrangement、single computed publication与产品pane |

Workbench shell已有typed skeleton/region/tier/constraint、componentized Taffy tree、drawer resize和大量局部测试，但声明与产品没有闭环：tracked `shell_regions.toml`、asset parser和CSS-like declaration parser都只有tests consumer，资产内`panel_asset`从未实例化。产品同一次recompute还分别运行custom `WorkbenchShellGeometry`与template/Taffy布局；render、pointer、drag、viewport使用template frames，minimum、floating和frame reuse却依赖legacy geometry，两套Narrow/Ultra规则并不相同。template recompute失败只记日志，随后继续把旧template frames与新model/legacy geometry发布成一个snapshot。

报告登记3项P0、52项P1、12项P2与36项资格门，要求建立versioned `WorkbenchLayoutSource -> Compiler -> CompiledWorkbenchLayout`、recursive RegionGraph、typed provider/token link、单一componentized/Taffy geometry authority和原子published generation。Editor01/Runtime11A继续拥有generic retained/Taffy布局，Editor13拥有dock/tab/floating persistence，Editor52/50拥有pane provider与extension lifecycle。本轮只写review与索引，未修改production/tests，未运行Cargo、GUI、DPI/monitor、fault、pixel、a11y、soak或性能测试。详见`zircon_editor/54-editor-workbench-shell-autolayout-constraint-language-responsive-region-binding-geometry-product-integration-review.md`。

## 200. Editor Structured Clipboard / Cut-Copy-Paste / Duplicate / Delete / Cross-Document Remap / Drag Payload 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| 聚焦Zircon源码与资产 | 78 / 17,182 / 602,452 | command/intent/delete/selection/history、hierarchy drag、DynamicScene、exact detach与text clipboard；93个test、1个ignore |
| Editor产品caller反查 | default command/keymap/menu、workbench state、retained host | Delete已接通；Copy/Cut/Paste/Duplicate production入口为0 |
| 聚焦源码合计 | 78 / 17,182 / 602,452 | working-tree fingerprint `f31924c2c742d6ae368be14ece7edbd786b5f5f647b8a9751b70bb90d569f7bd` |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics参考 | 13 / 29,536 / 1,117,327 | capability routing、root normalization、deep clone、dependency closure、identity remap与single transaction |

Editor当前没有structured Copy/Cut/Paste/Duplicate产品链；hierarchy drag只用`scene://node/{id}`字符串和旁路NodeId数组完成reparent。更严重的是Delete command只捕获固定`NodeRecord`，apply取得包含完整component/observer/tick/storage状态的`DetachedEntityBatch`后立即丢弃，Undo再按固定字段重建，因此会静默丢失任意registered typed component、dynamic/plugin component、observer与原始tick/storage状态。

报告登记1项P0、64项P1、12项P2与40项资格门，要求先以exact detached batch关闭Delete撤销数据损失，再建立domain-provider驱动的portable payload、subset capture、component clone/serialize/remap policy、destination plan、document transaction、OS clipboard与drag convergence。Editor02/03/05/08/23/24、Runtime11A/11B/24继续拥有各自父边界。本轮只写review与索引，未修改production/tests，未运行Cargo、Editor、OS clipboard、跨文档、crash、soak或性能测试。详见`zircon_editor/55-editor-structured-clipboard-cut-copy-paste-duplicate-delete-cross-document-remap-drag-payload-product-integration-review.md`。

## 201. Editor Search / Filter / Query Index / Result / Find Usage / Reference Navigation 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| 聚焦Zircon源码与资产 | 85 / 27,565 / 1,143,307 | Asset/Hierarchy真实搜索、Workbench module/extension/generated入口、reference graph/row/pointer/navigation；137个test、1个ignore |
| 真实产品链 | Asset Browser、Hierarchy pane | Asset为当前目录ASCII子串+单kind线性扫描；Hierarchy保留祖先/Unicode fallback/5k fixture，但active filter强制全量reflow |
| 能力真实性链 | Scene、Effect、Tags、Blend Space、Icon Usage、Tag Reference Scan | Scene无events且filter route漂移；Effect/Tags静默no-op；Blend Space固定3行；Usage固定14；Scan仅静态route |
| 聚焦源码合计 | 85 / 27,565 / 1,143,307 | 13个非本轮在途文件；working-tree fingerprint `28afd9d66c3667997612067aedc4a4ccf98d5818d5fefa591f3a5dd6a3812627` |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics参考 | 17 / 12,198 / 483,650 | compiled expression、thread-safe asset filter、versioned async index、registry/referencer、fuzzy/max-result、find/cancel/progress、provider/result UX；fingerprint `170aeb7da4ca9f7d684462ed8fa5f709fb4d76c790236e94e7c17161550f7f76` |

报告登记2项P0、44项P1、12项P2与38项资格门。先把Scene/Effect/Tags搜索接到真实query owner或硬切Unavailable，并删除Icon/Gameplay Tags伪造Usage/Scan结果；再建立typed query/AST、provider registry、index generation、budget/cancel/deadline、paged result、qualified target、stale resolve和navigation receipt。Editor04/03/08/10/15/20/21/23/25、Runtime24/51继续拥有各自父边界。本轮只写review与索引，未修改production/tests，未运行Cargo、Editor、规模、fuzz、soak或参考性能对比。详见`zircon_editor/56-editor-search-filter-query-index-result-find-usage-reference-navigation-product-integration-review.md`。

## 202. Runtime Dynamic Scene Session Archive / Slot / Capture-Restore / Path / Merge / Retention / Durability 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Session production owner | 565 / 10,510 / 360,657 | 547个public function、367个不同public name；505个文件不超过25行；7个内部test；fingerprint `771b808da25814d2d6746cd44dc4f7e8d691677cd32898bec4947ce5727d3a4f` |
| focused direct tests | 25 / 6,502 / 244,347 | 126个test、0 ignored；archive core/manifest/mutation、level apply、runtime absorption与plan11 contract；fingerprint `49b88f71669c992575e59d0dfc3878516dea55a4534cd63f33758aab6a394a5d` |
| DynamicScene support | 22 / 3,327 / 120,526 | capture/spawn/document/level support；24个test、1 ignored；fingerprint `e9618b9b0b48b64d2925497bdd9d0d5348663e1fe60838b8b04e5eee54981551` |
| 产品caller反查 | scene root re-export + 1 integration test | 普通Runtime、Editor与App consumer为0；不能把facade和test-only caller当产品接入 |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics参考 | 19 / 13,975 / 528,190 | platform/user slot、pack/resolve/apply、visitor/schema与authoring/runtime state分层；fingerprint `97e1ac5eeb38ede917aa4dc1add4fb1b60a52b345a69fdbefe8064436407ba78` |

当前archive保留了dense row/index、deterministic manifest、lineage/revision plan、不可变artifact、512 MiB限制和bounded writer，但path RMW仍会跨进程丢更新，进程内revision map不能承担持久CAS；文件与目录不fsync，backup/restore错误被忽略且没有journal/startup recovery。query完整解析所有scene，outer schema/metadata/retention/merge不足；restore缺world epoch、安全点、participant、rollback和原子world/metadata发布。

报告登记0项新增P0、60项P1、16项P2与40项资格门。先建立Runtime40持有的唯一Session Archive Service、platform/project/principal scoped store、manifest/chunk、expected revision/digest CAS、journal/fsync/recovery与Runtime41 operation；再收敛snapshot participant、Restore Coordinator、query index、three-way merge、retention/GC并hard cut数百个组合facade。Runtime04/05/22/24/25/40/41继续拥有共享父边界。本轮只写review与索引，未修改production/tests，未运行Cargo、跨进程、断电、慢盘、soak或性能测试。详见`zircon_runtime/52-runtime-dynamic-scene-session-archive-slot-capture-restore-path-merge-retention-durability-product-integration-review.md`。

## 203. Runtime Dynamic Scene Asset Reload / Event Generation / Reconciliation / Stage-Apply / Instance Replacement 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Asset reload production owner | 14 / 2,966 / 103,682 | 4个内部test；event ingest、prepare/deferred、gap reconcile、target stage/apply、budget与report逐文件审查；fingerprint `059c599adebec78ac798bed8943826cd2d0d8995cc09599ccd630b7aea768429` |
| focused direct tests | 3 / 1,498 / 54,629 | 26个test、0 ignored；reload主流程、byte budget与Scene patch document；fingerprint `06a1df00e7b123ccd7415aa0aefe5058387369012756f13f0a639eb39ce8bc19` |
| spawn/preflight support | 10 / 1,877 / 63,185 | 6个test；spawn task、compiled preflight与transaction commit；fingerprint `e381afa2c55d697ca739ab575460049026b7f64c64e3bfc96ac4e516bcf825c2` |
| 产品集成链 | 7 / 2,875 / 105,245 | 22个test；project construction、session state/tick、diagnostics、inspection subscription与world-sync invalidation；fingerprint `665f76a77d57e3f46b01368cc56c521a6a5e2cd273588b66f48680948370e898` |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics参考 | 15 / 25,705 / 998,003 | package/object reinstancing、scene/resource reload、asset-instance registry、selection/override/reference repair及Editor-only resource reloader边界；fingerprint `1209933a1740800722af295f701545dd46bdceca0cc2f0fb401580178619b853` |

当前reload queue保留了raw event数量/字节/时间预算、per-AssetId single-flight prepare、deferred successor、取消与stale结果检查、增量gap reconciliation、active/schedule/ready/apply/metadata/prepared/resident容量限制、target generation/change-tick CAS、reactive wake和compiled spawn transaction。这些机制应保留，但还没有形成场景实例替换系统。

本轮冻结3项P0。第一，订阅事件与gap reconcile均没有project/world/Level/instance qualification，Added/Modified/Renamed和所有Ready/Reloading SceneAsset都可注入当前Level；现有测试甚至明确把两个无关scene asset应用到同一空World。第二，没有instance registry，Modified/Renamed只追加实体，Removed/ReloadFailed不清理旧实例，apply remap在frame report后丢失，selection、override、reference与provenance均无修复owner。第三，`latest_revision`在locator、容量和schedule成功前推进，ready resident overflow又只增加drop计数；同revision随后被判stale，资源代际可前移而World永久保留旧内容且无retry/terminal failure。

报告登记3项P0、60项P1、16项P2与40项资格门。目标是建立qualified `DynamicSceneInstanceRegistry + SceneReloadCoordinator`：以project/world/Level/instance/source generation限定事件，只对已实例化资产执行replace；prepare阶段保留last-known-good，validate后在安全点原子切换实体、reference、selection、override和diagnostics，并为每个accepted event留下可查询terminal disposition。Runtime04 bounded single-flight与Runtime08 compiled transaction failure仍为open，必须取得current-source受管Windows terminal receipt。本轮只写review与索引，未修改production/tests，也未运行Cargo、产品reload、fault injection或benchmark。详见`zircon_runtime/53-runtime-dynamic-scene-asset-reload-event-generation-reconciliation-stage-apply-instance-replacement-product-integration-review.md`。

## 204. Runtime Scene Event Mirror / Registration / Subscription / Cursor / Backlog / Overflow / Reclaim / ABI / Consumer 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Runtime core/World/Session owner | 10 / 4,110 / 150,274 | 19个inline test；registration、per-sub queue、reclaim、World、Session page/FFI与plugin registration逐文件审查；fingerprint `4a31cd849a6db4d9be983fc8cd8eb14d47d52aac8efd92bda4e909b58b36cf44` |
| Interface/Gateway/Editor consumer | 10 / 1,833 / 64,023 | 2个inline test；V7 DTO、Gateway decode、consumer registry、pending/fair pump、sequence validation与lifecycle；fingerprint `d8bcad6c9787a58826fc1c86ce0f001dd636c543b86872543c2278a487fbd9e7` |
| SDK、AI与Navigation产品链 | 5 / 1,710 / 63,689 | 2个inline test；SDK registration、AI producer/consumer与Navigation reader-count producer gate；fingerprint `281e178f6baa503775fdc5b2260b742f300b80e8d69d6e58388d1a892d0ceb3f` |
| focused direct tests | 11 / 5,330 / 189,428 | 100个test、2 ignored；scene queue/reclaim、dynamic ABI、Editor pump、Navigation/AI与interface safety；fingerprint `174d77a4072cb23bf77ac685eb0431c208dc3feb6548c69504be3cfaf72235c9` |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics参考 | 8 / 4,535 / 142,845 | shared message storage/cursor/missed count、router scope/thread/tracing、signal snapshot/lifetime、generational broadcaster与Editor callback teardown；fingerprint `c772f5e7c4c98667976aeb59f3f0196dd26b494aeaefa9b27fcecaacfc984e58` |

当前mirror保留了fixed page/queue、payload/wire硬限制、raw JSON直写、Session allocation rollback、generational reclaim、reader-count rollback以及Editor单页pending/round-robin/锁外callback。这些是有效底座，但还没有形成shared、acknowledged、recoverable event stream。

本轮冻结3项P0。第一，AI Runtime以普通`event()`注册两个Editor consumer所需event，SDK无mirrored builder，三个AI Editor consumer在真实Runtime无法订阅。第二，Runtime在foreign allocation成功后即commit，Gateway decode和typed callback在后；overflow/oversized/callback failure不产生连续gap或resync，Host只检查单调sequence并可永久消费失败delivery。第三，每subscriber重复ECS observer、JSON encode和64 MiB queue，subscription count无全局预算，publish CPU、lock wait与RSS可无界放大。

报告登记3项P0、60项P1、16项P2与40项资格门。目标是建立`SceneEventBroker + SharedEncodedStream + AcknowledgedConsumerCursor`：provider/schema/scope/delivery class由单一exposure contract生成，同一event只编码一次并进入global-budget shared segments，cursor以credit/ack/lag推进，overflow返回dropped range与resync token，Editor语义成功后才ack。Runtime02/05/43、Interface01/07、Editor47与Plugins01/12/14/15继续拥有共享父边界。本轮只写review与索引，未修改production/tests，也未运行Cargo、真实产品、fault、soak或benchmark。详见`zircon_runtime/54-runtime-scene-event-mirror-registration-subscription-cursor-backlog-overflow-reclaim-abi-consumer-product-integration-review.md`。

## 205. Runtime Foundation Module / Config / Event / Service / Driver / Manager / Persistence / Lifecycle 物理范围

| 范围 | 文件 / 行 / bytes / tests | 本轮证据 |
|---|---:|---|
| Foundation contract、manager、Core config/event owner | 31 / 2,658 / 85,478 / 5 | contract、module、manager、worker、atomic persistence、commit fence与Core EventBus逐文件审查；fingerprint `8a20ba6f32d9b4bf1f0dafed551deb4ff73fc86842961c1a1013f3ae7a14d917` |
| App、dynamic session、Editor及邻接module产品链 | 14 / 4,915 / 183,004 / 31 | bootstrap/profile/config绕行、module dependency、manager resolver与产品consumer逐调用点审查；fingerprint `7412444bbd8c7a44bb1f66abc62c22db779a36c6e2f5e6bdb3d08fdf6fd7a27b` |
| focused direct tests | 15 / 3,335 / 120,234 / 80 | config/event/manager、entry profile、dynamic session与persistence行为/结构守卫；fingerprint `54feed85b074ec5ad5ce41d4c8d25cf9a139c6decb08dda7c70a1a66ac018803` |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics参考 | 14 / 9,199 / 348,007 / 25 | config hierarchy/layer、module lifecycle、typed message cursor、resource event subscription、Editor plugin与debug registration teardown；fingerprint `0f09719889061633ed598e52c8518a0608aaf4802b40b410adfc014e592b455e` |

当前Foundation保留dirty/persisted generation、25ms debounce worker、atomic staging/commit、backup recovery、path epoch fence、有界flush、失败指标、CoreWeak ownership及Core EventBus的topic/queue/drop基础。这些机制应进入重构，不应退回同步整文件写或无界临时channel。

本轮冻结3项P0。第一，dynamic session profile只在Foundation激活前直写Core，磁盘恢复会无条件覆盖；任一持久设置变化又snapshot整个Store，把session/window/platform值反向写入全局文件。第二，所有manager默认共享同一路径，第二个live manager注册会递增epoch并静默使第一个manager的后续commit stale。第三，两个Immediate driver均为空ZST且无人解析，manager不依赖driver，EventManager production resolver为0，Asset/Platform只有module name依赖却没有真实service边，load report因此可伪报能力Ready。

报告登记3项P0、56项P1、14项P2与40项资格门。目标是建立`CompiledFoundationContract + LayeredConfigAuthority + TypedEventService + ScopedPersistenceBroker`：启动前编译带source/scope的effective config revision，仅持久化schema授权projection；同进程多Runtime按显式scope lease共享或fail-close；provider必须有行为、依赖、health和真实consumer后才可Ready，并硬切删除Core配置/事件旁路和空driver。Runtime01/02/03/25/42/43/45/46/50与App01继续拥有共享父边界。本轮只写review与索引，未修改production/tests，也未运行Cargo、双Runtime、Editor、fault、soak或benchmark。详见`zircon_runtime/55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md`。

## 206. Runtime Input Device / Event / Frame State / Action Map / Focus / Gamepad / Recording / Replay / Host 物理范围

| 范围 | 文件 / 行 / bytes / tests | 本轮证据 |
|---|---:|---|
| Input contract与runtime实现 | 46 / 3,642 / 112,613 / 2 | framework contract、module/driver/manager、physical state、action evaluator、event retention与record/replay逐文件审查；fingerprint `323352589249ae826ba1122053726980c22af5fc8cc7eeaaeedd873eff83c74e` |
| focused direct tests | 14 / 2,846 / 100,990 / 64 | action mapping/transition、frame state、event buffer、host request、touch/gamepad与record/replay行为；fingerprint `8c0666d95426f2d041ad026a920a2dba636e4585c02c3a95e61ab943f9b37b46` |
| dynamic session、App、script与sample产品链 | 45 / 6,774 / 245,875 / 49 | UI先行dispatch、ABI conversion、keyboard/pointer/IME/file drag/gamepad/rumble、frame与Gameplay Host/Vampire caller逐调用点审查；fingerprint `8627041cf26bf2193cdd8a25b0b8d05267312a9f5460d41d7778097beed08b6b` |
| Unreal、Godot、Bevy、Fyrox、Unity Graphics参考 | 20 / 15,964 / 621,744 / 44 | context priority/consumption、user mapping、device/window/key identity、connection schedule、validated settings、shortcut边界与debug action lifecycle；fingerprint `53424dd5b8a5b5d1b9de2ad9bb58971e8a6a0a29a22faecc78599dc460a07531` |

当前输入实现保留button level/edge、frame transition、focus release、compiled action generation/index、可复用workspace、相邻cursor/motion合并、count-bounded recorder、gamepad poll预算及host request page rollback。这些是有效底座，但Action/Recording/Replay都没有进入真实产品，不能以测试helper或manager注册名宣称工程化完成。

本轮冻结3项P0。第一，Input module为client/server/editor注册空ZST driver和Immediate manager，production没有Action evaluator、Action Map或Recording/Replay consumer，脚本/Vampire仍走raw snapshot，readiness可伪绿。第二，dynamic session在提交物理状态前先交给Runtime UI，UI停止传播会删除整条事件；capture/focus变化若只吞掉press或release会永久卡住held状态。第三，persisted Action Map直接保存临时`gilrs::GamepadId`，键盘又使用左右modifier折叠、WASD特判、Debug字符串FNV和平台raw code，重连、版本或平台变化可静默错绑。

报告登记3项P0、64项P1、16项P2与40项资格门。目标是建立`InputIngressBroker + QualifiedPhysicalState + CompiledActionProgram + InputOwnershipArbiter + StableDeviceBinding + DeterministicInputJournal`：物理事实先按window/device/user/seat/clock/sequence发布，再对同代事实执行UI/gameplay ownership；Local Player消费compiled action；持久绑定使用versioned stable control/device profile；record/replay使用bounded、checked、无默认OS副作用的journal。Runtime06/11A/24/38/43/46/50、Interface01/03/07、App01、Tooling32与PERF-MVP-012/426继续拥有共享父边界。本轮只写review与索引，未修改production/tests，也未运行Cargo、真实窗口、设备矩阵、replay、fault、soak或benchmark。详见`zircon_runtime/56-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-review.md`。

## 207. Runtime Platform Host / Window Registry / Monitor / Display / Event Loop / Application Lifecycle / Surface Command 物理范围

| 范围 | 文件 / 行 / bytes / tests | 本轮证据 |
|---|---:|---|
| Window与Platform contract/production | 34 / 2,794 / 96,162 / 0 | framework Window contract、Platform capability/backend matrix、module/driver/manager逐文件审查；fingerprint `751e8042a97c7001a04700c3d910e6f40a6c62b12e218f283793459890b857f3` |
| App product host完整目录 | 79 / 6,266 / 228,880 / 98 | ApplicationHandler、window creation/event/lifecycle、cadence、native target、surface binding/present与host request逐文件审查；fingerprint `aa39192ae7c8cbdd59a4d97ee0513a019cb4531f6e1e3320bef31efc07390be8` |
| Dynamic ABI、surface与export integration | 26 / 8,375 / 336,478 / 25 | viewport surface ABI、dynamic session、Graphics unsafe surface、runtime library与generated platform host callback逐调用点审查；fingerprint `9eb604923562893b0b3459191f390633d303985d95711a33aacf00f778e62d88` |
| focused direct/source-shape tests | 59 / 11,355 / 423,337 / 218 | window/platform matrix、App source/surface/lifecycle guard、dynamic viewport、export host与ABI contract测试；fingerprint `9bb0df174da2ae2cae3145249f1760966a3aa28cc2db66061f91bf756b01c852` |
| Bevy、Fyrox、Godot、Unreal、Unity Graphics参考 | 23 / 18,519 / 754,201 / 6 | window registry、display topology、application lifecycle、surface teardown、observed output state与native event routing；fingerprint `f2b39ce28278be9a0e95d5eb78e5e8a6fddf6bd7aa550cb5e56163f7ee1766b3` |

当前实现保留typed WindowDescriptor、winit ApplicationHandler、Win32 raw surface、CPU reference presenter和frame cadence等底座，但Platform driver/manager只拥有Preferences，能力矩阵仍可在`enabled=false`且没有已安装/已观测backend时报告window/monitor/event/lifecycle Supported。App只有单`Option<Window>`、固定viewport 1，winit WindowId被丢弃；monitor只在创建期按临时Index选择，requested/observed state、hotplug和命令receipt均不存在。

本轮冻结2项本地P0。第一，Platform capability/readiness把compile selection冒充runtime事实，产品无法区分Compiled、Installed、Observed与Ready。第二，Graphics unsafe raw-handle surface要求原生window在surface释放前有效，但OS `Destroyed`只分发状态，App没有`suspended`/`destroy_surfaces`且不清surface/presenter/window，可能继续访问已失效原生对象。generated browser/Android/iOS callback忽略payload并恒真则继续由Tooling03的`TOOL-EXPORT-P0-005`唯一拥有，不重复计数。

报告登记2项P0、64项P1、16项P2与40项资格门。目标是建立`PlatformHostService + WindowRegistry + DisplayTopologySnapshot + ApplicationLifecycleMachine + SurfaceLeaseRegistry + HostCommandBroker + EventLoopScheduler`，让窗口、显示器、surface与命令都带稳定identity、generation、owner、observed state和terminal receipt。Runtime06/09A/11A/24/43/45/46/50/56、App01、Interface01/05/07与Tooling03/16继续拥有共享父边界。本轮只写review与索引，未修改production/tests，也未运行Cargo、真实窗口、多显示器、移动生命周期、surface fault、soak或benchmark。详见`zircon_runtime/57-platform-host-window-registry-monitor-display-event-loop-application-lifecycle-surface-command-product-integration-review.md`。

## 208. Runtime Plugin Interface Bridge / Slot / Generation / Strong-Weak / Native-VM / Lifecycle / Diagnostics 物理范围

| 范围 | 文件 / 行 / bytes / tests | 本轮证据 |
|---|---:|---|
| core bridge、extension registration与runtime lifecycle | 14 / 2,325 / 77,145 / 8 | frozen table、slot/generation/provider state、weak/strong/import、owner transition与dependency blocker逐文件审查；fingerprint `2160e55ebeb01403d293fd595c28fd479aa500f412704a2aa8a68b73a57cb6ab` |
| native bridge、reload/replay与VM adapter | 11 / 4,846 / 177,623 / 2 | bridge scope/method directory、library lease、load/reload/unload、registration replay与VM host module逐调用点审查；fingerprint `f8686e23ff189231e05a55857d226e0daebccc3402039ebcbf8c84a409c0a460` |
| App、Editor与AI/Physics/ZrVM产品消费者 | 17 / 3,445 / 123,124 / 6 | 静态composition、Editor Play activation/matrix、三项真实typed interface和manifest/source parity；fingerprint `502e788b030ea0c56bb18bcf7d820b26e72ce02da3ada8734fb4c2198c8cfeaf` |
| focused direct/source-shape tests | 19 / 9,334 / 332,200 / 199 | stable snapshot、dependency、native binding/reload/replay、VM host、App profile、Editor matrix和AI consumer；fingerprint `5e2dae85ccb0bacc7f85bde23685f59334cab52bf6fcb2fb70b2edc94d48651f` |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics参考 | 14 / 8,427 / 306,210 / 19 | module unload/quiesce、extension instance rebind、scene state restore、startup lifecycle与typed context slot；fingerprint `c8a37a108ee813b7c856893b935b2f63b5ff0111e98f0e1616ec0227c05477ee` |

当前bridge实现保留frozen table、generation/provider同快照、weak observation、owner transition report、native dynamic-library callback lease、registration replay generation和Editor matrix等有效底座。但`InterfaceSlot`仍是无table/contract/generation的裸`u32`，`WeakBridge::call`在generation检查后可跨disable/deactivate执行，`BridgeGuard`/`StrongBridge`又没有quiescence或retirement。required blocker只来自manifest，不来自live call/task/World holder。

本轮冻结3项P0。第一，bridge调用没有覆盖完整operation的lease，lifecycle不能证明provider已停止。第二，native registration replay两项公开API只有HostHandle转发和tests caller，App/Editor Play不会把native systems安装进World。第三，replay closure捕获固定旧scope；测试明确保留旧binding generation给旧World，真实hot reload后旧owner会transition closed，system静默no-op并强持旧DLL。报告登记64项P1、16项P2与40项资格门，目标是`InterfaceContractCatalog + BridgeTableGeneration + BridgeCallLease + BridgeLifecycleTransaction + WorldBridgeBindingRegistry + VmBridgeAdapter + BridgeObservationStream`。Runtime07、Plugins01、Runtime05/24及Tooling35/36/37继续拥有共享父控制面。本轮只写review与索引，未修改production/tests，也未运行Cargo、DLL、Editor Play、reload、stress、soak或benchmark。详见`zircon_runtime/58-runtime-plugin-interface-bridge-slot-generation-strong-weak-native-vm-lifecycle-diagnostics-product-integration-review.md`。

## 209. Runtime Task Execution / Job Scheduler / Handle / Dependency / Cancellation / Thread Budget / Timer / Shutdown / Diagnostics 物理范围

| 范围 | 文件 / 行 / bytes / tests | 本轮证据 |
|---|---:|---|
| framework DTO、runtime scheduler/pool/timer/bounded lane与Core接线 | 36 / 6,040 / 196,351 / 43 | task descriptor、pool allocation、dependency state、timer、diagnostics、shutdown guard和Core ownership逐文件审查；fingerprint `610eb404f451426d2b0c04dcee452cae3d3a225ce01f2109ece1e24b3b806bd2` |
| Runtime、Editor、App production consumers与私有worker | 82 / 32,222 / 1,164,302 / 161 | Asset、Graphics、Scene、Script、Text、Operation、Editor jobs/autosave/settings和产品teardown逐调用点审查；fingerprint `34c749adb75de82b37560a174dcd2b0e1690a6b8d0c578ad1224a434b9654a8d` |
| focused behavior、pressure、source-shape与Editor job tests | 27 / 7,057 / 238,450 / 142 | pool、scheduler、dependency、panic、bounded I/O、diagnostics、pending admission和shutdown测试逐项审查；fingerprint `8654ba155073a0b1593447908aedad18699a9c8dcaafe01dad856ab78b704142` |
| Unreal、Bevy、Godot、Fyrox与Unity Graphics参考 | 29 / 9,738 / 357,657 / 18 | typed task/result、priority/prerequisite、scope cleanup、worker join、group progress、owner completion和data-parallel dependency合同；fingerprint `67c0d07427e0e44fca9d8a563e23746292d9132ea78828fa78785ce30ca3ff6d` |

当前execution实现保留三类Rayon pool、dependency continuation、worker内协助等待、panic捕获、64-shard diagnostics，以及entry/byte budget、key coalescing、fence、deadline-before-start、cancel authority、terminal ticket与shutdown guard较完整的bounded keyed I/O lane。但`AsyncTaskDescriptor`、`AsyncTaskHandle`、cancellation policy与poll budget只是framework DTO，真实`JobScheduler/JobHandle`不接受这些合同；`TasksModule`又只是descriptor，不创建service、不关闭admission、不取消scope、不排空task、不停timer且不join worker。进程级`OnceLock<TaskPools>`、`JobScheduler::default()`和Text/Graphics/Asset私有worker还绕开统一线程预算。

本轮不新增P0：Runtime02继续唯一拥有task/timer越过dynamic session与DLL unload的P0，Editor09继续唯一拥有shutdown未完成仍拆project/settings的P0。本篇登记72项P1、18项P2与40项资格门，目标是`ExecutionRuntime + WorkerDomain + TaskScope + TaskDescriptor + Task<T> + DependencyGraph + DeadlineService + BoundedOperationLane + DedicatedWorkerLease + ExecutionDiagnostics`，让每项异步工作具备identity、owner、admission、typed result、cancel、deadline、shutdown和observation。Runtime01/02/11/24/40/41/43/46、Editor09、Tooling24/35与Runtime11 implementation plan继续拥有共享父边界。本轮只写review与索引，未修改production/tests，也未运行Cargo、真实shutdown、DLL unload、stress、soak、sanitizer、profiler或benchmark。详见`zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md`。

## 210. Runtime Scene ECS / Entity / Component / Storage / Archetype / Query / Access / Change Detection / Command / Schedule / Parallel / Event 物理范围

| 范围 | 文件 / 行 / bytes / tests | 本轮证据 |
|---|---:|---|
| `scene/ecs`完整kernel | 143 / 22,031 / 701,817 / 42 | identity、component schema、archetype table、sparse storage、query/access/change tick、commands、schedule、events/messages/observers逐文件审查；fingerprint `2142a8d985a23a34deb5ecfbb3f8cbfe60ef111e0249a84939fc3e305a424de6` |
| World、module、LevelSystem与event mirror产品接线 | 93 / 24,796 / 940,182 / 70 | World owner、system registry、frame stage、plugin/native system、maintenance与产品consumer逐调用点审查；fingerprint `f1bec6a150cfd9c073779d0049ff82799186eccead16de9db5f3c517434bd056` |
| focused ECS tests与直接support | 44 / 10,614 / 400,945 / 269 | identity/storage/query/change/command/event/message/observer/schedule行为与source-shape测试；23个文件含`include_str!`；fingerprint `67e63eff0369e106f5691c58ee39b764de909bdece5266e32aeffc6dd762c8e7` |
| Unreal Mass、Bevy ECS、Fyrox、Godot与Unity Graphics参考 | 53 / 53,524 / 1,996,738 / 185 | chunk/archetype/query cache、WorldId/access proof、dependency executor、generational pool、deferred queue与GPU-driven jobs；fingerprint `2c207fedfc1fa1f8282c42def53f66c6e8bef5974a30428a0c6d677ab5f142a6` |

当前ECS保留columnar archetype table、generation-aware sparse locator、stable query order、compiled archetype plan、lazy changed tick、bundle/structural preflight、packed inline command arena、stable worker merge、active event worklist、bounded message retention与copy-on-write observer bucket等真实底座。但system `Query::iter(&self)`、`iter_combinations/count/is_empty`会把共享raw state转成`&mut QueryState`，首个iterator又借用plan slice；第二次安全调用可制造并存可变别名，并在新archetype触发Vec扩容时潜在悬空slice。`RemovedComponentEvents`则只追加Vec，reader仅推进私有cursor，World没有update/clear/retention，长期mutation使内存随历史永久增长。

本轮冻结2项P0、72项P1、18项P2与40项资格门。除两项P0外，Query/SystemState缺World identity、mutable tuple query缺失、真正访问World的系统仍串行、显式dependency未进入worker batch barrier、pause跳过Event maintenance、generic command无完整事务、event reader不注销等均需按Runtime03/05/08/24和本篇owner收敛。目标是`WorldIdentity + EntityAllocator + ComponentSchemaRegistry + ArchetypeStore + QueryPlan + WorldCell + SystemMeta + ScheduleGraph + EcsExecutor + DeferredWorld + EventRegistry + ObserverGraph + EcsDiagnostics`。本轮未修改production/tests，也未运行Cargo、Miri、sanitizer、fuzz、真实多线程World、soak、profiler或benchmark。详见`zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md`。

## 211. Runtime Scene World / Level / Registry / Lifecycle / Project I/O / Snapshot / Clone / Serialization Schema / Transaction 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Level/module lifecycle | 16 / 2,664 / 95,965 | LevelManager、LevelSystem、module contract、replacement epoch、frame state与render extract逐调用点审查 |
| World/project document | 31 / 7,483 / 286,154 | World bootstrap/clone/records/derived state/dynamic component、canonical SceneAsset、document codec与project I/O逐字段审查 |
| Product/dynamic consumers | 16 / 3,712 / 134,191 | Editor Save/Play/startup、runtime play snapshot、DynamicScene capture/spawn与产品调用链逐阶段审查 |
| Public scene contracts | 11 / 374 / 10,299 | framework scene contract、dynamic session project contract与公开Level/Scene request/result逐项审查 |
| 去重合计 | **74 / 14,233 / 526,609** | production fingerprint `23aa86d8421657bf4df5b4882743b5aab06d65b686ace6d45278782840d1ae34`；冻结时6文件dirty，实施前必须重核 |

当前World/Level与project I/O实现保留World replacement epoch、DynamicScene compile/preflight/staging/commit generation fencing、严格resource reference resolution、physics/animation短锁`Arc` snapshot、bounded keyed I/O和单文件原子替换等底座。但Level registry只有只增不减的HashMap，lifecycle只有Loaded/Unloaded且tick不读取；World replacement依赖多个独立Mutex与手工清理；canonical scene没有root schema version、通用component row、stable scene entity identity、迁移链或source revision/CAS。

本轮冻结5项P0。第一，Editor Save先做有损`World::clone`，成功后仍清dirty。第二，Exit Play用同一有损clone覆盖authoring World。第三，runtime把Play snapshot追加进含Camera/DirectionalLight/Cube的默认World并重映射实体。第四，canonical SceneAsset虽声明terrain/tilemap/prefab，loader/saver仍双向写None，Vampire真实terrain也会丢失。第五，World已支持的Sprite2D/Mesh2D没有canonical document字段。报告另登记60项P1、14项P2与45项资格门，目标是`WorldContextRegistry + LevelInstanceRegistry + WorldLifecycleCoordinator + AuthoringSceneDocument + SceneSchemaRegistry + SnapshotCompiler + ScenePersistenceService + WorldReplacementTransaction`。Runtime05/24/39/40/52/53/60继续拥有ECS、identity、prefab、SaveGame、archive与reload父边界；本轮未修改production/tests，也未运行Cargo、Editor、Vampire、fault、multi-world、durability、soak或benchmark。详见`zircon_runtime/61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md`。

## 212. Runtime Scene Hierarchy / Transform Propagation / Reparent / Activation / Mobility / Visibility / Bounds / Render Handoff 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Authority与contract | 20 / 5,663 / 218,544 | Hierarchy、LocalTransform、Mobility、typed/generic/reflection mutation、topology validation与destruction逐入口审查 |
| Derived state与render handoff | 10 / 3,747 / 131,366 | dirty frontier、active/world transform propagation、node cache、visibility input与frame extract逐阶段审查 |
| Editor产品链 | 7 / 1,970 / 64,917 | command/undo、gateway、viewport geometry/gizmo generation与Workbench消费逐调用点审查 |
| 聚焦测试 | 12 / 3,287 / 125,396 | hierarchy/derived diagnostics、subscription、performance acceptance、Editor node/viewport/gateway测试逐断言审查 |
| 去重合计 | **49 / 14,667 / 540,223** | Zircon fingerprint `27d7f49d6ecaabe36cb1a075f658bc4b01609516a455d65c3054faa27129602c`；冻结时12文件dirty，实施前必须重核 |

当前Scene hierarchy实现保留stable entity order、checked parent入口、mobility validation、derived schedule、WorldMatrix/ActiveInHierarchy投影、subscription coalescing与renderer static/dynamic分类等底座。但Hierarchy、LocalTransform和Mobility仍是可经公共generic/reflection API直接写删的普通组件，raw cycle会绕过validation并使无visited guard的ancestry walker挂起；WorldMatrix与ActiveInHierarchy派生事实也可被伪造或删除，clean domain不会自动修复，renderer会直接消费错误状态。

本轮冻结2项P0。第一，protected hierarchy/transform/mobility authority可被generic insert/get_mut/remove绕过，破坏forest、Static与transform不变量。第二，derived WorldMatrix/ActiveInHierarchy可被公共API和reflection伪造/删除并污染render truth。报告另登记64项P1、16项P2与48项资格门：reparent必须支持KeepWorld/KeepRelative/Snap和typed receipt，destruction必须显式区分subtree/orphan policy；World级dirty bool与wide SceneNode cache必须替换为dirty-root/subtree增量传播；activation、mobility和visibility需要分域participation；Scene必须发布有bounds、generation和remove tombstone的persistent spatial delta。Runtime05/09B/23/24/29/60/61继续拥有World、renderer、precision、identity、partition、ECS与persistence父边界；本轮未修改production/tests，也未运行Cargo、Editor、fuzz、scale、fault或benchmark。详见`zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md`。

## 213. Runtime Scene Reflection / Type Schema Registry / Dynamic Component / Property Address / Inspection Artifact / Subscription 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Public reflection与path contracts | 18 / 1,405 / 39,455 | registration、metadata、stable path DTO与公开reflection读写合同逐字段审查 |
| Runtime registry/adapters/dynamic values | 27 / 4,058 / 141,277 | TypeRegistry、component/resource adapters、VM schema sync与dynamic payload admission逐入口审查 |
| Property binding与inspection publication | 34 / 10,488 / 392,354 | address parse/compile、mutation、field artifact/cache、watch/subscription与publication逐阶段审查 |
| Builtin component declarations | 20 / 1,523 / 45,765 | scene builtin component导出、derive metadata、skip字段与reflection inventory逐类型审查 |
| Editor产品consumer | 6 / 2,645 / 89,599 | Inspector command、draft/undo、Runtime gateway与字段消费逐调用点审查 |
| 聚焦外部测试 | 19 / 4,626 / 186,494 | registry、path、dynamic component、property、inspection、subscription与Editor命令断言审查 |
| 去重合计 | **124 / 24,745 / 894,944** | Zircon fingerprint `7bba91e6466367a94c2b35c277f25006f87fafd368796a4d5c44d0b005f49e36`；冻结时124文件clean |

当前Scene reflection实现保留derive field metadata、full/short type lookup、dense slot adapter、VM clone/preflight、dynamic component batch generation、不可变inspection field slice、typed subscription index与Editor reflection命令等真实底座。但schema descriptor、adapter、stable identity/version/provider generation没有统一admission transaction；`ComponentPropertyPath`的raw/component/segments可经serde形成矛盾状态，普通路径与compiled dynamic路径又分别按第一个和最后一个点号解释plugin type；inspection逐次扫描registry、吞adapter错误、只缓存单entity，watch溢出也不发布resync事实。

本轮冻结1项P0：`World::register_component_type`先提交live `ComponentTypeRegistry`，随后更严格的`TypeRegistry::register`失败时不回滚，使失败调用永久留下descriptor/payload可用而reflection缺失、重试又被duplicate拒绝的半注册World。报告另登记67项P1、17项P2与48项资格门，目标是`ReflectionCatalogTransaction + StableTypeSchema + TypedPropertyAddress + CompiledPropertyPlan + ReflectionMutationTransaction + InspectionPublication + SubscriptionCursor`。Runtime24/54/60/61/62、Interface02与Editor05继续拥有identity、cursor/resync、ECS、persistence、protected mutation、wire及Inspector父边界；本轮未修改production/tests，也未运行Cargo、Editor、fuzz、plugin reload、跨进程、scale或benchmark。详见`zircon_runtime/63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-review.md`。

## 214. Runtime Resource Authority / Asset Handle / Load Request State Machine / Version Lease / Cache / Dependency / Reload / Cancellation / Product Integration 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Public contracts与facade | 27 / 2,393 / 74,073 | resource DTO、handle、facade、load/readiness/revision公开合同逐入口审查 |
| Authority/readiness/lease/event | 57 / 11,559 / 391,154 | record、payload、typed load、lease/refcount、dependency readiness、event publication逐状态审查 |
| Project load/publication/lifecycle | 31 / 3,748 / 141,773 | project asset manager、artifact access、resource sync、reload candidate与close_project逐事务审查 |
| Runtime/Editor产品consumer | 79 / 16,281 / 607,986 | frame extract/submission、ResourceStreamer、Editor asset refresh、PBR viewer与产品调用链逐调用点审查 |
| 聚焦外部测试 | 17 / 2,541 / 90,998 | manager、facade、project pipeline、load state、lease、close与错误路径断言审查 |
| 去重合计 | **205 / 35,248 / 1,257,681** | Zircon fingerprint `20e6da07298cb6d44ff069936c5e36dffffc3667e866e8334cf07c3f91a1d485`；冻结时3文件dirty，实施前必须重核 |

当前resource实现保留record/revision/readiness、typed downcast、project dependency闭包、reload candidate、bounded keyed I/O与局部lease等真实底座。但公开payload mutation只验证broad kind，不验证exact payload type；官方测试可把`ShaderAsset`写入Texture record并成功，使broad readiness报告Loaded、typed readiness报告NotLoaded，而`ensure_resident`因“已有任意payload”拒绝重新加载。raw `Arc<T>`读取又绕过lease refcount，ResourceId-only handle可跨project/generation别名，payload-only publication不产生足够event事实。

本轮冻结2项P0。第一，错误精确类型可经公共API进入authoritative store并形成不可自愈的三重事实。第二，frame submission和ResourceStreamer可在帧关键路径同步执行冷盘加载、依赖闭包、decode及大payload深clone，且没有ticket、priority、deadline、progress、cancel或frame guard。报告另登记66项P1、17项P2与50项资格门，目标是`ExactAssetTypeCatalog + QualifiedAssetHandle + AssetLoadCoordinator + ResourceVersionSlot + VersionLease + CachePolicy + TypedDependencyGraph + ReloadCandidateTransaction + AssetPublicationReceipt`。Runtime04/09D/24/51/54/59、Editor04与Plugins07继续拥有artifact、GPU residency、identity、registry、cursor、scheduler、Editor和importer父边界；本轮未修改production/tests，也未运行Cargo、GPU、Editor、fault、scale或benchmark。详见`zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md`。

## 215. Runtime Scalability / Quality Profile / Device Profile / Capability Tier / Dynamic Resolution / Frame Budget / LOD / Feature Fallback / Product Integration 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Public contracts、resolution、profile与budget定义 | 19 / 6,920 / 280,839 | profile字段、camera resolution、frame/memory budget与序列化边界逐字段审查 |
| Render-framework validation、state、profiling与fallback | 35 / 7,262 / 315,174 | capability、compile裁剪、profile apply、frame profiler、degrade state与viewport记录逐状态审查 |
| Feature、LOD、residency与diagnostics consumer | 40 / 12,904 / 535,827 | shader/TAA/volumetric/SSR/advanced feature/mip/LOD与stats消费链逐调用点审查 |
| Runtime、Editor与App product consumer | 3 / 1,133 / 49,493 | Editor viewport硬编码profile、lifecycle reapply与PBR viewer迁移路径审查 |
| Focused external及inline-test-bearing文件 | 99 / 39,172 / 1,562,410 | profile开关、advanced provider、dynamic controller、degrade ladder和产品fixture断言审查 |
| 去重合计 | **196 / 67,391 / 2,743,743** | Zircon fingerprint `97ff36acb69e1398062f3e6fe3362208c9836cf594de49357baf25387347fa80`；冻结时19个入选路径dirty，实施前必须重核 |

当前实现保留`RenderQualityProfile`主要feature开关、shader/TAA tier、mip/anisotropy，profile应用的snapshot/锁外compile/加锁commit，capability summary与compile option裁剪，bounded GPU profile ring、memory degrade hysteresis，以及带scope/generation/reason/history-reset的动态分辨率控制器类型。但profile没有稳定ID/schema/source/device/budget/revision，capability validation只硬覆盖AA/Solari，advanced support又被简化为offscreen+graphics queue；requested与resolved fallback没有receipt。

生产提交仍直接消费camera作者写死的scale，动态控制器和delayed GPU sample/decision没有实例或consumer；固定14 ms与固定内存预算不绑定device/display/product，degrade又在GPU timing合并前求值。唯一global ladder通过`min` scale、global mip和字符串feature关闭影响所有viewport，一个view的pressure可污染其他view。Editor是唯一明确production profile caller并硬编码VG false、HGI true及固定budget，App/runtime preview/PBR viewer没有统一quality policy。

本轮登记0项新增P0、64项P1、16项P2与48项资格门；目标是`QualityProfileCatalog + DeviceProfileResolver + CapabilityTierResolver + ProductQualityPolicy + FrameBudgetController + DynamicResolutionCoordinator + FeatureScalabilityRegistry + PerViewportQualityState + QualityTransitionTransaction + EffectiveQualityReceipt`。Runtime09A/09B/09C/09H1/09H2/22/24/42/45/57继续拥有RHI、具体feature/time/identity/composition/preference/platform父边界。本轮未修改production/tests，也未运行Cargo、GPU、多viewport、display/power/device-loss、soak或benchmark。详见`zircon_runtime/65-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-review.md`。

## 216. Runtime XR / OpenXR / Device / Session / Stereo View / Tracking / Input / Late Update / Foveation / Compositor / Product Integration 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Manifest、public render contract与builtin module组合 | 281 / 50,983 / 1,781,220 | Cargo feature、camera/view contract、module composition和XR精确零搜索逐路径审查 |
| Frame/view、camera submission、history与multiview consumer | 167 / 32,745 / 1,268,812 | camera stack、ViewFamily、submission、history key及全部multiview assignment逐状态审查 |
| RHI、WGPU、device、surface与presentation | 94 / 24,822 / 855,380 | adapter/device request、capability、native surface、ordinary present与external image边界逐调用点审查 |
| App host、platform/input与Editor viewport product consumer | 143 / 10,436 / 359,322 | cadence、frame loop、device event、input action/gamepad和Editor viewport产品路径审查 |
| 聚焦external/inline-test-bearing文件 | 34 / 14,695 / 547,513 | camera/surface/native submission/render framework与input相关断言审查 |
| 去重合计 | **719 / 133,681 / 4,812,247** | Zircon fingerprint `36aa70c3d7c4ef5ea42b86a71034c6804533c05f1e999d0ed55cd720547f4c10`；冻结时101个入选路径dirty，实施前必须重核 |

当前camera snapshot、camera stack、`RenderViewExtract`、ViewFamily分辨率计划、temporal history、RHI capability、WGPU backend、window surface、input action和gamepad是真实底座，但均属于单观察点、普通window present和传统2D input模型。产品没有OpenXR loader/instance/system/session、graphics requirements/binding、runtime-owned swapchain lease、`wait/begin/locate/render/end`、per-view pose/FOV/slice/history、reference/action space、late update、composition layer、visibility mask、foveation、mirror、asset/editor/cook或conformance闭环。119处可识别`multiview_mask`赋值全部为`None`，WGPU backend还会在获得任何OpenXR graphics requirement前自行选择adapter。

本轮登记0项新增P0、72项P1、16项P2与48项资格门；目标是`XrRuntimeProvider + XrInstanceAuthority + XrSystemProfile + XrSessionSupervisor + XrFramePacer + XrSpaceGraph + XrActionRuntime + XrGraphicsBindingBridge + XrSwapchainLease + XrViewFamily + XrLateUpdateCoordinator + XrCompositionLayerGraph + XrExtensionRegistry + XrDiagnosticsReceipt`。Runtime09A/09B/09H1/24/37/42/56/57/58/65继续拥有RHI、visibility/history、identity、camera、composition、input、platform、bridge与quality父边界。本轮未修改production/tests/Cargo，也未运行真实OpenXR runtime、头显、GPU capture、conformance、latency、soak或benchmark。详见`zircon_runtime/66-runtime-xr-openxr-device-session-stereo-view-tracking-input-late-update-foveation-compositor-product-integration-review.md`。

## 217. Runtime Console Command / CVar Registry / Cheat / Exec / Config Layer / Replication / Remote / Product Integration 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Manifest、catalog、profile与capability truth | 315 / 15,559 / 555,597 | Cargo/profile/module/plugin capability、产品声明与console/CVar精确零搜索逐路径审查 |
| Runtime config、operation、diagnostics、dynamic/interface控制底座 | 68 / 10,982 / 368,562 | raw ConfigStore/Foundation persistence、operation lifecycle、devtools/dynamic API/interface逐合同审查 |
| App、Editor Console/command与产品consumer | 109 / 13,371 / 480,927 | startup args、log Console ZUI/output、Editor command/commandlet与host service接线审查 |
| 聚焦external/inline-test-bearing文件 | 31 / 5,194 / 176,294 | config/operation/console/command治理断言与零产品caller证据审查 |
| 去重合计 | **523 / 45,106 / 1,581,380** | Zircon fingerprint `e31d719d45fea5e679dd9bd99a7581f0f2787a0f314884fa2aa07ee57914ea34`；冻结时29个入选路径dirty，实施前必须重核 |

当前`ConfigStore`与Foundation manager能以任意字符串键/JSON值持久化整份map，`RuntimeOperationService`具有有界prepare/apply/progress/cancel/deadline底座，diagnostic log与Editor Console能过滤、虚拟化和清空输出，Editor也有独立command registry。但这些分别是raw config、长操作、日志pane与Editor action，不是runtime command/CVar authority。第一方production精确搜索只有Editor测试文本命中；App无`--exec`/CVar override，devtools无control snapshot，catalog/profile/plugin/script/headless/remote/replication均无接线。

五参考冻结32个文件、26,526行、930,695 bytes，SHA-256为`04dbdfa0efc60afefdd0f1f5d15636ac79bdbdb93950ac2c5f52438a55a0d91c`。本轮登记0项新增P0、72项P1、16项P2与48项资格门；目标是`RuntimeControlRegistry + ConsoleCommandCatalog + ConsoleVariableCatalog + ConsoleVariableLayerStack + ConsoleMutationTransaction + ConsoleApplyScheduler + ConsolePrincipalPolicy + ConsoleSession + ConsoleOutputStream + ConsoleHistoryStore + RemoteControlGateway + ConsoleDiagnosticsReceipt`。Runtime01/02/03/24/41/42/45/46/50/55/57/65、App01、Interface01/05/07、Editor08/25与Plugins01继续拥有共享父边界。本轮未修改production/tests/Cargo，也未运行产品、网络安全、复制、fault、soak或benchmark。详见`zircon_runtime/67-runtime-console-command-cvar-registry-cheat-exec-config-layer-replication-remote-product-integration-review.md`。

## 218. Runtime Sprite2D / Canvas2D / Sprite Atlas / TileSet / TileMap / Batching / Sorting / Lighting / Physics / Streaming / Product Integration 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Asset、Scene Component、RenderExtract与2D core contract | 57 / 14,011 / 533,042 | Sprite/Atlas/TileSet/TileMap schema、typed asset链、World extract、phase/sort与visibility逐字段审查 |
| Graphics renderer、pipeline、diagnostics与focused tests | 25 / 6,751 / 248,591 | CPU tessellation、batch key、material/pipeline语义、buffer/pass分配、统计与测试边界逐调用点审查 |
| Tilemap plugin、catalog/profile、App/Editor product boundary与manifest | 50 / 7,446 / 266,012 | Partial/DiagnosticOnly capability truth、产品preset、Editor owner、持久化与consumer closure逐路径审查 |
| 去重合计 | **132 / 28,208 / 1,047,645** | Zircon fingerprint `db7b26bc2c83c386fd3ad4115379275faf9559830747cfc111a261a73a2fe0b3`；冻结时8个入选路径dirty，实施前必须重核 |

当前Sprite2D路径能够真实提交像素，TileSet/TileMap也已进入`AssetKind`、typed marker、imported asset、cache payload和load facade，Tilemap插件明确报告`Partial`并用`DiagnosticOnlyAssetImporter`拒绝伪装Tiled后端。但Scene没有Canvas2D/CanvasLayer/Camera2D/Light2D authority；bounds不进入可见性，camera order/sorting layer/Y-sort被固定为零/None，material handle被renderer丢弃，Opaque/Mask/Transparent共用SrcAlpha且depth-write-off管线，每个batch每帧创建buffer并开启pass。SpriteAtlas不是runtime asset/artifact，Mesh2D无extract/render，TileMap没有component、chunk compiler、renderer、projection、dirty region、physics/navigation/occlusion、streaming或runtime mutation receipt。

五参考冻结69个文件、42,763行、1,622,957 bytes，SHA-256为`63f41fa3474bf3fa64c5765d394dbc9a95322f078ced3a32d03509d9745b763c`。本轮登记0项新增P0、72项P1、16项P2与48项资格门；目标是`Canvas2dWorldService + Canvas2dResourceAdapter + SpriteAtlasArtifact + SpriteAnimationProgram + Canvas2dSceneExtract + Canvas2dSpatialIndex + Canvas2dSortCompiler + Canvas2dBatchCompiler + Canvas2dGpuScene + Canvas2dRenderPipeline + Canvas2dLightingGraph + TileMapChunkStore + TileMapDerivedArtifact + Canvas2dPhysicsNavigationBridge + Canvas2dStreamingCoordinator + Canvas2dDiagnosticsReceipt`。Runtime61与Editor34继续拥有持久化丢失和2D authoring/schema硬阻断；本轮未修改production/tests/Cargo，也未运行产品、GPU、pixel golden、physics/navigation、fault、soak或benchmark。详见`zircon_runtime/68-runtime-sprite2d-canvas2d-sprite-atlas-tileset-tilemap-batching-sorting-lighting-physics-streaming-product-integration-review.md`。

## 219. Runtime Mesh3D / Static Mesh / Skeletal Mesh / Submesh / LOD / Instancing / Skinning / Morph / Collision / Streaming / Product Integration 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Asset、Scene、Animation、Render Framework与Physics contract | 62 / 8,350 / 286,546 | Mesh/Model/skin/morph/scene schema、project I/O、LOD selection、pose binding与collision DTO逐字段审查 |
| Graphics resource、GPU Scene、Mesh renderer与visibility | 180 / 40,626 / 1,512,870 | prepared generation、resource ensure、bounds、GPU Scene、draw build、skin/morph、command cache、indirect与culling逐调用点审查 |
| Focused asset/scene/graphics/diagnostic tests | 36 / 10,159 / 367,107 | validation/roundtrip、direct section、mesh cache、velocity/PNG、queue/GPU Scene断言与ignored边界审查 |
| 去重合计 | **278 / 59,135 / 2,166,523** | Zircon fingerprint `9afe296d977f277d02a7cf1e02e128e5dae5116e46c6df785ec4f47808463b4f`；483个test attribute、8 ignored；冻结时4个入选路径dirty |

当前typed Mesh attributes/index/morph、Scene primitive/material/LOD、真实GPU Scene current/previous history、cached command/indirect replay及skin/morph PNG测试是可保留底座。但Mesh/Model没有stable section/material slot/LOD artifact，Scene同时暴露Model/Mesh/primitive/LOD四类geometry authority，LOD只按节点translation距离选择；Prepared local bounds未进入当前可见性，产品每pending draw固定`instance_count = 1`，GPU skin/morph又预先执行CPU变形，morph delta与palette没有共享arena，resource prepare同步load/clone，collision mesh也没有产品cook链。

五参考冻结37个文件、50,833行、2,084,265 bytes，SHA-256为`71498c935f4a9085312a35aa9b5173421874a661bba719c54fce27c9aaf9539a`。本轮新增1项P0：Mesh material overrides、tint与alpha mode已参与reflection/extract/render，却不进入Scene asset或project I/O，save/reopen静默恢复默认；另登记48项P1、12项P2与48项资格门。目标是`MeshArtifactManifest + MeshGeometryArtifact + MeshDeformationArtifact + MeshDerivedArtifactSet + MeshInstanceComponent + MeshSceneDelta + PreparedMeshGeneration + MeshDeformationInstance + MeshSubmissionReceipt`。Runtime09B、Runtime64/09D、Runtime08A、Runtime08C/Editor32继续唯一拥有bounds/instancing、同步冷加载、physics cook与inverse-bind/skeleton安装父阻断。详见`zircon_runtime/69-runtime-mesh-static-mesh-skeletal-mesh-submesh-lod-instancing-skinning-morph-collision-streaming-product-integration-review.md`。

## 220. Runtime Scene Text / Text2D / Text3D / Billboard / Font / Layout / Localization / Extract / Render / Product Integration 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Font asset、Scene schema/component/project I/O、Editor入口 | 72 / 16,907 / 621,510 | asset、NodeKind、record、snapshot、reflection、property、extract与create/inspection逐字段审查 |
| 共享text/font/shaping/layout/raster/atlas | 286 / 63,068 / 2,156,067 | 可复用font、BiDi、shaping、layout、glyph artifact、SDF/MSDF、atlas、generation、cache与测试边界逐模块审查 |
| UI GPU text与Dynamic HUD consumer | 112 / 27,101 / 980,032 | UI batch identity、screen rect shader、无depth pipeline、product fallback与framebuffer测试逐调用点审查 |
| 去重合计 | **470 / 107,076 / 3,757,609** | Zircon fingerprint `c45fae6a5346516d080765015564da290e226e7806972291e4cb99041a0f1088`；1,164个test attribute、16 ignored；冻结时33个入选路径dirty |

当前Font asset/import/load、shared shaping/layout/glyph artifact及UI atlas/SDF/GPU framebuffer是真实基础，但它们只接到UI像素空间。NodeKind、SceneEntityAsset、SceneNode/Record、fixed snapshot、project I/O、reflection、property、render extract与Editor create/inspect均无Text2D/Text3D/SceneText；精确Scene/asset/graphics/editor类型检索为0。`ScreenSpaceUiTextBatch`身份是UI tree/node/source range，几何是UiFrame/clip frame，atlas instance只有`screen_rect_px`，shader直接写clip-space `z = 0`，UI/atlas/SDF pipeline均无depth，不能通过加transform冒充world text。

五参考冻结19个文件、9,405行、328,432 bytes，SHA-256为`155154a89f0d135ef34f9c10058f89730dbee46e4112077f8e5640c3f7c79fd1`。本轮登记0项新增P0、48项P1、12项P2与48项资格门；目标是`SceneTextSource + SceneTextStyleDescriptor + SceneTextSpatialPolicy + SceneTextComponent + SceneTextLayoutArtifact + SceneTextDeltaExtract + PreparedSceneTextGeneration + SceneTextViewProjection + SceneTextSubmissionReceipt + SceneTextAuthoringAdapter`。Runtime11B/11C、09B/09C/09D、43、61/62/63/64/65、Editor03/05/33、App06与Runtime09 UI bridge failure继续唯一拥有父问题。本轮未修改production/tests/Cargo，也未运行产品、GPU、pixel、fault、soak或benchmark。详见`zircon_runtime/70-runtime-scene-text-text2d-text3d-billboard-font-layout-localization-extract-render-product-integration-review.md`。

## 221. Runtime Scene Light / Directional / Point / Spot / Rect / Photometry / Layer / Shadow / Cookie / IES / Extract / Authoring / Product Integration 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Scene component、asset、store、project I/O、property | 27 / 9,606 / 370,668 | 默认值、组合不变量、save/reopen、reflection/compiled path逐字段核对 |
| Framework DTO、readiness、world extract | 8 / 2,901 / 102,436 | identity/layer/mobility/shadow、per-view扫描、sideband与降级语义 |
| Editor、tests、product fixture | 18 / 5,289 / 186,059 | create/inspect/gizmo、显式roundtrip、render fixture与异常/规模资格缺口 |
| 去重合计 | **53 / 17,796 / 659,163** | Zircon fingerprint `70b95e0afedbb10f285f3f37ec7a096b2680766b48543575f0971b7758b056b9`；80个test attribute；冻结时9个入选路径dirty |

五类light component、asset、typed ECS、project I/O、reflection、snapshot与Editor create/inspect是真实可保留底座，但当前仍没有工程级light product object。`SceneSpotLightAsset`缺字段时静默得到`+Y / 1,000,000 / 20 / 0 / 0`，与runtime默认`-Y / 8 / 12 / 0.3 / 0.55`冲突；`SceneEntityAsset`又允许五种light component共存，load只用优先级选择单一NodeKind却保留并渲染全部component。Direction/Spot忽略world rotation而Rect使用world forward；property/compiled path遗漏volumetric；extract按view全扫、分配、排序并硬编码shadow None。

五参考冻结31个文件、14,862行、593,772 bytes，SHA-256为`9718752e66737ede52fe7ae327ea99d7083d588806a6ca9066852fcee3126acb`。本轮登记2项新增P0、48项P1、12项P2与48项资格门；目标是`SceneLightCommon + SceneLightShape + SceneLightShadowSource + SceneLightTextureBindings + ValidatedSceneLightDescriptor + SceneLightMutation + SceneLightDeltaExtract + PreparedSceneLightGeneration + SceneLightSubmissionReceipt + SceneLightAuthoringAdapter`。Runtime09E继续唯一拥有cluster/shadow/area-light renderer父P0，Runtime61/62/63/64/65与Editor03/05/22保留共享父边界。本轮未修改production/tests/Cargo，也未运行产品、GPU、pixel、fault、soak或benchmark。详见`zircon_runtime/71-runtime-scene-light-directional-point-spot-rect-photometry-layer-shadow-cookie-ies-extract-authoring-product-integration-review.md`。

## 222. Runtime Core Lifecycle / Registry / Concurrency / Service Quiescence / Product Shutdown 当前源码物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Lifecycle vocabulary、descriptor、context、state、registration | 36 / 4,367 / 160,961 | graph freeze、注册事务、service identity、module/service state与callback surface |
| Activation、resolution、observer、runtime facade | 20 / 3,508 / 129,301 | single/batch transition、rollback、call admission、drain、unload、shutdown |
| Runtime behavior/structure tests | 46 / 8,556 / 357,947 | 115个test attribute；并发join、veto、rollback、reactivation与resolver行为 |
| App/Editor/Dynamic product owner | 7 / 3,494 / 129,277 | 6个test attribute；static bootstrap/drop、dynamic teardown/retry、FFI fail-stop |
| 去重合计 | **109 / 19,925 / 777,486** | Zircon fingerprint `44ab9cf76588998fb1c2011277facfc3b5c949255f1a857d155d85ab1169a42c`；131个test attribute；冻结时18个入选路径dirty |

当前Core graph validation、module/service拓扑、single dependency closure、batch rollback、observer veto原子性、generation-qualified `ServiceCallGuard`和dynamic session shutdown是真实可保留底座。但全局Condvar会对同一waiter重复计数并回放旧结果，single activation closure不是原子transition集合，service instance会在registry mutex内析构，batch取得部分token后遇错不会释放，static App/Editor只drop `CoreHandle`，声明图shutdown又会被未启动module提前中止。

五参考冻结12个文件、13,783行、518,624 bytes，SHA-256为`07c90c6448b988e0504d3807ffc60e1fa5c7c9ba02b2b58db4caff5e5e2e0f91`。本轮登记6项新增P0、18项P1、8项P2与40项资格门；目标是`FrozenRuntimePlan + LifecycleTransactionSet + ModuleRuntimeRecord + ActivationLedger + ServiceCallLease + ModuleLifecycleExecutor + RuntimeLifecycleEventStream + RuntimeShutdownReport + OwnedCoreRuntime`。Runtime46 lazy factory panic与Runtime50裸Arc resolver不重复计P0，Runtime01两个open failure缺少本轮Cargo证据并保持open。本轮未修改production/tests，也未运行Cargo、dynamic DLL、Editor、fault、soak、loom或benchmark。详见`zircon_runtime/72-runtime-core-lifecycle-registry-concurrency-shutdown-current-source-review.md`。

## 223. Plugin SDK Example / Native Dynamic Fixture / Editor Contribution Fixture / Test Carrier / Artifact Isolation / Product Truth 物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| 三个carrier根 | 18 / 1,708 / 65,063 | manifest、source/dist/native entry、registration、import framing、贡献声明与缺失资源逐文件核对 |
| 产品catalog/loader/manifest consumer | 20 / 5,508 / 208,569 | Editor build-generated builtin descriptor、manager phase、status/export、package schema与native replay |
| 聚焦tests/CI | 9 / 4,074 / 147,588 | source metadata test、真实fixture loader、workspace shape、Python regex gate与standalone dist matrix |
| 去重合计 | **47 / 11,290 / 421,220** | Zircon fingerprint `16f49498897d75469a036d8e7ad9db7569b12dd85d18a21b7843d80dd02c2f7a`；冻结时4个入选路径dirty |

三个carrier都使用正常第一方package identity与顶层`plugin.toml`。`zircon_editor/build.rs`按物理目录扫描所有manifest，只要声明editor module就生成builtin descriptor；`EditorPluginManager::builtin`随后推进Default phase，而schema没有Sample/TestFixture/DeveloperTool、hidden、explicit-load、shipping/configuration/architecture隔离。SDK sample缺少声明的asset/content roots、两个ZUI与settings，importer output和source/native/file贡献互相漂移；Native Fixture声明`write:world` system与event却没有可观察副作用/emit，negative ABI/export/capability变体又共用正常artifact identity；Editor Fixture的command必然DENIED，view/settings/asset只有metadata，Python gate也不构建或加载真实DLL。

Unreal、Fyrox、Bevy、Godot与Unity Graphics参考冻结18个文件、8,289行、322,642 bytes，SHA-256为`0e44deb7f9232f3fd4eaef433639954608a864fe52ddf0e310a1aca90b2e0041`。本轮登记3项P0、40项P1、10项P2与32项资格门；目标是`PluginPackageInventory + PackageRole + ProductPluginCatalog + ArtifactEligibilityDecision + ArtifactVariant + CarrierParityReceipt + NativeEditorOperationAdapter + PluginArtifactClosureReceipt`。本轮未修改production/tests/Cargo，也未运行Cargo、真实DLL、Editor UI、跨平台ABI、fuzz、fault、soak或shipping artifact scan。详见`zircon_plugins/20-plugin-sdk-example-native-editor-fixture-test-carrier-artifact-isolation-product-truth-review.md`。

## 224. Runtime UI Style / Theme / Token / Cascade / Selector / Pseudo-state / Invalidation / Transition / Product Integration 当前源码物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Public style/theme/V1/V2 schema | 6 / 1,705 / 56,378 | color/theme DTO、selector grammar、specificity、raw declaration/resolved maps、component scope |
| Compiler/cascade/component/surface integration | 36 / 12,938 / 450,195 | V1/MUI、V2 instancing/import/cache/static/runtime cascade、slot、token与surface metadata |
| Theme/reload/product/transition/Editor projection | 15 / 4,319 / 149,066 | gameplay/Editor builder差异、theme registry、reload receipt、transition descriptor与静态投影 |
| 聚焦测试 | 13 / 5,394 / 162,808 | scope parser/compile、prototype、file cache、theme、pseudo-state、inline override与transition metadata |
| 非测试 `.zui` 资产语料 | 297 / 49,773 / 3,012,860 | 1,017 selectors、149 pseudo selectors、2,376 token引用、132个非空style imports、214个component header |

当前tuple specificity、V2 static/runtime rule分离、runtime ancestor pseudo检测、token source metadata和Editor依赖加载是真实可保留底座。但V1 typed/MUI/V2是三套并行权威；默认Closed component scope、`:part()`与`:host`没有真实匹配；gameplay prototype-store路径不合并`imports.styles`或component-local stylesheet/token且没有theme owner；theme reload只替换registry并标dirty；Collapse/Fade/Grow/Slide/Zoom没有runner、clock或progress writer。

核心57个production/product文件的fingerprint为`f6549fa8040e551eaf0c7e453e2347ce57a0e7c8f7abb69b809d27c76f47779b`，13个聚焦测试为`fa2f8748cb3b502f1cc21f3d08c77fdd1ac43c6985e54558ac1d77fbf299cb8e`，297份非测试ZUI为`35811f294fddfa16d75401b4b4c58ed63d0a1483e5f7048b766c2075a776d511`。五参考实际冻结9个文件、8,948行、326,125 bytes，fingerprint为`d81c430cc54118b8d2d7acfefe9c19e804c24a80edfe5a58334a535a1988286d`。本轮登记5项P0、48项P1、12项P2与44项资格门；目标是`UiStyleSchemaRegistry + UiStyleDependencyGraph + CompiledSelectorProgram + CompiledUiStyleBundle + ComponentStyleBoundary + ComputedStyleArena + ThemeGeneration + UiSurfaceStyleBinding + UiStyleMutationTransaction + UiTransitionScheduler + UiCascadeDiagnostics`。本轮未修改production/tests/assets，也未运行Cargo、Editor、WOC、pixel、fault、soak或benchmark。详见`zircon_runtime/73-runtime-ui-style-theme-token-cascade-selector-pseudo-state-invalidation-transition-product-integration-review.md`。

## 225. Runtime UI Template / Component / Binding Expression / Model / Event / Command / Hot Reload / Product Integration 当前源码物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Public template/component/binding/event schema | 95 / 6,339 / 196,551 | AST、target、document、component contract、binding update/event DTO |
| Runtime compiler/component/router/reload | 154 / 29,953 / 1,029,287 | validation、prototype expansion、package/cache、reducers、event manager、reload |
| Surface execution与Editor交叉消费者 | 41 / 14,425 / 516,902 | action evaluator、component event matcher、control index、Editor adapter/projection |
| 聚焦测试 | 16 / 5,730 / 181,320 | validation/package/reload/router/component data binding/event dispatch/Editor adapter |
| Editor与WOC产品 `.zui` | 267 / 48,298 / 2,951,961 | 150份含events、1,429个event行、真实route/action命名与未使用target/expression能力 |

当前binding AST、component contract、prototype expansion、component reducer、surface mutation和reload plan是真实可保留底座。但`UiBindingTargetAssignment`可被验证/编译/打包却没有runtime target executor；`ParamRef`合法穿过compiler后在Runtime与Editor evaluator均返回`None`；component event靠CamelCase字符串子串猜测，而产品使用lower_snake route；重复component instance的`control.X.prop`读取全surface最小node id；reload executor只清cache/换theme/标dirty并回报rebuild target，不重建tree、迁移state或重绑subscription。

290个production/cross-product文件的fingerprint为`2a7ae05badf0f39629823862fdf39916dbfd4f54d9c99e3a122dadca64064774`，16个聚焦测试为`29c1cfff384020a2a9ed868a132cb0fb7797ee96b89f4c0727c3c0da5474e516`，267份产品ZUI为`4c1b8679da2f9734512b723c96f065efd831a593aadc3f58bda8183ad7f0e9b4`。五参考实际冻结21个文件、23,464行、841,107 bytes，fingerprint为`904502e5486169afba563f73e027893747f1b4d1e6d1985c5a62a9975c74aa04`；Unity Graphics仅作为serialized property/editor资产证据，不冒充Unity UI Toolkit runtime binding。登记5项P0、48项P1、12项P2与48项资格门；本轮未修改production/tests/assets，也未运行Cargo、Editor、WOC、fault、soak或benchmark。详见`zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md`。

## 226. Runtime UI Component Catalog / Widget Behavior / State Reducer / Interaction Semantics / Accessibility / Product Integration 当前源码物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Public component/widget contracts | 26 / 2,004 / 68,249 | descriptor、prop/state/slot/event schema、capability、fallback与widget contract |
| Runtime catalog/reducer/v2/surface/a11y/render | 189 / 45,363 / 1,639,983 | 两套catalog、20余个reducer、v2 admission、live behavior、mutation、render与a11y |
| Editor registry/palette/adapter/material painter | 232 / 14,094 / 510,221 | merged registry、showcase-only palette、local builtin、retained kind与painter分流 |
| 聚焦测试 | 69 / 17,374 / 619,616 | 107个catalog tests与179个widget behavior tests所在文件 |
| Editor与WOC产品 `.zui` | 282 / 46,657 / 3,131,228 | 267个Editor资产、15个WOC资产、component引用与definition交叉核对 |

当前showcase有69项descriptor、Material有211项，只重合22项并形成258个唯一id；Editor retained host将两者后注册覆盖式合并，palette与v2 interaction却仍只读showcase，Editor-local builtin和retained kind又形成额外owner。v2 compiler只拒绝空component id，不执行descriptor defaults/schema/slot/event/capability/fallback；tree builder把widget/a11y写成default。20余个state reducer在生产中只接Editor showcase demo，真实surface另走`UiWidgetBehavior + default_interactions + mutate_tree_property`；component event effect只检查node存在便回报delivered，live mutation和direct reducer又都允许unknown prop写入。输入、behavior、render、dirty与a11y各有字符串classifier，无法证明同一component拥有一致语义。

447个production/cross-product文件的fingerprint为`91c12aac743c24ca5586c33a077cb418f15a5d6605856513850868b3354bb018`，69个聚焦测试为`1edbb0d2f1bfee550a35ac1287fe43fa6ad6fed124c7134acd689e6e391aaccd`，282份产品ZUI为`0d529fa03d5d866b36101afd3f5ae477c80a3a649da9b76915b1bfdf346d4ba7`。五参考实际冻结21个文件、25,395行、1,097,043 bytes，fingerprint为`bd196e639e2ab70f1f63a6aaa470159feaa2f6ebdc40cb052db7effc6cd28886`；Unity Graphics仅作为DebugUI widget/value validation参考，不冒充Unity UI Toolkit。登记6项P0、48项P1、12项P2与48项资格门；本轮未修改production/tests/assets，也未运行Cargo、Editor、WOC、a11y平台、fault、soak或benchmark。详见`zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md`。

## 227. Runtime UI Layout / Box Model / Measure / Arrange / Flex / Grid / Overflow / Scroll / Virtualization / DPI / Product Integration 当前源码物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Public layout contracts | 12 / 1,855 / 52,728 | constraints、style、engine、geometry、metrics、scroll、slot与derived cache |
| Runtime layout execution | 46 / 11,923 / 416,858 | parser、measure/arrange、Taffy、incremental、scroll、pool、window与dynamic product |
| Editor product Rust consumers | 49 / 6,600 / 232,795 | CSS-like autolayout、virtual rows、view materialization与workbench layout |
| 聚焦测试 | 54 / 13,783 / 468,773 | 218项test、2项ignored；布局、Taffy、scroll、virtualization、dirty、DPI与pool |
| 真实产品 `.zui` | 48 / 14,910 / 854,682 | 36份Editor、12份WOC；Grid/Scroll/virtual collection与responsive消费者 |

当前`UiLayoutStyle`公开支持percentage、MinMax Grid、absolute inset、reverse flow和双轴overflow，Editor autolayout也能生成它；但retained tree不保存/消费该style，实际pass仍由`UiContainerKind + BoxConstraints`临时合成缩水style。Taffy每个container每次arrange新建parent+leaf浅树并立即丢弃，intrinsic content只作为预先算好的leaf尺寸；Grid强制等分/`fr(1)` tracks。`.zui` parser对columns/rows/span只验证非负，后续直接按track count分配数组并存在普通`usize`加法。backend selection只对preferred完整检查request，fallback仅检查family，Zircon capability又默认宣称全部family/content measure/DPI支持。

所谓virtualization仍先递归measure全部materialized children，再计算全部child positions，最后只清空视窗外frame；ScrollableBox固定extent window、Table/Tree字符串metadata reducer和node pool没有logical provider或同代materialize/recycle transaction。dynamic product初始硬编码1280x720，后续又把camera physical `UVec2`直接作为layout root size；window logical metrics只在局部测试/状态存在，未形成layout-render-hit geometry barrier。

107个production/cross-product文件fingerprint为`851ff50c4e5e575057e547339c18dae1f9f4d3c03527b4fc00f216c22ce2c2a3`，54个聚焦测试为`c4bba75f0d627f5b09a81423084f298c3d4d822f7c98f3981ed5d14f2184910b`，48份产品ZUI为`795c8a1238a205cbba62b54e6119989040907f0e6b2e5eebea97b58dedabc444`。五参考实际冻结21个文件、30,794行、1,095,023 bytes，fingerprint为`7ba80d528f7a1096c07e45cd672ba3871878e6b3426cf657efae53b2d208c51a`；Unity Graphics仅作为DebugUI observable hierarchy/dirty/container invariant参考，不冒充UI Toolkit布局源码。登记3项新增P0、48项P1、12项P2与48项资格门；本轮未修改production/tests/assets，也未运行Cargo、Editor、WOC、real window、fuzz、fault、soak或benchmark。详见`zircon_runtime/76-runtime-ui-layout-box-model-measure-arrange-flex-grid-overflow-scroll-virtualization-dpi-product-integration-review.md`。

## 228. Runtime UI Input / Dispatch / Routing / Focus / Navigation / Pointer Capture / Gesture / Drag-Drop / IME / Window Lifecycle / Product Integration 当前源码物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Public input contracts | 34 / 4,049 / 128,430 | dispatch、window、focus、navigation、a11y合同；6项内联测试 |
| Runtime execution | 123 / 18,611 / 640,793 | dispatch、platform input、surface input/focus、hit、a11y和dynamic session；44项内联测试 |
| Product consumers | 25 / 3,415 / 123,466 | Editor Winit/retained shell与WOC touch/HUD；48项内联测试 |
| Production union | 182 / 26,075 / 892,689 | 98项内联测试，0项ignored |
| 聚焦测试 | 96 / 30,613 / 1,063,518 | 460项test，0项ignored；routing、focus/nav、ownership、reply、window、IME和a11y |

当前`UiInputEvent`、`UiDispatchEffect`、preview/bubble、focus scope/restore、tab/spatial navigation、capture、drag state、IME surrounding text和AccessKit中性转换是真实底座；但input metadata的user/device/window/surface/pointer/timestamp均可缺省，Dynamic Runtime把非pointer事件依次克隆到多个surface，Editor绕过Runtime window pump和`UiInputManager`，WOC又局部实现long press/double tap/pinch。Winit wheel translation落在`(0,0)`，Editor再靠last pointer位置补偿；Dynamic file drag不进入UI，通用UI tick和IME host request drain也没有形成production caller链。

新增P0是独立的effect原子性故障：`apply_dispatch_reply_core`顺序修改focus、capture、drag和component状态，尾项拒绝时不回滚已提交前缀；drag Begin/Complete内部又包含多步可失败mutation，result没有transaction generation、commit outcome、host ack或compensation receipt。multi-surface arbitration、bool-only产品返回、secure text/IME、UI吞physical state、component假投递、DPI geometry和Editor完整IME丢失仍分别由Runtime11A、11B、56、75、76与Editor01拥有，不重复计数。

182个production/cross-product文件fingerprint为`693bdf3f437d18a69d888c740991fd249f0fde3ebb74e450541099ad6dc9ed8e`，96个聚焦测试为`8d1389c4a57d49ca6a1c182bf93c1edb57aaec53e31cbe922dce5412c42dcd4a`。五参考实际冻结23个文件、49,605行、1,838,600 bytes，fingerprint为`d232693ad4a926afe37386bdb7cb4bfbe13a67803681727bb09b97db46837c94`；Unity Graphics仅作为Debug UI input action/lifecycle参考，不冒充Unity UI Toolkit。登记1项新增P0、48项P1、12项P2与48项资格门；本轮未修改production/tests/assets，也未运行Cargo、Editor、WOC、real window、fuzz、fault、soak或benchmark。详见`zircon_runtime/77-runtime-ui-input-dispatch-routing-focus-navigation-pointer-capture-gesture-drag-drop-ime-window-lifecycle-product-integration-review.md`。

## 229. Runtime UI Accessibility / Semantic Tree / Name / Description / Relation / State / Action / Live Region / Platform Adapter / Product Integration 当前源码物理范围

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Public contracts | 14 / 3,685 / 117,725 | 中立tree/state/relation/action与dispatch DTO |
| Runtime execution | 58 / 10,344 / 362,174 | extract、validate、AccessKit、action、surface与Dynamic API；10项inline test |
| Product ZUI/reachability | 320 / 55,847 / 3,218,769 | 317份tracked ZUI、5,594个node section，零显式a11y/relation authoring |
| Production union | 392 / 69,876 / 3,698,668 | App/Editor/WOC无production AccessKit adapter consumer |
| 聚焦测试 | 21 / 8,387 / 296,580 | 123项test、0 ignored；name、relation、action、AccessKit与Dynamic ABI |

当前`UiAccessibilityTreeSnapshot`、多阶段extract、名称/description解析、relation diagnostics、分模块action dispatcher与AccessKit converter是真实底座；但snapshot没有window/surface/generation/locale identity，action没有tree/request/user/window generation，`generation_hint`不消费，`source`不进入执行，typed action result降为notes/string/bool。relation只有单个`labelled_by`/`label_for`，description用`#id`魔法引用，cycle validator只识别二节点环，tree validator又不校验dangling child、multiple parent、orphan和root reachability。

产品证据更直接：317份tracked `.zui`与5,594个`[nodes.*]`中没有一行显式`a11y =`、`labelled_by =`或`label_for =`；语义全部依赖component/text/tooltip启发式。`accesskit.rs`没有production `accesskit_winit::Adapter` consumer；converter使用可碰撞的`u64::MAX` synthetic root，丢`pressed`与outbound selection，把无payload的`ScrollIntoView`映射到必需payload的Runtime `ScrollTo`，并把Dismiss/Blur/HideTooltip混成不等价动作。Dynamic JSON capture/action是唯一产品可达面，capture还会先resize viewport，action与capture不绑定同代publication。

392个production/cross-product文件fingerprint为`db4504abdbe7c9e2c2db19193fa695dbd15ece07adcdfcd12b734ece808d9576`，21个聚焦测试为`cbdd167710a7c887e4dca92f19b2e8ef3e578d950265c1adbd442697ddc6dfa3`。五参考冻结25个文件、19,473行、757,814 bytes，fingerprint为`55772c9ac14ab8c818331064b82b9ba0a55d642271abab9fecbe9f3a69e24f39`；Fyrox本地snapshot未找到原生a11y adapter，Unity Graphics也不含UI Toolkit accessibility，均已明确限制。登记0项新增P0、48项P1、12项P2与48项资格门；本轮未修改production/tests/assets，也未运行Cargo或真实screen reader/fault/soak/benchmark。详见`zircon_runtime/78-runtime-ui-accessibility-semantic-tree-name-description-relation-state-action-live-region-platform-adapter-product-integration-review.md`。

## 230. 后续扫描队列

Runtime UI Accessibility/Semantic Tree/Relation/Action/Live Region/Platform Adapter纵切面已经完成首轮E3；后续保持Runtime11A/11B/43/75/76/77与Editor23的既有P0及父owner不重复计数，按本篇建立typed semantic source、compiled name/relation graph、rich state/text/range/collection/live facets、incremental publication、per-window adapter、generation-bound action transaction和真实AT资格。不得用JSON snapshot冒充产品publication、用converter单测冒充screen reader adapter、用interactive→Button启发式冒充authoring、用full TreeUpdate冒充增量树，或用bool/notes冒充action receipt。

Runtime UI Input/Dispatch/Routing/Focus/Navigation/Capture/Gesture/Drag-Drop/IME/Window Lifecycle纵切面已经完成首轮E3；后续先以fault injection关闭effect前缀partial commit这一新增P0，再按Runtime11A/11B/43/56/75/76及Editor01/23边界建立qualified input envelope、单一window/router、atomic effect transaction、per-seat focus/capture/gesture、typed drag/host result、per-window IME/a11y、teardown和统一Dynamic/Editor/WOC产品receipt。不得用effects applied列表冒充原子提交、用surface广播冒充focus arbitration、用WOC局部touch算术冒充gesture arena、用bool handled冒充产品receipt或用中性AccessKit tree冒充平台adapter。

Runtime UI Layout/Measure/Arrange/Flex/Grid/Scroll/Virtualization/DPI纵切面已经完成首轮E3；后续先关闭track/span无界分配、public style与产品tree断链、backend fallback假能力三项新增P0，并继承Runtime11A的window DPI/product owner与tree/hit safety父问题；再按Runtime11A/11B/11C/43/73/74/75及Editor01/23边界建立bounded compiled layout artifact、persistent full backend graph、constraint-aware measure、真实virtual collection、logical/physical geometry barrier和产品scale qualification。不得用每容器临时Taffy浅树冒充layout engine、用隐藏frame冒充virtualization、用metadata visible range冒充data provider、用backend名称冒充capability或用physical viewport直接冒充logical layout size。

Runtime UI Component Catalog/Widget Behavior纵切面已经完成首轮E3；后续先关闭多authority、v2 admission绕过、reducer/live surface断链、字符串classifier分裂、schema外mutation和component-specific a11y丢失六项P0，再按Runtime11A/43/64/73/74及Editor01/23父边界建立provider-qualified component authority、typed implementation/facet、schema-safe state transaction、controlled proposal/commit、render/a11y adapter、collection model与真实Editor/WOC conformance。不得继续添加component字符串分支、用catalog inventory冒充implementation、用node存在冒充event delivered、用generic painter/interactive→Button冒充fallback或用showcase adapter冒充产品执行链。

Runtime UI Template/Component/Binding纵切面已经完成首轮E3；后续先关闭target假能力、`ParamRef`静默失效、字符串猜event、重复实例串读和reload假成功五项P0，再按Runtime11A/43/64/73及Editor01/23父边界建立typed schema/endpoint/event/action、compiled binding artifact、instance scope、batch mutation transaction、model subscription、command admission、state migration与generation-qualified rebind。不得继续按名称token扩展事件、在input callback逐次parse expression、以package enum冒充compiled section、以dirty flag冒充reload或用Editor adapter冒充gameplay provider。

Runtime UI Style/Theme/Token/Cascade/Selector/Pseudo-state/Invalidation/Transition纵切面已经完成首轮E3；后续先关闭component scope/part/host假合同、gameplay style import失效、gameplay theme owner缺失、theme reload假成功和transition假能力五项P0，再按Runtime11A/11B/11C/43/64、Editor01/23及Tooling32父边界建立typed schema、compiled style bundle、component boundary、computed arena、theme generation、dependency index、transition scheduler和真实WOC/Editor parity资格。不得继续flat extend依赖、按value猜inline origin、以dirty flag冒充restyle、以静态progress冒充动画或让renderer解释任意字符串map。

Plugin SDK Example、Native Dynamic Fixture与Editor Contribution Fixture carrier纵向首轮E3已经完成；后续先关闭物理目录扫描自动提升fixture/sample、SDK example无可兑现交付形态、negative native fixture进入正常dist/export/shipping三项P0，再按Plugins01/06、Editor06/50、Runtime58与Tooling BuildSet/Artifact边界建立强类型PackageRole、显式product catalog、artifact eligibility/variant、真实native editor adapter、carrier parity与最终artifact exclusion。不得以`maturity=experimental`、category、空descriptor、metadata-only registration或source regex冒充隔离与行为资格。

Runtime Core Lifecycle当前源码纵向复审已经完成；后续先以deterministic model关闭waiter重复登记、dependency closure/unload竞态、锁内析构、batch token泄漏、static owner shutdown缺失和partial activation shutdown中止六项P0，再按Runtime01/24/42/43/46/50、App01与Tooling24/35/37边界建立owned runtime、activation ledger、RAII transaction set、canonical call lease、锁外retire、deadline/report和产品资格。不得用sleep/timeout扩大、`Arc::strong_count`、`CoreHandle::Drop`、声明图逆序或旧结果重放掩盖所有权与线性化错误。

Runtime Scene Light纵向首轮E3已经完成；后续先关闭Spot partial/default migration与单NodeKind隐藏多light component两项P0，再按Runtime09E、61/62/63/64/65及Editor03/05/22父边界建立versioned schema、canonical defaults、photometry/shape/transform authority、shadow/cookie/IES dependency、change-driven extract、unit-aware Inspector、完整handles和产品资格。不得只改默认常量、只在Editor菜单阻止非法组合、以裸float标签冒充物理单位，或复制Runtime09E的cluster/shadow owner。

Runtime Scene Text/Text2D/Text3D/Billboard纵向首轮E3已经完成；后续先冻结consumer-neutral Font/Text artifact、SceneText component/schema/spatial policy和Editor create/save/reopen合同，再按Runtime11B/11C、09B/09C/09D、61/62/63/64/65、Editor03/05/33父边界建立change-driven layout、world bounds、Scene extract、depth/material/visibility、多view、persistent glyph instance和产品资格。不得把`ScreenSpaceUiTextBatch`加transform冒充Text3D，不得把world label塞进HUD overlay绕过depth/picking，也不得复制font database、shaper或atlas。

Runtime Mesh3D/Static Mesh/Skeletal Mesh/Submesh/LOD/Instancing/Skinning/Morph/Collision/Streaming纵向首轮E3已经完成；后续先关闭Mesh material override/tint/alpha mode的Scene保存丢失，再由Editor32与Runtime asset pipeline冻结canonical Mesh artifact、stable section/skin/morph/LOD identity，按Runtime09B/09D/64/08A/08C/24/61/62/65父边界建立qualified bounds、true instancing、deformation arena、async residency、collision cook和产品资格。不得以multi-draw冒充instancing、以CPU预变形冒充GPU skin/morph、以inline primitive silent fallback冒充ready。

Runtime Sprite2D/Canvas2D/Sprite Atlas/TileSet/TileMap纵向首轮E3已经完成；后续先由Runtime61与Editor34关闭canonical persistence、Play/Save守恒、formal asset/schema与authoring硬阻断，再按Runtime08A/08D/09A-09E/24/37/42/56/62/64/65、App01、Plugins01/08父边界建立Canvas authority、atlas/artifact generation、bounds/spatial extract、稳定排序、material-aware batch、persistent GPU allocation、tile chunk/derived artifact、lighting/physics/navigation/streaming和产品资格，并随source drift复核。不得以单Sprite draw path、inline UV、dense TileMap DTO或Partial/DiagnosticOnly插件冒充完整2D引擎。

1. Runtime systems：Physics、Audio、Animation、Navigation、Network、AI Behavior Tree/Blackboard/Perception与Gameplay Ability/Effect/Attribute/Tag/Cue/Prediction已完成首轮E3静态审查；script/plugin公共控制面、Zr parser/type system/SemIR/bytecode/compiler artifact/VM runtime、Time/Clock Domain/Fixed Step/Determinism/RNG/Replay/Scheduling、Coordinate Space/Unit/Precision/Transform/Numeric Robustness/Large World、Stable Identity/Handle/Generation/Owner Epoch/Stale Reference/Exhaustion、Filesystem/Path/URI/VFS/Mount/Watch/Sandbox/Atomic I/O、Particle/VFX System/CPU-GPU Simulation/Rendering/Scalability/Determinism/Product Integration、Cloth/Fabric/Soft Body/Garment simulation/collision、Hair/Groom source/binding/simulation/cache、Destruction/Fracture source/compiler/clustered physics/damage/field/cache、Vegetation species/compiler/prototype/instance-set/wind/interaction，Decal material/projector/instance/lifecycle、Weather program/World authority/transition/region/wind/precipitation/surface/network/save、Camera source/program/director/evaluator/player-view ownership/cut/history/network/save、Gameplay Framework/Game Instance/World Context/Level registry/Experience/Game Rule/Player/Controller/Pawn/Possession/Travel，以及Prefab unique importer/compiler/artifact/Instance Registry/provenance/rebase/update/streaming/network/save已完成首轮E3，后续随implementation与source drift复核。
   SaveGame/Checkpoint/Slot/Participant/Capture/Migration/Platform/Cloud运行时也已完成首轮E3；后续按Editor24、Runtime05/12/24/38、App03/05/06与Interface02父owner关闭service、无损capture、durable storage和产品资格，并随source drift复核。
   Operation Service/Registry/Admission/Prepare/Apply/Progress/Cancel/Shutdown运行时也已完成首轮E3；后续按Runtime02/24、Editor09/19、Interface01/05、App01与Tooling37父owner关闭identity、scheduler、cancel/progress/wake、effect receipt和session-owned drain，并随source drift复核。
   Builtin Runtime Module/Catalog/Profile/Target/Feature/Extension Assembly运行时也已完成首轮E3；后续按Plugins06/01、App01、Runtime01/07/21父owner关闭单一composition plan、effective selection、唯一provider、capability admission、稳定schema、activation transaction与final receipt，并随source drift复核。
   Dynamic Runtime Session/Registry/FFI/Frame/Event/UI/World Sync/Shader Prewarm运行时也已完成首轮E3；后续按Interface01/03/05、Runtime01/02/05/07/09A/09C/11A/22/24/41/42、App01/06与Plugins05父owner关闭generation identity、host scope/budget、bounded lifecycle、typed disposition、多viewport/platform、event/wake/page receipt、产品UI隔离和真实shader cache，并随source drift复核。
   Process Diagnostic Log Router/Filter/Record/Queue/Sink/Durability/Rotation/Crash/Multi-Session运行时也已完成首轮E3；后续按Runtime03/07/43、App01、Editor11/25、Interface01/03/04与Plugins01父owner关闭单进程router、结构化record、字节预算、公平admission、sink隔离、flush fence、rotation/retention、crash artifact和统一产品日志事实，并随source drift复核。
   Preference/Settings/Scope/Storage/Overlay/Bounded I/O/Generation/Fence/Durability/Migration/Multi-Process运行时也已完成首轮E3；后续按Runtime03/25/40、Runtime01/02/42、App01/03/04与Editor12父owner关闭product/principal identity、schema/revision、真实durability receipt、hung-I/O bounded teardown、multi-process CAS/watch、reactive read/terminal harvest和单一持久化authority，并随source drift复核。
   Engine Module/Service Contract/Context/Factory/Descriptor Snapshot/Composition/Lifecycle运行时也已完成首轮E3；后续按Runtime01/07/24/42/45、App01与Plugins01父owner先独立关闭lazy factory panic slot P0，再收敛single proposal/compiled graph、Runtime→App→Core同代snapshot、typed binding/context、reload policy和规模资格，并保持descriptor regeneration failure为open直至端到端验收通过。
   Runtime-wide State/NextState/Transition/Hook/History/Schedule/Scope运行时也已完成首轮E3；后续按Runtime01/02/03/05/07/22/24/38/41/46、Tooling35与PERF-MVP-320父owner关闭runtime-owned service、显式scope/descriptor、deterministic request admission、bounded cursor journal、subscription generation/quiescence和真实产品接线，并随source drift复核。
   Runtime Debug Gizmo/Command Buffer/Retained/Extract/View Filter/Budget/Render运行时也已完成首轮E3；后续按PERF-MVP-333、Runtime09A/09B/23/24/47、Editor03、Runtime08D/08F与Plugins14/15父owner关闭唯一debug-draw service、配置消费、qualified producer/view identity、retained lifetime、正确transform、compiled extract、预算诊断、persistent GPU arena和真实产品接线，并随source drift复核。
   Runtime Asset Registry/Index/Persistence/Rebuild/Incremental/Query运行时也已完成首轮E3；后续先关闭duplicate GUID reference closure migration与durability-before-publication两项P0，再按Runtime04/24/25、Editor04、Tooling37与PERF-MVP-556父owner收敛exact row schema、current persistence、deterministic scan、single incremental authority、indexed query和同代产品发布，并随source drift复核。
   Dynamic Scene Session Archive/Slot/Capture-Restore/Path/Merge/Retention/Durability运行时也已完成首轮E3；后续按Runtime04/05/22/24/25/40/41父owner先收敛唯一产品service、platform/principal store、CAS/journal/fsync/recovery与restore transaction，再关闭schema/provider migration、bounded parse、query index、three-way merge、retention/GC和facade hard cut，并随source drift复核。
   Dynamic Scene Asset Reload/Event Generation/Reconciliation/Stage-Apply/Instance Replacement运行时也已完成首轮E3；后续先关闭跨project/world/Level串扰、append-without-replace/remove和revision-before-admission silent-drop三项P0，再按Runtime04/05/24/39/41/43/51、Editor03/04/07与Tooling37父owner建立qualified instance registry、last-known-good、replace/repair/publish事务、terminal journal、bounded shutdown和产品故障/规模证据，并随source drift复核。
   Scene Event Mirror/Registration/Subscription/Cursor/Backlog/Overflow/Reclaim/ABI/Consumer运行时也已完成首轮E3；后续先关闭AI product exposure断链、decode/apply前commit且无gap/resync、per-subscriber encode/64 MiB queue三项P0，再按Runtime02/05/43、Interface01/07、Editor47与Plugins01/12/14/15父owner建立shared stream、acknowledged cursor、global budget、provider generation、product real-ABI与fault/scale资格，并随source drift复核。
   Foundation Module/Config/Event/Service/Driver/Manager/Persistence/Lifecycle运行时也已完成首轮E3；后续先关闭启动层覆盖与全Store durable projection污染、同进程多Runtime同路径静默supersede、空driver与零产品consumer能力伪绿三项P0，再按Runtime01/02/03/25/42/43/45/46/50与App01父owner建立compiled foundation contract、layered config authority、typed event service、scoped persistence broker及真实产品资格，并随source drift复核。
   Input Device/Event/Frame State/Action Map/Focus/Gamepad/Recording/Replay/Host运行时也已完成首轮E3；后续先关闭空driver与零Action/Replay产品consumer伪Ready、UI早退删除物理事实造成卡键、临时gamepad ID与无版本键盘码持久化三项P0，再按Runtime06/11A/24/38/43/46/50、Interface01/03/07、App01、Tooling32与PERF-MVP-012/426父owner建立qualified ingress、ownership arbitration、player-scoped compiled action、stable binding、deterministic journal与真实产品资格，并随source drift复核。
   Platform Host/Window Registry/Monitor/Display/Event Loop/Application Lifecycle/Surface Command运行时也已完成首轮E3；后续先关闭disabled/未安装backend仍报Supported及Destroyed/suspend不撤销unsafe surface两项本地P0，并把Tooling03的`TOOL-EXPORT-P0-005`保留为继承发布阻断；再按Runtime06/09A/11A/24/43/45/46/50/56、App01、Interface01/05/07与Tooling16父owner建立qualified host、window/display generation、lifecycle machine、surface lease、command receipt和真实平台资格。
   Runtime Plugin Interface Bridge/Slot/Generation/Strong-Weak/Native-VM/Lifecycle/Diagnostics运行时也已完成首轮E3；后续先关闭无call lease、native registration replay无产品caller、旧World scope静默停跑且阻止DLL retirement三项P0，再按Runtime01/05/07/24/42/46、Plugins01/12、Interface01与Tooling35/36/37父owner建立typed contract、transactional generation、holder census、World safe-point replacement、VM同代dispatch和revisioned diagnostics，并随source drift复核。
   Runtime Task Execution/Job Scheduler/Handle/Dependency/Cancellation/Thread Budget/Timer/Shutdown/Diagnostics运行时也已完成首轮E3；后续保持Runtime02与Editor09两项P0的唯一owner，按Runtime01/02/11/24/40/41/43/46、Editor09、Tooling24/35及Runtime11 implementation plan建立execution service、typed task、scope cancellation、dependency graph、全进程线程预算、deadline/timer、bounded shutdown和产品诊断，并随source drift复核。
   Runtime Scene ECS/Entity/Component/Storage/Archetype/Query/Access/Change Detection/Command/Schedule/Parallel/Event运行时也已完成首轮E3；后续先关闭Query共享receiver制造并存可变别名及RemovedComponentEvents永久累积两项P0，再按Runtime03/05/08/24与本篇owner建立World/schema identity、proof-carrying WorldCell、真实World system并行、统一dependency/access DAG、typed DeferredWorld和有界event/observer lifecycle，并随source drift复核。
   Runtime Scene World/Level/Registry/Lifecycle/Project I/O/Snapshot/Clone/Serialization Schema/Transaction运行时也已完成首轮E3；后续先以Save/reopen、Enter/Exit Play和exact runtime fork数据守恒测试关闭五项P0，再按Runtime05/24/39/40/52/53/60与本篇owner建立WorldContext/Level registry、lifecycle participant、versioned authoring document、schema-driven entity remap、bounded persistence operation、CAS/journal/durability和多World产品资格，并随source drift复核。
   Runtime Scene Hierarchy/Transform/Reparent/Activation/Mobility/Bounds与Scene Reflection/Type Schema/Dynamic Component/Property Address/Inspection/Subscription运行时也已完成首轮E3；后续先关闭protected/derived component authority两项P0及reflection catalog半提交P0，再按Runtime24/54/60/61/62、Interface02与Editor05父owner建立typed scene mutation、stable schema/address、同代inspection publication、cursor/resync和Editor stale-revision拒绝，并随source drift复核。
   Runtime Resource Authority/Asset Handle/Load Request/Version Lease/Cache/Dependency/Reload/Cancellation运行时也已完成首轮E3；后续先关闭错误exact payload进入authoritative store及帧关键路径同步冷加载两项P0，再按Runtime04/09D/24/51/54/59、Editor04与Plugins07父owner建立qualified handle、async request、version lease、typed dependency、transactional reload、同代publication与产品级fault/scale/performance资格，并随source drift复核。
2. Graphics/UI：RHI、Render Graph、GPU lifetime、renderer/visibility/GPU Scene、material/shader/pipeline/PSO、render asset streaming/residency、direct lighting/clustered light grid/shadow、environment/sky/IBL/reflection probe、baked lighting/lightmap/irradiance volume/offline bake、Hybrid GI、Volumetric Fog/Froxel、advanced surface lighting、temporal AA/velocity/history/upscaling、exposure/color/bloom/DOF/motion blur/SSR/terminal composition、SSAO/GTAO/denoise/temporal/depth-normal integration/scalability、Hardware Ray Tracing/BLAS/TLAS/Ray Query/Pipeline/SBT/denoise/scalability、Terrain/Landscape/Heightfield/Quadtree LOD/Material Layer/Foliage/World Partition/Physics/Navigation/scalability、Water/Ocean/Lake/River Surface/Wave/FFT/Shallow Water/Underwater/Buoyancy/Query/scalability、Cloth deformation/Fabric shading/dynamic bounds/velocity/RT fallback、Hair strand/cards/meshes visibility/lighting/deep shadow/velocity/RT/LOD/streaming、Destruction piece transform/visibility/interior material/shadow/velocity/RT/LOD、Vegetation instance cluster/mesh-card-billboard-impostor LOD/wind deformation/thin-leaf shading/shadow/velocity/GI/RT/streaming、Decal projection/receiver/cull/batch/DBuffer/GBuffer/forward/atlas/temporal/RT、Weather atmosphere/cloud/IBL/fog/wind/precipitation/surface/multi-view adapters、Camera endpoint/view family/cut/history key/retirement/split-screen/XR、runtime UI architecture/tree/layout/input/accessibility、runtime text/font/shaping/layout/editing/IME，以及GPU UI renderer/atlas/SDF/batch/clip/submit已完成首轮E3静态审查。
3. Host/ABI/Plugin：`zircon_app`产品host/bootstrap/runtime-library/runtime-entry、PBR viewer/evidence链、WOC多角色产品/VM/state/client-server集成、native client window/input/settings/shell/presentation、native server/bot/headless service/commit/operations、Vampire产品样例source/gameplay/artifact/evidence及Renderable Empty template/create/import/render/export/evidence，`zircon_runtime_interface` stable runtime DLL ABI/FFI/ownership、serialization/resource/reflection/world-sync、UI、profiling/plugin-event/script-diagnostic/native manifest、`zircon_runtime_host` safe owner/admission/fuse/output policy、project manifest/session/Hub/focus/recent protocol及contract certification/ABI layout/version skew/cross-language/fuzz七轮，`zircon_plugins` SDK/package/catalog/dist/native admission纵向边界、Editor plugin UX/authoring、Neural模型/ONNX/CPU-GPU推理/Post Process、Desktop Export/Native Window Hosting、Rendering umbrella/15个feature bundle/Solari、Shader WGSL/family importer/compiler/native产品链、39包source/editor/runtime/dist catalog/profile/capability closure、first-party Asset Importer source/dependency/subasset/artifact/sandbox/product integration、九个first-party Editor authoring extension的document/operation/toolkit/runtime contract，Particles、Network、Sound、Physics、Animation、Navigation、AI、Zr VM Language、Virtual Geometry、Texture与Hybrid GI的source/runtime/editor/dist/catalog纵向链，以及Plugin SDK Example、Native Dynamic Fixture、Editor Contribution Fixture的carrier/artifact isolation/product truth均已完成首轮E3静态审查；WOC package kernel/world storage/fixed schedule/serialization、combat/casting/effect/aura/damage/threat/death、progression/inventory/item/economy/crafting/quest/talent、social identity/party/chat/arena/matchmaking/minigame、instance/dungeon/delve/pet/companion、world/terrain/collision/locomotion/spawn/spatial/targeting、content/generated/BuildSet/install/query、command protocol/payload codec/admission/movement/outcome、package root/World API facet/snapshot/publication及source oracle/trace/golden/differential parity内部审查也已完成；已确认`examples`只有WOC与Vampire两个tracked产品域且均完成首轮，下一步继续逐包核对剩余first-party plugin业务真实性、未映射artifact/Marketplace/第三方/non-Cargo surface与仓库其余物理域，并保持全局owner/schema/ABI总账增量更新。
   Sound单包的manifest/source runtime、ray/timeline feature、Editor、dist、catalog、mixer/spatial/reverb/timeline产品链也已完成首轮E3；后续按Runtime08B、Editor17、Plugins01/06/07及既有Kira failure关闭基础owner与产品资格，并随source drift复核。
   Physics单包的manifest/source runtime、Editor、dist、catalog、simulation/collision/joint/ragdoll产品链也已完成首轮E3；后续按Runtime08A、Editor18、Runtime22/24/42、Plugins01/06及开放overlay failure关闭基础owner与产品资格，并随source drift复核。
   Animation单包的manifest/source runtime、fallback、Editor、dist、catalog、skeleton/clip/pose/graph/state-machine/IK/skinning产品链也已完成首轮E3；后续按Runtime08C、Editor14、Runtime22/24/42、Plugins01/06/07/08及三份开放failure关闭唯一provider、artifact、phase、consumer与产品资格，并随source drift复核。
   Navigation单包的manifest/source/native runtime、fallback、Editor、dist、catalog、Recast/Detour/Crowd/TileCache/query/bake产品链也已完成首轮E3；后续按Runtime08D、Editor19、Runtime22/24/42、Plugins01/06/07及五份开放failure关闭唯一provider、真实geometry/artifact、per-world generation、movement authority、carrier parity与产品资格，并随source drift复核。
   AI单包的manifest/source runtime、Editor、dist、catalog、Behavior Tree/Blackboard/Perception/EQS与Vampire asset产品链也已完成首轮E3；后续按Runtime08F、Editor20、Runtime22/24/42、Plugins01/06/07关闭唯一provider、source/compiler/artifact、per-world generation、budgeted scheduler、Perception/EQS、carrier parity与产品资格，并随source drift复核。
4. Editor/Hub：Editor retained UI、document/transaction/save/autosave/recovery、scene/prefab/selection/mode/gizmo/picking、asset index/import/reimport/catalog/thumbnail/reference workflow、Inspector/property authoring、Plugin Manager/live reload/settings、Play/PIE/Game View/live edit/recovery、Command Registry/keymap/menu/palette/context/remote automation、Background Jobs/admission/scheduling/cancellation/progress/shutdown、Notification Center/toast/decision/history/action/retention/accessibility、Logging/diagnostic journal/output console/status routing/retention/export、Settings/Preferences/scope persistence/locale/i18n/appearance/plugin extensibility、Layout Profile/Workspace State/dock-tab-window restore/schema migration、Animation Sequence/Graph/State Machine/Timeline/Curve/Preview/Compiler Authoring、Material/Shader Graph/Instance/VFX/Particle/Preview/Compiler Authoring、Terrain/Landscape/Foliage/Scatter/World Partition/Level Streaming Authoring、Sound/Audio Clip/Mixer/Spatial/Acoustic/Timeline Authoring、Physics Material/RigidBody/Collider/Joint/Collision Cook/Ragdoll/Debug Authoring、Navigation Settings/NavMesh/Agent/Area/Surface/Modifier/Obstacle/Off-mesh Link/Bake/Query/Debug Authoring、AI Behavior Tree/Blackboard/Perception/EQS/Debug Authoring、Gameplay Ability/Effect/Attribute/Tag/Cue/Debug Authoring、Render Pipeline/Frame Capture/Lighting Bake/Reflection Probe/Post Process/Debug Authoring、UI Asset/HUD/Widget/Binding/Theme/Icon/Accessibility/Menu Flow/Font Atlas Authoring、Data Table/Structured Data/Schema/Import/Validation/SaveGame/Slot/Migration/Platform/Cloud Storage Authoring、Runtime Diagnostics/Performance Timeline/Console/Telemetry/Observability Authoring、Multiplayer Lobby/Matchmaking/Online Services/Replication/Network Emulation/PIE Authoring、Project Operations/Source Control/Automation Report/Submit Gates/Health Dashboard、Spawn Rules/Encounter/Population/World State/Scenario/Quest Flag/Authority/Simulation Authoring、Input Action/Mapping Context/Device/User/Rebinding、Camera Asset/Component/Rig/Director/Blend/Shake/Cinematic Cut/Preview、Script Source/Code Editor/Build/Hot Reload/Debugger/Visual Script/Class/Component、Model/Mesh/Skeleton/Geometry Import/LOD/Collision/Retarget/Preview、Localization/String Table/Culture/Translation Import-Export/Fallback/Pseudo-localization/Preview、Sprite/Atlas/TileSet/TileMap/Canvas 2D、Texture/Image/Cubemap/RenderTarget/Sampler/Compression/Streaming/Preview、Video/MediaSource/Player/Track/Clock/MediaTexture/Playback/Capture/Recording、Volume/Zone/Trigger/Region/Gameplay-Audio-PostProcess Environment、Weather/Climate/Time-of-Day/Wind/Precipitation/Cloud/Atmosphere/Environment、Spline/Path/Road/River/Decal/Brush/Geometry、Procedural Content Generation/Rule Graph/Biome/World Generation、Level Variant/Data Layer/Level Instance/World Outliner、Scene Snapshot/World Diff/Merge/Restore/Conflict Resolution、Multi-User/Collaborative Editing/Session Replication/Locks/Presence/Transaction Conflict、Archetype/Class Defaults/Instance Override/Property Propagation/Reset-to-Default、Cinematic Sequencer/Shot/Track/Binding/Take Recorder/Movie Render Queue、LiveOps/Feature Flag/Remote Config/Segmentation/Experiment/Patch/DLC/Crash Control Plane、Runtime Gateway/Session/Event Consumer/World Sync/Generation/Backpressure/Reconnect/Shutdown、Editor Message Bus/Topic/Subscription/Inbox/Retention/Admission/Dispatch/Request/Dirty Projection/Shutdown、Editor Event Runtime/Journal/Replay/Listener、Editor Extension/Contribution Store/Toolkit/Provider/Reload Lifecycle、Project preflight/admission/activation/ready/focus/close、Builtin View/Window Catalog/Content Provider/Capability/Template/Localization，以及Interactive Tool Scheduler/Resource Lease/Input Capture/Scene Mode/Modal/Extension Lifecycle产品集成已完成首轮E3静态审查。Builtin catalog后续先关闭假Available P0；Interactive Tool后续先关闭零产品consumer、active set覆盖泄漏及state/event因果倒置三项P0，再按Editor03/09/48/50边界完成instance/lease/capture/terminal authority。Hub Project/Engine/Build/launch/persistence/delivery、web shell/catalog/settings/Team/Cloud/accessibility、Marketplace/Account Auth/Organization/Cloud Repository provider、Command/Action/Message Delivery/Task/History/ViewModel/Localization内部operation控制面、Application Host/Bootstrap/Window/IPC/Close/Shutdown/Crash Recovery与跨域Update/release/install/rollback控制面也已完成首轮；其余跨平台bundle细节随Tooling09实施复核。
5. Tooling：workspace/toolchain/CI/validation/dev entrypoint、`cargo-zircon`、export/platform packaging、reflection/codegen、Session Coordinator、benchmark/evidence、DDC、release、test、parity、acceptance archive、Codex control、DesignSpec、BuildSet、Capability Truth、SourceArchive、Target/Command/Cargo Graph、Unsafe、Policy、Failure、Concurrency、Memory、Security、Version、Documentation、Source/Format、Hot Path、Reference、Global State、Ownership、Type Erasure与Transaction共37篇首轮E3已经完成。按当前目标暂停新增tooling优化专题，后续迁移到Rust实现时复用这些既有合同；当前review继续转向Runtime、Editor、App、Plugin、Hub与Runtime Interface的产品/代码域。
   Version Domain/Schema Compatibility/Support Window/Migration/Deprecation/Upgrade-Downgrade控制面也已完成首轮E3；后续随各domain implementation复核reader-writer矩阵、migration receipt、legacy退出和release rollback的数据兼容资格。
   Documentation/API Reference/Plan Currentness/Link/Source Trace/Knowledge Publication控制面也已完成首轮E3；后续先消减required docs gate红baseline，再按ContentClass清理current graph，并逐PublicApiSet接入rustdoc/example/versioned publication与source-bound currentness。
   Rust Module Boundary/Root Entry/Large File/Declaration-Behavior/Folder Topology控制面也已完成首轮E3；后续先冻结13个超限、87个预警与24个root/binding候选，统一Cargo-resolved SourceSet、FileRole和required exit，再按功能owner逐批拆TestOwner、root behavior与混域文件。
   Cross-Language Source Architecture/Entry/Service/Schema/Generated-Test/Folder Topology控制面也已完成首轮E3；后续先冻结2,684路径、23个千行热点、30个目录外generated Zr和368/246/85/62等高fanout目录，再接入Node/Python/PowerShell/C++/WGSL/Zr required matrix，优先拆`world/state.zr`与Coordinator巨型application/service。
   Declarative Project/Asset/UI/Scene/Manifest/Schema/Generated Artifact Physical Authority控制面也已完成首轮E3；后续先冻结715个聚焦文件的Format Catalog与canonical reader/writer graph，再优先消除JSON-in-TOML、mixed ZRP syntax、默认版本、重复ID择胜、tracked-but-ignored重复ZRO、绝对路径manifest与外部dirty compiler隐式权威。
   Hot Path/Algorithmic Complexity/Data Movement/Batching/Cache Locality控制面也已完成首轮E3；后续先把Runtime07当前35个missing anchor分为moved/renamed/retired/true-missing/structure-debt并建立typed catalog，再按Runtime05/08A/08D/08E/08F、Editor25与Plugins04 owner迁移frame whole-world projection、dynamic decode、重复copy/rebuild和metric handoff，最后接入steady/dirty scale与BuildSet qualification。
   Reference Engine Source Corpus/Snapshot/Provenance/Citation/Applicability/Comparison Currentness控制面也已完成首轮E3；后续先冻结五个核心snapshot与142篇legacy locator ledger，建立case-sensitive resolver和Engine/Snapshot/Citation/Claim schema，再把124个目录转为SearchScope/ResolvedCitationSet、按高风险owner迁移claim edge并接入Tooling28 drift currentness。不得把reference pull、nearest tag、路径存在或五家数量当作review/性能资格。
   Global State Scope/Singleton/Service Locator/Static Registry/Cache/Initialization/Reset/Multi-instance Isolation控制面也已完成首轮E3；后续先用AST/BuildSet重取253处static和39个thread-local候选并定义scope/owner/generation，再优先硬切TaskTimer、plugin shared manager、NodeProjectionSession、project font/theme/design token及retryable OnceLock failure，最后以双Project、多World/PIE、A-B-A reopen与DLL reload receipt验收。不得机械删除所有static或把进程退出当作shutdown。
   Ownership Graph/Shared-Weak-Borrow-Lease/Callback-Subscription/RAII-Cycle-Detach-Leak Isolation控制面也已完成首轮E3；后续先用AST/Cargo resolved graph重取所有权边并冻结新增naked registration、unknown strong root capture和无监督detach，再优先硬切UiEventManager、state hooks、World/EventStore observer、UI/Editor subscriber、RPC revoke与ResourceLease强环，最后以World/Host/plugin/DLL terminal后的LeakCensus和OwnershipReceipt验收。不得机械把所有Arc改成Weak或用进程退出证明清理。
   Type Erasure/Dynamic Dispatch/Any-Downcast/Trait Object/Reflection Type Identity/Compiled Dispatch控制面也已完成首轮E3；后续先以Cargo-resolved AST/ABI schema重取边界并冻结新增string-only public contract、silent replacement与hot-path late downcast，再优先硬切service/reflection/bridge、VM backend、render executor binding、Editor composition key与RPC transaction，最后以plugin/DLL reload、schema migration、wrong-type fault和同workload dispatch cost receipt验收。不得机械消灭trait object、序列化TypeId或用字符串ID存在证明contract兼容。
   Transaction Atomicity/Prepare-Commit-Publish/Rollback-Compensation/Idempotency/Crash Recovery控制面也已完成首轮E3；后续先以BuildSet/AST/operation trace重取mutation与commit point并冻结新增无OperationId的跨边界effect、unknown outcome自动重试和silent rollback failure，再优先硬切Scene route、Session archive、Hub create、UI/plugin/module reload、RPC/HTTP retry及Coordinator外部effect，最后以kill-point、restart recovery、duplicate delivery、partial compensation和operator reconciliation receipt验收。不得把database transaction、rename、undo stack、deferred queue或GPU submit一概称为原子事务。
6. Script Entrypoint：PowerShell/Python/Shell/Node/workflow入口、InterpreterSet、Command Registry、CLI/exit、mutation/admission和OperationReceipt已完成首轮E3；后续逐目录核对remaining artifact/native/non-Cargo孤岛与仓库其余物理域，并在新增入口时增量复核registry覆盖和clean-clone可达性。

## 231. Plugin Artifact / Marketplace / Third-party Package / Install / Update / Trust / Non-Cargo / Product Integration 当前源码物理范围

| 范围 | 文件 / 行 / bytes / test attributes | 本轮证据 |
|---|---:|---|
| Runtime/Editor control | 253 / 50,543 / 1,818,070 / 419 | package/project manifest、native loader全目录、VM package、export selection、Editor manager/actions/projection与App caller |
| Product UI/Hub | 8 / 3,720 / 159,387 / 3 | Workbench静态Plugin Manager、route/binding/feedback、Hub coming-soon与web data |
| Manifest/dist/project surface | 121 / 8,797 / 316,314 / 79 | 39份plugin manifest、39份dist Cargo、39份dist lib、workspace/README和两个project manifest |
| 去重合计 | **382 / 63,060 / 2,293,771 / 501** | Zircon fingerprint `dc79aca7d60e5295b847f7406ce03a786de6eba64eea99fcaebd4b29519d6ab1`；冻结时5个入选路径dirty |

当前Native/VM discovery budget、generation snapshot、path containment、Editor enablement/hot reload和export selection是真实可保留底座；但39份首方manifest/dist之外没有tracked可安装artifact，project selection不锁version/source/digest/signer，`native_plugins.toml`又会在id/path mismatch时只写diagnostic并继续发布candidate。现有测试明确固化mismatch后`discovered().len() == 1`，候选随后可进入`Library::new`，因此新增1项P0要求可执行制品准入fail-closed。

真实Project Plugins只扫描`<project>/zircon_plugins`并管理enablement/packaging/target/feature/unload/hot reload；Workbench Plugin Manager则写死三个插件、版本和统计，只返回queued文字；Hub明确将plugin install/toggle/Marketplace标为禁用。五参考冻结19个文件、17,691行、659,576 bytes，fingerprint `5bb518c7c3ca726bd39c25ef4507d97f894e76f1e74d6b08359144f6aca219ca`。报告登记1项P0、48项P1、12项P2与48项资格门；目标是`SignedRepositorySnapshot + PluginLockfile + VerifiedPluginArtifact + TransactionalPluginInstaller + InstalledPluginGeneration + ActivationCoordinator + PluginDeploymentService`。本轮只修改review文档，未运行Cargo、DLL、Editor/Hub、网络、签名、fault、soak或benchmark；tooling按用户要求暂不纳入。详见`zircon_plugins/21-plugin-artifact-marketplace-third-party-package-install-update-trust-non-cargo-product-integration-review.md`。

## 232. Runtime UI Renderer / Display List / Paint Order / Clip / Transform / Opacity / Atlas / Text / Glyph / Batch / WGPU Submit 当前源码物理范围

| 范围 | 全部文件 / 行 / bytes / test attributes | production 文件 / 行 / bytes | 本轮证据 |
|---|---:|---:|---|
| Scene UI renderer | 108 / 25,849 / 925,995 / 301 | 65 / 13,253 / 473,042 | command plan、shape/image、三text route、bitmap/SDF atlas、upload、record、WGSL |
| Runtime paint + icon | 49 / 17,992 / 600,925 / 36 | 42 / 16,846 / 562,218 | widget paint producer、cache/damage、icon parse/plan |
| Interface render contract | 35 / 5,857 / 187,138 / 26 | 34 / 5,697 / 182,484 | paint/brush/batch/cache/parity/debug DTO |
| `zr_rhi` + `zr_rhi_wgpu` UI | 24 / 10,731 / 374,982 / 133 | 18 / 7,286 / 262,712 | native command、batch、pipeline、text/image cache、retained present、surface |
| Editor stream + GPU presenter | 48 / 4,893 / 159,829 / 48 | 41 / 3,206 / 107,902 | ordered stream、icon atlas、RHI conversion、full/damage/resize present |
| 去重合计 | **264 / 65,322 / 2,248,869 / 544** | **200 / 46,288 / 1,588,358** | 全集 fingerprint `f9ab0319758e401becc1b742e78c6b247798c723fc94b475325c8a204c0e16d3`；production fingerprint `b1480967a8c77077e97643099846a1719c9d700be31eb5af5bec5ef7b35bb071`；冻结时4个production路径dirty |

当前game command仍被fanout为shape、image、native/bitmap/SDF text与post-text decoration，再固定按资源类别提交；runtime `Icon`仍走实心矩形fallback；Editor damage仍在retained旧像素上以`TargetLoad::Load`和premultiplied blend重放。三项均为Runtime11C已登记且仍开放的P0，本轮不重复计数。

新差异包括：`UiBatchPlan`、stencil clip、render transform与draw effect没有产品consumer；普通Editor full/damage frame显式保持`generation=None`；runtime icon atlas在4096边长clamp后仍可生成越界slot/UV；native UI只接受Win32 raw handle及non-sRGB BGRA/RGBA UNORM。已修正旧结论：native image cache当前会在admission时把straight RGBA8预乘一次，shader不再二次预乘；glyph instance与bitmap shadow也已有前向实现，但相关failure在managed Cargo/WGPU/product pixel回执前继续open。

参考冻结Unreal 8、Bevy 8、Fyrox 2、Godot 3、Unity Graphics atlas 2，共23个文件、24,534行、971,132 bytes，fingerprint `90cb52a6a7874859cbdc60d415c221b0b1527fb06ccf5fad50f04f2222add2ce`。报告登记0项新增P0、48项P1、12项P2与48项资格门；目标是`UiPresentationArtifact + UiPresentationCompiler + OrderedUiDrawOp + DeviceUiResourceService + UiSurfacePresenter + UiPresentReceipt`。本轮只修改review文档，未运行Cargo、Editor、真实窗口、RenderDoc、HDR、fault、soak或benchmark；tooling按用户要求暂不新增专题。详见`zircon_runtime/79-runtime-ui-renderer-display-list-paint-order-clip-transform-opacity-atlas-text-glyph-batch-wgpu-submit-product-integration-current-source-review.md`。

## 233. Runtime Font Asset / Source / Cook / Database / Face / Fallback / Variation / Color / Resolved Glyph / Cache 当前源码物理范围

| 范围 | 全部文件 / 行 / bytes / test attributes | 本轮证据 |
|---|---:|---|
| Asset / cook | 11 / 2,441 / 90,071 / 18 | manifest schema、artifact payload、source decode、metadata import、auxiliary scan |
| Font database | 43 / 7,911 / 301,263 / 99 | face metadata、backend、fallback、variation、system policy、shared owner、handle registry |
| Service / shaping handoff | 9 / 2,654 / 99,644 / 25 | neutral projection、generation retry、synthetic fallback、resolved artifact |
| Render / SDF consumers | 25 / 6,921 / 276,221 / 75 | renderer manifest cache、TextRenderState、SDF source/offline/cache/budget |
| UI / Editor surfaces | 8 / 2,280 / 85,788 / 12 | dead UI registry、UI artifact、layout consumer、Editor runtime line consumer |
| 去重合计 | **96 / 22,207 / 852,987 / 229** | 全集 fingerprint `7d318b995d53f3f0b5e9e94b28a594e114f2bb417d23d26fe1705981d8db7266` |
| Production | **68 / 15,789 / 598,749** | production fingerprint `15dfceb551b90f06161271481d015dfaf28f61f0300ba1c78122c4b6e9803548` |

默认 `ZirconDefaultComposite-subset.ttc` 为103,624 bytes，SHA-256 `bf25507c694c39e9ffd514f8f8ab3b79ced814cd0e96bb60a95c5bb6434936c7`；只提供Fira Mono和小型CJK SC proof，日/韩/繁中/Arabic/Hebrew/Emoji仍主要依赖主机family。当前importer读取并解码source bytes后只发布metadata `FontAsset`，artifact payload无blob，而scan把字体source视为auxiliary；runtime因此从manifest重开project/source-tree path。Runtime11B P0-1继续open且由其唯一计数。

共享 `FontDatabase`、layout service与handle registry仍由process-global static持有，global初始化强制Discover系统字体；一个renderer/session mutation会推进所有consumer generation并清bitmap/SDF/atlas derived state。Cosmic每线程最多保留四个locale `FontSystem`与backend clone，face retire留下monotonic tombstone。另一方面metadata、fallback candidate、effective instance、batch handle、SDF source/offline budget等历史修复已进入当前源码，应在新service内保留。

真实shaping全失败时仍会生成无face/instance的synthetic glyph，service又把非空白项标记为`requires_rasterization=true`；该阻断继续由Runtime11B P0-2唯一计数。报告新增0项P0、48项P1、12项P2与48项资格门；目标是`FontBlobArtifact + FontCollectionSnapshot + FontCollectionService + SystemFontProvider + ResolvedFontFaceLease + ResolvedGlyphStatus + FontDiagnosticsReceipt`。

参考冻结Unreal 7、Godot 6、Bevy 4、Fyrox 2、Unity Graphics atlas 1，共20个文件、20,837行、958,439 bytes，fingerprint `97a6194cbb7bcd0003d7c3d30f39fcf3e58a13fc30552049f8f7638530da912d`。Unity Graphics本地corpus没有TextCore/TMP源码，只作atlas生命周期旁证。本轮只修改review文档，未运行Cargo、Editor、clean package、系统字体隔离、真实WGPU、恶意字体fuzz、跨平台golden、fault、soak或benchmark；tooling按用户要求暂不新增专题。详见`zircon_runtime/80-runtime-font-asset-source-cook-database-face-fallback-variation-color-resolved-glyph-cache-product-integration-current-source-review.md`。

## 234. Runtime Text Shaping / Unicode / BiDi / Script Run / Cluster / Line Break / Wrap / Layout 当前源码物理范围

| 范围 | 全部文件 / 行 / bytes / test attributes | 本轮证据 |
|---|---:|---|
| Shaping core | 41 / 6,398 / 206,598 / 86 | RustyBuzz backend、direction/features、script/language、cluster provenance、fallback与cache |
| Layout core | 44 / 11,261 / 372,030 / 168 | paragraph、line break、wrap、whitespace、hyphen、justification、metrics与viewport |
| UI text | 64 / 13,175 / 440,935 / 194 | resolved text artifact、visual projection、caret/hit-test、measurement cache与renderer handoff |
| Product contract | 13 / 5,987 / 208,111 / 45 | runtime/editor/interface产品接线、public DTO、command consumer与测试合同 |
| 去重合计 | **162 / 36,821 / 1,227,674 / 493** | 全集 fingerprint `2759b66d8b81aa7990a3195f486db466d7d40c4ea0a2f8a9414b9a5de898292c`；冻结时15个范围内路径dirty |
| Production | **114 / 25,861 / 859,391 / 141** | production fingerprint `8e65b085208d7d575d17d50b6beec41205e072d0949188a2bf2da461ec0fabd2` |

当前实现已具备真实RustyBuzz横排/竖排、`unicode-bidi` L1/L2、`unicode-linebreak` UAX14、grapheme、hard-line streaming、bounded cache和resolved glyph artifact，应作为后续重构底座保留。与此同时，normalization被显式禁用，language只做trim/underscore/lowercase，script没有Script_Extensions/paired-bracket继承，emoji/kinsoku/Arabic joining/smart punctuation主要是手写表，fallback按grapheme而非完整sequence，UI与interface又重做BiDi、visual range和synthetic glyph projection。

本轮新增1项P0：`TEXT_SHAPING_RUN_MAX_BYTES`的定义与re-export已删除，但7个测试文件仍保留24处引用，当前test target静态无法解析该标识符。旧64 KiB cap会把单个logical line切成不同语义，不能恢复；修复必须将source-line identity与具备budget、deadline、cancellation的backend work unit分离。Runtime11B登记的synthetic rasterizable glyph阻断仍开放，但不在本报告重复计数。

布局层还存在固定8-grapheme边界上下文、ligature advance按grapheme等分、soft hyphen始终生成ASCII `-`、`EndWord`依赖空白、tab仅uniform、clip先截行再测量、preedit复制整串且禁用viewport、measure cache仅按entry数限额、document key信任owner/revision以及没有retained dirty-paragraph增量模型等差异。报告登记48项P1、12项P2与48项资格门，目标是`UnicodeDataSnapshot + TextAnalysisArtifact + ShapingOutcome + GlyphClusterMap + ParagraphLayoutArtifact + DocumentLayoutSession`。

参考冻结Unreal 12、Godot 6、Bevy 4、Fyrox 4，共26个文件、26,977物理行、23,325非空行、1,042,517 bytes，fingerprint `4dfba95cf95fe8ab01136c84f85c15da77f4af72afd53420f19c5bc7a398cd18`。Unity Graphics本地corpus只有TMP shader/sample资源，没有TextCore、TMP或主文本引擎源码，因此未用于本专题结论。本轮只修改review文档，未运行Cargo、Editor、Unicode官方corpus、真实产品、同负载参考引擎benchmark、fault、soak或性能资格；tooling按用户要求暂不新增专题。详见`zircon_runtime/81-runtime-text-shaping-unicode-bidi-script-run-cluster-line-break-wrap-layout-product-integration-current-source-review.md`。

## 235. Runtime Text Editing / Document / Selection / Caret / Hit Test / IME Composition / Clipboard / Secure Text 当前源码物理范围

| 范围 | 全部文件 / 行 / bytes / test attributes | 本轮证据 |
|---|---:|---|
| Editing core | 8 / 1,735 / 56,781 / 16 | edit action、grapheme/word、caret geometry、hit-test |
| Input / document / IME / clipboard | 20 / 3,801 / 123,475 / 11 | widget dispatch、keyboard/pointer、constraints、focus、host effect |
| Render / component / accessibility | 52 / 6,865 / 223,757 / 14 | render extract、secondary reducer、semantic value/action |
| Interface / ABI / App host | 21 / 4,292 / 147,707 / 25 | public DTO、window event、dynamic ABI、winit/App IME |
| Product ZUI / focused tests | 16 / 5,419 / 196,140 / 117 | WOC auth/recovery及IME/keyboard/pointer测试 |
| 去重合计 | **117 / 22,112 / 747,860 / 183** | 全集 fingerprint `4921228228e82e4f1b8164b9b626fd2f8a5c7e1a20596803ab5b1849589c55ac`；冻结时0个范围内路径dirty |
| Production | **96 / 15,713 / 521,889 / 37** | production fingerprint `9720d179a2d271e0328d8899c0646d300c98fe71da2755f84d0f741ef0e9387d` |

当前grapheme insert/delete、resolved-layout hit-test、bounded surrounding text和winit/App IME接线是真实可保留底座；但widget metadata与component reducer各自重建并顺序写回text/selection/composition，public snapshot没有document id/revision/transaction/history，caret/navigation不消费visual wrapped/BiDi geometry，preedit又直接混入committed `String`并保存明文restore副本。

Runtime能够生成clipboard host request，但App、Editor和dynamic ABI没有clipboard consumer/result route；cut先删除再等待无回执write，paste read也无法关联原document revision。UI request中的composition rects在转换到core host request时丢失，production preedit clause固定为空。WOC密码字段使用`input_kind = "password"`，classifier却只识别secure布尔键，原文进入render command、snapshot、event/binding与accessibility；该P0继续由Runtime11B唯一计数，Runtime77保留通用effect非原子P0。

参考冻结Unreal 9、Godot 8、Bevy 4、Fyrox 1，共22个文件、40,529物理行、34,297非空行、1,501,536 bytes，fingerprint `68303f040321f46de5841868c31aafaefa287cd4741e679f8dc6ffe1d4c3ae94`。Unity Graphics本地corpus不含TextCore、UI Toolkit或TMP编辑引擎源码，未用于本专题结论。报告新增0项P0、48项P1、12项P2与48项资格门；目标是`TextDocumentSnapshot + TextEditTransaction + TextSelectionSet + TextCompositionSession + TextInputHostSession + SecureTextPolicy + ClipboardHostResult`。本轮只修改review文档，未运行Cargo、App/Editor、真实IME/clipboard/screen reader、fault、soak或benchmark；tooling按用户要求暂不纳入。详见`zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md`。

## 236. Runtime Localization / Internationalization / Locale / Culture / Message Format / Plural / Number-Date / String Table / Resource Fallback 当前源码物理范围

| 范围 | 全部文件 / 行 / bytes / test attributes | 本轮证据 |
|---|---:|---|
| Runtime identity / catalog contracts | 12 / 749 / 22,361 / 3 | localized ref/report、key-set catalog、locale helper、resource kind |
| Compiler / artifact / render / reload | 20 / 2,532 / 90,445 / 0 | validation、package/cache、tree/surface、renderer/a11y、hot reload |
| Editor bridge / preview | 22 / 5,007 / 190,297 / 23 | shell i18n、settings/bus、UI Asset report/catalog/preview |
| Product assets / focused tests | 303 / 56,230 / 3,193,729 / 22 | 296份ZUI/TOML、WOC key模型、focused contract tests |
| 去重合计 | **357 / 64,518 / 3,496,832 / 48** | 全集 fingerprint `98d7f826947e54dd59e7ef9e9604002ff4e9be6d7a7f98ffd66b4c1010db6af0`；冻结时6个范围内路径dirty |
| Production | **351 / 62,939 / 3,447,624 / 16** | production fingerprint `5dc3d110d72c6ff45ca4fef22dfb301457b149be0bbb66b8743112430c190bb0` |

当前localized ref、dependency report/package sidecar、versioned UI artifact/cache、Editor 54-key shell i18n及有界locale事件是真实可保留底座；但compiler用空String绕过typed schema，catalog只保存key，tree保留raw table而renderer/a11y只读scalar，dependency/fallback没有Runtime consumer，hot reload/App/ABI/script也无Localization service。296份产品ZUI/TOML中的272个ZUI有3,239处text-like赋值却0处text_key，WOC native 32处key声明同样没有translator；Locale Preview仍是三项硬编码报告。

参考冻结Unreal 14、Godot 8、Bevy 1、Unity Graphics 1，共24个文件、13,048物理行、10,972非空行、501,100 bytes，fingerprint `706ea1234c802e678a33fb0b277ef75809609f998fd79e61202e27f2e570effa`；Fyrox全树无专用Localization系统，只作负边界。报告新增0项P0、48项P1、12项P2与48项资格门；Editor33五项P0不重复计数。目标是`CultureTag + LocalizedTextIdentity + LocalizationCatalogArtifact + LocalizationCatalogGeneration + LocalizationSnapshot + RuntimeLocalizationService + LocalizationOutcome`。本轮只修改review文档，未运行Cargo、App/Editor、locale switch、translated render、fault、soak或benchmark；tooling按用户要求暂不纳入。详见`zircon_runtime/83-runtime-localization-internationalization-locale-culture-message-format-plural-number-date-string-table-resource-fallback-product-integration-current-source-review.md`。

## 237. Runtime Rich Text / Markup / Parser / Token / Style Span / Inline Object / Link / Image / Table / List / Layout / Selection / Accessibility / Security 当前源码物理范围

| 范围 | 全部文件 / 行 / bytes / test attributes | 本轮证据 |
|---|---:|---|
| Parser / model / cache production | 19 / 4,580 / 151,387 / 19 | DTO、HTML/BBCode/Markdown、table/list、decorator、compiled artifact、registry与cache |
| Layout / UI / render / a11y / interface production | 41 / 11,529 / 410,343 / 28 | 横竖排/table layout、artifact handle、resource、paint、link、semantic extraction与public DTO |
| Production去重合计 | **60 / 16,109 / 561,730 / 47** | fingerprint `3270460f2e25d334fa62631c93b689be44c3841f4ac0d205bea9420303a9381e` |
| Focused tests / proofs | **45 / 15,401 / 554,770 / 332** | parser/layout/cache/artifact/hit/render/table/product proof静态证据；fingerprint `c3c759a469ffe3c6772d41731b46bd25f4c202c65f708a8d86dfc14915e802e1` |
| 全集去重合计 | **105 / 31,510 / 1,116,500 / 379** | fingerprint `214be21bc8e025fa99af60d9477f05595354a58f126d8e459e92f8eaa5f3addf`；冻结时5个范围内路径dirty |

当前`CompiledRichText`已成为Arc-backed canonical artifact，UI layout、renderer、image dependency和link hit-test共享同一产物；HTML/BBCode/Markdown subset、decorator、block/list/table、横竖排inline layout与256-entry/8 MiB cache是真实可保留底座。dirty parser中的unterminated delimiter frontier与深层active-tag index已修复两类具体搜索热点，但所属Session的managed validation仍未完成，不能把实现存在写成accepted。

关键差距包括：parse入口没有input/token/node/span/attribute/depth/output/time/deadline/cancellation预算和structured diagnostic；grapheme对齐与active tag仍复制累计metadata；decorator是无owner/隔离/budget的frame-path任意代码；global parser/cache没有project/plugin生命周期。`StyleOverride`中的italic、letter spacing和OpenType features未完整进入shaping，`UiRichTextArtifactHandle`又只按`TypeId`判等。Widget只渲染实心矩形，Image/Link语义不足，rich document被排除在viewport/editing之外，a11y仍从raw scalar取名。

对`zircon_editor/src`、`zircon_app/src`和`assets`共5,949个tracked文件检索`UiRichTextFormat`、`rich_text_format`与`RichText`，产品命中为0；能力目前主要存在于runtime、fixture、test和proof command。参考冻结Unreal 13、Godot 7、Bevy 3、Fyrox 3、Unity Graphics 3，共29个文件、18,589物理行、16,038非空行、651,548 bytes，fingerprint `311d118fe2872285aebbf62c0aedd0ca5496d3c47b950eb0b2a633e6c9b4691c`。Unity Graphics本地corpus无TextCore/TMP engine源码，只作负边界。

报告新增0项P0、48项P1、12项P2与48项资格门；Runtime11B、78、79、82继续唯一拥有text budget、a11y、GPU与editing父边界。目标是`RichParserDescriptor + RichParseBudget + RichParseOutcome + CompiledRichTextArtifact + RuntimeRichTextService + RichLayoutDocument + RichInlineObjectLease + RichSemanticProjection + RichStyleAsset`。本轮只修改review文档，未运行Cargo、Editor、WGPU、screen reader、fuzz、fault、soak或benchmark；tooling按用户要求暂不纳入。详见`zircon_runtime/84-runtime-rich-text-markup-parser-token-style-span-inline-object-link-image-table-list-layout-selection-accessibility-security-product-integration-current-source-review.md`。

## 238. Runtime Asset Import / Source Discovery / Importer Recipe / Subasset / Derived Data / Artifact / Cook / Package / Incremental Build / Worker / Determinism 当前源码物理范围

| 范围 | 文件 / 行 / bytes / test attributes | 本轮证据 |
|---|---:|---|
| Artifact | 18 / 6,235 / 222,002 / 18 | v5 manifest、zstd、BLAKE3、64 KiB chunk、resident LRU、bounds与restore |
| Importer | 52 / 9,788 / 346,855 / 55 | descriptor/context/outcome、registry、native envelope、builtin格式、subasset与cook request |
| Project | 41 / 7,255 / 257,364 / 29 | discovery、v7 meta、full/targeted candidate、dependency projection、durable publication |
| Pack | 17 / 1,968 / 64,553 / 1 | sorted writer、dedup、reader、delta、stage/promote/install receipt |
| Pipeline | 61 / 7,393 / 275,164 / 42 | project/runtime manager、bounded worker、resource publication；另有3项ignored |
| Watch | 19 / 817 / 26,537 / 0 | notify mapping、fold/batch、URI mapping、dispatch与diagnostic |
| Virtual Geometry + Mesh SDF cook | 14 / 2,321 / 76,371 / 12 | import-time cook、settings、budget、artifact projection与确定性proof |
| Runtime85 core去重合计 | **222 / 35,777 / 1,268,846 / 157** | fingerprint `255aca3da8942186a000643717cb7d65c6858005fb9b885626a8f248ba95d35d`；冻结时21个相关source路径dirty/untracked |
| Runtime asset tests | **167 / 35,003 / 1,215,488 / 628** | fingerprint `11e2242205cdbaf5a68c3da2685063b9ebd7a8d3bcc300f95ee7312788f47a69`；另有1项ignored |
| Runtime export pack binary | **6 / 922 / 32,795 / 4** | fingerprint `7c0062ce8990cdc787d85709ac99ae4891bbe29ff5327f924c2413998e62afe3` |
| Editor export touchpoints | **44 / 11,308 / 386,306 / 90** | fingerprint `6d906242cf4825e4d619c1eb7cb40d752414818154dc640099c28088071f2061`；另有1项ignored |

当前COW importer registry、v7 sidecar meta、候选代/full-targeted publication、durable journal/recovery、BLAKE3+zstd+64 KiB chunk artifact、64 MiB resident chunk LRU、bounded/single-flight worker以及pack delta/install是真实可保留底座。旧结论中若仍把worker描述为无界或每waiter深拷贝payload已不符合当前源码；现实现有unique request/waiter/completion/bytes/TTL边界并共享`Arc` payload。

本轮新增1项P0：source discovery把`.bin`、`.ttf/.otf/.woff/.woff2`等当作auxiliary，单文件source snapshot不携带included files，restore key只覆盖根bytes/settings/importer；glTF和font importer却从相对filesystem path直接读取外部bytes，watch又没有辅助URI到父source/action的反向owner。只改变external buffer/image/font blob时，父资产可能不重建并恢复陈旧artifact。修复必须建立immutable source snapshot、declared read receipt、reverse dependency index和包含全部输入content digest的build key。

其余差距集中在typed/versioned recipe、target/toolchain identity、streaming/budgeted importer、stable subasset UID、canonical build DAG、hierarchical DDC、artifact provenance/GC/semantic pages、独立VG/SDF cook action、qualified worker、canonical cooked closure、streaming/signed pack及跨机determinism。手写export manifest当前读取raw source，Editor `CookAssets -> Pack`又只交接`assets.json`，尚未形成同一资产图的build receipt。

参考冻结Unreal DDC/Cooker/IoStore、Godot ResourceImporter/EditorFileSystem、Bevy AssetProcessor、Fyrox ResourceManager和Unity Graphics importer consumer共19个文件、15,122物理行、13,151非空行、585,539 bytes；规范化相对路径与原始bytes排序串联fingerprint `befd038b847096384f8d909bf6ddeda12929c05beaa027cc5ef31d0993e965f4`。Unity Graphics本地corpus不含完整Unity AssetDatabase/DDC，只作consumer旁证。

报告新增1项P0、48项P1、12项P2与48项资格门；Runtime04/51/64、Editor04/32/35及既有export报告继续唯一拥有父边界。目标是`AssetSourceAuthority + AssetImportRecipeCatalog + AssetBuildGraph + AssetBuildScheduler + DerivedDataService + ArtifactRepository + CookVariantResolver + SubassetIdentityRegistry + ContentPackCompiler + InstallAndMountService`。本轮只修改review文档，未运行Cargo、Editor、真实import/cook/package、网络DDC、签名、fault、soak或benchmark；tooling按用户要求暂不纳入。详见`zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md`。

## 239. Runtime Asset Type / Schema / Imported Payload / Project Document / Validation / Dependency / Serialization / Versioning 当前源码物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes | 本轮证据 |
|---|---:|---|
| Production、contract与consumer去重集合 | **233 / 34,524 / 31,672 / 1,195,124 / 未单独冻结** | asset/assets 101文件、facade、artifact payload/store、ingest、dependency extractor、project publication、typed load及Interface project/resource/UI schema；fingerprint `3a4aae807406ceb1a204c65b918107fb8995a58afc9b964754bec10920257a2e`。其中asset/assets、facade、ingest与dependency extractor四个核心子集共96项inline test属性，不把该子集数字冒充233文件全集统计 |
| Runtime asset tests | **167 / 35,003 / 32,094 / 1,215,488 / 628** | 另有1项ignored；fingerprint `17ddde25feaf268f782e5b7b73de1041e1ced3811e172b1135a189297feabdb2` |
| 参考引擎 | **35 / 21,456 / 777,029 bytes** | Unreal AssetRegistry/CustomVersion、Bevy typed id/meta/processor、Fyrox type UUID、Godot UID/dependency/format version、Unity Graphics type remap/upgrader；fingerprint `2ecdc384d8f51897066db2a118bff4f92f5b8711d627290e6645b777c9b3ee4a` |

当前31种`ImportedAsset`、typed `Asset/Handle/Assets`、UUID+locator与project GUID/path hint/subasset引用、Material/Model/Scene正式document wrapper、局部schema/version/validator，以及v5 artifact的magic/hash/chunk/size验证是真实可保留底座。exact type却没有进入record/handle/event/manifest，UiIcon/Texture与legacy/V2 UI等共享coarse kind，typed load按固定顺序试探downcast；type、codec、validator、dependency和migration由多组手写match维护。

本轮新增1项P0：Prefab、MaterialGraph、Terrain、TerrainLayerStack、TileSet、TileMap、UiV2 View/Component/Style与UiIcon都能从payload计算直接引用，但fresh/targeted/restore写入`.zmeta`和`ResourceRecord`的extractor只覆盖Scene、Material、Model。Editor catalog直接读payload，因此可显示Runtime ready/reload/affected referencer/package closure完全看不到的边。Runtime51 P1-056至P1-058继续拥有通用extractor registry缺口，本报告拥有已激活类型上的确定性正确性升级。

其余差距集中在stable `AssetTypeId/SchemaId/CodecId`、per-type support window、强制validation/migration、全部authoring asset的正式project reference codec、unknown-field/lossless rewrite policy、typed dependency edge、subasset label保留、per-type artifact schema、plugin-defined asset type、compatibility state、golden/fuzz/conformance矩阵和Runtime/Editor唯一graph snapshot。

参考冻结显示：Unreal以package/object/value identity、依赖category/property和GUID custom version表达工程合同；Bevy typed id把类型进入hash且meta记录loader/processor/settings/process dependency hash；Fyrox typed request校验type UUID；Godot external dependency保留UID、fallback path与type并支持rename；Unity Graphics保留type metadata/remap和显式upgrader。本地Unity corpus不含完整AssetDatabase，只作包内旁证。

报告新增1项P0、48项P1、12项P2与48项资格门；Runtime04/51/61/64/68/69/73/74/85与Editor04/24/32/34/35继续唯一拥有父边界。目标是`AssetTypeCatalog + AssetEnvelope + ProjectDocumentCodec + AssetDependencyGraph + AssetCompatibilityService`。冻结时production集合17个、test集合7个路径dirty；本轮只修改review文档，未运行Cargo、Editor、真实migration、artifact/plugin skew、fuzz、fault、soak或benchmark；tooling按用户要求暂不纳入。详见`zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md`。

## 240. Runtime Asset Reference / Identity / Locator / GUID / Subasset / Redirector / Rename-Move / Resolution / Repair / Migration 当前源码物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes | 本轮证据 |
|---|---:|---|
| Runtime asset production | **386 / 63,517 / 58,250 / 2,231,172 / 270** | reference DTO/resolver、persist writer、registry、full/targeted generation、importer、migration、watch与pipeline；fingerprint `84162C789B120A284CF1CDA5CFE31003A0CDF819715A293763B82E954843760C` |
| Runtime asset tests | **167 / 35,003 / 32,094 / 1,215,488 / 628** | resolver、registry、watch、sidecar move、migration与product pipeline静态证据；fingerprint `845C2427C6E850968542276E6730721D256CAAB7D0918D54936450A8920BB8A7` |
| Runtime Interface project/resource contracts | **51 / 2,674 / 2,388 / 83,475 / 29** | `AssetRef`、`PersistedAssetReference`、locator、UUID、record与retired migration；fingerprint `F59789AE731D79F65734D66C78115CB93DC171C1909EFEECC2C177281C7D7053` |
| Editor asset/project consumers | **98 / 15,817 / 14,305 / 531,168 / 139** | dirty save、catalog generation、reference graph、project/template product touchpoints；fingerprint `64F34F5E47516C9B23AB7FF347A3D74AD7B51294C81BA6C38B1407167130AC9D` |
| 五引擎参考切片 | **22 / 15,116 / 13,147 / 561,638 / 48** | Unreal SoftObjectPath/Rename/Redirector/Registry、Bevy path/id/handle/meta、Fyrox move、Godot UID/dependency rename、Unity Graphics GUID consumer；fingerprint `116E17DDF99930193EB04AEDD4022F78E65B5705D5CCA76594E16BB06DC7D5B7` |

当前`PersistedAssetReference`已区分project与builtin，project `AssetRef`保存GUID、movable path hint与subasset；registry、candidate generation、sidecar identity恢复、`DanglingSubasset`以及migration transaction是真实可保留底座。当前源码也已拒绝缺失label回退父资产，但其Runtime04 failure仍处于`validation_pending`，本报告不抢占该owner。

本轮新增两项P0。第一，missing GUID时path hint会直接产生GUID repair；GUID存在但subasset不匹配时，path+label也可替换现有GUID。删除旧资产后同路径创建新资产会静默错绑，声明的`Conflict`没有production构造分支，migration还会复用并自动应用该语义。第二，Material/Model/Scene importer产生的`reference_repairs`没有任何full/targeted/pipeline/Editor consumer，native adapter又固定清空；本次运行payload与authoring source可持有不同引用事实。

参考源码表明最低工程线是：Unreal把rename建成referencer/source-control/redirect/save控制面；Bevy分离source/path/subasset、typed identity和runtime generation；Fyrox至少有显式resource move及UUID-preserving registry update；Godot由format loader拥有dependency rewrite；Unity Graphics consumer区分GUID与path fallback。本地Unity corpus不含完整AssetDatabase，因此不用于宣称其事务实现细节。

报告新增2项P0、48项P1、12项P2与48项资格门；Interface02、Runtime24/25/51/64/85/86、Runtime04 failure及Editor04/10继续唯一拥有父边界。目标是`QualifiedAssetReference + AssetResolutionSnapshot + AssetReferenceResolver + ReferenceRepairPlanner + AssetMutationTransaction + ReferenceCodecCatalog + AssetReferenceGraphSnapshot`。本轮只修改review文档，未运行Cargo、Editor、真实rename/migration、source-control、fault、soak或benchmark；tooling按用户要求暂不纳入。详见`zircon_runtime/87-runtime-asset-reference-identity-locator-guid-subasset-redirector-rename-move-resolution-repair-migration-product-integration-current-source-review.md`。

## 241. Editor Asset Workspace / Content Browser / Folder-Source Tree / Selection / Activation / Mutation / History-Collection 当前源码物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes | 本轮证据 |
|---|---:|---|
| Zircon资产产品链 | **209 / 33,959 / 30,988 / 1,205,005 / 250** | ZUI、asset core、catalog/details/preview、event/runtime access、retained pointer/effects、workspace snapshot/layout和focused tests；另有3项ignore；fingerprint `b62334ab04a3e22ee0fd9cbfdf74a1bd065b721624feda05a181b267ad06f6ea` |
| 五引擎参考切片 | **17 / 22,457 / 19,478 / 842,734 / 21** | Unreal Content Browser/rename、Godot FileSystem dock/index、Fyrox asset browser、Bevy source/path/event与Unity Graphics consumer；fingerprint `6cc4f7152ec189fb0002bde57974c3efed0331b4f944d99438868b935f72252b` |

当前catalog/details/preview generation、稳定folder排序、`navigate_to_asset`意图、enabled Asset Type Registry、operation write-target准入与typed asset drag payload是真实可保留底座；但Asset Browser生产pointer只发Select Folder/Item，没有双击、Enter或Open Asset路由，toolkit/context descriptor主要停留在文本或snapshot。`Locate selected asset`事件无target，只打开Assets pane并请求preview refresh。

本轮新增四项P0。第一，default scene asset/resource change会触发重新open ProjectManager、scan/import、load scene并直接replace authoring world，完全绕过dirty/history/conflict transition。第二，Locate按钮文案与行为不符。第三，Browser条目无法激活已存在的toolkit打开内核。第四，catalog只保存`ResourceKind`并从其重建AssetTypeId，使插件exact type、toolkit、context和presentation在真实资产条目上丢失。

报告新增4项P0、56项P1、12项P2与44项资格门；Editor02/03/04/08/09/50/55/56及Runtime85/86/87继续唯一拥有dirty scene transition、catalog/import、command/job、extension、transfer/query和asset type/reference父边界。目标是`AssetWorkspaceService + AssetBrowserInstanceRegistry + ContentSourceProviderRegistry + AssetActivationRouter + AssetActionProviderRegistry + AssetMutationCoordinator + AssetChangeReconciler + AssetCollectionService`。冻结时209份范围内有22个非本轮dirty路径；本轮只修改review文档，未运行Cargo、Editor、真实mutation/watcher conflict、plugin reload、fault、soak或benchmark；tooling按用户要求暂不纳入。详见`zircon_editor/57-editor-asset-workspace-content-browser-folder-source-tree-selection-open-create-import-rename-move-delete-history-collection-product-integration-review.md`。

## 242. Editor Scene Viewport Host / Render Product / Surface Lifecycle / Frame Currentness / Multi-Viewport 当前源码物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes | 本轮证据 |
|---|---:|---|
| Zircon视口产品链 | **86 / 11,974 / 11,055 / 446,169 / 109** | retained host/controller、pane presentation、Scene/Game descriptor、runtime viewport/product/capture、submit generation guard、wgpu external image cache与focused tests；fingerprint `737c56f8ec9561d1a25a426a0458bf98d06d5c155344641fa930c6cf3c516b20` |
| 五引擎参考切片 | **14 / 31,013 / 26,550 / 1,231,129 / 0** | Unreal per-instance editor viewport、Godot multi-viewport、Fyrox scene/preview target、Bevy retained view identity与Unity Graphics per-camera consumer；fingerprint `ea01c41e5c8eca9ab76821ddc07d7e6161fed2c157f3e29d9f8e983f89b4e640` |

当前真实底座包括runtime viewport create/destroy、scene extract、GPU external image、CPU capture、generation-keyed presenter cache和retryable native surface present。但`RetainedEditorHost`只有一个controller/size，host contract只有一个global image，painter把它画进所有Scene/Game pane；默认Scene、Game、duplicate和floating view没有独立render产品。

本轮新增四项P0。第一，Scene/Game与多pane共享单一产品，Game没有play world/camera事实。第二，resize先销毁旧viewport，create/quality/submit错误又消费dirty，旧图可无stale标识地冻结且不重试。第三，world-space UI公开完整3D字段却只画screen rectangle、只改status而不派发真实control。第四，runtime两个submit入口在post-render generation复核前发布direct GPU产品，失败产品仍可能对UI可见。

报告新增4项P0、56项P1、12项P2与46项资格门；Editor03/07/30/53与Runtime09a/09b/11c/57/65/79继续唯一拥有scene交互、PIE、camera、tool lease、renderer/RHI/window/quality/UI renderer父边界。目标是`EditorViewportSessionRegistry + ViewportRenderRequest + ViewportRenderReceipt + ViewportFrameProduct + ViewportPresentationState + ViewportPresentationLease + ViewportProductMap + WorldUiSurfaceProduct`。冻结时86份范围内有10个非本轮dirty路径；本轮只修改review文档与索引，未运行Cargo、Editor、真实GPU presenter、fault、device loss、multi-window、soak或benchmark；tooling按用户要求暂不纳入。详见`zircon_editor/58-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-product-integration-current-source-review.md`。

## 243. Editor Scene Viewport Interaction Controller / Input / Picking / Selection / Highlight / Gizmo Transaction / Cancel 当前源码物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes | 本轮证据 |
|---|---:|---|
| `scene/viewport/**` | **116 / 6,926 / 6,314 / 239,804 / 38** | controller、handles、projection、interaction extract、pointer/picking adapter与render packet；fingerprint `39a0740ad6ef6fbb71618b030cacef974343e67b2e180f2ef5578bdf3e6a031f` |
| Scene Mode + Selection | **24 / 2,341 / 2,054 / 70,747 / 24** | mode registry/stack/isolation/context及selection domain/mutation；fingerprint `7aae5ed6537f9af44180db25115edc815699e78c53f772967f180e544a3df7f6` |
| Workbench、host bridge与focused tests | **16 / 2,683 / 2,452 / 96,971 / 40** | pointer cancel、focus/Escape、Gizmo transaction、render dirty与产品tests；fingerprint `d396670dc2c371859a959eb346e675fd94c10d789f3c013611784543fa34cb34` |
| Gateway + Runtime integration | **14 / 2,927 / 2,546 / 99,442 / 27** | highlight gateway/store/session、dynamic extract cache与visible spatial query；fingerprint `a832a90531256d52e35ff62ada6d0bae4ce35845ff81a8cca44c6c8dd8cb5fd7` |
| Zircon合计 | **170 / 14,877 / 13,366 / 506,964 / 129** | working-tree current-source aggregate；fingerprint `b1ea75a64ffdc616bacdd1fc7a35f0a5566eeb444ea813e8aab5fb42d6807643` |
| 五引擎参考切片 | **23 / 31,789 / 26,957 / 1,213,892 / 0** | Unreal input/capture/TransformProxy、Godot viewport/gizmo、Fyrox modes、Bevy pointer/mesh picking与Unity GPUDriven picking/outline views；fingerprint `43a2b678b3e1f347c9c29613f4c27b18e86c7ad7488fdba18cb7c9b5ca65a567` |

当前真实底座包括Scene Mode先行输入、Edit/Play双域有序多选、Handle preview由workbench提交单个undo command、Escape/focus loss显式Cancel、renderer-visible broad phase以及per-viewport HighlightSet store。旧Editor03中“highlight与Frame Selection只看primary”的描述已部分过时：当前highlight DTO遍历全部active selection，Frame Selection也遍历多选位置；Gizmo仍单node，highlight没有runtime frame consumer，Frame Selection仍不读取真实bounds。

本轮新增三项P0。第一，platform与UI dispatcher产生Pointer Cancel并释放capture，host却固定不生成viewport command，且focused test明确固化该行为，active Gizmo preview可能无法rollback。第二，controller只有一个无owner drag slot，Handle期间右/中键会覆盖session，workbench随即把variant消失误判为正常结束并Commit。第三，Highlight gateway错误让`render_frame_submission`返回`None`，host随后消费render dirty，使可选overlay失败压掉base Scene帧并冻结旧图。

报告新增3项P0、8项P1、6项P2与36项资格门；Editor03继续拥有Scene selection/picking/Gizmo语义，Editor53拥有通用tool/capture lifecycle，Editor58拥有viewport session/currentness，Runtime47拥有Picking frame authority，Runtime10开放failure拥有HighlightSet runtime frame consumption。目标是`ViewportInteractionSessionId + ViewportInputEnvelope + ViewportCaptureLease + InteractiveEditSession + SelectableSpatialProduct + ResolvedViewportHitList + ViewportHighlightProduct + ViewportInteractionReceipt`。冻结时170份范围内有17个非本轮dirty路径；本轮只修改review文档与索引，未运行Cargo、GUI、真实GPU picking、touch/pen、多window、fault、soak或benchmark；tooling按用户要求暂不纳入。详见`zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md`。

## 244. Runtime Asset Watch / Change Ingress / Coalescing / Rename / Overflow / Targeted Reimport / Generation / Reload 当前源码物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes | 本轮证据 |
|---|---:|---|
| Runtime production | **99 / 10,570 / 9,679 / 388,691 / 32** | watcher、pipeline manager、project manager、targeted/full import、catalog generation与reload product；fingerprint `445994ccbc7fe1a3fe38c2e0664d13610bc41edb1db22b5154eadd9da5a4736e` |
| Runtime focused tests | **8 / 2,720 / 2,477 / 97,477 / 55** | watcher、overflow、targeted/full generation与catalog input generation；fingerprint `297403893dc7a3935f99cd754fa1be13b1dcec2f4aab00f3dd3224461cad3f00` |
| Editor consumers | **84 / 10,043 / 9,192 / 358,671 / 102** | asset manager boundary、watcher、refresh、event bridge与retained host startup；fingerprint `025eb008f1ac6d08ea62a93e5fed83830291ac29659874f7944e57504bcc4bfc` |
| 五引擎参考切片 | **18 / 16,251 / 14,113 / 596,488 / 44** | Unreal DirectoryWatcher/FileCache/AutoReimport、Godot EditorFileSystem、Bevy watcher/processor、Fyrox resource events与Unity Graphics reimport consumer；fingerprint `58bfc1355bd03ce303ada402b78ea78c4019c0c6c7ca191e51db68b96760eb13` |

当前双层有界变化队列、project activation/deactivation、candidate generation、targeted/full import以及Editor asset delta桥接是真实可保留底座。冻结时8个范围内路径已有非本轮dirty修改，报告按working-tree current source取证，并记录baseline HEAD与协调器epoch。

本轮新增三项P0。第一，普通`notify::Error`只发布诊断，不把project标为dirty，也不请求完整reconcile；只有错误队列自身溢出才触发。第二，reconcile虽能提交新generation，却仍发布原始输入变化；overflow可使变化列表为空，而Editor没有generation wake订阅且只在非空delta时刷新，Runtime与Editor可永久分叉。第三，Compound source成员的单文件Modified会被shape heuristic当作独立Single source targeted import，产生错误成员sidecar并让父Compound保持陈旧。

参考冻结表明：Unreal将平台buffer失真提升为rescan并以持久FileCache重建差异；Godot用完整filesystem snapshot、dependency closure和更新后信号发布产品；Bevy区分Asset/Meta/Folder/RemovedUnknown与source identity并测试反向依赖；Fyrox以`need_rescan()`重建资源并处理metadata event；Unity Graphics本地corpus只作为重导入consumer旁证，不用于推断完整AssetDatabase实现。

报告登记3项P0、48项P1、12项P2与48项资格门；Runtime25/51/53/64/85/86/87与Editor57继续唯一拥有filesystem、registry、scene reload、resource、build graph、schema、reference和workspace父边界。目标是`AssetWatchAuthority + AssetChangeAccumulator + AssetSourceOwnershipIndex + AssetReconciliationPlanner + AssetGenerationCommit + AssetGenerationDelta + AssetProductDelivery`。本轮只修改review文档与索引，未运行Cargo、Editor、真实filesystem fault、overflow、rename storm、soak或benchmark；tooling按用户要求暂不纳入。详见`zircon_runtime/88-runtime-asset-watch-change-ingress-coalescing-rename-overflow-targeted-reimport-generation-reload-product-integration-current-source-review.md`。

## 245. Runtime Render Graph / Builder / Compiler / Lifetime / Culling / Aliasing / Barrier / Queue / Execution 当前源码物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test markers | 本轮证据 |
|---|---:|---|
| Render Graph core production | **8 / 3,659 / 3,372 / 130,450 / 4** | builder、compiler、graph、types、error、dump与store lint；fingerprint `87a894c9aa9966d3351677d07d23d312ff523d2b87886cf5444178719aefd6df` |
| Render Graph core tests | **10 / 2,626 / 2,423 / 90,709 / 58** | handle、cycle、ordering、version、culling、alias、compute与scale；fingerprint `4e1a47650d9d6d375cf47c3a6e8638a0660419d023058ce631f7ad1a0db50ab4` |
| Product compile/materialization/execution | **108 / 30,891 / 28,889 / 1,151,567 / 274** | pipeline authoring/cache、materialization/pool、resource lookup、encoder、stage、submit与focused tests；fingerprint `23e4cb23632d63f05a52e9c4b7a4d423cd18cc2d4def7265cec822ae195eda63` |
| 五引擎参考切片 | **36 / 34,455 / 29,123 / 1,504,933 / 128** | Unreal RDG、Unity RenderGraph、Godot barrier graph、Bevy command recording与Fyrox graphics/readback；fingerprint `209cbf8ee478b207b4ac28af7e01d0dad807445945b5c2e3e9bfcba613191282` |

当前generation-scoped handle、logical resource version、RAW/WAW/WAR、version-aware culling、descriptor-keyed transient slot、WGPU materialization与CPU parallel command recording是真实底座。旧Runtime09A中foreign handle可误用、资源无version、产品pass全局串链三项描述已被当前源码修复；barrier、native queue与GPU completion仍由09A拥有。

本轮新增三项P0。第一，pass name唯一性只在生成pass插入前校验，核心builder允许重名，产品执行又按name取第一个graph pass；IBL/transparent等生成pass可被同名插件pass替代并形成重复/漏执行。第二，SparseReserved被allocation/materialization跳过且validation仍判完整，但WGPU backend明确不支持并拒绝该residency。第三，插件storage texture缺typed descriptor，未知名称默认推断为`Rgba8UnormSrgb`，materializer静默剥离storage usage，generic compute直到执行期才拒绝。

Unreal RDG证明subresource、barrier、async fork/join、fence-qualified lifetime和transient acquire/discard必须来自同一compiler；Unity RenderGraph证明versioned handle、Cull/Merge/Lifetime/Sync/Compact阶段与async tests的最低线；Godot只作为subresource barrier旁证，Bevy只作为CPU recording/submit owner旁证，Fyrox不具备同级RDG。

报告登记3项P0、48项P1、12项P2与48项资格门；目标是`RenderResourceSchemaCatalog + RenderGraphNormalizer + RenderGraphCompiler + DeviceQualifiedRenderGraphPacket + RenderGraphFrameTransaction`。backend adapter/device/capability/resource/queue/GPU completion的当前源码owner由Runtime90接替，Runtime09A保留历史总览。本轮只修改review文档与索引，未运行Cargo、真实GPU、RenderDoc、device loss、multi-queue、fault、soak或benchmark；tooling按用户要求暂不纳入。详见`zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md`。

## 246. Runtime RHI / WGPU / Device / Submission / Completion / Readback / Surface 当前源码物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test markers | 本轮证据 |
|---|---:|---|
| `zr_rhi` production | **10 / 3,545 / 3,183 / 105,878 / 4** | capability、descriptor、device、handle、surface与UI contract；fingerprint `b4008a1546a4bbc320fa9a37e3aef060c23d0315a2c69c3d9d967bdec968b53a` |
| `zr_rhi` focused tests | **5 / 1,255 / 1,167 / 43,529 / 31** | boundary、capability、descriptor与UI contract；fingerprint `1df874b23df91a668ed32833ffeb4024a4c2e6af25eb35b57514f9ea5e785d5f` |
| `zr_rhi_wgpu` production-path files | **31 / 10,544 / 9,853 / 385,818 / 36** | capability、readback、timer/statistics与完整UI surface子树；fingerprint `0f258e4dab3f5768b399f513b5968f5d0960486251d538f368044c0fb2b10ad8` |
| `zr_rhi_wgpu` focused tests | **30 / 8,700 / 8,067 / 292,637 / 180** | resource/command/pass/copy/readback/UI/native submission/framework boundary；fingerprint `8fc67cf4a11aa57178e5a023b952890b60b084d9ac2d2ef55b71fb872f9bcc22` |
| 产品WGPU集成切片 | **34 / 12,500 / 471,829 bytes** | device request、blocking readback、compiled/direct submit、surface、streaming、IBL与history；fingerprint `e114e585de8c43861cee54745d3c53f69dfe40fafcbc74aa5b44812bac12eb95` |
| 五引擎参考切片 | **19 / 28,985 / 24,675 / 1,228,675 / 29** | Unreal submission/retirement/crash、Godot queue/swapchain/fault、Bevy WGPU recovery/readback、Unity Graphics pool/readback与Fyrox最低接口；fingerprint `4a284efbef6a415a6e038a411e74b6d5d0778506392787515fb691f191ebd857` |

当前中立descriptor、typed enum、validation、WGPU capability mapper、三槽聚合readback staging、GPU timer/statistics、shared-device UI context、retained cache和局部budget是真实可保留底座。产品compiled scene也已把graph command buffers集中为一次submit，但它不是全局submission owner。

本轮新增三项P0。第一，`zr_rhi::RenderDevice`没有production实现且因generic label非object-safe，`zr_rhi_wgpu`唯一device实现被`#[cfg(test)]`隔离，framework test还明确固化产品绕过中立RHI。第二，capability mapper声明Graphics/Compute/Copy、async copy、indirect/multi-draw和capture，但neutral command/queue没有对应可执行operation，admission可false positive。第三，全产品没有DeviceGeneration、SubmissionTicket、targeted completion、deferred retirement或device-loss supervisor；readback ticket/取消/Drop/配额/同device证明与surface cache readiness因此无法形成一次终态。

Unreal证明queue-local monotonic fence、cross-queue sync point、interrupt completion、completion-qualified payload/resource release和DRED/fault artifact必须属于同一submission owner；Godot证明中立driver至少应表达typed resource、queue family、fence/semaphore、swapchain/HDR、memory/limit/API trait与device fault。Bevy提供WGPU device-lost/uncaptured-error recovery状态机交叉检查；Unity Graphics只作为readback subresource和pool provenance旁证；Fyrox只作为object-safe backend/async read buffer最低线。

报告登记3项P0、48项P1、12项P2与48项资格门；目标是`RhiDeviceOwner + DeviceProfile + ResourceRegistry + SubmissionService + CompletionService + ReadbackService + SurfaceService + DeviceErrorSupervisor`。Runtime90接替09A的RHI currentness，Runtime89继续拥有Render Graph builder/compiler/execution packet，Runtime79继续拥有UI渲染算法。本轮只修改review文档与索引，未运行Cargo、真实GPU、RenderDoc、device loss、multi-queue、fault、soak或benchmark；tooling按用户要求暂不纳入。详见`zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md`。

## 247. Runtime Material / Shader Module Graph / Permutation / Compiler / Reflection / Pipeline / PSO Cache / Prewarm / Hot Reload 当前源码物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test markers | 本轮证据 |
|---|---:|---|
| Runtime production owner roots | **194 / 42,335 / 38,961 / 1,553,883 / 279** | asset、framework、material、shader、pipeline、Mesh cache、prewarm CLI与dynamic API；fingerprint `572cc9bb3280c7059e5db917a05bcb0db575a7110192bbb7a2b8a18e3ff53d82` |
| Runtime focused tests | **36 / 12,831 / 11,940 / 482,583 / 204** | source、template、cache、async、prewarm与product guards；fingerprint `9467b7fa47ff9afe252dd386352247b6eefd996f0eed91267ba8d4ab40868862` |
| canonical WGSL | **42 / 3,828 / 3,523 / 133,668 / 0** | material、geometry、environment、pass与shared include源码；fingerprint `ceb8c33da67ee71f2bb089d718d39e4318d5e10348ccdb250c363dda49ac13d2` |
| Authoring plugin production | **15 / 1,552 / 1,410 / 58,169 / 13** | Material Editor、Shader Graph与WGSL importer产品实现；fingerprint `3ac21e5e23aa9c4173143e2040bf790c2c98c7b3cd826096df8dd367daa13951` |
| Authoring plugin focused tests | **1 / 277 / 247 / 9,052 / 9** | Material Graph compiler/descriptor tests；fingerprint `5f1bddddec1991432fe82e10139dd961673b31ab32f0e8e1b5c7322282eb77f4` |
| direct shader/pipeline/cache callsite corpus | **87 / 18,994 / 17,819 / 735,183 / 108** | 全产品直接WGPU创建与cache接入；fingerprint `4a1bf070cdad79849de91a2f6982a04ed3c6787a19d765084f5cfc776186fecc` |
| 五引擎参考切片 | **23 / 37,144 / 32,026 / 1,436,530** | Unreal compiler/job/PSO/library、Bevy cache/reload、Godot version/reflection、Fyrox asset/editor与Unity Graph/stripping；fingerprint `7664495eb87579a86166effc315e932a72df6b9bb7276ea10fae439984c80e8b` |

`.zshader v2`、Material ABI、Geometry/Shading registry、content-addressed prewarm source table、bounded asset inventory、SCC module DAG、Mesh exact WGSL source hash与局部Naga/WGPU tests是真实可保留底座。Runtime09C关于prewarm无DAG、重复无界扫描与Mesh key完全不含source的描述已经过时，本轮按current source纠正。

本轮没有新增重复P0；Runtime09C的7项P0仍开放并由其唯一计数。当前production-like机械扫描仍有81处`create_shader_module`、49处`create_render_pipeline`、27处`create_compute_pipeline`，71处`cache: None`，`cache: Some`只有`pipeline_cache_gate.rs`内部1处。Shader readiness不要求非空entry/kind stage/layout，raw WGSL importer固定Surface并清空schema/layout；Mesh async默认关闭且只覆盖Base，失败固定保留`SkipDraw`。

作者链同时存在Material Editor graph、rendering Shader Graph和graphics shader DTO三套模型。Material Editor声明的ZUI/template不存在，dist无command invocation/bridge；compiler只常量折叠为传统MaterialAsset。Shader Graph按Vec顺序拼WGSL且executor为noop，均未进入canonical artifact/PSO generation。

报告新增0项P0、48项P1、12项P2与48项资格门；目标是`ShaderSourceAuthority + ShaderModuleGraphCompiler + ShaderArtifactService + ShaderReflectionArtifact + ShaderPermutationDomain + PipelineLayoutCatalog + PipelineService + PipelineCacheStore + ShaderPrewarmService + ShaderGenerationCoordinator + PreparedMaterialService + ShaderAuthoringService`。本轮只修改review文档与索引，未运行Cargo、Editor/App、真实GPU、RenderDoc、cook/export、device loss、compile storm、soak或benchmark；tooling按用户要求暂不纳入。详见`zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md`。

## 248. Runtime Texture / Image / Cubemap / Array / Volume / Format / Sampler / Mip / Compression / Upload / Streaming / Residency / Budget / Eviction / Virtual Texture 当前源码物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test markers | 本轮证据 |
|---|---:|---|
| Runtime production owner roots | **99 / 23,316 / 21,577 / 847,675 / 109** | asset texture、artifact/load、image contract、GPU resource、streamer与budget；fingerprint `38e01a21e18e2577f7ead0e9e5ad286bf1c0dda7d12edefc70937e30da8953ff` |
| Runtime focused tests | **4 / 1,375 / 1,218 / 44,974 / 57** | upload、mip planner/state、budget与结构guard；fingerprint `d86b7958e06d95acfb1c67425d392a79b7597a44e11df31b01afde2c6bab761c` |
| Texture importer production | **22 / 5,123 / 4,821 / 179,512 / 12** | image/PSD/container、array/cube、mipgen、normal与BC5；fingerprint `ed34f2c3112f3df341fd49ef980ffb04ac6c96712f81a174ae56434595b01efd` |
| Texture importer tests | **18 / 4,560 / 4,014 / 150,050 / 162** | DDS/KTX/ASTC、settings、mipgen、BC5与diagnostic；fingerprint `e6ef2375ef9fda433c3fcde34998e44a7109d89397ef5d1c1c050cbe74c7b009` |
| direct texture callsite corpus | **86 / 27,520 / 25,700 / 1,031,654 / 164** | Runtime与Plugins production-like直接texture create/write/copy全集；fingerprint `0ab50bc45d5bc393907ea778694926b76459aa7fe6df28357a669b6f3e048de5` |
| 五引擎参考切片 | **26 / 19,780 / 17,066 / 758,851 / 15** | Unreal streamer/VT/SVT、Bevy lifecycle、Godot storage、Fyrox async cache与Unity VT/atlas；fingerprint `379de2c10956ef7aa17fde616049d664a8a3274259b81bbc0ecc5fb99623defb` |

DDS/KTX/KTX2/ASTC upload plan、KTX2 Zstd/Zlib展开、BC5 normal转码、Box/Kaiser mip kernel、artifact integrity和mip planner是真实可保留底座。Runtime09D关于预算完全未接线与所有KTX2 supercompression均未处理的描述已被current source纠正；固定1 GiB persistent texture budget确已传入streamer，标准Zstd/Zlib也已在importer展开。

本轮没有新增重复P0；Runtime09D的6项P0仍开放并由其唯一计数。whole bincode + generic 64 KiB chunk不能按mip/layer/page读取，submission仍同步load/decode/clone/create，首次full resident后同帧同步replacement，compressed无partial residency；demand只含main-view material texture且使用transform scale近似，ordinary D3与SVT/VT产品链缺失，persistent预算不覆盖special/history/retired等多数纹理。

生产式机械扫描有86处texture create、83处write、12处texture-to-texture、3处buffer-to-texture和9处texture-to-buffer copy。报告新增0项P0、48项P1、12项P2与48项资格门；目标是`TextureSchemaAuthority + TextureBuildService + TextureArtifactStore + TextureGenerationService + TextureUploadService + TextureResidencyService + TextureBudgetController + SamplerCatalog + VirtualTextureService`。本轮只修改review文档与索引，未运行Cargo、Editor/App、真实GPU、RenderDoc、cook/export、device loss、OOM、soak或benchmark；tooling按用户要求暂不纳入。详见`zircon_runtime/92-runtime-texture-image-cubemap-array-volume-format-sampler-mip-compression-upload-streaming-residency-budget-eviction-virtual-texture-product-integration-current-source-review.md`。

## 249. Runtime Mesh / Geometry / Section / LOD / Instancing / Skinning / Morph / Deformation / Bounds / Collision / Streaming 当前源码物理范围

| 范围 | 文件 / 行 / bytes / test markers | 本轮证据 |
|---|---:|---|
| Mesh-related production / contract / consumer corpus | **420 / 80,945 / 2,972,710 / 未单独归一** | asset mesh/model、import、scene、render extraction、GPU resources、streaming、deformation、physics及插件产品消费者；fingerprint `c891c859fd819d3999d352f800a9057a61a8967694d3d8060ccd19c202d5ecd8` |
| Runtime与插件focused tests | **37 / 10,746 / 387,698 / 108** | mesh/model load、import、scene binding、section extraction、GPU cache、glTF与physics边界；fingerprint `ac485a918e5690991cb795d2b847ff43bcd81761196662fedf76b4111083ef5f` |
| 五引擎参考切片 | **35 / 49,725 / 1,989,302 / 0** | Unreal Static/Skeletal Mesh、LOD/section/streaming/skin cache，Bevy skin/morph/bounds，Fyrox surface/resource，Godot surface/import/collision，以及Unity Graphics instance/LOD/AABB；fingerprint `f551a04763c17c29385907055400940689723a17430d59ef0c1d905d7a02e6f5` |

typed mesh/model document、section与LOD schema、artifact integrity、GPU mesh cache、morph changed-row upload、visibility/streaming入口及独立PhysicsMeshAsset是真实可保留底座。冻结范围中存在其他Session的current-source dirty修改，本报告按working tree取证并记录baseline HEAD与协调器epoch，只写review文档和索引。

本轮不重复新增P0；Runtime69记录的MeshRenderer材质override/tint/alpha mode无法经过scene project I/O的P0仍开放并由其唯一计数。新增48项P1与12项P2：高优先级glTF插件比builtin丢失更多属性且现有测试直接调用builtin；ZMeshDocument不校验版本，validation不足，Model/Mesh保留双份几何并可静默回退；LOD只按平移距离，stable key截断ordinal，`static_batches`不是实际批提交，GPU Mesh始终构建wire segments；GPU Scene每draw固定`register(..., 1)`且以translation加最大轴缩放单位球估算bounds；GPU skin在GPU路径前仍做CPU skin，palette忽略导入inverse bind并吞掉joint overflow；collision cooker则把triangle mesh拆成逐三角shape并聚合为静态compound，builtin backend又拒绝mesh/heightfield/compound。

五引擎参考共同要求把source/imported mesh、cooked platform artifact、render resource、section/LOD、deformation、bounds、collision与streaming lifetime分层。Unreal证明LOD resource/section、streaming与GPU skin cache需要generation和fence控制；Bevy明确拥有inverse bind asset、per-joint skinned bounds及current/previous morph storage；Fyrox与Godot证明surface/material/resource、LOD/import/collision cache应为正式资产边界；Unity Graphics本地语料只用于persistent instance handle、world AABB、LODGroup与batched GPU update/cull旁证，不推断其缺失的原生importer/physics实现。

报告登记0项新增P0、48项P1、12项P2与48项资格门；目标是`MeshSchemaAuthority + MeshBuildService + MeshArtifactStore + MeshGenerationService + GeometryResidencyService + MeshDrawPacketCompiler + DeformationService + MeshBoundsService + MeshCollisionCookService`。本轮未运行Cargo、Editor/App、真实GPU、RenderDoc、cook/export、physics stress、soak或benchmark；tooling按用户要求暂不纳入。详见`zircon_runtime/93-runtime-mesh-geometry-section-lod-instancing-skinning-morph-deformation-bounds-collision-streaming-product-integration-current-source-review.md`。

## 250. Runtime Visibility / Spatial Index / Bounds / Frustum / Occlusion / HZB / Culling / Batching / Instancing / GPU Scene / Indirect Submission 当前源码物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test markers | 本轮证据 |
|---|---:|---|
| Runtime visibility / GPU Scene / HZB / mesh submission产品语料 | **188 / 35,468 / 32,436 / 1,286,197 / 288** | visibility、GPU Scene、HZB、mesh build/pass、extract与shadow consumer；fingerprint `ff7170941560799d45029169bef8b580c06f0252702b28eb2fe7873cacc0da66` |
| focused visibility tests | **4 / 2,136 / 1,979 / 77,070 / 25** | graphics visibility外部测试；内嵌GPU/HZB/indirect tests计入产品语料；fingerprint `f70f4cb4927d84fd1061fe2340d2a8d716466f987bcede9f76ea28d3d8639b5e` |
| 五引擎参考切片 | **35 / 39,103 / 33,396 / 1,644,455 / 17** | Unreal GPUScene/SceneCulling、Bevy preprocess/meshlet、Fyrox visibility/octree、Godot scene cull与Unity GPUDriven；fingerprint `c121798ebe43fda7c40b3fe78ac57f55e9769885b020709735503fa284b8d465` |

逐view `FrameVisibility`、GPU Scene stable span/dirty upload/current-previous history、真实GPU HZB compaction、indirect count与replay是当前可保留底座。Runtime09B关于逐view identity丢失和indirect仅为CPU命令的旧子句已过时：shadow renderer现在按view key消费精确visible entity set，HZB compute也会重写draw args、count和instance remap。

但完整GPU-driven产品链仍未形成。`VisibilityContext`每帧从完整extract重建BTree/Vec平行集合；资源已有真实local/morph bounds，`RenderMeshSnapshot`却不传递，visibility和GPU Scene分别以translation/scale单位球近似，HZB shader再应用一次instance transform。空间索引是单层uniform grid，历史snapshot下增量更新可触发COW整图复制，大bounds进入全局overflow，过多cell退化为全entry。关闭阴影的方向光仍制造一个shadow visibility view。

Mesh load/prepare/material/deformation/pending command与全GPU Scene sync发生在visibility过滤前；每pending draw固定`instance_count = 1`，visibility instancing/upload/HGI计划无产品消费者。CPU先构造完整command、indirect args、metadata和candidate，GPU只做末端压紧；`TwoPhaseRetest`有枚举无executor，产品仅previous-frame HZB单pass且只覆盖部分phase。

报告新增0项P0、48项P1、12项P2与48项资格门；Runtime09B的7项P0继续由其唯一计数。目标是`RenderSceneService + RenderBoundsService + SpatialSceneIndex + ViewFamilyService + GpuSceneService + VisibilityPipeline + MeshDrawPacketCompiler + GpuSubmissionPlanner + VisibilityHistoryService`。本轮只修改review文档与索引，未运行Cargo、Editor/App、真实GPU、RenderDoc、10K/100K soak、device loss、OOM或benchmark；tooling按用户要求暂不纳入。详见`zircon_runtime/94-runtime-visibility-spatial-index-bounds-frustum-occlusion-hzb-culling-batching-instancing-gpu-scene-indirect-submission-instance-lifecycle-product-integration-current-source-review.md`。

## 251. Runtime Direct Lighting / Photometry / Light Grid / Clustered Forward+ / Shadow Atlas / Cascade / Point-Spot-Rect / Cookie / IES / Submission 当前源码物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes | 本轮证据 |
|---|---:|---|
| Runtime direct lighting / grid / shadow / cookie / contact产品语料 | **97 / 21,581 / 20,101 / 848,142 / 146** | authoring、extract、ABI、shader、grid、plan、visibility、atlas、cache、graph与submission；fingerprint `41878c168236e1c703bd1c402e10dbaab67f061ef2e08db6f457bb5f3813f545` |
| focused shadow产品测试 | **7 / 2,997 / 2,836 / 107,404 / 22** | shadow graph、capture、wide、many-light与scene extract；fingerprint `f8bf45553f362bb5e74ee27c98b55ad091323b0be0f6c22b35e274d769ac7db8` |
| 五引擎参考切片 | **48 / 50,572 / 43,415 / 2,180,867 / 0** | Unreal LightGrid/shadow/VSM/light authoring，Bevy cluster/shadow，Godot cluster/area light，Fyrox light/CSM与Unity HDRP/URP light loop、shadow、cookie、IES/LTC/contact；fingerprint `46c6ee7e634b80f60a93d4adbe728365cdcefad20dd5c822dc3e3e58fd2d4846` |

固定128-byte `GpuLightData`、被forward/deferred/froxel实际读取的CPU light grid、带generation与preemption的shadow atlas allocator、稳定cascade split/texel snapping、方向光级联/point六面/spot视图、PCF、WGPU capture与`ShadowCache` identity kernel是真实可保留底座。但普通scene asset/component没有shadow、cookie、IES、photometric unit或lighting channel authoring，world extract对四类直接光强制写`shadow: None`；现有产品测试通过手工构造snapshot/settings绕过断链，readiness则按light数量直接报ready。

材质实际消费的grid仍在CPU每帧pack、分配并全量upload；标成`AsyncCompute`的第二条路径只汇总有限方向光为二维颜色buffer，却宣称写入grid params/z-bins/tile masks/light list。CPU路径又静默截断65,535盏光、按sphere处理spot/rect、错误剔除near-plane crossing与camera-inside大光源，并有orthographic尺寸重复减半。forward/deferred复制非物理punctual loop，Rect退化为point light，layer、shadow strength/normal bias与IES没有有效shader闭环。

shadow planner、visibility、atlas allocation与cache没有共享的prepared generation。planner只选第一盏投影方向光且使用固定near/cascade/distance，visibility却为未投影方向光和全部point/spot预建视图；directional cascade绕过allocator并伪造generation。`ShadowCache`无生产consumer，graph每帧`clear_store`整张atlas并为每个slot创建uniform/bind/pass后重放完整caster stream。cookie atlas固定8x8/64格且每帧重建，无residency/overflow receipt；IES没有runtime contract，contact shadow则将单通道后置遮蔽乘到包含ambient、IBL、baked、reflection与emissive的整张scene color。

报告不重复登记父P0：Runtime09E的10项P0与Runtime71的2项P0均仍开放并由原owner唯一计数。本文新增0项P0、48项P1、12项P2与48项资格门，目标是`LightSourceAuthority + PreparedLightingGeneration + LightAssignmentService + DirectLightingService + ShadowPolicyService + ShadowAssignmentService + ShadowViewService + ShadowAtlasService + ShadowCacheService + LightProfileResidencyService`。本轮只修改review文档与索引，未运行Cargo、Editor/App、真实GPU、photometric migration、golden、cluster overflow、atlas thrash、device loss、24h soak或同画质Unreal benchmark；tooling按用户要求暂不纳入。详见`zircon_runtime/95-runtime-direct-lighting-photometry-light-grid-clustered-forward-plus-shadow-atlas-cascade-point-spot-rect-cookie-ies-submission-product-integration-current-source-review.md`。

## 252. Runtime Environment / Sky / Atmosphere / Cloud / IBL / Reflection Probe / Capture / Convolution / Cache / Residency / Submission 当前源码物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes | 本轮证据 |
|---|---:|---|
| Runtime environment / IBL / probe产品语料 | **131 / 32,653 / 29,938 / 1,189,658 / 281** | scene/asset/editor入口、neutral contract、artifact、bake、cache、upload、probe、shader、capture plugin与submission；fingerprint `da296a02877523a0ce25cd19b749f4519ba03b179269c83d76cdff711fb3b663` |
| focused product tests | **4 / 1,593 / 1,481 / 57,981 / 19** | project render、scene extract与authoring boundary；识别手工environment注入和ignored capture；fingerprint `ca142fbc5f67202b56c91f40f91f84e1befebb5f64d503c674edc7ac87d48340` |
| 五引擎参考切片 | **27 / 26,640 / 22,700 / 1,192,747 / 0** | Unreal atmosphere/cloud/skylight/probe、Unity HDRP、Bevy、Godot与Fyrox；fingerprint `fe5bfa503c2d873adbe8667d2e3ee4af2a2cc84028f0a5a427d97f390ea812f3` |

Versioned IBL recipe/artifact、canonical cubemap projection、CPU/GPU PMREM与SH9、prepared upload、probe influence/top-two/slot generation，以及realtime IBL generation token、双缓冲、按face/mip切片、compiled topology cache和timestamp是真实可保留底座。旧09F1关于重复`CaptureCloud`和逐帧深复制Arc payload的描述已被current source纠正：重复capture已删除，clone主要是浅拷贝。

普通scene asset/component/NodeKind仍没有Environment、Sky、Atmosphere、Cloud或Reflection Probe，World只把Editor viewport的`preview_skybox`转换成默认gradient；产品测试手工替换`EnvironmentExtract`，Editor capture trigger没有`zircon_editor`消费者。天空仍是三色插值+sun disc，没有物理介质/LUT/cloud/shared sun truth。cold submission同步read/decode，hydration与pending bake固定4项，cubemap prepare可私自submit；probe固定64×128 RGBA16F及planar资源约74.67 MiB，新probe执行48次write，fragment扫描全部active probe且不消费已打包layer mask。

报告新增1项P0：`zircon_plugins/Cargo.toml`中的reflection-probes workspace member调用已删除的`SceneRenderer::render_scene_color_hdr`，当前公开HDR capture属于RenderFramework边界；本文未运行Cargo，因此该结论是静态类型面阻断而非伪造编译日志。Runtime09F1的10项父P0继续唯一计数；另登记48项P1、12项P2与48项资格门，目标是`EnvironmentSourceAuthority + EvaluatedEnvironmentGeneration + EnvironmentResourceManager + EnvironmentJobScheduler + ProbeAssignmentService + EnvironmentSubmissionService + EnvironmentAuthoringService`。本轮只修改review文档与索引，未运行Cargo、Editor、真实GPU、fault、scale、soak或同画质Unreal benchmark。

## 253. Runtime Baked Lighting / Lightmap / Probe Volume / Bake Job / Artifact / Residency / Sampling 当前源码物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test markers | 本轮证据 |
|---|---:|---|
| Runtime baked-lighting产品语料 | **77 / 15,735 / 14,606 / 690,650 / 79** | DTO、scene、mesh UV import、asset、offline bake、GPU scene、binding、shader、plugin、Editor表面与owner计划；fingerprint `f53750186ac3870c75f5f1a90fe723a069f86d28a66b0733719daa9e181a26fd` |
| focused tests与fixture | **9 / 2,554 / 2,355 / 90,731 / 35** | lightmap contract/binding、glTF UV1、外部fixture、advanced-lighting与Editor capability；fingerprint `b2bb4cca780e36957c3b2c6b87a10b4dc8fa06be29bb7a82432db47d0b3b715b` |
| 五引擎参考切片 | **31 / 26,018 / 21,742 / 1,133,143 / 8** | Unreal build registry/GPULightmass/VLM、Unity APV、Godot LightmapGI、Fyrox baker与Bevy消费侧；fingerprint `0990fd47f4a46704ca27172d9fc178831dfad09aa46a746b4a950c40ac7c1141` |

glTF `TEXCOORD_1`到GPU vertex、`LightmapBakeOutput -> TextureAsset/ConsumeContract`、Static mesh slot、SH9 CPU/GPU sampler、irradiance-volume transform与手工Forward/Deferred产品fixture是真实可保留底座。旧09F2关于“没有UV1通道”和“没有任何mobility”的措辞已被current source纠正：当前有UV1通道搬运和通用Static/Dynamic mobility，但没有lightmap unwrap/chart/overlap/padding/atlas工程，也没有Static/Stationary baked direct、shadowmask和light identity语义。

普通scene/prefab/project不能保存light build settings、build-data引用或probe-volume source，World不会创建BakeRequest或装载artifact，生产代码没有BakeOutput producer。`offline_bake_frame`只把方向光强度求和并制造没有cubemap的reflection probe，而资源准备会丢弃这类probe；Editor Lighting Bake工作区硬编码City_Block_A、87 assets、4 warnings与02:30，Bake/Preview只改queued字符串。baked plugin注入no-op scene-color pass，feature flag只切换两个相同零值分支，core/template仍始终准备并采样baked resources。

旧09F2的12项P0全部保持开放并由其唯一计数，本报告不重复新增父P0；另登记36项P1、8项P2与44项资格门。目标是`Scene Light Build Truth + Canonical BakeManifest + Bake Service + Build Data Registry + Resident Generation + Per-object Probe Assignment + Physical Baked Shading + Lighting Bake Editor`。本轮只修改review文档与索引，未运行Cargo、Editor、真实GPU、bake、cook、streaming、fault、soak或同画质Unreal benchmark；tooling按用户要求暂不纳入。详见`zircon_runtime/97-runtime-baked-lighting-lightmap-probe-volume-bake-job-artifact-residency-sampling-product-integration-current-source-review.md`。

## 254. Runtime Hybrid Global Illumination / Scene Representation / Surface Cache / Global SDF / Screen Probe / Radiance Cache 当前源码物理范围

| 范围 | 文件 / 行 / bytes / test attributes / ignored | 本轮证据 |
|---|---:|---|
| HGI package全部tracked文件 | **270 / 42,539 / 1,585,115 / 239 / 21** | manifest、runtime、shader、tests、Editor与dist；fingerprint `a52c3c99e4215c4b680e503c454fe6e511c6d019753238bf4909bbc087780d8a` |
| runtime production-like源码 | **221 / 29,964 / 1,105,137 / 98 / 1** | provider、representation、GPU lifecycle、16个WGSL shader与四pass executor；fingerprint `5a52d50e99cf296382c8085b6710fd8f80485b921cabc479620cccb90f039f9b` |
| tests、test sources与support | **39 / 12,241 / 468,626 / 138 / 20** | CPU/WGPU、PNG exporter、profile、invalidation与product fixtures；fingerprint `3e2b35f0129c2b14a82db573303dcdd35ceaec2c5e247d2f26934f2e0674afb7` |
| package、Editor与dist | **10 / 334 / 11,352 / 3 / 0** | capability、carrier、extension和默认装配；fingerprint `0c0d3e7ad7379785937d936afd56ea1ec6ddecb69290be1b85a73ca344ecfa0e` |
| 五引擎参考切片 | **35 / 36,091 / 1,562,537 / 未归一 / 未归一** | Unreal Lumen、Unity APV/SSGI/RT、Godot SDFGI、Bevy probe与Fyrox deferred owner；fingerprint `89670ea60492310877cc82a0d96b1a127cbcac1a51bd633511eafb3b2b9a2322` |

first-party package、typed provider、四pass graph、Mesh SDF artifact、camera-snapped Global SDF、Surface/voxel state、software trace、temporal resolve和大量characterization tests是真实可保留底座。旧09F3关于“最终输出纹理只有8x8”的措辞已被收紧：attachment确实为viewport尺寸，但scene/trace payload只有固定8x8、64 tile，所有full-res像素仍从这64份深度/法线/命中/radiance信息重建。

Surface Cache每页仍只有中心UV生成的两个RGBA8样本和量化depth，每mesh一卡、每卡最多一probe；Visibility接收HGI extract后明确构造空probe/update/feedback/request。plugin四pass与core post-process的16 probe/16 trace region路径重复拥有GI和history；正常路径把cache、atlas、capture、voxel、RC和trace结果广泛readback到CPU，在全局Mutex collector内重打包后再上传。Radiance Cache固定32 probe并把同一RGB8值写满2x2 interior，Global SDF无lineage时制造蓝灰色，production没有ray query、AS、BLAS/TLAS、SBT或dispatch rays。Scene无持久HGI truth，Editor强制启用并写死32/64/16预算，插件引用不存在的`authoring.zui`。

旧09F3的14项P0全部保持开放并由其唯一计数，本报告不重复新增父P0；另登记36项P1、8项P2与44项资格门。当前working tree把Surface Cache slot membership改为`BTreeSet`，只关闭局部O(n²)查询，不关闭整帧clone、CPU authority或residency根因。目标是`HybridGiSceneCompiler + HybridGiArtifactStore + HybridGiRenderSceneService + SurfaceCacheResidencyService + GlobalSdfResidencyService + HybridGiTraceBackendRegistry + RadianceCacheService + HybridGiReconstructionService + HybridGiBudgetController + HybridGiAuthoringService`。本轮只修改review文档与索引，未运行Cargo、Editor/App、真实GPU、RenderDoc、cook/export、fault、scale、soak或同画质Unreal benchmark；tooling按用户要求暂不纳入。详见`zircon_runtime/98-runtime-hybrid-global-illumination-scene-representation-surface-cache-global-sdf-screen-probe-radiance-cache-product-integration-current-source-review.md`。

## 255. Runtime Volumetric Fog / Froxel / Local Fog Volume / Lighting / Shadow / History / Temporal Reprojection 当前源码物理范围

| 范围 | 文件 / 行 / bytes / test attributes / ignored | 本轮证据 |
|---|---:|---|
| current symbol-bearing与focused总语料 | **146 / 42,278 / 1,676,278 / 280 / 5** | Scene、extract、plugin、graphics、shader、Editor、tests与artifact索引；fingerprint `c09adacfa7d853420ee7d85f4be75970ffa4b20424a4fdc368f5fe927757f591` |
| production-like源码 | **119 / 32,405 / 1,301,627 / 149 / 0** | typed contract、World extract、三段froxel graph、history、OIT、diagnostics与plugin wiring；fingerprint `6f1f3169327d922e53e8faa73c7cce5cb6aa0c4e4ea16fd58b3e2cfa5a9bf7dc` |
| focused tests与support | **27 / 9,873 / 374,651 / 131 / 5** | CPU/WGPU contract、PNG exporter、temporal、media、scatter、integrate与product fixtures；fingerprint `d17a848a3ac31160fcc661a445d67c2bd7b061b9510ed8c3bb351149ad110f34` |
| Editor focused set | **8 / 2,296 / 98,451 / 0 / 0** | editor plugin、capability与post-process workspace；fingerprint `6232aac0be053dbec14dc7766a73939df00cb8d7f805d7aea96e0ab3f0d3f97c` |
| product/render artifacts | **20 / 未归一 / 28,250,439 / 未归一 / 未归一** | failed shaft report、PNG与DX12 RenderDoc resource snapshots；fingerprint `5cccf51e9d49ecda25d85c9789f2fbfb9dbc40955bdfe94b3b3f3cffcc2f5066` |
| 五引擎参考切片 | **47 / 21,664 / 929,669 / 未归一 / 未归一** | Unreal、Unity HDRP、Godot、Bevy与Fyrox renderer owner边界；fingerprint `41eb85cca7dfe8b5ecd80c3974cffb65a3b4ce9bdc69a64792c7f2072e58d316` |

first-party plugin、Media Inject/Light Scatter/Integrate三段compute、RGBA16F 3D VBuffer、clustered light/shadow atlas消费、Henyey-Greenstein phase、front-to-back Beer-Lambert积分，以及Forward/Deferred/Sky apply是真实可保留底座。DX12 RenderDoc artifact也证明High 160x90x96资源和dispatch真实存在；但resource存在不等于画质、性能或产品闭环通过。

Local Fog仍由Post Process Volume加Collider临时表达，Sphere与rotated Box在extract时退化为AABB，blend distance、priority和原始shape丢失；XY固定160x90，Z只在48/64/96切换，每froxel无界遍历全部volume。global height density绑定绝对Y=0；pixel jitter直接当froxel cell偏移，history只用nearest固定0.9与extinction阈值拒绝。Rect只剩方向cosine、cookie未采样、environment/baked/HGI未进入体积光照；OIT保存已经fogged的RGBA8 layer再与已经fogged scene合成，particle/sprite绕过该路径。Editor没有真实volumetric authoring，diagnostics也没有GPU ms、occupancy、overflow、history rejection、memory或debug view。

旧09G1的11项P0全部保持开放并由其唯一计数；本文新增1项P0：plugin启用而Scene没有author fog时，fallback post-process settings仍注入`VolumetricFogSettings::DEFAULT`的density=0.02，graph没有per-frame admission来表达真正关闭。另登记36项P1、8项P2与44项资格门。目标是`VolumetricSceneCompiler + FroxelLayoutService + ParticipatingMediaVoxelizer + VolumetricLightingService + VolumetricHistoryService + VolumetricCompositionService + VolumetricResourceService + VolumetricAuthoringService`。

产品artifact `plan18_volumetric_compiled_scene_window_light_shaft_perf_wgpu_20260711.txt`仍为`diagnostic_failed`：4398个shaft samples中brighter为0、平均差-2.347、control同为-2.347、contrast约0；5项exporter/product tests仍ignored，至少6项非ignored WGPU测试在无adapter时直接return。本轮只做静态review和已有artifact复核，未运行Cargo、Editor/App、真实GPU、RenderDoc capture、cook/export、fault、scale、soak或同画质Unreal benchmark；tooling按用户要求暂不纳入。详见`zircon_runtime/99-runtime-volumetric-fog-froxel-local-fog-volume-lighting-shadow-history-temporal-reprojection-product-integration-current-source-review.md`。

## 256. Runtime Particle / VFX System / Emitter / CPU-GPU Simulation / Rendering / Scalability / Determinism 当前源码物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes / ignored | 本轮证据 |
|---|---:|---|
| Zircon production / contract / product | **106 / 18,409 / 16,916 / 675,558 / 78 / 0** | package、asset/component、manager/CPU/GPU、Render Graph、core renderer/history、Scene/Script/Vampire与Editor入口；fingerprint `266455362cdf75cc98005f6dd68c21c89b9620ff024606e61fe4714165eae9ec` |
| dedicated tests / acceptance | **20 / 3,373 / 3,105 / 125,276 / 53 / 1** | CPU/GPU、graph、snapshot、diagnostic、Scene extract与旧acceptance；fingerprint `2497f972e051556a44893b723d44c7b56b58d17bcc33b067bce52e4bf467c7c9` |
| 五引擎参考切片 | **27 / 35,009 / 30,199 / 1,494,534 / 1 / 0** | Unreal Niagara、Unity Graphics VFX Graph、Godot、Fyrox与Bevy render architecture；fingerprint `c7a925264ef06f0ab80494a709b8611952e28ec90a9abcd2ee6a1f783730ea2a` |

当前源码新增的shared `Arc<[T]>` snapshot和256条有界、64条分页、sequence/stale cursor/ack diagnostic是真实进展；旧Runtime26“diagnostic无界/每次snapshot全量clone”已经关闭。旧“aggregate以全局`max_dt`推进所有emitter”也不成立：顶层frame虽保留max摘要，每个emitter仍编码独立dt，WGSL读取`emitter.sim.y`。

核心断层没有关闭：生产代码仍无`ParticlesManager::tick` scheduler，`ParticleSystemComponent`不进入Scene/ECS load/save/attach/detach，Vampire仍通过`set_particle_sprites`写dynamic JSON最终sprite；GPU资产同时由manager CPU fallback与renderer owner GPU simulation推进。Scene还允许调用者自报`gpu_frame` count/bounds，不能作为GPU执行证据。

真实spawn/update、compact和indirect compute在runtime prepare中由owner提前录制，随后才把已执行buffer登记成static external resource；Render Graph三个compute executor只校验resource/queue contract。VFX Graph另定义五节点asset、固定`[1,1,1]` workload和两个直接`Ok(())`的executor。GPU aggregate在拓扑/暂停/asset变化时重建，固定1,048,576 slot按顺序争用，readback queue无本地上限且FIFO队首可阻塞后续完成；CPU/GPU renderer仍缺material/texture完整消费、GPU velocity/history、renderer family与world scalability。

报告新增0项P0，因为particles仍诚实标为experimental/Partial且相关optional feature默认关闭；Editor15的可见假成功P0继续唯一计数。本文登记48项P1、12项P2、M0-M12与44项资格门，目标是`ParticleSourceDocument -> ParticleSemanticCompiler -> CompiledParticleProgram -> ParticleWorldRuntime -> authoritative CPU/GPU executor -> immutable ParticleRenderPacket -> Render Graph`。本轮只修改review与索引，未运行Cargo、WGPU、Editor、RenderDoc、产品场景或benchmark；tooling按用户要求排除。详见`zircon_runtime/99d-runtime-particle-vfx-system-emitter-cpu-gpu-simulation-rendering-scalability-determinism-product-integration-current-source-review.md`。
