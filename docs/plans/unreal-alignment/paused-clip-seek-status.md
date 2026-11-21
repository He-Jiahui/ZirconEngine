# 暂停的音频片段定位成功后，声音服务仍报告旧位置

## 1. 使用场景与合同

宿主用 `SoundManager` 在当前项目加载普通单声道 PCM16 WAV，以每秒 8,000 帧播放 0.5–2.5 秒的非循环片段。播放一小段后暂停，等后端确实进入 `Paused`，再对同一播放句柄定位到 1.5 秒。[声音服务合同](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/architecture/runtime-sound-extension.md#L716-L718)明确写了：成功定位要更新 `playback_status` 返回的游标。即使不恢复播放，后续查询也应返回素材的第 12,000 帧、1.5 秒。

## 2. 受影响的读回

按公开主干源码，定位返回 `Ok(())`，播放仍处于暂停，`playback_status` 却继续返回暂停前的帧。即使后端已消费定位命令、又处理了后续回调，结果仍旧。受影响的是接入 Sound 的 Rust 宿主通过中立服务拿到的位置，不是已观测到的编辑器拖杆故障，也不是声音没有跳到目标。本轮未运行该定位用例。[公开播放接口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/sound/manager/playback.rs#L6-L33) · [实际服务注册](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/module.rs#L24-L70) · [正式播放入口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/service_types/playback.rs#L17-L73)

## 3. 根因：出样时钟覆盖了暂停游标

ZE 将 1.5 秒换成绝对帧 12,000，扣除切片起点 4,000，把相对 1 秒提交 Kira，并将 12,000 保存到活动记录。`playback_status` 却优先采用 Kira 句柄位置，只有读不到句柄位置才使用该记录。[定位](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/service_types/playback_controls/seek.rs#L14-L61) · [状态读取](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/service_types/playback_status.rs#L29-L79) · [Kira 桥接](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/kira_bridge/manager/playback.rs#L82-L127)

[锁定的 Kira 0.12.2](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/Cargo.lock#L2212-L2215) 在 [`src/sound/static_sound/sound.rs:130–157、187–221`](https://docs.rs/crate/kira/0.12.2/source/src/sound/static_sound/sound.rs) 消费 seek 并移动 transport，但已暂停的处理不往重采样器推进样本。`position()` 发布的是[重采样器的出样帧](https://docs.rs/crate/kira/0.12.2/source/src/sound/static_sound/sound/resampler.rs)，所以句柄仍能返回一个成功且过时的位置。不能把命令入队、暂停淡出中或声卡已经出样当作同一件事。

## 4. Unreal 的真实读者与原测试

本机参考树为 `6.0.0 / UE5 / CL 0`。普通本地文件经 `Engine/Source/Runtime/MediaAssets/Private/Assets/MediaPlayer.cpp:565–606` 打开，异步就绪与请求受理分开。`Engine/Plugins/Media/MediaPlayerEditor/Source/MediaPlayerEditor/Private/Widgets/SMediaPlayerSlider.cpp:53–97、129–150` 在可 scrub 时暂停，松开后用 `GetDisplayTime` 读滑杆。`Engine/Source/Runtime/MediaUtils/Private/MediaPlayerFacade.cpp:536–575、1243–1316、2584–2615` 区分已接纳的目标时间和样本时钟，V2 显示读回可先采用目标时间；这不要求原始 `GetTime` 或 DAC 当即出样。`Engine/Plugins/Media/ImgMedia/Source/ImgMedia/Private/Player/ImgMediaPlayer.cpp:646–683` 也会在暂停时更新目标并请求新帧。

`Engine/Plugins/Media/ImgMedia/Source/ImgMedia/Private/Tests/ImgMediaTests.cpp:150–192` 的 `System.Plugins.ImgMedia.LoaderUtils` 断言八种帧率下，帧起始 tick 和下一帧前一 tick 均能换回相同帧，未打开播放器或测暂停定位。未找到同触发的已注册 UE 原测；这是按生产链和 ZE 自身合同设计的回归，并非原测移植。

## 5. 正控和问题归属

[#251 已有原生项目测试](https://github.com/He-Jiahui/ZirconEngine/blob/0fd8dfe4ce448088fa0035ee4aa781873bd26b05/docs/plans/unreal-alignment/sound-clip-project-reopen.md#L43-L55)用真实 WAV 导入、资产管理器和中立声音服务核对磁盘 PCM 及素材身份；它没有开声卡或测定位。本项不改 #251 的项目缓存，也不涉及 [#156 的非有限采样](https://github.com/He-Jiahui/ZirconEngine/pull/156)、[#247 的音量曲线末键](https://github.com/He-Jiahui/ZirconEngine/pull/247)、[#165 的动画绑定](https://github.com/He-Jiahui/ZirconEngine/pull/165) 或 [#178 的视口截图](https://github.com/He-Jiahui/ZirconEngine/pull/178)。[#256](https://github.com/He-Jiahui/ZirconEngine/pull/256) 属于预制体迁移后的场景保存，[#257](https://github.com/He-Jiahui/ZirconEngine/pull/257) 属于工具栏图标资源。本文不将缺少完整视频、XR 或虚拟制作管线列为错误。

## 6. 最小修复判据与回归

让播放状态在本次暂停期间报告该句柄最后一次成功显式定位的逻辑游标；没有定位的普通暂停仍按停住的位置报告。恢复后重新随真实后端推进，不更改公开接口、ABI、时间单位、完成队列或设备所有权。

用实际 WAV parser 生成并装入片段，由生产 `SoundManager` 执行播放、暂停、定位、读回与恢复；仅在测试中用 Kira 的 `MockBackend` 手工处理真实 mixer 的音频帧，不开桌面设备。先确认真实 Kira `Paused` 和原游标稳定，定位后跨两组回调读同一播放句柄，两次均应是 **12,000 帧 / 1.5 秒**；最后不再次定位就恢复，确认从目标附近继续推进。只要求定位期间的逻辑读回一致，不断言已出样；真实 CPAL/声卡是另外的产品验证。既有 [Kira 句柄调用测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/tests/kira_bridge/lifecycle/playback.rs#L9-L25)未驱动回调或断言位置，不能替代它。此处暂无行为 RED/GREEN。

## 修前原生回归：成功定位后仍读到暂停前的位置

测试解码并装入 8 kHz、单声道、非循环 PCM16 WAV，使用生产 `SoundManager` 播放素材 0.5–2.5 秒的切片。先确认 Kira 已进入 `Paused`，且未定位时位置稳定；再经公开接口成功定位到绝对 1.5 秒（第 12,000 帧）。Kira 的 `MockBackend` 手工驱动真实 mixer，不开音频设备。定位后分别在三次和六次回调已处理时读取同一播放句柄，两次仍为 **4,576 帧 / 0.572 秒**；当时保持暂停。未再次定位，恢复播放后读到 **12,445 帧**，证明后端已采纳目标，失败在暂停时的状态读回。

Windows x64/MSVC 下构建 `zircon_plugin_sound_runtime` 的 `--lib` 测试目标，关闭默认特性、无额外特性；最终只执行一个精确用例，**0 通过、1 失败，405 项被过滤，退出码 101**。编译成功，进程在有界 Job 中运行，原位 PE 在运行前后指纹一致；未留存独立 PE。未测试 CPAL 声卡、编辑器或视频播放，未运行修后版本。

[暂停、定位与读回断言](https://github.com/He-Jiahui/ZirconEngine/blob/f36731ab854072da6003b32876bb6c864ecea29e/zircon_plugins/sound/runtime/src/service_types/tests/paused_clip_seek.rs#L52-L100)

## 同一原生回归修后通过（历史版本 63b477f，不代表后续 HEAD）

修前的两次读回都是第 4,576 帧（0.572 秒）；修后同一测试在保持暂停时两次都读到第 **12,000 帧 / 1.5 秒**。恢复后仍能在同一播放句柄上继续推进。测试字节不变，使用真正的 Kira mixer 处理块，不开启声卡，也未观测 DAC 音频。

Windows x64/MSVC 上，同一个 `zircon_plugin_sound_runtime` 库测试从 **0 通过、1 失败，退出 101** 变为 **1 通过、0 失败，退出 0**；两次均精确执行一项，另 405 项被过滤。修前使用 D 盘构建池，修后使用 F 盘构建池；核对了相同的编译器、构建选项和测试选择器。修后试验源为 `bcb18a844b6ab0a45143649843be9b6347320f2c`。这次把已验证的七处文件内容接到公开基线记录之后；候选与试验树只有本文不同。

暂停期间，仅对成功定位过的播放句柄返回已保存的逻辑游标。普通暂停仍读后端位置；恢复或切换到播放后清除此状态，重新随实际后端前进。没有新增 ABI、修改媒体导入或改变设备所有权。

[状态读回](https://github.com/He-Jiahui/ZirconEngine/blob/2de0971fc46bc266def7968fc523378bd863ee7a/zircon_plugins/sound/runtime/src/service_types/playback_status.rs#L40-L74) · [记录定位](https://github.com/He-Jiahui/ZirconEngine/blob/2de0971fc46bc266def7968fc523378bd863ee7a/zircon_plugins/sound/runtime/src/service_types/playback_controls/seek.rs#L40-L53) · [恢复时清除](https://github.com/He-Jiahui/ZirconEngine/blob/2de0971fc46bc266def7968fc523378bd863ee7a/zircon_plugins/sound/runtime/src/engine/state/playback.rs#L51-L60) · [没有改动的精确断言](https://github.com/He-Jiahui/ZirconEngine/blob/2de0971fc46bc266def7968fc523378bd863ee7a/zircon_plugins/sound/runtime/src/service_types/tests/paused_clip_seek.rs#L52-L100)

## 续审：恢复命令已返回，不代表新的混音位置已发布

旧修复在 `resume_playback` / toggle-to-play 成功入队时立即清除暂停游标。原测试在恢复后又处理了八个块才读状态，观察不到入队到新位置发布之间的窗口。Kira 0.12.2 的 `StaticSound::on_start_processing` **先发布上一块的重采样器位置，再消费命令**；即使同一回调已处理恢复，句柄此时也可能仍暴露旧位置。比较位置大小无法判定反馈新旧（seek 可以向后）。这是同一个 clip 游标读回的交接问题，不是 sequence 末键或 WAV 导入问题。

| 本机 UE 生产或测试所有者 | 在 ZE 中采用的合同与区别 |
| --- | --- |
| `Engine/Source/Runtime/Engine/Private/Components/AudioComponent.cpp:1091–1114,1183–1193`；`Engine/Source/Runtime/AudioMixer/Private/AudioMixerSource.cpp:1224–1249,1281–1298` | 组件意图和音频线程上的音源分开；百分比来自 mixer 源帧，并非控制调用的回执。ZE 以 `SoundPlaybackId` 标识 clip，以绝对素材帧/秒回读，不复制 UObject 或声部百分比 API。 |
| `Engine/Source/Runtime/MediaUtils/Private/MediaPlayerFacade.cpp:536–575,1243–1316` | 仅类比 V2 显示时钟：接受的目标可先于样本时间；不把 UE 的媒体播放器当作音频组件实现，也不把图片序列帧率测试当音频原测。 |
| `Engine/Source/Runtime/MovieSceneTracks/Private/Systems/MovieSceneAudioSystem.cpp:785–812,1049–1091` | UE section 的评估时间驱动组件，而非反过来。ZE 的 `advance_timeline_sequences(delta)` 明确接受调用方时间增量；同一个 SoundManager 上，clip 冻结或 seek 不得改写独立自动化序列的时间。未改序列采样。 |
| `Engine/Plugins/Runtime/Harmonix/Source/HarmonixMetasoundTests/Private/MusicClock/MusicClockTests.cpp:233–281`，`Harmonix.Metasound.MusicClock.RuntimeMusicSource.TransportStates` | 对应生产 `RuntimeMusicSource.cpp:267–305`，仅 Running 时从时间源更新位置。移植原测的**暂停冻结、Continue 后从原位置推进**断言；Rust 回归通过真实 Kira mixer 手工驱动，不使用 UE 的手动节拍时钟。UE Stop 后时间归零；ZE Stop 移除该 ID 并保留终态记录。原测没有暂停 seek；seek 与过期反馈断言是 ZE 合同回归。 |

**实现。** 最新成功的暂停 seek 由该 clip 的活动记录持有；负数、非有限值及被拒绝的控制不替换它。有限非负目标仍饱和到切片。恢复时同步改变服务的 paused 标志，游标暂不交还：命令入队后，新建该输出代的请求序号；混音回调在读命令前采集序号，到下一次 Kira 发布上一块位置后才确认。只接纳覆盖本句柄请求的序号，允许同一块覆盖多个请求；不靠时间估计、播放方向或状态轮询次数。普通未 seek 的播放不需要序号。这里确认的是**混音反馈批次**，不是某一源帧已到达 DAC；交接后位置由 Kira 的实际重采样器决定。

仅调整可裁剪 Sound 插件内部。保留 Kira/CPAL 和设备生命周期、中立框架接口、Wasm/宿主与插件边界；不涉及自研 ECS、父子变换矩阵、VM、CI，也不重新启用生产手动拉取音频块的入口。无设备入口仅用于测试，不替代 CPAL 或跨平台实机验收。

**证据与待验。** 已核对原 RED 和 GREEN 的独立测试日志（均非仅 `--no-run` 的回执），且试验提交 `bcb18a844b6ab0a45143649843be9b6347320f2c` 与旧 PR 头 `63b477f82a7b93449ead47d4b5b650974260d01a` 仅本文有差异。上述 1/1 GREEN 不覆盖这次改动。当前过滤器共五项：原精确回归、普通暂停/恢复、先后定位与非法输入及恢复首块（前/后向、resume/toggle）、两句柄错开一块恢复、clip 与 sequence 独立。两次读到目标后，还要求同一 ID 的真实 mixer 位置继续推进，不允许永远钉住目标。

[服务回归](../../../zircon_plugins/sound/runtime/src/service_types/tests/paused_clip_seek.rs)和[反馈发布顺序](../../../zircon_plugins/sound/runtime/src/kira_bridge/cursor_feedback.rs)已做静态检查：14 个所改 Rust 文件通过定向 rustfmt。独立的块事件顺序模型枚举了 136 个控制写入/请求与回调交错，未见先确认、后发布的顺序；这不执行 Rust 或 Kira。

三盘均不足 50 GiB，本轮未编译或运行这些 Rust 用例。下列命令仅以 `-DryRun` 核对过选包、工作树、串行参数和专用缓存路径；盘余量恢复后，由已准备 MSVC 的环境在本任务树执行（不使用旧树的 Jenkins 入口）：

```powershell
& 'E:\Git\ZirconEngine\tools\dev\local-cargo.ps1' `
  -RepoRoot 'D:\CodexData\worktrees\ze-pr259-sound-review-20261121' `
  -TargetDir 'F:\cargo-targets\zircon-local\windows\pr259-review-20261121\target' `
  test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_runtime `
  --no-default-features --lib --locked --jobs 1 paused_clip_seek_tests '--' '--test-threads' 1
```

预期只运行上述五个库单测（test profile、无可选特性）。须见真实测试数和退出码后再做验收决定；未执行 UE 自动化、真实声卡或 Wasm 产品测试。
