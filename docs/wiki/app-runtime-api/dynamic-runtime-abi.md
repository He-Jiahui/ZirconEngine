---
related_code:
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/artifact_manifest.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_shape.rs
  - zircon_runtime_interface/src/runtime_api/abi/host_api_shape.rs
  - zircon_runtime_interface/src/runtime_build_set/mod.rs
  - zircon_runtime_interface/src/version.rs
implementation_files:
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_shape.rs
  - zircon_runtime_interface/src/runtime_api/abi/host_api_shape.rs
plan_sources:
  - user: 2026-09-09 为 ZirconEngine 构建引擎说明书级 Wiki
  - docs/zircon_runtime_interface/runtime_api.md
  - docs/zircon_runtime/dynamic_api/session.md
tests:
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_shape_tests.rs
  - zircon_runtime_interface/src/runtime_api/abi/host_api_shape_tests.rs
  - zircon_runtime/src/dynamic_api/tests/api_table.rs
  - zircon_app/src/entry/runtime_library/tests.rs
doc_type: module-detail
---

# 动态运行时与 ABI V8

## 设计目标

动态边界允许 `zircon_app` 在不静态绑定完整 runtime 实现的情况下加载 `zircon_runtime` 动态库。接口 crate 只传输固定布局记录、数值判别、opaque handle 和函数指针；ECS、GPU、窗口对象、trait object 与 Rust 容器都不能跨边界。

当前入口符号和表版本为：

```rust
// 符号字节串由 runtime_build_set 的冻结 InterfaceSpec 生成。
ZR_RUNTIME_GET_API_SYMBOL_V8 // b"zircon_runtime_get_api_v8\0"
ZIRCON_RUNTIME_API_VERSION_V8 // 8
```

“V3”仅指当前会话配置 `ZrRuntimeSessionConfigV3`。函数表已经硬切到 `ZrRuntimeApiV8`，加载器没有 V7/V6/V3 回退。

## 加载与握手

`LoadedRuntime::load_default()` 的顺序是：

1. 从产品可执行文件相邻位置，或 `ZIRCON_RUNTIME_LIBRARY` 覆盖路径选择动态库。
2. 校验 runtime artifact sidecar、摘要和 BuildSet 身份。
3. 通过 `libloading::Library` 加载动态库。
4. 构造 `ZrHostApiV1`，解析 `zircon_runtime_get_api_v8`。
5. 拒绝空指针、未对齐指针、错误 `abi_version`、非精确 `size_bytes` 和缺失的必需函数。
6. 保存动态库 owner、API 指针和已复制的必需函数指针。只要 `LoadedRuntime` 存活，表存储就保持有效。

预检 receipt 不持有代码。`RuntimeLibraryPreflight::load_after_preflight()` 会在实际加载前再次校验 sidecar 和摘要，并确认 BuildSet 未在 TOCTOU 窗口中变化。

## Host API V1

```rust
#[repr(C)]
pub struct ZrHostApiV1 {
    pub abi_version: u32,
    pub size_bytes: usize,
    pub diagnostics_sink: Option<unsafe extern "C" fn(ZrByteSlice)>,
    pub fetch_resource: Option<ZrRuntimeHostFetchFnV1>,
}
```

两个 callback 都是可选槽位。`null` host pointer 是合法的“无宿主回调”形式。非空 pointer 必须满足对齐、可读和精确表形状；同进程代码无法证明任意地址已映射，因此来自不可信库的指针需要未来的进程隔离边界。

## Runtime API V8 槽位

表含 2 个 header 字段和 26 个函数槽位。`Option<extern fn>` 是 C-compatible 空槽表达，但“可为空”不等于“宿主可忽略”：加载时按下面的能力分区校验。

| 函数组 | 必需槽位 | 用途 |
| --- | --- | --- |
| 生命周期 | `create_session`, `destroy_session`, `release_allocation` | 创建/销毁会话，释放运行时分配 |
| 事件与帧 | `handle_event`, `capture_frame`, `tick_frame` | 输入、离屏 RGBA 捕获、推进帧 |
| 编辑器视口 | `submit_highlight_set`, `request_viewport_pick`, `poll_viewport_pick`, `cancel_viewport_pick` | 高亮与异步拾取 |
| 插件事件 | `subscribe_plugin_event`, `unsubscribe_plugin_event`, `drain_plugin_events` | typed event mirror |
| 长操作 | `submit_operation`, `poll_operation`, `harvest_operation` | 提交、读取进度、取回终态结果 |
| 世界同步 | `query_world`, `watch_world`, `unwatch_world`, `drain_world_invalidations` | runtime world 的只读投影和失效通知 |

可选槽位：

| 槽位 | 缺失时的宿主行为 |
| --- | --- |
| `capture_accessibility_tree` | 不暴露 accessibility tree；不得伪造空树为成功 |
| `bind_viewport_surface`, `unbind_viewport_surface`, `present_viewport` | 三者必须作为完整 native-present 能力使用；否则走 frame capture/fallback presenter |
| `profile_control` | 隐藏或禁用运行时 profiling 控件 |
| `drain_host_requests` | 不执行 clipboard、IME、rumble 等反向请求泵 |

## 逐函数参考

下表中的“输出指针”都必须由调用者提供有效、对齐、可写的存储；实现会在可行时先写入 invalid/empty 值，使错误路径不遗留未初始化数据。

| 槽位 | 核心签名摘要 | 契约 |
| --- | --- | --- |
| `create_session` | `(ConfigV3, *mut SessionHandle) -> Status` | 校验 profile/project/play/wake 配置并创建非零会话句柄 |
| `destroy_session` | `(SessionHandle) -> Status` | 等待活动调用静止；存在未释放 allocation 时拒绝销毁 |
| `release_allocation` | `(SessionHandle, AllocationId) -> Status` | 只释放由该 session 登记的 runtime-owned 输出；一次性 authority |
| `handle_event` | `(SessionHandle, EventV1) -> Status` | 同步接收生命周期、窗口、输入和宿主结果事件 |
| `capture_frame` | `(SessionHandle, FrameRequestV1, *mut FrameV2) -> Status` | 生成受尺寸预算约束的离屏 RGBA 帧 |
| `capture_accessibility_tree` | `(SessionHandle, AccessibilityRequestV1, *mut OwnedResultV2) -> Status` | 可选；返回预算化 JSON accessibility tree |
| `bind_viewport_surface` | `(SessionHandle, BindSurfaceRequestV1) -> Status` | 可选；绑定默认 viewport 的 native surface target |
| `unbind_viewport_surface` | `(SessionHandle, ViewportHandle) -> Status` | 可选；在 surface/窗口销毁前解除绑定 |
| `present_viewport` | `(SessionHandle, FrameRequestV1) -> Status` | 可选；向已绑定 native surface 呈现一帧 |
| `submit_highlight_set` | `(SessionHandle, HighlightSetV1) -> Status` | 提交 generation-qualified entity 高亮集合 |
| `profile_control` | `(SessionHandle, request JSON, *mut OwnedResultV2) -> Status` | 可选；执行 profiling/diagnostics 控制并返回 JSON |
| `tick_frame` | `(SessionHandle, *mut FrameDemandV1) -> Status` | 推进 runtime schedule 并返回下一帧需求 |
| `drain_host_requests` | `(SessionHandle, *mut OwnedResultV2) -> Status` | 可选；事务性取出 clipboard/IME/rumble 等 JSON 请求 |
| `subscribe_plugin_event` | `(SessionHandle, request JSON, *mut SubscriptionHandle) -> Status` | 按 typed filter 建立插件事件订阅 |
| `unsubscribe_plugin_event` | `(SessionHandle, SubscriptionHandle) -> Status` | 撤销订阅并停止未来投递 |
| `drain_plugin_events` | `(SessionHandle, SubscriptionHandle, *mut OwnedResultV2) -> Status` | 事务性取出一页事件 JSON；成功消费后再 commit |
| `submit_operation` | `(SessionHandle, request JSON, *mut OperationHandle) -> Status` | 提交有界长操作并返回 opaque handle |
| `poll_operation` | `(SessionHandle, OperationHandle, *mut OperationStatusV2) -> Status` | 返回固定布局、无分配的阶段/进度快照，不取走终态结果 |
| `harvest_operation` | `(SessionHandle, OperationHandle, *mut OwnedResultV2) -> Status` | 在 terminal 后取回最终结果并结束 handle 所有权 |
| `query_world` | `(SessionHandle, query JSON, *mut OwnedResultV2) -> Status` | 执行 generation-aware 世界只读查询 |
| `watch_world` | `(SessionHandle, registration JSON, *mut WatchToken) -> Status` | 注册 runtime-owned 事实 watch，返回非零 token |
| `unwatch_world` | `(SessionHandle, WatchToken, *mut u8) -> Status` | 撤销 watch，0/1 输出表示是否实际移除 |
| `drain_world_invalidations` | `(SessionHandle, *mut OwnedResultV2) -> Status` | 事务性取出 generation、dirty token 和 world facts |
| `request_viewport_pick` | `(SessionHandle, PickRequestV1, *mut PickTicket) -> Status` | 发起异步 viewport picking |
| `poll_viewport_pick` | `(SessionHandle, PickTicket, *mut PickResultV1) -> Status` | 轮询 pending/ready 结果；ticket 必须属于同一 session |
| `cancel_viewport_pick` | `(SessionHandle, PickTicket) -> Status` | 取消并释放未完成 picking ticket |

`drain_*` 采用 prepare/register/commit 语义：只有输出 allocation 成功建立后才从 runtime 队列提交消费；登记失败会 rollback，避免事件或失效通知静默丢失。

## 直接 ABI 调用骨架

正常产品代码应使用 `zircon_app` 的加载器；下面示例只用于解释 FFI 责任：

```rust
use std::ptr;
use zircon_runtime_interface::{
    ZrHostApiV1, ZrRuntimeApiV8, ZrRuntimeGetApiFnV8,
    ZIRCON_RUNTIME_ABI_VERSION_V1,
};

unsafe fn acquire(get_api: ZrRuntimeGetApiFnV8) -> Result<ZrRuntimeApiV8, String> {
    let host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
    let ptr = unsafe { get_api(&host) };
    if ptr.is_null() {
        return Err("runtime rejected host ABI".into());
    }

    // 实际加载器还会先检查对齐，并逐槽校验所有必需函数。
    let api = unsafe { ptr::read(ptr) };
    zircon_runtime_interface::validate_runtime_api_v8_shape(&api)
        .map_err(|error| error.to_string())?;
    Ok(api)
}
```

不要把这个简化示例用于生产加载：它没有完成 artifact/BuildSet 验证、library lifetime 绑定和 required-slot 检查。

## ABI 设计规则

- 所有跨边界结构使用 `#[repr(C)]`，数值 enum 使用显式底层类型或原始整数。
- handle 和 token 是 opaque 数值，不允许宿主自行构造有效身份。
- borrowed input 的内存只需覆盖同步调用；runtime 需要保留时必须先复制。
- runtime-owned output 必须由原始 session 的 `release_allocation` 释放。
- 函数表增加字段需要新表版本和宿主/提供者协调硬切，不能在 V8 尾部“偷偷扩展”。
- DTO 的 V1/V2/V3 命名独立演进；同一函数表可以引用多个 DTO 版本。
- panic 必须在动态导出内部转换为 `ZrStatusCode::Panic`，不得穿越 ABI。

## 当前状态

- 已实现：V8 精确形状、BuildSet 预检与二次校验、必需槽检查、宿主表 V1 校验。
- 部分实现：可选能力随 runtime build/profile 和平台变化。
- 规划：不可信 native provider 的进程隔离；当前 API 不宣称跨任意引擎发行版二进制兼容。
