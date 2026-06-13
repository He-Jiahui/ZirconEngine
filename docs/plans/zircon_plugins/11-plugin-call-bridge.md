# 11 · 插件调用桥计划（强/弱依赖 · 接口直调 · 事件机制优化）

> 状态：工程化细化版 v2 · 优先级：P1（横切基建，02/05/06/07 的可选依赖调用均建立在其上）
> 前置：[01 插件架构核心](01-plugin-architecture-core.md) M2（TypedExtensionPoint/owner）、M3（finish/CapabilityView）；M3 里程碑另依赖 01-M5（热重载快照）
> 参考实现：Godot GDExtension 的 method bind 表（预解析槽位直调）、Bevy Events（双缓冲 + cursor）、OSGi 服务注册表（强/弱服务绑定语义，仅取形态）

## 0. 当前落地状态（2026-06-13）

本计划按里程碑推进，当前状态如下：

| 里程碑 | 状态 | 已完成项目 | 待完成项目 |
|---|---|---|---|
| M1 桥核心 | 完成 | `PluginInterface`、`InterfaceSlot`、`FrozenBridgeTable`、`StrongBridge`、`WeakBridge`、owner-tracked `export_interface(...)`、finish 期强/弱解析、manifest `provides_interfaces` / dependency `interfaces` 解析与校验、强依赖闭包诊断、强依赖 disable-blocker 查询、owner activate/deactivate/reload 表级世代翻转、桥矩阵快照、owner transition report、`bridge.table_summary` 汇总、debug 弱调用计数；`StrongBridge` Arc layout、`WeakBridge` cached-generation hit path、`BridgeGuard` resolved-target-only、debug 单次解析计数和 direct/strong/weak/pinned 墙钟采样性能基线测试已拆入 `extension_registry_bridge_performance_baseline.rs`。 | 无。 |
| M2 事件机制优化 | 完成 | `EventStore` 已从 `TypeId -> Box<dyn Any>` 运行期查找推进为 dense `EventTypeId` 槽位表；`EventReaderParam<T>` / `EventWriterParam<T>` 在初始化期缓存槽位；无读者通道 dormant，`send` 早退返回 `false`，`send_batch` 早退返回 `0`；`events.update_all` 已收口为 `SystemStage::First` 内置系统 `zircon.scene.events_update_all`；`EventSubscription<T>` 支持弱依赖事件订阅休眠/接通，接通时从当前游标点开始且不回放历史；`Events<T>` 已加入高水位预分配、连续低水位帧防抖收缩和容量诊断；`EventPayloadProfile` 固化 128 字节 inline payload 准则并建议更大 payload 通过 `Arc` 间接承载；测试覆盖 dense id 稳定性、dormant 早退、读者激活、First 阶段统一翻转、弱订阅接通、容量防抖和 payload profile。 | 无。 |
| M3 动态启停与热重载一致性 | 实现完成，验证受阻 | 桥表层已支持 owner activate/deactivate/reload 和 provider-clearing deactivation，weak bridge 可在 provider reload 后自动重连；catalog 合并阶段已保留 interface export 到最终 `RuntimeExtensionRegistry`；`RuntimePluginCatalog` 已提供帧边界 activate/disable/deactivate helper，并在 disable/deactivate strong provider 时返回 `RuntimePluginBridgeLifecycleError::StrongDependentsBlocked`；activate helper 已能从最终 registry 恢复 deactivation 清掉的 provider；`BridgeOwnerTransitionMode::Reload`、`FrozenBridgeTable::reload_owner_exports_with_report(...)`、`RuntimePluginCatalog::reload_bridge_provider_at_frame_boundary(...)`、`RuntimePluginBridgeLifecycleEvent::reload_provider(...)` 和 `RuntimePluginBridgeLifecycleState::reload_provider_at_frame_boundary(...)` 已打通完整 provider replacement，使 replacement registry 中的 interface export 可按当前 owner slot 回填到同一 frozen bridge table；`CoreRuntime` / `CoreHandle` 已能安装、读取、清理 lifecycle state，提供显式 provider activate/disable/deactivate façade，并在 linked runtime module `activate_module(...)` / `deactivate_module(...)` 时自动路由 provider lifecycle event，强依赖阻断会在服务卸载前返回 `CoreError::PluginBridgeLifecycleBlocked(...)`；`zircon_app` registration-aware bootstrap 已为 linked runtime plugin registration 安装 bridge lifecycle state；`NativePluginLiveHost` runtime load/unload/hot-reload 已有 `*_with_bridge_lifecycle(...)` helper，并新增 provider reload + descriptor scope rebuild 报告；M3/M4 外围性能结构基线已写入 `extension_registry_bridge_performance_baseline.rs`，覆盖 Native/VM 预解析调用路径。 | 新性能基线的 Cargo lib-test 验证被 render-owned E0061 调用签名漂移阻塞；独立源结构检查已通过。 |
| M4 Native/VM 接入与诊断 | 部分完成 | `ZrHostBridgeApiV1`、`ZrHostApiV3.bridge`、`ZrStatusCode::BridgeNotEnabled`、ABI layout/safety tests、`NativeHostBridgeCallScope` slot/method dispatch、`NativeHostBridgeCallScope::method_count()`、registration-scope `UnsupportedVersion`、disabled-provider `BridgeNotEnabled`、enabled/not-enabled diagnostics counter recording；`provides_interfaces.methods` 已成为包清单中的桥方法反射描述来源并经过方法名/slot/参数/capability 校验；`NativeBridgeMethodDescriptor` 已支持通过 interface id + method slot 元数据自动解析 dense interface slot 并构建 native bridge method table，`native_bridge_method_descriptors_from_manifest(...)` 可从 package manifest + native binding 生成 descriptor；`NativePluginLiveHost::runtime_bridge_call_scope_from_loaded_manifest(...)` 已能从已加载 runtime package manifest + native binding 构建 `NativeHostBridgeCallScope`；`NativePluginLiveHost::install_runtime_bridge_method_bindings(...)` 已提供 host 侧安装式 native binding 注册表，安装时要求 loaded manifest 并先校验 manifest/binding 一致性，`runtime_bridge_call_scope_from_installed_bindings(...)` 可在热重载后用当前 loaded manifest 与已安装 binding 重新生成 descriptor，`reload_runtime_bridge_provider_and_scope_from_installed_bindings(...)` 已把 provider reload event 与当前 loaded manifest descriptor rebuild 合并成 `NativePluginLiveHostBridgeReloadReport`；Native ABI v3 entry report 已新增 `bridge_methods` callback table（`NativePluginBridgeMethodTableV3` / `NativePluginBridgeMethodV3` / `NativePluginBridgeMethodCallV3`），`NativeBridgeMethodFn` 统一 Rust test callback 与 DLL ABI callback，`NativePluginEntryReport` 会把 ABI table 解析为 `NativeBridgeMethodBinding`；`NativePluginLiveHost` runtime load/hot-reload 会自动发现、校验并安装 DLL 暴露的 bridge method binding，runtime unload 会清理已安装 binding；VM 侧已新增 `zr.zircon.bridge` host export module 构建器，可从 `ScriptBridgeMethodDescriptor` 解析 bridge slot 并把桥方法暴露成脚本 host call，`register_bridge_host_module_from_manifest(...)` 可从 package manifest + VM binding 注册 host module；`HostExportRegistry::script_call_table()` 已生成 dense `ScriptCallTable`，真实 `zr_vm` backend 注册 host native function 时预解析到 `ScriptCallSite`，运行期回调不再走模块名/函数名查找；Native/VM 外围性能结构基线已写入，约束运行期 callback 不回退到 interface/name 查找；Native live host 运行期生命周期报告已接入桥生命周期状态；`BridgeDiagnosticsMatrix` / `diagnostic_lines()` 已提供 editor-facing 桥矩阵数据面；`EditorBridgeDiagnosticsSnapshot`、`EditorRuntimePlayModeBackendReport.bridge_diagnostics` 和 play-mode enter/exit 同步已把桥矩阵接入 editor state/snapshot 消费链路。 | retained UI 可视化面板；新性能基线 Cargo lib-test 验证需等待 render 编译漂移修复。 |

本次一方插件合约验证补充（2026-06-14 00:21 +08:00）：`zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs` 的 clip/graph/state-machine timeline event 测试已改为在 tick 前显式连接 `EventSubscription<AnimationClipEvent>`，验证 M2 dormant channel 的真实运行期语义：无订阅者早退不写缓冲，有订阅者时事件经 `World` event store 发布并可由订阅者读取。临时 manifest 验证中 animation 联动合约 16 项通过，physics runtime 合约 32 项通过，`zircon_plugins/Cargo.lock` 未修改。

本次 M3/M4 外围性能基线补充（2026-06-14 00:34 +08:00）：`extension_registry_bridge_performance_baseline.rs` 新增 `bridge_performance_baseline_native_bridge_calls_use_pre_resolved_slots`、`bridge_performance_baseline_vm_bridge_callbacks_capture_resolved_slot`、`bridge_performance_baseline_script_call_table_calls_dense_id_without_name_lookup` 和 `bridge_performance_baseline_real_zr_vm_callbacks_capture_call_sites`，分别约束 Native ABI callback、VM bridge host callback、`ScriptCallTable::call(...)` 与真实 `zr_vm` host callback 不在运行期重新做 interface/name 查找。`rustfmt --edition 2021 --check zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge_performance_baseline.rs` 已通过；直接冲突标记与尾随空白扫描通过；独立 PowerShell 源结构检查通过并输出 `outer bridge performance source-structure guard checks passed`。Cargo 验证未通过到目标测试：冷目标目录 `cargo test -p zircon_runtime --lib bridge_performance_baseline_ --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-performance-coremin-0614 ...` 15 分钟超时且未输出测试结果，已停止匹配 target-dir 进程；复用同一 target-dir 跑最小过滤器 `bridge_performance_baseline_native_bridge_calls_use_pre_resolved_slots` 时，lib-test 编译在目标测试执行前被 active render 区域的 E0061 阻断：`graph_execution/render_pass_execution_context/gpu.rs:628` 缺 2 个参数，`overlay/passes/base_scene_pass.rs:42` 缺 1 个参数。该 render 编译漂移由活跃 wgpu render session 负责，本插件会话不编辑该区域。

最新验证：`rustfmt --edition 2021 --check` 已通过桥 ABI/host-adapter相关文件、M2 dense/dormant event slice 文件、First-stage update-all 接线文件、弱依赖事件订阅接通文件、容量/payload profile 文件、M3 catalog lifecycle 文件、M3 bridge lifecycle state/event 文件、CoreRuntime bridge lifecycle state 入口文件、`zircon_app` bootstrap bridge lifecycle 接线文件、M4 native method metadata 文件、VM bridge host module 文件、manifest-driven native/VM descriptor source 文件、native live-host bridge lifecycle/binding 文件和本次 VM `ScriptCallTable` / real-zr-vm backend 接线文件；直接尾随空白/冲突标记扫描通过，`git diff --check -- <本次触达路径>` 仅报告既有 LF/CRLF 提示。容量/payload、catalog lifecycle、bridge lifecycle state/event、CoreRuntime event apply、`zircon_app` bootstrap lifecycle install、native method metadata、VM bridge host module、manifest bridge method metadata、native manifest descriptor source、VM manifest host registration、native live-host bridge lifecycle/binding、VM `ScriptCallTable` 接线新测试已写入对应测试文件。`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0613 --message-format short --color never` 已通过（仅既有 warning 噪声）。`cargo test -p zircon_runtime --lib script_call_table_pre_resolves_host_export_callbacks --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0613 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1 个聚焦测试；`cargo test -p zircon_runtime --lib zr_vm_real_backend_uses_script_call_table_for_host_callbacks --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0613 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1 个聚焦测试。`cargo test -p zircon_app --lib runtime_plugin_bootstrap_installs_bridge_lifecycle_state --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-entry-0613 --message-format short --color never -- --test-threads=1 --nocapture` 曾通过 1 个聚焦测试，并同时暴露 `ZrStatusCode::BridgeNotEnabled` 需要同步到 `zircon_app` runtime-library status mapping；映射已补齐，随后为测试 fixture manifest 补充 `provides_interfaces` 声明以消除 exported-but-undeclared 诊断。该 manifest 清理后的默认特性与 core-min 重跑均在 runtime 编译阶段超过等待窗口，未产出最终可采信通过结果。`cargo test -p zircon_runtime --lib native_live_host_builds_bridge_call_scope_from_loaded_manifest --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0613 --message-format short --color never -- --test-threads=1 --nocapture` 首次暴露测试代码 `unwrap_err()` 需要 `NativeHostBridgeCallScope: Debug` 的问题，已改为显式 match；重跑 10 分钟超时且未产出可采信测试结果。此前 `cargo test -p zircon_runtime --lib native_live_host_load_report_applies_runtime_bridge_lifecycle_state --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0613 --message-format short --color never -- --test-threads=1 --nocapture` 与对应 `--no-default-features --features core-min` 尝试均 10 分钟超时且未产出可采信测试结果。进程复查只发现无关 cargo/rustc 任务并已保持不动。上一轮 `cargo test -p zircon_runtime --lib dormant_subscription_connects_on_plugin_activate --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0612 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1 个聚焦测试（仓库既有 warning 噪声仍在）。

本次 native live-host installed-binding registry 与 bridge matrix data-plane 验证：`rustfmt --edition 2021 --check` 通过 `bridge.rs`、`bridge/table.rs`、`plugin/mod.rs`、`extension_registry_bridge.rs`、`host_api_adapter.rs`、`native_plugin_live_host.rs`、`bridge_methods.rs`、`native_plugin_live_host/tests.rs`；本次代码/文档/会话路径的冲突标记与尾随空白扫描通过，`git diff --check -- <本次触达路径>` 仅报告既有 LF/CRLF 提示。`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-bindings-coremin-0613 --message-format short --color never` 通过（仅既有 warning 噪声）。`cargo test -p zircon_runtime --lib native_live_host_rejects_installed_bridge_bindings_without_loaded_manifest --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-bindings-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture` 仍未运行到目标测试：此前曾在 lib-test 编译阶段 10 分钟超时并已清理遗留进程，warmed rerun 被无关 `zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs:535/537/552` 缺失 `STATIC_INDEX_PREFILTER_MIN_STATIC_INSTANCES` 阻断；不声明 Cargo test 通过。

本次 editor bridge matrix snapshot 消费验证：`EditorBridgeDiagnosticsSnapshot` 已把 `BridgeDiagnosticsMatrix` 投影为 editor state/snapshot DTO，保留 summary、row、owner slot、status、debug call counters 与稳定 `bridge.interface` 诊断文本；`EditorRuntimePlayModeBackendReport` 可携带 `bridge_diagnostics`，`NativePluginEditorRuntimePlayModeBackend::new_with_bridge_lifecycle(...)` 可在项目 runtime plugin load/exit 后读取 lifecycle bridge table，menu enter/exit play mode 会同步报告提供的桥矩阵，报告未提供矩阵或 enter 失败时清空 editor state 中的桥矩阵。新增 `play_mode_backend_bridge_matrix_projects_to_editor_snapshot` 覆盖 enter 时矩阵投影、`bridge.interface` 文本暴露、default exit report 清空快照。`rustfmt --edition 2021 --check` 通过本次 editor 桥矩阵路径；`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-editor-bridge-matrix-0613 --message-format short --color never` 通过（仅既有 warning 噪声）。`cargo test -p zircon_editor --lib play_mode_backend_bridge_matrix_projects_to_editor_snapshot --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-editor-bridge-matrix-0613 --message-format short --color never -- --test-threads=1 --nocapture` 输出目标测试 `1 passed` 后撞上外层 15 分钟超时；随后直接执行生成的 `zircon_editor` lib-test 二进制同一过滤器通过 1 个聚焦测试，当前匹配 target-dir 无遗留 cargo/rustc 进程。

本次 linked runtime module bridge lifecycle 验证：`FrozenBridgeTable::restore_owner_exports_with_report(...)`、`RuntimeExtensionRegistry::interface_exports_owned_by(...)`、`RuntimePluginCatalog::provider_package_id_for_runtime_module(...)`、`RuntimePluginBridgeLifecycleState::provider_package_id_for_runtime_module(...)`、CoreRuntime provider lifecycle façade、`activate_module(...)` provider restore、`deactivate_module(...)` pre-unload bridge deactivation 和 `CoreError::PluginBridgeLifecycleBlocked(...)` 已落地。新增测试 `core_runtime_module_deactivation_drives_plugin_bridge_lifecycle` 与 `core_runtime_module_deactivation_rejects_strong_bridge_dependents_before_unload` 已写入。`rustfmt --edition 2021 --check` 通过本次 core/bridge lifecycle 文件；`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-runtime-lifecycle-coremin-0613 --message-format short --color never` 通过（仅既有 warning 噪声）。`cargo test -p zircon_runtime --lib core_runtime_module_deactivation --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-runtime-lifecycle-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture` 两次在 lib-test 编译阶段超时，已清理匹配 target-dir 的 cargo/rustc 进程；后续直接执行 warmed `zircon_runtime` lib-test 二进制通过 `core_runtime_module_deactivation` 过滤组 2 项，并通过 `core_runtime_applies_plugin_bridge_lifecycle_events` 1 项门面测试。

本次 hot-reload provider/descriptor reload 验证：`BridgeOwnerTransitionMode::Reload`、`FrozenBridgeTable::reload_owner_exports_with_report(...)`、`RuntimePluginCatalog::reload_bridge_provider_at_frame_boundary(...)`、`RuntimePluginBridgeLifecycleEvent::reload_provider(...)`、`RuntimePluginBridgeLifecycleState::reload_provider_at_frame_boundary(...)`、`NativePluginLiveHost::reload_runtime_bridge_provider_and_scope_from_installed_bindings(...)` 和 `NativePluginLiveHostBridgeReloadReport` 已落地。`rustfmt --edition 2021 --check` 通过本次 reload 相关 runtime/native/test 文件；`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-reload-coremin-0613 --message-format short --color never` 通过（仅既有 warning 噪声）。`cargo test -p zircon_runtime --lib bridge_table_reloads_owner_exports_with_report --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-reload-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture`、`cargo test -p zircon_runtime --lib bridge_lifecycle_reload_replaces_provider_from_reloaded_registry --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-reload-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture`、`cargo test -p zircon_runtime --lib native_live_host_reloads_bridge_lifecycle_and_installed_binding_scope --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-reload-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture` 和 `cargo test -p zircon_runtime --lib bridge_lifecycle_state_owns_frozen_table_for_provider_events --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-reload-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture` 均通过对应 1 个聚焦测试。最初把 3 个 `cargo test` 并行写入同一 target-dir 的尝试因 target-dir 竞争超时，已停止匹配进程并改为串行重跑通过。

本次 DLL-level native bridge binding discovery 验证：ABI v3 `NativePluginEntryReportV3.bridge_methods`、`NativePluginBridgeMethodTableV3` / `NativePluginBridgeMethodV3` / `NativePluginBridgeMethodCallV3`、`bridge_method_bindings_from_abi_v3(...)`、`NativeBridgeMethodFn` ABI/Rust callback wrapper、`NativePluginEntryReport.bridge_method_bindings`、live-host load/hot-reload 自动发现与校验安装、runtime unload 清理已落地。首次定向测试曾被无关 UI 测试态 `state.values.get(property)` 的 `&&str` 查询编译错误阻断；已按 `&str` 查询修正后继续验证。`cargo test -p zircon_runtime --lib bridge_method_bindings_parse_abi_v3_callback_table --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-abi-bindings-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture`、`cargo test -p zircon_runtime --lib native_live_host_auto_installs_discovered_bridge_bindings_from_load_report --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-abi-bindings-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture` 和 `cargo test -p zircon_runtime --lib native_host_bridge_call_scope_dispatches_registered_method --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-abi-bindings-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture` 均通过对应 1 个聚焦测试；`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-abi-bindings-coremin-0613 --message-format short --color never` 通过（仅既有 warning 噪声）。`rustfmt --edition 2021 --check` 通过本次 ABI/native live-host/UI 编译修正代码范围；冲突标记/尾随空白扫描干净；`git diff --check -- <本次触达路径>` 仅报告 LF/CRLF 提示；当前 target-dir 无遗留 cargo/rustc 进程。

本次 M1 结构性与墙钟采样性能基线验证：`bridge_performance_baseline_strong_layout_matches_direct_arc`、`bridge_performance_baseline_weak_hot_path_keeps_cached_generation_before_provider_lookup`、`bridge_performance_baseline_pin_guard_keeps_only_resolved_target_for_batch_calls`、debug-only `bridge_performance_baseline_pin_guard_records_single_resolution_for_batch_calls` 和 `bridge_performance_baseline_samples_wall_clock_hot_paths` 已覆盖计划 §3.3 的 StrongBridge 直接 Arc/dyn 调用形态、WeakBridge 命中路径先查 cached generation 后才回退 table provider lookup、`BridgeGuard` 只持有已解析 target、pin 批量调用只记录一次桥解析计数，以及 direct/strong/weak cached/pinned 四条热路径的 65,536 次墙钟采样。测试位于 `zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge_performance_baseline.rs`，使主桥行为测试文件保持在大文件阈值以下。`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-next-coremin-0613 --message-format short --color never` 通过（仅既有 warning 噪声）。`cargo test -p zircon_runtime --lib bridge_performance_baseline_samples_wall_clock_hot_paths --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-next-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture` 通过 1 个聚焦测试，输出 `bridge.performance_sample` 诊断：direct `Arc` 17.61 ns/call、`StrongBridge` 17.36 ns/call、cached `WeakBridge` 125.23 ns/call、pinned `BridgeGuard` 20.50 ns/call（debug build，本机采样）。M1 桥核心的采样性能基线待办已关闭；M3/M4 外围 Native/VM 性能结构基线已写入并通过独立源结构验证，但 Cargo lib-test 验证仍等待 render 编译漂移恢复；retained UI 可视化面板仍在 M4 后续追踪。

## 1. 目标

为插件之间提供统一的**调用桥（Plugin Call Bridge）**：

1. **强依赖**：依赖方在注册期声明，目标插件必须启用——闭包验证在计划期完成，运行期调用**零检查零查找**（直接引用 + 虚表调用）。
2. **弱依赖**：目标插件可缺席/可禁用——调用方拿到的桥永远有效，目标未启用时调用返回结构化状态 `BridgeError::NotEnabled`，启用后自动接通；热路径成本恒为 **1 次原子读 + 1 次虚表调用**。
3. **事件机制优化**：事件通道 dense id 化、无订阅者通道零成本、弱依赖事件订阅休眠/自动接通，与调用桥共用同一启用/世代模型。

调用桥与既有机制的关系裁决：**Manager 保留为引擎本体的全局服务单例；插件之间的一切同步调用一律经调用桥**（替代 01 §3.2 原"只允许事件或 Manager"中对插件间 Manager 直查的容忍）；capability 仍是声明/探测层（finish 期决策"要不要接"），bridge 是调用层（运行期"怎么调"）。

## 2. 现状基线（实查）

- 依赖声明：`PluginPackageManifest.dependencies`（`plugin/package_manifest/plugin_dependency_manifest.rs`）已有 `{ id, required: bool, capability: Option<String> }`——**`required` 字段即强/弱语义的现成单源**，缺接口粒度声明。
- 跨插件调用现状：无标准通道——sound occlusion、ai sight 等计划目前只到"CapabilityView 探测"一层，实际调用要么经 Manager 全局查找（字符串/TypeId），要么无法表达"对方未启用"的状态。
- 事件：`scene/ecs/events.rs` 的 `Events<T>`（send/send_batch/update 双缓冲 + `EventCursor` 读游标）形态良好；但 `EventStore` 以 `TypeId` 哈希查通道（`events<T>()`），每次收发都付哈希成本，且无"无订阅者早退"。
- 01-M2 交付的 `TypedExtensionPoint`/`FrozenExtensionTable`/`PluginModuleId` 是本计划的直接地基。

缺口：

| # | 缺口 |
|---|------|
| C1 | 无接口级导出/导入声明与注册 API；插件间调用无类型安全通道 |
| C2 | 无强依赖闭包验证（缺目标时应拒绝加载并指明依赖链）；无弱依赖"未启用"结构化状态 |
| C3 | 无启用/禁用/热重载下的桥一致性模型（世代号） |
| C4 | EventStore 哈希查通道；无 dormant 通道；事件类型无 dense id |
| C5 | Native/VM 插件无桥 ABI 通道 |

## 3. 架构设计

### 3.1 接口声明（plugin.toml 单源 + 中立契约 trait）

接口 trait 定义在 `zircon_runtime::core::framework::<domain>`（中立契约层），实现在提供方插件；版本进 id，破坏性变更开新 id（同插件可同时导出 v1/v2 平滑过渡）：

```toml
# 提供方 plugin.toml
[[provides_interfaces]]
id = "physics.query.v1"          # [新增节] 导出接口目录，契约测试核对实际 export

# 依赖方 plugin.toml —— 复用现有 dependencies 节，required 即强/弱
[[dependencies]]
id = "physics"
required = false                  # true = 强依赖；false = 弱依赖（现有字段）
interfaces = ["physics.query.v1"] # [新增字段] 本插件实际导入的接口
```

```rust
// core/framework/bridge.rs [新增] 中立契约
pub trait PluginInterface: Send + Sync + 'static {
    /// 全局唯一接口 id，如 "physics.query.v1"；注册期 intern 为 InterfaceSlot。
    const INTERFACE_ID: &'static str;
}
// 示例（framework::physics [改造] 增加）：
pub trait PhysicsQueryInterface: PluginInterface {
    fn ray_cast(&self, query: &PhysicsRayCastQuery, filter: &PhysicsQueryFilter,
                out: &mut Vec<PhysicsRayCastHit>) -> Result<(), PhysicsBackendError>;
}
```

接口实现的并发契约：实现体 `Send + Sync`，只触碰提供方插件内部状态，**不得访问 World**（World 访问一律走系统 + `SystemParamAccess`，保证调用桥不绕过调度器的冲突图）。

### 3.2 注册与解析（`plugin/bridge/` [新增]，对接 01 生命周期）

```rust
// 注册期（RuntimePlugin::register）—— 提供方导出：
impl RuntimeExtensionRegistry {
    pub fn export_interface<T: PluginInterface + ?Sized>(
        &mut self, owner: PluginModuleId, implementation: Arc<T>,
    ) -> Result<(), RuntimeExtensionRegistryError>;       // 重复导出同 id → DuplicateExtension
}

// finish 期（RuntimePlugin::finish）—— 依赖方解析：
impl PluginFinishContext<'_> {
    /// 强依赖：目标缺席/未启用 → Err（含依赖链诊断），本插件激活失败。
    pub fn resolve_strong<T: PluginInterface + ?Sized>(&self)
        -> Result<StrongBridge<T>, RuntimeExtensionRegistryError>;
    /// 弱依赖：永远成功；目标缺席时返回 dormant 桥。
    pub fn resolve_weak<T: PluginInterface + ?Sized>(&self) -> WeakBridge<T>;
}
```

- 解析必须与 plugin.toml 的 `dependencies.interfaces` 声明一致（多解析/漏声明 → 契约测试失败），保持四源一致性纪律。
- 强依赖闭包验证（解决 C2）：finalize 前对全部 `required = true` 依赖做拓扑验证，缺失输出 `RegistrationDiagnostic { code: "bridge.strong_dependency_missing", chain: ["ai" → "navigation" → "physics"] }`；强依赖目标在运行期**拒绝单独禁用**（请求被拒并列出依赖者，禁用必须从依赖链叶子开始）。

### 3.3 桥的运行期形态与性能模型（解决 C1/C3）

```rust
// plugin/bridge/table.rs [新增]
/// finalize 产物：dense 槽位数组，运行期唯一权威。
pub struct FrozenBridgeTable { entries: Box<[BridgeEntry]> }
struct BridgeEntry {
    /// 类型擦除的提供者；启停/热重载经 ArcSwapOption 原子替换。
    provider: arc_swap::ArcSwapOption<dyn Any + Send + Sync>,
    /// 偶数 = 启用，奇数 = 禁用/缺席；每次状态翻转 +1。单次 Acquire 读完成"是否启用 + 是否变代"双判定。
    generation: AtomicU32,
    owner: PluginModuleId,
}

// plugin/bridge/strong.rs [新增]
/// 强依赖桥：finish 解析后即为直接引用，调用 = 一次虚表调用，无任何检查。
pub struct StrongBridge<T: ?Sized> { target: Arc<T> }
impl<T: ?Sized> std::ops::Deref for StrongBridge<T> { type Target = T; /* … */ }

// plugin/bridge/weak.rs [新增]
pub struct WeakBridge<T: ?Sized> {
    slot: InterfaceSlot,                       // dense 索引
    cached: UnsafeCell<Option<(u32, Arc<T>)>>, // (已验证世代, 已 downcast 引用) —— 慢路径填充
}
impl<T: PluginInterface + ?Sized> WeakBridge<T> {
    /// 热路径：1 次原子读判世代——命中缓存世代 → 直接虚调用；
    /// 世代变更 → 慢路径重新 downcast（每次启停后仅一次）；奇数世代 → NotEnabled。
    pub fn call<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R, BridgeError>;
    /// guard 形态：系统体起始处 pin 一次，整个系统执行期内免重复检查
    ///（安全性：启停/热重载仅发生在帧边界，由 01 生命周期保证——帧内 pin 不会悬挂）。
    pub fn pin(&self) -> Result<BridgeGuard<'_, T>, BridgeError>;
    pub fn is_enabled(&self) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BridgeError { NotEnabled /* 目标插件未启用/已禁用 */, Absent /* 目标未安装 */ }
```

性能预算（写进基准测试）：StrongBridge 调用 ≈ 裸 `dyn` 调用；WeakBridge 命中路径 ≤ 裸调用 + 1 次 Acquire load + 1 次分支；`pin` 后批量调用与 StrongBridge 同价。

启停/热重载一致性（解决 C3）：`activate` → 写 provider + 世代置偶；`deactivate` → 世代置奇 + 清 provider（帧边界执行，01 §3.4 时序保证）；热重载 = deactivate → 替换 → activate，世代跨两次递增，所有 WeakBridge 自动慢路径重连——**调用方零感知、零接线**。

### 3.4 事件机制优化（解决 C4，`scene/ecs/events.rs` [改造] + `event_channel.rs` [新增]）

1. **dense 通道**：`register_event::<E>`（01-M2）注册期 intern `EventTypeId(u32)`；`EventStore` 从 `TypeId → Box<dyn Any>` 哈希图改为 `Box<[ErasedEventChannel]>` 槽位数组 + 注册期一次性的 TypeId→id 映射（仅慢路径/动态侧使用）。`EventReader<E>/EventWriter<E>` SystemParam 在 `init_state` 解析一次槽位，**运行期收发零哈希**。
2. **统一翻转**：现 `Events<T>::update`（双缓冲交换）收口为 `events.update_all` 内置系统 ∈ First（在 `net.poll_ingress` 之前，order 前置），删除散落的手动 update 调用点。
3. **dormant 通道（无订阅者零成本）**：finalize 后无任何 reader 声明的通道标记 inactive——`EventWriter::send` 编译为一次分支早退，不写缓冲；弱依赖插件 activate 注入新 reader 时通道激活（与 §3.3 同一世代模型）。发布者对"订阅者可能不存在"完全无感。
4. **弱依赖事件订阅**：订阅目标插件（weak）声明的事件类型时，目标缺席 → cursor 进入 dormant 列表；目标 activate 时按事件类型 id 自动接通并从接通点开始读（不回放历史，语义与 `EventCursor` 现行为一致）。
5. **容量与分配**：通道按高水位自适应预分配（帧末收缩滞后 N 帧防抖）；大 payload 准则：> 128 字节的事件类型 payload 走 `Arc`，契约测试静态断言尺寸。
6. **选择准则（写进规范）**：需要同帧返回结果的查询/命令 → 调用桥；单向通知/一对多广播/帧间解耦 → 事件。禁止用事件模拟同步调用（回包事件反模式），禁止用桥做广播。

### 3.5 Native / VM 插件接入（解决 C5）

- ABI v3（01 §3.7）新增域表：

```rust
#[repr(C)]
pub struct ZrHostBridgeApiV1 {
    /// 预解析槽位直调：interface_slot/method_slot 在加载期经反射描述解析一次（与 08 CompiledCallSite 同机制）。
    pub call: unsafe extern "C" fn(ZrRuntimePluginHandle, u32 /* interface_slot */, u32 /* method_slot */,
                                   *const u8, usize, ZrByteBufferRef) -> ZrStatus,
}
// ZrStatus 新增码：ZR_STATUS_BRIDGE_NOT_ENABLED（弱依赖未启用，对应 BridgeError::NotEnabled）
```

- Native runtime entry report v3 同时暴露 `bridge_methods: *const NativePluginBridgeMethodTableV3`。该表只跨 DLL 边界传递 C ABI callback、`interface_id`、`method_name` 与 `user_data`；host 端以 package manifest `provides_interfaces.methods` 为描述单源，解析成 `NativeBridgeMethodBinding` 后再构建 `NativeHostBridgeCallScope`。动态库不得把 Rust trait object 直接传出 ABI 边界。
- Native/VM 插件之间不直接互调，一律经 host 桥表（owner 追踪、世代一致性、诊断免费获得）；VM 侧经 [08](08-zr-vm.md) §3.2 的 `ScriptCallTable` 走同一槽位。

### 3.6 管理与诊断

- `FrozenBridgeTable` 随 01 的 `finalize()` 一并冻结；`ExtensionOwnership` 反查覆盖 export 条目（卸载插件 → 其导出接口全部置奇世代）。
- 诊断（[10 规范](10-editor-integration.md) §5）：每槽位调用计数与最近 `NotEnabled` 次数（debug 构建原子计数，release 编译为空操作）进 rolling diagnostics；editor 增加桥矩阵视图（提供方 × 依赖方 × 状态），挂 `view.core.plugin_bridges`。

## 4. 模块文件树

```
zircon_runtime/src/core/framework/bridge.rs        [新增] PluginInterface trait / BridgeError
zircon_runtime/src/plugin/bridge/
  interface_id.rs                                  [新增] InterfaceSlot intern
  table.rs                                         [新增] FrozenBridgeTable / BridgeEntry / 世代协议
  strong.rs                                        [新增] StrongBridge
  weak.rs                                          [新增] WeakBridge / BridgeGuard
  diagnostics.rs                                   [新增] 调用计数（debug）
zircon_runtime/src/plugin/extension_registry/register/bridge_registration.rs  [新增] export_interface
zircon_runtime/src/plugin/runtime_plugin/lifecycle_context.rs  [改造] resolve_strong/resolve_weak
zircon_runtime/src/plugin/package_manifest/plugin_dependency_manifest.rs  [改造] interfaces 字段
zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs     [改造] provides_interfaces 节
zircon_runtime/src/plugin/native_plugin_loader/abi_declarations.rs        [改造] ABI v3 bridge method callback table
zircon_runtime/src/plugin/native_plugin_loader/bridge_method_abi.rs       [新增] ABI v3 bridge method table parser
zircon_runtime/src/plugin/native_plugin_loader/bridge_method_bindings.rs  [新增] Native bridge method binding/callback wrapper
zircon_runtime/src/scene/ecs/events.rs             [改造] dense 通道 + dormant
zircon_runtime/src/scene/ecs/event_channel.rs      [新增] ErasedEventChannel / 槽位表
zircon_runtime_interface/src/plugin_api.rs         [改造] ZrHostBridgeApiV1 + 新状态码
```

## 5. 里程碑与任务分解

### M1 桥核心（强/弱语义闭环）

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M1-T1 | PluginInterface/InterfaceSlot/export_interface | framework/bridge.rs、interface_id.rs、bridge_registration.rs | 01-M2 | `duplicate_interface_export_rejected` |
| M1-T2 | FrozenBridgeTable + 世代协议 | table.rs | M1-T1 | `generation_parity_encodes_enabled_state` |
| M1-T3 | StrongBridge：finish 解析 + 闭包验证 + 依赖链诊断 | strong.rs、lifecycle_context.rs | 01-M3、M1-T2 | `missing_strong_dependency_fails_with_chain`、`strong_call_has_no_runtime_check`（基准断言）、`bridge_performance_baseline_strong_layout_matches_direct_arc`、`bridge_performance_baseline_samples_wall_clock_hot_paths` |
| M1-T4 | WeakBridge：call/pin/NotEnabled + 缓存世代慢路径 | weak.rs | M1-T2 | `weak_call_returns_not_enabled_when_target_absent`、`weak_call_hot_path_single_atomic_load`、`pin_guard_amortizes_checks`、`bridge_performance_baseline_weak_hot_path_keeps_cached_generation_before_provider_lookup`、`bridge_performance_baseline_pin_guard_keeps_only_resolved_target_for_batch_calls`、`bridge_performance_baseline_pin_guard_records_single_resolution_for_batch_calls`、`bridge_performance_baseline_samples_wall_clock_hot_paths` |
| M1-T5 | plugin.toml 双节解析 + 声明/解析一致性契约 | package_manifest/* | M1-T3/T4 | `resolved_interfaces_match_manifest_declaration` |

### M2 事件机制优化

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M2-T1 | EventTypeId dense 通道 + Reader/Writer 槽位解析 | events.rs、event_channel.rs | 01-M2 | `event_send_receive_has_no_hash_lookup`、既有 Events 测试保绿 |
| M2-T2 | events.update_all ∈ First 收口 | events.rs、注册路径 | 01-M1、M2-T1 | `double_buffer_swaps_once_per_frame` |
| M2-T3 | dormant 通道 + 无订阅者早退 | event_channel.rs | M2-T1 | `unsubscribed_channel_emit_is_branch_only`、`channel_activates_on_late_subscriber` |
| M2-T4 | 弱依赖事件订阅休眠/接通（已落地为 `EventSubscription<T>` + `World` facade；插件 lifecycle 接线留到 M3） | events.rs、world/events.rs | M1-T4、M2-T3 | `dormant_subscription_connects_on_plugin_activate` |
| M2-T5 | 通道容量高水位预分配、低水位防抖收缩、payload 大小准则（已落地为 `EventCapacityMetrics` / `EventPayloadProfile`） | events.rs、world/events.rs | M2-T1、M2-T3 | `event_channel_preallocates_next_queue_from_high_water`、`event_channel_shrinks_after_debounced_low_water_frames`、`event_payload_profile_marks_large_payloads_for_arc_indirection` |

### M3 动态启停与热重载一致性

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M3-T1 | activate/deactivate 帧边界世代翻转；强依赖禁用拒绝（已落地为 catalog/CoreRuntime lifecycle helper、registration-aware bootstrap lifecycle state、linked runtime module activate/deactivate 自动路由、deactivation 后 provider restore） | table.rs、runtime_plugin_catalog/bridge_lifecycle.rs、extension merge、CoreRuntime activation/runtime_extensions、zircon_app entry bootstrap | M1、01-M3 | `runtime_plugin_catalog_merges_bridge_exports_into_final_registry`、`disable_strong_target_rejected_with_dependents`、`bridge_lifecycle_disable_and_activate_flips_provider_at_frame_boundary`、`runtime_plugin_bootstrap_installs_bridge_lifecycle_state`、`core_runtime_module_deactivation_drives_plugin_bridge_lifecycle`、`core_runtime_module_deactivation_rejects_strong_bridge_dependents_before_unload` |
| M3-T2 | 热重载经世代跨两次递增自动重连；owner-level reload report 已能从 replacement registry 替换 provider；catalog/state reload provider event 已接通；Native live host 可在 hot reload 后同时应用 provider reload 并按当前 loaded manifest + installed binding 重建 descriptor scope | table.rs、runtime_plugin_catalog/bridge_lifecycle.rs、runtime_plugin_catalog/bridge_lifecycle_state.rs、native live host 接线 | M3-T1、01-M5 | `hot_reload_swaps_provider_without_caller_rewiring`、`bridge_table_reloads_owner_exports_with_report`、`bridge_lifecycle_reload_replaces_provider_from_reloaded_registry`、`bridge_lifecycle_state_owns_frozen_table_for_provider_events`、`native_live_host_reloads_bridge_lifecycle_and_installed_binding_scope`、`native_live_host_load_report_applies_runtime_bridge_lifecycle_state`、`native_live_host_unload_runtime_plugin_applies_bridge_lifecycle_state`、`native_live_host_unload_runtime_plugin_is_blocked_by_strong_bridge_dependents` |

### M4 Native/VM 接入与诊断

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M4-T1 | ZrHostBridgeApiV1 + NOT_ENABLED 状态码 + host adapter；Native method metadata 自动解析 interface slot；package manifest `provides_interfaces.methods` 生成 native descriptor；Native live host lifecycle report 已承载 bridge lifecycle outcome；loaded manifest + native binding 可构建 `NativeHostBridgeCallScope`；host 侧 installed binding registry 已能在安装时校验 loaded manifest，并在重载后按当前 loaded manifest 重新生成 descriptor；`reload_runtime_bridge_provider_and_scope_from_installed_bindings(...)` / `NativePluginLiveHostBridgeReloadReport` 已把 provider reload 与 bridge scope rebuild 绑定成单个 live-host 入口；DLL-level 自动 binding discovery 已通过 ABI v3 `bridge_methods` method callback table 接入，load/hot-reload 自动刷新 installed binding registry，runtime unload 清理 binding | plugin_api.rs、host_callbacks.rs、native_plugin_loader/host_api_adapter.rs、native_plugin_loader/bridge_method_bindings.rs、native_plugin_loader/bridge_method_abi.rs、native_plugin_loader/abi_declarations.rs、native_plugin_loader/native_plugin_abi.rs、package_manifest/*、native_plugin_live_host/* | 01-M5、M1 | `native_bridge_call_round_trips`、`native_weak_call_maps_not_enabled_status`、`native_host_bridge_call_scope_builds_method_table_from_interface_metadata`、`native_host_bridge_call_scope_rejects_unknown_interface_metadata`、`native_bridge_method_descriptors_use_package_manifest_metadata`、`native_bridge_method_descriptors_reject_missing_manifest_binding`、`bridge_method_bindings_parse_abi_v3_callback_table`、`native_live_host_auto_installs_discovered_bridge_bindings_from_load_report`、`native_host_bridge_call_scope_dispatches_registered_method`、`native_live_host_load_report_applies_runtime_bridge_lifecycle_state`、`native_live_host_builds_bridge_call_scope_from_loaded_manifest`、`native_live_host_reuses_installed_bridge_bindings_for_loaded_manifest_scopes`、`native_live_host_rebuilds_bridge_scope_from_reloaded_manifest_and_installed_bindings`、`native_live_host_reloads_bridge_lifecycle_and_installed_binding_scope`、`native_live_host_rejects_installed_bridge_bindings_without_loaded_manifest`、`native_live_host_rejects_loaded_manifest_bridge_method_without_binding`、`bridge_performance_baseline_native_bridge_calls_use_pre_resolved_slots` |
| M4-T2 | VM 经现有 host export module 接桥（已落地 `zr.zircon.bridge` builder、package manifest 注册 helper、`ScriptCallTable` dense call site 和真实 `zr_vm` backend 预解析接线） | script/vm/host/bridge_host_module.rs、script/vm/host/script_call_table.rs、script/vm/backend/zr_vm_project_backend/real_backend/host_modules.rs、package_manifest/*、08 host_interface 接线 | 08-M2、M4-T1 | `bridge_host_module_dispatches_vm_calls_through_resolved_bridge_slots`、`bridge_host_module_reports_disabled_bridge_to_vm_callers`、`bridge_host_module_registers_methods_from_package_manifest`、`bridge_host_module_rejects_manifest_method_without_binding`、`script_call_table_pre_resolves_host_export_callbacks`、`zr_vm_real_backend_uses_script_call_table_for_host_callbacks`、`bridge_performance_baseline_vm_bridge_callbacks_capture_resolved_slot`、`bridge_performance_baseline_script_call_table_calls_dense_id_without_name_lookup`、`bridge_performance_baseline_real_zr_vm_callbacks_capture_call_sites` |
| M4-T3 | 调用计数诊断 + editor 桥矩阵数据面（已落地为 `BridgeDiagnosticsMatrix`、单行 `bridge.interface` 诊断文本、按 owner 过滤；`EditorBridgeDiagnosticsSnapshot` / play-mode backend report / enter-exit state sync 已接入 editor snapshot 消费，retained UI 可视化面板仍待后续） | bridge/table.rs、bridge.rs、plugin/mod.rs、editor play-mode backend、workbench snapshot/state | [10 规范](10-editor-integration.md)、M1 | `bridge_diagnostics_paths_registered`、`bridge_diagnostics_matrix_projects_editor_rows`、`play_mode_backend_bridge_matrix_projects_to_editor_snapshot` |

## 6. 对既有计划的接线变更（落地时同步修订）

- [01](01-plugin-architecture-core.md) §3.2 跨插件通信规则更新为："只允许通过**事件、调用桥（本计划）**或引擎本体 Manager；禁止插件间直接类型依赖"。
- [02 Sound](02-sound.md) occlusion、[05 Navigation](05-navigation.md) 几何收集、[06 AI](06-ai.md) sight raycast：finish 期 `CapabilityView` 探测语义不变，运行期调用从"Manager 查找"升级为 `WeakBridge<PhysicsQueryInterface>`（physics 侧在 [03](03-physics.md) M2 后导出 `physics.query.v1`）。
- [06 AI](06-ai.md) MoveTo 写组件字段、[03↔04](04-animation.md) ragdoll 双资源通道**维持现状**——组件/资源数据流不属于调用桥范畴（经 ECS 与调度器管理）。

## 7. 验收命令

```bash
cargo test -p zircon_runtime --lib --locked
cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked
```

## 8. 风险

- `WeakBridge` 的缓存重验依赖"启停只在帧边界"这一生命周期不变量；若未来引入帧内禁用，需把 `pin` 升级为读写世代锁——在 table.rs 注释中显式记录该不变量，并以 `debug_assert` 在非帧边界翻转时炸断。
- `ArcSwapOption<dyn Any>` 的 downcast 仅在慢路径发生，但接口 trait 必须 `'static`；带借用参数的接口经引用参数传递（如 `&mut Vec<…>` out 参数），禁止返回内部引用——接口设计准则写入 framework/bridge.rs 文档注释。
- 事件 dense 化触及 `EventStore` 全部调用点（runtime 内部 + 插件），与 01-M2 的 register_event 同窗口落地避免两次迁移。
- dormant 通道的"激活时不回放历史"语义要在文档与测试中钉死，避免插件作者误以为可收到激活前事件。

## 9. 附录 · dev 参考源码对位

实现各任务前**必须先读对应参考实现**，并发模型与世代语义对照真实代码核对，禁止凭空实现：

| 设计点 | 参考源码（已核验存在） | 看什么 |
|--------|----------------------|--------|
| 事件双缓冲/cursor/漏读语义 | `dev/bevy/crates/bevy_ecs/src/event/` | Events 的 update 时序、EventCursor 落后两帧的漏读行为、send_batch——M2 dense 化重构的行为基线 |
| 跨边界函数表/方法 bind 槽位直调 | `dev/godot/core/extension/gdextension_interface.cpp` | method bind 的预解析与缓存、版本协商失败的报错形态——§3.5 ZrHostBridgeApiV1 的判例 |
| 扩展启停的注册回收 | `dev/godot/core/extension/gdextension_manager.cpp`、`gdextension_library_loader.cpp` | 库卸载时已注册 class 的撤销顺序、重载世代处理 |
| 插件容器内服务相互访问 | `dev/Fyrox/fyrox-impl/src/plugin/` | 插件间经容器查找的边界与限制（反例参照：我们用编译期解析的桥替代运行期查找） |
