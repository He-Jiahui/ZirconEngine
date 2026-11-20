# 右侧分屏相机重建内部场景附件时会丢失左侧画面

## 1. 用户可见的结果

在一个 160×120 的视口中依次提交两台 `PrimarySurface` 相机。第一台把左半区清红，第二台把右半区清绿。提交成功，截图尺寸和来源均符合接口约定。左侧内区的 6,656 个像素中，红色像素为 0；右侧内区的 6,656 个像素全部为绿色。[现有双相机回归](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/tests/primary_surface.rs#L167-L247)

## 2. 触发条件与影响

两台相机共用同一个 `RenderViewportHandle` 和显示目标，渲染顺序为左、右。它们的矩形宽度相同，水平起点分别是 0 和 80。框架逐相机解析矩形，内部场景附件按矩形右下角坐标向上对齐到分配粒度。在此配置下，显示输出始终为 160×120，内部尺寸却依次为 80×120 和 160×120。使用局部相机合成的调用者因此可能丢失先前相机已经写好的区域。[相机循环](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/runtime/framework/extract/submit/camera_loop.rs#L237-L260) · [当前相机矩形](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/runtime/extract/build/effective_view_state.rs#L17-L35) · [内部尺寸计算](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/render/view_family/pipeline.rs#L412-L430)

## 3. 纹理为何丢失

渲染器把 `SceneLinear` 的分配尺寸传给 `ensure_offscreen_target`。显示尺寸和内部尺寸任一变化，都会重建整个 `OffscreenTarget`。其中 `final_color` 本来按显示尺寸分配，也跟随内部颜色、深度和 GBuffer 一起更换。局部目标沿用 `Load`，右相机写入右半区时，新纹理里已经没有左相机的像素。[帧提交](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/core/pipeline/frame_submission_owner.rs#L93-L125) · [重建条件](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene_renderer_target/ensure_offscreen_target.rs#L3-L22) · [最终颜色与场景颜色的尺寸](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/backend/offscreen_target_construct/construct.rs#L18-L40) · [局部写入的 Load 行为](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/execution/context.rs#L470-L490)

## 4. Unreal 的实现和功能测试

本地 Unreal 参考树的版本信息为 `6.0.0 / UE5 / CL 0`。`Engine/Source/Runtime/Engine/Private/GameViewportClient.cpp:1568–1573、1788–1803` 的 `UGameViewportClient::Draw` 为传入的 viewport 建立 view family，并用同一个 family 构造每个玩家的 view。`Engine/Source/Runtime/Renderer/Private/ScreenPass.cpp:33–42、173–198` 从 family 的渲染目标注册同一张外部纹理，再为每个 view 选择其矩形。首个局部 view 可清屏，随后同一 family 的 view 使用 `ELoad`。

UE 原有 `AScreenshotFunctionalTestBase` 在允许模拟分屏时依据 `r.ScreenshotTest.EmulateSplitscreenMode` 调整输出尺寸。`AScreenshotFunctionalTest` 按模式裁取第二个 view 的矩形，把获得的像素交给截图比较框架。ZE 原有等价测试使用两种清屏色，额外校验先写入的区域。此轮实际执行的是 ZE 的测试。UE 对应位置为 `Engine/Source/Developer/FunctionalTesting/Private/ScreenshotFunctionalTestBase.cpp:200–243` 及 `Engine/Source/Developer/FunctionalTesting/Private/ScreenshotFunctionalTest.cpp:121–150、198–203`。

## 5. 实测和问题归属

Windows x64 上使用 NVIDIA GeForce RTX 3060 Laptop GPU 和 DX12 运行 `render_product_split_screen_viewports`。该测试只提交相机清屏，在比较像素前已检查截图存在、宽高、`PrimarySurface` 和 `FrameworkOffscreen` 来源。两份原生程序使用同一份分屏测试源码和相同的测试配置。原渲染实现的运行树只额外包含测试。第二份程序使用 #178 的请求归属修复。两次读数如下。[原始像素断言](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/tests/primary_surface.rs#L203-L246) · [已发布的 #178 验证](https://github.com/He-Jiahui/ZirconEngine/blob/6b814bb5d5dc31e948049a25f23fa6556e8a7096/docs/plans/unreal-alignment/viewport-capture-owner.md)

| 运行配置 | 左红 | 左绿 | 右红 | 右绿 | 每侧检查像素数 |
|---|---:|---:|---:|---:|---:|
| 原渲染实现 | 0 | 0 | 0 | 6,656 | 6,656 |
| 加入 #178 的按视口请求修复 | 0 | 0 | 0 | 6,656 | 6,656 |

已核对 251 条开放、合并和关闭的 PR。#178 处理多个视口之间的同步截图选择，#235 处理 Windows 交换链重配，#248 处理 LUT 所用的后处理图。本项对应同一逻辑输出在两次相机提交间的物理纹理生命周期。[#178](https://github.com/He-Jiahui/ZirconEngine/pull/178) · [#235](https://github.com/He-Jiahui/ZirconEngine/pull/235) · [#248](https://github.com/He-Jiahui/ZirconEngine/pull/248)

## 6. Rust 修复与验收

用当前视口、逻辑输出和显示尺寸约束最终纹理与其 view 的复用。只有相机内部尺寸发生变化时，保留这张已写入的最终纹理，并按新内部尺寸分配场景颜色、深度和 GBuffer。显示输出实际调整大小或切换到不同的输出所有者时，为新输出提供相应的纹理。显式相机纹理仍按资产的直接导入或转换写回路径工作。可在现有 WGPU 后端与提交路径内实现。

沿用现有 160×120 双区回归，两侧各自应有超过 90% 的预期颜色，错误颜色均低于 5%。增加一项真实 WGPU 纹理与 view 身份对照，覆盖同视口内部尺寸变化、等尺寸切换视口和真实输出 resize。通过实际分屏像素和资源身份共同验证修订。
