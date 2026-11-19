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
