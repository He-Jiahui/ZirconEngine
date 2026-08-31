---
related_code:
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/script/vm/host
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/
  - zircon_runtime/src/script/vm/capability_set.rs
  - zircon_runtime/src/script/vm/handles.rs
  - zircon_runtime/src/script/vm/runtime_context.rs
  - zircon_runtime/src/script/vm/scene_system.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/error.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/garbage_collection.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/memory.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/policy.rs
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs
  - zircon_runtime/src/tests/runtime_absorption/script_binding.rs
  - zircon_runtime/src/tests/runtime_absorption/script_binding/gameplay_host.rs
  - zircon_runtime/src/tests/runtime_absorption/script_binding/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/script_host_ledger/ledger.rs
  - zircon_runtime/src/tests/runtime_absorption/script_host_ledger/capability.rs
  - zircon_runtime/src/tests/runtime_absorption/script_host_ledger/ecs_facade.rs
  - zircon_runtime/src/tests/runtime_absorption/script_binding/mirror_docs.rs
  - tools/tests/test_runtime_script_binding_audit.py
  - tests/acceptance/runtime-script-binding-audit-owner-sync.md
  - zircon_runtime/src/tests/runtime_absorption/script_binding/inventory.rs
  - zircon_runtime/src/tests/runtime_absorption/script_binding/support.rs
  - zircon_runtime/src/tests/runtime_absorption/script_binding/split_layout.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/script_binding_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/script_binding_markdown.py
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/reflect
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/zircon_runtime/script/vm/host/function_ledger.md
status: in_progress
last_refined: 2026-08-15
---

# 13 脚本绑定面与反射收束

Runtime13 scene-transition contract sync (2026-08-15, source-owner inventory updated 2026-08-29): `script_binding_boundary` reports `expected_source_file_count = 28`, `expected_test_file_count = 3`, `expected_guard_file_count = 8`, `gameplay_callback_count = 40`, `host_capability_count = 13`, `missing_source_files = []`, `missing_guard_files = []`, and `risks = []`. `script.rs` remains the public facade for `argument_views`, `call_frame`, `descriptors`, `hot_path_metrics`, and `value_contracts`; `argument_views.rs` is now a folder-backed route over byte transport, borrowed value, argument source, and typed conversion owners. Capability-gated `request_scene_transition` produces a canonical `ReplaceActive` pending request only. It is not a scene replacement or completion contract: the missing frame-boundary consumer, staged prepare/rollback, lifecycle handoff, and terminal result publication remain Runtime10's [project-script-scene-transition-host-request](10/failure-2026-07-19-project-script-scene-transition-host-request.md) failure. The managed script Cargo gates remain pending.

Runtime 13 current child-owner sync (2026-08-02): `script_binding_boundary` reports `expected_source_file_count = 18`, `expected_test_file_count = 3`, `expected_guard_file_count = 9`, `missing_source_files = []`, `missing_guard_files = []`, `fixed_host_module_count = 6`, `fixed_host_function_count = 61`, `type_descriptor_count = 2`, `builtin_callback_count = 20`, `gameplay_callback_count = 39`, `macro_host_function_count = 2`, `host_capability_count = 12`, `guard_anchor_count = 9`, `native_ecs_abi_references = []`, `oversized_test_files = []`, `mirror_docs_guard_present = true`, and `risks = []`. The nine guard owners include the two route parents plus ledger/capability/ECS-facade, gameplay-host/mirror, despawn behavior, and Runtime 13 Cargo children. `runtime_13_script_binding_mirror_docs_match_structure_audit_counts` keeps the plan, runtime index, function ledger, M0 review, and interface-convergence mirror aligned; script package gates remain pending.

06 管 VM 插件生命周期（activate/空参数修复/热重载），10 管函数表 ABI 结构——本计划管两者之间的**语义层**：host function/module 注册面、`ZirconScriptType` 反射 marshalling、脚本对 ECS/资产/事件的能力面（capability）治理。

## 现状与证据（2026-06-13 实仓盘点）

- **绑定面三件**：反射宏家族 `zircon_host_function`/`zircon_host_module`/`ZirconScriptType`（`zircon_runtime_reflection_macros` crate 经 `lib.rs:32-34` 再导出）；使用点实测 4 文件——`script/vm/host/builtin_host_modules.rs`（内建宿主模块注册）、`core/framework/script.rs`（契约层）、`script/vm/tests.rs`、`lib.rs`。
- **VM 子系统形状**（`script/vm/` 当前根）：`backend/`（zr_vm 真实后端 + fallback，06 已细化）、`host/`（宿主函数注册）、`gameplay_host(.rs+/)`（玩法宿主面）、`capability_set.rs`（**能力门控已有雏形**）、`handles.rs`（脚本句柄）、`module/`、`plugin/`、`runtime/` + `runtime_context.rs`、`scene_system.rs`。旧 `scene_hook.rs` 已被硬切替换，不是可恢复的并行入口。
- **缺口 1——宿主面无清册**：Runtime 13 M0.1 已在 `docs/zircon_runtime/script/vm/host/function_ledger.md` 建立当前权威清册，记录固定内建面 6 个模块、52 个函数、2 个类型描述符，以及 `zr.zircon.bridge` 动态模块形状；新增宿主函数的机器守卫仍归 M1。
- **缺口 2——类型 marshalling 规则已裁决**：Runtime 13 M0.2 已把 `ZirconScriptType`/VM host 跨界形状定为 value descriptors / host handles / serialized payloads 三类，并声明 `zircon_runtime_interface::reflect` 保持 editor/remote schema 面，VM host calls 仍走 `ScriptHostValue` 描述符。
- **缺口 3——脚本-ECS 能力面已静态收束**：Runtime 13 M0.2/M2 已裁决脚本侧 ECS 默认访问路径为 `zr.zircon.gameplay` gameplay facade，经 `ScriptRuntimeCallContext` 持有 `LevelSystem`、实体与帧上下文；`ZrHostEcsApiV1` 保持 native/plugin ABI 层，脚本源码旁路由 `script_ecs_access_path_stays_on_gameplay_facade_not_native_ecs_abi` 守卫。
- 参考锚点（2026-06-13 实测核验，动工前先读——index 公约 §7.9）：
  - Godot GDExtension 注册面与 API dump（宿主面清册的机器可读形态）— `dev/godot/core/extension/{gdextension.{h,cpp},extension_api_dump.cpp}`
  - Fyrox 脚本 trait + 反射（Rust 侧脚本对象模型）— `dev/Fyrox/fyrox-impl/src/script/`（执行时核验：`ls dev/Fyrox/fyrox-impl/src/script/`）
  - Piccolo 反射/绑定生成（meta 解析路径）— `dev/Piccolo/engine/source/runtime/core`（执行时核验 reflection 子目录）
  - 本仓同构参照：06 的 native ABI 协商、10 的函数表清册——三计划共用"清册 + 守卫"方法论。

## 目标

1. 宿主面清册化：全部 builtin host module/function 成册（名称/签名/capability 要求/所属模块），新增必须过清册守卫。
2. marshalling 规则定稿：`ZirconScriptType` 跨界允许形状三分类 + 守卫；与 10 的 ABI 规则单一口径。
3. 脚本-ECS 能力面单点：数据访问路径裁决（gameplay_host 为唯一玩法面 or 与 EcsApi 分层），实体句柄失效语义与 08 对齐。
4. capability 门控从"存在"到"可审计"：每个宿主函数声明所需 capability，越权调用有显式拒绝路径与测试。

## 非目标

- VM 生命周期与空参数修复归 06；函数表 repr(C) 结构归 10；ZrVM 语言本体（仓外 `../../zr_vm`）不在本计划。
- 不新增脚本语言后端；不做脚本调试器（backlog）。

### 全局硬约束（继承总计划 §4）

- 动态边界只传 ABI-safe 值与序列化负载；硬切换；不新增 crate；非网络语义 server 命名是 blocker。

## 执行前检查清单

1. 活动会话对齐：06 的 M1 收尾（real-backend 验证）与本计划共享 `script/vm`——`git status --porcelain -- zircon_runtime/src/script/`，错峰执行。
2. 事实重核：`ls zircon_runtime/src/script/vm/host/`；Grep `zircon_host_function`，path `zircon_runtime/src`（注册点全集）；Grep `CapabilitySet|capability`，path `zircon_runtime/src/script/vm`（门控现状）。
3. 基线记录：`cargo test -p zircon_runtime --lib script --locked` 通过数。

## 里程碑

### M0 宿主面与 marshalling 审计

- 切片 0.1（清册）：`docs/zircon_runtime/script/`（执行时核验镜像文档）落宿主面清册——逐 builtin host module 列函数/签名/capability；来源 = `builtin_host_modules.rs` + Grep `zircon_host_function` 全集。DoD：清册覆盖全部注册点，每函数有 capability 列。
- 切片 0.2（裁决）：marshalling 三分类判词（值类型 serde / 句柄经 `handles.rs` / 大负载序列化缓冲）+ 与 `interface/reflect` 的关系声明；脚本-ECS 路径裁决（gameplay_host vs EcsApi 分层判词，与 10 的 0.1 清册互引）。DoD：判词落文档，越界形状清单（可为空）。

### M1 清册守卫与 capability 审计化

- 切片 1.1：结构守卫 `host_function_registry_matches_documented_ledger`（注册点全集与清册一致，新增未登记即失败；当前归属 `runtime_absorption/script_host_ledger.rs`，以避开 plugin bridge 会话正在修改的 `script/vm/tests.rs` 与 `script/vm/host` 源文件）+ 负例自检。
- 切片 1.2：capability 拒绝路径测试——`host_capability_representatives_are_declared_on_registered_modules` + `host_function_without_required_capability_is_rejected_with_explicit_error`（按 `capability_set.rs` 现状 API 定名）；每类 capability 至少一正一反。
- DoD：`cargo test -p zircon_runtime --lib script --locked` 全绿；清册进 CI 守卫。

### M2 实体句柄与 ECS 能力面对齐

- 切片 2.1：脚本侧实体句柄失效语义测试（despawn 后脚本持有句柄的行为，与 08-M1 的 `despawned_entity_handle_is_rejected_by_world_access` 同口径）：`script_held_entity_handle_reports_invalid_after_despawn`。
- 切片 2.2：数据访问路径按 0.2 判词收束（若裁决单点，旁路调用方迁移清单执行时枚举：Grep `gameplay_host|HostEcsApi`，path `zircon_runtime/src/script`）。
- DoD：句柄语义测试绿；访问路径单点或分层判词与代码一致。

### 测试阶段（milestone-first，每里程碑末）

- `cargo test -p zircon_runtime --lib script --locked -- --nocapture`；涉及 zr_vm 真实后端的用例沿用 06 的 feature 分层命令。
- `runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass` 在该 Cargo filter 通过前保持本计划 `in_progress`，并要求 M1/M2 代码守卫行继续带 `code_static_pending_cargo`。

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`13/2026-07-09-script-binding-and-reflection-output-records.md`](13/2026-07-09-script-binding-and-reflection-output-records.md)
- fixed 已修复：[host-registry-generational-handle-consumer-cutover](../../zircon_editor/editor/09/fixed-2026-07-13-host-registry-generational-handle-consumer-cutover.md)

- 2026-08-15 current static status, updated 2026-08-29: the Runtime13 framework facade is split across five focused leaves; the source/test/guard inventory is 28/28, 3/3, and 8/8. The 40-callback, 13-capability audit has no missing source, guard, ledger-anchor, or Runtime13 guard entries, and `tools/tests/test_runtime_script_binding_audit.py` is aligned to that inventory. `zr_vm_host_reflection.md` now hard-cuts its retired `scene_hook` entries to `scene_system` and its typed-error leaf, and its body describes the current scene dispatcher, borrowed ZrVM argument source, and VM byte-array lowering rather than the retired owned-input path. `ScriptSceneSystemError` and `ScriptSceneSystemResult` now expose only the `crate::script::vm` boundary needed by sibling gameplay-host callers, while constructors remain scene-system-private; the Runtime13 typed-error guard covers that boundary, pending UI12's next managed Editor compile. The retained owned scene-call context now records one weak-handle construction and one level clone at the exact construction sites; exact dispatcher cardinality and reset-free counter monotonicity are statically covered. Runtime07's product-snapshot projection remains an open lower-owner dependency, so the counters are not yet product-readable. This establishes the source-owned measurement baseline but does not complete the borrowed-scope/lifecycle hard cut. `zr.zircon.math` now has the versioned `math.scalar` Float ABI, its default host capability, ledger rows, and static negative vectors; `failure-2026-07-30-woc-deterministic-scalar-math-host.md` remains open until the managed Runtime13 gates and dependent WOC project vectors validate an immutable source snapshot. `failure-2026-08-15-gameplay-scene-transition-ledger-sync.md` remains open until the declared managed module-surface and script Cargo gates validate an immutable source snapshot.

- 2026-08-29 argument-view owner hard cut: status `runtime_13_argument_views_folder_backed_owner_static_passed_cargo_deferred`. The 354-line mixed owner was replaced by an 18-line route plus `argument_source.rs`, `byte_view.rs`, `typed_conversion.rs`, and `value_ref.rs`; inline tests moved to `argument_views/tests.rs`. Public exports, visitor/error text, scalar coercion, borrowed string/byte behavior, and business-boundary copy metrics remain unchanged. The focused structure and Runtime13 aggregate Python tests pass 2/2, the isolated Rust behavior/module harness passes 4/4, and the isolated Runtime13 mirror guard passes 2/2. Managed Cargo and product profiling remain explicit follow-up gates, so this record does not promote Runtime13 to complete.

- 2026-07-14 owner 硬切同步：`script_binding_boundary` 当前 `expected_source_file_count = 18`、`expected_test_file_count = 3`、`expected_guard_file_count = 9`、`missing_source_files = []`、`missing_guard_files = []`、`mirror_docs_guard_present = true`、`risks = []`。ZrVM `real_backend/host_modules.rs` 已归插件 crate，Runtime13 清单不再读取或计入插件实现 owner。
- 2026-07-18 host-call性能交接：framework script 2/2及production ScriptCallSite确认每call重建module/function owned String、深cloneCapabilitySet Strings，String/Bytes参数存在再次clone；reflect→script descriptor field投影接近O(F²)。Runtime13联动Runtime07硬切interned IDs、borrowed/arena call frame、shared capability bitset与generation compiled ABI，不保留同时构建旧owned context的兼容路径；见PERF-MVP-331及`docs/plans/performance/01/2026-07-18-runtime-core-framework-script-static-review.md`。
- 2026-07-22 runtime script热路径交接：历史 `scene_hook` 的逐文件审查确认每 Fixed/Update 全 World JSON 投影、binding identity/package callback 重复解析，以及 gameplay host 每 call clone runtime context/String、重复 manager/input snapshot、World 锁和 node scan。当前 `scene_system` 已以前向 hard cut 取代该 hook，并以 generation-owned active binding projection 移除稳定 generation 的全量扫描/JSON 解码；其剩余每 export context-handle clone、生命周期顺序与错误 fan-out 根因继续归 PERF-MVP-442/443，且与 PERF-MVP-331 共用 borrowed HostCallFrame、typed query index、预解析 callback 和单一 runtime-call scope 设计。详见`13/failure-2026-07-22-runtime-script-binding-hotpath.md`。
- 2026-07-22 reflection proc-macro补充：真实owner路径`zircon_runtime/reflection_macros/src/**` 8/8静态读完；field attr双parse与module item双scan已直接收敛为单遍。宏展开仍生成按调用构造owned module/type/function descriptor的函数，Runtime13在PERF-MVP-331的registration-generation compiled ABI内缓存/共享，不把proc-macro改成另一套运行时descriptor owner；见performance静态证据。
- 2026-07-22 World dynamic component补充：retained VM payload验证的registrations×entities重扫已止损为type index；单字段写仍clone整JSON并O(F²) schema验证，VM catalog sync仍clone/rebuild registry。Runtime13联动Plugins08按PERF-MVP-443/446发布generation registry与dense field accessor，稳定脚本调用不得走JSON整包事务；见dynamic-components静态证据与PERF-MVP-461。
- 2026-07-23 runtime-interface reflect合同补充：`reflect/**` 15/15与合同测试1/1确认schema/fields仍为全owned DTO、无generation/page/bytes/depth，公共read/write只携带field-name String；Editor动态Inspector稳定snapshot会clone schema/全fields并O(F²) join。Runtime13按PERF-MVP-567发布唯一generation-owned interned catalog与TypeSlot/FieldSlot，进程内与脚本compiled ABI共享同一字段表；远程DTO只在版本化有界边界物化，不保留第二registry authority。
