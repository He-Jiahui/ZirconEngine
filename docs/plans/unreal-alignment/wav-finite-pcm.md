# 音频资源：IEEE FLOAT WAV 的 NaN 被当作成功 PCM 发布（独立 Draft 问题草案）

> 取证说明：UE 的本机路径与文件 SHA256 是用户提供参考树的依据；本轮 D: 审计快照/工作记录仅本地保存，不随本 PR 发布，也不冒充 GitHub 可读证据。UE/ZE 测试均未在本轮执行。

- 状态：**源码确定的问题；建议仅据本文开一个问题定义 Draft。** 只调查，不改引擎/测试，不 Cargo，不 Git 写、不远端操作。
- 读取源为共享根 `HEAD=cb495bb1c952a24c86f4513b8099bc4942c3a87d`。父任务确认公开 main=`e89349ca51b4e9d024c194cd3bdf645bd25b6c37`；只读 diff 确认五个引擎 crate/插件范围在两者间无变化，因此 ZE blob 固定 e893，不夹带仅本地的退役流程提交。
- UE 为本地 [Build.version:1–9](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L1-L9>) 的 6.0.0 / BranchName UE5；原文件 SHA-256 列于文末，不把这份镜像冒充某个官方发布 tag。
- 已读规范根/任务树 AGENTS、zircon-engineering / zircon-dev、checkout 与 validation policy。只读共享原根，保留他人脏技能文件；唯一写入为本报告。未恢复协调器/Jenkins，也未把旧流程当阻塞。

## 1. 差异、精确 owner / 符号与生产可达性

**主缺陷在 [SoundAsset::decode_samples:314–319](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/assets/sound.rs#L314-L319>)。** IEEE_FLOAT_FORMAT / 32-bit 分支将四字节解释为 f32 后仅调用 Rust `clamp(-1.0, 1.0)`；NaN 输入仍返回 NaN，却被收进 `Ok(Vec<f32>)`。这不是 Infinity：±Infinity 会被此 clamp 饱和，只有 NaN 不能由该 API 达成有限归一化。

一个头部、block-align、数据长度均合法的单声道 IEEE32 WAV，sample bytes 含 quiet NaN `00 00 c0 7f`（bits `0x7fc00000`），会通过 [from_wav_bytes:134–178](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/assets/sound.rs#L134-L178>) 并返回 samples 中含 NaN 的 SoundAsset。此为源码推导，**没有实际执行 parser/导入器**。

实际 owner 不只在 runtime：

| ZE owner / 符号 | 可达性及与本问题的关系 |
|---|---|
| [AudioImporterRuntimePlugin::register:47–58](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/audio_importer/runtime/src/plugin.rs#L47-L58>)、[WAV descriptor:98–101](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/audio_importer/runtime/src/plugin.rs#L98-L101>) | 真正注册 FunctionAssetImporter，wav priority=120；不是只声明插件名称 |
| [import_wav:109–121](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/audio_importer/runtime/src/lib.rs#L109-L121>) | 调用上述 parser，直接以 ImportedAsset::Sound 返回成功；没有 PCM finite 检查 |
| [ImportedAsset::into_resource_data:95–103](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/assets/imported.rs#L95-L103>)、[artifact Sound 分支:105–127](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/artifact/cache_payload.rs#L105-L127>) | 直接包裹/投影 SoundAsset；纹理有专门校验，Sound 分支没有同类样本检查 |
| [ProjectAssetManager::load_sound_asset:162–164](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/pipeline/manager/assets/loading/load_asset.rs#L162-L164>) | typed Sound 读取入口 |
| [SoundModule:24–70](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/module.rs#L24-L70>) | 注册依赖 ProjectAssetManager 的 lazy DefaultSoundManager 和中立 SoundManager alias |
| [SoundPlaybackManager 实现:8–23](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/service_types/manager_trait/playback.rs#L8-L23>) → [load_clip_impl:40–74](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/service_types/clip_assets.rs#L40-L74>) | 生产 trait 入口，读取 SoundAsset 后构造 LoadedClip |
| [LoadedClip::new:24–30](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/engine/state/playback.rs#L24-L30>) → [cached_static_sound_data / stereo_frames:40–65](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/kira_bridge/playback_data.rs#L40-L65>) | mono 逐样本复制为左右 Frame；stereo 按两项复制；没有 finite guard，NaN 可到后端输入帧 |
| [play_clip_impl:17–73](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/service_types/playback.rs#L17-L73>) → [KiraEngine::play:13–40](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/kira_bridge/manager/playback.rs#L13-L40>) | 检查参数/轨道/后端，最后 main_track/track.play(data)；参数合法不等于样本合法 |

条件边界：需要选择/加载可执行 audio_importer；到真实播放还需选择 sound provider 并启动输出。已查 [first-party catalog feature:10–26](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/first_party_runtime_catalog/Cargo.toml#L10-L26>)：Sound 属 base-runtime-plugins，**不是默认所有 Client 必然装配**。本问题首先在生产导入函数返回错误 PCM 数据就成立，不依赖默认产品有无自动播放。

对后续 guard 的反查：已搜索 sound/runtime 全部 `is_finite / is_nan / samples.iter` 命中。finite 检查存在于 [External block:37–40](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/descriptor_validation/external_source.rs#L37-L40>)、IR/HRTF 等入口，不在 Clip 路径；[playback validation:17–23](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/service_types/playback_validation.rs#L17-L23>) 仅检查 gain/pan/speed。未发现消费前把本条 clip PCM NaN 清洗/拒绝的 ZE 代码。

## 2. 有界影响

影响格式正确但携带 NaN 的 IEEE FLOAT32 WAV；普通整数 PCM WAV、有限浮点样本不受此条件影响。被成功接受的资产不满足“归一化 f32 PCM”的有限数值前提，可让 LoadedClip 的 Kira 输入帧携带 NaN；后端/后续 DSP 不能依赖该资源只含有限样本。

**不推断已经发生全声道污染、必定爆音、声卡崩溃、内存破坏或任意代码执行。** 未运行 Kira/CPAL、声卡、编辑器试听；第三方 Kira 内部是否另行修复最终输出不改变 ZE importer/缓存已接受错误样本的事实，但其具体输出后果仍未知。范围不扩大到所有 Symphonia 格式、所有 public SoundAsset 构造器或整个 DSP finite 政策。

## 3. 根因与 UE 的解决方式（已核 Clamp 实现，不凭名字推断）

已读关联 Build.cs：

| UE模块 | 实际依赖/门控 |
|---|---|
| [Core:8–20,39–49](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Core.Build.cs#L8-L49>) | 数学实现与基础测试所在模块；BuildSettings/基础依赖，不依赖AudioMixer来定义Clamp |
| [AudioEditor:27–53](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/AudioEditor/AudioEditor.Build.cs#L27-L53>) | AudioMixer、Engine、SignalProcessing、UnrealEd；真实 SoundFactory owner |
| [AudioMixer:14–49](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/AudioMixer.Build.cs#L14-L49>) | Core/CoreUObject、Engine、AudioMixerCore、SignalProcessing 和 codec依赖 |
| [AudioMixer:51–92](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/AudioMixer.Build.cs#L51-L92>) | libsndfile I/O只在支持的平台 Editor/Program 定义 WITH_SNDFILE_IO；不能把该导入能力泛化为所有运行平台 |
| [AudioMixerCore:9–18](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixerCore/AudioMixerCore.Build.cs#L9-L18>) | Core/SignalProcessing/TraceLog |
| [SignalProcessing:9–41](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SignalProcessing/SignalProcessing.Build.cs#L9-L41>) | Core/MathCore、codec库；强制纳入 Tests folder |
| [AudioCQTests:13–25](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Tests/AudioCQTests/Source/AudioCQTests/AudioCQTests.Build.cs#L13-L25>) | CQTest、Engine、AudioMixer、SignalProcessing、SoundFieldRendering |

UE 生产导入链与关键短引文：

1. [USoundFactory 的32位源转换:381–440](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/AudioEditor/Private/Factories/SoundFactory.cpp#L381-L440>)：`*WaveInfo.pBitsPerSample != 16` 时走 ConvertAudioToWav；转换失败明确返回 nullptr，不发布原格式源。
2. [FSoundFileConvertFormat::CreateDefault:54–61](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/SoundFileIO/SoundFile.h#L54-L61>) 的格式为 `WAV | PCM_SIGNED_16`，默认不峰值归一化。
3. [ConvertAudioToWav:53–84](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/SoundFileIO/SoundFileIO.cpp#L53-L84>) 初始化解码器/目标writer；[首块:130–148](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/SoundFileIO/SoundFileIO.cpp#L130-L148>)、[后续块:159–176](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/SoundFileIO/SoundFileIO.cpp#L159-L176>) 都执行 `FMath::Clamp(..., -1.0f, 1.0f)`，然后写入整数 PCM。
4. [FMath::Clamp<float>:590–600](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Math/UnrealMathUtility.h#L590-L600>) 实际表达式为 `Max(Min(X, MaxValue), MinValue)`，不是 Rust clamp 的 NaN 语义。[Generic Min/Max:942–971](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Public/GenericPlatform/GenericPlatformMath.h#L942-L971>) 分别以 `(A < B) ? A : B`、`(B < A) ? A : B` 比较：若解码器交来 NaN，则 Min(NaN,1)因比较为false取1，Max(1,-1)取1。**这是当前源码的标量推导，不是声称已跑UE导入后测得NaN映射+1。**
5. 为排除 Windows overload：已读 [Windows typedef:16–19](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Windows/WindowsPlatformMath.h#L16-L19>)、[Microsoft继承:6–18](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Microsoft/MicrosoftPlatformMath.h#L6-L18>)，并搜索 [SSE基类](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Math/UnrealPlatformMathSSE.h>) / [SSE4基类](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Math/UnrealPlatformMathSSE4.h>)，两者无 Min/Max override，沿用 Generic 实现。未据此外推所有平台/编译选项的动态结果。

UE 播放路径也核实了有限 PCM 的消费，不只比较 helper：

```text
UAudioComponent::Play -> PlayInternal -> AudioDevice::AddNewActiveSound
 -> FMixerSource::PrepareForInitialization -> FMixerBuffer::Init + FMixerSourceBuffer::Create
 -> PCM/PCMPreview SetPCMData -> SubmitInitialPCMBuffers -> FRawPCMDataBuffer::GetNextBuffer
 -> int16 / 32768.0f -> source voice buffer
```

证据：[AudioComponent:478–483](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/AudioComponent.cpp#L478-L483>) / [851](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/AudioComponent.cpp#L851>)；[MixerSource:915–988](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSource.cpp#L915-L988>) / [623–654](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSource.cpp#L623-L654>)；[初始PCM提交:381–406](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceBuffer.cpp#L381-L406>)；[有限整型到浮点:39–78](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceBuffer.cpp#L39-L78>)。

借鉴契约是“导入成功的普通源PCM在进入播放前已经规范化，非有限源不能原样成为已接纳帧”。未审 libsndfile 内部NaN政策，故**不承诺 UE 对同一异常文件的最终确切量化样本值**；accepted PCM16 不可能保留 IEEE NaN 位型。也不要求 ZE 仿制 UE 的16位精度损失。

## 4. Rust 如何照 UE 思路实现（尚未实施）

在资源入口显式完成非有限政策，而非让后端自行猜测。最小候选是在 IEEE32解码分支使用明确的 ordered clamp/NaN分支，使这个分支不可能以 Ok 返回 NaN；保留正常有限 f32 精度、±Infinity当前饱和行为和整数PCM分支。

如果选择逐标量对齐本地 UE Clamp，则 NaN分支应显式映射 +1.0，而不是调用 Rust clamp 后假定已对齐；此选择须在 Draft 后用真实 UE 解码/导入控制确认，再定最终回归数值。若产品选择拒绝 NaN，应新增类型化错误并明确这是 ZE 更严格的导入策略，不能假称 UE 已有同样 reject 行为。**本轮不在这两种最终产品政策间实施或验收任何一种。** 两者共同的确定要求是已接受样本必须 finite，不得静默成功携带 NaN。

可在 LoadedClip::new 增加同一契约的防御校验，但该处需先确认反序列化、直接构造等来源的政策；不要为本条 WAV 缺陷夹带整个 codec/asset-validation 架构重写。

## 5. ABI / ECS / 后端差异与可行性边界

- ZE [Sound插件清单:8–12](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/Cargo.toml#L8-L12>) 使用 Kira 0.12.2；[DefaultKiraEngine:13–30](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/kira_bridge/manager.rs#L13-L30>) 后端为 OwnerCpalBackend。UE AudioMixer/USoundWave/解码线程不是此后端的 ABI。
- ZE SoundAsset 是 Vec<f32> + sample rate/channel layout；UE普通导入规范源为PCM16。可对齐“finite且有界”，不复制C++实现，不引入libsndfile，也不强制ZE量化为int16。
- 不改 `zircon_runtime_interface` ABI/DTO布局、插件函数表、SoundClipId，不把 Kira类型泄漏到中立接口。
- ECS并非此资源错误的owner。完成动作、entity销毁、Scene/World音频系统缺口不随本问题扩展。
- 可行性：入口集中，已有 typed SoundAssetError 与 importer Parse边界，可落最小修复；但 NaN canonicalize/reject政策、旧artifact处理、直接构造防御和三方后端输出仍需验证。**本轮只有静态可行性分析，未编译、未跑单测/UE/硬件，没有“可行修复已验证”结论。**

## 6. 具体修改草案与严格后续顺序（不实施）

1. 先单独开问题定义 Draft，只放本问题文档，不能混入以下测试/修复。
2. 在同一 Draft 先加 parser 与真实 plugin importer 的原始源回归，先记录 RED：合法IEEE32 WAV含quiet NaN、负NaN、finite控制、±Infinity控制；所有Ok样本必须finite且范围[-1,1]。不得把原错误fixture改成非法header来“修过”。
3. UE 控制同时记录“标量 Clamp(NaN,-1,1) 的源推导”和“同文件经实际 SoundFactory/ConvertAudioToWav 的结果”；两者不能混成已迁移的同一原测试。确认最终NaN政策后，再定数值oracle。
4. 修复候选仅落 [decode_samples:265–324](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/assets/sound.rs#L265-L324>) 与确有需要的错误变体/注释；使用原测试输入比较原始与候选。若扩展到 LoadedClip 防御，仅补相同单一finite契约，不改资源驻留/卸载/缓存代际。
5. 后续如执行构建，先另行声明精确package/features/filter/profile并获原任务执行阶段授权。本轮不启动任何该命令。

### 测试对照 / 迁移表

| UE原有测试名 / 行号 | 原输入和预期 | ZE现有测试落点 / 本条拟回归 | 结论 |
|---|---|---|---|
| **未找到IEEE32 WAV NaN导入的原UE同例测试** | 由生产链推导：若导入成功，普通PCM源不保留NaN；标量有NaN时当前Clamp源表达式选+1，最终PCM16精确值另核 | [sound_asset_wav_parse_reports_typed_error_variants:179–195](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/tests/assets/sound.rs#L179-L195>) 与 [WAV fixture helper:197–251](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/tests/assets/sound.rs#L197-L251>) 可补格式3/32-bit合法fixture | **源码派生新回归，不称迁移；未写未运行** |
| 同上，无原名可填 | sample sequence `[0, quietNaN, 0.25, +Inf, -Inf]`，sample rate8000/mono/blockAlign4；Ok时全部finite、finite控制不变、Inf饱和；NaN最终数值/拒绝政策须先核UE控制 | [wav_importer_decodes_sound_asset:135–161](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/audio_importer/runtime/src/tests/lib.rs#L135-L161>) 应经 plugin_registration/真实importer，不绕过生产函数 | 同上，不拿当前PCM16正例冒充NaN覆盖 |
| [FNanInfVerificationTest / System::Core::Math::NaNandInfTest:4652–4697](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Tests/Math/UnrealMathTest.cpp#L4652-L4697>) | quietNaN/Inf/0/extremes；检验IsNaN/IsFinite分类 | 可作异常值fixture分类的参考；不是音频导入测试，也没有断言本条NaN PCM行为 | **非等价，不称音频迁移** |
| [FAudioMixerBridgeRegistrationTest:16–29](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/Tests/AudioMixerBridgeRegistrationTest.cpp#L16-L29>) | LoadModuleChecked(AudioMixer)后3个bridge非空 | [playback_lifecycle_round_trips_through_kira:10–24](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/tests/kira_bridge/lifecycle/playback.rs#L10-L24>) 仅MockBackend控制成功 | 已查两侧existing测试；不等价，不覆盖NaN |
| [FDelayIntegerDelay:29–44](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SignalProcessing/Tests/DelayTests.cpp#L29-L44>) | impulse延迟2sample，输出[0,0,1,0] | [loaded_clip_reuses_arc_frames_across_playback_data:11–33](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/tests/kira_bridge/playback_data/cache.rs#L11-L33>)、[non_loop_duration_is_encoded_as_kira_slice:10–28](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/tests/kira_bridge/playback_data/range.rs#L10-L28>) | 只是覆盖清单：分别DSP/共享帧/切片，不是同题迁移 |

原UE同例搜索范围：AudioEditor（含Factories，未发现test-named文件）、AudioMixer/Private/Tests、AudioMixer SoundFileIO、SignalProcessing/Tests、Engine/Private/Tests、Plugins/Tests/AudioCQTests；另查 Core/Tests/Math 的NaN/Clamp。关键词 WAV / SoundFile / NaN / IsFinite / Clamp / automation test。ZE已查 SoundAsset测试、audio_importer测试、sound/runtime现有Kira缓存/范围/MockBackend生命周期与finite命中。没有宣称穷尽UE所有插件测试。

### 模块覆盖、去重与未清事项

- 实际资源解码owner为 runtime SoundAsset + `zircon_plugins/audio_importer/runtime`；已查 [asset_importers/audio runtime:38–45](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/asset_importers/audio/runtime/src/plugin.rs#L38-L45>) 只有metadata投影，非本WAV执行owner；[Opus runtime:20–21](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/opus_importer/runtime/src/lib.rs#L20-L21>) 明确需要NativeDynamic libopus，不误报成已执行解码。
- 播放owner为 sound/runtime；已核服务/LoadedClip/Kira/CPAL类型链。空间声学、timeline、DSP和NativeDynamic能力广度只查owner与既有计划，**不声称已穷尽这些子系统**。
- 已读 [runtime音频架构](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/architecture/runtime-sound-extension.md>)、[SoundAsset契约文档:25–50](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/crates/zircon_runtime/asset/assets/sound.md#L25-L50>)、[Runtime139 canonical音频账:172–249](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/99zn-runtime-audio-sound-clip-streaming-device-mixer-bus-effect-spatial-occlusion-reverb-timeline-event-voice-chat-editor.md#L172-L249>)、[Runtime218当前重判:185–205](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/218-runtime-audio-sound-device-realtime-streaming-mixer-dsp-spatial-acoustics-event-voice-chat-editor-product-current-working-tree-review.md#L185-L205>)。不重复开全PCM驻留、多声道、provider装配、voice预算、非实时render、无World consumer等已有大项。
- 对全部docs的“WAV与NaN/非有限/IEEE_FLOAT_FORMAT/decode_samples”定向搜索未找到本条精确问题；[open135–145快照](<D:/CodexData/worktrees/ze-ue-audit-20261117/reports/ue-module-audit-20261117/github-pr-snapshot.json>) 无音频同题。仅承诺已列范围去重，不承诺所有历史PR均查尽。
- 未清：UE实际异常文件输出、libsndfile内部政策、ZE候选策略与旧artifact兼容、第三方Kira最终输出、声卡/编辑器试听、编译与focused tests、性能。**全部未执行/未验证。** 不写“测试迁移完成”“可行修复通过”或“音频总体已审完”。

### UE原文件 SHA-256

| 来源 | SHA-256 |
|---|---|
| [Build.version](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version>) | 3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6 |
| [Core Build](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Core.Build.cs>) | 2bae34201d978e95d778674f5293b80683c8844ba8963184cf306c20ab06e558 |
| [AudioMixer Build](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/AudioMixer.Build.cs>) | 0b5313e49cd9146a19ea88701191c4e197a27f9599e4681814fe8c46482a0d60 |
| [AudioEditor Build](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/AudioEditor/AudioEditor.Build.cs>) | 707476cc8500ec1124769c6fb4aac5d25a47176066e30206b7e7c4d8a40e716f |
| [AudioMixerCore Build](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixerCore/AudioMixerCore.Build.cs>) | 8802e095ced81cfa33f2308a130ce9cdda3c05292d68e90c5e141b6bd95a9ed5 |
| [SignalProcessing Build](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SignalProcessing/SignalProcessing.Build.cs>) | 49e66ae0115da5a8cc9e59c7937c49eee439c0c0031ad1162fee604e83658e46 |
| [AudioCQTests Build](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Tests/AudioCQTests/Source/AudioCQTests/AudioCQTests.Build.cs>) | 60cbfc211131d0304cf0fa6326f2546dec4e0b3db0ccf563785bc5e3038fdca5 |
| [SoundFactory](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/AudioEditor/Private/Factories/SoundFactory.cpp>) | bb716aadffe1a3bde410a69c2f45fb81b68999b694a0e512b29caa33ae48c549 |
| [SoundFileIO](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/SoundFileIO/SoundFileIO.cpp>) | 72111da2e808aa338f680e828101930ebe5bf4fa2fd43d381bd572abdf29887c |
| [SoundFile](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/SoundFileIO/SoundFile.h>) | 12b8a947c64712a9a33871b056d4da09d484908455e1b93899ecf0434a524de1 |
| [UnrealMathUtility](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Math/UnrealMathUtility.h>) | 666e89fe51afaf0092750e8562bdc1bbbd269fc7b3b5432a42a5cd5a17014f92 |
| [GenericPlatformMath](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Public/GenericPlatform/GenericPlatformMath.h>) | c6e6f9081945fb208c481ed15917f5abcd253c8a72cc82095b3bfeeaa866f669 |
| [WindowsPlatformMath](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Windows/WindowsPlatformMath.h>) | 486ef9e0f282fccfa9da6d723524234b6f434186d1ac12f75a5eb0db0aab69cf |
| [MicrosoftPlatformMath](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Microsoft/MicrosoftPlatformMath.h>) | 39034c7de083a4b1eb93838969e148395dd0457cce469329422c99f237c9b2d8 |
| [UnrealPlatformMathSSE](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Math/UnrealPlatformMathSSE.h>) | 2e5b7b988e5aa7a66a1e6254317d625518a4c9a7a626669894c6ee1aa566103d |
| [UnrealPlatformMathSSE4](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Math/UnrealPlatformMathSSE4.h>) | eb12a6acb3219a4e19f944b4608040e4e53c5483f900ef8993702cc5a2da1640 |
| [AudioMixerSourceBuffer](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceBuffer.cpp>) | 2df6d8c8105c5d1d3488e61132d10093c9460f17df6bf8ad803d90fdff14b562 |
| [AudioMixerSource](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSource.cpp>) | 8c7b1a8a30bf2fcf24eceea649de1f8229964756819efad35c15cc8f6a02fff5 |
| [AudioComponent](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/AudioComponent.cpp>) | 87023fb59fba31754f12519cd9b32801154c9f34388be82ae9fdd442a4735c6f |
| [AudioMixerBridgeRegistrationTest](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/Tests/AudioMixerBridgeRegistrationTest.cpp>) | a2b8c28d6d0edf52b1f3f21785da377ce003c4f7bc4e20ece20b449259458826 |
| [DelayTests](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/SignalProcessing/Tests/DelayTests.cpp>) | dfefbe32f26064b4b5509390b5304942b3beaf0e18cb508b7354e07cf44a9b05 |
| [UnrealMathTest](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Tests/Math/UnrealMathTest.cpp>) | f0276a149caab14a3cdae4f873e7fb151b701696c1e72022407bb8d6d2101cbe |
