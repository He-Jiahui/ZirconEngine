# WebGPU Canvas 的交换链格式协商

## 1. 产品表现

Windows 上的 Chrome 157 为 `WebEngine.load` 返回了 NVIDIA Ampere 适配器，随后 ZE 报 `Rhi(SurfaceUnavailable("no adapter compatible with the native surface satisfied selection policy: no adapter satisfied the declared selection policy"))`。产品在创建设备前退出，呈现 0 帧。该产品包含 Wasm 库编译修复和浏览器模块启动顺序修复。[Web 入口][app] · [#239](https://github.com/He-Jiahui/ZirconEngine/pull/239) · [#240](https://github.com/He-Jiahui/ZirconEngine/pull/240)

## 2. 复现和对照

给 320×240 Canvas 加载仓内预烘焙验收场景。将 HUD 放入普通预载资源，场景 JSON 和八个资产文件均保持原内容，浏览器成功读取九个文件。实际产品请求适配器一次。浏览器返回的适配器为非 fallback，首选画布格式为 `bgra8unorm`。同一浏览器的独立 WebGPU 对照使用该格式配置 64×64 画布，取得同格式纹理，执行清屏、提交及队列等待，验证错误为空。保留原 HUD 并提供有效的项目默认字体时也得到相同的产品错误。[浏览器启动顺序][open] · [后端创建画布][canvas]

## 3. 筛选失败的位置

锁定的 wgpu 29.0.3 在浏览器上报告 `Rgba8Unorm`、`Bgra8Unorm`、`Rgba16Float`，将浏览器首选项排在前面，并支持 `Fifo`。ZE 的 Wasm 启动代码从这份列表选出 8 位 SDR 格式。接下来的通用兼容性检查只接受 `Bgra8UnormSrgb` 和 `Rgba8UnormSrgb`，格式检查在候选进入适配器列表前失败。后续初次配置、调整尺寸和反向映射交换链回执也会用到这项选择。浏览器的显式 `Fifo` 满足呈现模式要求。锁版后端将 Web 适配器报告为 `Other`，默认策略接受这类设备。[wgpu 能力表][webcaps] · [适配器信息][web-facts] · [ZE 启动选择][bootstrap] · [格式协商][rhi] · [设备策略][policy]

## 4. Unreal 源码及原有测试

本地 UE 6.0.0（`BranchName=UE5`）的 Vulkan RHI 向具体表面查询格式和色空间，用选定值创建交换链，再以相应的像素格式建立 viewport backbuffer 和 view。`PF_B8G8R8A8` 可对应 `VK_FORMAT_B8G8R8A8_UNORM`，表面色空间单独记录。原有 `Rendering.RHI` 测试在实际 RHI 上创建 32×32 纹理并检查创建结果。`RHICopyTexture` 的 BGRA8 线性及 sRGB 组合向 GPU 提交复制，随后通过 staging 读回检查像素位值。本轮检查了注册与断言，UE 执行结果待验证。

| UE 树内源码位置 | 文件 SHA-256 |
|---|---|
| `Engine/Source/Runtime/VulkanRHI/Private/VulkanSwapChain.cpp:227–375` | `dd5bb512689daaeec8667e516043ea64aae3c3564ac89cfd8eca5d9e5f65d396` |
| `Engine/Source/Runtime/VulkanRHI/Private/VulkanSwapChain.cpp:529–595` | `dd5bb512689daaeec8667e516043ea64aae3c3564ac89cfd8eca5d9e5f65d396` |
| `Engine/Source/Runtime/VulkanRHI/Private/VulkanViewport.cpp:425–505` | `13bdcc6fe0de5c07703e4215cbb91b2738560f25410ae6fc6156a4f94381b6a4` |
| `Engine/Plugins/Tests/RHITests/Source/RHITests/Private/RHITests.spec.cpp:18–98` | `062e7dbe9286457baed67522045e41a667bc39679216e926821fb767e88f1bc5` |
| `Engine/Plugins/Tests/RHITests/Source/RHITests/Public/RHITextureTests.h:257–355` | `04d813c69313160e68442ec1ad15cbcf95aab0fc951ed1ab60982e961304e497` |
| `Engine/Plugins/Tests/RHITests/Source/RHITests/Private/RHITextureTests.cpp:18–64` | `52b442d6ae04a256b4a47273a36fc4cbbc99aeae84e69a83eac2b1f7d9aae9e3` |

## 5. 修复范围

在 BrowserWebGpu 的 Canvas 路径按实际能力选择 BGRA8 或 RGBA8 的 UNORM。用同一选择规则完成候选筛选、首次配置、尺寸变化，并把协商格式准确回填 RHI。原生生产窗口继续使用其 SDR sRGB 策略。Graphics 使用回执配置最终呈现管线，已有的 UNORM 输出分支负责 RGB 编码。相邻的 Windows 调整尺寸草稿针对已呈现帧的引用释放，WebGPU 构建草稿针对 Wasm 编译时报出的不可变描述符。[渲染目标][blit] · [呈现 pass][render-pass] · [原生约定][plan] · [#235](https://github.com/He-Jiahui/ZirconEngine/pull/235) · [#239](https://github.com/He-Jiahui/ZirconEngine/pull/239)

## 6. ZE 等价回归计划

对 BGRA8 和 RGBA8 UNORM 逐项验证真实格式到中立交换链回执的映射。用 Win32 描述符验证原生 sRGB 准入、UNORM-only 表面拒绝和显式呈现模式约束。将相同的真实无 UI 场景交给修订后的 Wasm，记录产品是否进入 `requestDevice`、是否调用画布配置及后续诊断。完整呈现的检查包括初始尺寸与非零调整尺寸的真实纹理、最终管线目标、提交、帧回执和关闭。代码对照来自现有 RHI 呈现模式测试和 Graphics 目标格式测试。[RHI 原有用例][rhi-test] · [Graphics 原有用例][gfx-test]

[app]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/web/mod.rs#L12-L49>
[open]: <https://github.com/He-Jiahui/ZirconEngine/blob/e7683efa1adc2d704718f2184ce7008a53779bed/zircon_runtime/src/browser/open.rs#L68-L120>
[canvas]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/backend/backend/render_backend_new_offscreen.rs#L73-L122>
[bootstrap]: <https://github.com/He-Jiahui/ZirconEngine/blob/233f17f0e6a14c05a7a8dd620e99acd03cefb1d6/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface_bootstrap.rs#L79-L145>
[lock]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/Cargo.lock#L8426-L8430>
[webcaps]: <https://github.com/gfx-rs/wgpu/blob/4cbe6232b2d7c289b6e1a38416a6ae1461a22e81/wgpu/src/backend/webgpu.rs#L3845-L3873>
[web-facts]: <https://github.com/gfx-rs/wgpu/blob/4cbe6232b2d7c289b6e1a38416a6ae1461a22e81/wgpu/src/backend/webgpu.rs#L860-L875>
[rhi]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface.rs#L394-L473>
[policy]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi/src/device_profile.rs#L215-L307>
[lease]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface.rs#L134-L210>
[viewport]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/backend/backend/viewport_surface.rs#L277-L329>
[blit]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/backend/backend/viewport_surface.rs#L400-L430>
[render-pass]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/backend/backend/viewport_surface.rs#L447-L497>
[plan]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/zircon_runtime/render/17/2026-08-24-wgpu-diagnostic-submission-architecture-and-measurement-plan.md#L906-L961>
[rhi-test]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/tests/surface.rs#L5-L81>
[gfx-test]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/backend/backend/tests/viewport_surface.rs#L46-L88>

## 7. 空场景与原生格式回归

Chrome 157 通过真实 WebEngine.load 加载无资产的空场景，在设备创建前返回 SurfaceUnavailable。页面记录了零次画布配置和零个呈现帧，监听器、引擎及画布计数均回到零。

仓库中的生成脚本为测试建立目录和内容寻址的空场景。另有 Rust 用例分别检查 BGRA8/RGBA8 的交换链回执、Win32 sRGB 和呈现模式。 在源码 `b02f701abe030f97b676564e638f50ffad1ea214` 上使用同一测试页面与生成的目录。[可复跑页面](https://github.com/He-Jiahui/ZirconEngine/blob/b02f701abe030f97b676564e638f50ffad1ea214/tools/validation/browser/web-canvas-format.html)

## 8. 适配器进入设备创建后的结果

Windows 格式与呈现模式回归从 5/6 到 6/6。Chrome 157 的完整产品选中了适配器，并通过 requestDevice 取得设备。空场景加载结果为 Rhi(SurfaceUnavailable("native surface sessions are unavailable for this build target"))，记录到 0 次画布配置和 0 帧。

原生失败用例原先将 BGRA8 和 RGBA8 UNORM 都映射为空。修订后回执返回对应的 UNORM 格式，Win32 的 sRGB 和 Fifo 控制均通过。用同一个包含八个资源的场景复测时，产品从适配器筛选推进到设备创建。三次 Git 文件合并覆盖了本修复与 [#235](https://github.com/He-Jiahui/ZirconEngine/pull/235) 和 [#239](https://github.com/He-Jiahui/ZirconEngine/pull/239) 的共有源码。 在源码 `523022eadd3fd89f16f93e6826e4ecae4d03b5bd` 上使用同一测试页面与生成的目录。[可复跑页面](https://github.com/He-Jiahui/ZirconEngine/blob/523022eadd3fd89f16f93e6826e4ecae4d03b5bd/tools/validation/browser/web-canvas-format.html)
