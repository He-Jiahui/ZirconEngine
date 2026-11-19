# [Web 启动] `WebEngine.load` 在登记资产模块前冻结了模块图

## 1. 实测与入口

以 [ZE 基线][base] 为准。完整 `zircon_app` 库（关闭默认特性，仅 `target-web`，`wasm32-unknown-unknown`）在 [#239][pr239] 的编译修正 `7ee0081901559eeb5d3cedfb6a5ce2e00d71ccdf` 上完成链接与 wasm-bindgen。启动源码与基线相同。Chrome 157 在 Windows 上取到 NVIDIA Ampere 非 fallback 适配器。页面导入生成的 JS/Wasm，给真实 `WebEngine.load(canvas, assetRoot)` 传入正常的桌面预烘焙场景：服务端送出了 `scene/catalog.json`，但没有送出任何启动资产或场景 payload。`load` 以 `ModuleGraphFrozen` 拒绝，实际 **0 帧**。同浏览器的独立 WebGPU `bgra8unorm` 清屏提交未报验证错误，该对照的提交来自浏览器 WebGPU API。[完整库入口][app-web] · [产品 feature][app-cargo] · [#239][pr239]。

## 2. 用户影响与范围

通过 Wasm 库嵌入 ZE 的工程，使用合法目录仍拿不到引擎实例，无法进入场景、UI、输入监听或画布格式选择。用同一个完整产物再测两个极小目录：错误目标字段得到预期的资产 schema/target 错误。合法 `wasm32-webgpu` 空资产目录仍报 `ModuleGraphFrozen`，没有到达应有的 `browser bundle has no startup scene`。两次均实际 GET 目录，失败后监听器、引擎、已登记画布计数均为 0。仓内验证页也调用此入口。[目录校验][asset-open] · [无启动场景分支][load-scene] · [验证页][memory-page] · [产品角色][roles] · [独立导出宿主][export-host]。

## 3. 根因及 Unreal 对照

ZE 先登记 Tasks、Time、FrameCount、Foundation、Platform，然后首次 `activate_registered_modules()` 将当前描述符冻结并缓存。目录打开返回后，代码又要登记 Asset、Text、Input、Scene、Animation、Graphics、UI。第一项 Asset 在任何写入前即被拒绝。`register_module` 是生产源码中唯一返回此错误的位置。Core 在首次生命周期操作时锁定声明图。此后预载、场景创建、UI 和异步 Canvas 建立均未执行。[两次登记与激活][browser-open] · [批量激活][batch] · [冻结快照][freeze] · [登记保护][register] · [下游启动][downstream]。

本地 UE 6.0.0（`BranchName=UE5`）是原生对照：C++ 空白工程把主模块声明为 `Runtime/Default`，由 `IMPLEMENT_PRIMARY_GAME_MODULE` 提供入口。`FEngineLoop` 在 PreInit 的 `PreDefault → Default → PostDefault` 逐阶段驱动 Project/Plugin Manager。描述符加载仅选当前阶段及配置允许的模块，跳过已加载，并让 `FModuleManager` 调 `StartupModule`。普通 Game 路径在 `GEngine->Init`（初始化 GameInstance 和视口）之后仍加载 `PostEngineInit` 模块，成功才 `GEngine->Start`，由 `StartGameInstance` 选择/进入启动地图。对应到 ZE，模块描述符应在首次激活前齐备，随后按图启动服务并消费启动地图。[UE 模块文档][ue-doc]。本地源码定位见下。

```text
Templates/TP_Blank/TP_Blank.uproject:6–12。Source/TP_Blank/TP_Blank.cpp:3–6（相对模板目录）
Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp:4021–4030、4802–4827、4959–5013
Engine/Source/Runtime/Projects/Private/ProjectManager.cpp:51–67。PluginManager.cpp:2775–2778、2884–2923。ModuleDescriptor.cpp:779–806
Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp:1074–1112、1220–1261
Engine/Source/Runtime/Engine/Private/GameEngine.cpp:1253–1294、1333–1338。GameInstance.cpp:669–687、722–738
```

## 4. 最小处理

让 Web 宿主从刚创建的 Core 所有的 TaskGraph 打开 cooked 目录。不先激活第一批。`CoreRuntime` 构造时已建好可接收 scope 的 TaskGraph，`BrowserAssetSource::open` 只借它提交 Window 侧本地 I/O future 取 `catalog.json`、校验目录，并建立自己的 `ResourceManager`，不解析已激活的 Tasks、Platform 或 Asset 服务。目录成功后用真实 `source.manager()` 登记 Asset 和其余全部描述符，再做一次统一激活。目录打开使用同一个 Core 和原有容量、取消及错误处理。[TaskGraph 构造][core-create] · [scope 准入][graph] · [目录打开][asset-open] · [实际 Fetch 提交][requests] · [资产描述符][asset-module]。

## 5. 模块、渲染与接口边界

修复位置为 Browser 启动装配。Core 负责冻结已登记图，资产模块声明对 Tasks 的依赖。保留 `GraphicsModule.Manager.RenderFramework` 的 `Lazy` 及 Asset/FontServices 依赖：等真实 Canvas renderer 异步构造后经原工厂槽发布，不能在激活时提前解析。保留真实场景/UI 与关闭路径。与只解除编译阻塞的 [#239][pr239]、输入的 [#151][pr151]/[#238][pr238]、SDK 测试宿主补依赖的 [#216][pr216] 及 SDK 插件目录排序的 [#217][pr217] 分开。本次尚未检验后续 Canvas 格式准入或首帧。[Graphics 声明][graphics] · [异步工厂及发布][renderer] · [Web 生命周期][app-web]。

## 6. UE 原测与 ZE 等价回归

- **UE 原测（查源码，未执行）。** `TEST_CASE_NAMED` 注册 `System::Engine::Projects::Loading::PluginDescriptor`（Automation 面板显示为 `System.Engine.Projects.Loading.PluginDescriptor`）。`REQUIRE` 验证 3 个模块、首模块 `DeveloperTool`、`LoadingPhase::Default` 和依赖/目标字段，不调用实际模块加载。另有 Gauntlet 的 `UE.TargetBootTest`：按全名发现真实 Client 启动节点，父节点缺引擎初始化日志时判 `InitializationFailure`，不是只看进程成功启动。它不检查模块内部顺序或已呈现 GPU 帧。两组原测分别覆盖模块声明和真实 Client 启动。下列本地注册、断言及文件 SHA-256。
- **ZE 保留原断言，再测同一真入口。** `first_lifecycle_operation_freezes_the_declared_module_graph` 先激活，再断言晚注册返回 `Err(CoreError::ModuleGraphFrozen)`，不得改绿标准。新增回归用同一完整 wasm-bindgen 库和同一 HTTP 输入做前后配对：错误目标仍得 schema/target 错误。合法空目录应从现有的 `ModuleGraphFrozen` 推进到缺启动场景诊断（保证不是靠 GPU 才能看见时序恢复）。两条失败路径均不发布实例。另用真实有场景、资产和 UI 的包运行 `load → render_frame → close`，检查实际请求、帧 `presented`、可见实例及 UI 命令、关闭后回到基线。若真实包随后卡在独立图形门，单独记阶段，不能记首帧通过，也不能用监听器样例或裸 WebGPU 代替。[ZE 原断言][core-test] · [缺场景守卫][load-scene] · [真帧回执][frame] · [现有验收页][memory-page]。

```text
UE 源码位置与文件 SHA-256（本地 6.0.0 / UE5）：
Engine/Build/Build.version:2–9  3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6
Engine/Source/Runtime/Projects/Private/Tests/RapidJsonPluginLoadingTests.cpp:140–142、192–219、264–266、316–343  bab745cc11602b8cf20a02e9d90bc2b575785e5fd94cb42ded2694d3963e8204
Engine/Source/Programs/AutomationTool/Gauntlet/Unreal/Automation/UE.BootTest.cs:55–70、95–103、229–245  8552fae6ab131dba61d16b86a95c2844eb0f0ed4c41bf3a6f259be1dcf9440e7
Gauntlet/Framework/Gauntlet.Utils.cs:725–751（按全名发现）。Gauntlet/Unreal/Base/Gauntlet.UnrealTestNode.cs:2112–2128（退出判定）。Gauntlet/Unreal/Utils/Gauntlet.UnrealLogParser.cs:291–294、352–358（初始化标记）
```

[base]: <https://github.com/He-Jiahui/ZirconEngine/tree/e89349ca51b4e9d024c194cd3bdf645bd25b6c37>
[pr239]: <https://github.com/He-Jiahui/ZirconEngine/pull/239>
[pr151]: <https://github.com/He-Jiahui/ZirconEngine/pull/151>
[pr238]: <https://github.com/He-Jiahui/ZirconEngine/pull/238>
[pr216]: <https://github.com/He-Jiahui/ZirconEngine/pull/216>
[pr217]: <https://github.com/He-Jiahui/ZirconEngine/pull/217>
[app-web]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/web/mod.rs#L15-L76>
[app-cargo]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/Cargo.toml#L155-L161>
[roles]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/product_host_config/product_role_catalog.rs#L86-L105>
[memory-page]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/validation/browser/validate-web-memory.mjs#L28-L83>
[export-host]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/plugin/export_build_plan/platform_host_files/browser.rs#L157-L180>
[browser-open]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/browser/open.rs#L13-L67>
[downstream]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/browser/open.rs#L68-L128>
[batch]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/handle/activation/batch.rs#L36-L62>
[freeze]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/handle/core_handle.rs#L75-L87>
[register]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/handle/registration/register_module.rs#L21-L36>
[core-create]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/runtime.rs#L156-L187>
[graph]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/tasks/task_graph/engine_task_graph.rs#L48-L98>
[asset-open]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/runtime/browser/mod.rs#L63-L179>
[load-scene]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/runtime/browser/mod.rs#L207-L238>
[requests]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/runtime/browser/request.rs#L6-L46>
[asset-module]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/runtime/module.rs#L36-L74>
[graphics]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/host/module_registration/module_descriptor.rs#L75-L105>
[renderer]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/browser/open.rs#L87-L128>
[core-test]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/tests/activation/behavior.rs#L53-L69>
[frame]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/browser/frame.rs#L61-L117>
[ue-doc]: <https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-engine-modules>

## 7. 两个可复跑的 Web 产品用例

独立草稿新增两项 HTTP 目录回归。实际 WebEngine.load 对错误平台目录给出预期诊断，对合法空目录返回 ModuleGraphFrozen，未到缺少启动场景的检查。同一完整产品使用 #239 的两行 Wasm 编译修正。测试页面提交为 `415e09c7ec2c915cd6dda7c2d65c92b1eb5e24d1`。[可复跑的浏览器回归](https://github.com/He-Jiahui/ZirconEngine/blob/415e09c7ec2c915cd6dda7c2d65c92b1eb5e24d1/tools/validation/browser/web-startup-order.html)

| HTTP 资源目录 | WebEngine.load 实际结果 | 预期 |
|---|---|---|
| `wrong` | `Initialization("AssetModule.Manager.ProjectAssetManager", "unsupported browser asset catalog schema, target, or entry count")` | `Initialization("AssetModule.Manager.ProjectAssetManager", "unsupported browser asset catalog schema, target, or entry count")` |
| `correct` | `ModuleGraphFrozen` | `Initialization("AssetModule.Manager.ProjectAssetManager", "browser bundle has no startup scene")` |

## 8. 统一登记后的产品结果

错误平台目录收到预期的资产校验错误，合法空目录由 ModuleGraphFrozen 推进到预期的缺少启动场景诊断。两条真实 Web 产品回归均通过，三个浏览器 owner 计数均回到零。同一完整产品使用 #239 的两行 Wasm 编译修正。测试页面提交为 `50ce6ca18adaf778c48e1c803e79ed18b7d9ee42`。[可复跑的浏览器回归](https://github.com/He-Jiahui/ZirconEngine/blob/50ce6ca18adaf778c48e1c803e79ed18b7d9ee42/tools/validation/browser/web-startup-order.html)

| HTTP 资源目录 | WebEngine.load 实际结果 | 预期 |
|---|---|---|
| `wrong` | `Initialization("AssetModule.Manager.ProjectAssetManager", "unsupported browser asset catalog schema, target, or entry count")` | `Initialization("AssetModule.Manager.ProjectAssetManager", "unsupported browser asset catalog schema, target, or entry count")` |
| `correct` | `Initialization("AssetModule.Manager.ProjectAssetManager", "browser bundle has no startup scene")` | `Initialization("AssetModule.Manager.ProjectAssetManager", "browser bundle has no startup scene")` |

Web 宿主使用同一 Core TaskGraph 取得并验证目录，登记全部模块后执行一次统一激活。Core 的晚注册保护继续由既有原测覆盖。[启动顺序](https://github.com/He-Jiahui/ZirconEngine/blob/50ce6ca18adaf778c48e1c803e79ed18b7d9ee42/zircon_runtime/src/browser/open.rs#L13)

## 9. 同一预烘焙场景的加载位置

在公开提交 `ca47f2c35066492f22a4445308c35b2a02e13395` 对应的启动修复与 [WebGPU 编译修复 #239](https://github.com/He-Jiahui/ZirconEngine/pull/239) 上，Chrome 157 读取同一份预烘焙目录。修复前仅请求目录，`WebEngine.load` 返回 `ModuleGraphFrozen`。修复后请求了目录、8 个内容寻址的资产文件和 1 个场景文件，随后返回 `Source(ProjectAssetUnavailable)`，截至该次加载结束呈现 0 帧。两次目录 SHA-256 均为 `6a43dd44f07e58dd360dfee4608cedf7a9c03aa3a85e695692d1215012bb609b`。

目录包含 1 个 UI 布局根。该次输入中记录的资产种类为 Mesh、AnimationSkeleton、Model、Shader、Material、AnimationClip、UiLayout 和 Texture。检查项目字体需求、目录登记和已驻留资产的对应关系是下一项运行时调查。
