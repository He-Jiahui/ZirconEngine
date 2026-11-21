# WebGPU 接管 Canvas 后，系统纹理准备中断启动

## 已复现的阶段与范围

Windows / Chrome 157（UA `Chrome/157.0.0.0`）在同一安全回环 HTTP 源，使用公开 `WebEngine.load`、同尺寸 64×48 的调用方 Canvas 及相同合法空场景。目录不含任何用户资产，场景 payload 为 31 字节。两份已存的 `zircon_app` 产品都是 `wasm32-unknown-unknown`、dev、关闭默认特性、仅 `target-web`。它们是历史组合夹具，不是各独立 PR 的交付构建；本次重新运行浏览器，未重建 Wasm。

| 配对输入 | 实际阶段 |
|---|---|
| 接管前产品 | 回来 NVIDIA 非 fallback 适配器和相配 `GPUDevice`；`WebEngine.load` 在首次 configure 前以 `SurfaceUnavailable` 拒绝。没有帧，三个 owner 计数回到基线。 |
| 已含 [#243](https://github.com/He-Jiahui/ZirconEngine/pull/243) 接管修复的产品 | 同样返回 NVIDIA 非 fallback 设备。对原 Canvas 执行了恰好一次 `configure`，读取到活动设备与 `bgra8unorm` 格式；随后收到 `RuntimeError: memory access out of bounds`。取得的观察是 `load` 仍 pending，0 次产品队列提交、0 帧；未取得引擎句柄或关闭证据。 |

首次新的可证失败阶段是**构造系统纹理上传批次**，不是选择适配器、设备创建、初次配置、视口交接或提交完成。符号化浏览器栈与此前空场景和含 HUD 的两个观察一致：`SystemTextureResources::prepare → push_upload → WgpuTextureUploadBatch::push → Vec::push → RawVecInner::finish_grow → Layout::repeat → Layout::from_size_align_unchecked`。该栈定位了观察点，不证明 `Vec`、Rust 分配器、wgpu 或某一个纹理描述符本身有缺陷，也不证明内存破坏或栈溢出。不要把测试页停止等待当成取消或关闭 Wasm。

| 保留证据 | SHA-256 |
|---|---|
| 接管前绑定后 Wasm | `4148cc13d15fd79d82d943d528068b31d1a618fe3a88af96be59519855c95c0d` |
| 接管后绑定后 Wasm | `b47a5166ec35c5f5dec9606fa7a4aaeb25933485898ab6752c9acbc049736d68` |
| 同一空目录 | `94752d984ac78276ea81450f03c070e27b4a23fa968cbc5532cc4c194478c77e` |
| 同一空场景 payload | `68d78f71b988a87761170421b757a2ff31b41c9dcfb5c44bd4e5124142c78bd2` |
| 历史系统纹理观察（空场景、已有 HUD） | `68b02f5202cc1bdd3780309133b6c2d9bf932435f9c05a0e5a19fe6250c6b596`、`97d7d3cb72561bb68585a6f604c82f4040093d3a5374ed991495bb8bc50ca7d3` |
| 接管后历史 `zircon_app` 构建的普通回执（不封存源码） | `3ef9ca86d3adc1fdd0ca62a6890ec2a0812f9d51e2e3b90a8903212ab1b65953` |

## UE / ZE 责任边界

- UE 6.0.0 / UE5 / CL 0：`FColoredTexture::InitRHI` 在 RHI 初始化器中写入 1×1 RGBA，再 finalize 与创建 SRV；黑、白、不透明和透明默认值由 `TGlobalResource` 的 `Pre` 阶段登记。`FSolidColorTextureCube::InitRHI` 为六面分别写入，再 finalize。源码见本地 [GlobalRenderResources.cpp:47–74](../../../dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/GlobalRenderResources.cpp#L47-L74)、[242–267](../../../dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/GlobalRenderResources.cpp#L242-L267)、[529–575](../../../dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/GlobalRenderResources.cpp#L529-L575)。原测 [Rendering.RHI → RHI Update Texture2D](../../../dev/UnrealEngine/Engine/Plugins/Tests/RHITests/Source/RHITests/Private/RHITests.spec.cpp#L75-L82) 实际调 RenderThread；[实现](../../../dev/UnrealEngine/Engine/Plugins/Tests/RHITests/Source/RHITests/Public/RHITextureTests.h#L858-L871) 也覆盖三维及支持时的 buffer upload。没有找到它对 Wasm 初始系统纹理建批的同形测试，未执行 UE。
- ZE [构造 Renderer](../../../zircon_runtime/src/graphics/scene_renderer_construct/new_with_icon_source.rs#L310-L322) 先取得实际设备代的系统纹理租约。该 [代际 owner](../../../zircon_runtime/src/graphics/backend/backend/textures.rs#L133-L164) 准备资源和整批数据，之后才入队、flush 和发布租约；[prepare 与 push](../../../zircon_runtime/src/graphics/backend/backend/textures/resources.rs#L199-L221) 属于调用 `enqueue_native_texture_upload_batch` **之前**的分支。保留 Rust/wgpu、中立 RHI、每设备代的资源所有权和一批一个票据，不照搬 UE 的全局初始化或 Vulkan/D3D 锁定接口。不能用提高 Canvas 能力、压低桌面格式或改 cooked 字体绕过。
- ZE [payload 长度原测](../../../zircon_runtime/src/graphics/backend/backend/textures/tests/payloads.rs#L10-L30) 不建真实 batch、设备或画布；[紧密行上传与 GPU 回读原测](../../../zircon_runtime/crates/zr_rhi_wgpu/src/production/tests/texture_copy.rs#L175-L222) 只验证 2D copy，不覆盖启动路径。不要把它们或历史 #243 首次配置通过当作这个故障通过。

## 独立回归顺序

1. 用不变空场景、相同 Canvas/设备追踪和确切 Cargo/wasm-bindgen 输入复现同一边界；先确认产品 Wasm、profile/linker 与构建来源，保留原始浏览器异常和停在哪条准备操作。改变编译器、profile 或栈配置只能作区分原因的对照，不把绕过 trap 当成修复。
2. 以真实队列和 GPU 回读移植 UE 相关初始化/上传断言：确认批次计数和切片范围，六面与卷的层数据正确、一次提交且资源在当前代归属；重复取得同一代不重复上传，错误不得发布部分资源。通过实际 Web 产品重复无资源和带 UI 的正常场景，检查 `load`、提交、首帧、resize 与释放，不只看配置或浏览器控制台停止报错。
3. 首次提交只登记实证、职责和后续验证；生产修复/测试另在本 Draft 上交付。仍要求实际 GitHub `editor-compile-gate`。不混入 [#243](https://github.com/He-Jiahui/ZirconEngine/pull/243) 或其前置 Draft。
