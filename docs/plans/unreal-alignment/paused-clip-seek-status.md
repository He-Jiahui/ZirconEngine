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
