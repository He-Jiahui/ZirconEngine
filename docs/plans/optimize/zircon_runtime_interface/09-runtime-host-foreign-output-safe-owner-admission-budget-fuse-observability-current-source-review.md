---
related_code:
  - zircon_runtime_host/Cargo.toml
  - zircon_runtime_host/src/lib.rs
  - zircon_runtime_host/src/foreign_output
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/handles.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/runtime_api
  - zircon_runtime_interface/src/profiling.rs
  - zircon_runtime_interface/src/world_sync
  - zircon_runtime_interface/src/ui/accessibility.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/dynamic_api/session/registry/allocation_registry.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_library/runtime_session
  - zircon_editor/src/core/gateway/session
tests:
  - zircon_runtime_host/src/foreign_output/tests.rs
  - zircon_app/src/entry/runtime_library/runtime_session/foreign_output/tests.rs
  - zircon_app/src/entry/runtime_library/runtime_session/foreign_output/performance_tests.rs
  - zircon_editor/src/core/gateway/session/tests.rs
  - zircon_editor/tests/runtime_foreign_output_policy.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
  - docs/plans/optimize/zircon_runtime_interface/08-runtime-dll-abi-ffi-version-handle-foreign-ownership-current-source-review.md
  - docs/plans/optimize/zircon_runtime_interface/07-contract-certification-abi-layout-version-skew-cross-language-fuzz-test-architecture-review.md
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
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: partial_foundation_only
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
source_recheck_required: true
---

# 09 · Runtime Host Foreign Output Safe Owner / Admission / Budget / Fuse / Observability 当前源码复核

## 1. 结论

Interface05 不是一份“全部仍未实施”的旧清单。当前 `zircon_runtime_host` 已经把六类 JSON 输出的 interface budget、shape/release 组合、per-session protocol fuse 和基础计数收敛到共享 crate；Runtime V7 也用 session-scoped opaque allocation registry 替换了主输出路径上的 `Vec::from_raw_parts` 释放。最新实现又在业务类型反序列化前加入 allocation-free JSON syntax-graph preflight，并让预检和 typed decode 共用一个 deadline。App 与 Editor 的主消费路径确实复用了这些基础，不能把它们当成临时代码全部推倒。

但 crate 的核心承诺仍然不成立。`RuntimeForeignOutputState::decode_json` 和 `ensure_call_succeeded` 是 safe API，却接受 Safe Rust 可任意构造的 `ZrOwnedResultV2`/`ZrStatus`，随后在内部解引用裸指针；`RuntimeOwnedOutputReleaser` 仍是可安全构造且 `Clone + Copy` 的 function-pointer token，没有持有 provider、Build Set/load epoch、session generation 或 lifecycle lease。App/Editor 的外层 owner 只能保护当前常规调用路径，不能修复公共 safe abstraction 的健全性。

fuse 也仍不是调用隔离器。所有消费者先 `ensure_available`/`ensure_session_available`，再执行 unsafe FFI；检查和调用之间没有原子 admission lease、in-flight census 或 `Open -> Fusing -> Fused -> Closing` 状态机。`acceptance_gate` 只避免在另一个线程熔断后把已解码结果记为 accepted，不能阻止已经通过检查的线程继续进入坏 provider，也不能让 release、destroy 和 unload 等待同一批在途工作。

新增 syntax preflight 是局部性能/防御改进，不是完整预算。它按 `max_encoded_bytes + 1` 允许 JSON value，而不是按 typed `max_items`；map key 不计数；随后还会再次完整解析为 `T`。因此合法 16 MiB 载荷仍可触发接近字节数级别的语法节点扫描、两次 parse 和未核算的 decoded heap。当前小载荷 microbenchmark 的 p99/throughput 不能证明最大载荷、复杂 key、内存放大或调度尾延迟合格。

本轮逐项重判 Interface05 的 91 项旧差异，并新增 1 项 P1。旧项现为 **85 Open、5 Partial、1 Closed**；新增项为 **1 Open**。合并账本是 **3 项 P0、73 项 P1、16 项 P2，共 92 项：86 Open、5 Partial、1 Closed**。3 项 P0 全部 Open；`Partial` 只表示存在可保留底座，不表示风险可接受。

## 2. 审查边界与证据

### 2.1 冻结范围

| 集合 | 文件 / 行 / bytes | 当前观察点 SHA-256 | 证据等级 |
|---|---:|---|---|
| Runtime Host production/build | 12 / 1,254 / 42,311 | `d000fe86a300eb12c0c6f74fb08b3bad562e5ae92ec59990951e3aee0d60e4f7` | E3：Cargo、crate root及11个非测试模块逐文件审读 |
| Runtime Host tests | 1 / 522 / 18,457 | `2cfdba53b5fdb85d39587d9d3d288c84ad4c11470fc361967fb4ff32fda5fc22` | E3：10个测试逐项复核 |
| App direct consumer | 5 / 1,796 / 66,012 | `6cf64ad8a4da14c6b838994be5085c38179727f7afe5c1caf3eedccb09b33ae3` | E3：loader、session、operation、frame/output纵向调用链 |
| Editor direct consumer | 7 / 934 / 32,952 | `51dda99c59c2fc4ca6af71fb0bcf0c7bec74611f161afc9a258728055b277c34` | E3：gateway、output owner、frame、profile/plugin/world消费链 |
| Interface carrier/policy DTO | 12 / 2,234 / 77,062 | `efc59b841bbfc2d51330ca6878aad55583b4d1d99dc4ff8db362f55e0f8b6404` | E3：buffer/status/handle/table/policy与accessibility声明闭包 |
| Runtime producer/allocation | 3 / 1,130 / 39,696 | `9cdfe9714ee2a3fd4c749f8b7168d080304fdbd4f78ec1fe7e2d4f994c5fa55e` | E3：frame encode、FFI输出和allocation registry |
| Zircon union | 40 / 7,870 / 276,490 | `3a9d7b6126ed03bffbc88f949331dac8d5a373841be772a5968b1aaa36c6a9c3` | 上述集合去重 |
| reference engines | 16 / 16,702 / 611,289 | `bb2fe92a52a13130fa0ac4bbe8ab295ade7b3f87866a0606d3d47de3e888bbdd` | E2/E3：owner、module lifetime、interface schema、task、pool和profiling scope |

指纹算法为：workspace 相对路径转 `/` 并小写，逐文件计算 lowercase SHA-256，按 path ordinal 排序，再对每行 `path + NUL + hash + LF` 的 UTF-8 字节流计算 SHA-256。Interface05 使用 `path<TAB>hash<LF>`，所以旧、新总指纹不可直接比较。本轮基线提交为 `79f64878f3b9526517644c055ad3bf5cadfccd0f`，观察日期为 2026-08-24。

### 2.2 源码漂移与动态证据边界

Host 历史只有三次直接提交：`05678e8d8` 建立共享 policy，`c6ba29949` 接入 V7 allocation registry，`ae2be3d86` 增加 JSON preflight 和性能收据。当前 Host 从 Interface05 的 12个Rust文件/1,555行增长到13个文件/1,776行；测试从旧报告观察的9个变为10个。历史 M2.4 文档记录8/8并宣称 Critical 0 / Important 0，已不能代表当前树。

本轮是 review-only，没有修改 production/tests，没有运行 Cargo、Miri、sanitizer、真实 DLL、unload/reload、guard-page child process、loom/stress 或性能基准。静态源码足以确认 safe/unsafe 签名、指针解引用、owner字段、check-call时序、budget顺序、计数语义和消费者分叉；它不能证明动态健全性或性能资格。后续不得通过在普通测试进程中直接解引用坏地址来“验证”问题，必须使用 compile-fail/Miri 和隔离子进程。

参考树版本冻结为 Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`；本地 Unreal 源树不单独构成 Git repository。

## 3. 当前可保留底座

1. V7 `ZrOwnedResultV2` 使用 opaque allocation ID，Runtime registry 校验session owner、拒绝forged/cross-session/duplicate release，并维护outstanding/high-water census。
2. 六类 JSON output policy 从 Interface bytes/items/depth/time/empty 常量派生；当前测试会检查六类映射和全局 nesting 值，不再完全依赖 App/Editor 手写常量。
3. shape validation、success/error cleanup、cleanup failure合并和per-session fuse集中到 Host；普通runtime status failure会release output但不会无条件熔断。
4. JSON decode在typed deserialize前做无分配syntax-graph preflight，preflight与typed parse共享deadline，并在validation前后重复检查总耗时。
5. `acceptance_gate` 可保证并发protocol rejection发生后，在途decode不再计为accepted；这是有价值的局部性质，但不是call admission。
6. metrics至少记录accepted/rejected/call failure/blocked、bytes、total/max decode time；App可在session teardown输出摘要。
7. App借用frame owner、Editor `_runtime_owner: Arc<dyn Send + Sync>` 和Runtime destroy outstanding gate共同降低了常规主路径的提前卸载概率。
8. frame、JSON、status已有共享字节上限，避免最原始的无限载荷；这些上限应迁移到新owner/admission架构，不应回退。

## 4. Interface05 原始差距复核

状态定义：`Open`表示目标合同未建立；`Partial`表示已有实质基础但风险或全路径未闭合；`Closed`表示旧finding的具体问题已由当前源码和本文索引修正消除。任何`Partial` P0仍按发布阻断处理。

### 4.1 P0

| ID | 当前状态 | 当前源码判断 |
|---|---|---|
| RHOST-P0-001 safe decode/status解引用可伪造裸指针 | Open | `decode_json`和`ensure_call_succeeded`仍是safe API；公开carrier仍可由Safe Rust拼出坏地址，内部仍执行`from_raw_parts`/diagnostics读取。 |
| RHOST-P0-002 `Copy` releaser无provider/session lease | Open | `RuntimeOwnedOutputReleaser`仍`Clone + Copy`，safe `new`保存裸session与unsafe function pointer；没有provider、epoch、generation或drop ordering。 |
| RHOST-P0-003 fuse无call admission lease | Open | App/Editor仍是`ensure_* -> unsafe FFI`两步；没有原子lease、in-flight census、drain或熔断后调用隔离。 |

### 4.2 P1：safe carrier、所有权与生命周期

| ID | 当前状态 | 当前源码判断 |
|---|---|---|
| P1-001 shape validator返回裸`usize` | Open | `validate_owned_result`仍只返回len，Editor随后自行`from_raw_parts`，类型没有表达“只验证形状”。 |
| P1-002 trusted output无私有构造边界 | Open | public carrier、session和allocation ID仍可重组；Host没有不可伪造的adopted owner。 |
| P1-003 output无provider/Build epoch | Open | releaser/output/state都没有BuildSet、provider ID或load epoch。 |
| P1-004 output无session generation | Open | owner只保存透明session handle，不能区分复用代际。 |
| P1-005 publication后immutable/lifetime合同未编码 | Open | host直接借用foreign bytes，类型无法阻止provider并发写或提前释放。 |
| P1-006 release失败后的所有权状态未定义 | Open | release错误后caller丢弃raw handle；terminal consume、可重试或仍owned没有公共规则。 |
| P1-007 release线程与重入约束缺失 | Open | safe helper和Drop可从任意线程调用function pointer，没有thread-affinity/reentrancy capability。 |
| P1-008 Drop只能吞cleanup failure | Open | Editor Drop触发fuse但不能返回错误；App/Host仍无可消费的typed cleanup receipt。 |
| P1-009 App/Editor重复RAII wrapper | Open | App frame/owned helper与Editor `GatewayOwnedOutput`仍各自组合slice、release、Drop；Host只提供raw helper。 |
| P1-010 App borrow与Editor Arc owner语义分叉 | Open | 同一ABI payload能否跨线程、跨operation存活仍由产品层各自决定。 |
| P1-011 state可为同一session创建多份 | Open | `RuntimeForeignOutputState::default()`仍公开且广泛直接构造，“session-wide fuse”不是类型不变量。 |
| P1-012 state与session靠手工配对 | Open | `SessionGateway::new`仍接收任意state；releaser/state/capability无法证明属于同一session。 |

### 4.3 P1：admission、fuse与teardown

| ID | 当前状态 | 当前源码判断 |
|---|---|---|
| P1-013 fuse只有bool | Open | 仍只有`protocol_failed: AtomicBool`，没有Fusing/Draining/Closing/Destroyed阶段。 |
| P1-014 无first-fault receipt | Open | 不保存首个kind/operation/raw status/session/allocation/provider/thread/time/build identity。 |
| P1-015 后续fault无原因历史 | Open | 只有aggregate rejected计数，没有bounded fault ring或causal grouping。 |
| P1-016 `protocol_failures`命名与语义不符 | Open | 仍只在首次false->true时加一，本质是fuse transition count。 |
| P1-017 call error/protocol error矩阵不完整 | Open | policy仍散在字符串错误和ownership/release分支，未知status、bad diagnostics、crash无集中决策表。 |
| P1-018 release在acceptance gate外 | Open | foreign release先执行，之后才拿Mutex；release不属于quiesce/in-flight census。 |
| P1-019 cleanup fault缺transaction因果 | Open | 并发release failure和fuse统计没有call/decode/release transaction identity。 |
| P1-020 poison后静默继续 | Open | Mutex poison仍直接`into_inner`，没有fatal invariant receipt。 |
| P1-021 Runtime destroy与Host fuse两套状态机 | Open | Runtime registry阶段、Host bool和App abort仍未统一。 |
| P1-022 reload无旧epoch drain | Open | 没有旧output/callback/release归零后再发布新table的协议。 |
| P1-023 fuse后cleanup白名单未生成 | Open | cancel/unsubscribe/unbind/release/destroy是否允许仍靠分散调用点选择。 |
| P1-024 blocked call无reason/deadline/retry | Open | 仍只有立即返回的字符串错误与总量计数。 |

### 4.4 P1：budget、decode与producer/consumer闭环

| ID | 当前状态 | 当前源码判断 |
|---|---|---|
| P1-025 budget丢弃per-policy nesting depth | Partial | policy test会在当前六类depth不等于全局值时失败；budget本身仍不保存该字段，不能表达未来per-policy差异。 |
| P1-026 nesting仍依赖serde实现 | Open | 错误文案宣称全局depth，但Host没有显式parser depth配置/生成validator。 |
| P1-027 public duration可让`Instant + duration` panic | Open | `decode_bounded_json`仍直接相加，custom public budget无`checked_add`。 |
| P1-028 chunk deadline不等于抢占/取消 | Partial | preflight、typed reader和validation阶段已有共享deadline/重复elapsed检查；单chunk visitor、allocation或业务validate仍不可抢占。 |
| P1-029 typed item count晚于完整反序列化 | Partial | 新syntax preflight在业务deserialize前限制通用JSON graph；精确typed item和decoded heap仍在完整`T`构造后才检查。 |
| P1-030 decoded heap amplification无预算 | Open | 两次parse、String/Vec/Map容量、typed copies和后续clone均未计入资源预算。 |
| P1-031 host cap晚于producer materialization | Partial | producer共享部分bytes/items/time上限；world/profile/accessibility/frame等仍可先构造完整domain/Vec再被Host拒绝。 |
| P1-032 Host无outstanding bytes/age | Open | borrowed frame/output数量、bytes、age、callsite和retention high-water不在Host state。 |
| P1-033 budget不参与握手 | Open | 不同build只靠编译常量，没有negotiated limit table或identity匹配。 |
| P1-034 `allow_empty`多义 | Open | NoContent/empty collection/empty carrier仍没有typed envelope。 |
| P1-035 item定义无公共schema | Open | row/delivery/node/span/request等口径仍是手写函数约定。 |
| P1-036 object key不计syntax item | Open | 新preflight仍用`next_key::<IgnoredAny>`，只对value调用counter；key数量和长度不进item口径。 |
| P1-037 profile嵌套集合计数不完整 | Open | typed counter仍手写，schema扩展不会自动纳入。 |
| P1-038 plugin-event只计deliveries | Open | raw JSON payload复杂度仍主要只受encoded bytes限制。 |
| P1-039 operation result只近似success tree | Open | envelope/key/error detail与schema演进仍无统一复杂度模型。 |
| P1-040 producer/Host重复item counter | Open | 仍有两份实现且无generated/cross-test等价证明。 |
| P1-041 host request只计request数 | Open | URI、IME text、path和嵌套数据没有string/substructure policy。 |
| P1-042 invalidation只计batch/dirty/fact | Open | 未来嵌套字段不会自动收紧预算。 |
| P1-043 无统一page/cursor abstraction | Open | family各自处理分页，profile/accessibility/frame仍整块输出。 |
| P1-044 payload copy/borrow/shared/stream策略未类型化 | Open | JSON release后owned、frame借用foreign bytes的差异仍在consumer私有wrapper。 |

### 4.5 P1：output family与消费者一致性

| ID | 当前状态 | 当前源码判断 |
|---|---|---|
| P1-045 accessibility无Host consumer policy | Open | API/producer存在，App/Editor/Host仍无真实capture consumer、kind、policy或OS bridge。 |
| P1-046 accessibility counter只有producer版本 | Open | 尚无shared generated tree/relation/item validator。 |
| P1-047 frame归入`SessionProtocol` | Open | 无frame bytes/capture/release/retention/dropped专属kind与metrics。 |
| P1-048 App/Editor empty frame语义分叉 | Open | Editor仍允许任意empty RGBA，App要求exact bytes；Interface08新增finding也确认此问题。 |
| P1-049 frame descriptor不完整 | Open | stride/format/color space/alpha/origin/HDR/content rect/fence仍缺失。 |
| P1-050 status diagnostics不在registry | Open | shared Host和App/Editor私有`ensure_status`并存，坏diagnostics不统一分类/fuse/计量。 |
| P1-051 non-output call与frame共用kind | Open | tick/event/surface/frame ownership仍混为`session_protocol`。 |
| P1-052 kind/index/label/policy手工平行 | Open | 7个kind及6个policy仍靠人工同步。 |
| P1-053 无API table output coverage证明 | Open | 没有枚举V7所有owned output/status slot并要求owner/policy/kind/validator的测试。 |
| P1-054 capability missing与Host未实现不可区分 | Open | null slot只返回capability missing，没有availability reason和host coverage truth。 |

### 4.6 P1：error、metrics与证据

| ID | 当前状态 | 当前源码判断 |
|---|---|---|
| P1-055 error只有两类 | Open | RuntimeCall/ProtocolViolation字符串仍压缩budget/decode/ownership/release/fused/closing/crash语义。 |
| P1-056 unknown status raw丢失 | Open | unknown raw code仍映射到通用Error。 |
| P1-057 error无typed operation/correlation/provider | Open | operation只进入message；session/allocation/build/time无结构字段。 |
| P1-058 metrics总量可回绕 | Open | `fetch_add`仍非饱和，长期bytes/count/time可wrap。 |
| P1-059 relaxed多字段不是一致快照 | Open | snapshot由独立Relaxed load组成，没有epoch/seqlock。 |
| P1-060 只有total/max无分布 | Open | 无histogram/quantile/bucket，不能回答长尾与SLO。 |
| P1-061 decode time不含producer/release | Open | 指标只覆盖Host decode阶段，不能解释端到端FFI latency。 |
| P1-062 无allocation age/retained high-water Host指标 | Open | Runtime census不能替代consumer callsite/age/retention观测。 |
| P1-063 `reported_len`在32-bit丢raw u64 | Open | 仍饱和到`usize::MAX`，receipt不保留原始len。 |
| P1-064 counters语义重叠 | Open | call failure伴随ownership fault仍可同时进入多个counter，无transaction outcome。 |
| P1-065 blocked session call无kind分解 | Open | 仍只有总数与自由字符串operation。 |
| P1-066 diagnostic line无schema | Open | 空格拼接文本仍无version/identity/timestamp/escaping/stable fields。 |
| P1-067 10个unit test不能证明safe abstraction | Open | 新增preflight测试不提供compile-fail/Miri/unsafe-boundary证明。 |
| P1-068 无bad pointer guard-page child process | Open | 仍只有shape错误，不能覆盖可读性/provenance/宿主隔离。 |
| P1-069 无真实DLL unload/reload/stale releaser | Open | 测试release函数仍是进程内静态符号。 |
| P1-070 无真实registry纵向release集成 | Open | Host fake allocation与Runtime registry资格仍未形成artifact-bound lane。 |
| P1-071 并发测试只证明不误accept | Open | 无check-call race、release/destroy、closing、cancel、unload、loom/stress。 |
| P1-072 历史M2.4 evidence不fresh | Partial | 本文用当前源码、测试数和可重建指纹刷新静态currentness；真实构建/性能/故障qualification仍缺失，旧8/8结论不得复用。 |

### 4.7 P2

| ID | 当前状态 | 当前源码判断 |
|---|---|---|
| P2-001 crate文档过薄 | Open | 顶层仍只有简短safe adapter说明，无trust/owner/thread/fuse/teardown合同。 |
| P2-002 `session(self)`强化Copy token语义 | Open | API形状未改。 |
| P2-003 state/releaser无bounded Debug | Open | identity/fuse/provider摘要仍不可安全查看。 |
| P2-004 operation限定`&'static str` | Open | 无generated operation ID或动态correlation。 |
| P2-005 kind label手写 | Open | 稳定字段ID/schema version仍缺失。 |
| P2-006 metrics snapshot不可稳定遍历kind | Open | exporter仍依赖已知kind，没有公开schema iterator。 |
| P2-007 单一test文件过大 | Open | 已从489行增至522行，ownership/decode/admission/metrics/perf尚未拆分。 |
| P2-008 microbenchmark混入unit tests | Open | wall-clock acceptance仍在普通test文件，未绑定受控perf artifact。 |
| P2-009 App重复release组合helper | Open | `owned_buffer.rs`仍保留薄包装之外的result组合。 |
| P2-010 `serde_json`未继承workspace policy | Open | Host仍直接固定`1.0.149`。 |
| P2-011 error Clone复制任意message | Open | detail仍无结构化长度上限。 |
| P2-012 diagnostic renderer多次分配 | Open | 仍构造Vec和多个String。 |
| P2-013 public module surface过宽 | Open | raw shape/release/item/budget implementation仍广泛公开。 |
| P2-014 linked/dynamic差异无能力文档 | Open | provider类型、卸载风险和线程保证未区分。 |
| P2-015 policy名无schema generation identity | Open | limit常量与payload schema兼容窗口仍靠命名。 |
| P2-016 Interface index过度宣称闭合 | Closed | 本轮索引明确标注safe owner、admission、accessibility、frame/status registry和qualification仍未闭合，并将09设为05的currentness owner。 |

## 5. 本轮新增差距

### RHOST-P1-073 · syntax preflight以字节数代理节点数并强制双重解析

`decode_bounded_json` 把 `json_value_limit` 设为 `max_encoded_bytes.saturating_add(1)`，先用 `IgnoredAny` visitor完整遍历一次，再用 `serde_json::from_reader::<_, T>`完整解析第二次。这个上限不是interface的typed `max_items`：16 MiB policy理论上允许约1,677万个value观察，远高于多数业务item上限；object key又完全不调用`observe`。因此preflight既不能准确表达schema complexity，也会给每个合法payload固定增加一次parse和一次4 KiB reader复制，最大载荷的CPU/尾延迟没有独立预算。

应由schema/policy registry生成单遍budget-aware seed/visitor，在读取时同时计encoded bytes、node/key/string、typed item和estimated heap，并把business object构造放在admission-controlled arena/page中。若短期保留两阶段设计，syntax budget也必须是独立的`max_json_nodes/max_keys/max_string_bytes`，使用`checked_add` deadline，记录preflight/typed/validate/release分阶段指标，并以最大合法/恶意payload基准证明额外成本。

## 6. 参考引擎差异与适用边界

| 参考 | 可学习的工程机制 | Zircon当前差异 | 不能误抄的部分 |
|---|---|---|---|
| Unreal | `FUniqueBuffer`/`FSharedBuffer`把data、size、owner和释放行为封装在不可随意重组的owner中；不能保证view寿命时转为owned。ModuleManager又显式区分loaded、pre-unload、shutdown、unload/abandon，MemoryTrace按heap/address记录alloc/free | Host仍把raw carrier、Copy releaser、provider owner和fuse state拆开；module unload与allocation/release census没有同一lease | Unreal内部C++所有权和module API不是Rust safe abstraction证明，不能只复制类名或引用计数 |
| Godot | GDExtension使用JSON/schema和header generator统一name/type/since/deprecated/allocator接口；worker pool有TaskID/GroupID、completion与协作等待 | Zircon policy/kind/operation/error schema手写；decode工作没有task identity、admission、cancel或completion receipt | Godot宽接口的兼容历史不应成为继续扩大V7手写表的理由 |
| Fyrox | `DyLibHandle`让plugin字段先drop、library字段最后drop，并明确dynamic Rust plugin的生产风险 | App/Editor局部保持owner，Host公共releaser却可脱离library长期复制 | Fyrox Rust trait-object plugin不是稳定第三方ABI方案，只能参考drop ordering和风险声明 |
| Bevy | Asset I/O以`Reader`/`AsyncRead`分层，TaskPool控制线程数、线程生命周期和task执行 | Host把最大16 MiB control payload同步双parse，deadline只在read chunk检查，没有受控executor/admission/cancel | Bevy asset reader不是foreign pointer validator；流式读取仍需Zircon自己的owner和schema预算 |
| Unity Graphics | CommandBufferPool显式Get/Release，ProfilingScope/Sampler把resource lifecycle、命名scope与CPU/GPU计量结合 | Host release lease和metrics分离，只有total/max decode且没有call/release scope | Graphics源码只提供managed render-resource模式，不能推断Unity Player native plugin ABI |

## 7. 目标架构

### 7.1 类型与生命周期边界

```text
RuntimeProviderLease
  owns: library/linked provider, validated API, BuildSet/load epoch, unload gate
    -> RuntimeSessionLease
       owns: session handle + generation, admission gate, lifecycle state, in-flight census
         -> ForeignCallLease
            owns: operation/correlation/deadline/cancel, provider+session liveness
              -> OwnedRuntimePayload<TPolicy>
                 owns: allocation ID, immutable bytes, usage receipt, exactly-once terminal release
```

raw FFI capture是唯一`unsafe`入口，并在同一处验证table/session/out-param/status shape、采用allocation、绑定provider/session lease。外部safe API只看见不可伪造的`OwnedRuntimePayload`、`BorrowedRuntimeFrame`或`RuntimeOutputPage<T>`。`RuntimeOwnedOutputReleaser`和产品私有wrapper硬切删除，不留compat shim。

### 7.2 Admission与关闭状态机

`SessionAdmissionGate`至少具有`Open -> Fusing -> Fused -> Closing -> Closed/Abandoned`。只有Open能原子增加call lease；首个fault保存typed receipt并拒绝新普通call。已经获得lease的call/decode/release进入统一census，Fusing等待或取消它们；cleanup capability按生成表继续允许release/cancel/unsubscribe/unbind/destroy。Runtime registry、Host fuse、App explicit close和provider unload必须消费同一个generation/epoch状态，不得再用四套互不理解的bool/abort规则。

### 7.3 Budget与output registry

建立单一`ForeignOutputRegistry`，每个V7/Vnext owned output和status family声明：stable ID、carrier/schema、storage mode、provider/consumer budget、node/key/string/typed item口径、deadline/cancel、empty/no-content语义、validator、metric schema和cleanup capability。由registry生成Interface常量、Runtime producer guard、Host decode seed、App/Editor adapter和coverage test。

大对象不得默认`domain Vec -> JSON Vec -> foreign allocation -> Host双parse -> typed Vec`。按family选择paged typed DTO、shared blob、borrowed frame lease或stream；producer在收集前获得reservation并使用fallible allocation，consumer在admission-controlled executor中单遍decode。frame/status/accessibility必须成为独立family，App与Editor只调用同一generated validator。

### 7.4 Error与观测

每次foreign transaction生成结构化receipt：provider/build/load epoch、session generation、operation/correlation、output kind/schema、encoded/decoded/retained bytes、preflight/decode/validate/release阶段耗时、raw status、outcome、cleanup disposition和first-fault linkage。计数从receipt聚合，使用饱和或checked累加、稳定schema、histogram/quantile和一致snapshot；文本行只是renderer，不再是真相源。

## 8. 分阶段重构

### M0 · 关闭safe abstraction与admission P0

1. 将raw capture/releaser构造收进明确`unsafe` provider integration边界；建立持有provider/session lease的不可伪造owner。
2. 删除`Copy` releaser和App/Editor自制foreign owner，所有slice/release只经typed payload owner。
3. 实现带in-flight census的call admission/fuse/closing状态机，并接入release、destroy和unload。
4. 用compile-fail/Miri证明Safe Rust不能伪造trusted carrier，用child process覆盖bad pointer与stale code pointer。

### M1 · 统一output registry与单遍预算decode

1. 从InterfaceSpec/output registry生成kind、policy、validator、item/node/key/string counter和API coverage test。
2. 用单遍seed/visitor或paged typed transport替换无界typed materialization和固定双parse。
3. 增加decoded/retained heap reservation、checked deadline、cooperative cancel和受控executor。
4. accessibility、frame、status diagnostics全部进入registry并消除App/Editor语义分叉。

### M2 · Provider/session lifecycle和关闭恢复

1. provider/build/load epoch与session generation进入owner、error、metrics和registry。
2. 建立explicit close：停止新call、drain/cancel、保留cleanup白名单、报告outstanding、release后retry、finalize/abandon。
3. DLL reload先drain旧epoch，再原子发布新validated table；旧payload只能调用旧provider的terminal release。

### M3 · 观测与性能工程

1. transaction receipt、first-fault receipt、bounded fault history和一致metrics snapshot。
2. 分阶段延迟、heap/retained age/high-water、blocked reason和release disposition进入结构化sink。
3. 最大合法/恶意payload、key-heavy/deep/wide JSON、slow validate、release stall和多session公平性进入artifact-bound benchmark。

### M4 · 发布资格

按Interface07的C1-C5执行真实DLL、linked/dynamic provider、unload/reload、cross-session/replay/double-release、concurrent fault/destroy、Miri/sanitizer/fuzz/guard-page/hang child process和历史skew资格。任何source `.contains()`、fake static release或普通unit microbenchmark都不能替代artifact receipt。

## 9. 验收Gate

| Gate | 当前 | 通过条件 |
|---|---|---|
| HOST-01 trusted carrier safe boundary | Fail | Safe Rust不能构造未验证pointer/status/output；raw adoption仅在有完整contract的unsafe入口 |
| HOST-02 provider/session/generation owner | Fail | payload持有provider epoch和session generation，releaser不可复制/悬空 |
| HOST-03 atomic call admission | Fail | check-call race消失，fuse拒绝新lease并drain/cancel in-flight |
| HOST-04 release/destroy/unload lifecycle | Partial | opaque registry基础保留；Host/App/Runtime统一explicit close、retry和epoch drain |
| HOST-05 producer/consumer resource budget | Partial | 当前共享bytes/items基础前推到domain work、fallible allocation和retained heap |
| HOST-06 syntax/typed/heap budget | Partial | preflight基础改为单遍schema预算，key/string/node/heap均受控 |
| HOST-07 deadline/cancellation/executor | Fail | 任意阶段可观测取消，不以4 KiB read checkpoint冒充抢占 |
| HOST-08 complete output family coverage | Fail | accessibility/frame/status及全部owned V7 slot进入registry |
| HOST-09 App/Editor protocol equivalence | Fail | 同一generated validator/corpus，empty frame等无双重合法性 |
| HOST-10 generated policy/schema registry | Partial | 六类映射测试扩展为API/producer/consumer全覆盖生成物 |
| HOST-11 typed fault/cleanup receipt | Fail | raw status、operation、identity、disposition与causal link结构化 |
| HOST-12 consistent observability | Partial | 基础counter扩展为一致snapshot、stage histogram、retained age/high-water |
| HOST-13 actual DLL unload/reload | Fail | stale releaser/old epoch被类型或runtime gate确定拒绝 |
| HOST-14 soundness/fault qualification | Fail | compile-fail、Miri、sanitizer、guard-page/fuzz child process通过 |
| HOST-15 concurrency/teardown stress | Fail | check-call、fault、release、destroy、closing、cancel、unload模型与stress通过 |
| HOST-16 artifact-bound performance | Partial | 当前小payload基准扩展最大载荷/heap/尾延迟并绑定build/hardware/artifact |
| HOST-17 public API/document truth | Fail | 删除“Safe host-side ownership”超前承诺或使其由资格证据成立 |
| HOST-18 index/currentness routing | Pass | 09明确接管05 currentness并保留旧报告为历史基线 |

当前汇总：**1 Pass、6 Partial、11 Fail**。HOST-01/02/03/13/14/15任一Fail时，不得宣称Host foreign output是safe abstraction、可靠熔断、可卸载或工程级发布边界。

## 10. Owner与后续顺序

| Owner | 必须承担的内容 |
|---|---|
| `zircon_runtime_interface` | InterfaceSpec/output registry声明、fixed identity、schema/policy/validator生成与ABI coverage |
| `zircon_runtime` | producer reservation、opaque allocation、session lifecycle、release/destroy/reload epoch与structured status |
| `zircon_runtime_host` | trusted payload owner、call admission、single-pass decode、fuse/fault receipt、metrics；不再暴露raw helper组合 |
| `zircon_app` | validated provider owner、explicit close、artifact/BuildSet策略与产品级crash/abandon政策 |
| `zircon_editor` | 只消费typed gateway/output owner和shared validator，不自行构造slice/release/protocol语义 |
| `zircon_tooling` | spec/codegen、真实DLL/fault/concurrency/perf qualification与artifact receipt；不拥有运行时策略 |

实施依赖顺序必须是：`InterfaceSpec/output registry -> provider/session lease -> unsafe raw adoption -> atomic admission/close -> single-pass budgeted transport -> App/Editor migration -> fault/performance qualification`。不能先给现有safe函数再加几个shape check，也不能把preflight测试数或小payload吞吐当成P0已经关闭。

## 11. 当前完成定义

本轮完成的是Interface05当前源码复核、91项旧差距逐项状态对账、1项新增差距、参考引擎对照、目标架构、实施阶段与18项Gate；没有实现代码修正。旧05保留为历史基线，后续判断Host currentness以本文为准。实施前必须重新计算冻结集合指纹；若Host、App/Editor消费链或Runtime allocation/lifecycle漂移，必须逐项重开ledger，不能只更新总数或测试数量。
