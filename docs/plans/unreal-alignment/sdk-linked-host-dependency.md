# [Plugin SDK] 测试 SDK 不应把已经登记的宿主依赖误判为缺失

基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。先单列独立调查；本 PR 未改 SDK、引擎或原测试。

## 1. 差异 / 用户路径

SDK `with_runtime_plugin` 收真报告，已登记真 Foundation，仍把合法 host 边送进只认识插件清单的 `runtime_extensions()`；固定原普通双插件事件用例和 #214 四案同处 `physics.runtime → FoundationModule` exit 101。现有生产 `compiled_project_plan` → `RuntimeModuleCompositionCompiler` 明确不这么做。

## 2. 影响

测试调用者不能凭当前组合开首级 host-dependent 插件，自定义时钟/场景断言均未触达；即使补齐默认 Tasks/Time，插件子图的输入仍不含 Foundation。反证测试应隔离掉 BASE，不把另一个真实初始化错误当过关。

## 3. 根因 / UE 对照

这是 SDK 把 plugin-only 汇总当 host-validation，Core 没做错依赖拒绝。UE `USelfTest_RequiredPluginGate`（`Engine/Plugins/Tests/ModularTestFrameworkTests/Source/ModularTestFrameworkTests/Private/ModularTestSelfTests.cpp:56–68`，`3853b5111429168169c094426d42fec6a98153b3631c0d1bd8ee2b0937e61a2a`）经真 PluginManager 分未知和已配置依赖；`System::Engine::Projects::Loading::PluginDescriptor`（`Engine/Source/Runtime/Projects/Private/Tests/RapidJsonPluginLoadingTests.cpp:141–142,192–219,316–362`，`bab745cc11602b8cf20a02e9d90bc2b575785e5fd94cb42ded2694d3963e8204`）验证依赖字段；`FModuleDescriptor::LoadModulesForPhase` 是源面不是原测。前两者分别是插件准入与元数据，**没有**“宿主 descriptor 已登记仍被 plugin-only 图误拒绝”的等价原测或相同缺边。

[同一实际可执行文件和未经修改的原有对照](https://github.com/He-Jiahui/ZirconEngine/blob/6d81e87ed8047a53b2133dae41f5c0f046890969/docs/plans/unreal-alignment/paused-transition-event-replay.md#L35-L47)，原控制 `state_machine_transition_emits_from_and_to_graph_clip_events`，0 过 1 败，实际执行回执 `019de66ba625c13cc9627cc0c37cb90b4e8cd46d314fbe8997b293a7658dd8f9`。SDK 的失败出现在执行第一帧之前。

ZE 源码：[SDK 公开的默认根](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/plugin_sdk/src/test.rs#L93-L120)；[真实注册、目录、WorldDriver 安装次序](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/plugin_sdk/src/test.rs#L312-L370)；[真实 Physics 依赖](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/physics/runtime/src/module.rs#L30-L82)；[插件子图的严格检查](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/plugin/runtime/catalog/registration/order.rs#L48-L108)；[正式计划的宿主边](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/plugin/runtime/catalog/project/selection.rs#L128-L145)；[真实产品组合原测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/builtin/runtime_modules/tests/registration/composition.rs#L60-L120)；[SDK 旧注册兼容例](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/plugin_sdk/src/tests/test.rs#L74-L106)。UE 本机 `6.0.0 / UE5 / CL 0`，上游 revision 未知；没有运行 UE。`Engine/Source/Runtime/Projects/Private/ModuleDescriptor.cpp:779–805`（SHA-256 `649753fd059af601b972042c4474165e60e250874c7eb529eb52aa284a92465f`）是按加载阶段使用真实宿主模块管理器的生产参照，不是这两项测试的通过记录。

## 4. Rust 做法

可单在 SDK 提供显式 linked fixture 模式，选 target、使用既有报告和明确 manifest，复用同一 compiled plan 的真实模块和 world registry、绑定/激活/时钟/错误照旧；不改生产 catalog 排序/注册依赖，不添 SDK 跨包代码、Cargo 依赖或 ABI。模式若接受现有可选 flags，必须保持其诚实语义或对显式不兼容组合报错，不静默忽略自定义 base 或混合两张图。**不要机械换掉无 target 的旧路径**：[真实 project merge 只留声明的模块名](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/plugin/runtime/catalog/contributions/extension.rs#L25-L66>)；现有 SDK 原例额外登记的 `SdkTestRuntimeModule` 不是其 package runtime row，通改会丢它；产品角色/target/feature 策略也非既有 all-report 合约。仅具备显式 linked 模式不等于旧 API 普遍已修好。

## 5. 接口/ECS 非等价

保留真实 `FoundationModule.Manager.ConfigManager`、原物理查询桥与系统登记；ZE 合成并验证 per-Core `FrozenModuleGraph`，不移植 UE DLL/全局 ModuleManager；不能伪造 Foundation（也不能给清单塞“空宿主 plugin”只过排序）、删依赖、忽略 fatal、直接 tick evaluator。上层 session 的 fallback 也不等于精确无插件负控。

## 6. 分离控制及准入

最小隔离 witness 可在现有 SDK 测试模块内用测试 consumer 的 **真实声明边到 canonical Foundation**、真实 ConfigManager 依赖，而不是新引 Physics dev-dependency：现有旧 builder 配 `[Foundation]`、不装 Scene plan（仅此隔离探针，没有 Scene/Asset，绝不是 #214 配置）会同样在 catalog gate 失败，须实际取得该独立回执；正式 linked 模式再验证实际 manager、World 路径及 production graph 顺序，刻意声明一个缺失的真实所需 provider 时仍拒绝（不期待固定 production 基座能按请求剔除 Foundation）。保留现有 SDK module 原测作为兼容对照，并另由动画既有 owner 选择新模式、保 D11，重跑同二进制的新旧控制。若要求**无选择的旧默认 API**同样无损地支持任意 host 边，尚无可签 SDK-local 通改；先记录独立阻塞，不开跨 Runtime 的泛用排序修复，也不夹入 SDK-BASE 或 #214。这一张的限定修法和测试仅为候选。
