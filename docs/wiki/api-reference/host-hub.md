---
related_code:
  - zircon_runtime_host/src/lib.rs
  - zircon_runtime_host/src/foreign_output/mod.rs
  - zircon_runtime_host/src/foreign_output/state.rs
  - zircon_runtime_host/src/foreign_output/owned_buffer.rs
  - zircon_runtime_host/src/viewport_surface.rs
  - zircon_hub/src/lib.rs
  - zircon_hub/src/tauri_app/mod.rs
  - zircon_hub/src/account/mod.rs
  - zircon_hub/src/service/lifecycle.rs
  - zircon_hub/src/service/http/mod.rs
implementation_files:
  - zircon_runtime_host/src/foreign_output/state.rs
  - zircon_runtime_host/src/foreign_output/owned_buffer.rs
  - zircon_runtime_host/src/viewport_surface.rs
  - zircon_hub/src/lib.rs
  - zircon_hub/src/tauri_app/mod.rs
  - zircon_hub/src/account/mod.rs
  - zircon_hub/src/service/lifecycle.rs
plan_sources:
  - user: 2026-09-09 为 ZirconEngine 构建引擎说明书级 Wiki，并完善公开接口、机制案例、最佳实践和教程
  - docs/plans/astra/features/hub/02-local-service-authority.md
tests:
  - zircon_runtime_host/src/foreign_output/tests.rs
  - zircon_runtime_host/src/viewport_surface.rs
  - zircon_hub/tests/service_authority_contract.rs
  - zircon_hub/tests/project_workflow_contract.rs
  - zircon_hub/tests/tauri_react_shell_contract.rs
doc_type: api-reference
---

# Runtime Host 与 Hub API 参考

`zircon_runtime_host` 与 `zircon_hub` 都是产品宿主侧 crate，但职责完全不同。前者是动态 Runtime ABI 的**内存所有权、解码预算和 native viewport 绑定安全层**；后者是桌面 Hub、项目工作流、账户 broker 与本地服务的**操作编排层**。两者目前是 workspace 内部复用边界，公开 Rust 可见性不等于向第三方承诺独立 semver SDK。

本页以当前源码为准，列出可从 crate root 到达的公共模块、调用形状、feature gate、生命周期与失败语义。需要动态 ABI 的表结构和 `ZrOwnedResultV2` wire contract 时，同时阅读 [Runtime Interface](runtime-interface.md)；需要产品会话的完整输出消费流程时阅读 [运行时会话与宿主输出](../app-runtime-api/runtime-session-and-host-output.md)。

## 选择正确的 crate

| 需求 | 首选 API | 不应做的事 |
| --- | --- | --- |
| 消费 Runtime 返回的 owned JSON | `zircon_runtime_host::foreign_output` | 直接把 `ZrOwnedResultV2.data` 转成长期借用或自行猜测 allocator |
| 跟踪 viewport 与 native surface 的 bind/rebind/unbind | `ViewportSurfaceBindings` | 在 ABI 调用期间持有 registry mutex，或在失败 rebind 后忘记恢复旧绑定 |
| 构建、项目、目录、编辑器启动 | `zircon_hub` 的 `desktop` 模块 | 把 `tauri_app` 内部 state 当成普通 Rust facade |
| OIDC 登录、凭据、账户服务请求 | `zircon_hub::account`，`account-broker` feature | 让 WebView 或调用方保存 refresh token |
| 本机组织、catalog、云 blob 服务 | `zircon_hub::service`，`local-service` feature | 绕过 `service::run` 直接暴露数据库或 `BlobStore` |

## Feature 与二进制入口

`zircon_runtime_host` 没有 feature gate，始终公开 `foreign_output` 与 `viewport_surface`。`zircon_hub` 的 root 只在相应 feature 打开时声明模块；因此依赖侧必须把 feature 当作 API 可用性的编译期条件，而不是运行时能力探测。

| `zircon_hub` feature | 打开的 root module / binary | 用途 | 依赖侧建议 |
| --- | --- | --- | --- |
| `desktop`，默认 | `assets`、`build`、`engines`、`error`、`learn`、`plugins`、`process`、`projects`、`settings`、`state`、`tauri_app`、`team`；`zircon_hub` binary；root re-export `HubError` | Tauri 桌面 Hub | 桌面产品或 Hub 前端集成 |
| `account-broker` | `account` | OIDC、Windows credential store、账户 operation journal、包安装 | 只在可信桌面进程启用 |
| `local-service` | `service`；`zircon_hub_service` binary | loopback HTTP、SQLite、OIDC bearer 校验、catalog 与加密云存储 | 单独的本地服务进程 |

`desktop` 本身会启用 `account-broker`，但不会启用 `local-service`。推荐依赖声明如下：

```toml
# 只使用纯宿主 ABI 安全适配器。
[dependencies]
zircon_runtime_host = { path = "../zircon_runtime_host" }

# 桌面 Hub。明确关闭默认值后再选择 desktop，可避免组合漂移。
zircon_hub = { path = "../zircon_hub", default-features = false, features = ["desktop"] }

# 服务进程不要隐式带入 Tauri/desktop。
# zircon_hub = { path = "../zircon_hub", default-features = false, features = ["local-service"] }
```

构建时可用下列命令验证 feature 表面。`account-broker` 和 `local-service` 依赖网络、平台凭据或服务端依赖；文档引用它们不表示任意平台都可直接运行完整登录流程。

```powershell
cargo check -p zircon_runtime_host
cargo check -p zircon_hub --no-default-features --features desktop
cargo check -p zircon_hub --no-default-features --features account-broker
cargo check -p zircon_hub --no-default-features --features local-service
```

## `zircon_runtime_host`

### 模块总表

| 模块 | 公开入口 | 解决的问题 | 关键不变量 |
| --- | --- | --- | --- |
| `foreign_output` | `RuntimeForeignOutputState`、`RuntimeOwnedOutputReleaser`、预算与 item counter | 把 Runtime 拥有的 `ZrOwnedResultV2` 安全地转为宿主值 | 同一 provider/session 的 output 必须恰好释放一次；协议违规熔断整个 session |
| `viewport_surface` | `ViewportSurfaceBindings` 与两种 operation token | 把 ABI bind/unbind 调用与宿主已发布绑定状态同步 | 相同 viewport 同时最多一个 transition；丢弃未完成 token 自动回滚 |

### `foreign_output`：所有权与受限解码

#### 公共符号索引

| 符号 | 调用形状 | 作用 |
| --- | --- | --- |
| `RuntimeForeignOutputBudget` | `new(max_bytes, max_items, max_decode_time)`、`from_interface(limit)`、`allow_empty()`、`max_decode_time()`、`validate_decode_duration(elapsed, operation)` | 输出族的字节、typed item、总解码时间与空值策略；字段本身不公开，调用方不能直接修改已创建 budget |
| `RuntimeForeignOutputKind` | `SessionProtocol`、`HostRequests`、`ProfileResponse`、`OperationResult`、`PluginEvents`、`WorldQuery`、`WorldInvalidations`；`label()` | metrics 与熔断诊断中的稳定分类标签 |
| `RuntimeOwnedOutputReleaser` | `unsafe { RuntimeOwnedOutputReleaser::new(session, release_fn) }`、`session()` | 把唯一 session handle 与其原始 release callback 绑定；`session()` 只读返回 handle，不暴露 callback |
| `RuntimeForeignOutputState` | `Default::default()`、`ensure_available`、`is_protocol_failed`、`ensure_session_available`、`reject_protocol`、`ensure_call_succeeded`、`decode_json`、`metrics`、`diagnostic_line` | 一次性协议熔断、并发 acceptance gate 与输出统计 |
| `RuntimeForeignOutputError` / `RuntimeForeignOutputErrorKind` | `runtime_call(message)`、`protocol_violation(message)`、`.kind()`、`.with_cleanup_failure(&error)` | 可显示且实现 `std::error::Error` 的稳定错误分类；状态机中的 cleanup/协议故障会升级为 protocol violation |
| `validate_owned_result` | `validate_owned_result(&output, operation)` | 验证 `data/len/allocation` 的结构形状，不解码 payload |
| `release_owned_result` | `unsafe { release_owned_result(output, releaser, operation) }` | 经拥有者 Runtime 回调释放 allocation |
| `release_owned_result_after_error` / `release_owned_result_after_result` | `unsafe { ... }` | 在已经得到业务 `Err` 或 `Result<T, RuntimeForeignOutputError>` 后完成唯一 release；cleanup 失败会合并到最终错误 |
| `validate_owned_result_releasing_on_error` | `unsafe { validate_owned_result_releasing_on_error(output, releaser, operation, release_operation) }` | 验证失败时仍执行 release，成功时原样交还 `ZrOwnedResultV2` 的所有权 |
| `*_item_count` | 例如 `world_query_item_count(&result)` | 将解码后的具体 DTO 映射到 budget 的 item 数 |
| `*_OUTPUT_BUDGET` | 例如 `WORLD_QUERY_OUTPUT_BUDGET` | 与 `zircon_runtime_interface` 版本化 payload limit 同步的默认策略 |

`RuntimeForeignOutputMetricsSnapshot::for_kind(kind)` 返回指定输出族的 `RuntimeForeignOutputMetrics` 副本；`RuntimeForeignOutputKind::label()` 返回用于诊断/日志的 snake-case 标签。`reject_protocol(kind, error)` 是显式熔断入口，适合宿主在发现非 JSON 的 ABI 违反时记录统一错误；它总是返回 `Err`，不会尝试伪造成功值。

`RuntimeForeignOutputMetrics` 的字段均为公开计数：`accepted_payloads`、`accepted_bytes`、`rejected_payloads`、`rejected_bytes`、`call_failures`、`blocked_calls`、`total_decode_nanoseconds`、`max_decode_nanoseconds`。通过 `RuntimeForeignOutputMetricsSnapshot::for_kind(kind)` 按输出族读取；snapshot 自身还公开 `protocol_failed`、`protocol_failures` 和 `blocked_session_calls`。

`RuntimeForeignOutputBudget::from_interface` 复制 `ZrRuntimePayloadLimitV1` 的 encoded-byte、item、processing-time 和 empty 策略；`validate_decode_duration` 是少数可独立调用的预算检查，超过时间即返回 `ProtocolViolation`。`RuntimeForeignOutputError::runtime_call` 适合已知 Runtime 业务失败，`protocol_violation` 适合所有权/解码/边界错误；两者的原始 message 通过 `Display` 输出，调用方应以 `.kind()` 决定能否继续当前 session，而不是解析文案。通用 `release_owned_result_after_*` helper 会合并 cleanup 错误；`RuntimeForeignOutputState` 的 acceptance path 则把 cleanup failure 明确记录为 protocol violation 并熔断。

item counter 与 DTO 的对应关系固定如下：`json_value_item_count` 递归计算 JSON array/object 图；`operation_result_item_count` 计成功 output 图并包含结果 envelope；`plugin_event_batch_item_count` 计 `deliveries.len()`；`profile_control_response_item_count` 计 files、snapshot、diagnostic、hotspot 等结构；`world_query_item_count` 按 query variant 计 row/field/JSON value；`world_invalidation_item_count` 计 dirty、facts 和 batch envelope。将错误的 counter 传给 `decode_json` 会把本应限制的 payload 误视为安全，不能用一个通用常数代替它们。

#### 输出消费生命周期

```text
ABI call 返回 (ZrStatus, ZrOwnedResultV2)
  -> state.ensure_session_available(operation)
  -> state.ensure_call_succeeded(status, output, releaser, kind, ...)
       失败状态：仍释放 output；普通 Runtime call error 不自动熔断
  -> state.decode_json(output, releaser, kind, budget, ...)
       验证 data/len/allocation
       -> 限制编码长度
       -> JSON 深度/图规模预检
       -> 限时 typed decode
       -> typed item 计数校验
       -> release_allocation
       -> acceptance gate 发布值
  -> 成功值或 RuntimeForeignOutputError
```

非空结果必须有非空 `data`、正 `len` 和有效 `ZrRuntimeAllocationId`；空结果只有在 `data == null`、`len == 0`、allocation 无效时才是规范形状。`validate_owned_result` 会拒绝 address-space 无法容纳的长度、超 `isize::MAX` 的 slice 长度、非空空结果、缺 allocation 的 owned storage 等错误。

`RuntimeOwnedOutputReleaser::new` 是 `unsafe`，因为 Rust 类型系统无法证明传入的函数指针属于 output 的 provider。调用方必须同时保证：

1. `session` 与 `release` 来源于产生 `output` 的同一个仍加载 provider；
2. 在 `decode_json` 或 `release_owned_result` 返回前，provider、session、函数指针和 output 所指内存均仍存活；
3. release authority 已唯一转交给该调用，不会在其它路径重复释放；
4. 失败 status diagnostics 在同步读取期间仍有效。

#### 真实调用形状：成功状态后解码 world query

下面函数刻意把 ABI call 放在外层：`RuntimeForeignOutputState` 不知道哪一个 V8 slot 产生 output，只负责在 output 已返回后执行安全协议。`WorldQueryResult` 和 `world_query_item_count` 都是当前公开接口。

```rust
use zircon_runtime_host::foreign_output::{
    world_query_item_count, RuntimeForeignOutputKind, RuntimeForeignOutputState,
    RuntimeOwnedOutputReleaser, WORLD_QUERY_OUTPUT_BUDGET,
};
use zircon_runtime_interface::{world_sync::WorldQueryResult, ZrOwnedResultV2, ZrStatus};

unsafe fn consume_query_result(
    state: &RuntimeForeignOutputState,
    status: ZrStatus,
    output: ZrOwnedResultV2,
    releaser: RuntimeOwnedOutputReleaser,
) -> Result<Option<WorldQueryResult>, Box<dyn std::error::Error>> {
    state.ensure_session_available("query_world")?;

    let output = unsafe {
        state.ensure_call_succeeded(
            status,
            output,
            releaser,
            RuntimeForeignOutputKind::WorldQuery,
            "query_world",
            "release query_world output",
        )?
    };

    Ok(unsafe {
        state.decode_json(
            output,
            releaser,
            RuntimeForeignOutputKind::WorldQuery,
            WORLD_QUERY_OUTPUT_BUDGET,
            "decode world query",
            "release world query output",
            |result: &WorldQueryResult| Ok::<usize, &'static str>(world_query_item_count(result)),
        )?
    })
}
```

`ensure_call_succeeded` 在 Runtime 返回非 OK status 时会尝试释放 output。若 output 结构和 release 都正常，错误种类是 `RuntimeCall`，session 可以继续；若 output 所有权本身异常或清理失败，错误升级为 `ProtocolViolation` 并熔断 session。调用方不得在它返回错误后再次 release 同一 output。

#### 预算与空结果策略

这些常量是调用正确 slot 时应使用的默认策略，不是可由 UI 放宽的建议值。

| 常量 | 典型 payload | 最大 encoded bytes | 最大 typed items | 最大 decode time | 是否允许规范空结果 |
| --- | --- | ---: | ---: | ---: | --- |
| `HOST_REQUEST_OUTPUT_BUDGET` | host request batch | 256 KiB | 256 | 10 ms | 是 |
| `PROFILE_RESPONSE_OUTPUT_BUDGET` | profile control response | 16 MiB | 65,536 | 250 ms | 是 |
| `OPERATION_RESULT_OUTPUT_BUDGET` | operation result | 1 MiB | 16,384 | 25 ms | 否 |
| `PLUGIN_EVENT_OUTPUT_BUDGET` | plugin event page | 256 KiB | 64 deliveries | 10 ms | 是 |
| `WORLD_QUERY_OUTPUT_BUDGET` | world query result | 1 MiB | 16,384 | 25 ms | 否 |
| `WORLD_INVALIDATION_OUTPUT_BUDGET` | invalidation batch | 1 MiB | 16,384 | 25 ms | 是 |

所有 JSON 输出还受 `RUNTIME_FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH` 约束，当前值来自 interface V1 的 128 层。profile response 额外预检 `files` 数量；这避免业务反序列化器在明显过大或过深的输入上分配。

#### 熔断、指标与恢复

`RuntimeForeignOutputErrorKind` 只有两类：

| kind | 表示 | 下一步 |
| --- | --- | --- |
| `RuntimeCall` | Runtime slot 返回非 OK status，且 output 清理协议仍完整 | 根据具体 `ZrStatus` 决定重试、降级或向用户报告 |
| `ProtocolViolation` | 形状、budget、JSON、typed validation、cleanup 或显式 `reject_protocol` 不可信 | 停止该 session 的 foreign-output 与普通 session 调用，执行有序 teardown 并新建 session |

第一次 protocol violation 将 `RuntimeForeignOutputState` 熔断。之后 `ensure_available(kind)`、`ensure_session_available(operation)` 和 `decode_json` 都会拒绝继续接受数据，并把 blocked counter 计入 metrics。并发情况下 acceptance gate 确保“另一个线程在同一时刻成功解码”的值不会在已经熔断后被发布。

```rust
let snapshot = state.metrics();
let query = snapshot.for_kind(RuntimeForeignOutputKind::WorldQuery);

eprintln!(
    "fused={} query_rejected={} query_max_decode_ns={}",
    snapshot.protocol_failed,
    query.rejected_payloads,
    query.max_decode_nanoseconds,
);
if let Some(line) = state.diagnostic_line() {
    eprintln!("runtime foreign-output: {line}");
}
```

恢复不是清除本地 atomic flag。应停止投递新工作、释放所有仍由宿主持有的 Runtime allocations、解除 viewport surface、等待 callback quiescence、销毁 session，最后在可信 provider 上重新创建 session。详见 [兼容性、错误处理与排错](../app-runtime-api/compatibility-and-errors.md)。

### `viewport_surface`：bind/rebind/unbind 的宿主事务

`ViewportSurfaceBindings` 不调用 Runtime ABI；它为 ABI 调用前后提供一个线程安全的发布状态机。每个 `ZrRuntimeViewportHandle` 的状态是未记录、`Binding`、`Bound` 或 `Releasing`。正在 transition 的 viewport 会拒绝并发 bind、rebind 与 release，并返回 `ViewportSurfaceOperationInFlight`，可用 `.viewport()` 找到冲突 handle。

| API | 返回值 | 语义 |
| --- | --- | --- |
| `ViewportSurfaceBindings::begin_binding(viewport)` | `Result<ViewportSurfaceBindingOperation, ViewportSurfaceOperationInFlight>` | 预留第一次 bind 或 rebind；rebind 会记住旧的 bound 状态 |
| `ViewportSurfaceBindings::begin_release(viewport)` | `Result<Option<ViewportSurfaceReleaseOperation>, ViewportSurfaceOperationInFlight>` | 未绑定时 `Ok(None)`，无需发 ABI unbind；已绑定时预留 release |
| `ViewportSurfaceBindings::bound_viewports()` | `Vec<ZrRuntimeViewportHandle>` | 返回当前已发布绑定，按 `handle.raw()` 升序，适合确定性 teardown |
| `ViewportSurfaceBindingOperation::finish(succeeded)` | `bool` | 成功 bind 或失败 rebind 后保留 `Bound`；首次 bind 失败则移除；返回是否仍有任一 bound viewport |
| `ViewportSurfaceReleaseOperation::finish(succeeded)` | `bool` | release 成功则移除；失败则恢复 `Bound`；返回是否仍有任一 bound viewport |

两个 operation token 都带 `#[must_use]`。若在调用 `.finish(...)` 前 drop，`Drop` 实现会恢复 transition 前的已发布状态。这使宿主在 `?` 提前返回、panic unwind 或 FFI 调用错误时不会把 registry 永久卡在 `Binding`/`Releasing`。

#### 真实调用形状：把 token 包在 ABI 调用周围

下面的 `bind_runtime_surface`/`unbind_runtime_surface` 是调用方已有的 ABI 封装，故意没有伪造 V8 函数签名。只有 binding registry 的调用是此 crate 的实际公开 API。

```rust
use zircon_runtime_host::viewport_surface::ViewportSurfaceBindings;
use zircon_runtime_interface::ZrRuntimeViewportHandle;

fn rebind_surface(
    bindings: &ViewportSurfaceBindings,
    viewport: ZrRuntimeViewportHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    let operation = bindings.begin_binding(viewport)?;

    // 这里调用产品已选定的 bind_viewport_surface ABI 封装；不要持有 registry mutex。
    let succeeded = bind_runtime_surface(viewport).is_ok();
    let any_bound = operation.finish(succeeded);

    if !succeeded {
        // 对已有 surface 的失败 rebind，registry 保留旧 Bound 状态供下一次重试。
        return Err("runtime rejected viewport surface binding".into());
    }
    if !any_bound {
        return Err("a successful bind must leave one viewport bound".into());
    }
    Ok(())
}

fn teardown_surfaces(bindings: &ViewportSurfaceBindings) {
    for viewport in bindings.bound_viewports() {
        let Ok(Some(operation)) = bindings.begin_release(viewport) else {
            continue;
        };
        let succeeded = unbind_runtime_surface(viewport).is_ok();
        let _any_bound = operation.finish(succeeded);
    }
}

# fn bind_runtime_surface(_: ZrRuntimeViewportHandle) -> Result<(), ()> { Ok(()) }
# fn unbind_runtime_surface(_: ZrRuntimeViewportHandle) -> Result<(), ()> { Ok(()) }
```

推荐时序是：窗口/surface 即将重建时 release 旧 native target；创建可用 native target 后 reserve bind；执行 ABI bind；以真实 ABI 结果调用 `finish`；只有 registry 已发布为 bound 后才请求 present。`finish(false)` 不是失败重试本身，它只恢复可重试状态。

不要用该 registry 代替 Runtime 的 viewport 生命周期：Runtime 销毁 viewport 前仍必须遵守 ABI 的 unbind/destroy 顺序。原生 surface optional slots 需以完整 capability group 探测，详细规则见 [窗口与 Surface 生命周期](../app-runtime-api/reference/app-window-surface-lifecycle.md)。

## `zircon_hub` 的 desktop API

### Root 与错误模型

在 `desktop` feature 下，crate root 唯一的根 re-export 是 `zircon_hub::HubError`。其余接口通过模块路径访问，例如 `zircon_hub::projects::RecentProject`。`HubError` 可来自 I/O、TOML/JSON、项目 DTO、Tauri、后台队列和结构化状态消息。

```rust
use zircon_hub::{projects::validate_project_root, HubError};

fn validate(path: &std::path::Path) -> Result<(), HubError> {
    match validate_project_root(path) {
        zircon_hub::projects::ProjectValidation::Valid => Ok(()),
        other => Err(HubError::message(format!("project is not launchable: {other:?}"))),
    }
}
```

`HubError::status(detail, recovery)` 适合能转换为 `HubMessage` 的 UI 失败；`into_status_messages()` 保留 detail/recovery 结构。不要先 `.to_string()` 再把文本解析回错误类型。

### Desktop 模块目录

| 模块 | 主要 public API | 生命周期与边界 |
| --- | --- | --- |
| `assets` | `discover_asset_catalog`、`discover_asset_catalog_for_scope`、`AssetCatalogEntry` | 扫描项目 `Assets/assets` 与 engine assets；跳过 `.git`/`target`，保留最多 256 项 |
| `build` | `BuildCommandOptions`、`BuildCommand`、`run_build_command`、`BuildExecutionReport` | 生成参数数组并由受监督子进程运行；结果是 `TaskExecutionOutcome`，不是单纯 exit code |
| `engines` | `SourceEngineInstall`、`SourceBuildRecord`、校验/注册 helper | Source checkout、staged output 和 project binding 分开维护 |
| `learn` | `discover_learn_catalog`、`discover_learn_catalog_for_scope`、`LearnCatalogEntry` | 扫描 Markdown 文档，selected project 优先，最多 128 项 |
| `plugins` | `discover_plugin_catalog`、`discover_plugin_catalog_with_project_roots`、`PluginCatalogEntry` | 扫描 engine `zircon_plugins` 与项目 `Plugins/plugins` manifest |
| `process` | `EditorLaunchRequest`、`EditorLaunchCommand`、folder/open command | 返回安全的 program + args；不要拼 shell 字符串 |
| `projects` | 创建、验证、最近项目、共享历史、打包、设备安装、回收站 | package/device 等复制操作接受 `TaskCancellationToken` 并以 `TaskExecutionOutcome` 表示取消；回收站删除是同步调用 |
| `settings` | `HubConfig`、`HubSettings`、`BuildProfile`、默认路径 helper | Hub 持久化 config 原子写入；`repair_registries` 是加载后修复入口 |
| `state` | `HubSnapshot`、`HubScope`、`TaskStatus`、`HubMessage`、action history | UI 的投影模型；并非任意后台 worker 可直接修改的全局状态 |
| `tauri_app` | `run()` | 组装 Tauri state 与已注册 command；内部 command DTO 不构成独立库 SDK |
| `team` | `discover_team_overview`、`TeamOverview`、`TeamMemberEntry` | 从第一个有效 Git 仓库读取身份与最近 author 汇总 |

#### 桌面公开符号索引

下面的索引按 `zircon_hub` crate root 的 re-export 逐项列出可供 desktop consumer 使用的构造器、函数和状态方法。除特别标注外，方法不会启动后台任务；需要取消或子进程监督的调用仍由上层 action runner 负责。

##### `engines`：Source checkout 注册

| 符号 | 调用形状与行为 |
| --- | --- |
| `SourceBuildRecord` | 持久化 `finished_unix_ms`、`status`、`profile`、可选 `jobs`、`output_dir`、结构化 `detail`/`log_excerpt` 和 `command_line`；`SourceEngineInstall::record_build` 只保留最新 8 条记录 |
| `SourceEngineInstall` | 字段为 `id`、`display_name`、`source_dir`、`output_dir`、`last_build_unix_ms`、`build_history`；可 serde 持久化 |
| `SourceEngineInstall::staged_engine_dir()` | 返回 `output_dir/ZirconEngine`，只计算路径，不检查或创建目录 |
| `SourceEngineInstall::record_build(record)` | 将记录插入历史头部；只有 `status == "success"` 才更新 `last_build_unix_ms` |
| `active_source_engine(engines, active_engine_id)` / `_mut` | 按 ID 取 active；ID 缺失或无效时回退到 slice 第一项，空 slice 返回 `None` |
| `ensure_active_source_engine(engines, &mut active_engine_id)` | 清理不存在的 active ID，并在有记录时选第一项；不改变 engine 列表 |
| `upsert_source_engine(&mut Vec<SourceEngineInstall>, engine)` | 以 `engine.id` 替换现有记录，否则追加；不会自动验证 source checkout |
| `prune_project_engine_bindings(&mut ProjectMetadataMap, engines) -> usize` | 清掉指向已删除 engine 的 metadata binding；返回被修改/移除的条目数 |
| `remove_source_engine(&mut Vec<_>, &mut Option<String>, id) -> Option<SourceEngineInstall>` | 删除指定记录并修复 active selection；找不到 ID 返回 `None` |
| `source_engine_id(&Path) -> String` | 对规范化路径计算稳定 `source-<fnv64>` ID；同一路径不同分隔符得到同一 ID |
| `same_source_engine_path(left, right) -> bool` | 使用与 project metadata 相同的 canonical/key 规则比较路径 |
| `source_engine_display_name(&Path) -> String` | 目录名非空时返回 `<name> Source`，否则返回 `Local Source` |
| `validate_source_engine(path) -> SourceEngineValidation` | 依次检查目录、workspace `Cargo.toml`、`zircon_runtime` member 和 `tools/zircon_build.py` |
| `SourceEngineValidation::{summary, recovery_hint}` | 将 `Valid`、`MissingRoot`、`MissingWorkspaceManifest`、`MissingRuntimeWorkspaceMember`、`MissingBuildTool` 转为稳定 UI 文案/恢复提示 |

##### `process`：程序、编辑器与目录边界

| 符号 | 调用形状与行为 |
| --- | --- |
| `EditorLaunchRequest::open_project(path)` | 生成带 Hub source/profile 和单调 operation ID 的 open-existing `ProjectLaunchIntent` |
| `EditorLaunchRequest::create_project(request)` | 将 `CreateProjectRequest` 转成 create-project intent；请求字段校验错误返回 `HubError` |
| `EditorLaunchRequest::intent()` | 借用内部 typed `ProjectLaunchIntent`，不返回可变引用 |
| `EditorLaunchCommand::new(executable, request)` | 生成 `["--project-launch-intent", <JSON>]` 参数数组；不会 shell-quote 或执行进程 |
| `EditorLaunchCommand::from_staged_engine(engine_root, request)` | 选择 `<engine_root>/zircon_editor(.exe)` 后调用 `new` |
| `EditorLaunchCommand::command_line()` | 返回 program 加 args 的展示/审计数组；调用方仍应使用 `Command::new(program).args(args)` |
| `EditorLaunchCommand::with_hub_handshake(session)` | 追加 `--hub-session <uuid> --hub-protocol 1`；不改变 intent JSON |
| `staged_editor_executable(root)` / `_exists(root)` | 计算 staged editor 路径，后者只做 `is_file()` 检查 |
| `FolderPickerRequest::new(title, initial_dir)` | 创建 Windows folder picker 请求；title/initial_dir 仅在执行时传给 native dialog |
| `pick_folder(&request)` | Windows 使用 `powershell -NoProfile -STA` 的 `FolderBrowserDialog`；取消返回 `Ok(None)`，非 Windows 返回明确 `HubError` |
| `OpenFolderCommand::new(path)` / `command_line()` | 按平台选择 `explorer`、`open` 或 `xdg-open`，并保留 program/args 分离 |
| `open_folder(&command) -> Result<Child, HubError>` | 启动已构造的 program + args；不会接受任意 shell 字符串 |

##### `projects`：项目、历史与交付

| 符号 | 调用形状与行为 |
| --- | --- |
| `project_cover_path(project_root) -> Option<PathBuf>` | 在项目封面候选路径中返回首个存在文件；不存在时为 `None` |
| `CreateProjectRequest::{new, validate_launch_fields, target_root}` | 创建值对象、验证名称/非空 location、计算 `location/project_name`；不创建目录。绝对路径要求由 Tauri action 层追加 |
| `project_template_catalog() -> &'static [ProjectTemplateInfo]` | 返回当前模板目录（含 disabled 预留项）；UI 应读取 `enabled`，不要硬编码模板列表 |
| `enabled_project_template_id(id) -> Option<ProjectTemplateId>` | 只接受 canonical spelling 且 `enabled == true` 的模板；当前仅 `renderable-empty` |
| `validate_project_root(path) -> ProjectValidation` | 返回 `Valid`、`MissingRoot`、`MissingManifest` 或 `InvalidManifest`；只读，不修复 manifest |
| `RecentProject::{from_summary, from_project_path, with_now, refresh_summary, display_name}` | 从共享 `ProjectManifestSummary` 建立/刷新历史项；`with_now` 使用当前 Unix ms，显示名始终来自 manifest |
| `now_unix_ms() -> u64` / `RECENT_PROJECT_LIMIT` | 提供饱和的 Unix 毫秒时钟和 interface v1 历史上限 |
| `ProjectMetadata::{is_empty}` 与 `ProjectMetadataMap` | metadata 字段为 `pinned`、`engine_id`、`last_selected_template`；空值可被 prune |
| `project_metadata_key` / `project_filesystem_path_key` / `normalize_project_root` / `project_paths_match` | 分别生成持久化 key、canonical filesystem key、规范化路径和跨平台路径相等判断 |
| `metadata_for_path` / `metadata_for_path_mut` / `prune_empty_metadata` | 以同一 key 读、创建或清理 metadata；不要手写 key 字符串 |
| `ProjectPackageRequest::new` / `package_project` | 构造 project/output 根并复制文件；跳过 `.git`/`target`，返回 `TaskExecutionOutcome<ProjectPackageReport>` |
| `DeviceInstallRequest::new` / `install_package_to_device` | 将 package 复制到 device root 并生成 install receipt；禁止 device root 位于 package 内 |
| `DeviceInstallReceipt`、`DeviceInstallFileReceipt`、`HubContentDownloadManifest`、`HubContentDownloadChunk` | 公开 serde/serialize receipt DTO；包含文件相对路径、字节数、SHA-256、可恢复 range 信息和稳定 download/resource ID |
| `RecycleDeleteCommand::for_project` / `recycle_delete_project` | 构造并执行 Windows Recycle Bin 删除；不是递归永久删除，当前非 Windows 返回错误 |
| `SharedRecentProjectsSnapshot::{revision, projects, into_projects}` | 读取共享 registry 的 revision 和只读项目 slice；`into_projects` 转移 Vec 所有权 |
| `load_shared_recent_projects[_snapshot]` | 从版本化 Hub/Editor registry 读取 projection；损坏或超限由 `SharedRecentProjectsError` 表达为空投影语义 |
| `merge_recent_project_entries(left, right)` | 使用共享 v1 DTO 规则去重、规范化并排序两组历史 |
| `reconcile_shared_recent_projects[_snapshot]` | 以 revision/CAS 合并 Hub 变化；最多 4 次冲突重试，避免陈旧 Hub 复活 Editor 删除项 |

##### `settings` 与 `state`：持久化投影和任务状态

| 符号 | 调用形状与行为 |
| --- | --- |
| `default_hub_config_path`、`default_project_dir`、`default_source_dir`、`default_build_output_dir`、`default_device_install_dir` | 返回平台默认路径；调用方可覆盖，不应把返回值当作已创建目录 |
| `HubConfig::{load, save, repair_registries}` | 读取 TOML（不存在时 default）、以临时文件 + replace 原子保存、修复 recent/metadata/history/active engine；`HubConfigRepairReport::repaired_anything` 汇总是否发生修复 |
| `HubRuntimeState::normalize` | 清理空 selection/engine、trim draft name、补默认 template/location；不触碰持久化文件 |
| `BuildProfile::{as_mode, from_ui_value}` | `Debug`/`Release` 与 `debug`/`release` 的唯一转换；未知 UI 值为 `None` |
| `HubLanguage::{as_ui_value, from_ui_value}` | English/Chinese 与 `en`/`zh`/`cn` 输入的转换；未知值为 `None` |
| `HubSnapshot::{scope, filtered_recent_projects}` | 从 snapshot 计算统一 `HubScope`，按 filter/search/sort 返回 UI 历史副本 |
| `HubScope::{resolve, selected_project, selected_or_latest_project, has_stale_selected_project}` | 先解析 selected project，再决定 project-bound/active engine；过时 selection 不会静默回退 |
| `ProjectScopeProject::can_build` / `SourceEngineScope::engine_id` | 分别判断 engine state 是否 `Ready`、读取当前 scope engine ID |
| `HubPage::id/from_id`、`ProjectFilterMode::{id,label,next,from_id}`、`ProjectSortMode::{id,label,next,from_id}`、`ProjectViewMode::{id,from_id}`、`ProjectSubpage::{id,from_id}` | UI enum 的稳定字符串投影；`from_id` 对未知值返回 `None`，`next` 循环到下一项 |
| `TaskCancellationToken::{new, task_id, request_cancellation, is_cancellation_requested}` | clone 共享同一 cancellation flag；task ID 应非零，取消是 cooperative，不强杀线程 |
| `TaskExecutionOutcome<T>` | 只有 `Completed(T)` 与 `Cancelled` 两个终态；不能把取消映射为成功结果 |
| `TaskStatus::{idle, running, running_operation, success, warning, cancelled, error}` | 建立结构化任务状态；再用 `with_task_id`、`with_cancellable`、`with_progress_percent`、`set_progress_percent`、`with_operation` 和 `operation_summary` 补全投影 |
| `TaskSeverity` / `TaskOperationKind` / `TASK_PROGRESS_*` | 稳定严重性、操作域和 0/10/35/100 进度常量 |
| `HubMessage::{new, with_params, raw_text, empty, is_empty, contains, render, render_with_recovery}` | 以 `HubMessageId` 和参数保存本地化 detail/recovery；UI 应在渲染时选择 `HubLanguage` |
| `HubMessageId::{as_str, from_str_id, param_count, template, all}` 与各 `*MessageId::ALL` | 稳定 message ID/模板目录；业务分支应匹配 ID，不解析渲染后的文案 |
| `HubActionKind::{id,label}` / `HubActionStatus::{label,succeeded}` / `push_action_record` | action history 的稳定投影；push 时插入头部并裁剪到 `ACTION_HISTORY_LIMIT`（当前 16） |

##### `assets`、`learn`、`plugins` 与 `team`

| 符号 | 调用形状与行为 |
| --- | --- |
| `discover_asset_catalog(project_roots, repo_roots)` / `_for_scope(selected, project_roots, repo_roots)` | 扫描项目 `Assets/assets` 与 engine `zircon_editor/assets`、`zircon_runtime/assets`；按 selected/project/engine 优先级排序并截取 256 项 |
| `discover_learn_catalog(repo_roots)` / `_for_scope(selected, repo_roots)` | 扫描每个 root 的 `docs/**/*.md`，读取首个 H1/摘要，跳过 `.git`/`target`，截取 128 项 |
| `discover_plugin_catalog(repo_roots)` / `discover_plugin_catalog_with_project_roots(project_roots, repo_roots)` | 扫描 engine `zircon_plugins` 和 project `Plugins/plugins` 下 `plugin.toml`；去重排序并生成 `PluginCatalogEntry` |
| `AssetCatalogEntry` / `LearnCatalogEntry` / `PluginCatalogEntry` | 提供路径、source/scope、名称与分类；plugin 还公开 editor scope、maturity、packaging 和 module count |
| `TeamOverview::empty` / `discover_team_overview(repo_roots)` | 从第一个有效 Git repository 读取本地 user.name/user.email 与最多 8 位近期作者；无 Git 时返回 empty，不代表远程团队状态 |
| `TeamMemberEntry` | 公开 `name`、`email`、`commits`，commit 计数来自最近 200 条 Git log |

#### 案例：构建 editor/runtime

`BuildCommand::for_editor_runtime` 的实际形状固定为调用 `tools/zircon_build.py`，目标为 `editor,runtime`。`BuildCommandOptions::new` 与 `BuildCommandOptions::with_source_output` 都是公开构造器；`BuildCommand` 没有公开的任意命令构造器。

```rust
use zircon_hub::{
    build::{run_build_command, BuildCommand, BuildCommandOptions},
    settings::BuildProfile,
    state::{TaskCancellationToken, TaskExecutionOutcome},
};

let options = BuildCommandOptions::new(
    "python",
    "cargo",
    "E:/Git/ZirconEngine",
    "E:/ZirconBuild",
    BuildProfile::Debug,
    Some(4),
);
let command = BuildCommand::for_editor_runtime(&options);
let cancellation = TaskCancellationToken::new(42);

match run_build_command(&command, &cancellation)? {
    TaskExecutionOutcome::Completed(report) if report.process_exited_successfully() => {
        println!("{}", report.summary_line());
    }
    TaskExecutionOutcome::Completed(report) => {
        eprintln!("{}\n{}", report.recovery_hint(), report.log_excerpt());
    }
    TaskExecutionOutcome::Cancelled => {
        eprintln!("build cancelled");
    }
}
# Ok::<(), zircon_hub::HubError>(())
```

`TaskCancellationToken::new(task_id)` 要求非零 task ID（debug build 有断言）。调用者可从另一 owner 调 `request_cancellation()`；runner 在 spawn 前、等待期间和收集输出前检查该 token。`BuildExecutionReport::process_exited_successfully()` 只报告进程阶段成功，仍应由后续 staged artifact/receipt 校验决定是否可发布。

#### 案例：项目创建、历史与编辑器启动

```rust
use zircon_hub::{
    process::{EditorLaunchCommand, EditorLaunchRequest},
    projects::{enabled_project_template_id, CreateProjectRequest, RecentProject},
};

let template = enabled_project_template_id("renderable-empty")
    .expect("the only currently enabled template");
let request = CreateProjectRequest::new("Weather Lab", "E:/Projects", template);
request.validate_launch_fields()?;
assert_eq!(request.target_root(), std::path::PathBuf::from("E:/Projects/Weather Lab"));

let launch = EditorLaunchRequest::create_project(request)?;
let command = EditorLaunchCommand::from_staged_engine("E:/ZirconBuild/ZirconEngine", launch)?;
let argv = command.command_line();

// 打开已有项目时，先从真实 manifest 建立 recent entry。
let recent = RecentProject::with_now("E:/Projects/Weather Lab")?;
println!("{}", recent.display_name());
# let _ = argv;
# Ok::<(), zircon_hub::HubError>(())
```

`CreateProjectRequest::new(project_name, location, template)` 只创建值对象；`validate_launch_fields()` 检查项目名称和非空 location（不负责把路径转换为绝对路径），`target_root()` 不创建目录。Tauri action 层另行要求项目 location 为绝对路径。`project_template_catalog()` 会展示多种模板，但当前只有 `renderable-empty` 由 `enabled_project_template_id` 返回。

`EditorLaunchCommand::from_staged_engine` 生成 `zircon_editor(.exe)` 路径；`EditorLaunchRequest::open_project` 与 `create_project` 均封装版本化 `ProjectLaunchIntent`。需要 Hub/Editor handshake 时使用 `.with_hub_handshake(HubSessionToken)`，它追加 `--hub-session <uuid> --hub-protocol 1`；不要手工拼 JSON launch intent 或自行增加未定义的 CLI 参数。

#### 项目、路径与打包 API

| API | 关键输入/返回 | 使用规则 |
| --- | --- | --- |
| `validate_project_root(path)` | `ProjectValidation::{Valid, MissingRoot, MissingManifest, InvalidManifest}` | 在读取 manifest 或启动 editor 之前调用 |
| `RecentProject::{from_project_path, with_now, refresh_summary}` | 从 `zircon-project.toml` 解析 `ProjectManifestSummary` | 不使用目录名替代 manifest 的显示身份 |
| `project_metadata_key` / `project_filesystem_path_key` / `project_paths_match` | 规范化比较键 | Windows 大小写、分隔符和 canonical path 比较用这些 helper |
| `metadata_for_path[_mut]` / `prune_empty_metadata` | `ProjectMetadataMap` 访问 | key 由 helper 生成，不自行格式化字符串 |
| `load_shared_recent_projects[_snapshot]` | `SharedRecentProjectsSnapshot` | 共享 Hub/Editor 投影损坏时加载为空投影，不阻塞 Hub |
| `reconcile_shared_recent_projects[_snapshot]` | 受限 CAS 合并 | 使用上一次 snapshot，避免陈旧 Hub 内存复活 Editor 已删项目 |
| `package_project` | `TaskExecutionOutcome<ProjectPackageReport>` | output root 不能位于 project root 内；跳过 `.git`/`target` |
| `install_package_to_device` | `TaskExecutionOutcome<DeviceInstallReport>` | device root 不能位于 package dir 内；成功后生成 install receipt |
| `recycle_delete_project` | `Result<(), HubError>` | 当前 build 仅 Windows；通过 Recycle Bin 而非递归永久删除 |

`ProjectPackageRequest::new(project_name, project_root, output_root)` 与 `DeviceInstallRequest::new(package_dir, device_root)` 是可直接使用的公开构造器。它们的长操作都可能返回 `TaskExecutionOutcome::Cancelled`，所以把取消当成功、或在取消后假定输出仍完整，都是错误的。

#### Catalog、插件、学习资源与团队

四个 catalog 入口接受 `IntoIterator<Item = PathBuf>`，便于传入 `Vec<PathBuf>` 或数组。它们都以路径 key 去重并排序；前端不应把 scan 返回顺序误解为文件系统原始枚举顺序。

```rust
use std::path::PathBuf;
use zircon_hub::{
    assets::discover_asset_catalog_for_scope,
    learn::discover_learn_catalog_for_scope,
    plugins::discover_plugin_catalog_with_project_roots,
    team::discover_team_overview,
};

let selected = Some(PathBuf::from("E:/Projects/Weather"));
let projects = vec![PathBuf::from("E:/Projects/Weather")];
let engines = vec![PathBuf::from("E:/Git/ZirconEngine")];

let assets = discover_asset_catalog_for_scope(selected.clone(), projects.clone(), engines.clone())?;
let learn = discover_learn_catalog_for_scope(selected, engines.clone())?;
let plugins = discover_plugin_catalog_with_project_roots(projects, engines.clone())?;
let team = discover_team_overview(engines)?;
# let _ = (assets, learn, plugins, team);
# Ok::<(), zircon_hub::HubError>(())
```

`AssetCatalogEntry` 提供 `name`、`kind`、`source`、`size_bytes`、`path`；当前扫描上限是 256。`LearnCatalogEntry` 提供 `title`、`category`、`source`、`summary`、`path`，上限为 128。`PluginCatalogEntry` 还包含 plugin ID、版本、scope、category、description、module count 与 source path；深入字段和 manifest 规则见 [Manifest、模块与 Feature Bundle](../plugins/reference/manifest-modules.md)。`TeamOverview` 仅是本地 Git 摘要，Git 缺失或目录不是 repository 时返回空概览而不是远程团队真相。

#### 设置、快照、任务和消息

| 类型/API | 作用 |
| --- | --- |
| `HubConfig::{load, save, repair_registries}` | 持久化 settings、recent projects、engine registry、window/runtime state 与 action history；save 使用临时文件 + replace |
| `HubConfigRepairReport::repaired_anything` | 指示 dedupe、过期 metadata、action history 或 active engine 是否被修复 |
| `HubSettings` / `HubRuntimeState` | 持久化/草稿状态；`HubRuntimeState::normalize` 清理空 selection 与默认值 |
| `BuildProfile::{as_mode, from_ui_value}` | `Debug`/`Release` 到 build-script mode 的唯一转换 |
| `HubLanguage::{as_ui_value, from_ui_value}` | English/Chinese UI 值的解析与投影 |
| `HubSnapshot::{scope, filtered_recent_projects}` | 给 UI 的只读聚合；`scope()` 统一选择项目与 source engine |
| `HubScope::resolve` | 防止 selected project 缺 engine 时错误回退到 active engine |
| `TaskStatus` / `TaskCancellationToken` / `TaskExecutionOutcome<T>` | 后台任务的用户可见状态、取消请求和终态 |
| `HubMessage` / `HubMessageId` | 结构化、本地化 detail/recovery；不要由 UI 根据显示文本分支 |
| `push_action_record` | 将 `HubActionRecord` 插到历史头部，最多保留 `ACTION_HISTORY_LIMIT`（当前 16） |

### Tauri Desktop 边界

`tauri_app::run() -> Result<(), HubError>` 是 public desktop 应用装配入口。它创建 `HubCommandState`、`AccountCommandState`，注册 `hub_state`、`hub_action`、`account_state`、`account_action`，并在窗口重新获得焦点时刷新共享 recent-project projection。虽然 command 函数带 `#[tauri::command]`，它们在模块内是私有实现；外部 WebView 以注册名称调用，而 Rust crate consumer 不应依赖私有 state 类型。

```ts
import { invoke } from "@tauri-apps/api/core";

// 命令名称是稳定的 UI 边界；DTO 字段以当前 web/src/types/hub.ts 为准。
const snapshot = await invoke("hub_state");
const next = await invoke("hub_action", {
  request: {
    actionId: "search-projects",
    targetId: null,
    payload: { query: "weather" },
  },
});
```

上面的 action 使用当前源码中的 `search-projects` 作为可执行示例；可接受 action ID 和 payload schema 由 `HubActionRequest::parse_as` 的当前实现决定，未知 action、缺失 required payload、相对路径和错误 task ID 必须在 Rust 侧拒绝。不要根据这段示例新增前端 action；专题见 [Hub 状态与命令](../hub-tooling/reference/hub-state-commands.md)。

## `account-broker`：账户与凭据 API

### 配置和入口

```rust
use zircon_hub::account::{AccountBroker, AccountConfig, ServiceRequest};

async fn account_flow(config_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let config = AccountConfig::load(config_path)?;
    let broker = AccountBroker::new(config)?;

    let signed_in = broker.authenticate(false).await?;
    let organizations = broker
        .service_request(
            &signed_in.generation,
            ServiceRequest::Organizations { after: None },
        )
        .await?;

    println!("{}", organizations);
    let _signed_out = broker.logout().await?;
    Ok(())
}
```

`AccountConfig::load(path)` 读取最多 65,536 bytes 的常规 JSON 文件，拒绝 unknown field、低于 1024 的 callback port、空/过长 client ID、危险 URL 与不安全 journal path。`endpoint(value)` 只接受 HTTPS，除非 `allow_loopback_http` 为 true 且 host 是 IP loopback；URL 不能包含 username、password、query 或 fragment。`redirect()` 始终构造 `http://127.0.0.1:<callback_port>/callback`，因为浏览器返回由本机临时 listener 接收。

配置文件字段与 Rust 结构一一对应（`operation_journal_path` 可省略）：

```json
{
  "issuer": "https://identity.example/realms/zircon",
  "client_id": "zircon-hub",
  "service_url": "http://127.0.0.1:8787",
  "callback_port": 8480,
  "allow_loopback_http": true,
  "operation_journal_path": "E:/ZirconLocal/Account/operations.dat"
}
```

`AccountConfig` 使用 `serde` 的字段名（snake_case），与 service HTTP DTO 的 camelCase 规则不同；不要把这份本地 JSON 配置当作 `ServiceRequest` body。`service_url` 在生产环境应为 HTTPS，示例仅因 loopback 地址且显式开启 `allow_loopback_http` 才合法。

| `AccountBroker` API | 语义 | 失败和并发规则 |
| --- | --- | --- |
| `new(config)` | 创建 Windows credential backed broker 并检查 pending revocation | credential store 不可用返回 `CredentialStore` |
| `view().await` | 返回可序列化 `AccountView` | 不含 refresh token、nonce 或 access token |
| `authenticate(refresh).await` | `false` 发起交互登录；`true` 尝试已存 refresh session | 同时 login 返回 `Busy`；logout/取消使旧 attempt 返回 `Cancelled` |
| `logout().await` | 清状态、取消未完成 operation、清本地凭据并尝试 revoke | provider 不可用时可能进入 `RevocationPending`，但 view 仍为 signed-out |
| `organizations().await` | 使用当前 generation 查询组织 | 未登录/过代分别为 `SessionExpired`/`Cancelled` |
| `service_request(generation, request).await` | 通用账户服务请求 | generation 或登录态在响应期间变化时，读取请求为 `Cancelled`，mutation 为 `OutcomeUnknown` |
| `catalog` / `catalog_entitlements` | 受限 typed catalog 映射 | 页大小、ID、revision、artifact metadata 再校验 |
| `package_inventory` / `install_package` | 本地 package policy + 可恢复安装 | mutation lock、防重复 operation ID、trust/capacity 错误 |
| `recovery_snapshot` / `retry_operation` / `reconcile_operation` / `acknowledge_operation` | 非 secret operation journal 恢复 | mutation 与 acknowledgement 排他，避免错误确认正在重试的记录 |

精确的公开方法签名（省略错误类型展开）是：`view(&self) -> Future<Output = AccountView>`、`authenticate(&self, refresh: bool) -> Future<Output = Result<AccountView, AccountError>>`、`logout(&self) -> Future<Output = Result<AccountView, AccountError>>`、`organizations(&self) -> Future<Output = Result<serde_json::Value, AccountError>>`、`service_request(&self, expected_generation: &str, request: ServiceRequest) -> Future<Output = Result<serde_json::Value, AccountError>>`、`catalog(&self, generation: &str, after: Option<String>, query: Option<String>) -> Future<Output = Result<serde_json::Value, AccountError>>`、`catalog_entitlements(&self, generation: &str, organization: String, after: Option<String>) -> Future<Output = Result<serde_json::Value, AccountError>>`、`package_inventory(&self, generation: &str, organization: String) -> Future<Output = Result<serde_json::Value, AccountError>>`、`install_package(&self, generation: &str, organization: String, operation_id: String, package_id: String, revision: String, expected_inventory_revision: String) -> Future<Output = Result<serde_json::Value, AccountError>>`、`recovery_snapshot(&self) -> (AccountView, Result<(Vec<OperationSummary>, String), AccountError>)`、`retry_operation(&self, expected_generation: &str, operation_id: &str) -> Future<Output = Result<serde_json::Value, AccountError>>`、`reconcile_operation(&self, expected_generation: &str, operation_id: &str) -> Future<Output = Result<serde_json::Value, AccountError>>` 和 `acknowledge_operation(&self, expected_generation: &str, operation_id: &str) -> Future<Output = Result<(), AccountError>>`。调用方应保存 `generation` 字符串原值，不能自行递增或转成用户身份。

`ServiceRequest` 是 tagged JSON DTO，`action` 使用 kebab case，字段使用 camelCase。源码当前的完整 variant 集合是 `Catalog { after, query }`、`CatalogEntitlements { organization, after }`、`CatalogArtifact { organization, package_id, revision }`、`CatalogLicense { organization, operation_id, expected_policy_revision, package_id, revision, license_id }`、`Organizations { after }`、`Invitations { after }`、`IssuedInvitations { organization, after }`、`Members { organization, after }`、`Projects { organization, after }`、`CreateOrganization { operation_id, name }`、`Mutate { organization, operation_id, expected_policy_revision, mutation }` 和 `Receipt { operation_id }`。它不是开放的“任意 URL”请求：broker 从固定 `service_url` 派生 endpoint，验证 ID/revision，向请求加 bearer token，并限制响应体。

#### `AccountConfig`、错误和 operation journal

| 符号 | 真实调用形状与约束 |
| --- | --- |
| `AccountConfig::load(&Path)` | 读取不超过 65,536 bytes 的 regular JSON；拒绝 unknown fields、危险 endpoint、`callback_port < 1024`、空/超过 256 字节的 `client_id` |
| `AccountConfig::endpoint(&str)` | 解析并校验 URL；只允许 HTTPS，或在 `allow_loopback_http` 下允许 IP loopback HTTP；拒绝 user/password/query/fragment |
| `AccountConfig::redirect()` | 返回固定 `http://127.0.0.1:<callback_port>/callback`，供 PKCE callback listener 使用 |
| `AccountConfig::journal_path()` | 返回显式绝对路径（不得含 `..`），否则回退到 `%LOCALAPPDATA%/ZirconHub/Account/operations.dat`；显式路径不安全返回 `Configuration`，无法得到绝对 `%LOCALAPPDATA%` 回退目录才返回 `OperationStore` |
| `AccountView::default()` | 产生 `configured=false`、`status="unavailable"`、generation `"0"` 的无 secret 初始投影 |
| `AccountView` | 仅包含 `configured`、`status`、`issuer`、`subject`、`display_name`、`generation`、`error`；不得向 UI 序列化 token、nonce 或 credential blob |

`AccountError` 的完整稳定分类为：`Configuration`、`ProviderUnavailable`、`CredentialStore`、`Callback`、`Cancelled`、`Timeout`、`Browser`、`InvalidIdentity`、`SessionExpired`、`Busy`、`ServiceFailure`、`OutcomeUnknown`、`OperationStore`、`OperationConflict`、`RevocationPending`、`PackageUnavailable`、`PackageTrust`、`PackageCapacity`、`PackageConflict`。这些值的 `Display` 是机器可读 code（例如 `account_service_outcome_unknown`），但 UI 应按枚举语义映射 detail/recovery，不要依赖英文文案。

交互登录的 callback listener 只绑定 `127.0.0.1:<callback_port>`，等待授权结果最多 120 秒；query 最大 8 KiB，`state` 和 `code` 必须各出现一次并匹配 PKCE/CSRF state。callback 必须带正确 `Host` 且不得带 `Origin`，成功后 listener 最多再等待 1 秒收尾。OIDC HTTP client 禁止自动跟随 redirect，connect timeout 为 3 秒、总请求 timeout 为 15 秒；discovery/token 响应按 1 MiB（provider）或 256 KiB（service response）上限读取。

operation journal 只保存 identity-scoped 的非 secret payload、generation、attempt/status 和 operation ID。`recovery_snapshot()` 原子读取当前 `AccountView` 与 `(Vec<OperationSummary>, journal_revision)`；未登录时返回空列表和 `"0"`。`retry_operation(generation, operation_id)` 重新取得 journal record 并使用原 operation ID；`reconcile_operation` 先查询 service receipt/package install state，再将可验证结果写回 journal；`acknowledge_operation` 只删除/确认已展示记录，generation 过期或 mutation 正在占用时分别返回 `Cancelled`/`Busy`。journal 文件上限为 16 MiB、最多 128 条记录，写入由 credential lock 串行化。

#### Catalog 与本机 package workflow

`catalog(generation, after, query)` 将 service catalog page 解码为受限 release DTO，最多接受 100 条；每个 release 的 package ID、正 revision、kind、digest、size、license 和 expiration 都会再次校验。`catalog_entitlements(generation, organization, after)` 最多接受 50 条 entitlement，并检查 package ID/revision/license ID。两者都在响应发布前重新检查 generation。

`package_inventory(generation, organization)` 先确认 entitlement，再读取 `ZIRCON_HUB_PACKAGE_CONFIG` 指向的 regular JSON 配置（最多 65,536 bytes），验证 executable/policy/runtime library 的绝对路径和 SHA-256，最后调用受监督 package executable 的 `inventory` 动作。`install_package(generation, organization, operation_id, package_id, revision, expected_inventory_revision)` 只接受 `kind == "plugin"` 的 release；artifact 最大 16 MiB，下载后核对 content length、SHA-256、签名/策略 digest 和 inventory revision，再由 package process 执行安装。任何可疑 digest、owner、路径或 revision 都是 `PackageTrust`/`PackageCapacity`，不得降级为普通 service error。

#### 账户状态机与安全规则

```text
AccountConfig::load
  -> AccountBroker::new
  -> view: configured + signed-out
  -> authenticate(false | true)
       -> browser/OIDC callback + credential store write
       -> view: signed-in，generation 增加
  -> service_request(expected generation, ServiceRequest)
  -> logout
       -> 取消 in-flight work，清 credential，记录/重试 revoke
       -> view: signed-out，generation 增加
```

`AccountView` 的 public fields 是 `configured`、`status`、`issuer`、`subject`、`display_name`、`generation`、`error`。UI 必须将 `generation` 带回 service action；它是防止 logout/账号切换后迟到响应覆盖新会话的版本 fence，不是用户身份或长期 session key。

错误 `AccountError` 需要按恢复语义处理：`Configuration` 修配置；`ProviderUnavailable` 可用于非 mutation 重试；`Busy` 等待当前操作；`Cancelled` 丢弃过时 UI 结果；`OutcomeUnknown` 则查询/reconcile operation，不能盲目重复 mutation；`RevocationPending` 表示本地已登出但远端撤销要在后续 broker 启动或登录前重试。

账户专题和 Tauri action DTO 见 [账户与本地服务](../hub-tooling/reference/account-service.md)。

## `local-service`：本地服务、HTTP 与存储 API

### 运行与配置

`zircon_hub_service` binary 的真实入口是：创建 Tokio multi-thread runtime，调用 `zircon_hub::service::run()`，失败时写出 `error.code()` 并以非零退出。`service::run` 从**第一个命令行参数**取得 service TOML 路径；因此启动形状是：

```powershell
cargo run -p zircon_hub --no-default-features --features local-service --bin zircon_hub_service -- `
  E:/ZirconLocal/service.toml
```

`ServiceConfig::load(path)` 读取最多 65,536 bytes 的 TOML，随后调用 `validate()`。有效配置要求：

- `bind` 必须是 loopback 地址；
- `database` 与 `introspection_secret_file` 必须是绝对路径；可选 `catalog_policy_file` 也是绝对路径；
- `issuer` 必须是 HTTPS，除非 `allow_loopback_http = true` 且目标确为 loopback；
- `cloud.root` 和 `cloud.key_file` 必须是绝对路径、没有 `..`，且 key file 不在 blob root 内。

示例配置只表达字段形状；敏感文件路径应来自本机部署配置，不能提交密钥：

```toml
bind = "127.0.0.1:8787"
database = "E:/ZirconLocal/hub.db"
issuer = "https://identity.example/realms/zircon"
audience = "zircon-hub-service"
introspection_client_id = "zircon-hub-service"
introspection_secret_file = "E:/ZirconLocal/secrets/introspection.txt"
allow_loopback_http = false

[cloud]
root = "E:/ZirconLocal/cloud"
key_file = "E:/ZirconLocal/secrets/cloud.key"
```

服务启动顺序是 Ctrl-C supervisor -> config -> migrated `Database` -> recovered `BlobStore` -> discovered `OidcVerifier` -> loopback `TcpListener` -> Axum router。任一启动阶段失败或接到 shutdown 后会停止 database admission，并在 supervisor 的 20 秒 deadline 内关闭 job；CLI binary 在 Tokio runtime 退出时再执行 2 秒 `shutdown_timeout`。无法确认收尾时返回 `ServiceError::OutcomeUnknown`，而不是假装安全关闭。

### 公共 Rust 模块表

`service` root 当前只 re-export `run`。源码中的 `config`、`error`、`identity`、`storage`、`http`、`organization`、`catalog` 和 `cloud` 都以私有 `mod` 声明，因而**不能**从外部 crate 通过 `zircon_hub::service::<child>` 调用；它们的 `pub` 项是 service crate 内部跨模块/测试契约。下表保留这些真实 Rust 符号，帮助维护实现和阅读 HTTP handler，但对外集成应使用 `service::run`、`zircon_hub_service` 二进制和 loopback HTTP 协议。

| 路径 | 核心 API | 约束 |
| --- | --- | --- |
| `service::config`（内部） | `ServiceConfig::{load, validate, validate_endpoint}` | 配置和 endpoint 安全边界 |
| `service::error`（内部） | `ServiceError`, `ServiceError::code()` | HTTP status/JSON error code 映射 |
| `service::identity`（内部） | `OidcVerifier::{discover, verify}`, `Principal`, `now_seconds` | bearer 验签 + introspection；不把 `Principal` 伪造成客户端输入 |
| `service::storage`（内部） | `Database::{open, execute}`, `receipt::*` | SQLite admission、迁移、幂等 operation receipt |
| `service::http`（内部） | `router(identity, database, policy_file, cloud)` | 完整 HTTP surface，含 origin 禁止、15 秒 request timeout、全局并发 32 |
| `service::organization`（内部） | `create`、`mutate`、分页 query DTO | 组织 policy revision 与角色授权 |
| `service::catalog`（内部） | policy、release、publish/list/entitlement/artifact | signed release、publisher identity 与 license entitlement |
| `service::cloud`（内部） | config、manifest、retention、commit、blob store | 加密 blob、manifest 约束、quota、receipt 与 retention |

#### `local-service` 源码符号索引（仅 crate 内）

下列项目来自 `service` 子模块的 `pub` 声明，但由于 `service/mod.rs` 使用私有 `mod`，外部 crate 不能通过同名路径导入。它们仍是实现、handler 和测试之间的真实调用契约；第三方集成应使用上一节的 binary/HTTP 协议。

| 内部路径 | 公开类型/函数 | 真实职责和调用边界 |
| --- | --- | --- |
| `service::config` | `ServiceConfig::{load, validate, validate_endpoint}` | 从 regular TOML 读取 65,536-byte 上限配置；校验 loopback bind、绝对路径和 HTTPS/loopback HTTP endpoint |
| `service::error` | `ServiceError::{Configuration, IdentityUnavailable, Unauthorized, Forbidden, Conflict, InvalidRequest, Storage, Capacity, OutcomeUnknown, OperationConflict}`、`code()` | 将错误稳定映射到 JSON `error` code 与 HTTP status；不携带 secret 或 SQL 细节 |
| `service::identity` | `Principal { issuer, subject }`、`OidcVerifier::{discover, verify}`、`now_seconds()` | discovery 同源校验、RS256/JWKS cache、每请求 token introspection；handler 只能使用 verifier 生成的 Principal |
| `service::storage` | `Database::{open, execute}`、`receipt::{validate_id, fingerprint, replay, commit, lookup}`、`OperationStatus` | SQLite WAL/FULL、最多 16 个 job permit；receipt 以 `(issuer, subject, operation_id, fingerprint)` 幂等化写入 |
| `service::http` | `router(identity, database, catalog_policy_file, cloud)` | 建立 Axum route tree、body/request/cloud/catalog semaphore、15 秒 timeout 与 Origin 拒绝 middleware |
| `service::organization` | `Organization`、`CreateOrganization`、`Mutation`、`MutationRequest`、`Receipt`；`create`、`mutate`；`query::{PageQuery, Page, Organization/Invitation/IssuedInvitation/Member/Project}` 与 `members/projects/list/invitations/issued_invitations` | 在 `&mut rusqlite::Connection` 上开启 Immediate transaction，执行角色授权、policy revision CAS、audit 和 receipt |
| `service::catalog` | `CatalogPolicy::load`、`Publisher`、`Release`、`PublishRequest`、`Publication`、`CatalogQuery`、`CatalogPage`、`Entitlement`、`AcceptLicense`；`publish`、`entitlements`、`authorized_artifact`、`list`、`accept_license`；`artifacts::{MAX_ARTIFACT_BYTES, upload, download, envelope}` | RS256 signed envelope、publisher subject、release digest/size、entitlement/license 读写；artifact 子模块按 16 MiB 上限处理上传/下载 |
| `service::cloud` | `CloudConfig::validate`；`Manifest::validate`、`FileEntry`、`digest`；`CommitRequest`、`Snapshot`、`SnapshotReceipt`、`CommitOutcome`、`head`、`commit`、`authorize_receipt`；`Usage`/`usage`；`RetentionPolicy`、`RetentionState`、`RetentionRequest`、`RetentionOutcome`、`retention`、`update_retention`；`MaintenanceRequest`/`MaintenanceReport`/`maintain`；`BlobStore::load`、`upload`、`read_blob`；常量 `MAX_BLOB_BYTES`、`MAX_MANIFEST_BYTES`、`MAX_PROJECT_BYTES`、`IGNORE_POLICY` | 通过 AES-256-GCM BlobStore 和 manifest/revision/retention/quota 规则完成 cloud snapshot；任何写入都必须经过 receipt |

service 内部 Rust 调用的共同形状是“handler 先取得 `Principal`，再通过 `Database::execute` 传入 `&mut Connection`”；不要在请求线程直接持有全局 connection，也不要把 `Principal` 从 JSON body 反序列化。以下是边界示意（仅用于阅读源码，不能作为外部 import 路径）：

```rust,ignore
// 在 crate::service::http handler 内部：
let principal = principal(&state, &headers).await?;
state
    .database
    .execute(move |connection| {
        crate::service::organization::list(connection, &principal, query)
    })
    .await
```

#### HTTP 路由与认证

`service::http::router` 注册下面的 service API。除了 `/health` 外，业务路由都要求 `Authorization: Bearer <token>`，并由 `OidcVerifier` 验证。含 `Origin` header 的请求会在全局 middleware 被拒绝：浏览器 origin 不得直接调用 privileged local API，桌面 broker 使用 native HTTP。

| HTTP 路径 | 方法 | API/语义 |
| --- | --- | --- |
| `/health` | GET | 返回 `{ "status": "alive", "protocolVersion": 1 }`，不做 bearer 认证 |
| `/v1/organizations` | GET / POST | 分页列出组织；创建组织需 `CreateOrganization { operation_id, name }` |
| `/v1/organizations/{organization}/mutations` | POST | `MutationRequest`，对 policy revision 做 compare-and-update |
| `/v1/organizations/{organization}/members` | GET | 成员分页 |
| `/v1/organizations/{organization}/invitations` | GET | admin/owner 查询已发邀请 |
| `/v1/organizations/{organization}/projects` | GET | 组织项目分页 |
| `/v1/invitations` | GET | 当前 principal 的邀请 |
| `/v1/operations/{operation}` | GET | 幂等 operation receipt/reconciliation 状态 |
| `/v1/catalog` | GET / POST | catalog list / signed release publish |
| `/v1/catalog/{package}/{revision}/artifact` | PUT | 仅 publisher 上传，最大 16 MiB artifact |
| `/v1/organizations/{org}/catalog/{pkg}/{rev}` | GET | entitlement 后读取 release metadata |
| `/v1/organizations/{org}/catalog/{pkg}/{rev}/artifact` | GET | entitlement 后下载 artifact |
| `/v1/organizations/{org}/catalog/{pkg}/{rev}/manifest` | GET | entitlement 后读取 `application/jwt` signed release envelope |
| `/v1/organizations/{org}/licenses` | GET / POST | router 当前注册的 entitlement list / license accept 路径 |
| `/v1/organizations/{org}/projects/{project}/cloud/head` | GET | 当前 project head snapshot；无 head 时返回 `null` |
| `/v1/organizations/{org}/projects/{project}/cloud/usage` | GET | logical/physical bytes、objects、revisions、quota 和 GC 指标 |
| `/v1/organizations/{org}/projects/{project}/cloud/retention` | GET / POST | 读取或以 `expectedRevision` CAS 更新 retention policy |
| `/v1/organizations/{org}/projects/{project}/cloud/maintenance` | POST | 按 operation ID 执行 bounded prune/GC |
| `/v1/organizations/{org}/projects/{project}/cloud/commit` | POST | 以 `baseRevision` compare-and-commit manifest；冲突返回 409 |
| `/v1/organizations/{org}/projects/{project}/cloud/blobs/{digest}` | GET / PUT | 读取已引用 blob 或上传 SHA-256 匹配的 blob |

通用 HTTP body 默认限制为 65,536 bytes，cloud manifest 路由使用 `MAX_MANIFEST_BYTES + 65,536` 读取，blob upload 使用 `MAX_BLOB_BYTES`。路由层将 `ServiceError` 映射为结构化 `{ "error": "..." }`：401 `unauthorized`，403 `forbidden`，409 `policy_conflict`/`operation_id_conflict`，400 `invalid_request`，429 `capacity_exceeded`，其它为 503。调用方必须把 503 `operation_outcome_unknown` 当作需要 receipt 查询的未知提交结果。

**Broker 路径注意事项：** 当前 `account::ServiceRequest::CatalogEntitlements` 和 `CatalogLicense` 的源码会生成 `/v1/organizations/{organization}/entitlements`，而 `service::http::router` 注册的是 `/v1/organizations/{organization}/licenses`。这是当前 checkout 的跨 feature 路径不一致；直接组合 `account-broker` 与 `local-service` 时 entitlement/license 请求会得到未匹配路由，直到两侧收敛前应以实际 HTTP 路由或同版本修复为准，不要把两种路径当作可互换别名。

#### CLI/HTTP 调用示例

服务监听后，受信任的桌面 broker 可以用 native HTTP 调用 loopback API。下面的 PowerShell 片段只展示协议形状；`$accessToken` 必须由 OIDC broker 在内存中取得，不能写入脚本或日志。除 `/health` 外每个请求都带 bearer，且不要发送 `Origin` header：

```powershell
$base = "http://127.0.0.1:8787"

# 健康检查不要求 bearer。
Invoke-RestMethod -Method Get -Uri "$base/health"

$headers = @{ Authorization = "Bearer $accessToken" }

# 分页查询当前 principal 可见的组织。
$page = Invoke-RestMethod -Method Get `
  -Uri "$base/v1/organizations?limit=50" `
  -Headers $headers

# 创建组织是幂等 mutation；operationId 必须是 canonical UUID。
$body = @{
  operationId = "00000000-0000-4000-8000-000000000021"
  name = "Weather Lab"
} | ConvertTo-Json -Compress
$created = Invoke-RestMethod -Method Post `
  -Uri "$base/v1/organizations" `
  -Headers $headers -ContentType "application/json" -Body $body
```

`limit` 的有效范围是 1..100；响应中的 `nextCursor` 原样放入下一页的 `after`。网络超时或 503 `operation_outcome_unknown` 时，使用相同 operation ID 查询 `/v1/operations/{operation}`；不要生成新的 ID 重试同一个写入。浏览器/React WebView 即使能访问 loopback，也会因 `Origin` middleware 得到 403，这是刻意保留的 broker 权限边界。

#### Database 与 operation receipt

```rust,ignore
use crate::service::{
    error::ServiceError,
    identity::Principal,
    storage::{receipt, Database},
};
use rusqlite::TransactionBehavior;

async fn idempotent_operation(
    database_path: std::path::PathBuf,
    principal: Principal,
) -> Result<(), ServiceError> {
    let database = Database::open(database_path)?;
    database
        .execute(move |connection| {
            let operation_id = "00000000-0000-4000-8000-000000000001";
            let fingerprint = receipt::fingerprint(&("example", operation_id))?;

            // 先取得 Immediate transaction；replay 检查与写入必须共用它。
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            receipt::validate_id(operation_id)?;
            if let Some::<serde_json::Value>(_previous) =
                receipt::replay(&transaction, &principal, operation_id, &fingerprint)?
            {
                return Ok(());
            }

            // 业务写入、receipt 与 commit 必须位于同一个 transaction。
            // 在这里执行 authorization 和业务变更。
            receipt::commit(
                &transaction,
                &principal,
                operation_id,
                &fingerprint,
                &serde_json::json!({"status": "committed"}),
            )?;
            transaction.commit()?;
            Ok(())
        })
        .await
}
```

上例是 service crate 内部的 transaction 形状（标为 `ignore`，外部依赖无法导入这些私有 child module），不是第三方 SDK 示例。真实 mutation 应通过 loopback HTTP 或 `service::run` 的 handler 进入 `organization`、`catalog`、`cloud`，由它们在 transaction 内执行 authorization、fingerprint、replay、audit 与 commit。`Database::execute` 通过受限 job queue 运行闭包；queue 饱和返回 `Capacity`，worker/回复异常返回 `OutcomeUnknown`。不要从请求线程长期持有 SQLite connection，也不要绕过 receipt 用“HTTP 重试”重复写入。

#### Organization 与 catalog 调用形状

`organization::Mutation` 是强类型操作集：`CreateProject`、`SetMember`、`TransferOwnership`、`Invite`、`AcceptInvite`、`RevokeInvite`。mutation request 必须带 `operation_id` 和 `expected_policy_revision`。过期 policy revision 得到冲突，而不是自动覆盖。

```rust,ignore
use crate::service::{
    error::ServiceError,
    identity::Principal,
    organization::{create, mutate, Mutation, MutationRequest},
};

fn organization_flow(
    connection: &mut rusqlite::Connection,
    principal: &Principal,
) -> Result<(), ServiceError> {
    let organization = create(
        connection,
        principal,
        "00000000-0000-4000-8000-000000000010",
        "Zircon Team",
    )?;

    let _receipt = mutate(
        connection,
        principal,
        &organization.id,
        MutationRequest {
            operation_id: "00000000-0000-4000-8000-000000000011".into(),
            expected_policy_revision: organization.policy_revision,
            mutation: Mutation::CreateProject {
                name: "Weather Lab".into(),
            },
        },
    )?;
    Ok(())
}
```

`catalog::publish(connection, principal, policy, PublishRequest)` verifies an RS256 signed release envelope, matching publisher issuer/subject, ID/revision/digest/size constraints，再写入 operation receipt。`catalog::artifacts::upload` checks publisher ownership, expected SHA-256 and 16 MiB per-artifact capacity; `download` requires organization entitlement。catalog client 不应把“HTTP 200 的 metadata”当作 artifact 已被验证，必须走带 entitlement 的 artifact endpoint 或 broker 的 package workflow。

#### Cloud manifest、blob 与 retention

`service::cloud` 的协议类型是带 camelCase 字段的 serde DTO，但父模块对外私有；因此这些类型是实现契约，不是外部 crate API。`Manifest::validate()` 要求 schema version 1、`ignore_policy == "zircon-project-v1"`、最多 10,000 files、每 blob 不超过 16 MiB、总 logical project size 不超过 512 MiB、lowercase SHA-256 digest，以及严格的 relative path allowlist。它拒绝 `..`、Windows reserved names、secret-like paths/extensions、`.git`、`target`、`node_modules` 等。

| API | 核心规则 |
| --- | --- |
| `cloud::BlobStore::load(&CloudConfig)` | key 必须正好 32 bytes；root/key path 抗 symlink/reparse；store lock 防止多 owner；at-rest 使用 AES-256-GCM |
| `cloud::upload(...)` | digest 与 body 必须匹配，作者须有写权限，记录短期 upload lease |
| `cloud::read_blob(...)` | blob 必须已被 snapshot 引用，读取也写 audit |
| `cloud::commit(...)` | manifest canonicalization + base revision compare；返回 `CommitOutcome::Committed` 或 durable `Conflict` |
| `cloud::usage(...)` | 返回 logical/physical bytes、objects、revision、retention 与 pending GC 指标 |
| `cloud::retention(...)` / `update_retention(...)` | policy revision compare-and-update；admin/owner 才能更新 |
| `cloud::maintain(...)` | 两阶段 prune/GC；storage/capacity fault 会转为 `OutcomeUnknown` 以保护幂等语义 |

对于含写入的 cloud 请求，操作 ID 不是装饰字段：commit、retention update 和 maintenance 都通过 receipt 固化终态。网络超时后应调用 `/v1/operations/{operation}` 或 broker 的 `reconcile_operation`，不要使用新的 operation ID 重发同一 mutation。

## 生命周期案例：安全关闭 Hub 与 Runtime

下面顺序串联两个 crate 的职责。它不是某个函数一键完成的 API，而是产品 host 的 owner 顺序：

```text
停止 Hub 新任务 admission / UI action
  -> 请求取消 Build、Package、Device install 等 TaskCancellationToken
  -> 停止向 Runtime session 提交新请求
  -> 消费或释放全部 ZrOwnedResultV2
  -> 对 bound_viewports() 逐个 begin_release + ABI unbind + finish
  -> 等待 runtime wake callback 和 child process 收割
  -> destroy Runtime session，随后 drop 动态 library owner
  -> service 侧 stop database admission，优雅停止 HTTP，按 deadline shutdown job
  -> 持久化 HubConfig / action history
```

最重要的跨层规则是：Runtime allocation 的 provider/session 必须活到 release 完成；native surface 在 Runtime/session teardown 前解绑；账户 UI response 只能发布到匹配 generation 的 signed-in view；本地服务 shutdown 的 storage deadline 失败必须上报 unknown outcome。

## 常见错误与最佳实践

| 症状 | 根因 | 正确处理 |
| --- | --- | --- |
| `RuntimeForeignOutputErrorKind::ProtocolViolation` | output 形状、budget、decode 或 release 协议不可信 | 读取 metrics，销毁并重新创建 session；不要 clear fuse |
| `ViewportSurfaceOperationInFlight` | 同一个 viewport 的 bind/release 并发 | 等待或串行化前一个 token；不要绕过 registry 发 ABI call |
| build 进程 exit 0 但后续不能启动 editor | staged artifact/handshake 尚未验证 | 保留 `BuildExecutionReport`，执行产品 staging/receipt 校验 |
| `AccountError::OutcomeUnknown` | mutation 的网络/持久化结果无法证明 | 用同一 operation ID reconcile，不能盲目重投 |
| `ServiceError::Capacity` | request、job、blob、quota 或并发 permit 超限 | 缩小/分页/退避；不提高常量来绕过保护 |
| service 返回 403 且浏览器请求带 `Origin` | local API 明确拒绝浏览器 origin | 通过 trusted desktop broker，而非给 WebView 直连权限 |
| 共享 recent project 被旧 Hub 状态复活 | 未使用 revisioned reconciliation | 保留上次 `SharedRecentProjectsSnapshot` 并调用 snapshot variant |

**实践清单：**

- 只从 public re-export 路径调用 Hub 领域 API；`tauri_app` 内部 state、`pub(crate)` process helper 和 private DTO 不应被外部依赖。
- 对 `Result<TaskExecutionOutcome<T>, HubError>` 先处理外层错误，再显式处理 `Completed` 与 `Cancelled`，避免把取消当作失败重试或成功提交。
- 所有 native program 调用使用 `program + args` 形状，例如 `BuildCommand::command_line()`、`EditorLaunchCommand::command_line()`，不要拼 shell command。
- `RuntimeOwnedOutputReleaser` 与 foreign output state 按 session 持有；不要跨 provider、跨 session 或跨动态库 reload 复用。
- `ViewportSurfaceBindingOperation` 和 `ViewportSurfaceReleaseOperation` 必须在 FFI 调用结果已知后立即 `finish`；不要把 token 存入长期容器。
- 对账户与 service mutation 保留 operation ID、generation、policy revision 和 receipt；日志仅记录可安全审计的 ID，永不记录 token、nonce、密钥或 secret file 内容。

## 源码与验证入口

- [runtime host crate root](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_host/src/lib.rs)
- [foreign-output state](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_host/src/foreign_output/state.rs)
- [viewport surface registry](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_host/src/viewport_surface.rs)
- [Hub crate root 与 feature gate](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_hub/src/lib.rs)
- [Hub desktop command registration](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_hub/src/tauri_app/mod.rs)
- [account broker](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_hub/src/account/mod.rs)
- [local service lifecycle](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_hub/src/service/lifecycle.rs)
- [foreign-output regression tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_host/src/foreign_output/tests.rs)
- [Hub service authority contract](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_hub/tests/service_authority_contract.rs)
