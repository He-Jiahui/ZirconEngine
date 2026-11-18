# [Plugin SDK] 测试 SDK 的默认模块栈漏装 Asset 和 Scene 的必需依赖

基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。先单列独立调查；本 PR 未改 SDK、引擎或原测试。

## 1. 差异 / 用户路径

插件作者调用 SDK `TestRuntime::builder().with_config_file_path(...).build()`，不送额外 report，期望默认的真 Foundation/Asset/Scene 能创建默认关卡并 tick。现有 `level_tick_without_animation_plugin_does_not_advance_sequence_players` 在固定生产 + tests-only 同一二进制中失败于 `install world runtime extensions`，缺 `AssetModule → TasksModule`，未到关卡。不是客户端游戏启动故障。

## 2. 影响

零插件负控和依赖默认 Scene/Asset 的插件测试失去真实运行证据。零插件控制证明与 Physics/Animation 的报告排序无关；仅修插件排序不会救它。

## 3. 根因 / UE 对照

SDK 的默认三项是 root，不是依赖闭包；真 Scene 还声明 Time。对照 UE `Project.Maps.AllInPIE`（`Engine/Plugins/Tests/EditorTests/Source/EditorTests/Private/UnrealEd/PlayMapInPIETests.cpp:80–111,152–205`，`370a3f182121c52de38526078a6e560fa8c0c7d2318f93edfbe4d1232bb10836`）先通过真 ModuleManager 取得必要宿主服务；`USelfTest_ConditionalSkip`（`Engine/Plugins/Tests/ModularTestFrameworkTests/Source/ModularTestFrameworkTests/Private/ModularTestSelfTests.cpp:735–750`，`3853b5111429168169c094426d42fec6a98153b3631c0d1bd8ee2b0937e61a2a`）验证实际 AutomationWorker 存在才执行。二者都不构造空 ZE Core 或验证 Tasks/Time，不能照搬缺失即 skip。

[同一实际可执行文件和未经修改的原有对照](https://github.com/He-Jiahui/ZirconEngine/blob/6d81e87ed8047a53b2133dae41f5c0f046890969/docs/plans/unreal-alignment/paused-transition-event-replay.md#L35-L47)，原控制 `level_tick_without_animation_plugin_does_not_advance_sequence_players`，0 过 1 败，实际执行回执 `ddcaa1ca7ce99a2739dc671d75c106b487db914cbc4c2950f44df11abdcadc67`。SDK 的失败出现在执行第一帧之前。

ZE 源码：[SDK 公开的默认根](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/plugin_sdk/src/test.rs#L93-L120)；[真实注册、目录、WorldDriver 安装次序](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/plugin_sdk/src/test.rs#L312-L370)；[Asset → Tasks](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/module.rs#L42-L70)；[Scene → Asset / Time](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/module/mod.rs#L46-L59)；[原无插件夹具](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/runtime_helpers.rs#L35-L45)；[SDK 原 Level 测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/plugin_sdk/src/tests/test.rs#L108-L124)。UE 本机 `6.0.0 / UE5 / CL 0`，上游 revision 未知；没有运行 UE。`Engine/Source/Runtime/Projects/Private/ModuleDescriptor.cpp:779–805`（SHA-256 `649753fd059af601b972042c4474165e60e250874c7eb529eb52aa284a92465f`）是按加载阶段使用真实宿主模块管理器的生产参照，不是这两项测试的通过记录。

## 4. Rust 做法

只在 `zircon_plugin_sdk::test` 的私有 base 注册环把实际选择中的硬 Kernel 前提补全：若有 Asset，登记/按语义激活真正 `zircon_runtime::core::runtime::modules::TasksModule`；若有 Scene，登记真 `TimeModule`；这条缺边已从声明查到，当前先实际报的是 Tasks。保留三公开根及其 identity、Foundation config factory 绑路径、时钟策略和安装/激活错误。不得隐式装可选择的 Asset/Scene/插件；`.without_base_modules()` 仍不装 base；非法缺失（例如人为只选 Scene、没选 Asset）仍由 Core 拒绝。最窄改动不改 Runtime、Physics、Animation、manifest/lock 或任何生产 ABI。这个闭包是固定源码可行边界，不宣称补完即全绿。

## 5. 接口/ECS 非等价

UE 加载进程与目标选中模块；ZE 保留 per-Core engine task graph、真实配置 service、正常 WorldDriver 和 `LevelSystem::tick`。已有 Core 内 scheduler 并不能代替 Tasks descriptor；不要注册同名空假模块或去掉依赖。No-plugin 的 controls 控制的是插件选择，不是刻意不装底座必需依赖。

## 6. 最窄后续门

独立文档 Draft 后在原 SDK 单测宿主加一个无 registration/feature 的默认 build → typed asset manager → default level → 非零真 tick（检查 `world_time()` 推进），检查三 roots 与真实 Kernels 可用；保持既有 `test_runtime_builder_*` 两原例，另做显式缺 Asset 的 fail-closed 负控，不通过关掉计划重播。SDK 单包 `--manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --no-default-features --features runtime --lib --locked` + 精确 `test_runtime_builder_` 过滤/测试 profile，遵守该 fixture 物理 E 根；须先声明独立构建并保留原生回执。已有 scene-only 原回执是启动 RED；直接独立 SDK 原测还未在此 pin 实跑。这里既不是 [#137](https://github.com/He-Jiahui/ZirconEngine/pull/137) 的编译错误，也不是 [#214](https://github.com/He-Jiahui/ZirconEngine/pull/214) 的状态机通知问题。

## 7. 测试 SDK 自有入口：首次行为实证

[实际执行的四个新用例及两个未改原例](https://github.com/He-Jiahui/ZirconEngine/blob/93c5ed9e18af02627638dc9a31e3a679c8dd3d87/zircon_plugins/plugin_sdk/src/tests/test.rs)。四个新用例分别从默认、故意漏选 Asset、完全省略 base、只登记而不激活四种配置建真 Core。夹具继续要求原物理 E 盘，并使用独占配置目录；没有替换为模拟的模块、服务或时钟。

| 入口 | 本次原生结果 |
|---|---|
| 默认三根、无任何插件报告 | 失败：在安装 World 扩展时，正式依赖图报告 `AssetModule → TasksModule` 缺失，尚未到 World 帧。 |
| 显式选择 `[Foundation, Scene]`，故意不选 Asset | 通过：仍准确拒绝 `SceneModule → AssetModule`，不会偷偷补用户省略的根。 |
| 不登记任何 base，不请求 Scene 扩展 | 通过：Core 对五个名字都报告未登记。 |
| 只登记、不安装 Scene 扩展且不显式激活 | 失败：第一次冻结这个未激活的依赖图时亦报缺少 Tasks。 |
| 既有自定义模块/manager 转移、既有默认 Level tick | 两项也在建宿主时失败，未把“文件名/模块名存在”计通过。 |

主过滤 **2 过、4 败**。从同一产物另用 `sdk_base_selected_` 分开运行两个不重叠的选择性控制：**1 过、1 败**。仅选 Foundation 时，原始配置哨兵确实只在激活后出现，两个 kernel 均未被偷偷登记。仅选 Asset 的控制在原版仍缺相应 Tasks，未到后续 manager 与 dormant 断言。 另组回执 `3c59ed28da75db719450d7bf398cb71279e080db28ee6ba50108fd847319805e`。仅提交回归；本身已有的单测一起运行。 Windows x64/MSVC 1.94.1、`test` profile，`zircon_plugin_sdk --no-default-features --features runtime --lib`，唯一过滤 `test_runtime_builder_ --test-threads 1 --show-output`。绑定编译回执 `10c36e4eb621ff0dbdda59419903d56e89376bfc95a1651794b4dd0e78f4abfa`、实际可执行文件 SHA-256 `43de818551831e4df68a3c9641881e0759a35d708e3b3eba26680a52ecc5d3b4`、执行回执 `79c6f34add313cf6fc114b9b9864706becded591c065d9daf6c55a83d0f838a1` 及 stdout `1a65aa4da5820a340d145e431f9dd89e3ee4dbbb0cc3a25604bb721dad62eade`。不代表带宿主依赖的插件排序 [#217](https://github.com/He-Jiahui/ZirconEngine/pull/217) 已修，更不代表 [#214](https://github.com/He-Jiahui/ZirconEngine/pull/214) 已推进动画帧。

## 8. 同一真实 SDK 宿主修后配对

[实际执行的四个新用例及两个未改原例](https://github.com/He-Jiahui/ZirconEngine/blob/d17ac41552d16a68f3c68739e876660cae9bb943/zircon_plugins/plugin_sdk/src/tests/test.rs)。四个新用例分别从默认、故意漏选 Asset、完全省略 base、只登记而不激活四种配置建真 Core。夹具继续要求原物理 E 盘，并使用独占配置目录；没有替换为模拟的模块、服务或时钟。

| 入口 | 本次原生结果 |
|---|---|
| 默认三根、无任何插件报告 | 通过：解析真实 ProjectAssetManager，创建并 tick 原 Level 31.25ms，固定步实际提交 1 次。 |
| 显式选择 `[Foundation, Scene]`，故意不选 Asset | 通过：仍准确拒绝 `SceneModule → AssetModule`，不会偷偷补用户省略的根。 |
| 不登记任何 base，不请求 Scene 扩展 | 通过：Core 对五个名字都报告未登记。 |
| 只登记、不安装 Scene 扩展且不显式激活 | 通过：五项都保持 Registered，不把 providers 隐式启动。 |
| 既有自定义模块/manager 转移、既有默认 Level tick | 两项均通过；保留原声明外的附加模块及服务。 |

主过滤 **6 过、0 败**。从同一产物另用 `sdk_base_selected_` 分开运行两个不重叠的选择性控制：**2 过、0 败**。仅选 Foundation 时，原始配置哨兵确实只在激活后出现，两个 kernel 均未被偷偷登记。仅选 Asset 时，只有 Tasks 随之注册，Time 和 Scene 仍缺席；关闭激活保持 Registered。 另组回执 `cf9d34db6906dfc8ce6ccd9cb16010efb802615fdf28482fc7542fc44f53e64c`。只按实际根选择注册真实 Tasks 或 Time；三项公开根、配置寿命、activation flags、Core 决定的依赖顺序不变。活跃宿主的停止预检分别返回真实 dependent；它是拒绝停止的证据，不声称调用对所有配置均无副作用。 Windows x64/MSVC 1.94.1、`test` profile，`zircon_plugin_sdk --no-default-features --features runtime --lib`，唯一过滤 `test_runtime_builder_ --test-threads 1 --show-output`。绑定编译回执 `2304403315b628cf898244890e7c1e9c53dde4e151441e583166dde96374270b`、实际可执行文件 SHA-256 `4f161c028b0c6cf2dfc68d1891efaf90a0525f1c95263e7875143496f4299446`、执行回执 `009ba3b818ea8815a3a00217aa5706a583abb9574027d2d585000e72bd9765d9` 及 stdout `7586c20a70de3d4e5bc54bd162114015ebb298b63d50d76972abd4c575880c2f`。不代表带宿主依赖的插件排序 [#217](https://github.com/He-Jiahui/ZirconEngine/pull/217) 已修，更不代表 [#214](https://github.com/He-Jiahui/ZirconEngine/pull/214) 已推进动画帧。
