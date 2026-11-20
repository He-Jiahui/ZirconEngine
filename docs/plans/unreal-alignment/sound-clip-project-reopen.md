# 项目重开后，按相同地址加载声音可能得到上一项目的片段

## 1. 现象与合同

`SoundPlaybackManager::load_clip` 的公开合同要求通过当前打开的项目解析音效。已加载过 `res://audio/beep.wav` 的宿主如果保留同一声音服务，关项目、替换该 WAV 并重新打开项目，下一次按地址请求仍会命中旧的 `SoundClipId`。通过这个 ID 查询采样率和帧数时，声音管理器读取它先前保存的 `SoundAsset`。这一结论来自加载与关闭路径的源码核对。[声音服务合同](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/sound/manager/playback.rs#L6-L10) · [加载和查询实现](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/service_types/clip_assets.rs#L40-L90)

## 2. 触发范围

Sound 模块依赖资产模块。其延迟创建的经理持有 Core 的弱引用，并通过中立服务接口提供加载操作。资产管理器可以在同一 Core 中关闭当前项目，移除其资源地址，随后重新激活同一根目录或另一项目。每次激活都有独立的 `ProjectActivationIdentity`。错误影响在项目切换前已经按地址加载过、切换后再以同一服务发起的新请求。[Sound 注册](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/module.rs#L24-L70) · [项目关闭](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/pipeline/manager/assets/close_project.rs#L10-L57) · [重开与激活](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/pipeline/manager/assets/open_project.rs#L19-L69) · [激活身份](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/project_activation_identity.rs#L7-L40)

## 3. 根因与已有测试

`load_clip_impl` 有两处按原始 locator 字符串查询缓存。第一处在解析 URI 语法后、取得项目资产服务前。第二处在完成加载、准备登记新片段时。两处都接受上一项目留下的同名条目。[缓存字段](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/engine/state/storage.rs#L34-L44) · [两个命中位置](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/service_types/clip_assets.rs#L40-L74)

ZE 已有用例 `cached_clip_load_returns_before_project_asset_resolution` 验证先返回缓存的行为。另一项 `loaded_clip_reuses_arc_frames_across_playback_data` 验证已解码片段可以复用帧。两项用例在来源已核对的同一原生测试程序中分别通过。新的项目重开回归以真正激活的资产和声音服务检验这两个优化之间的边界。[已有早返用例](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/service_types/tests/clip_assets.rs#L5-L21) · [帧数据消费](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/sound/runtime/src/kira_bridge/playback_data.rs#L40-L65)

## 4. Unreal 对照与已注册原测试

本机 UE 参考树为 `6.0.0 / UE5 / CL 0`。`Engine/Source/Runtime/CoreUObject/Private/UObject/PackageReload.cpp:456–494` 的 `LoadReplacementPackage` 先重命名旧包，再以原名从磁盘加载替换包。`Engine/Source/Runtime/Engine/Private/SoundNodeWavePlayer.cpp:44–69` 的声音消费路径从软对象指针取得 `USoundWave`，需要加载时使用该资产的包路径。

`Engine/Source/Runtime/CoreUObject/Tests/PluginManagerTest.cpp:130–199` 的 `UE::CoreUObject::PluginHandler::LeakDetection` 卸载含旧对象的挂载，随后以原名建立包并断言新旧对象不同。同文件 `:201–243` 的 `UE::CoreUObject::PluginHandler::GarbageCollection` 验证卸载前可解析对象、卸载后以同一路径查询为空。这两项提供地址重用与退休的断言依据。ZE 对应的验证使用真实音频导入器和两个项目激活周期。

## 5. 重现步骤及相反输入

在同一 Core 中解析一次 Sound 服务。打开项目，加载单声道 PCM16 的 `beep.wav` 并保存片段。关闭项目后保留原 `.zmeta` 和磁盘缓存，在同一位置写入替换 WAV。重开时先通过资产状态、typed loader 和磁盘产物确认当前声音已切换，随后通过原 Sound 服务再次按 URI 请求，核对其 `clip_info`。

| 输入 | 首次工程 | 同根重新激活后的期望 |
|---|---|---|
| `beep.wav` | 8 kHz，3 帧 | 16 kHz，4 帧 |
| `finite.wav` 对照 | 8 kHz，3 帧 | 8 kHz，3 帧 |
| 改开一个只含 `finite.wav` 的工程 | 上一工程曾成功加载 `beep.wav` | 当前工程对 `beep.wav` 报 `InvalidLocator` |

测试计划同时检查同一激活内重复加载复用 ID。已有磁盘样本的有限 PCM 帧已逐位核对。新回归将从实际场景入口获取采样率和帧数，验证上述源码推演。已有 #156 处理 WAV 导入时的非有限样本，#247 处理非循环音量曲线的末键，#227 处理主窗关闭重开。本项的独立实现位置是声音服务的 locator 缓存。[WAV 导入器](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/audio_importer/runtime/src/plugin.rs#L47-L59) · [#156](https://github.com/He-Jiahui/ZirconEngine/pull/156) · [#247](https://github.com/He-Jiahui/ZirconEngine/pull/247) · [#227](https://github.com/He-Jiahui/ZirconEngine/pull/227)

## 6. 修复判据

让 `res://` 的缓存命中和并发加载后的登记都使用当前项目激活身份。同一激活继续复用已加载片段。已有片段可以继续服务持有它的播放句柄。关闭、替换项目或加载期间发生新的激活时，新的地址请求应解析此刻的项目。验收使用同根更换素材和另一项目缺少原地址两项回归，并核对旧项目缓存、未改的合法 WAV 及同一激活的命中行为。[项目代数校验入口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/pipeline/manager/assets/runtime.rs#L179-L242)
