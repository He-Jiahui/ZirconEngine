# Web 启动时的默认字体与项目字体覆盖

## 1. 用户可见结果

在 Chrome 157 中，普通文字 HUD 会令 `WebEngine.load` 返回 `Source(ProjectAssetUnavailable)`。仓内验收夹具的 `res://browser/hud.view` 只有文字 `Zircon WebGPU 64 bones`，使用默认样式。浏览器成功读取目录、8 个资源和场景文件，随后在向 WebGPU 请求适配器前退出，呈现 0 帧。测试使用 [#239][pr239] 的可编译 Wasm 库和 [#240][pr240] 修订后的启动顺序。[HUD 夹具][fixture]

## 2. 相同产品的输入对照

| HUD 或资源变化 | `WebEngine.load` 实际进度 |
|---|---|
| 原 HUD，目录没有项目默认 Font | `Source(ProjectAssetUnavailable)`，产品请求适配器 0 次 |
| 保留全部 8 个资源和场景，把 HUD 从 UI 根移至普通预载资源 | 产品收到浏览器返回的 NVIDIA 适配器 |
| 原 HUD，在启动资源中加入有效的项目默认 Font | 产品收到相同适配器 |
| 显式引用不存在的默认或自定义字体 | `Source(ProjectAssetUnavailable)` |
| 自定义字体和默认覆盖均有效且已预载 | 产品收到相同适配器 |
| 项目默认 Font 已登记但未预载，或 URI 指向其他类型 | `Source(ProjectAssetUnavailable)` |
| 已预载的 Font 缺少 blob，或 blob 内容损坏 | 分别返回 `Source(ProjectCookedPayloadUnavailable)` 和 `Database(InvalidCookedArtifact)` |

收到适配器的用例随后止于 Canvas 表面选择，呈现 0 帧。所有用例通过同一 Wasm 的 `WebEngine.load`，场景文件和原有资产均用内容哈希寻址。

## 3. 源码定位

每个保留树的字体集合都先加入 `res://fonts/default.font.toml`，再加入解析样式时得到的显式字体。Web 启动把集合中每个 URI 交给项目资产加载器，并传播准入失败。本例的默认 URI 没有对应目录记录，`resolve_asset_id` 返回空值。此时 `FontCollection` 里已经有编入运行时的默认清单与 TTC，使用独立 owner，提供 Fira Mono 主字体及简中回退子集。普通文字使用有效默认字体的顺序是项目覆盖、内置默认。显式字体引用优先于这两层。[保留树收集][deps] · [Web 启动][ui] · [项目读取][read-font] · [内置字体注册][bootstrap] · [字体清单][manifest] · [优先级约定][policy]

## 4. 修复位置与影响范围

Web 建立 UI 时收集作者显式写下的字体 URI。项目目录登记了默认 URI 时，把该覆盖也加入准入集合。这样普通文字可以使用已注册的内置字体，显式写下同名 URI 的文档仍按项目资源读取。对登记的资源继续执行驻留、类型、blob 及字体注册检查，使用原有的作用域退库与样式解析。原生运行时已有独立的准入计数，编辑器使用自己的字体 URI。既有构建和模块注册草稿分别负责此前两个启动阶段。[样式解析][style] · [项目覆盖与回收][claim] · [原生运行][native] · [编辑器选择][editor] · [#239][pr239] · [#240][pr240]

## 5. Unreal 源码与既有测试

本地 UE 6.0.0（`BranchName=UE5`）中，`FCoreStyle::GetDefaultFontStyle` 使用 Slate 提供的合成字体。引擎内容目录为其提供 Roboto 与回退字体。显式 `FontObject` 有单独的解析及字体 face 加载路径。原测 `Slate.Core.Text.TextLayout.LazyGeneration` 对默认构造的文本样式检查普通文本生成的行模型和视图。`UE.Engine.Canvas.BasicMultilineWidth` 先检查 Tiny、Small、Medium 引擎字体中的可用 `UFont`，再检查多行文本的最大宽度。本轮核对了这两项原测的注册及断言，UE 执行结果待验证。

| UE 树内源码位置 | 文件 SHA-256 |
|---|---|
| `Engine/Source/Runtime/SlateCore/Private/Styling/CoreStyle.cpp:108–126` | `403c8e8d08e015b2b02259b97a0873b04bc06ce46f288922b4b211b4ceb98829` |
| `Engine/Source/Runtime/SlateCore/Private/Fonts/LegacySlateFontInfoCache.cpp:94–137` | `8007e064609f5a67444f5d07ee3f4ef00b5ca01c7e54f6f5461b4cfc0d46d8d0` |
| `Engine/Source/Runtime/SlateCore/Private/Fonts/SlateFontInfo.cpp:186–207` | `1b9ed9b4e8782852ea5761444e3dc46edcdd153b65871b4c3eba9be91bf0ecc1` |
| `Engine/Source/Runtime/SlateCore/Private/Fonts/FontCacheCompositeFont.cpp:694–794` | `4a45fff7761c391053830b51dd2a890d526e3e529b727b81276a542929f82582` |
| `Engine/Source/Runtime/SlateCore/Private/Fonts/SlateFontRenderer.cpp:354–417` | `90597f59e68705fce20296f300845ae220818876f7ca759e8ebd11bacfa2857f` |
| `Engine/Source/Runtime/Slate/Private/Test/TextLayoutTest_LazyGeneration.cpp:6–103` | `c379beb1d405ff6ce162de30817ed5befe321edbf2dc604994a4f839ae10b542` |
| `Engine/Source/Runtime/Engine/Private/Tests/UserInterface/CanvasTests.cpp:13–66` | `344cbf74c0bbb0552700ec271e8a3a974dfc70886fd1055e2fc0230bdb6af36a` |

## 6. ZE 等价回归计划

用同一个 cooked 场景验证两项正向变化。默认样式且没有项目覆盖的 HUD 应进入渲染器。只登记所需自定义字体的显式 HUD 也应进入渲染器。对照用例固定有效项目覆盖、显式同名引用缺失、错误类型、未预载和坏 blob 的实际行为。观察点依次是字体准入、产品发出的 WebGPU 请求、首帧文字提交及关闭后的资源计数。已有测试分别覆盖保留树去重与默认选择。[既有 UI 去重][ui-test] · [既有字体选择][text-tests]

[pr239]: <https://github.com/He-Jiahui/ZirconEngine/pull/239>
[pr240]: <https://github.com/He-Jiahui/ZirconEngine/pull/240>
[pr174]: <https://github.com/He-Jiahui/ZirconEngine/pull/174>
[deps]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/surface/render/font_dependencies.rs#L8-L19>
[ui]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/browser/ui.rs#L117-L138>
[read-font]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/text/font/source_manifest.rs#L248-L273>
[bootstrap]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/text/font/shared.rs#L348-L387>
[manifest]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/assets/fonts/default.font.toml#L21-L48>
[fixture]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/artifact/web_bundle/acceptance_fixture.rs#L233-L248>
[style]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/surface/render/resolve.rs#L78-L89>
[policy]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/engine-code-structure-convention.md#L1406-L1429>
[claim]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/text/font/runtime_asset.rs#L334-L400>
[native]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/runtime_ui/font_admission.rs#L22-L47>
[editor]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/window/constants.rs#L5-L7>
[last]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/text/font/fallback.rs#L475-L531>
[text-tests]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/text/tests/service.rs#L52-L116>
[ui-test]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/surface/render/tests/font_dependencies.rs#L8-L25>

## 7. 正常 WebEngine 入口的九组回归

Chrome 157 的九组字体准入回归通过 7/9。未写字体的 HUD 和已预载自定义字体的 HUD 均因默认项目字体缺失而返回 ProjectAssetUnavailable。显式资源缺失、已登记但未预载、类型错误及坏 blob 均返回预期诊断。 本组使用固定输入差分目录。下表的渲染器请求次数来自产品对浏览器 WebGPU API 的实际调用。

| 场景 | 渲染器请求适配器次数 | 字体准入断言 |
|---|---:|---|
| 只预载 HUD | 1 | 通过 |
| HUD 使用默认样式 | 0 | 失败 |
| 有效的项目默认覆盖 | 1 | 通过 |
| 显式同名默认字体缺失 | 0 | 通过 |
| 显式自定义字体缺失 | 0 | 通过 |
| 自定义字体就绪，目录无默认覆盖 | 0 | 失败 |
| 覆盖已登记、未预载 | 0 | 通过 |
| URI 对应资源类型错误 | 0 | 通过 |
| 已预载字体的 blob 哈希不符 | 0 | 通过 |

每个用例结束时，监听器、引擎实例和已登记画布的计数均为零。页面提交为 `ec39dfc609c9fdac1487a91051d2843c104a8349`。[可复跑页面](https://github.com/He-Jiahui/ZirconEngine/blob/ec39dfc609c9fdac1487a91051d2843c104a8349/tools/validation/browser/web-font-policy.html)

产品进入渲染器后返回 `Rhi(SurfaceUnavailable("no adapter compatible with the native surface satisfied selection policy: no adapter satisfied the declared selection policy"))`。本次呈现 0 帧。

## 8. 使用内置默认字体后的实测

Chrome 157 的九组字体准入回归通过 9/9。未写字体的 HUD 和已预载自定义字体的 HUD 均进入了渲染器。显式资源缺失、已登记但未预载、类型错误及坏 blob 均返回预期诊断。 本组使用固定输入差分目录。下表的渲染器请求次数来自产品对浏览器 WebGPU API 的实际调用。

| 场景 | 渲染器请求适配器次数 | 字体准入断言 |
|---|---:|---|
| 只预载 HUD | 1 | 通过 |
| HUD 使用默认样式 | 1 | 通过 |
| 有效的项目默认覆盖 | 1 | 通过 |
| 显式同名默认字体缺失 | 0 | 通过 |
| 显式自定义字体缺失 | 0 | 通过 |
| 自定义字体就绪，目录无默认覆盖 | 1 | 通过 |
| 覆盖已登记、未预载 | 0 | 通过 |
| URI 对应资源类型错误 | 0 | 通过 |
| 已预载字体的 blob 哈希不符 | 0 | 通过 |

每个用例结束时，监听器、引擎实例和已登记画布的计数均为零。页面提交为 `3dc6edcd5abd8842563c3867052f56913487195e`。[可复跑页面](https://github.com/He-Jiahui/ZirconEngine/blob/3dc6edcd5abd8842563c3867052f56913487195e/tools/validation/browser/web-font-policy.html)

产品进入渲染器后返回 `Rhi(SurfaceUnavailable("no adapter compatible with the native surface satisfied selection policy: no adapter satisfied the declared selection policy"))`。本次呈现 0 帧。
