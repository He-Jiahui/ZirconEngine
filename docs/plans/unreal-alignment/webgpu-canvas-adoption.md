# WebGPU 设备创建后的 Canvas 表面接管

## 1. 启动表现

Chrome 157.0.8103.52 在 Windows 上运行 ZE 的完整 Web 库。`WebEngine.load` 使用页面传入的 320×240 Canvas，先取得 NVIDIA Ampere 适配器，再成功取得 `GPUDevice`。随后的加载返回 `Rhi(SurfaceUnavailable("native surface sessions are unavailable for this build target"))`。页面记录到 0 次画布配置和 0 帧。该产品包含项目默认字体和画布格式的修复。[字体准入 #241](https://github.com/He-Jiahui/ZirconEngine/pull/241) · [画布格式 #242](https://github.com/He-Jiahui/ZirconEngine/pull/242)

## 2. 复现与对照

| 输入 | 可观察的启动进度 |
|---|---|
| 预烘焙场景、8 个资产及文字 HUD | 产品的 `requestAdapter` 和 `requestDevice` 都成功，接管错误出现在第一次 `configure` 调用前 |
| 同一场景及 8 个资产，HUD 仅预载 | 相同的 GPU 请求顺序和接管错误 |
| 64×48 Canvas 与无资产的空场景 | 相同错误，未取得画布纹理，关闭后的浏览器监听器、引擎和已登记画布计数均为 0 |

这三次运行使用相同的 JavaScript 和 Wasm，绑定后的 Wasm SHA-256 为 `4148cc13d15fd79d82d943d528068b31d1a618fe3a88af96be59519855c95c0d`。独立的 64×64 WebGPU 对照以 `bgra8unorm` 取得纹理并完成清屏提交，浏览器验证错误为空。[空场景回归入口](https://github.com/He-Jiahui/ZirconEngine/blob/1ce9fbedf01451889a4daebbb6ae402466c16c41/tools/validation/browser/web-canvas-format.html)

## 3. 错误来源

`from_canvas` 已为调用者的 DOM Canvas 建立表面，并用它选择适配器。`request_device_inner` 等待设备创建、校验设备信息，然后把**同一个表面**移交给 `adopt_surface_session`。该函数在配置前读取 `supports_surface`。生产设备用 `cfg!(target_os = "windows")` 初始化这一能力，因此 Wasm 设备在这里返回上述错误。

另一个入口 `create_surface_session_impl` 也会生成相同文字。对于本次正尺寸启动，首次接管成功后 RHI 才会完成初次 `configure` 并交出会话回执，随后 viewport 才可能走另开会话的路径。实测的设备成功、首次配置调用数和源码顺序共同指向首次接管。[Canvas 建立][canvas-bootstrap] · [设备移交][device-handoff] · [能力初始化][caps] · [两个会话入口][lifecycle] · [表面配置][configure] · [viewport 接收回执][viewport]

## 4. Unreal 源码与原测

本地 UE 6.0.0（`BranchName=UE5`）的桌面 Slate 路径将宿主窗口和 GPU viewport 分别管理。`FWindowsWindow::Initialize` 建立原生窗口。Slate RHI renderer 取得现有窗口的 HWND，D3D12 RHI 再将设备队列和该 HWND 用于 `CreateSwapChainForHwnd`。调整尺寸作用于已有 viewport。

原测 `System.Windows.Set Resolution` 在真实地图上执行 `setres 640x480`，等待后恢复原尺寸，覆盖命令执行路径。`Rendering.RHI / Test RHI Clear / RHI Clear Render Targets` 对实际纹理执行绘制、清屏与 GPU 提交，并通过 staging 读回比较目标 mip 和 slice。本轮核对了两项原测的注册及断言，UE 执行结果待验证。

| UE 树内源码 | 文件 SHA-256 |
|---|---|
| `Engine/Source/Runtime/ApplicationCore/Private/Windows/WindowsWindow.cpp:194–208` | `a4eb28fd237b822d03fe68021feeddaf0e6ff3e3c127aaf9e7b25af9664adda6` |
| `Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHIRenderer.cpp:309–339` | `ecb1ad7f5d2b4ea7c10b4d0dd376fdff584c992ee2166afa5a717c0d85457a03` |
| `Engine/Source/Runtime/D3D12RHI/Private/Windows/WindowsD3D12Viewport.cpp:100–138` | `bbd0df190f24c327806f08c9361f312ed2d439dccd7d751ba48e59db4e9c34e9` |
| `Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHIRenderer.cpp:539–587` | `ecb1ad7f5d2b4ea7c10b4d0dd376fdff584c992ee2166afa5a717c0d85457a03` |
| `Engine/Source/Runtime/Engine/Private/Tests/EngineAutomationTests.cpp:72–108` | `158c214df2c035c6d3f3075b7613b00bca6f2943486847ab6db4c8a073d19910` |
| `Engine/Plugins/Tests/RHITests/Source/RHITests/Private/RHITests.spec.cpp:18–65` | `062e7dbe9286457baed67522045e41a667bc39679216e926821fb767e88f1bc5` |
| `Engine/Plugins/Tests/RHITests/Source/RHITests/Private/RHIClearTextureTests.cpp:1095–1111` | `d20c8d2cb2e7db556118ad2483314d106564d70027215dc1eb16785a208342ac` |
| `Engine/Plugins/Tests/RHITests/Source/RHITests/Private/RHIClearTextureTests.cpp:963–995` | `d20c8d2cb2e7db556118ad2483314d106564d70027215dc1eb16785a208342ac` |

## 5. 修复位置和相邻工作

建议在设备初始化时区分两个操作。对经 `BrowserWebGpu` 验证且带有效 identity 的 Canvas，允许接管 bootstrap 已创建的表面。接管成功后向 renderer 发布实际会话及该设备代的能力。对于从宿主句柄追加会话的公开入口，按其原有平台条件判断。设备故障检查、实际交换链格式、会话票据和释放顺序继续由原 owner 负责。[设备构造][device-new] · [能力消费][caps-consumer] · [既有设计记录][plan]

截至本次对 242 个全状态 PR 的核查，相关文件的两个开放草稿分别处理 DX12 resize 时的帧引用释放和调用者提供的适配器 profile。前者修改尺寸变化时的资源结算，后者调整请求设备前的校验。此前的 WebGPU 格式草稿已经使同一实例成功取得设备。[#235](https://github.com/He-Jiahui/ZirconEngine/pull/235) · [#236](https://github.com/He-Jiahui/ZirconEngine/pull/236) · [#242](https://github.com/He-Jiahui/ZirconEngine/pull/242)

## 6. ZE 等价回归计划

从相同 64×48 空场景及公开 Web 入口开始，记录设备、Canvas context、首次成功的 `configure` 和由 viewport 接管的会话。用同一实例绘制，再调整为 96×64，检查真实纹理的格式与尺寸、最终呈现 pass 的提交以及资源归零。另用原场景验证 UI。对普通离屏设备和追加原生会话保留各自的许可检查。已有 Windows 能力和生命周期用例作为原生对照，首次配置、真正提交和关闭分别记录。[Graphics 的 Canvas 入口][canvas-backend] · [会话移交][device-handoff] · [viewport 接管][viewport]

[canvas-bootstrap]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface_bootstrap.rs#L21-L43>
[canvas-backend]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/backend/backend/render_backend_new_offscreen.rs#L73-L154>
[device-handoff]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface_bootstrap_adapter.rs#L71-L118>
[device-new]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/device.rs#L87-L149>
[lifecycle]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/device/surface_lifecycle.rs#L37-L68>
[caps]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/device/capabilities.rs#L30-L45>
[configure]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface.rs#L53-L112>
[viewport]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/backend/backend/viewport_surface.rs#L277-L333>
[caps-consumer]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/runtime/capability_summary/capability_summary.rs#L7-L20>
[plan]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md#L159-L173>
[plan-rhi]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/zircon_runtime/render/17/2026-08-24-wgpu-diagnostic-submission-architecture-and-measurement-plan.md#L906-L961>

## 7. 原 Canvas 的首次配置回归

空场景和带 HUD 的原始场景均返回了与适配器配对的设备，加载期间原 Canvas 的 configure 调用计数均为 0。两个用例结束时监听器、实例和登记画布均回到基线。

| 正常 WebEngine.load 输入 | 加载完成 | 初始配置 | 视口接管 | 两尺寸画布提交 | 结束时 owner 回到基线 |
|---|---|---|---|---|---|
| 空场景 | 否 | 否 | 否 | 否 | 是 |
| 原始 HUD 场景 | 否 | 否 | 否 | 否 | 是 |

测试页面与启动素材通过同一 HTTP 源加载。观测对 WebGPU 原方法发起调用，按返回的适配器、设备、Canvas context、目标纹理与提交命令缓冲区建立对应关系。测试包含于 `8bccbca8cb8ec889e38a9ec82d446f78107b8be6`。[浏览器回归](https://github.com/He-Jiahui/ZirconEngine/blob/8bccbca8cb8ec889e38a9ec82d446f78107b8be6/tools/validation/browser/web-canvas-adoption.html)

本次运行的诊断：

- `Rhi(SurfaceUnavailable("native surface sessions are unavailable for this build target"))`

## 8. 接管后在原 Canvas 上的首次配置

空场景和原始 HUD 场景均从首次配置前退出推进到在原 Canvas 上成功配置 bgra8unorm。修复前每例配置 0 次，修复后每例配置 1 次。Windows 设备能力及会话释放对照 7/7 通过。产品随后在准备系统纹理时发生 Wasm 运行错误，首帧仍待验证。

同一个测试页面运行两份 `target-web` 产品。Chrome 157 返回 NVIDIA WebGPU 适配器和对应设备。修复后页面在这台设备和传入的 64×48 Canvas 上观测到 `configure` 成功返回，`getConfiguration()` 读出的活动设备与格式相符。两组场景读取同一批配对输入。

| 可观察结果 | 空场景 | 预烘焙场景与文字 HUD |
|---|---|---|
| 修复前首次配置调用 | 0 | 0 |
| 修复后成功配置 | 1 | 1 |
| 首次配置的活动格式 | `bgra8unorm` | `bgra8unorm` |
| 返回帧 | 0 | 0 |
| 修复后关闭与资源释放 | 未取得引擎句柄 | 未取得引擎句柄 |

首个活动配置出现后，两次运行都在系统纹理准备过程中记录到 `WebAssembly.RuntimeError`。原测试页等待最终 Promise 达到 65 秒上限。补充观测中，38 秒时该 Promise 仍在等待。修订后的页面能报告这一阶段已经完成的配置，并保留未知的关闭状态。视口二次配置、渲染提交和调整尺寸继续等待产品验收。

Windows 主机以 `zr_rhi_wgpu --lib --no-default-features` 运行能力投影与 session teardown 两组回归。与 [#235](https://github.com/He-Jiahui/ZirconEngine/pull/235)、[#236](https://github.com/He-Jiahui/ZirconEngine/pull/236) 的两个相交文件通过三方 Git 文件合并。产品修复提交为 `784d8b6a50e0c134e1831f049debbd8ee1d8fd2d`，修前和修后采用的同一回归页面来自 `0880334302f5245f3bf121076f880b4108d8b0e4`。[回归页](https://github.com/He-Jiahui/ZirconEngine/blob/0880334302f5245f3bf121076f880b4108d8b0e4/tools/validation/browser/web-canvas-adoption.html)

## 9. 双尺寸真实呈现判定与 Chrome 复跑

修订前页面计算了 `followupSubmitted`，但 `adopted` 可在该值为 false 时成立。新 [观察判定](../../../tools/validation/browser/canvas-adoption-checks.mjs#L2-L33) 必须同时满足：首帧和 96×64 resize 帧各有真实产品 `presented` 回执，且产品队列实际返回过引用**同一 Canvas context、相同活动格式、各自正确尺寸**的提交。没有提交、尺寸/上下文错误、未呈现或 owner 未释放都不能通过。保留 UE 的“操作实际目标”的观察量；ZE 不把 Win32 `setres` 或 UE staging 像素验证当作浏览器 Canvas 不同实现的帧证据。

用 Node v24.22.2 实跑 [六个判定用例](../../../tools/validation/browser/canvas-adoption-checks.test.mjs)，**6/6 通过、0 跳过**。包含旧判定会漏掉的「回执已呈现但队列没有实际引用 Canvas」反例、错误尺寸/上下文、提交未返回、异常阶段、owner 未释放，以及完整事实的正对照。它们校验验证器判定，**不是模拟通过的产品帧**。日志 SHA-256 `551b79ea4df2198388950263636933ceddbccdcab39ede8777950a93cf2a5c2a`。

```text
node --test tools/validation/browser/canvas-adoption-checks.test.mjs
```

Windows / Chrome 157、真实 NVIDIA 非 fallback 适配器。同一次配对把[新版观测页](../../../tools/validation/browser/web-canvas-adoption.html)及其同级判定模块从同源 HTTP 提供给两份保留的绑定产品，合法空源目录和场景相同，无用户资产。没有重编 Rust、重新链接或替换产物，不能归为当前分支的独立 Wasm 构建。

| 产品 Wasm SHA-256 | 结果 | 成功 configure | 产品队列提交 / 帧 | 释放 |
|---|---|---:|---:|---|
| `4148cc13d15fd79d82d943d528068b31d1a618fe3a88af96be59519855c95c0d` | 取得设备后在首次配置前拒绝 | 0 | 0 / 0 | owner 计数回基线 |
| `b47a5166ec35c5f5dec9606fa7a4aaeb25933485898ab6752c9acbc049736d68` | `load-interrupted-after-first-configure`；活跃配置确认为原 64×48 Canvas 上的同一设备和 `bgra8unorm` | 1 | 0 / 0 | 引擎未返回，未知 |

第二组的符号栈落在 `SystemTextureResources::prepare → WgpuTextureUploadBatch::push`，未得到视口交接、首帧、resize 或关闭证据。已在独立纯文档 Draft [#268](https://github.com/He-Jiahui/ZirconEngine/pull/268) 记录该故障，不推断底层根因。配对观察摘要 SHA-256 `71e7d76686f1ac18152080964f984713350168493a6119a8a5bc97b70976a039`。本次改动仅限验证与证据，保持真实产品和 `editor-compile-gate` 验收。
