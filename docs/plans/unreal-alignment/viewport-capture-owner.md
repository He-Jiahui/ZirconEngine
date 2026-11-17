# [Media/Capture] 对照 Unreal：同步视口截图不能返回另一视口的帧

**状态：独立问题 Draft 研究稿，仅源码论证。** 无测试、实现、构建或 GPU/GUI 实测，无行为 RED/GREEN。ZE 固定 [独立基线 e89349c](<https://github.com/He-Jiahui/ZirconEngine/tree/e89349ca51b4e9d024c194cd3bdf645bd25b6c37>)。本机 UE 参考树 [Build.version:1–10](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L1-L10>) 声明 `6.0.0 / UE5 / CL 0`，UE 提交未知；仅提供路径、符号、行号和文件指纹，不公开 Epic 源码正文。视频、XR 和虚拟制作能力缺席不属于此问题。

## 1. 不一致：视口 A 的同步请求返回 B 的帧

[ZE 捕获合同](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/wiki/tutorials/advanced/render-viewport-frame-capture.md#L207-L226>) 按 `RenderViewportHandle` 请求，无可用帧是 `None`；同一文档区分了[全局统计](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/wiki/tutorials/advanced/render-viewport-frame-capture.md#L186-L188>)。`capture_frame` [不筛 generation](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/rendering/render-framework-architecture.md#L1089-L1091>)，不等于放弃视口归属。文档还[明确区分无完成捕获与未知句柄](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/wiki/tutorials/advanced/render-viewport-frame-capture.md#L273-L280>)。

在一个 WGPU framework 中创建 A=`64×48`（从未提交）和 B=`40×24`（成功提交一帧普通离屏相机），无其它提交。GPU 读回成功时，当前 `capture_frame(A)` 会返回并记录 B 的尺寸、RGBA、来源报告和 generation。A 应返回 `Ok(None)`，B 仍正常可捕获。对曾经提交的 A，只要求「不返回别的视口」，不要求为每个旧视口另存 GPU 输出。

## 2. 实际调用链与有界影响

- [Graphics 注册生产 WGPU RenderFramework 服务](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/host/module_registration/module_descriptor.rs#L90-L137>)，支持[独立 viewport record](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/runtime/framework/create_viewport/create.rs#L9-L24>)；公开 `submit_frame_extract` [进入相同 renderer 提交路径](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/runtime/framework/extract/submit/submit.rs#L161-L202>)。现有 in-crate headless fixture 可用 [`pub(crate)` 的 `submit_runtime_frame`](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/runtime/extract/submit/submit_runtime_frame.rs#L161-L213>)；[PBR 诊断查看器](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs#L582-L626>)也消费生产 Rust capture。查看器示范同一视口上提交再读取，不声称其现有使用路径误取。
- 错帧[被写入 A 的缓存及提升代数](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/runtime/framework/viewport_record/capture.rs#L43-L61>)，随后[直接 nonblocking A poll](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/runtime/framework/capture_frame/capture_frame.rs#L111-L146>)在锁可取得时会读到该条目。未经此前同步写入，未渲染 A 的记录为空，poll 返回 `None`；锁竞争的立即 `None` 是另一正常情况。不将此归咎于异步 mailbox；[动态 ABI 捕获桥接使用的是 poll](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/runtime_loop.rs#L142-L164>)，不声称 ABI、App PNG、Play 或常规 UI 受该回退影响。

## 3. 根因与 UE 的真实处理

**ZE：** [同步回退](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/runtime/framework/capture_frame/capture_frame.rs#L11-L78>)验证 A 存在，但仅在 `Some(last_generation)` 且 A 有更新的缓存时采用该缓存。无筛选或无合格缓存时，进入[无视口参数的 `capture_latest_frame()`](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/renderer/core/pipeline/pipeline/readback.rs#L25-L78>)。B 通过普通 [`frame_submission_owner` 提交](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/core/pipeline/frame_submission_owner.rs#L344-L358>)后留下共享目标和当前 renderer generation。读取该目标后，调用方未核对来源即存入 A。尺寸不是身份：视口可能同尺寸，纹理目标还可有独立尺寸。

**UE：** [`GetViewportScreenShot`](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/UnrealClient.cpp#L2413-L2423>)调用指定接收者 `ReadPixels`；[读回逻辑](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/UnrealClient.cpp#L66-L84>)跨线程携带该接收者并[取其目标纹理](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/UnrealClient.cpp#L198-L216>)。[FSceneViewport 实现](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Slate/SceneViewport.cpp#L2095-L2102>)同样取实例的目标。MediaIOCore 的独立旁证是 [`CaptureSceneViewport`](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Media/MediaIOFramework/Source/MediaIOCore/Private/MediaCapture.cpp#L272-L304>)绑定指定来源、[取该弱引用视口的纹理](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Media/MediaIOFramework/Source/MediaIOCore/Private/MediaCaptureSources.inl#L353-L372>)、[OnTick 路径的渲染回调比较视口身份](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Media/MediaIOFramework/Source/MediaIOCore/Private/MediaCapture.cpp#L1546-L1552>)。只对照「来源精确匹配」；不推断 UE 对未初始化视口的返回状态和 ZE 一样。

## 4. Rust 方案：在现有回退前验证物理来源

保留现有同步完成、锁和错误语义。在共享 readback 前，使用提交权威校验保留产物属于请求视口及其当前有效版本。证明不成立时仅可用 A 自己已完成且合格的缓存，否则 `None`；不读 B，也不更新 A 缓存或成功计数。不引入硬件录制、额外每视口 GPU 读回或新架构。可参考 [HDR owner guard](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/runtime/framework/capture_frame/capture_frame.rs#L81-L108>)，但需复核发布/失效时机，不能把 HDR 的旧 handle 直接视作 LDR 证明：renderer [分配可先于外层提交成功](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene/core/pipeline/frame_submission_owner.rs#L93-L125>)。可行性复核涵盖失败、resize、纹理目标和 present，保留已有缓存、严格较新过滤与同视口读回。

## 5. 接口与 ECS

UE 依赖 `FViewport/FRenderTarget` 接收者的 RHI 资源；ZE 已有 [Rust `RenderFramework` 和视口句柄](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/render/framework.rs#L147-L160>)、每视口记录和共享 renderer。修复限定在此内部身份边界，不改 ABI、Scene ECS、相机世界变换、HDR 色彩域、非阻塞协议、App 或 UI。

## 6. 原测和回归草案

- **UE 原有相邻测试：** [截图功能测试](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/FunctionalTesting/Private/ScreenshotFunctionalTest.cpp#L92-L174>)验证所选游戏视口的截图流程（含可选区域）；[MediaIO 去隔行测试](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Media/MediaIOFramework/Source/MediaIOCore/Private/Tests/MediaIODeinterlacerTests.cpp#L140-L162>)验证不同场处理。有限源码搜索未发现同一 A/B 步骤的 UE 原测；以下是**源码派生回归，不是移植**。
- **ZE 落点：** [既有 native headless WGPU 双视口 HDR 测试](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/tests/surface_targets.rs#L366-L404>)及[同视口 RGBA8 捕获控制](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/tests/surface_targets.rs#L205-L247>)。仅提交 B；B 先正常捕获 `40×24`、正确 RGBA 字节形状及非零 generation。A 的第一次 poll 应为 `None`。采集同步 A 和紧随后 A poll 的两个结果后再断言，均应 `None`；原实现预期二者带 B 尺寸、generation 和同一 RGBA。独立用全新 A 验证 `capture_frame_if_newer(A, Some(0))` 应 `None`。保留 B 正控及既有 HDR/单视口对照，另加未知 handle 对照，不用假 renderer 或源码字符串代替见证。
- 先仅问题 Markdown 独立 Draft，再同 PR 添加窄回归；只有 B 的真实 GPU 正控成立且原实现在目标断言失败才算 RED。无 adapter、编译失败或源对照不算行为结果。完成独立可行性评估后才最小修复并复跑；本稿未执行这些步骤。

本机 UE 指纹（raw SHA-256）：

| 本机文件 | SHA-256 |
|---|---|
| [Build.version](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version>) | `3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6` |
| [UnrealClient.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/UnrealClient.cpp>) | `901a858bcacb50cbe9d97b5392431ffa0c2df157725bc270c933955b2ab9c482` |
| [SceneViewport.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Slate/SceneViewport.cpp>) | `d0ba08074bd6a111ab1bb918102ccdff358a6749ed4f2614b164ce8eb0f9d3b4` |
| [MediaCapture.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Media/MediaIOFramework/Source/MediaIOCore/Private/MediaCapture.cpp>) | `e1d32c9b2dbc361c90f8ce5900daa1737829d901c07e4b5e5931dfbae4f69df0` |
| [MediaCaptureSources.inl](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Media/MediaIOFramework/Source/MediaIOCore/Private/MediaCaptureSources.inl>) | `2d99edc9148c61abdff634d64e3389e2b701df03f9e8ae3207bc3e54924a82f9` |
| [ScreenshotFunctionalTest.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/FunctionalTesting/Private/ScreenshotFunctionalTest.cpp>) | `952b2f7571dc657317a148d1ce68db70811db8e0a537e1b7b5104d14099953dd` |
| [MediaIODeinterlacerTests.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Media/MediaIOFramework/Source/MediaIOCore/Private/Tests/MediaIODeinterlacerTests.cpp>) | `b928b6e49be586c4f6bf6a23ec6500365378286808d19497be01e99e62a2d4a8` |
