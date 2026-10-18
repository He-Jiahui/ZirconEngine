# Sound 插件核心完善计划

## Summary
- 以“核心先行”为第一阶段：把 `sound` 从 clip 播放器扩展为插件拥有的音频引擎，包含中立运行时接口、可测 mixer graph、DSP 链、场景音频组件、插件选项和依赖槽位。
- `zircon_runtime::core::framework::sound` 只放 DTO/trait/handle/error；混音、DSP、空间声学、卷积等重实现放在 `zircon_plugins/sound/runtime`；编辑器工作流放在 `zircon_plugins/sound/editor`。
- 架构参考 Unreal Submix/AudioVolume/Dynamics、Godot AudioServer bus、Fyrox bus graph/HRTF；Bevy 的组件播放模式只作为 ECS 接入参考，不限制能力上限。

## Interfaces And Data Model
- 新增/扩展中立类型：`SoundTrackId`、`SoundNodeId`、`SoundEffectId`、`SoundSourceId`、`SoundListenerId`、`SoundVolumeId`、`SoundParameterId`、`SoundAutomationBindingId`、`SoundImpulseResponseId`。
- 新增 mixer graph 描述：固定 `master` 轨道、自定义轨道、send 边、sidechain tap、effect chain、meter、latency/status snapshot。
- 新增 `SoundEffectDescriptor`：Gain、Filter、Reverb、ConvolutionReverb、Compressor(sidechain)、WaveShaper、Flanger、Phaser/Phase、Chorus、Delay、Pan/StereoWidth、Limiter；统一支持 enabled、bypass、wet/dry、顺序和可自动化参数。
- 扩展 `SoundManager`：clip/source 生命周期、graph 配置、track/effect/send CRUD、参数 set/query、automation binding、listener/source/volume 更新、`render_mix`、meter/status 查询；现有 clip 播放成为新 graph 上的默认 `master` 路由便利路径。
- 注册插件动态组件：
  - `AudioSource`：clip/external/synth 输入、输出轨道、send、gain/loop/play state、spatial blend、attenuation、cone、doppler、occlusion、convolution send、参数绑定。
  - `AudioListener`：active listener、ear offsets、HRTF profile、doppler tracking、mixer target。
  - `AudioVolume`：区域形状、priority、内外 gain/filter/reverb/convolution/send override、crossfade distance。
  - 其他插件的声音源通过中立 `ExternalAudioSourceHandle`/provider 接入，不直接耦合具体插件。

## Runtime Implementation
- 将当前 flat software mixer 替换为 `SoundEngineState`：clip cache、voice/source registry、mixer graph、DSP node instances、listener/volume state、automation bindings、synth parameter table、IR cache、meters。引擎状态在 `src/engine/state/` 中按 storage、graph mutation、snapshot projection、dynamic event executor、playback record、source voice record 拆分。
- 固定 block 渲染顺序：收集 source -> decode/resample/gain/loop/fade -> 空间化/衰减/doppler/occlusion/volume 影响 -> route 到 track/send/sidechain -> 执行 effect chain -> track gain/pan/L-R trim/delay/phase -> master mix/meter/limit。整体 render root 在 `src/engine/render/` 中保持 root structural：`orchestration.rs` 负责 `render_mix` 的 graph validation、track buffer flow、sidechain tap、DSP/meter 和 master gain 编排，`playback/`、`routing.rs`、`runtime_state.rs`、`sampling/`、`source/` 负责各自行为，其中 `playback/` 按 active-playback routing、clip playback sampling、pan/gain projection、finish reporting 拆分，`sampling/` 按 resample step、cursor/range position、interpolation、frame/channel folding 拆分。显式 source 渲染在 `src/engine/render/source/` 中保持 root structural：`orchestration.rs` 负责 source buffer 编排、环境委托、send 和 finish reporting，`input.rs`、`external.rs`、`parameters.rs`、`range.rs` 分别负责输入混合、外部 block、参数绑定和 clip range。
- Source environment 在 `src/engine/source_environment/` 中保持 root structural：`apply.rs` 负责环境处理编排，`listener.rs`、`spatial/`、`hrtf/`、`volume/`、`convolution.rs`、`constants.rs` 各自拥有独立行为；其中 `spatial/` 按 profile composition、attenuation、cone、doppler、pan 拆分，`volume/` 按 influence selection、shape/crossfade weight、low-pass filter 拆分。
- Loaded HRTF runtime state 在 `src/engine/hrtf/` 中按 `(source, listener, profile)` key、FIR tail history state、loaded-profile convolution apply、stale-state prune 拆分；`src/engine/source_environment/hrtf/` 按 loaded profile dispatch、preview fallback policy、tail query 拆分 source-environment HRTF 接入。
- Occlusion runtime helper 在 `src/engine/occlusion/` 中按 query DTO、deterministic fallback gain、provider-fed gain entry point、ray-traced descriptor matching/specificity 拆分；`src/engine/source_environment/spatial/profile.rs` 继续只负责把 source/listener/volume 查询接入空间化计算。
- `AudioSource` descriptor validation 在 `src/descriptor_validation/source/` 中按 graph-track 引用、clip start/duration 范围、clip/external/synth/silence input、source 参数绑定、spatial settings、source-local scalar/time guard 拆分，`mod.rs` 只保留当前 source validation 入口导出。
- 第一阶段实现确定性 CPU DSP：gain、pan/stereo、delay、filter、sidechain compressor、wave shaper、chorus、flanger、phaser/phase、algorithmic reverb、limiter。运行时代码按 `src/engine/dsp/` 的 effect-chain、track-control、meter 与效果家族模块组织，其中 `effects/` 按 chain orchestration、effect-kind dispatch、sidechain tap lookup 拆分，state 由 `src/engine/dsp_state/` 按 effect key、effect runtime、track runtime、delay line、history buffer 拆分持有；biquad filter 实现由 `src/engine/filter/` 按状态、block apply、normalized coefficients、shelf formulas、clamp constants 拆分。ConvolutionReverb 先支持静态 IR asset，接口与后续 ray-traced IR 共用。
- Production DSP 当前收束点是输入安全和图一致性：所有 effect descriptor 在进入 graph 前验证 finite/range 约束，`src/engine/validation/` 按 graph、track、effect、reference、ordering、value guard 拆分这些规则，`add_or_update_effect` 与完整 graph import 一样重新验证 sidechain track 引用和 post-effect sidechain dependency，避免无效参数或无效 sidechain 延迟到 render 阶段才暴露。
- 输出设备 lifecycle 在 `src/output/lifecycle/` 中按 storage、config、start_stop、callback、status、session 拆分，保留 `software-null` 的确定性 CI 路径，同时把 CPAL backend session、不可用后端诊断、callback 计数和状态投影限制在输出生命周期边界内。
- Sound service surface 在 `src/service_types/` 中保持 root structural：`mod.rs` 只声明/导出服务子模块，`manager_state.rs` 持有 `SoundDriver`、`DefaultSoundManager`、共享 config/state/core 句柄和构造逻辑，其余子模块按 trait dispatch、asset/playback/source/output/graph/preset/event/timeline/acoustics/IR/HRTF/runtime-setting 行为拆分；其中 `mixer_graph/` 继续按 full graph import、snapshot、track CRUD、send CRUD、effect CRUD 拆分服务入口，`output_device/` 按 backend status、device configuration、start/stop lifecycle、status projection、catalog listing 拆分输出设备服务入口，`dynamic_events/` 按 catalog registration/snapshot、handler registration、pending invocation queue、dispatch fan-out 拆分动态事件服务入口，`dynamic_event_executors/` 按 executor registration、unregistration、execution report assembly 拆分执行器服务入口。
- Ray-traced convolution 作为 gated advanced path 纳入设计：可选依赖 scene geometry/ray query，按 listener/source/volume cell 缓存 impulse response，并可发布同一 cell 的 occlusion gain；能力缺失时回退 static IR、deterministic occlusion fallback 或 algorithmic reverb。
- 轨道能力按 FMOD 风格设计：`master` 永远存在，自定义轨道可接收 `AudioSource`、外部组件声音源、synth 输出，可 route/send/sidechain，并暴露可自动化参数。

## Editor, Options, Timeline
- 编辑器替换占位 authoring：Mixer Console、track meter、gain/pan/delay/L-R 控制、mute/solo/bypass、send matrix、effect rack、sidechain source picker、automation 标记；静态 UI template 的 `route = "..."` 必须与 Sound editor 注册的 operation descriptor 同步，当前为 mixer/acoustic/debug/drawer 模板增加 asset id 与 route 合同测试。
- 为 `AudioSource`、`AudioListener`、`AudioVolume` 增加场景 Inspector drawer；增加 Acoustic Debug 视图展示 listener/source cone、volume influence、occlusion rays、convolution IR/probe cache，并保持 drawer template asset id 与注册的 Sound drawer/surface 元数据一致。
- 扩展插件 manifest/registry：声明 backend、sample rate、channel count、global volume、block size、max voices/tracks、HRTF profile、convolution budget、ray tracing quality、default mixer preset、Timeline integration 开关，并通过中立 `SoundPluginOptions` 到具体 `SoundConfig` 的转换保留这些 profile/export/editor 选项。
- 静态 `zircon_plugins/sound/plugin.toml` 必须与 runtime `sound_options()` 贡献保持选项 key、label、类型、默认值和 required capability 同步，当前补齐并锁定 `sound.global_volume_gain`、`sound.default_spatial_scale`、HRTF/convolution/default-preset/Timeline/dynamic-event 等选项元数据；可选功能 bundle manifest 也必须与 runtime package manifest、runtime built-in catalog 和 linked provider manifest 同步，当前锁定 timeline animation track 与 ray-traced convolution reverb 的 owner、capability、dependency、runtime/editor module、editor module capability、default packaging 和 enabled-by-default 字段，且 linked provider runtime crate 的 `feature_manifest()` 必须与 owner/static bundle 的 display name 与元数据一致，防止导出/Hub/Editor host 读取旧选项集或旧 feature bundle。
- 依赖声明：必需 `asset`、`scene`；可选 ray-query/geometry provider；可选 Timeline/animation authoring。
- Timeline 集成使用 `SoundAutomationBinding`：以稳定 sound ID 绑定 track/effect/source/listener/volume 参数，接受现有 `AnimationTrackPath` 风格曲线，也能绑定到 synthesizer parameter ID；当前 runtime 在直接绑定和 mixer graph import 时都会通过共享 `AnimationTrackPath::parse(...)` 校验并归一化 `timeline_track_path`。
- Automation target runtime 在 `src/automation/target/` 中保持 root structural：`apply.rs` 负责 target dispatch、clone-then-validate commit 和 synth 参数写入，track/source/listener/volume 子模块只负责具体参数名映射，`effect/` 继续由 structural `mod.rs`、effect-kind dispatch、common、gain/filter/reverb/dynamics/modulation/delay/stereo/shaper 参数族拆分。
- Dynamic events 已从最初空占位推进到可测的 versioned registry、handler descriptor、pending invocation queue、deterministic fan-out、runtime executor 和 sound-specific ABI callback adapter；后续仍保留 generic native plugin loader 自动发现 `ZrPluginApiV1::invoke_event` 并绑定跨插件 handler descriptor 的深化项。

## Test Plan
- 插件启用/禁用：runtime module、manager handle、动态组件、options、dependencies、空 dynamic event registry 只在 sound 插件启用时贡献。
- 静态 manifest：`plugin.toml` 选项 key、label、类型、默认值、required capability 与 runtime `sound_options()` 贡献完全一致，且中立 `SoundPluginOptions` 能完整转成具体 `SoundConfig`；`plugin.toml` optional feature bundle 与 runtime package manifest optional feature bundle 完全一致，防止静态插件目录、Hub catalog、native/export materialization 读到缺失的 sound option 或旧 feature module/dependency 声明。
- Mixer graph：`master` 存在；自定义轨道/send/effect 可增删；source-to-track 和 track-to-track routing 正常；cycle、缺失目标、非法 DSP 参数返回 typed error。
- DSP：对 gain、pan/L-R trim、delay、filter、sidechain ducking、wave shaper、chorus、flanger、phaser/phase、reverb tail、bypass、wet/dry 做确定性 block 测试；同时覆盖非法 wet/feedback/reverb/shaper/pan 参数和非法 sidechain track/self post-effect tap 在 graph 写入前被 typed error 拒绝。
- 空间音频：`AudioSource` + `AudioListener` 可渲染；listener 选择、attenuation、doppler、stereo/HRTF policy、occlusion flag、`AudioVolume` priority/crossfade 都可预测影响输出。
- 卷积/射线路径：static IR convolution 有输出和 latency；ray capability 缺失时回退明确；缓存失效规则和 provider-fed occlusion gain 可测。
- Timeline/events：automation curve 更新 sound 参数；非法 target path 和裸属性路径在绑定/导入阶段干净失败，合法路径归一化后进入 snapshot；dynamic event registry 初始为空但可发现，注册、队列、dispatch、executor success/failure、缺失 executor、ABI callback success/failure 和 folder-backed 边界合同均可测。
- 回归：现有 sound asset/import/playback、manager handle、plugin package、runtime/editor plugin registration 测试继续通过。

## Assumptions
- 第一实施阶段采用“核心先行”：先完成稳定接口、runtime mixer/DSP、组件、选项和编辑器骨架，再深化完整 ray-traced acoustics。
- 不增加兼容 shim；旧 clip 播放 API 只作为新 graph 默认路由的便利调用保留。
- 重音频行为全部留在 `zircon_plugins/sound`，`zircon_runtime` 只承载中立合同。
- Timeline 和 dynamic events 仍是 gated 集成点；当前 Sound runtime 已实现本地 registry/handler/queue/executor/ABI adapter，跨插件 native loader 发现和通用 host command/event 路由后续再实现。
