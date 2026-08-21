---
related_code:
  - zircon_app/src/bin/editor.rs
  - zircon_app/src/bin/runtime_preview.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/background_load.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/project_assets.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_editor/src/core/asset/import_flow/flight.rs
  - zircon_editor/src/core/export/inventory.rs
  - zircon_editor/src/core/export/preset.rs
  - zircon_editor/src/core/jobs/system/progress_observer.rs
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/core/play/process_backend/output.rs
  - zircon_editor/src/core/plugin/manager.rs
  - zircon_editor/src/core/process.rs
  - zircon_editor/src/core/recovery/autosave.rs
  - zircon_editor/src/core/recovery/session_guard/ownership_lease.rs
  - zircon_editor/src/core/runtime_event_consumer/host.rs
  - zircon_editor/src/core/settings/snapshot.rs
  - zircon_editor/src/ui/asset_editor/tree/tree_editing.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/host/export_process_support/child_guard.rs
  - zircon_plugins/ai/runtime/src/manager/execution_gate.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/direct_clip_worker.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/task_pool.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/manager/resend.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/dispatch.rs
  - zircon_plugins/net/features/websocket/runtime/src/backend/reader.rs
  - zircon_plugins/neural/runtime/src/cpu/interpreter.rs
  - zircon_plugins/neural/runtime/src/model/format.rs
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/texture_importer/runtime/src/container/ktx/ktx2/supercompression.rs
  - zircon_runtime/build.rs
  - zircon_runtime/src/asset/assets/texture/upload_support/ktx.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/runtime_asset_path.rs
  - zircon_runtime/src/core/resource/io/atomic_file/transaction.rs
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/registration/descriptor_entries.rs
  - zircon_runtime/src/core/runtime/handle/registration/descriptor_entries_five.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/foundation/runtime/config_manager/worker.rs
  - zircon_runtime/src/graphics/pipeline/async_compile.rs
  - zircon_runtime/src/graphics/scene/resources/fallback/create_fallback_texture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_post_process_resources/profiled_scene_post_process_resources.rs
  - zircon_runtime/src/operation/service.rs
  - zircon_runtime/src/plugin/native_plugin_loader/ffi_panic_guard.rs
  - zircon_runtime/src/plugin/runtime_profile/availability_projection/generation.rs
  - zircon_runtime/src/scene/ecs/commands/queued_command.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/system/native/scene_system.rs
  - zircon_runtime/src/scene/world/commands.rs
  - zircon_runtime/src/scene/world/schedule.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/text/sdf/generation_scheduler.rs
tests:
  - zircon_runtime/src/tests/tasks.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/ffi_panic_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/p0_robustness/native_host_callbacks.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/optimize/zircon_tooling/21-unsafe-rust-ffi-native-memory-thread-affinity-panic-unload-safety-governance-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/AssertionMacros.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Templates/ValueOrError.h
  - dev/godot/core/error/error_macros.h
  - dev/godot/core/io/logger.h
  - dev/bevy/crates/bevy_app/src/panic_handler.rs
  - dev/bevy/crates/bevy_ecs/src/error/command_handling.rs
  - dev/bevy/crates/bevy_ecs/src/error/handler.rs
  - dev/bevy/crates/bevy_render/src/error_handler.rs
  - dev/Fyrox/fyrox-core/src/log.rs
  - dev/Fyrox/fyrox-core/src/visitor/error.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.ExceptionMessages.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/UnifiedRayTracing/UnifiedRayTracingException.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 23 · Failure Contract、Panic/Unwind、Error Propagation、Poison Recovery 与 Result Observability 审查

## 1. 结论

Zircon并不是完全没有错误工程基础。保守扫描识别出451个以Error结尾的enum/struct定义，408处thiserror/derive Error相关使用；Runtime operation在snapshot/prepare/apply周围使用catch_unwind并生成失败状态，dynamic export和native plugin有panic guard，job/task保留部分panic信息，App在动态Runtime无法证明worker与callback已停时选择abort以避免DLL卸载后继续执行。这些机制应该保留。

但它们目前只是分散的局部策略，不是引擎级Failure Contract。相同的panic、poison、I/O cleanup、worker退出或无效状态，在一个owner中被转成typed outcome，在另一个owner中被expect终止线程，在第三个owner中被unwrap_or_else(poisoned.into_inner())继续使用可能已破坏的不变量，在第四个owner中又通过let _ =、.ok()或unwrap_or_default静默折叠。产品没有办法回答“哪个failure domain失败、是否可重试、哪些state可能被污染、是否需要隔离provider/world/device、当前结果是否仍可发布”。

本篇使用路径排除、纯cfg(test)尾部截断和人工复核组合扫描11,498个production-like Rust文件、约1,023,340行前缀代码。该方法仍会把由父mod以cfg(test)接入、但文件本身没有纯cfg(test)边界的test-only代码计入，所以以下数字是风险inventory，不是缺陷计数：

| 词法信号 | occurrence / 文件 | 本轮解释 |
|---|---:|---|
| todo!/unimplemented! | 0 / 0 | 正向基础；不能据此推导行为完整 |
| panic! | 24 / 16 | 包含build-time、test-only漏网、fixture与真实产品panic，必须逐点分类 |
| unreachable! | 190 / 60 | 107项以上集中在1-5 service descriptor手写组合，其余分散于状态/enum假设 |
| .unwrap() | 69 / 26 | 多数是先验长度或test-only漏网，仍有GPU/format/cache与plugin路径需typed invariant |
| .expect() | 950 / 435 | 943项捕获到字面message；399项带must/invariant/preflight等不变量语言 |
| production assert宏 | 243 / 53 | 包含GPU native layout、profiling、transaction与benchmark harness |
| .ok() | 804 / 374 | 同时包含checked conversion与真实错误降格，单靠词法无法判定 |
| let _ = | 600 / 283 | 同时包含collection旧值与I/O/send/join/cleanup结果丢弃 |
| unwrap_or_default() | 1,061 / 547 | 同时包含合法Option默认与错误/缺失/未实现语义折叠 |
| poison后继续 | 421 / 159 | Editor 202、Runtime 193、Plugins 25、App 1 |
| poison即expect终止 | 91 / 35 | Plugins 66、Editor 24、Runtime 1；与继续策略没有共同owner |
| catch_unwind | 74 / 33 | 已有重要containment，但continue/resume/count/log/outcome不一致 |
| Result<_, String>候选 | 948 / 328 | 576项为保守public-function-shaped匹配；regex可跨签名，需AST重取 |
| process abort / exit | 2 / 2 + 2 / 2 | 两处abort用于动态库安全；两个CLI tool直接exit |
| stdout/stderr打印 | 114 / 39 | binary可合理使用，但产品日志、receipt与终端输出尚未统一 |

本篇不重复Runtime lifecycle/task/asset/scene、Editor job/recovery/export、Plugin ABI/network、Tooling Test/Unsafe报告拥有的具体P0。**没有新增P0，登记40项P1和12项P2**。这些P1要求把已有typed error和containment接成可执行的failure control plane，而不是禁止panic、禁止unwrap或把所有错误改成一个中央enum。

## 2. 审查边界与方法

### 2.1 Production-like口径

1. 输入为Git追踪且位于zircon_app、zircon_editor、zircon_hub、zircon_plugins、zircon_reflect_derive、zircon_runtime、zircon_runtime_host、zircon_runtime_interface下的Rust文件和build.rs。
2. 排除路径级tests、benches、examples、fixtures、generated、vendor、target、test_sources、*_tests目录与tests.rs/test_*.rs等明显测试文件。
3. 文件内遇到第一处纯cfg(test)时截断后续内容；cfg(all(test,...))、由父mod条件接入的独立文件和宏展开仍可能漏入。
4. 对panic/unwrap/expect/unreachable/assert、Result丢弃、default fallback、poison、catch_unwind、process exit与String error做词法inventory，再人工读取高密度和产品边界路径。
5. 本轮没有把Drop中的best-effort cleanup、checked conversion的.ok()、collection remove返回值、build.rs fail-fast或World::resource这类显式panic API机械登记为缺陷。

### 2.2 Evidence等级

| Evidence | 本轮状态 |
|---|---|
| E1 tracked inventory | 已完成；source revision为ae2be3d865a937b9ed368bf965592045346c64e3 |
| E2 symbol/caller与失败分支阅读 | 已覆盖高密度panic、poison、discard、String Result与FFI/worker边界 |
| E3 产品语义与跨报告owner | 已对Runtime session/job/scene/graphics、Editor export/recovery/play、Plugin task/network作代表性闭环 |
| E4 dynamic failure injection | 未执行；当前443条scoped source dirty且既有Editor/Hub/WOC lane阻断未变化 |
| E5 crash/recovery/soak/performance | 未建立 |

### 2.3 失败类型必须先分域

| 类型 | 允许行为 | 禁止行为 |
|---|---|---|
| Caller/Input Error | typed reject并保留field/path/offset/owner | panic、空默认、String-only |
| Transient Dependency | bounded retry/backoff/cancel/deadline | 无限重试、静默降级 |
| Internal Invariant | fail current operation或隔离subsystem；必要时fatal | 继续使用未知一致性的state |
| Provider/Device Failure | quarantine/recreate/last-good generation | 把旧代对象当当前代继续 |
| Data Corruption | fail-close、保留artifact、进入repair/migration | 覆盖原件或默认化 |
| Process Safety Terminal | crash receipt后abort/exit | 跨FFI unwind、卸载后继续、报告成功 |

## 3. 必须保留的工程基础

### 3.1 Typed error并非空白

451个Error类型定义分布为Runtime 220、Editor 130、Plugins 49、Runtime Interface 39、App 8、Hub 4、Runtime Host 1。Editor dirty/save/job/recovery与Runtime asset/plugin/serialization等近期代码已使用thiserror形成局部typed cause；修复方向应聚合稳定code和boundary envelope，而不是推翻这些owner enum。

### 3.2 Operation和FFI已有panic containment

RuntimeOperationService在snapshot、worker prepare和owner apply周围catch_unwind，并区分WorkerPanic、OwnerApplyFailed等detail；dynamic exports与native plugin guard阻止Rust unwind越过C ABI。这是正确边界，但error identity、source、generation、retryability与product health尚未贯通。

### 3.3 Dynamic runtime teardown的abort有明确安全理由

RuntimeSession drop在destroy_session失败时记录teardown failure并abort；dynamic session bootstrap若log worker无法shutdown也abort。此处不是应当机械改为Result的panic hygiene问题，而是“无法证明foreign callback/DLL worker停止时不能卸载”的soundness终点。缺的是crash envelope、host receipt和统一fatal policy。

### 3.4 Panic API可以保留，但必须明确

World::resource/resource_mut与get_resource/get_resource_mut同时存在，属于常见的strict/optional双API；checked slice conversion后的unwrap、writing to String cannot fail等局部证明也可保留。要求是把precondition、failure domain和不可跨越的boundary写进合同，并让lint识别exemption。

### 3.5 Reference engines没有追求“零错误”

Unreal区分fatal check、shipping verify与可继续ensure并接入crash reporter；Godot区分ERR_FAIL返回与CRASH不变量；Bevy为ECS error提供ErrorContext、Severity、FallbackErrorHandler，并为renderer提供Ignore/StopRendering/Recover状态机；Fyrox Visitor把PoisonedMutex、I/O、parse、type mismatch编码成variant；Unity图形代码同时使用version mismatch诊断、typed error code和异常。共同约束是失败语义有层级、有上下文、有owner，而不是全部吞掉或全部崩溃。

## 4. P1差距：Inventory、Schema 与 Product Truth

### FAIL-P1-001 · 没有canonical FailureSiteInventory

当前只能用词法扫描重建panic/expect/discard/poison/catch位置，无法知道cfg-expanded product reachability、owner、failure domain、是否允许fatal或哪条验证lane覆盖。必须由AST/Cargo resolved graph生成source-bound inventory，并允许definition-bound exemption。

### FAIL-P1-002 · 没有FailureDomainDefinition

Runtime task、render device、asset transaction、Editor document、plugin generation、network session与process teardown各自定义局部错误，却没有stable FailureDomainId、parent domain、isolation unit、owned state与recovery authority。

### FAIL-P1-003 · 错误缺少跨边界稳定身份

大量错误只有Display字符串。跨App/Runtime DLL、plugin ABI、Editor process、Hub与tool receipt后，caller不能稳定聚合code、severity、retryability、user action或compatibility version。

### FAIL-P1-004 · Failure与Capability/Qualification没有闭环

同一feature在执行失败、fallback、poison恢复或output丢失后仍可能维持Available/Complete；Tooling16的Capability Truth没有canonical FailureEvent和HealthTransition输入。

### FAIL-P1-005 · 没有source/build/generation-bound FailureReceipt

日志文本不绑定BuildSet、provider generation、world/session、operation、artifact和device。历史失败、当前失败与重试后的成功不能形成可审核因果链。

### FAIL-P1-006 · 没有panic/expect/discard分类规则

仓库没有声明哪些目录允许build-time fail-fast、哪些public API允许strict panic、哪些Drop只能best-effort、哪些Result必须传播或观测。简单clippy deny会同时误伤合法invariant和漏过String/default语义。

### FAIL-P1-007 · Cargo profile没有显式panic策略

追踪manifest中未找到panic=abort/unwind策略。FFI containment、task catch_unwind、release artifact大小、crash behavior和platform unwinder因此没有进入ResolvedPackageGraphReceipt。

### FAIL-P1-008 · Test-only reachability没有机器真相

路径排除后仍能看到由父mod条件接入的test函数，说明SourceSet与Cargo cfg-expanded module graph没有形成统一产品清单。Failure inventory必须复用Tooling17/20的Resolved SourceSet，不得凭文件名推断production。

## 5. P1差距：Panic、Unreachable 与 Invariant

### FAIL-P1-009 · 产品环境变量错误直接panic

runtime_asset_path读取ZIRCON_ASSET_ROOT后，无法相对executable解析就panic。部署配置错误应返回带变量名、原值类别、executable和修复动作的startup error，不能在asset lookup helper内终止进程。

### FAIL-P1-010 · IBL参数合同在执行期panic

IBL command plan按字符串取face_size、sample_count和roughness，缺失或类型错误均panic；fixed dispatch遇到非Fixed又返回[0,0,0]。同一schema同时存在fatal与空dispatch两种失败语义，未在graph compile/admission阶段收敛。

### FAIL-P1-011 · Fallback texture对format用panic封闭

create_fallback_texture接收字符串format，仅支持两个值，其他值panic。fallback本应是renderer最后防线，应使用封闭enum或construction proof，并把不可构造状态挡在device资源创建之前。

### FAIL-P1-012 · Post-process资源通过Deref隐藏capability panic

OutputTransferOnly variant被Deref/full_resources访问时panic。类型系统允许持有“不支持compiled graph”的对象却暴露相同Deref接口，调用者只能靠隐式状态纪律避免崩溃。

### FAIL-P1-013 · Animation worker把调度失败与channel断开变成panic

direct_clip_worker忽略scheduler.schedule返回值，worker又忽略result_sender.send结果；接收端断线后panic。失败原因、shard、evaluator归还、部分pose与plugin generation没有进入typed batch outcome。

### FAIL-P1-014 · JobHandle::wait重新panic而非返回terminal outcome

JobHandle内部已保存panic message，却让wait()重新panic。库级caller无法选择fail operation、disable plugin、restart worker或terminate product，也不能把panic source绑定到JobId和BuildSet。

### FAIL-P1-015 · SceneSystem worldless能力由运行期panic兜底

trait默认run_without_world panic，supports_worldless_execution默认false，scheduler再依赖动态判断。错误实现或metadata漂移会在worker执行期崩溃；worldful/worldless capability应由不同trait/vtable或admitted descriptor表达。

### FAIL-P1-016 · strict World resource API缺统一panic contract

resource/resource_mut panic与get_resource双API可以保留，但公开文档、system failure handler、panic source context和Editor/script boundary规则不统一。来自脚本/plugin的缺资源不能无差别升级成process failure。

### FAIL-P1-017 · Plugin availability索引函数对合法enum variant panic

primary_category_index接收完整RuntimePluginAvailabilityCategory，却对MissingRequired panic。应由PrimaryAvailabilityCategory子类型或TryFrom拒绝index view，避免新增variant或错误caller进入fatal分支。

### FAIL-P1-018 · Service descriptor组合由190个unreachable分支维护

1到5 service的driver/manager/plugin组合展开为多文件手写match，单five文件有46个unreachable。计数和slice来自同一descriptor却没有封装为validated iterator；debug_assert在release消失后只剩panic，维护成本和状态空间均不必要。

### FAIL-P1-019 · Production assert混合验证、profiling与ABI不变量

243个assert候选集中于GPU native layout、UI profiling、transaction、benchmark harness等。assert是否仅debug、是否会在release产品终止、failure后GPU/world状态是否可复用没有manifest。

### FAIL-P1-020 · panic hook只覆盖部分产品入口

editor/runtime_preview安装diagnostic panic flush，但其他Rust binaries、Hub、plugin worker、build tools与native boundary没有同一ProductTarget panic/crash policy；eprintln和process exit不能替代CrashReceipt。

## 6. P1差距：Error Discard、Default 与 Transaction

### FAIL-P1-021 · Process log shutdown结果被顶层入口丢弃

editor与runtime_preview在决定exit code后let _ = shutdown_process_log。日志flush失败不会改变receipt或落入fallback sink，恰好可能丢失最关键的terminal诊断。

### FAIL-P1-022 · Atomic publish rollback存在未观测rename/remove

PBR viewer project asset发布在清理staging/displaced root和回滚rename时忽略错误。主错误返回后无法判断old root是否恢复、staging是否残留、下次启动应repair还是继续。

### FAIL-P1-023 · Export inventory只在Drop中静默persist

ExportGenerationInventory::drop忽略persist_cache错误。cache可重建不代表失败无需观测；下一次build可能把旧cache、缺cache与写失败都解释成普通miss，无法做性能和可靠性归因。

### FAIL-P1-024 · Play/export reader join错误被丢弃

PlayOutputPump和PlayOutputCaptureError在finish时忽略reader.join；export child wait和development watch join也有同类路径。线程panic、尾部输出丢失与正常EOF被折叠，最终产品/导出结果可在证据不完整时报告完成。

### FAIL-P1-025 · Channel send失败没有统一receiver-liveness语义

background load、job completion、animation worker、export progress等大量send结果被忽略。receiver主动退休可以合法，但必须由generation/cancellation状态证明；意外断线应形成DroppedDelivery observation。

### FAIL-P1-026 · Cleanup错误与主错误没有组合

autosave、settings、session guard、project create、asset sync和temporary artifact大量best-effort remove。多数失败发生在原操作已失败时，当前Result只能保留primary error，丢失cleanup cause、residual path和repair obligation。

### FAIL-P1-027 · .ok()同时承担checked conversion和错误吞没

KTX usize conversion使用.ok()?是合理的Option parser；current_exe().ok()、parse/metadata/codec等其他路径则可能把权限、损坏和不存在合并为None。必须按parser probe与required operation分开，不能全局替换或全局放行。

### FAIL-P1-028 · unwrap_or_default折叠absence、invalid与degraded

1,061个候选大量位于Editor presentation与Runtime projection。UI空文本可以合法，但settings decode、capability projection、artifact metadata、resource revision和runtime report若默认化，会把失败显示为“尚无值”或“正常零值”。

### FAIL-P1-029 · collection旧值丢弃与effect丢弃没有类型区分

let _ =同时用于HashMap::insert/remove旧值和fs/send/join/persist等effect result。lint无法仅凭语法区分；需要DiscardReason、best_effort helper或must-observe wrapper让意图进入类型与审查。

### FAIL-P1-030 · Drop路径不能承载唯一持久化authority

Drop不能返回Result，panic又可能双重panic/abort。cache、thread join、child termination、lease release和temporary cleanup若只有Drop路径，就没有调用方可等待、重试、记录或阻止promotion。

## 7. P1差距：Poison、Unwind 与 Supervision

### FAIL-P1-031 · Poison策略在421 continue与91 fatal之间分裂

Editor/Runtime多数Mutex/RwLock通过poisoned.into_inner继续；Network plugin多数expect("mutex poisoned")终止。两者都没有说明被保护state的不变量、panic写入窗口、repair函数或隔离范围。

### FAIL-P1-032 · 多字段authority在poison后继续可能发布不一致代次

ProjectAssetManager、Editor runtime event consumer、play controller、notification center、AI execution gate与HZB queue都保护多字段generation/queue/index/count。panic可能发生在部分字段更新后，直接into_inner无法证明一致性。

### FAIL-P1-033 · Lock owner没有Quarantined/NeedsRebuild状态

poison恢复后owner仍表现为正常Active；caller看不到recovered_from_poison、state reset、last-good restore或禁止写入。必须由owner选择Fail、Repair、Rebuild、Quarantine，不能由每个lock call site决定。

### FAIL-P1-034 · catch_unwind后的策略不一致

74个候选中有的转换为typed failure，有的只计数，有的忽略observer panic，有的清理后resume_unwind。没有PanicOrigin、contained side effect、unwind generation和continuation safety字段。

### FAIL-P1-035 · Observer/callback panic丢失source identity

Job terminal observer与若干callback只增加panic count或catch后继续。计数没有callback owner、subscription、thread、operation和payload identity，无法定位重复故障或执行自动disable。

### FAIL-P1-036 · Task supervision没有统一child outcome

scheduler、asset worker、shader compile、font SDF、navigation bake、plugin worker与Editor job各自管理thread/channel。parent没有共同的Started/Completed/Cancelled/Panicked/Disconnected/JoinFailed状态机与bounded child set。

### FAIL-P1-037 · FFI panic containment没有跨边界FailureEnvelope

dynamic exports和plugin guard能阻止unwind越过ABI，但通常返回status与短字符串；panic payload、owner generation、native call、quiescence影响和crash artifact没有稳定schema。

### FAIL-P1-038 · 必要abort没有CrashReceipt交付保证

两处dynamic runtime abort有正确soundness理由，但当前只eprintln。进程终止前没有证明diagnostic queue/minidump/session teardown state写入独立、async-signal-safe或host可消费的terminal receipt。

## 8. P1差距：Error Model 与 Observability

### FAIL-P1-039 · Result<_, String>形成第二套弱错误ABI

948个候选分布于328文件，集中在native plugin host、render graph executor、Editor callback/state和App bootstrap。String便于局部开发，却丢失code、source chain、field/resource、retryability、redaction和版本兼容。

### FAIL-P1-040 · 没有按失败语义做性能与可靠性资格

错误率、retry、recovery latency、quarantine次数、dropped diagnostic、cleanup residual、crash-free session和device restart都未绑定workload。通过happy-path benchmark不能证明比Unreal更稳定或更快。

## 9. P2长期能力

| ID | 能力 |
|---|---|
| FAIL-P2-001 | 由Rust AST、Cargo cfg-expanded graph、C/C++与shader/codegen输入生成FailureSiteInventory |
| FAIL-P2-002 | failure_contract属性/宏记录domain、boundary、fatality、recovery与exemption，并由CI验证 |
| FAIL-P2-003 | 跨Rust/FFI/process的versioned FailureEnvelope与ErrorCode Registry |
| FAIL-P2-004 | ProductTarget级panic hook、minidump、symbol、last-log与CrashReceipt service |
| FAIL-P2-005 | LockPoisonPolicy支持Fail、Repair、Rebuild、Quarantine并验证owner invariant |
| FAIL-P2-006 | 统一TaskSupervisor与structured child outcome、deadline、cancel、join、restart budget |
| FAIL-P2-007 | Render/asset/plugin/world可独立quarantine和generation restart，不拖垮整机 |
| FAIL-P2-008 | causal span把input、operation、worker、artifact、provider、failure和recovery串成同一trace |
| FAIL-P2-009 | typed retry policy包含idempotency key、backoff、jitter、deadline与retry budget |
| FAIL-P2-010 | 系统化fault injection覆盖panic、poison、I/O、OOM、device loss、disconnect与cleanup failure |
| FAIL-P2-011 | privacy/redaction policy使failure evidence可远程聚合而不泄露path、token、content |
| FAIL-P2-012 | 基于真实workload的reliability SLO、crash-free rate、recovery latency与竞争性对照 |

## 10. 目标架构

```text
Resolved SourceSet / Cargo cfg graph / ABI schema / codegen
                         |
                         v
                 FailureSiteInventory
                         |
          +--------------+--------------+
          |                             |
          v                             v
 FailureDomainDefinition        ErrorCode Registry
 owner / state / isolation      code / version / redaction
          |                             |
          +--------------+--------------+
                         v
                  Boundary Adapter
 Result / callback / task / FFI / process / GPU / persistence
                         |
                         v
                    FailureEvent
 source + BuildSet + generation + operation + causal chain
                         |
                         v
                  RecoveryDecision
 Reject | Retry | Degrade | Repair | Quarantine | Restart | Fatal
                         |
                         v
              Operation/Task/Product Outcome
                         |
                         v
 ProductHealthState + CapabilityTruth + QualificationReceipt
```

关键约束：

1. FailureSiteInventory是审查与lint输入，不是运行时热路径的中央字符串registry。
2. 具体error enum仍归domain owner；ErrorCode Registry只稳定跨边界投影。
3. catch_unwind只允许放在明确isolation boundary；捕获后能否继续由RecoveryPolicy决定。
4. poison处理只能由被保护state的owner定义，通用lock helper不得无条件into_inner。
5. Drop只做已被显式shutdown/finish路径覆盖的最后防线；promotion必须消费显式terminal receipt。
6. fatal abort可以是正确行为，但必须生成host可观察的terminal evidence并阻止旧代artifact被接受。
7. 性能热路径可用closed enum、validated token与unchecked内部helper；proof必须在admission边界生成。

## 11. 参考引擎对照

### 11.1 Unreal

AssertionMacros明确区分check、verify、ensure/ensureAlways；ensure有handler、failure count、stack capture与crash reporter识别。ValueOrError同时提供strict Get和TryGet。Zircon应学习fatal/nonfatal/optional三分与报告链，不照搬shipping中关闭check的具体宏策略。

### 11.2 Godot

error_macros把ERR_FAIL_INDEX/NULL/COND等“记录并返回”与CRASH_BAD_INDEX等“不变量终止”分开，并携带function/file/line、editor notify和handler type。Zircon需要同等明确的typed return与fatal boundary，不能复制宏式控制流。

### 11.3 Bevy

ECS ErrorContext记录System/RunCondition/Command/Observer身份，FallbackErrorHandler按Severity选择ignore/log/panic；RenderErrorPolicy明确Ignore、StopRendering、Recover并警告继续渲染的视觉安全风险。Zircon尤其需要renderer/device与system/task的policy state machine。

### 11.4 Fyrox

Visitor Error把I/O、field/type、parse、PoisonedMutex和user error分为variant；logger支持listener和one-shot抑制。它证明poison与serialization错误可以结构化，但其文件写失败静默等做法不是Zircon上限。

### 11.5 Unity Graphics

RenderGraph集中维护caller-aware exception message，debug protocol显式version mismatch，UnifiedRayTracingException携带error code。Zircon应采用稳定code+上下文+兼容版本，而不是依赖message文本路由。

## 12. 重构里程碑

### M0 · Truth Freeze

- 固定AST/Cargo-expanded production SourceSet与FailureSiteInventory；
- 标注test-only、build-time、strict API、checked invariant、best-effort cleanup与真实product boundary；
- 冻结新增未登记panic/expect/unreachable、poison recovery与effect Result discard；
- 不对现有24/190/950等词法数做机械清零KPI。

### M1 · Domain 与 Error Identity

- 定义FailureDomainId、ErrorCode、Severity、Retryability、IsolationScope、RedactionClass；
- 让现有451个Error owner映射稳定code，不建立单体错误enum；
- 先迁移App/Runtime ABI、plugin native、Editor process和release receipt边界；
- 为String error提供versioned adapter和移除期限。

### M2 · Panic、Task 与 Poison

- 将JobHandle wait、animation worker、asset/shader/navigation worker接入TaskSupervisor；
- 为catch_unwind登记PanicOrigin、side-effect boundary与resume/continue策略；
- 逐owner替换421处无条件poison continue和91处无条件fatal；
- 对asset/editor/play/plugin generation提供repair/quarantine state。

### M3 · I/O、Cleanup 与 Transaction

- 为atomic publish/save/export/cache建立PrimaryFailure + CleanupFailure集合；
- 将Drop-only authority改为显式finish/shutdown/close receipt；
- cleanup residual进入repair queue，禁止promotion忽略；
- 区分best-effort telemetry与required artifact durability。

### M4 · Runtime、Graphics、Plugin 与 Editor

- IBL/fallback/post-process把string/variant panic前移到compile/admission；
- service descriptor组合改为validated iterator/typed assembly；
- Editor presentation的default只消费明确Absent，不消费Invalid/Failed；
- network/plugin lock failure映射session/provider health与generation retirement。

### M5 · Crash、ABI 与 Evidence

- 每个ProductTarget安装同一panic/crash policy并绑定BuildSet；
- FFI FailureEnvelope支持稳定code、owned detail、release与redaction；
- 必要abort前交付host-visible terminal receipt/minidump/last-log；
- Test Service接入panic/poison/I/O/device/disconnect fault matrix。

### M6 · Qualification 与 Performance

- 统计crash-free session、failure rate、retry count、recovery latency、quarantine与residual；
- 用同workload比较fail-fast、recovery和degraded路径的CPU/GPU/memory成本；
- release gate拒绝missing failure evidence、unknown cleanup或unqualified recovery；
- 只有正确性、恢复性与性能同时达标后才允许竞争性结论。

## 13. 验收门

| Gate | 验收内容 |
|---|---|
| F01 | FailureSiteInventory绑定source revision、ResolvedPackageGraph与cfg-expanded ProductTarget |
| F02 | 每个panic/expect/unreachable有domain、classification、owner与exemption或迁移状态 |
| F03 | test-only与production reachability由module graph证明，不靠路径名 |
| F04 | public/user/script/plugin/asset/network input错误不触发未声明panic |
| F05 | strict panic API同时提供checked API并文档化precondition |
| F06 | catch_unwind只出现在登记的isolation boundary |
| F07 | FFI/exported callback无unwind越界，panic映射versioned status/envelope |
| F08 | task child均产生Completed/Cancelled/Panicked/Disconnected/JoinFailed outcome |
| F09 | JobHandle caller可消费typed terminal outcome，无强制repanic |
| F10 | poison后只有owner policy能Fail/Repair/Rebuild/Quarantine |
| F11 | poison恢复证明多字段invariant和generation，不直接继续Active |
| F12 | effect Result discard必须标best_effort reason并产生observation |
| F13 | cleanup error与primary error均保留，residual artifact进入repair |
| F14 | Drop不是save/export/persist/shutdown成功的唯一authority |
| F15 | default fallback区分Absent、Invalid、Unavailable、Failed与Degraded |
| F16 | 跨crate/process/ABI错误有stable code、version、source chain与redaction |
| F17 | String message不作为capability、retry、UI action或release gate主键 |
| F18 | renderer device error可Stop/Recover/Quarantine并绑定device generation |
| F19 | plugin/network failure绑定provider/session principal与generation |
| F20 | 必要abort前生成host可见CrashReceipt，旧代artifact不被promotion |
| F21 | panic/crash hook覆盖所有ProductTarget并交付symbol/minidump/last-log |
| F22 | fault injection覆盖panic、poison、I/O、OOM、device loss与disconnect |
| F23 | reliability evidence绑定BuildSet、workload、device、duration与统计口径 |
| F24 | 同场景竞争性报告同时包含happy path与failure/recovery成本 |

## 14. 本轮证据与限制

| 项目 | 结果 |
|---|---|
| Git source revision | ae2be3d865a937b9ed368bf965592045346c64e3 |
| production-like lexical scope | 11,498文件 / 约1,023,340行前缀代码 |
| positive typed base | 451个Error type定义；408处derive Error相关；74处catch_unwind |
| core risk signals | 24 panic / 190 unreachable / 69 unwrap / 950 expect / 243 assert |
| loss/default signals | 804 .ok / 600 let _ / 1,061 unwrap_or_default / 948 Result<_, String>候选 |
| poison split | 421处continue / 91处expect fatal |
| dynamic validation | 未运行；只读静态review，既有失败lane与source dirty条件未变化 |
| production mutation | 0 |

443条scoped Rust/manifest status变化表明多个Session仍在修改Runtime与Editor。所有计数和具体caller必须在实施前重取；本报告的source_recheck_required保持true。本轮没有修改production、tests、Cargo profile、panic hook、error enum、lock policy或CI，也没有重复已知被Editor、Hub、WOC或plugin lock阻断的动态lane。

## 15. 状态

| 项目 | 状态 |
|---|---|
| Failure contract review | review_complete |
| P0 | 0；具体高危P0继续由Runtime Interface/Plugin/Runtime/App原专项拥有 |
| P1 | 40 |
| P2 | 12 |
| Implementation | pending |
| Source recheck | required |
