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

## 双视口的实际像素与缓存归属

NVIDIA GeForce RTX 3060 Laptop GPU 的 DX12 实测复现截图归属错误。六个用例中有 5 个读取了 B 的图像或把它留在 A 的缓存中，两个既有离屏与 HDR 对照均通过。 每次只向同一渲染框架提交受控的红色或绿色相机。参考读回同时核对像素尺寸、RGBA 长度、generation 和捕获来源。失败中的 `equals_B` 逐字节比较整张图及其来源报告。调用对相同代数返回 `None` 后，A 的两种轮询仍读到了 B 的整帧。

| 顺序和请求 | 结果 |
|---|---|
| 仅 B 提交，A 与 B 尺寸不同，直接请求 A | 失败 |
| 两个视口尺寸相同，未提交的 A 请求任意一代 | 失败 |
| 未提交的 A 请求比 0 更新的一代 | 失败 |
| A 原有红帧，提交绿色 B 后重新截图 A | 失败 |
| A 拒绝 B 的相同代数后，再轮询 A 的缓存 | 失败 |
| 未知句柄、空缓存与有效 B 缓存对照 | 通过 |

中心像素的独立参照：A 的 40×24 第 1 代为 `(255, 0, 0, 255)`；B 的 40×24 第 1 代为 `(0, 255, 0, 255)`；B 的 40×24 第 2 代为 `(0, 255, 0, 255)`。保留 A 自己已有的红帧或返回 `None` 均可满足未取得新 A 产品时的直接请求。后续轮询仍须维持 A 自己的缓存。

运行使用 `zircon_runtime --lib` 的 `native-host,graphics` 配置。执行树采用 [#137 的模型测试夹具](https://github.com/He-Jiahui/ZirconEngine/blob/7f9a8a70e71a700e45300bab4f64094541507fa4/zircon_runtime/src/graphics/scene/resources/tests/model_geometry_resolution.rs#L293-L315) 解决同一测试库的借用前置。六项回归和两项既有对照共享该程序。UE `GetViewportScreenShot` 对传入的 `FViewport` 读取像素，`AScreenshotFunctionalTest` 提供 GameViewport 截图流程的原测参照。这里的双句柄与代数断言源自 ZE 的逐视口接口合同。

[双视口回归](https://github.com/He-Jiahui/ZirconEngine/blob/fcaf7f4000957dc746938d6ab5675740286b29a4/zircon_runtime/src/graphics/tests/surface_targets/capture_owner.rs)

## 按请求视口的已发布代数读取截图

修复后，六个按视口选择与筛选截图的用例由 1/6 提升至 6/6。插件实际写入的蓝色截图、A 的红色缓存和两个原有捕获对照均通过。同设备上共 17/24 项通过。

读回活纹理之前，渲染器先匹配最新一次成功提交的回执。视口自身已有的截图继续供查询使用，严格更新筛选在同步写入缓存之前执行。实现复用成功视口提交已有的代数和表面重配的失效处理。

| 选定用例 | 修订前通过 | 修订后通过 |
|---|---:|---:|
| 请求归属与同代筛选 | 1/6 | 6/6 |
| 已有离屏与 HDR 对照 | 2/2 | 2/2 |
| 失败提交、worker 错误和空写 | 0/3 | 1/3 |
| 相机纹理 | 6/8 | 6/8 |
| 分屏、场景和 UI 合成 | 1/3 | 1/3 |
| 插件直接写蓝色终态纹理 | 本次新增 | 1/1 |
| 主相机先提交，纹理相机后提交 | 0/1 | 0/1 |

B 的自定义最终执行器只声明写入，实际执行为空。B 成功提交后，同步截图和轮询仍把 A 的整张旧图标为 B 的第二代。这个独立用例继续失败。

修订前后在同一位置出现的其他结果：重分配夹具的测试通道缺少资源声明，在注册时退出。纹理目标截图本身的来源断言通过。随后查询到的渲染统计仍记录离屏来源。分屏右侧读到绿色，左侧未读到预期红色。动态分辨率 UI 用例在字体图集上传准备时退出。主相机先于纹理相机的用例在提交时遇到模型 UUID 与资源地址不符。

测试使用 Windows x64 的 `zircon_runtime` 库程序，启用 `native-host,graphics`。两次编译都使用同一份 [模型测试夹具](https://github.com/He-Jiahui/ZirconEngine/blob/7f9a8a70e71a700e45300bab4f64094541507fa4/zircon_runtime/src/graphics/scene/resources/tests/model_geometry_resolution.rs#L293-L315)。新增插件用公开 GPU 编码器录制 `Clear+Store`，校验读回蓝色，并逐字节核对 A 的红色缓存。窗口呈现、单相机局部写入和其它图形后端仍待实测。

[同步请求](https://github.com/He-Jiahui/ZirconEngine/blob/cc7bb9726301d1e25af275d1b7995b57f3d1da0e/zircon_runtime/src/graphics/runtime/framework/capture_frame/capture_frame.rs) · [读回判断](https://github.com/He-Jiahui/ZirconEngine/blob/cc7bb9726301d1e25af275d1b7995b57f3d1da0e/zircon_runtime/src/graphics/scene/renderer/core/pipeline/pipeline/readback.rs)
