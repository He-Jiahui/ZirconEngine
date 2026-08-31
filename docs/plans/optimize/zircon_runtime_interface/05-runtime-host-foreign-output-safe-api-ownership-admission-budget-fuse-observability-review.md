---
related_code:
  - zircon_runtime_host/Cargo.toml
  - zircon_runtime_host/src/lib.rs
  - zircon_runtime_host/src/foreign_output/budget.rs
  - zircon_runtime_host/src/foreign_output/decode.rs
  - zircon_runtime_host/src/foreign_output/error.rs
  - zircon_runtime_host/src/foreign_output/item_count.rs
  - zircon_runtime_host/src/foreign_output/kind.rs
  - zircon_runtime_host/src/foreign_output/metrics.rs
  - zircon_runtime_host/src/foreign_output/mod.rs
  - zircon_runtime_host/src/foreign_output/owned_buffer.rs
  - zircon_runtime_host/src/foreign_output/policy.rs
  - zircon_runtime_host/src/foreign_output/state.rs
  - zircon_runtime_host/src/foreign_output/tests.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/handles.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/runtime_api/session/requests.rs
  - zircon_runtime_interface/src/runtime_api/session/plugin_event_mirror.rs
  - zircon_runtime_interface/src/profiling.rs
  - zircon_runtime_interface/src/world_sync
  - zircon_runtime_interface/src/ui/accessibility.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/dynamic_api/session/registry/allocation_registry.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_library/runtime_session/foreign_output.rs
  - zircon_app/src/entry/runtime_library/runtime_session/operation.rs
  - zircon_app/src/entry/runtime_library/runtime_session/owned_buffer.rs
  - zircon_editor/src/core/gateway/session/gateway.rs
  - zircon_editor/src/core/gateway/session/frame.rs
  - zircon_editor/src/core/gateway/session/output.rs
  - zircon_editor/src/core/gateway/session/protocol.rs
  - zircon_editor/src/core/gateway/session/profile.rs
  - zircon_editor/src/core/gateway/session/plugin_events.rs
  - zircon_editor/src/core/gateway/session/world_sync.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/01/2026-08-17-host-output-policy-convergence.md
  - docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
  - docs/plans/optimize/zircon_runtime_interface/04-profiling-plugin-event-script-diagnostic-manifest-crate-ownership-consolidation-review.md
  - docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Memory/SharedBuffer.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Memory/MemoryView.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/ProfilingDebugging/MemoryTrace.h
  - dev/godot/core/extension/gdextension_interface.json
  - dev/godot/core/extension/gdextension_interface.schema.json
  - dev/godot/core/extension/gdextension_interface_header_generator.cpp
  - dev/godot/core/object/worker_thread_pool.h
  - dev/godot/core/object/worker_thread_pool.cpp
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/Fyrox/fyrox-dylib/src/lib.rs
  - dev/bevy/crates/bevy_asset/src/io/mod.rs
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/CommandBuffers/BaseCommandBufer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/CommandBufferPool.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/ProfilingScope.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 05 · Runtime Host Foreign Output 安全抽象、所有权、Admission、预算、熔断与观测审查

## 1. 结论

`zircon_runtime_host` 是一次有价值的 owner 收敛：App 与 Editor 不再各自维护 JSON byte/item/time budget、runtime-owned allocation release、per-session protocol fuse 和基础计数器；Runtime 的 allocation registry 也已经用 opaque allocation ID 取代旧的可复制 `Vec` metadata。App 的 `RuntimeFrame<'session>` 通过借用保持 session/DLL 存活，Editor 的 frame pixels 通过 `_runtime_owner: Arc<dyn Send + Sync>` 保持 provider 存活；Runtime session destroy 又拒绝存在 outstanding allocation 的关闭。这里已经不是“随手 `serde_json::from_slice`”的临时代码。

但是，该 crate 当前描述为“Safe host-side ownership, decoding, and protocol state”，公开 API 实际没有满足 Safe Rust 的健全性要求。`RuntimeForeignOutputState::decode_json` 与 `ensure_call_succeeded` 是安全函数，参数 `ZrOwnedResultV2`、`ZrStatus` 的裸指针字段却可由任意安全调用方构造；实现随后在内部执行 `slice::from_raw_parts` 或读取 status diagnostics。`RuntimeOwnedOutputReleaser::new` 同样是安全构造器并派生 `Copy`，但保存的 `unsafe extern "C" fn` 可以要求任意未表达前置条件，也没有持有 DLL/provider、session lease 或 generation。调用方不需要写一行 `unsafe`，就能传入地址 `1` 或需要特殊不变量的 release 函数，让这个“安全”crate 触发 UB。

第二个 P0 是 session fuse 的 admission 语义不成立。每个 App/Editor 调用先执行 `ensure_available`/`ensure_session_available`，随后才调用 FFI；这两步之间没有 admission lease。另一个线程可以在检查之后熔断 session，而已经通过检查的线程仍会进入已判定违反协议的 provider。现有 `acceptance_gate` 只保证在途 JSON 结果不会在熔断后记为 accepted，不能阻止在途或刚通过检查的 foreign call 执行。M2.4 产出记录已经把 admission token 列为保留风险，但同时宣称统一熔断完成并记录“Critical 0 / Important 0”；严格工程门下，这个结论不能成立。

输出覆盖也尚未闭合。Runtime API V7 发布 `capture_accessibility_tree`，Runtime producer 按 16 MiB/65,536 items policy 编码并注册 allocation，Interface 也公开该 policy；App、Editor 和 `zircon_runtime_host` 却没有任何 accessibility capture consumer、kind、policy 或 item counter。帧输出虽由两端校验和释放，却混入 `session_protocol`，没有 frame bytes/latency/release/retention 指标；Editor 还允许非零尺寸配 canonical empty RGBA，而 App 明确拒绝，形成同一 ABI 的 consumer 语义分叉。

本轮对 `zircon_runtime_host` 12 个 Rust 文件、1,555 行逐文件审查，并沿 producer/interface/App/Editor 追踪真实调用链。登记 3 项 P0、72 项 P1、16 项 P2，均为 `pending`。既有 Interface01 的 producer budget/ABI identity、Interface03 的 accessibility generation/producer capture、Interface04 的 profile producer exhaustion 继续由原报告 canonical 持有；本文只登记 host-safe abstraction、admission/fuse 和 shared consumer policy 自身新增的根因，不复制既有 P0。

## 2. 审查边界与证据

### 2.1 物理范围与观察点

| 集合 | 文件 / 物理行 / bytes | 指纹与证据等级 |
|---|---:|---|
| `zircon_runtime_host` production | 11 / 1,066 / 36,797 | `c8065c3ddd98809607463d82168969a5285fcb67069cd67e7390e17daf165a19`，E3 全量逐文件 |
| `zircon_runtime_host` tests | 1 / 489 / 17,379 | `637cc17915b780a696a517ab26f6bc21d1971900c72600f93ee4457a58ea81ed`，E3 全量逐测试 |
| App direct consumer | 5 / 1,928 / 70,479 | `8a87ead75239b36e4d6d0fc7091dc15aaaf85e8239be31591b88c6d156dc4cbb`，E3 纵向调用链 |
| Editor direct consumer | 7 / 934 / 32,952 | `d60a8a5d185deff2cd9a8d6d873d365aabc6fd98a9bd69f443bb031f679788bb`，E3 纵向调用链 |
| Interface carrier/policy DTO | 12 / 2,230 / 76,919 | `fa547a36a94330be019d9cd5350f7f8d9a5f2dfd4a91cde3b85f2ec63b50ab4f`，E3 直接依赖闭包 |
| Runtime producer/allocation | 3 / 1,100 / 38,562 | `a6c3e6cc01842f9b2221b8ab24549e3660ddf7ba4c8efd9c33960001f9c5be45`，E3 allocation/capture 主路径 |

指纹按相对路径排序，对每个文件取 SHA-256，再对 `path<TAB>hash<LF>` 清单取 SHA-256。它们只固定本轮静态观察点，不是实施 baseline。成文时 `budget.rs`、`error.rs`、`policy.rs`、`tests.rs`、Interface buffer/status 和 App/Editor consumer 均有其他 Session 的未提交修改，因此本文必须保持 `source_recheck_required: true`。

### 2.2 实际调用链

```text
Runtime producer
  -> bounded encode / frame validation
  -> per-session allocation registry
  -> ZrOwnedResultV2(data, len, allocation)
  -> App RuntimeSession / Editor SessionGateway unsafe FFI call
  -> zircon_runtime_host shape/status/release/decode/fuse
  -> typed App/Editor value or borrowed frame pixels
  -> release_allocation(session, allocation)
  -> runtime census decrement / session teardown gate
```

审查没有只看 host helper。逐项核对了：

1. `ZrOwnedResultV2`、`ZrByteSlice`、`ZrStatus` 与 function pointer 的可构造性；
2. Runtime allocation ID 注册、session owner 检查、release 和 destroy census；
3. App `LoadedRuntime`、`RuntimeSession`、`RuntimeFrame<'session>` 的字段/drop 顺序；
4. Editor `SessionGateway`、`GatewayOwnedOutput`、`SessionRuntimeFramePixels` 的 provider owner；
5. 六类 JSON output policy、七类 metric kind、frame 与 accessibility 非对称覆盖；
6. deadline reader、serde decode、item counter 与 validation 的实际先后顺序；
7. 九个当前 host 单元测试和 M2.4 历史验收记录。

### 2.3 动态证据边界

本轮是 review-only，没有修改或运行生产代码。仓内已有的 M2.4 记录声称 host 8/8、App 17/17、Editor integration 1/1；当前 `zircon_runtime_host` 测试源码已有 9 个 `#[test]`，并存在未提交 policy convergence 修改，所以历史数字不能作为当前 source tree 的 fresh receipt。此前本目标中的 Editor lib validation 又被 239 个既有编译错误阻断；这些事实不影响通过 Rust 可见性和调用签名确认 soundness 缺口，但意味着本文不声称当前分支可编译或动态行为已通过。

坏地址不能在普通进程内“安全试读”。后续验证必须使用 compile-fail/Miri 证明 safe API 无法构造未验证 carrier，并用 child process、guard page、实际 DLL unload/reload 和 sanitizer 隔离故障；不能写一个会让测试进程本身 UB 的普通 unit test。

### 2.4 参考源码给出的基线

- Unreal `FSharedBuffer`/`FUniqueBuffer` 把 data、size、owner、free 行为和引用生命周期放在一个 owner 对象中；外部 view 不能证明寿命时要求 `MakeOwned`。这比“裸 output + 可复制 releaser + 调用方自己记住 DLL owner”更接近目标抽象。
- Unreal Module Manager 在 unload/abandon、pre-unload、shutdown 和 loaded-state 上有显式阶段。它不直接解决 Rust soundness，但说明 function pointer/provider lifetime 必须属于 module lifecycle，而不是一个与 owner 无关的 `Copy` 值。
- Godot GDExtension 通过 schema/generated interface 公布 allocator/free 与 since/deprecated 关系。Zircon 不必照抄 allocator API，但 foreign ownership、版本和生成式验证必须在同一接口体系中。
- Fyrox `DyLibHandle` 明确把 plugin object 放在 library 字段之前，并注明 library “Must be last”，确保 plugin 先析构、library 后卸载；它还明确警告 Rust dynamic plugin 的生产 ABI 风险。Zircon App/Editor 局部采用了相同思路，host crate 公共类型却没有把 owner 编码进去。
- Bevy asset I/O 的 reader 和 task pool 展示了流式读取与受控任务执行的分层。对 16 MiB control payload，Zircon 应把解析工作、取消和内存 budget 交给有 admission 的执行器，而不是把 `Instant` 检查伪装成可抢占 deadline。
- Unity Graphics 的 command buffer wrapper/pool 与 profiling scope 把 valid state、获取/释放和观测放在结构化生命周期中。这里可参考的是“资源 lease 与观测共存”，不是照搬渲染 API。

## 3. 已有可保留基础

1. Runtime allocation registry 以 opaque ID 为唯一 release key，记录 session、kind、bytes 和 high-water census；跨 session release 会返回 not-found。
2. Runtime session close 会阻止普通 action，并在 outstanding allocations 未归零时拒绝成功销毁；App 销毁失败会在 DLL unload 前 abort。
3. App `RuntimeFrame<'session>` 的 `PhantomData<&RuntimeSession>` 与 Editor `_runtime_owner: Arc<dyn Send + Sync>` 都能保护当前主调用路径的 provider lifetime。
4. JSON output 在构造 Rust slice 前检查 null/len/allocation canonical shape、`usize` 转换和 `isize::MAX`。
5. 六类 interface limit 已集中映射到 shared host policy；bytes/items/depth/time/empty 基本语义不再散落在 App 与 Editor。
6. success/failure cleanup 均尝试一次 release，cleanup failure 会提升为 protocol violation 并触发 session fuse。
7. `acceptance_gate` 能保证另一线程熔断后，在途 decode 不会被记为 accepted；并发单测覆盖了这个局部性质。
8. App 与 Editor 多数 foreign-output 调用共享同一个 `Arc<RuntimeForeignOutputState>`，从而避免产品两侧各自维护互不知情的 fuse。
9. metrics 至少区分 accepted/rejected/call failure/blocked、bytes 和 decode time；App drop 会输出一条当前 session 汇总。
10. producer 与 consumer 都使用同一 interface limit 常量，当前 policy test 会检查六类 bytes/items/time/empty 与全局 depth。

## 4. P0：必须先修复的安全与隔离合同

### RHOST-P0-001 · 公开安全 decode/status API 可解引用任意安全调用方伪造的裸指针

`ZrOwnedResultV2` 的 `data/len/allocation` 全部公开，`ZrStatus` 与其 `ZrByteSlice` diagnostics 也可由 Safe Rust 直接构造。`RuntimeForeignOutputState::decode_json` 和 `ensure_call_succeeded` 却是安全方法；前者在形状检查后执行 `slice::from_raw_parts`，后者经 `RuntimeForeignOutputError::from_status` 读取 diagnostics。null、长度和 allocation ID 只能验证 carrier 形状，不能证明地址可读、对齐、已初始化、在读取期间不变或仍存活。任意 Safe Rust 调用方可传 `data = 1 as *const u8, len = 1` 触发 UB，违反 safe abstraction 的基本合同。

应把唯一 raw ABI capture 入口设为 `unsafe` 并写完整 safety contract，立即将 FFI 返回值转换为私有、不可伪造的 `TrustedForeignOutput<'provider, 'session>`；safe decode/release 只接受该 trusted carrier。更稳妥的设计是 capture 函数本身持有 provider/session lease 并完成 out-param 初始化、status copy、ownership adoption，外部永远看不到可随意组合的 raw parts。`validate_owned_result` 可保留为 shape validator，但名称和返回类型不得暗示它证明了 pointer validity。

### RHOST-P0-002 · `Copy` releaser 的安全构造/调用未携带 provider、session lease 与 unsafe function 前置条件

`RuntimeOwnedOutputReleaser::new(session, unsafe extern fn)` 是 `pub const fn`，类型派生 `Clone, Copy`；`release_owned_result` 是安全函数，却无条件在内部调用该 unsafe function pointer。安全调用方可以传入一个要求特殊前置条件的 `unsafe extern fn`，也可以从 `libloading` 取出 symbol、卸载 library 后继续保留 releaser。类型既不持有 `Arc` provider，也不绑定 session generation、lifecycle lease 或 allocation provenance，因而无法证明函数代码仍驻留、handle 属于该 provider、调用线程/重入规则满足或 session 未销毁。

App/Editor 当前上层 owner 降低了主路径暴露，但不能修复公共 crate 的 soundness。应让 `RuntimeProviderLease` 拥有 table/library epoch，`RuntimeSessionLease` 拥有 session/generation，`OwnedRuntimePayload` 同时持有 allocation 与两个 lease；release 在其 `Drop`/显式 `try_release` 内完成。若仍允许外部自定义 provider，创建 trusted provider/releaser 必须是 `unsafe`，且类型不可 `Copy`。

### RHOST-P0-003 · protocol fuse 没有 admission lease，熔断后仍可进入 foreign provider

App/Editor 的调用模型是 `ensure_*()` 成功后再取 function pointer 并执行 FFI。检查与调用之间没有原子 admission token；另一个线程可在两步之间发现协议违规并设置 `protocol_failed`，先前通过检查的线程仍会进入 provider。`acceptance_gate` 只在 decode/release 后决定是否接受结果，无法撤回已经执行的 foreign call，也没有统计/等待在途 call。对于已经返回坏 pointer/ownership 的 provider，“熔断后不再调用”是隔离要求，不只是指标要求。

应建立 `SessionAdmissionGate { Open, Fusing, Fused, Closing }` 和 RAII `ForeignCallLease`：只有 Open 能增加 in-flight；首个 protocol fault 原子转为 Fusing、拒绝新 lease、记录 first-fault receipt，并等待或取消所有 in-flight；quiesce 后进入 Fused。decode/release 也必须属于 lease，session destroy 与 provider unload 共用同一 census。M2.4 的 acceptance test 要从“熔断后一次新调用计数为 0”提升为并发 race、在途取消/quiesce 和 unload gate。

## 5. P1：发布前必须完成的工程化合同

### 5.1 Safe carrier、所有权与生命周期

#### RHOST-P1-001 · `validate_owned_result` 只验证形状却返回裸 `usize`

调用方容易把成功误解为“buffer 已安全”。应返回命名为 `ValidatedShape` 的信息，明确 pointer provenance 仍由 unsafe capture 证明；不要让 safe caller 在此后自行 `from_raw_parts`。

#### RHOST-P1-002 · trusted output 没有不可伪造的私有构造边界

allocation ID、session handle 和 raw output 都是公开字段/构造器，可以任意交叉组合。应由 provider capture 一次性生成 opaque payload owner，禁止外部重组 provenance。

#### RHOST-P1-003 · output 没有 provider/build epoch

相同 session/allocation 数字在 DLL reload 或不同 linked provider 中没有全局含义。owner 必须绑定 provider ID、Build Set ID 与 load epoch。

#### RHOST-P1-004 · output 没有 session generation

raw `u64` session handle 可在未来复用；host state 与 releaser只保存裸值。应绑定 session generation，并在 release/status/metrics 中保留 owner mismatch 分类。

#### RHOST-P1-005 · 只靠“immutable”注释，没有冻结/并发写合同

host 在 release 前直接借用 runtime memory，类型无法阻止 provider worker 修改或提前释放。ABI 必须规定 publication 后不可变、release 前持续有效，并由进程隔离或 trusted provider policy支持。

#### RHOST-P1-006 · release 失败后的所有权状态未定义

host 无论 release 返回什么都丢弃 output handle；第三方 provider 可能在失败时保留 allocation，也可能已释放后报告诊断。接口必须规定 release 是 terminal consume、可重试还是查询式，避免泄漏或二次释放猜测。

#### RHOST-P1-007 · release API 没有线程与重入约束

`release_owned_result` 可从任意线程和 `Drop` 调用，公共合同没有说明 provider 是否 thread-safe、是否允许从 callback/drop/unwind 路径调用。provider capability 必须携带 thread affinity 与 reentrancy policy。

#### RHOST-P1-008 · Drop 只能吞掉 release failure

Editor `GatewayOwnedOutput::drop` 会触发 fuse，但无法把错误返回给调用方；App frame drop只记录 teardown failure state。应保留 typed cleanup fault receipt，并让 session close 明确消费所有异步 drop fault。

#### RHOST-P1-009 · App 与 Editor 各自再实现一次 owned-output wrapper

App `RuntimeFrame`、Editor `GatewayOwnedOutput` 各自构造 slice、持有 owner和处理 drop，shared crate只共享裸 helper。应把真正的 RAII owner 下沉到 host crate，产品层只映射 typed frame/value。

#### RHOST-P1-010 · App 借用 owner 与 Editor `Arc` owner语义不同

同一 ABI 输出在 App 不能越过 session borrow，在 Editor 可以通过 clone owner 延长。应统一说明 payload 是否可跨线程/跨 session operation 存活，并由一个 owner type执行。

#### RHOST-P1-011 · `RuntimeForeignOutputState::default()` 可为同一 session 任意创建多份

“session-wide fuse”不是类型不变量。未来 consumer 可意外创建第二个 state 绕过首个熔断。state 应由 session owner唯一创建并绑定 identity，子 facade只能 clone同一 handle。

#### RHOST-P1-012 · App/Editor shared state 靠手工参数传递

`SessionGateway::new` 接受任意 `Arc<RuntimeForeignOutputState>`，无法证明与 `session` 匹配。constructor 应接收 typed session lease，foreign-output state从 lease派生。

### 5.2 Admission、fuse 与 teardown

#### RHOST-P1-013 · fuse 只有 `bool`，没有阶段状态

无法区分 open、fault observed、draining、fused、closing 和 destroyed，也无法为管理界面解释当前能否 release/quiesce。

#### RHOST-P1-014 · 首个 protocol fault 没有结构化 receipt

state只增加计数并返回当前错误，不保存 first kind、operation、raw status、allocation/session/provider、thread、timestamp或build identity。后续 fused error只能说“prior violation”。

#### RHOST-P1-015 · 后续 protocol violations 被计数为 rejected payload，却不保存原因

多线程连锁故障无法归因。应保存 bounded fault ring或first + aggregate category，避免无限字符串日志。

#### RHOST-P1-016 · `protocol_failures` 实际只统计首次熔断

字段名像“违规次数”，实现仅在 `swap(false -> true)` 时加一。应改名 `fuse_transitions`，另设 protocol fault count。

#### RHOST-P1-017 · runtime call error 与 protocol error决策矩阵不完整

普通 status error不熔断，只有同时出现 ownership/release错误才熔断；未知 status code、坏 diagnostics和 provider panic等类别没有集中 policy。应生成 fail/fuse/quarantine/retry matrix。

#### RHOST-P1-018 · release 在 acceptance gate外执行

decode完成后先调用 foreign release，再获取 gate。这个顺序能避免接受已熔断值，却无法让 release参与同一 in-flight/quiesce census。session close和provider unload必须等待 release结束。

#### RHOST-P1-019 · release failure可在另一个线程熔断时重复改变统计语义

多个并发 cleanup failure会分别记 rejected，但只有一个 protocol failure；没有 causal grouping。admission lease和fault receipt应把 output call/decode/release作为一个 transaction。

#### RHOST-P1-020 · poison recovery无诊断地继续

`acceptance_gate` poison后直接 `into_inner`。当前 critical section很小，但若未来增加 fault receipt/observer，panic可能留下部分状态。应消除可panic操作或把 poison作为fatal invariant fault记录。

#### RHOST-P1-021 · session destroy与host fuse是两套状态机

Runtime registry有 open/closing/action census，host只有 bool；二者无法交换“停止新调用、仅允许 release、等待在途”的阶段。应由 session lifecycle owner统一协调。

#### RHOST-P1-022 · provider reload没有旧 epoch drain

当前 App不做runtime hot reload，但目标引擎需要它。旧 output、callback、release和metric state必须在新 table发布前归零或被隔离，不能只替换函数指针。

#### RHOST-P1-023 · fuse后的资源回收能力没有显式白名单

新普通调用应被拒绝，但 outstanding allocation release、cancel、unsubscribe、surface unbind、destroy等收尾动作仍必须可执行。当前 `ensure_session_available` 分散在调用点，没有生成式 teardown capability表。

#### RHOST-P1-024 · blocked call没有 admission原因与等待策略

所有 blocked 都立即返回字符串错误；没有区分 fused/closing/backpressure/quiescing，也没有 deadline或可观察retry-after。

### 5.3 Budget、decode 与 producer/consumer闭环

#### RHOST-P1-025 · `from_interface` 丢弃每个 policy 的 nesting depth字段

host只保存 bytes/items/time/empty，depth依赖一个全局常量和 serde默认行为。未来某一payload单独调整depth时会静默失效。budget应完整保存并由parser执行。

#### RHOST-P1-026 · nesting limit依赖 serde_json实现默认值

错误消息宣称最大128，但host没有显式配置parser。依赖升级或feature变化可能改变行为；应由测试过的parser配置/visitor强制执行。

#### RHOST-P1-027 · 任意 public duration可让 deadline相加 panic

`RuntimeForeignOutputBudget::new` 接受任意 `Duration`，`Instant + duration` 可能溢出并panic。应验证policy、使用 `checked_add`，并把自定义policy构造限制在受控registry。

#### RHOST-P1-028 · 4 KiB read检查不是可抢占deadline

一次chunk内部的字符串反转义、数值解析、typed allocation和visitor工作不能被打断；超时只能在下一次read或完整结束后发现。

#### RHOST-P1-029 · validation与item count发生在完整反序列化之后

payload已经构造全部Vec/String/Map，再检查items。对复杂profile/world响应，item budget不能防止峰值分配和CPU暂停。

#### RHOST-P1-030 · decode memory amplification没有预算

encoded bytes cap不等于decoded heap cap；字符串、map buckets、typed copies和后续clone都可能放大。需要 allocator/accounted arena或paged typed decode。

#### RHOST-P1-031 · host cap发生在producer完成分配之后

Runtime先生成frame/profile/world/accessibility JSON/bytes并注册allocation，host才看len。canonical producer-side风险继续依赖 Interface01 P0-03、Interface03 P0-02、Interface04 P0-01，本文不能把consumer policy称为端到端资源上限。

#### RHOST-P1-032 · 没有host outstanding bytes/age上限

Runtime有allocation census，但host state不追踪借出frame/output的数量、bytes、age和callsite。慢consumer可长期占用provider内存，直到session destroy失败。

#### RHOST-P1-033 · budget不是握手/能力的一部分

常量编译在interface中；host与runtime不同build即使table shape兼容也可能使用不同limit。需要build identity或negotiated limit table。

#### RHOST-P1-034 · `allow_empty` 混合“无数据”与“合法空文档”

有的page用canonical empty carrier表示None，有的JSON类型可编码空集合。应在response envelope中显式区分NoContent/NotModified/Page/Value，避免null/empty多义。

#### RHOST-P1-035 · item的定义没有公共schema

items有时是delivery，有时是row+JSON node，有时是frame/span/counter，有时是dirty/fact。limit值无法跨版本比较，也难以给运维解释。

#### RHOST-P1-036 · JSON object key不进入item count

`json_value_item_count`只遍历values，忽略key数量/长度的结构成本；world component map同样不显式计key。byte cap能限制总量，但item cap语义不完整。

#### RHOST-P1-037 · profile item count遗漏大量嵌套集合

host只计frames/spans/counters/retention、diagnostic history/tags和几个report Vec；嵌套字符串、labels、字段map或未来扩展不会自动纳入。应由schema生成budget visitor。

#### RHOST-P1-038 · plugin-event item count只计deliveries

每个delivery的raw JSON payload结构完全不计items，最多只受256 KiB encoded cap。若item limit代表parse complexity，应对raw payload流式计数或把payload视为独立opaque bytes并声明不解析。

#### RHOST-P1-039 · operation result只按success JSON tree近似计数

envelope、keys和错误detail的结构成本没有统一模型。operation schema扩展后容易在producer/consumer两侧漂移。

#### RHOST-P1-040 · producer与host各维护一份item-count实现

`zircon_runtime/src/dynamic_api/frame.rs` 与 host `item_count.rs` 分别实现profile/world/accessibility计数。没有共享生成器或cross-test证明相同payload得到相同items。

#### RHOST-P1-041 · host request只计request数量

URI、IME surrounding text、path和嵌套request数据只靠bytes cap；item limit不能表达单request内部复杂度或字符串长度policy。

#### RHOST-P1-042 · world invalidation只计batch/dirty/fact数量

固定字段当前成本小，但未来fact携带数组/map时host不会自动收紧。schema演进必须同时更新预算visitor并有compatibility test。

#### RHOST-P1-043 · 没有分页cursor/continuation的统一host abstraction

world/plugin部分有各自分页概念，profile/accessibility/frame仍整块输出。host应统一page receipt、remaining、cursor generation、resume和backpressure，而不是只统一JSON decode。

#### RHOST-P1-044 · decode结果在release后才被接受，但没有copy/amortization策略

JSON typed value已拥有heap所以可行；frame仍借用foreign bytes。应按payload明确CopyOwned、BorrowedLease、SharedBlob或Stream，而不是让每个consumer自己决定。

### 5.4 Output family覆盖与App/Editor一致性

#### RHOST-P1-045 · accessibility output完全缺少host consumer policy

Interface/Runtime已有API、limit、producer和allocation kind，App/Editor无调用点，host无kind/policy/counter。API table广告的能力无法进入真实OS accessibility bridge或Editor inspection。

#### RHOST-P1-046 · accessibility item counter只有producer版本

即使补consumer，也必须共享nodes/roots/diagnostics/actions/children的计数定义，并验证tree generation/relation完整性；不能临时再复制一份。

#### RHOST-P1-047 · frame被归入`SessionProtocol`

256 MiB frame bytes、capture latency、release latency、dropped/empty frame和GPU readback成本都没有专属metrics/policy，session protocol计数无法回答渲染性能问题。

#### RHOST-P1-048 · App与Editor对empty frame语义分叉

App拒绝width/height为0并要求rgba精确等于`width*height*4`；Editor `ensure_frame_rgba_shape` 对任何empty rgba直接Ok，包括非零尺寸或零维。两端必须共享一个typed frame validator和NoFrame表示。

#### RHOST-P1-049 · frame descriptor仍只有RGBA8隐含合同

没有stride、pixel format、color space、alpha、origin、HDR、content rect或sync fence。此根因由Interface01 P1-16持有，host owner必须消费生成descriptor而不是继续只看len。

#### RHOST-P1-050 · status diagnostics不在output kind/policy registry中

它有4 KiB cap，但没有decode/release/invalid-carrier metrics，坏diagnostics有时经shared error，有时经App/Editor各自`ensure_status`，不会统一触发fuse。

#### RHOST-P1-051 · non-output session calls与frame共用一个kind

tick/event/surface/capability等协议错误和frame ownership错误都落入`session_protocol`，无法按API族定位质量。

#### RHOST-P1-052 · kind数量与index是手写平行结构

常量7、enum、ALL、index、label、policy六项列表由人工保持一致；新增accessibility/frame时容易漏一处。应由一个declarative registry生成。

#### RHOST-P1-053 · policy registry没有证明API table全覆盖

没有测试枚举所有会返回runtime-owned output/status diagnostics的V7 slot并要求owner/policy/kind/validator。M2.4“统一7类”不是API coverage证明。

#### RHOST-P1-054 · capability缺失与policy缺失不可区分

consumer看见`None` slot只报capability missing；无法判断平台不支持、host尚未实现、build裁剪或policy未注册。需要capability truth和reason。

### 5.5 Error、metrics与运维证据

#### RHOST-P1-055 · error只有RuntimeCall/ProtocolViolation两类

budget、decode、ownership、release、stale session、fused、closing、capability和provider crash都压成两类字符串，调用方无法制定retry/quarantine/telemetry policy。

#### RHOST-P1-056 · unknown status code丢失raw值

`ZrStatusCode::from_raw`把未知值映射为Error，日志不再包含原值。应保留raw code和known category，支持前向诊断。

#### RHOST-P1-057 · error没有operation/correlation/provider字段

operation只拼进message；没有typed source、call ID、session、allocation、kind、build或timestamp，难以和Runtime trace关联。

#### RHOST-P1-058 · metrics总量可能回绕

`AtomicU64::fetch_add`对payload/bytes/time长期运行会wrap；单次usize/duration虽然饱和转换，总和不饱和。应使用checked/saturating strategy和overflow counter。

#### RHOST-P1-059 · relaxed多字段snapshot不是一致快照

accepted payloads、bytes、time等独立Relaxed load，可观察到不可能的组合。若只用于近似telemetry应显式标注；若用于验收/熔断诊断需sequence/seqlock或transaction receipt。

#### RHOST-P1-060 · 只有total/max，没有分布与长尾

无法看到p50/p95/p99、budget headroom、payload size bucket和release latency。高复杂引擎需要低成本histogram或trace event，而不是仅max。

#### RHOST-P1-061 · decode time排除了FFI producer与release

用户感知延迟从call开始到release完成，当前只测host JSON parse/validate。frame/profile等最贵的producer和GPU readback完全不在指标中。

#### RHOST-P1-062 · 没有allocation age与retained high-water host指标

Runtime census有high-water，但不进入host snapshot，也没有最老allocation、kind或consumer stack。session close失败时无法定位谁持有输出。

#### RHOST-P1-063 · `reported_len`在32-bit会饱和丢失原始u64

rejected bytes记为`usize::MAX`，无法保留provider实际报告长度。metrics/receipt应保留raw u64及conversion error。

#### RHOST-P1-064 · call failure/rejected/fuse计数语义可能重叠

一个status failure伴随坏ownership会同时增加call_failures和rejected；文档没有说明这些是非互斥维度。应提供transaction outcome枚举并从receipt聚合。

#### RHOST-P1-065 · blocked session call没有kind分解

`blocked_session_calls`只给总数，无法知道surface、event、destroy或gateway create被阻断；operation label应成为有界ID而非自由字符串。

#### RHOST-P1-066 · diagnostic line是无schema的空格文本

没有version/session/provider/build/timestamp、escaping或稳定field registry；只能在App drop时输出。应发布typed snapshot/trace事件，文本只是renderer。

### 5.6 测试、验证与产出证据

#### RHOST-P1-067 · 9个单元测试不能证明safe abstraction

现有测试覆盖正常release、oversize、empty、release failure、depth/time、ordinary call failure、一次并发acceptance、microbenchmark和policy映射；没有compile-fail或Miri证明Safe Rust不能伪造carrier。

#### RHOST-P1-068 · 没有坏指针child-process/guard-page测试

null/nonzero被覆盖不等于地址有效性。应在隔离进程使用guard page/invalid address验证unsafe capture失败分类和宿主崩溃隔离策略。

#### RHOST-P1-069 · 没有实际DLL unload/reload与stale releaser测试

测试函数全是静态进程内符号，无法发现provider先卸载、旧epoch output释放或function pointer悬空。

#### RHOST-P1-070 · 没有真实registry的跨session/replay/double-release集成

host tests使用自己的全局HashMap fake；Runtime registry测试与host owner未形成一个DLL/linked provider纵向gate。

#### RHOST-P1-071 · 并发测试只证明“熔断后不accept”

一个Barrier场景没有覆盖check-call race、release与destroy并发、多个fault、closing、cancel、unload或高并发stress/loom模型。

#### RHOST-P1-072 · 历史M2.4证据与当前源码不具fresh currentness

记录写8/8和“Critical 0 / Important 0”，当前文件已有9个测试且soundness/admission问题仍在；性能数字没有machine-readable source/build/hardware artifact。实施前必须用Tooling07/10的证据模型重建receipt，不能沿用结论。

## 6. P2：主重构中一并收敛

### RHOST-P2-001 · crate级文档只有一句module说明

缺少trust boundary、owner图、safe/unsafe示例、threading、fuse和teardown合同。

### RHOST-P2-002 · `RuntimeOwnedOutputReleaser::session(self)`消费一个Copy值

API形状暴露了“这是轻量token”而非owner语义；新lease类型应借用或返回typed identity。

### RHOST-P2-003 · state/releaser没有有界Debug实现

调试器无法安全查看session/provider/fuse摘要；Debug又不能泄露裸地址或无限diagnostic。

### RHOST-P2-004 · operation参数被限制为`&'static str`

动态call ID无法进入error；应使用generated operation ID并把display name留给renderer。

### RHOST-P2-005 · kind label为人工字符串

重命名会破坏日志字段且无schema version。应由registry生成稳定numeric/string ID。

### RHOST-P2-006 · metrics snapshot不能遍历公开kind集合

`ALL`和by_kind都是私有，外部只能逐个已知kind查询。typed exporter需要稳定iterator/schema。

### RHOST-P2-007 · test文件已达489行

后续增加soundness/concurrency/policy会继续膨胀；按ownership、decode、admission、metrics、performance拆分。

### RHOST-P2-008 · microbenchmark混在普通unit tests

wall-clock断言受机器负载影响；移到criterion/受控perf harness，unit test只验证逻辑预算。

### RHOST-P2-009 · App `owned_buffer.rs`重复shared helper组合

映射错误类型可保留薄adapter，但release-after-error/result组合应只存在一个owner实现。

### RHOST-P2-010 · `serde_json`版本未从workspace依赖继承

host、App、Editor、Runtime等分别写相同版本/feature，容易出现parser行为与RawValue feature漂移；应收敛workspace policy。

### RHOST-P2-011 · error `Clone`会复制任意长度message

当前message通常有界但接口未保证。typed receipt加有界detail后再决定是否Clone。

### RHOST-P2-012 · diagnostic renderer每次构造多个String

只在drop时影响不大；结构化sink后可直接写formatter，避免额外Vec/String。

### RHOST-P2-013 · module export面过宽

shape helper、release组合、item counter、budget constructor全部public。应区分provider integration API与internal policy implementation。

### RHOST-P2-014 · 没有feature/capability文档说明linked与dynamic provider差异

linked provider不存在DLL unload，但仍有session/unsafe函数前置条件；文档必须分别陈述，不要用dynamic风险覆盖全部场景。

### RHOST-P2-015 · policy常量名没有schema generation信息

V1 limit与payload schema/version的关系只靠命名。registry应携带policy ID、schema ID和compatibility window。

### RHOST-P2-016 · Interface index仍把host policy描述成主要闭合

索引应改为“共享consumer policy已建立，safe owner/admission/accessibility仍待修”，避免后续工程师误判完成状态。

## 7. 目标架构

### 7.1 类型边界

建议形成四层，而不是继续给裸helper加参数：

```text
RuntimeProviderLease
  owns: loaded/linked provider, API table, build/load epoch, unload gate
    -> RuntimeSessionLease
       owns: session generation, admission state, in-flight/release census
         -> ForeignCallLease
            owns: operation ID, deadline/cancel, trace correlation
              -> OwnedRuntimePayload<TPolicy>
                 owns: raw allocation, provider/session leases, kind/policy, release state
                   -> decode/copy/borrow/stream typed value
```

- 只有 `unsafe fn adopt_ffi_output(...)` 能把 raw out-param和status转成trusted payload，safety contract要求来自当前provider、指针在lease内有效且不可变。
- 所有safe decode/bytes/release只接受trusted payload；public caller不能再传raw pointer/function pointer。
- `OwnedRuntimePayload`不可Copy，drop顺序保证先release allocation、后释放session lease、最后允许provider unload。
- status diagnostics在unsafe capture期间立即有界copy到host-owned inline/Vec，不把foreign borrow带入safe层。

### 7.2 Admission与fuse

```text
Open --first fault--> Fusing --in-flight=0--> Fused
  |                       |                     |
  +--close request------> Closing <------------+
                              |
                      releases/cancel/destroy only
                              |
                           Destroyed
```

每次FFI调用必须先取得`ForeignCallLease`。fault transition原子拒绝新lease；已有lease完成、取消或被隔离后才进入Fused。release属于特殊cleanup lease，在Fusing/Closing仍允许。first fault保存typed receipt，后续fault只做bounded aggregation。Runtime registry与host gate应共享session generation和census，不再维护互不知情的两个状态机。

### 7.3 Policy registry

建立单一declarative registry，至少覆盖：

| Family | Carrier | Budget/validator | Consumption |
|---|---|---|---|
| Status diagnostic | copied bytes | 4 KiB、UTF-8/lossy policy、raw code | immediate |
| Frame | owned blob + descriptor | dimensions、bytes、format/stride/color/sync | borrowed lease / async readback |
| Accessibility tree | paged typed JSON/DTO | bytes/items/depth/time/generation/relations | host OS bridge + Editor inspect |
| Host requests | page | bytes/items/string sublimits | App dispatch |
| Profile | paged observation stream | producer+consumer+retained budget | Editor profiler/artifact service |
| Operation result | typed envelope | schema/items/deadline | App/Editor operation owner |
| Plugin events | cursor page + continuity | deliveries/bytes/payload policy | typed subscriber |
| World query/invalidation | cursor/generation page | rows/facts/JSON structure | Editor projection |

registry生成kind、stable label、interface limit、producer/consumer item visitor、API slot coverage test、metrics schema和documentation。Frame/accessibility不能继续留在`SessionProtocol`或只存在producer一侧。

### 7.4 预算执行

1. request/admission前验证协商limit和session retained budget；
2. producer在分配/遍历/encode前执行bytes/items/deadline/cancel；
3. 大payload使用page/stream/shared immutable blob，避免完整DOM与重复Vec；
4. host decode使用受控执行器、accounted allocator或typed visitor，deadline不是事后`elapsed`；
5. receipt同时记录producer、transfer、decode、validation、release各阶段耗时与bytes/items；
6. 超限返回typed `LimitExceeded { dimension, observed, allowed, stage }`，由policy决定是否fuse。

## 8. 重构里程碑

### M0 · 冻结错误完成声明并建立soundness reproduction

- 更新Interface index和M2.4 currentness，明确shared consumer policy完成但safe abstraction/admission未完成；
- 增加compile-fail/Miri harness，证明当前safe API可构造坏carrier，测试本身不得在主进程执行UB；
- 固定当前API slot/output family manifest和source/build指纹。

### M1 · Unsafe capture与RAII owner hard cutover

- 引入provider/session/call lease和不可伪造`OwnedRuntimePayload`；
- 将raw adoption集中为一个unsafe入口；
- 删除public safe raw decode/release/releaser constructor；
- App/Editor frame/JSON/status全部迁移，旧helper无兼容层删除。

### M2 · Admission/fuse/teardown统一

- 实现Open/Fusing/Fused/Closing/Destroyed状态机和in-flight census；
- 首fault receipt、cleanup lease、cancel/deadline、destroy/unload gate闭合；
- 用并发model/stress与实际DLL测试check-call、fault-release、destroy-unload race。

### M3 · Output family与policy registry闭合

- 生成frame/accessibility/status及六类JSON的kind/policy/validator/item visitor；
- App/Editor接入accessibility capture，建立OS bridge/Editor inspection的能力truth；
- 统一App/Editor frame NoFrame/descriptor/empty规则。

### M4 · Producer/consumer bounded pipeline

- 依赖Interface01/03/04 canonical P0完成producer admission；
- profile/world/accessibility等大输出分页/流式化；
- decode执行器执行memory/time/cancel预算，移除事后deadline伪保证。

### M5 · Observability与资格门

- typed outcome/fault/allocation receipt、stage latency、retained high-water和build/session identity；
- Miri/ASan/UBSan/fuzz/child process、跨语言/32-bit布局、旧新build skew与DLL unload矩阵；
- 受控机器性能基线输出machine-readable artifact，不在普通unit test用墙钟p99决定稳定性。

## 9. 验收门

1. Safe Rust无法构造或解引用任意foreign pointer；所有raw adoption都在有文档的unsafe边界。
2. 任意payload owner持有provider+session generation，provider unload前outstanding call/output/release census必须为0。
3. 首个protocol fault后不能再取得普通call lease；并发已通过检查但未进入FFI的线程必须被阻止。
4. Fusing/Closing阶段仍能执行明确白名单中的release/cancel/unbind/destroy，且有deadline和receipt。
5. API V7每个foreign output/status slot均映射到kind、policy、validator、owner和coverage test；未知slot/缺policy编译或测试失败。
6. Accessibility capture从Runtime到App/Editor host真实消费闭合，generation、relations、limit与OS bridge有产品测试。
7. App与Editor对frame ABI/NoFrame/dimensions/bytes/descriptor语义完全一致。
8. producer在工作前执行bytes/items/time/depth/retained budget；host cap不再被宣称为producer保护。
9. decode与item validation受内存/取消/deadline控制，超时不会等待完整DOM构造后才报告。
10. release失败语义冻结；所有success/error/drop路径一次且仅一次消费allocation，并可审计。
11. fault/metric receipt包含operation、kind、session/provider/build epoch、raw status、stage latency和allocation信息，计数不回绕且一致性语义有文档。
12. Miri/compile-fail、guard-page child process、fuzz、并发model/stress、实际DLL unload/reload、跨session/replay/double-release和32/64-bit/C consumer gate通过。
13. 性能receipt固定source/build、机器、配置、payload分布与原始结果；不得只引用一次console p99。
14. Interface01/03/04相关canonical P0验收后，本文M4才可标记完成；consumer policy不能替代producer和产品能力闭环。

## 10. 依赖与owner

| 责任 | Canonical owner | 本文关系 |
|---|---|---|
| ABI build identity、carrier、frame descriptor、producer budget | Interface01 | 依赖；本文不重复其P0-03/P0-04 |
| Accessibility generation、capture producer与UI publication | Interface03 + Runtime11A | 依赖；本文新增host consumer/policy覆盖 |
| Profile producer exhaustion与observation session | Interface04 | 依赖；本文负责host owner/decode/admission |
| Session task/quiesce/destroy | Runtime01/02 + App01 | 共同owner；host admission必须接入其lifecycle |
| Performance/evidence artifact | Tooling07/10 | 共同owner；替换历史console receipt |
| Safe foreign owner与shared policy | `zircon_runtime_host` | 本文canonical owner |

实施顺序必须是M0 -> M1 -> M2 -> M3；M4等待三个既有Interface canonical P0的producer工作。不要先补一个accessibility `decode_json`调用然后继续沿用safe raw API，也不要通过给`RuntimeOwnedOutputReleaser`再塞一个裸`Arc<dyn Any>`临时掩盖生命周期。目标是让不变量进入类型、admission和session lifecycle，而不是依赖调用方记住若干注释。
