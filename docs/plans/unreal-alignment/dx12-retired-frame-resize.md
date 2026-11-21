# DX12：已呈现帧的原生引用可能阻断正常窗口 resize

固定公开源码：[ZirconEngine main `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`](https://github.com/He-Jiahui/ZirconEngine/tree/e89349ca51b4e9d024c194cd3bdf645bd25b6c37)。这是测试与修复前的独立调查。已核对该源码与一份历史本地 Windows/DX12 实跑；**没有在当前基线执行，也没有证据历史修复已进入该公开基线**。范围是同一正常原生窗口已呈现后、两个非零 client extent 之间的 swapchain 重配置；不涉及注错、最小化、其它后端或屏幕截图归属。

## 1. 正常入口与预期合同

正常 Standalone 主窗口有 [Winit `SurfaceResized` 分支](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/runtime_entry_app/window_events/dispatch.rs#L16-L46)。不同尺寸会[更新 Runtime viewport 并重新绑定同一原生窗口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/runtime_entry_app/surface_present/resize.rs#L11-L62)，之后继续正常 acquire/render/present。这里的目标是保留窗口与设备，不把重配变成新的平台呈现路径。对同 target/device 的[图形框架重绑定](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/runtime/viewport_surface/viewport_surface.rs#L15-L73)先结束[框架的排队工作](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/wgpu_render_framework/wgpu_render_framework.rs#L545-L552)，并不直接回收 RHI retirement；随后复用原生 surface 走[现有 session reconfigure](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/backend/backend/viewport_surface.rs#L120-L151)。预期在旧 back buffers 可释放后重配置，原窗口/设备不变，旧租约失效，新 session 能以新 extent 获取并呈现。由 [DXGI `ResizeBuffers` 的资源生命周期合同](https://learn.microsoft.com/windows/win32/api/dxgi/nf-dxgi-idxgiswapchain-resizebuffers)，GPU 空闲不是旧 back buffer 直接/间接引用已释放的替代品。

## 2. 公开源码中的失效顺序

- [正常 `present_frame`](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface.rs#L214-L246)验证提交使用了该帧，退役其资源并消费 lease。此时 [registry](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/registry/resources.rs#L419-L461)使 Texture/View 的 public handles 立即失效，但有登记使用票据的原生对象仍由[退休队列](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/registry/retirement.rs#L34-L74)拥有，直到 `reap_retired` 发现其所有票据均为终态。提交完成不会自动 drop 该队列中的对象。
- [RHI `reconfigure_surface_session_impl`](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/device/surface_lifecycle.rs#L70-L92)结算 session 活跃帧的票据；[票据枚举](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface.rs#L314-L331)不含已呈现且从 `frames` 移除的帧，[settlement](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/submission.rs#L621-L653)也不是等待已提交工作的完成屏障。随后 [surface service](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface.rs#L88-L112)丢弃仍活跃的帧并调用 `Surface::configure`；[设备的终态修剪与 reaping](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/device.rs#L217-L233)在 configure **之后**。若完成与回收之间收到一次有效 resize，旧引用可能仍在。不是「每次 resize 都失败」。
- [锁定的 WGPU DX12 依赖](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/Cargo.lock#L8516-L8522)在已有 swapchain 的 configure 分支先等待 present queue、释放 HAL 自己的 buffer 引用，再调 `ResizeBuffers`（`wgpu-hal 29.0.4`, `src/dx12/mod.rs:1315–1363`）；这不会清掉上层 Zircon registry。HAL 调用失败可以在 `wgpu-core` 汇成 `Invalid surface`。该文案不唯一指向某个 DXGI 原始 HRESULT；此案以实际时序、源码和历史试验区分，而不是仅凭通用 Validation 字符串认因。

## 3. 实际证据及可迁移范围

2026-11-10 的独立本地调查：真实打包编辑器的 Tools → Launch Standalone 启动并已画出 Cube，client `1280×720 → 1280×704` 后 Runtime 由 `Validation` 退出（native exit 1）。同旧包的独立诊断捕获 `In Surface::configure` / `Invalid surface`；不计作新的打包首次启动验收。独立的隐藏 Win32/DX12 原生回归先真实 acquire/submit/present，确认 GPU 已完成而退休分配仍保留，再要求 resize/reacquire：旧产品 + 测试补丁的该项 **0/1，exit 101**，在无 fault 注入下出现同一个 configure Validation。后续历史本地候选的生命周期组 **6/6，exit 0**，含同名 DX12 用例；修后另行构建的本地包在三个新项目/新 profile 首次启动时均跨过相同尺寸变化并正常退出。

**不是当前 RED/GREEN 配对。** 独立检查了两次历史原始输出、源位置/退出回执和前后源码指纹。历史 RED 与后继修后的 test leaf 指纹不同，虽为同名用例与同契约，不能写成冻结的同字节测试双跑。公开基线与历史旧侧的关键 App/Framework/RHI resize、registry retire、submission、crate manifest 与 lock 对象一致（核对 14 个相关对象）；并非整个树或历史 EXE 与当前一致，邻近 texture-copy 代码另有变化。本地修复不是公开基线的祖先，且现有路径仍是先 configure 再 reaping；历史结果不授予当前产品/主线验收。

## 4. Unreal 的真实参照与原测试

- `Engine/Source/Runtime/D3D12RHI/Private/D3D12Viewport.cpp:849–853`：`FD3D12DynamicRHI::RHIResizeViewport` 在游戏线程调用 `FD3D12Viewport::Resize`。`:278–289` 防重入，`:312–357` flush/idle、清理 transient 绑定、通知 `CustomPresent`；逐个 `DoNotDeferDelete`、`SafeRelease`（包括配置下的 SDR back buffers），再清 present queue、flush/idle。`DoNotDeferDelete` 禁止延迟释放，不只是把 viewport 成员置空（`Engine/Source/Runtime/D3D12RHI/Private/D3D12Resources.h:355–360`、`Engine/Source/Runtime/D3D12RHI/Private/D3D12Resources.cpp:1343–1362`）。`D3D12Viewport.cpp:428` 才进入 `ResizeInternal`。
- `Engine/Source/Runtime/D3D12RHI/Private/Windows/WindowsD3D12Viewport.cpp:293–375`：`FD3D12Viewport::ResizeInternal` Windows 普通分支先 `ResizeBuffers`，再检查 back-buffer 指针为空并重新获取；MGPU 路径用 `ResizeBuffers1` 再重新获取。`:377–411` 是实际运行时 present 路径而非测试。两者共同说明 release-before-resize；不要求复刻 UE 的两次全局 idle 或多 GPU 实现。
- 查到真正近邻原测 `FSetResTest` / **`System.Windows.Set Resolution`**（`Engine/Source/Runtime/Engine/Private/Tests/EngineAutomationTests.cpp:72–106`）：desktop `ClientContext | EngineFilter`，打开真实 Game 默认地图，保存 `GSystemResolution`，等 2 秒，发 `setres 640x480`，再等 2 秒并恢复原值。命令走 GameViewport `Exec`（`Engine/Source/Runtime/Engine/Private/Tests/AutomationCommon.cpp:1123–1135`）、请求真实分辨率变更（`Engine/Source/Runtime/Engine/Private/GameViewportClient.cpp:4202–4233`）。这是 resize smoke，**未指定 DX12、没有显式断言新 back-buffer extent 或 present/重获、也不测已完成但未回收引用**。未找到同形 UE 自动测试，未运行 UE；不把源码证明或其它测试的 viewport 命令算成实测。

## 5. 独立边界与去重

只讨论 **renderable → renderable** 的已呈现原生 DX12 session 生命周期。共享 [RHI raw zero extent 非 renderable](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi/src/surface.rs#L15-L38)，[reconfigure 在该分支跳过 configure、acquire 返 NonRenderable](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface.rs#L96-L131)，而 [Standalone 的收到尺寸归一为至少 1](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/runtime_entry_app/surface_present/resize.rs#L11-L21)。不拿 raw zero、测试 headless Vulkan、CPU 诊断 presenter 或恢复窗口替代本案，也不据此变更最小化策略。[原确定性测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/tests/surface_lifecycle.rs#L85-L142)保 session 粒度/取消/零尺寸合同，但不触及真实 Windows swapchain。[Framework 的 source guard](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/tests/surface_targets.rs#L59-L109)不测原生资源引用。既有[视口 owner 审查](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/_archive/performance/01/2026-08-23-runtime-dynamic-api-root-viewport-json-current-architecture-review.md#L27-L35)关注 resize 不应销毁重建整个 viewport；[surface session plan](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/zircon_runtime/frameworks/01/2026-08-28-rhi-wgpu-surface-session-index-preflight.md#L69-L89)有方向性 owner 约束，不是当前缺陷的已发布修复或配对回执。已核 233 份 GitHub 全状态 PR 的标题/正文和 8,026 条改动路径记录，超分页的 14 项以公布的两端 Git 对象补全并校验计数。唯一改过同路径的 [#120](https://github.com/He-Jiahui/ZirconEngine/pull/120) 只在非 Windows 的 `create_native_surface` 接入 Linux 原生句柄，不触及 back buffers 或 resize。未见同题；[#178](https://github.com/He-Jiahui/ZirconEngine/pull/178) 是捕获归属、[#231](https://github.com/He-Jiahui/ZirconEngine/pull/231) 是 indexed VG adapter admission、[#232](https://github.com/He-Jiahui/ZirconEngine/pull/232) 是 UI 滚动距离；均不混入。这份 PR 在原 GitHub `main` 上从单一调查文档开始；未取得、搬运或修改其它分支的生产补丁。

## 6. 下一步：独立 Draft 后冻结回归，不预认历史修法

先提交调查文档；经授权后在同一 Draft 的公开基线上增加窄 tests-only 见证。Windows/MSVC `zr_rhi_wgpu`、默认关闭、无新增 feature、test profile、单个精确 libtest：真实 Win32+DX12，同 window/device 执行 acquire→用帧目标的真实命令提交→present，等指定提交完成但不提前修剪 retirement，有独立证据确认这一帧的真实旧引用仍被 registry 保留，再做非零 resize。记录 configure fault、replacement session/目标 extent 和重新 acquire；补正常再 submit/present、旧 lease/handle 拒绝与不影响 raw zero 路径的对照。不得以缺 DX12 adapter、仅较早编译失败、没有真实呈现或其它前置失败冒充 RED。先在固定旧产品取得实际 RED，再评审可行性：由 RHI owner 防并发 acquire，保留票据/错误合同，安全、可失败地等实际在途工作，**原生 configure 前**释放已终结的旧引用；不把等待放进正常每帧，也不抄历史本地补丁或无条件改所有后端/零尺寸政策。最后以相同冻结用例验旧/新并独立保留必要对照；新的产品或最小化结果须另测。


本地 Unreal 6.0.0 / UE5 / CL0 源参照指纹（没有可信上游提交号，未运行 UE）：

- `Engine/Source/Runtime/D3D12RHI/Private/D3D12Viewport.cpp` SHA-256 `d9db11434426058a95e8d55ae8771f0e6bb9f1447cf2d59f5936601e968669f3`
- `Engine/Source/Runtime/D3D12RHI/Private/Windows/WindowsD3D12Viewport.cpp` SHA-256 `bbd0df190f24c327806f08c9361f312ed2d439dccd7d751ba48e59db4e9c34e9`
- `Engine/Source/Runtime/Engine/Private/Tests/EngineAutomationTests.cpp` SHA-256 `158c214df2c035c6d3f3075b7613b00bca6f2943486847ab6db4c8a073d19910`
- `Engine/Source/Runtime/Engine/Private/Tests/AutomationCommon.cpp` SHA-256 `5b58be34bb20569f18c1cbf7339df9baf95a0caec5ae870d38cd71dfd8593673`

## 7. 第一次构建与真实执行：控制未到达设备身份记录

`0cb2e17cbf68a390d522aaba0672659ad8c7fad8`，Windows x64/MSVC、`zr_rhi_wgpu --no-default-features --lib`，test profile。真实编译回执 `125238be9cc33443d2110c7f504160f96c790654f2ef6bed5fd1fdb07a9f3d58`，EXE `3829e171fb86c9e5e38be950d2feb197f4826b7ebe6bce5905806aea6d6c7508`。只列出两项 `native_resize_`，同一枚 EXE 再按完整名称单跑 `native_resize_drained_control`。测试进程已进入 libtest 并打印这一项的名称，但未出现 `NATIVE_RESIZE_ADAPTER`、没有可证实的 submitted/presented、stderr 为空。60 秒外部限时届满，已绑定进程树全部终止，退出 124；运行回执 `7531c10c74498474b35f7ce0e9a221b7d6cacca4efdaf7040fbd621ac7ada584`。

这不算一项命中窗口重配的失败；按先对照的约束未启动 `native_resize_retired_frame`。后续仅给原测试夹具添加阶段观测以区别阻塞位置，生产实现仍未修改。独立于已有历史 DX12 非零 resize 回执。

## 8. 同一 EXE 的阶段和两种限时，未达成重配门

Windows x64 MSVC，`zr_rhi_wgpu --no-default-features --lib --profile test --locked --offline`，源码 `79521141905b4dda2bb89b0c6aeeac1c150889f1`。只加阶段打印，去掉它们后两测试及夹具与上轮的 Git 前像逐字节相同。实际 EXE `102da85832525e7c5bb2210dcf35c9811e386e1fcf23980f82274896c2cb1a8e` 不变；两次依相同完整名选中同一用例，限时和原始进程分开留存，过限均核到整个测试进程树已结束。

| 进程 | 实际退出 | 实际阶段 / 回执 SHA-256 |
|---|---:|---|
| 原限时（60 秒）正常回收对照 | 124 | `hwnd_create_start → hwnd_created → wgpu_instance_start → wgpu_instance_ready → win32_surface_start → win32_surface_ready_start_adapter`；`1bc18d67fb946e0ce36ae6f72207920acb09066c144ee0eb14a9ffae20cd958b` |
| 延长限时（180 秒）正常回收对照 | 124 | `hwnd_create_start → hwnd_created → wgpu_instance_start → wgpu_instance_ready → win32_surface_start → win32_surface_ready_start_adapter`；`4fc71e084d5db2f358158ba67c135250d5cdc75151b1eef7b2c4c016d69e7768` |
| 保留旧引用的见证 | — | 按前置约束未启动。 |

确认窗口/实例/surface 建立。`win32_surface_ready_start_adapter` 在正式 [select_compatible_adapter](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface_bootstrap.rs#L58-L144) 之前，`compatible_adapter_selected` 在其成功返回之后。该函数包括本机 DXGI 适配器枚举及 surface capabilities/策略筛选，两者还没有被单独计时；只凭这段等待不能归因到消息泵、驱动、窗口调整或设备故障。

真实非零窗口、WGPU 实例和 surface 均已建成；同一 EXE 在 60 秒和 180 秒均未完成兼容适配器选择。未到呈现或 resize，无本机重配 RED；没有提交推测性的引擎修复。 不把历史包、机上安装的显卡或成功编译替代当前用例运行。

## 9. Win32 对照的适配器枚举与能力查询

真实 Win32 窗口和 surface 创建成功。测试进入 DX12 适配器枚举后等待了 60 秒，枚举没有返回，因此还未对任何候选查询窗口能力。

| 进程 | 退出码 | 结果 |
|---|---:|---|
| 两个既有测试名的枚举 | 0 | 均在实际编译的 libtest 中 |
| `native_resize_drained_control` | 124 | 受限的 60 秒观察窗，`enum_begin` |
| `native_resize_retired_frame` | — | 运行条件取决于前一项正常回收对照通过 |

该观察点在实际 `enumerate_adapters` 和逐候选 `get_capabilities` 调用两侧。Windows x64/MSVC，`zr_rhi_wgpu --no-default-features --lib`，`test` profile。源码 `3a4d2d6c18e85b394145e5769113ee723670bc0d`，EXE `d7875e1a8795bd83984c49a8b08c5d9ace64035e2498a816182329a917cb512b`，对照执行回执 `04cdd7509df0248d56f827d3942cfd0a68521bb0b728c95c0363ca6a602785e5`。实际选择日志依次为 `enum_begin`。

## 10. 真实桌面上的正常重配与旧引用复现

在同一 RTX 3060 Laptop 上，正常回收旧帧后，192×128 的窗口成功调整到 256×144 并再次呈现。保留已完成帧的旧引用时，WGPU 报 `Surface::configure: Invalid surface`，ZE 同时返回 `Ok(Renderable)`。

Windows DXGI 在 0.047 秒枚举出三项。精简环境的独立系统进程在第一项 RTX 3060 上进入 `D3D12CreateDevice`，到 45 秒截点仍在调用中。恢复桌面用户及驱动所需的普通 Windows 环境变量后，三个条目都成功创建并释放设备。两轮系统探测的差异定位到环境变量集合，单个变量的归因仍待测量。

随后运行同一 Windows libtest EXE，设备为 Dx12、Discrete，vendor `10de`、device `2560`，驱动 `32.0.16.1714`。对照和见证分别在自己的进程中执行，保持原 60 秒上限、测试体和窗口尺寸。见证先收到已完成 GPU 提交的 `Completed` 票据，观察到 98,304 字节的旧资源进入退休队列，再调用 `reconfigure_surface`。

| 已编译用例 | 实际结果 |
|---|---|
| `native_resize_drained_control` | 首帧、正常资源回收、非零尺寸调整及下一帧均通过 |
| `native_resize_retired_frame` | `Surface::configure` 报 Validation / Invalid surface，应用侧 RHI 返回新尺寸 Renderable |

实跑源码 `3a4d2d6c18e85b394145e5769113ee723670bc0d`，EXE `d7875e1a8795bd83984c49a8b08c5d9ace64035e2498a816182329a917cb512b`。当前草稿 `d38adbfe42cbfcee887e0dea4f08e3060201f3f1` 在该源码上追加了观察记录。对照回执 `e02bc8d4cb3f24d5a054dff3dd37a1a06bd6159533542c7027b9c8b4b0663a33`，见证回执 `ad3e88cf58c4d1320198fe4bcdf564592e1e97c732d89bb13a5108dca13fc162`。

## 11. 已完成的旧帧资源在窗口重配前释放

同一台 RTX 3060 上，正常回收对照和保留旧引用的用例均完成 192×128 到 256×144 的窗口调整，并成功取得、绘制和呈现下一帧。原先失败的用例在重配前仍有 98,304 字节的已完成帧资源。

现在每个交换链从首次创建起保留自己的资源归属。Win32 DX12 在配置非零尺寸前，释放属于该交换链且所有使用票据都已终结的退休纹理和视图。已提交但仍在途的帧、外部持有的原生视图和同时调整两个窗口各有独立的生命周期。

定向验证使用 Windows x64/MSVC 的 `zr_rhi_wgpu --no-default-features --lib`，`test` profile。修复前同一份测试 1 过 1 败，修复后 2 过。源码 `00fe9442f0f7d63b7e0f01f0d90155f117a56980`，EXE `2be6d4bb24285accb4e440a21dc5cc2e95e5183a3ecbd1a4200f559fb83d1774`，两个执行回执分别为 `1c0142f6f3d5aaadf1c012c820e7c52c31c77b74d3720f6f8b426dcd526fcfb6` 和 `6b6e05093d861b5faaf78647966052ef9cdc563d2cf5f806e044ce1004eb896c`。

## 12. 同一测试程序的提交与资源生命周期回归

修复后用于窗口重配的测试程序 `2be6d4bb24285accb4e440a21dc5cc2e95e5183a3ecbd1a4200f559fb83d1774` 还通过了三项原有用例。每项均在单独的原生进程中执行。

| 原用例 | 检查的行为 |
|---|---|
| `surface_teardown_racing_flush_has_only_atomic_terminal_outcomes` | 窗口收尾与提交队列并发执行时，整批票据得到一致的终态 |
| `surface_teardown_settles_reserved_pending_and_duplicate_tickets_once` | 已保留、已入队和重复票据仅结算一次，命令上下文得到归还 |
| `native_submitted_cancel_refusal_and_abandoned_batch_keep_original_receipt_owned` | 已提交的资源继续由原票据持有，混合批次中可取消票据正常释放 |
