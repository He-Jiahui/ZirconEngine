# 声音时间轴结束时漏写阶梯曲线的最后一个增益值

## 1. 合法的结束键

声音自动化允许在恰好等于序列时长的位置设置关键帧。曲线约定把两端的采样钳制到端点值。非循环序列在最后一次推进中写入绑定目标、返回采样报告，然后移除已完成的序列。[Sound 曲线契约](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/architecture/runtime-sound-extension.md#L736) · [调度准入](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/timeline/validation.rs#L33-L52)

## 2. 复现输入与预期

向默认 `SoundManager` 注册主轨的 `gain` 绑定，在 1 秒长的序列中设置三个阶梯键。时间分别为 `0`、`0.5`、`1` 秒，增益分别为 `0.75`、`0.50`、`0.25`。从 `0.75` 秒推进到 `1` 秒，以及从 `0.75` 秒一步越过结尾，两种情况下报告和混音图都应读到 `0.25`。当前实现从左侧取 `0.50`，随后移除序列。相同的曲线设为循环时，在 `1` 秒回到开头，增益应为 `0.75`。[序列推进和目标写入](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/timeline/advance.rs#L29-L95)

## 3. 末键在取样过程中被跳过

采样器找到第一个满足 `time <= end.time` 的区间。阶梯插值返回该区间的左键值。精确末键因而返回倒数第二个键的值，而略大于末键时才能走到兜底分支并返回末值。序列完成时会把时间限制在末键，从而每次都落入前一个区间。[曲线采样和阶梯插值](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/automation/curve.rs#L15-L79)

## 4. Unreal 浮点通道与原测

Sequencer 为音频段的 `SoundVolume` 注册浮点通道，随后将求值结果应用于音量。浮点通道使用严格上界选择后一键。到最后一键时，它由该键提供数值。对应实现位于 `Engine/Source/Runtime/MovieSceneTracks/Private/Sections/MovieSceneAudioSection.cpp` 第 1053–1067 行、`Engine/Source/Runtime/MovieScene/Private/Channels/MovieSceneChannelData.cpp` 第 22–41 行，以及 `Engine/Source/Runtime/MovieScene/Private/Channels/MovieSceneCurveChannelImpl.cpp` 第 388–405 行。

已注册原测 `System.Engine.Sequencer.Decomposer.AdditiveFromBaseWithEasing` 给浮点通道写入第 0 和第 1000 帧的两个键。在第 1000 帧重组目标 `120.f` 时，它检查得到一个结果且值为 `90.0`。ZE 的回归沿用精确落到末键的取样位置，再检查声音序列的完成状态和主轨增益。原测见 `Engine/Source/Runtime/MovieSceneTracks/Private/Tests/MovieSceneDecomposerTests.cpp` 第 500–504、518–548 行。

## 5. 既有覆盖与归属

ZE 的阶梯测试覆盖首键前、两键之间和末键后。既有循环序列测试覆盖一次跨过时长后继续播放。在所有状态 PR 中，#156 处理 WAV 样本的导入，#165 处理暂停后动画绑定的失效。声音曲线和本回归所在文件均无已有 PR 的改动。[原有阶梯测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/tests/sampling/step_clamping.rs#L5-L39) · [原有循环测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/tests/timeline_loop/wraparound.rs#L5-L39)

## 6. 等价回归与修复范围

三个用例通过正式绑定、调度、推进和快照入口运行。每项都核对报告时间、采样值、完成标志、序列清单和 `master_track().controls.gain`。拟在共享声音采样器内处理结束端点，修复精确完成与越过时长两种输入，并用正好回到序列开头的循环用例作对照。这组验证在音频输出设备停止时进行。[声音管理器入口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/manager_trait_automation_timeline.rs#L18-L66)

## 7. 修复前的原生回归

非循环序列在恰好结束和越过时长两种情况下，采样报告与主轨增益都为 0.50，预期为 0.25。循环回绕用例通过。

每个用例从同一组三个阶梯增益键起步，验证采样时间、绑定、完成标志、序列清单和主轨配置。另有 1 项既有采样测试通过。

在 Windows x64 的 `test` 配置下，`zircon_plugin_sound_runtime` 运行定向用例，通过 1/3。测试提交为 `841b2a843d6611655eccd5f7bff823e86b0ff183`。[本次回归用例](https://github.com/He-Jiahui/ZirconEngine/blob/841b2a843d6611655eccd5f7bff823e86b0ff183/zircon_plugins/sound/runtime/src/tests/sampling/step_clamping.rs)
