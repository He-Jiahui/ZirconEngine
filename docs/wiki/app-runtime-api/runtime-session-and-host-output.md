---
related_code:
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_library/runtime_session/foreign_output.rs
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/drain.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime_interface/src/runtime_api/session/session.rs
  - zircon_runtime_interface/src/runtime_api/session/events.rs
  - zircon_runtime_interface/src/runtime_api/session/requests.rs
  - zircon_runtime_interface/src/runtime_api/host/host_requests.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_host/src/foreign_output/state.rs
  - zircon_runtime_host/src/foreign_output/owned_buffer.rs
implementation_files:
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime_interface/src/runtime_api/session/session.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_host/src/foreign_output/state.rs
  - zircon_runtime_host/src/foreign_output/owned_buffer.rs
plan_sources:
  - user: 2026-09-09 为 ZirconEngine 构建引擎说明书级 Wiki
  - docs/zircon_app/runtime-surface-present.md
  - docs/zircon_runtime/dynamic_api/session.md
tests:
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/dynamic_api/tests/session_profiles.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime_host/src/foreign_output/tests.rs
doc_type: workflow-detail
---

# 运行时会话与宿主输出

## 会话配置 V3

```rust
#[repr(C)]
pub struct ZrRuntimeSessionConfigV3 {
    pub abi_version: u32,
    pub profile: ZrByteSlice,
    pub project_root: ZrByteSlice,
    pub play_scene: ZrByteSlice,
    pub play_report_pipe: ZrByteSlice,
    pub wake_sink: ZrRuntimeWakeSinkV1,
}
```

`profile` 是会话策略名；`project_root` 是物理项目根锚点；`play_scene` 是可选项目相对版本化场景；`play_report_pipe` 是可选逻辑报告出口。`wake_sink` 要么完全禁用（token 0、callback `None`），要么完全注册（非零 token、callback `Some`）。callback 必须快速返回，而且不能在自身回调中同步销毁同一 session。

## 创建会话

生产调用通过 `RuntimeSession::create_with_profile` 完成。概念上的 ABI 调用如下：

```rust
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeSessionConfigV3, ZrRuntimeSessionHandle,
};

fn bytes(value: &str) -> ZrByteSlice {
    ZrByteSlice { data: value.as_ptr(), len: value.len() }
}

unsafe fn create_session(
    api: &zircon_runtime_interface::ZrRuntimeApiV8,
    project: &str,
) -> Result<ZrRuntimeSessionHandle, String> {
    let mut config = ZrRuntimeSessionConfigV3::empty();
    config.profile = bytes("runtime");
    config.project_root = bytes(project);

    let create = api.create_session.ok_or("missing create_session")?;
    let mut handle = ZrRuntimeSessionHandle::invalid();
    let status = unsafe { create(config, &mut handle) };
    if !status.is_ok() {
        return Err(format!("create_session failed: {:?}", status.status_code()));
    }
    Ok(handle)
}
```

示例中的字符串必须在调用返回前保持存活。真实宿主还必须同步复制/限制诊断文本，并在任何失败路径清理可能已创建的 session 或输出。

## 帧循环

窗口宿主的 `about_to_wait` 阶段执行以下工作：

1. 应用 event-loop policy，并轮询可选 gamepad backend。
2. 调用 `tick_frame` 推进 runtime-owned schedule。
3. 读取 `ZrRuntimeFrameDemandV1`，判断是否需要重绘、连续帧或 host wake。
4. 若提供 `drain_host_requests`，排空 runtime 到宿主的反向请求。
5. 对需要显示的窗口请求 redraw；redraw handler 再走 native present 或 frame capture。

输入先被 `zircon_app` 从 winit/gilrs 事件转换为 `ZrRuntimeEventV1`。文本、路径等 payload 是同步 borrowed byte slice，运行时在需要跨调用保存时复制数据。

## 帧输出与 surface

`capture_frame` 返回 `ZrRuntimeFrameV2`，RGBA 尺寸上限为每边 16,384，最大 256 MiB。请求或输出超过任一限制都返回 `LimitExceeded`。

原生 present 能力由 `bind_viewport_surface + unbind_viewport_surface + present_viewport` 三个可选槽共同形成。宿主必须把它们当成一个能力组：完整存在时将 native target 绑定到 runtime；否则可使用 reference CPU presenter。窗口销毁或重建 surface 时必须先解除绑定，不能让 runtime 保留失效的 OS 指针。

## Host Request

`drain_host_requests` 用于 runtime 反向请求宿主能力，当前涵盖 clipboard、IME 和 gamepad rumble 等类别。结果是 JSON 编码的 `ZrOwnedResultV2`。宿主执行请求后，某些结果再作为 `ZrRuntimeEventV1` 回送 runtime。

这条通道是“请求”，不是 runtime 直接调用 OS。这样可以把窗口线程亲和性、权限判断和平台错误留在产品宿主。

## Runtime-owned Output 所有权

`ZrOwnedResultV2` 只包含不可变地址、`u64` 长度和 opaque `ZrRuntimeAllocationId`。它没有 capacity、owner token 或每结果 free callback。

```text
runtime call succeeds
  -> host validates pointer/length/allocation shape
  -> host enforces family byte budget
  -> host performs bounded JSON preflight + typed decode
  -> host validates typed item count
  -> host calls release_allocation(origin_session, allocation_id)
  -> only then accepts the decoded value
```

空结果的规范形状是 null data、len 0、invalid allocation。非空结果必须同时具备非空 data、正长度和有效 allocation id。重复释放、伪造 id、错误 session 释放或并发失败方释放会返回 `NotFound`，不得改变真实 owner 的 census。

会话销毁时若仍有未释放 allocation，会被拒绝；调用者释放剩余结果后可以重试销毁。这是防止动态库卸载后遗留跨 allocator 指针的核心规则。

## 安全解码

`zircon_runtime_host::foreign_output::RuntimeForeignOutputState` 是宿主推荐入口。它按 host request、profile response、operation result、plugin events、world query 和 invalidation 分族统计：

- 编码字节数；
- JSON 结构深度（最高 128）；
- typed item 数；
- 解码时间；
- 调用失败、协议拒绝和阻断次数。

任何指针/所有权形状错误、超预算、非法 JSON、typed 校验失败或释放失败都会触发会话级 protocol fuse。熔断后，后续 foreign-output 和 session 操作被拒绝，防止在已失去信任的 provider 状态上继续运行。

推荐消费模式：

```rust
use zircon_runtime_host::foreign_output::{
    RuntimeForeignOutputKind, RuntimeForeignOutputState,
    RuntimeOwnedOutputReleaser, WORLD_QUERY_OUTPUT_BUDGET,
};
use zircon_runtime_interface::{WorldQueryResult, ZrOwnedResultV2};

unsafe fn decode_world_result(
    state: &RuntimeForeignOutputState,
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
) -> Result<Option<WorldQueryResult>, Box<dyn std::error::Error>> {
    Ok(unsafe {
        state.decode_json(
            output,
            releaser,
            RuntimeForeignOutputKind::WorldQuery,
            WORLD_QUERY_OUTPUT_BUDGET,
            "decode world query",
            "release world query",
            |result: &WorldQueryResult| Ok::<usize, &'static str>(1),
        )
    }?)
}
```

构造 `RuntimeOwnedOutputReleaser` 本身是 `unsafe`：调用者必须证明 release 函数属于拥有该 session 的同一 provider，并保证 provider 和 session 仍存活。

## 当前状态

- 已实现：会话 V3、wake sink、事件与帧循环、输出 allocation registry、预算解码、协议熔断。
- 部分实现：host request 与 native surface 取决于可选 V8 槽和平台能力。
- 规划：不可信 runtime 进程隔离；当前同进程 ABI 无法验证任意数值地址的可读性。
