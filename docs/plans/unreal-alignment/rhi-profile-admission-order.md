# [RHI] 表面启动在拒绝不匹配的适配器 profile 前请求设备

固定公开基线：[ZirconEngine `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`](https://github.com/He-Jiahui/ZirconEngine/tree/e89349ca51b4e9d024c194cd3bdf645bd25b6c37)。本文只界定 `zr_rhi_wgpu` 的公开 Rust 配置契约；没有执行使用不匹配 profile 的原生设备请求，没有行为回归 RED/GREEN，也没有生产修复。

## 1. 正常入口与既有契约

调用者先用 [`WgpuSurfaceBootstrap` 创建或接收表面并选择兼容适配器](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface_bootstrap.rs#L45-L72)，选择时过滤表面能力，再依策略取得绑定到该表面的 [one-shot 适配器 owner](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface_bootstrap.rs#L110-L144)。该对象是 [crate 公共导出](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/lib.rs#L78-L88)，可借读 `adapter_facts()`，再把拥有的 `RenderDeviceProfile` 交给 [`request_render_device_async` 或原生同步包装](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface_bootstrap_adapter.rs#L32-L68)。[已有 Render17 契约](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/zircon_runtime/render/17/2026-08-24-wgpu-diagnostic-submission-architecture-and-measurement-plan.md#L949-L961)明确要求在向已选 adapter 请求设备和队列**之前**校验外部 profile；[原测试的末段](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/tests/surface_bootstrap.rs#L91-L100)也要求这个先后关系。

## 2. 当前影响与实证边界

基线的[内部请求](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface_bootstrap_adapter.rs#L94-L118)先调用并 await `Adapter::request_device`。若 WGPU 成功，不匹配的输入才得到 [`NativeContextAdapterMismatch { profile_adapter, context_adapter }`](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi/src/device/error.rs#L35-L42)：可以先做本可省掉的设备／队列创建工作，却不会把错误 profile 或 surface session 交给调用者。若原生请求自然返回错误，`?` 先返回 [`SurfaceUnavailable`](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi/src/device/error.rs#L86-L87)，即使 profile 不一致也不会给出原可直接诊断的类型化错误。这是可从执行顺序确定的分支，不是已观察到的分配、超时或驱动故障。

原样执行 `surface_bootstrap_transfers_one_compatible_native_surface_into_the_neutral_device` 源码守卫为 **0 过 / 1 败，exit 101**；测试与五份 `include_str!` 源的 Git blob 均同基线，其中测试为 `dab15c35c6bd219156c3204baf00132bf39c3d41`、受查 owner 为 `4a9d35d47fdb1a53c774daf4f21e7e5d41ee750a`。Windows 的实际第一个失败是[第 18 行 selector 签名字面匹配](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/tests/surface_bootstrap.rs#L9-L21)：测试要求 LF，签出源为 CRLF。只读检查将文本规范为 LF 后，该签名字面可匹配，但[旧同步设备请求及构造器字面](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/tests/surface_bootstrap.rs#L24-L36)仍不匹配现行实现；后段排序断言并未运行。该执行没有调用原生 API，不是对上述生产行为的 RED。另 [#235](https://github.com/He-Jiahui/ZirconEngine/pull/235) 是呈现后的非零 resize；其本轮记录止于适配器选择，尚未到本案所讨论的请求阶段，也没有使用外部 profile 入参。

## 3. 根因与 Unreal 参照

本地 Git 父子差异确认：异步迁移的直接父实现先核 `profile.adapter()`，不匹配就返回；随后才请求设备。迁移提交 `eebf938778543ae9852606b93d7c864d253a0388` 把同步入口变成 async 包装，并将[外部 profile 的检查搬进 `FnOnce`](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface_bootstrap_adapter.rs#L52-L68)。这个闭包在传入时不执行；[设备请求成功后才调用它](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface_bootstrap_adapter.rs#L109-L116)。这一共用流程也服务于**确实需要**已创建 device 的协商 profile，不能把两个入口的阶段要求混为一谈。

**UE 生产近邻（非等价）：** 参考树自报 `6.0.0 / UE5 / CL 0`，提交未知。`Engine/Source/Runtime/VulkanRHI/Private/VulkanRHI.cpp` 中，`SelectPhysicalDevice` 读取 `r.GraphicsAdapter`/`graphicsadapter=`（202–223 行，无效索引按该实现回退）；启用 profile 检查的受支持路径在 `CheckVulkanProfile` 先用临时 `VkInstance` 选物理设备，再查 `vpGetPhysicalDeviceProfileSupport`（290–299、311–349、385–419 行）。正式实例随后再次选择物理设备（478–486、783–796、867–880 行），设备初始化才走 `Engine/Source/Runtime/VulkanRHI/Private/VulkanDevice.cpp` 的 `CreateDevice` / `vkCreateDevice`（427–437、1429–1436 行）。这是 Vulkan 功能 profile 的准入策略，不是对调用者提交的 `RenderAdapterFacts` 做相等比较；不推导 UE 有同样的错误类型、回退政策或 surface handoff。

## 4. Rust 回归与 UE 已有测试的取舍

| 参考树已有检查 | 原输入与结果 | 对 ZE 的迁移范围 |
|---|---|---|
| Gauntlet `UE.EditorBootTest` | `Engine/Source/Programs/AutomationTool/Gauntlet/Unreal/Automation/UE.BootTest.cs` 的 `EditorBootTest` 选择 Editor/CookedEditor 角色并请求 `QUIT_EDITOR`（197–223 行），沿用日志检查（130–148、161–192 行）；基础 `Gauntlet.UnrealTestNode.cs` 的 `GetExitCodeAndReason` 把未达 engine-init 判作失败（2112–2128 行）。现成 `RunUnreal` 可转发 Editor 角色参数（`Engine/Source/Programs/AutomationTool/Gauntlet/Unreal/RunUnreal.cs` 61–65、247–251 行；`Engine/Source/Programs/AutomationTool/Gauntlet/Unreal/Base/Gauntlet.UnrealTestNode.cs` 793–816 行）。 | 可作为选用受支持 RHI／adapter 的**启动正控参考**，不用为本案拼接一个新 UE 用例。该原测只验启动/进程和日志，不断言 `graphicsadapter=` 最终身份、拒绝错 profile 或请求次数；本调查未运行它。 |
| 未找到同形原测 | 在本地 VulkanRHI、WindowsD3D/D3D12RHI、DeviceProfiles、RHITests 与 Gauntlet 相关入口搜索，未找到“已选择适配器 + 外部 profile 不同 → 调用逻辑设备前返回类型化错误”的原测。 | 负例由 ZE 自身的 public `Result` 与顺序契约推导，不标成 UE 测试移植。 |

参考文件 SHA-256（实际参考字节）：`Build.version` `3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6`；`VulkanRHI.cpp` `931d1489573ab89cc86bbc07a18150c9ed356a806dc82f16011e5604e6b48463`；`VulkanDevice.cpp` `181390bd551ef771f6dbab2f4bf6100185158de00742b00524731c05651421e6`；`UE.BootTest.cs` `8552fae6ab131dba61d16b86a95c2844eb0f0ed4c41bf3a6f259be1dcf9440e7`；`RunUnreal.cs` `e43e4926c59fa812fc4ba0a7555106dbdbdce27f1e790ae0f5fb05029f4ad0e7`；`Gauntlet.UnrealTestNode.cs` `8e21af9de65aa5b7a61a4a613c36b0f68ee8b0ed8394cc93b4994e6f4275d090`。仅记录相对定位与指纹，未附参考源码正文。

## 5. 公开 Rust 接口、产品路由与所有权范围

- [`WgpuRenderDevice::new`](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/device.rs#L84-L107) 的输入已经包含 native device/queue；它先查适配器，再核真实 limits、features、队列拓扑，之后才安装错误监督。[已有真实 context 用例](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/tests/cases.rs#L50-L63)只证明这个后段入口应拒绝错 profile，不能替 bootstrap 证明未请求设备。
- 固定基线检索 tracked Rust 包、测试、示例和文档，未见首方生产代码调用**外部 profile**的两个入口。实际 [Browser Canvas 路由](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/browser/open.rs#L101-L120)使用 [`request_graphics_device_async`](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/backend/backend/render_backend_new_offscreen.rs#L100-L123)，先检查请求回执与 descriptor 的 feature 位，再用协商后的 [实际设备和 limits 生成 profile](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/device_profile.rs#L90-L109)。[Graphics 确实借这个 typed device 构造场景端](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/scene_renderer_construct/new_with_icon_source.rs#L311-L340)，不是借走 `Queue` 或提交权。桌面 [offscreen 设备路径](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/backend/backend/render_backend_new_offscreen.rs#L18-L50)另行选择并协商，不是本入口。不能据此推定已发货的应用、浏览器或普通 resize 已遇到此错序。
- 这是 Rust 对象与 `Result` 的冷启动边界，不是 ECS、编辑器设置迁移或产品 ABI 变更。核对的是完整的 adapter facts 相等性，不是比较指针或只比数值 ID；既有兼容性筛选不等于外部 profile 匹配。

## 6. 最小修正与等价回归门（待执行）

只在外部 profile 入口用 `self.adapter_facts()` 做前置比较，保留带两份 facts 的既有 `NativeContextAdapterMismatch`；通过后才进入共享请求。保留同步包装、真实 native 错误映射、同 generation 所有权、`WgpuRenderDevice::new` 后验校验和 `request_graphics_device_async` 按实际 device 构造 profile。不要为此重排 surface 选择、借用设备/队列语义或 resize；不扩大到全库源码守卫或换行策略。

同一 Draft 后续用正常 Rust 配置做正负对照：[`RenderDeviceProfile::new`](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi/src/device_profile.rs#L549-L570) 能接收独立的 [`RenderAdapterFacts`](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi/src/device_profile.rs#L137-L171)，例如保留协商输入，仅让 profile 里的 adapter 名称来自另一份存档；用相同兼容表面配置和合法原生 descriptor。负例核对 `Err` 的 variant 和两份 facts，并验证该入口未进入真实设备请求边界；正例用独立 one-shot 实例验证匹配 profile 可进入一次正常请求并移交表面。若只断言错误类型，旧代码在 WGPU 请求成功后也能通过，因此需对请求边界做专用、只读观测；不靠延时、假 GPU、故意提交坏 descriptor 或制造驱动故障。只有观察到真实调用顺序后才计入行为配对结果；若设备或 surface 前置未达到，则只记录前置，不能把未运行的回归或旧字符串失败当成修复验收。

## 7. 相同 profile 与不匹配 profile 的请求计数

Windows x64/MSVC 实际编译 `da3e99d63a66c7c1e4243d98b82613bd2994a55c`，包 `zr_rhi_wgpu` 选 `--no-default-features --lib --profile test --locked --offline`。完整测试 EXE 以 `profile_request_ --test-threads 1 --show-output` 运行。固定三个具名用例均被匹配，实际退出 101。

| 外部 profile 与请求结果 | 请求调用 / 首次轮询 | 已验证结果 |
|---|---:|---|
| 与选定适配器不匹配 | 1 / 1 | `NativeContextAdapterMismatch` 中的两份适配器资料逐项相等 |
| 匹配，提供方成功 | 1 / 1 | 设备与队列的观测值以及原 profile 完整交付 |
| 匹配，提供方返回错误 | 1 / 1 | `SurfaceUnavailable` 原错误透传 |

该测试运行外部 profile 入口使用的同一个 Rust 调度函数，以立即可完成的提供方计数。公开入口传入已保留 adapter 的原生请求。Graphics 入口继续以协商设备的实际 limits 构造 profile。真实设备与表面的端到端运行另有前置。

编译回执 `9e2b6a63a277ef2bce83c0bf38d8001a83687d3d7467a08a185015958d0e3d2b`，测试 EXE `d23e93818cc8fc6156d68237a874958dd9776b9a8cbc6993e7a90008152b3bbd`，实际执行回执 `f23e108ac395a2024e46de9fb9590a5933de0fe930c811408d30b27f34612263`。
