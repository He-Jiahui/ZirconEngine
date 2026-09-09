---
related_code:
  - zircon_runtime/src/operation/mod.rs
  - zircon_runtime/src/operation/context.rs
  - zircon_runtime/src/operation/handler.rs
  - zircon_runtime/src/operation/service.rs
  - zircon_runtime/src/operation/service/admission.rs
  - zircon_runtime/src/operation/maintenance.rs
  - zircon_runtime_interface/src/runtime_api/session/operation.rs
  - zircon_runtime/src/dynamic_api/session/operation.rs
implementation_files:
  - zircon_runtime/src/operation/service.rs
  - zircon_runtime/src/operation/service/admission.rs
  - zircon_runtime/src/operation/service/completion.rs
  - zircon_runtime/src/operation/maintenance.rs
  - zircon_runtime/src/operation/handler.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/mvp/index.md
  - docs/zircon_runtime/operation.md
tests:
  - zircon_runtime/src/operation/tests.rs
  - zircon_runtime/src/operation/tests/inflight_retention.rs
  - zircon_runtime/src/operation/tests/phase_indexes.rs
  - zircon_runtime/src/operation/tests/raw_admission.rs
  - zircon_runtime/src/dynamic_api/tests/operation.rs
  - zircon_runtime_interface/src/tests/runtime_operation.rs
doc_type: module-detail
---

# Runtime Operation

`zircon_runtime::operation` 为 runtime session 提供有界的异步操作注册表。它适合导入、保存、构建、世界写入和其他不能在一次宿主调用中完成的工作；操作把不可变输入捕获、后台准备和 owner-thread 提交分开，避免 worker 直接持有 `World` 可变引用。动态 ABI 的 `submit_operation`、`poll_operation`、`harvest_operation` 最终都落到同一个服务。

## 三阶段执行模型

```text
submit(request)
    |
    v
Queued -- owner tick --> snapshot(context + payload)
    |
    v
Preparing -- task graph worker --> prepare(snapshot)
    |
    v
ReadyToApply -- owner tick --> apply(context + command)
    |
    +--> Completed / Failed --> harvest(result) --> Harvested
    +--> Cancelled / Expired
```

`RuntimeOperationHandler` 的三个方法拥有明确线程语义：

| 方法 | 执行位置 | 输入/输出 | 约束 |
| --- | --- | --- | --- |
| `snapshot` | runtime owner thread | `RuntimeOperationContext` + JSON payload -> owned JSON snapshot | 可以读取或修改当前 World，但必须在本次调用内完成；不得把借用引用放到 worker |
| `prepare` | Core task graph worker | owned snapshot -> `RuntimeOperationPrepared(command, result)` | 不得访问 World；结果和 command 必须是有界 JSON |
| `apply` | runtime owner thread | `RuntimeOperationContext` + owned command -> `()` | 提交真正的 World 变更；panic 会转换为失败终态 |

`RuntimeOperationContext` 只暴露 `core()`、`world()` 和 `world_mut()`。它的生命周期被限制在 owner 阶段，不能存入 handler 的字段、静态变量或跨线程消息。

## 注册与 admission

一个 operation ID 在同一服务中只能注册一次；空 ID、未知 ID 和重复 handler 都是显式错误。`RuntimeOperationService::submit` 接收 `ZrRuntimeOperationSubmitRequestV1`，先验证 ABI 版本、handler 和 JSON 大小，再分配非零 `ZrRuntimeOperationHandle`。FFI 侧应使用 `submit_json`：它会先按原始字节数预留容量，再解析 JSON，解析失败时释放预留，避免恶意大 payload 绕过预算。

默认 admission 上限如下：

| 上限 | 默认值 | 超限错误 |
| --- | ---: | --- |
| 同时保留的 task | 1024 | `TaskCapacityReached` |
| 后台 prepare 并发数 | 32 | 进入队列，不能无限创建 worker |
| payload/command/result 总字节 | 4 MiB | `RetainedBytesCapacityReached` |
| 每次 owner tick 的 apply 数 | 8 | 延后到后续 tick |
| terminal result 保留时间 | 60 s | 自动转为 `Expired` |

已取消、已收割或已过期的 tombstone 可以在新 admission 时回收；仍有 prepare in-flight 的 task 不会被提前删除。

## Rust 调用接口

运行时模块可以直接使用 `RuntimeOperationService`。下面的 handler 展示最小、可测试的 echo 操作；真实实现应在 `apply` 中调用领域 manager，而不是在 handler 内部构造第二个 World。

```rust
use std::sync::Arc;
use serde_json::{json, Value};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::operation::{
    RuntimeOperationContext, RuntimeOperationHandler, RuntimeOperationHandlerError,
    RuntimeOperationPrepared, RuntimeOperationService,
};
use zircon_runtime::scene::World;
use zircon_runtime_interface::{
    ZrRuntimeOperationPhase, ZrRuntimeOperationSubmitRequestV1,
    ZIRCON_RUNTIME_ABI_VERSION_V1,
};

struct Echo;

impl RuntimeOperationHandler for Echo {
    fn snapshot(
        &self,
        _context: RuntimeOperationContext<'_>,
        payload: Value,
    ) -> Result<Value, RuntimeOperationHandlerError> {
        Ok(payload)
    }

    fn prepare(
        &self,
        snapshot: Value,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError> {
        Ok(RuntimeOperationPrepared::new(
            snapshot.clone(),
            json!({ "echo": snapshot }),
        ))
    }

    fn apply(
        &self,
        _context: RuntimeOperationContext<'_>,
        _command: Value,
    ) -> Result<(), RuntimeOperationHandlerError> {
        Ok(())
    }
}

let mut service = RuntimeOperationService::new();
service.register_handler("demo.echo", Arc::new(Echo))?;
let handle = service.submit(ZrRuntimeOperationSubmitRequestV1::new(
    ZIRCON_RUNTIME_ABI_VERSION_V1,
    "demo.echo",
    json!({ "value": 9 }),
))?;

let runtime = CoreRuntime::try_new()?;
let core = runtime.handle();
let mut world = World::empty();
loop {
    service.tick(&core, &mut world);
    let status = service.poll(handle)?;
    if status.phase().is_some_and(ZrRuntimeOperationPhase::is_terminal) {
        break;
    }
    std::thread::yield_now();
}
let result = service.harvest(handle)?;
assert_eq!(result.succeeded_output().unwrap()["echo"]["value"], 9);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`RuntimeOperationPrepared::new(command, result)` 的两个值都会计入 retained byte budget。`result` 是终态给调用方的输出，`command` 只在 owner apply 阶段使用；不要把未验证的输入直接当作 command。

## 状态、结果与 ABI

`ZrRuntimeOperationPhase` 的完整序列是 `Queued`、`Preparing`、`ReadyToApply`、`Completed`、`Failed`、`Cancelled`、`Expired`、`Harvested`。`ZrRuntimeOperationStatusV2` 是 `#[repr(C)]`、无分配的 poll 快照，调用方用 `phase()` 和 `detail_kind()` 将原始整数转换为枚举。`Completed`、`Failed`、`Cancelled`、`Expired` 和 `Harvested` 都是 terminal，但只有前两者能成功 harvest 结果。

`ZrRuntimeOperationResultV1` 包含 ABI 版本、handle、operation ID 和 `ZrRuntimeOperationOutcomeV1`。成功结果用 `succeeded_output()` 读取 JSON；失败结果用 `failure()` 读取受限错误字符串。动态 ABI 的函数类型为：

| 函数 | C ABI 形状 | 语义 |
| --- | --- | --- |
| `submit_operation` | `(SessionHandle, ZrByteSlice, *mut OperationHandle) -> ZrStatus` | 提交 JSON 请求并返回 opaque handle |
| `poll_operation` | `(SessionHandle, OperationHandle, *mut StatusV2) -> ZrStatus` | 读取阶段，不取走结果 |
| `harvest_operation` | `(SessionHandle, OperationHandle, *mut OwnedResultV2) -> ZrStatus` | terminal 后取走 runtime-owned 结果 |

ABI host 不应直接跨库调用 `RuntimeOperationService`；应使用[动态运行时与 ABI](../app-runtime-api/dynamic-runtime-abi.md)中的表槽和 `zircon_runtime_host` 解码/释放路径。

## Tick、取消与截止时间

服务不会自行在后台修改 World。`tick(core, world)` 才会执行 owner snapshot、收割 worker completion、apply prepared command 和维护 deadline/TTL。宿主应把它放在固定的 runtime 帧阶段，并为每帧设置明确的工作预算。

`cancel(handle)` 只允许在 `Queued`、`Preparing` 或 `ReadyToApply` 且尚未 claim owner apply 时调用；取消会释放 payload 和 prepared bytes，并把 detail 设为 `Cancelled`。`submit_with_deadline(request, Some(instant))` 的截止时间由 owner tick/maintenance timer 强制执行，超时变为 `Expired`，不会调用 handler 的 apply。

终态结果默认只保留 60 秒。调用方应尽快 harvest；TTL 到期后 poll 仍可能看到 `Expired`，但不能再读取结果。重复 harvest 返回 `AlreadyHarvested`，未知或跨 session handle 返回 `UnknownHandle`。

## 错误与 panic 隔离

服务把 handler 的 `snapshot`、`prepare` 和 `apply` panic 都捕获为可 harvest 的失败结果；worker completion channel 丢失会产生 `WorkerChannelLost` detail。常见错误包括：

- `UnsupportedAbiVersion`、`InvalidRequest`、`PayloadEncoding`：修正调用方版本或 JSON，不要原样重试。
- `UnknownOperation`、`DuplicateHandler`、`EmptyOperationId`：修正注册/选择阶段。
- `TaskCapacityReached`、`RetainedBytesCapacityReached`：降低并发或 payload，等待 tombstone/TTL 回收。
- `NotTerminal`、`NotCancellable`、`OperationCancelled`、`OperationExpired`、`AlreadyHarvested`：按 phase 和 owner 生命周期处理，不把它们当作瞬态网络错误。

错误字符串和 JSON 都受 retained byte 上限约束，UTF-8 截断保持有效边界。跨 ABI 的 `ZrStatus` 只传状态码和诊断 slice，宿主必须在 session 存活期间复制或释放 owned output。

## 与编辑器和插件的关系

Editor gateway、动态 runtime 和插件可以把自己的 operation ID 注册到同一服务，但 owner 必须绑定到当前 runtime session/generation。编辑器 command/transaction 负责撤销和作者态语义；runtime operation 负责有界执行和 World owner 提交，二者不能互相替代。插件卸载前应停止新 admission、取消或 harvest 自己的 task，再撤销 handler，防止旧 worker 回调已卸载代码。

## 实现状态

- **已实现**：三阶段 handler、owner-only World 访问、有界 task/byte admission、固定布局 poll、结果 harvest、取消、deadline、terminal TTL 和 panic containment。
- **受限**：操作服务是 runtime session 内部状态；跨动态库使用必须经过 ABI V1/V2 DTO 和 host ownership 规则。
- **受限**：`apply` 每帧有固定上限，长队列会增加 latency；调用方应在状态 UI 中显示 phase/detail，而不是阻塞等待。
- **内部实现**：phase index、maintenance timer、completion receiver 和 tombstone 回收策略。
