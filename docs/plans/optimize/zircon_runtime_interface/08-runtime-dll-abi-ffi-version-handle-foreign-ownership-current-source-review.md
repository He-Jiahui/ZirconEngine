---
related_code:
  - zircon_runtime_interface/src/runtime_api/mod.rs
  - zircon_runtime_interface/src/runtime_api
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/handles.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/version.rs
  - zircon_runtime_interface/src/profiling.rs
  - zircon_runtime/src/dynamic_api
  - zircon_app/src/entry/runtime_library
  - zircon_runtime_host/src/foreign_output
  - zircon_editor/src/core/gateway/session
tests:
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
  - zircon_runtime_interface/src/tests/runtime_owned_result.rs
  - zircon_runtime_interface/src/tests/runtime_operation.rs
  - zircon_runtime/src/dynamic_api/tests
  - zircon_runtime/src/dynamic_api/session/tests
  - zircon_runtime/src/dynamic_api/session/registry/tests.rs
  - zircon_runtime/tests/runtime_owned_result_v7.rs
  - zircon_runtime_host/src/foreign_output/tests.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/entry/runtime_library/runtime_session/tests.rs
  - zircon_editor/src/core/gateway/session/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
  - docs/plans/optimize/zircon_runtime_interface/07-contract-certification-abi-layout-version-skew-cross-language-fuzz-test-architecture-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleInterface.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManifest.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManifest.cpp
  - dev/godot/core/extension/gdextension_interface.json
  - dev/godot/core/extension/gdextension_interface.schema.json
  - dev/godot/core/extension/gdextension_interface_header_generator.cpp
  - dev/godot/core/extension/gdextension_interface.cpp
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/godot/core/extension/gdextension_library_loader.cpp
  - dev/bevy/crates/bevy_dylib/src/lib.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/Graphics/.yamato/wrench/api-validation-jobs.yml
  - dev/Graphics/Packages/com.unity.shadergraph/Tests/Editor/IntegrationTests/SerializationTests.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: partial_foundation_only
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
source_recheck_required: true
---

# 08 · Runtime DLL ABI / FFI / Version / Handle / Foreign Ownership 当前源码复核

## 1. 结论

Interface01 指出的风险没有“全部未实施”，但也远未达到可发布的工程级 Runtime DLL 边界。当前源码已经从 V6 推进到冻结的 V7 exact-table 合同，并用 session-scoped opaque allocation registry 替代了主 Runtime 输出路径上的 `Vec::from_raw_parts` 释放；输入 slice 有统一形状与预算检查，JSON 读写有字节、条目、嵌套和处理时限，frame 有 16,384 维度与 256 MiB RGBA 上限，session handle exhaustion 也已 fail-closed。这些是必须保留的真实底座。

然而，`Stable ABI` 仍是超前承诺。当前握手只有 table version/size/function pointer，没有 Build Set、target/data model、feature/schema fingerprint、artifact digest、能力与限制协商；host table 只检查 version；多数 carrier 没有 size/reserved/extension header；没有 InterfaceSpec、生成 C/C++ header、独立 consumer、真实 built DLL 正向资格、历史 skew matrix 或跨编译器布局证据。旧 `ZrOwnedByteBuffer` 仍以 `Clone + Copy` 公布在 host fetch callback 中，即使 App 当前传入 `None`，也不能把残留的公开 UB surface 算作关闭。

生命周期同样没有收口。Runtime 为 outstanding allocation 设计了“destroy 返回失败 -> release -> retry destroy”的安全恢复路径，但 App 的 `RuntimeSession::drop` 在第一次 destroy 失败时立即 `abort()`；destroy 对 action 与 wake callback 使用无期限 `Condvar` 等待。Editor 的 `SessionGateway::new` 现已在复制 table 前精确校验 V7 `size_bytes`，但仍接受调用方提供的 raw table，缺少 loader 构造的 validated token、BuildSet 与 library identity。Interface 现拥有 exact RGBA shape validator，Editor 与 App 均已直接消费它；跨 consumer shared corpus 与受管验证仍未形成证据。这些不是临时小修可以覆盖的缺口，而是 ABI 身份、所有权、关闭协议和消费端规范必须共同重构。

本轮复核 Interface01 的 43 项原始差距：**27 Open、14 Partial、2 Closed**；另新增 **3 项 P1**，其中 P1-32 已具备 exact-size 检查但仍缺 validated token，P1-33 已完成 Interface、Editor 与 App 的 shared validator migration，但仍缺 shared corpus 与受管验证。合并后当前账本为 **5 项 P0、33 项 P1、8 项 P2，共 46 项：28 Open、16 Partial、2 Closed**。`Partial` 只表示存在可保留基础，不表示风险可接受；5 项 P0 中仍有 2 Open、3 Partial、0 Closed。

## 2. 审查边界与证据

### 2.1 冻结范围

| 集合 | 文件 / 行 / bytes | 当前观察点 SHA-256 | 证据等级 |
|---|---:|---|---|
| Interface ABI declarations | 17 / 3,699 / 120,935 | `7dbff74fa25d8b20f2f97c02a36e049c809f993abea3d10559f2100af5ae3c9f` | E3：table、carrier、buffer、status、handle、version逐文件审读 |
| Runtime dynamic producer | 47 / 12,432 / 454,352 | `454ca7f9cdfee411b84c1fa734e963c824207d6ded64c5b7259a8697b15fc9e8` | E3：export、session registry、allocation、payload、event、frame、surface与operation控制流 |
| App loader/session consumer | 11 / 2,576 / 92,983 | `430039d6a5630ace1d3b60fd7d5ccdb1218931c836117d0cf79e85439e630e8d` | E3：library owner、table validation、session/drop、foreign output |
| shared Runtime Host owner | 10 / 1,240 / 41,904 | `cb507ae7f6d619bc212c69e213f9038d05c7eb487ee9dc5d6d5401790d25e0c4` | E3：admission、budget、release与fuse |
| Editor session gateway | 12 / 1,283 / 45,022 | `9537f2c7653936a7a0025a73c020a18282b53f23d7f91f2ce96ba8e2a54fef78` | E3：owner lifetime、API调用、decode、frame与protocol校验 |
| production union | 97 / 21,230 / 755,196 | `b80491cb3e2901c8ca2aee3d6700eaa91119c45015c56a1e628a5ae5c898d341` | 上述生产集合去重 |
| focused tests | 41 / 10,995 / 393,041 | `7e68307309a952ba744ccee9a0bf53dfd4c63f0a3d381b12e50d7d7e334b2be9` | E2/E3：layout/source guard、registry、loader、consumer与lifecycle测试 |
| Cargo/build surfaces | 6 / 513 / 15,592 | `2a75d93a7c4d2bc9b270459a3a79492b1c83cf69bad2e5356f476b92ef9cbde2` | E2：crate-type、features、linked/dynamic依赖与build入口 |
| Zircon union | 144 / 32,738 / 1,163,829 | `d4fb5ae3fbbd9e511f8d57be8e921ca89ea675ca79f12817c8f8304ce205484c` | 本轮可重建冻结集合 |
| reference engines | 16 / 17,531 / 637,269 | `242067c4408c619c4a440ed1a29c6680f1d73e0832fadea574e7dda0aec2cc98` | E2/E3：身份、接口生成、生命周期、动态库边界与package qualification |

指纹算法统一为：workspace 相对路径转 `/` 并小写，逐文件计算 lowercase SHA-256，按 path ordinal 排序，再对每行 `path + NUL + hash + LF` 的 UTF-8 字节流计算 SHA-256。Interface01 使用的是另一种 `path<TAB>hash<LF>` 算法，因此旧、新总指纹不能直接比较。本轮基线提交为 `79f64878f3b9526517644c055ad3bf5cadfccd0f`，观察日期为 2026-08-24。

### 2.2 并发修改与排除项

本轮观察时 `zircon_runtime/src/dynamic_api`、`zircon_runtime_interface/src/runtime_api/constants.rs`、`events.rs`、`profiling.rs` 等存在其他 Session 的未提交修改。本文冻结的是当前 worktree 观察点，不覆盖、不回退这些修改，因此保持 `source_recheck_required: true`。

serialization direct-decode 与 `math` 正由其他计划处理，本轮不修改也不重新拥有其差距。Interface07 已拥有真实 DLL、cross-language、version-skew、corpus/fuzz 的完整认证架构；本文只把这些资格作为 ABI 实现的依赖 Gate，不重复计算 Interface07 的 1 P0/48 P1/12 P2。

### 2.3 动态证据边界

本轮是 review-only，没有修改 production/tests，没有运行 Cargo、构建真实 DLL、编译 C/C++ consumer、执行 Miri/sanitizer/fuzz、unload/reload、hang/guard-page 子进程或性能基准。`audit_runtime_structure.py --json` 与普通模式均成功返回但没有输出，只记为工具调用，不记为通过证据。静态源码足以确认表布局、指针检查顺序、allocation registry、销毁状态机与消费端策略；它不能证明外部二进制、线程故障和平台 ABI 已合格。

## 3. 当前真实底座

1. `ZrRuntimeApiV7` 明确冻结 exact version/size，包含 version、size 与 23 个函数 slot；App loader 检查返回指针对齐、V7、精确大小和 required slot。旧 V6 及更早 API 被 source contract 硬删除。
2. `ZrOwnedResultV2` 不再 `Copy/Clone`，只暴露 data、固定宽度 len 与 opaque allocation ID；Runtime registry 校验 session owner、拒绝 duplicate/forged/cross-session release，并维护 outstanding/high-water census。
3. `ZrByteSlice::checked_slice(limit)` 在构造 Rust slice 前拒绝 null+nonzero、`isize::MAX` 越界和接口预算越界。dynamic API 主要 JSON 输入已走 bounded reader/writer。
4. frame dimensions、RGBA bytes、JSON bytes/items/nesting 和部分 processing deadline 已在 Interface 中共享，producer 与 consumer 不再完全各自猜测上限。
5. session registry 使用 checked monotonic handle 分配，`u64::MAX` 后永久 exhaustion，不再 wrap 到有效 ID。
6. App/Editor/Runtime Host wrapper 持有 provider owner，并把 foreign output release 绑定到 session 和 allocation ID；App `RuntimeFrame<'session>` 也阻止常规 Rust 路径让 frame 越过 session lifetime。
7. Runtime 所有 extern entry 通过 panic boundary；App wake callback trampoline 也阻止 Rust panic 穿越 C ABI。
8. Interface 测试固定当前 Rust layout、字段 inventory、退役 symbol 与 unsafe signature 规则；这些可作为未来生成 manifest 的输入，但还不是跨语言资格。

## 4. Interface01 原始差距复核

状态定义：`Open` 表示目标合同未建立；`Partial` 表示已有实质修复但风险或全路径尚未闭合；`Closed` 表示旧 finding 的具体问题已由当前源码消除。任何 `Partial` P0 仍按阻断项处理。

### 4.1 P0

| ID | 当前状态 | 当前源码判断 |
|---|---|---|
| P0-01 owned buffer 可复制/伪造释放 | Partial | 主 Runtime 输出已迁移到 `ZrOwnedResultV2` + opaque registry；但公开 `ZrOwnedByteBuffer` 仍为 `Clone + Copy`，host fetch callback 仍可携带该类型。App 当前传 `None` 只能说明路径休眠，不能删除 UB surface。 |
| P0-02 未校验 slice/status 即构造 Rust slice | Partial | `checked_slice(limit)` 已关闭常见 null/len/`isize::MAX` 问题；但任意可读地址无法在同进程证明，out pointer 只检查 null，Runtime host table 在解引用前也未检查对齐/size，status 仍是借用 pointer。 |
| P0-03 producer-side OOM/DoS | Partial | frame 与 bounded JSON 已有共享上限和阶段检查；world/accessibility/profile 等仍可先构造完整 domain object/Vec 再编码，256 MiB 合法请求仍依赖不可恢复 allocator，deadline 只是 cooperative checkpoint 而非取消。 |
| P0-04 缺少 Build Set/兼容身份 | Open | V7 仍只验证 version/size；没有 target、pointer model、feature/schema fingerprint、artifact hash、dependency set、capability/limit handshake。 |
| P0-05 destroy/quiesce 无 deadline/cancellation | Open | action 与 wake callback drain 仍是无期限 `Condvar::wait_while`，没有 deadline、cancel、escalation receipt 或隔离进程终止策略。 |

### 4.2 P1

| ID | 当前状态 | 当前源码判断 |
|---|---|---|
| P1-01 exact table 与 optional-tail 矛盾 | Closed | 当前明确选择 frozen exact V7，App 要求精确 size；残留 field-availability helper 可简化，但不再构成版本策略矛盾。 |
| P1-02 host table size/callback 不参与支持判断 | Open | Runtime 仅读取 `abi_version`，忽略 `size_bytes`、对齐、callback presence 与能力；App 传入两个 `None` callback。 |
| P1-03 capability 只靠 null slot | Open | 没有 capability set、依赖/互斥、limit table、availability reason 或协商 receipt。 |
| P1-04 carrier 缺少 size/reserved/extension header | Open | 多数 request/event/config 仍只有 version，只有少量 V2 carrier 有 reserved 字段。 |
| P1-05 API/ABI/schema version 命名混用 | Open | table `abi_version` 实为 API family version，carrier 与 payload version 仍未分层。 |
| P1-06 `usize` 与 data model 未握手 | Partial | 主 owned result 长度已用 `u64`，但 `ZrByteSlice`、host table size 等仍含 `usize`，target/endian/pointer width/alignment 仍未协商。 |
| P1-07 无生成 C/C++ header/IDL/consumer | Open | workspace 未发现 Runtime ABI 的 C/C++ header、InterfaceSpec 或独立非 Rust consumer。 |
| P1-08 无原子 runtime/host manifest | Open | App 仍按路径加载 DLL，没有 Build Set、artifact digest、dependency set 与原子安装验证。 |
| P1-09 status detail 所有权/编码不自描述 | Partial | diagnostics 有 4 KiB 上限且 Runtime 使用 thread-local buffer，文档要求同步读取；carrier 仍无 encoding、detail kind、truncation、correlation ID 与可携带 lifetime。 |
| P1-10 unknown status code 被压缩 | Open | `ZrStatusCode::from_raw` 仍把未知 raw value 映射成通用 `Error`。 |
| P1-11 JSON 无统一 envelope/schema identity | Open | 各函数族仍各自传 JSON，没有统一 content type、schema hash、encoding/compression 与版本 envelope。 |
| P1-12 out parameter 失败初始化规则 | Partial | 多数实现会先写 empty/default，但规则没有由生成 contract 覆盖全部 entry，misaligned/unwritable out pointer 仍属于同进程信任边界。 |
| P1-13 预算属于局部 consumer | Partial | frame/若干 payload limit 已共享；仍无握手 limit table、per-request negotiated budget、required/truncated/actual usage receipt。 |
| P1-14 decode time fuse 事后报告 | Partial | bounded reader/writer 会在流式阶段检查 processing deadline；业务验证与对象构造仍是 cooperative，不能取消阻塞或长计算。 |
| P1-15 大对象整块驻留 | Partial | plugin event 有分页基础；world/profile/accessibility/host request 等仍主要使用完整对象 + JSON + Vec，没有统一 stream/chunk/two-call/host allocator。 |
| P1-16 frame pixel contract 不完整 | Open | V2 仍只有 width/height/generation/RGBA allocation，没有 stride、format、color space、alpha、origin、HDR、content rect。 |
| P1-17 event 扁平过载 | Open | wheel float bits 仍复用 key/scan 字段，window move 仍经 f32；没有 typed union/header/device/window identity。 |
| P1-18 event 数值/enum 验证不一致 | Partial | 多个 enum/float 已严格拒绝坏值；unknown keyboard action 仍返回 success，pointer/touch 等也没有统一生成 validator。 |
| P1-19 surface 仅 None/Win32 | Open | 无 Wayland/X11/macOS/Web/mobile descriptor、ownership、thread affinity、recreate contract。 |
| P1-20 viewport 固定默认 handle 1 | Open | create/destroy/enumerate 与 generation-qualified multi-viewport contract 仍不存在。 |
| P1-21 callback thread/reentrancy/no-throw 不完整 | Open | 只局部阻止 wake callback 同步销毁；没有线程、并发、重入白名单、阻塞上限、userdata lifetime 的机器可检验合同。 |
| P1-22 handle 缺 generation/owner/DLL epoch | Partial | allocation ID 由 registry 校验 session owner；session/viewport/plugin/operation/subscription 等仍是透明 `u64`，缺 reload epoch 与统一 mismatch 分类。 |
| P1-23 session handle wrap/collision | Closed | checked monotonic allocator 在最大值后永久 exhaustion，并有行为/性能测试。 |
| P1-24 foreign allocation 观测/泄漏核算 | Partial | 已有 per-session outstanding bytes/count 与 high-water，teardown 会拒绝 leak；仍缺 age、call site、统一 dump/receipt、跨 owner 诊断。 |
| P1-25 unload/reload 产品状态机 | Open | 没有 begin-drain、epoch rollover、atomic replacement、compatibility decision；library owner 只能防明显提前卸载。 |
| P1-26 API table/常量人工维护 | Open | V7 table、function typedef、numeric namespace与 doc/test inventory 仍由手工同步。 |
| P1-27 layout test 仅当前 Rust target | Open | 无 MSVC/Clang/GCC、32-bit、packing、endianness 的 C static assertion matrix。 |
| P1-28 无 old/new skew matrix | Open | 没有冻结旧 DLL/header/manifest artifact，也没有 must-accept/must-reject 双向矩阵。 |
| P1-29 unsafe carrier 缺 sanitizer/fuzz/fault lane | Partial | null/misalignment、duplicate/forged/cross-session allocation 和并发 release 有 Rust 测试；仍无 ASan/UBSan/Miri、guard-page、hang、bad-address 和 child-process fuzz 资格。 |
| P1-30 ABI 发布无变更流程 | Partial | source guard 会硬删除旧 symbol/version，V7 已明确 hard cut；仍无 ABI diff artifact、owner approval、changelog/deprecation、symbol/schema release gate。 |

### 4.3 P2

| ID | 当前状态 | 当前源码判断 |
|---|---|---|
| P2-01 moved 坐标经 f32 损失 | Open | 仍未改为固定宽度整数 typed payload。 |
| P2-02 constructor 接受任意 ABI version | Open | 多个 public constructor 仍允许任意 version，坏版本到远端才拒绝。 |
| P2-03 translated event 命名混淆 | Open | Rust-only wrapper 与 ABI carrier 的命名/namespace 仍未由清单隔离。 |
| P2-04 默认 viewport `1` 扩散 | Open | App teardown 与多处产品调用仍直接构造 handle 1。 |
| P2-05 unknown input 有两套语义 | Open | keyboard unknown 仍“忽略成功”，其他 event family 多为错误。 |
| P2-06 optional free 的空/静态/owned 语义 | Open | V7 主输出已改善，但公开 legacy buffer 仍以 null/free 组合推断 storage kind。 |
| P2-07 无 machine-readable 字段文档链接 | Open | 函数/字段没有稳定 ID、since、ownership、threading、errors 的生成元数据。 |
| P2-08 `Stable ABI` 文档超前 | Open | crate 顶层与 runtime API 文档仍直接宣称 stable ABI。 |

## 5. 本轮新增差距

### P1-31 · Runtime 的 allocation teardown retry 与 App 的一次失败即 abort 互相冲突

Runtime 在 session 进入 `Closing` 后拒绝 release；等 action drain 完成，如仍有 allocation，则转入 `TeardownRetryPending` 并返回 teardown incomplete。此后 release 被重新允许，调用方应 release 后再次 destroy。registry test 正式验证了这条两步恢复路径。App `Drop` 却在第一次 destroy error 后立即 `std::process::abort()`，没有机会执行 retry；安全策略因此以进程可用性为代价，而且不同 consumer 可能采用不同关闭语义。应把 close 设计为显式 `begin_close -> drain/return outstanding census -> release/force-retire policy -> finalize_close`，由 App 在进入 Drop 前完成；Drop 只执行已证明不会阻塞或需要重试的终局动作。

### P1-32 · Editor `SessionGateway::new` 的“validated table”前置条件与实际检查不一致

构造器现已检查 session、`api.abi_version`、`api.size_bytes == size_of::<ZrRuntimeApiV7>()` 与 `release_allocation`，因而同版本但错误大小的手工表会在复制前被拒绝。Safety 文档仍只要求 owner 保持 library loaded，public unsafe constructor 仍允许调用方提供一个形状正确却不属于当前 loader/BuildSet 的 raw table。应让唯一 `ValidatedRuntimeApiV7` token 由 loader/linked-provider validator 构造，Editor 只能接收 token，并将 provider/build identity 纳入其不变量；当前 exact-size 检查只将该项降为 Partial。

### P1-33 · Frame payload consumer validation 尚缺共享语料与受管证据

Interface 以 `validate_runtime_frame_rgba_shape(width, height, rgba_len)` 统一正尺寸、维度上限、RGBA byte budget 与 exact length；Editor 的 `ensure_frame_rgba_shape` 与 App 的 `validate_runtime_frame` 都只将该 typed validator 的结果映射到各自边界错误，非零 width/height 配空 RGBA 已在两个 consumer 一致拒绝。V2 当前 producer 成功路径没有 no-frame carrier，必须保持正尺寸和 exact payload。P1-33 保持 Partial，直至同一组 valid、empty、overflow 与 exact-length 语料穿过两个 consumer 的受管测试门。

## 6. 参考引擎差异与适用边界

| 参考 | 可学习的工程机制 | Zircon 当前差异 | 不能误抄的部分 |
|---|---|---|---|
| Unreal | `ModuleManifest` 保存 BuildId、module filename 与 library dependencies；ModuleManager 在 Startup 完成后才发布 ready，支持 PreUnload、Shutdown、reverse load order、unload/abandon 与动态重载声明 | Zircon DLL 没有 Build Set manifest、dependency/artifact identity、显式 readiness 和分阶段 unload receipt | Unreal module API 是其内部 C++生态，不等于通用 C ABI；不能只复制类接口 |
| Godot | JSON/schema 驱动接口生成，统一 `get_proc_address`，导出 engine version/hash；extension config 声明 compatibility min/max 与 reloadable，manager 跟踪实例并重载 | Zircon table/carrier/header/doc/test inventory 全手写，无兼容窗口、版本 hash、实例/reload authority | Godot 的宽接口与兼容政策有自身历史负担，Zircon 应先定义内部 lockstep 与公开 SDK 两类边界 |
| Bevy | plugin 有 build/ready/finish/cleanup 生命周期；`bevy_dylib` 明确主要用于缩短开发构建并警告 release 分发成本 | Zircon 目前把 Rust-linked provider 与可发布 Runtime DLL 的证明混在一起 | Bevy dylib 不是稳定外部 ABI 参考，不能作为 Zircon release qualification |
| Fyrox | dynamic plugin 显式使用 Rust `fn() -> Box<dyn Plugin>` 和 `libloading::Library`，为 hot reload 复制库并谨慎安排 library drop 顺序 | 说明 owner/drop ordering 的必要性，也暴露 Rust trait object ABI 的工具期/同工具链限制 | 这是显式 unsafe 的 Rust hot-reload 对照，不是公共 ABI 方案 |
| Unity Graphics | package pack/API validation job 与目标 Editor/OS、日志/crash artifact绑定；serialization test保留 legacy/polymorphic样本 | Zircon 尚无 artifact-bound ABI qualification 和历史 corpus | 本地 Graphics 仓库没有 Unity Player native plugin ABI 实现，不能用它推断 Player 的二进制合同 |

## 7. 目标架构

### 7.1 分离两类边界

1. **内部产品 Runtime DLL**：Host、Runtime、Editor 必须来自同一 `BuildSetId`，manifest 固定 artifact digest、target/data model、feature set、InterfaceSpec digest、payload schema set、dependency digests 与签名。严格 lockstep 可以 hard cut，但必须在加载代码前 fail-closed。
2. **可发布 SDK/插件 ABI**：使用独立 interface family、生成 C header 与 versioned proc-address，不暴露 Rust allocator、trait object、`usize` 或内部 JSON DTO；声明支持窗口、deprecation 与 capability negotiation。

不能继续让同一个 `ZrRuntimeApiV7` 同时承担“内部同构 DLL”与“长期第三方稳定 ABI”的所有承诺。

### 7.2 单一 InterfaceSpec

建立机器可读 `InterfaceSpec`，生成：

- Rust `repr(C)` carrier/function types 与 validator；
- C/C++ header、visibility/calling-convention macro 和 static assertions；
- numeric namespace、reserved range、since/deprecated、unknown policy；
- ABI layout/symbol/schema/capability manifest；
- ownership、threading、reentrancy、error、budget 文档；
- producer/consumer conformance tests、skew fixtures 与 ABI diff。

Interface crate 只拥有声明、生成物和纯 validator；Runtime 拥有 handler/registry/lifecycle，App 拥有 process/library/window host，Editor 只消费经过验证的 capability gateway。禁止 Interface 重新吸收业务实现。

### 7.3 身份与握手

入口不再只返回 table pointer，而返回或填充 `RuntimeHandshakeReceipt`：

```text
HostRequest
  build_set_id, target_model, interface_family_range,
  required_capabilities, host_limits, host_callback_table
        |
        v
RuntimeReceipt
  selected_family, build_set_id, interface_spec_digest,
  payload_schema_digest, capabilities, negotiated_limits,
  table_pointer, runtime_epoch, structured_rejection
```

内部产品模式要求 BuildSet 完全相等；SDK 模式按明确支持窗口选择 family。任何代码执行、session 创建或 callback 注册都必须发生在握手成功之后。

### 7.4 所有权与关闭状态机

- 所有 runtime-owned bytes 统一为 opaque allocation ID；删除 legacy `ZrOwnedByteBuffer`，禁止 foreign caller 提供 Rust allocation metadata。
- allocation ID、session/viewport/operation/subscription handle 至少登记 kind、owner session、generation 与 runtime epoch。
- close 为显式、可观测、可超时状态机：`Open -> Quiescing -> AwaitingReleases -> Finalizing -> Closed/Abandoned`。
- `begin_close` 返回 active action/callback/allocation census 与 deadline；新调用 fail-closed，release 保持可用；超时产生结构化 escalation receipt。
- Drop 不启动可能阻塞/重试的关闭。产品 owner 在正常控制流显式关闭，进程隔离是未可信 ABI/hang 的最后防线。

### 7.5 Carrier 与 payload

- 固定 header：family/struct version、`struct_size`、flags/reserved、correlation ID；固定宽度长度和整数。
- status 保留 raw code，并分 category/detail kind/encoding/truncation/correlation；借用 detail 的 thread/lifetime 由 header 明确。
- event 使用 header + typed payload/union；保留 raw unknown value，统一 strict/lenient policy，坐标不经 f32 中转。
- frame 使用 stride、format、color space、alpha、origin、HDR transfer、content rect、generation/epoch；empty/no-frame 是显式状态。
- 大 payload 使用 schema envelope 与按域选择的 page/stream/two-call/host allocator，producer 在对象构造前执行 negotiated budget 与 cancellation。

## 8. 分阶段重构

### M0 · 先封闭残余 P0

1. 删除/迁移 `ZrOwnedByteBuffer` 及 `ZrRuntimeHostFetchFnV1`，所有跨界输出只允许 opaque allocation 或 host allocator。
2. 建立 out-pointer、host-table alignment/size 与全部 byte carrier 的统一 validator；未可信外部 ABI 进入隔离子进程。
3. 把预算前推到 domain collection，使用 fallible reservation、分页/流式生产和 cancellation checkpoint。
4. 引入 BuildSet/target/InterfaceSpec/schema/artifact manifest，在加载前拒绝混装。
5. 实现有 deadline 的 quiesce 与结构化 escalation；从 Drop 移出阻塞式 destroy。

### M1 · InterfaceSpec 与生成边界

1. 冻结 internal Runtime DLL 与 public SDK 两个 interface family。
2. 从 spec 生成 Rust/C、layout、symbol、schema、capability、documentation 与 validators。
3. 把 V7 作为迁移源而非永久手写真相；每次 diff 必须分类 compatible/breaking/internal-hard-cut。

### M2 · Handle、status、event、frame 与 payload 收口

1. generation/owner/epoch-qualified handle registry 与 mismatch diagnostics。
2. status V2、typed event、FrameImageDescriptor 与 multi-platform surface family。
3. versioned payload envelope、negotiated limits、分页/流式 response 与统一 usage receipt。
4. App、Runtime Host、Editor 全部改用 Interface-owned validator 与 `ValidatedRuntimeApi` token。

### M3 · 生命周期、reload 与产品接线

1. 实现 begin-close/drain/release/finalize/abandon 状态机和 reload epoch rollover。
2. App 明确持有 BuildSet/RuntimeLibrary/RuntimeSession/Editor gateway 的销毁顺序；Editor 不再复制未经身份约束的 table。
3. 覆盖 multi-session、multi-viewport、callback reentrancy、concurrent release/destroy、reload 与 crash recovery。

### M4 · 发布资格

按 Interface07 的 C1-C5 认证层执行真实 DLL、C/C++ consumer、cross-compiler/target layout、N/N-1 skew、golden corpus、Miri/sanitizer/fuzz/guard-page/hang child process 与性能基准。只有 artifact digest、selection、日志和 crash evidence 被同一 qualification receipt 绑定后，才允许恢复 `Stable ABI` 文案。

## 9. 验收 Gate

| Gate | 当前 | 通过条件 |
|---|---|---|
| ABI-01 InterfaceSpec 单一真相 | Fail | table/carrier/constant/header/doc/test manifest均由spec生成 |
| ABI-02 C/C++ header与consumer | Fail | MSVC/Clang/GCC编译并运行真实DLL smoke |
| ABI-03 frozen exact V7策略 | Pass | 当前version/size/required-slot hard cut保持确定；后续由manifest接管身份 |
| ABI-04 BuildSet/target/schema/artifact身份 | Fail | 加载前完整验证并提供结构化拒绝 |
| ABI-05 host table/capability/limit握手 | Fail | size、alignment、required callback、capability dependency、limits全验证 |
| ABI-06 inbound pointer/slice/out shape | Partial | 当前byte slice基础保留；所有entry和隔离策略进入生成矩阵 |
| ABI-07 runtime-owned output exactly-once | Partial | 删除legacy buffer，全部输出统一registry/host allocator |
| ABI-08 producer budget/cancel/fallible allocation | Partial | 工作前预算、流式/分页、deadline/cancel、OOM拒绝全覆盖 |
| ABI-09 structured status/raw-code preservation | Fail | raw/category/detail/lifetime/encoding/truncation/correlation完整 |
| ABI-10 payload envelope与大对象传输 | Fail | schema identity、page/stream/two-call策略和usage receipt完整 |
| ABI-11 frame/surface/multi-viewport | Fail | descriptor完整且跨平台、多窗口、generation-qualified |
| ABI-12 handle owner/generation/epoch | Partial | allocation基础扩展到全部handle family |
| ABI-13 bounded close/reload state machine | Fail | 无期限等待消失，reload/abandon有显式receipt |
| ABI-14 release/destroy/App Drop一致 | Fail | outstanding allocation不会把可恢复状态直接升级为进程abort |
| ABI-15 real built Runtime DLL positive lane | Fail | required CI从artifact加载并完成create/call/close/unload |
| ABI-16 cross-language/cross-compiler layout | Fail | generated header、static asserts、独立consumer矩阵通过 |
| ABI-17 old/new skew matrix | Fail | 保存历史artifact并验证must-accept/must-reject |
| ABI-18 sanitizer/fuzz/fault isolation | Fail | bad pointer/len/enum/hang/race在隔离lane得到确定分类 |
| ABI-19 ABI diff与发布审批 | Fail | manifest diff、owner、changelog、breaking policy成为required gate |
| ABI-20 Editor gateway/frame统一验证 | Partial | gateway仍接收raw table但已做exact-shape检查；frame使用共享validator且无双重语义 |
| ABI-21 文档承诺准确 | Fail | 在C1-C5通过前改为internal FFI under development |

当前汇总：**1 Pass、5 Partial、15 Fail**。任何 P0 或 ABI-04/13/15/16/17/18 为 Fail 时，不得宣称 Runtime DLL 可发布、稳定、跨版本或优于成熟引擎。

## 10. Owner 与后续顺序

| Owner | 必须承担的内容 |
|---|---|
| `zircon_runtime_interface` | InterfaceSpec、生成 carrier/header/manifest/validator、status/event/frame/handle/payload声明与兼容政策 |
| `zircon_runtime` | handshake provider、allocation/handle registry、producer budget、session close/reload状态机、structured rejection |
| `zircon_app` | artifact/BuildSet验证、library/process owner、显式close与abort/escalation产品政策 |
| `zircon_runtime_host` | safe foreign output owner、call admission、budget/fuse/metrics，禁止重新暴露raw release metadata |
| `zircon_editor` | 只接收 validated capability gateway，共享protocol validator，拒绝自行发明ABI合法性 |
| `zircon_tooling` | spec/header/manifest生成、ABI diff、artifact packaging、C1-C5 CI与qualification receipt |

实施依赖顺序必须是：`InterfaceSpec/BuildSet -> generated validators/owned output hard cut -> bounded close -> App/Host/Editor migration -> real DLL/cross-language/skew/fault qualification`。不能先给 V8 再继续手写一套相同问题，也不能用更多 source `.contains()` 测试替代外部 artifact 证据。

## 11. 当前完成定义

本轮完成的是 Interface01 的当前源码复核、状态对账、参考引擎差异、目标架构、实施顺序与 21 项资格门；其后 P1-33 已完成 Interface、Editor 与 App 的 shared validator migration。旧报告仍保留为历史基线，后续判断 currentness 以本文为准。该迁移没有提供真实 DLL、跨语言 consumer 或受管 Cargo 资格证据；若生产源码漂移，逐项重开 ledger，不能只更新计数。
