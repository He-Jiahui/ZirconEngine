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
  - zircon_runtime/src/script/vm/scene_hook.rs
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
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/late/runtime_13.rs
  - tools/tests/test_runtime_script_binding_audit.py
  - tests/acceptance/runtime-script-binding-audit-owner-sync.md
  - zircon_runtime/src/tests/runtime_absorption/script_binding/inventory.rs
  - zircon_runtime/src/tests/runtime_absorption/script_binding/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/script_binding/gameplay_host.rs
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
last_refined: 2026-07-14
---

# 13 脚本绑定面与反射收束

Runtime 13 current child-owner sync (2026-07-10): `script_binding_boundary` reports `expected_source_file_count = 19`, `expected_test_file_count = 3`, `expected_guard_file_count = 9`, `missing_guard_files = []`, `fixed_host_module_count = 6`, `fixed_host_function_count = 52`, `type_descriptor_count = 2`, `builtin_callback_count = 11`, `gameplay_callback_count = 39`, `macro_host_function_count = 2`, `host_capability_count = 11`, `guard_anchor_count = 9`, `native_ecs_abi_references = []`, `oversized_test_files = []`, `mirror_docs_guard_present = true`, and `risks = []`. The nine guard owners include the two route parents plus ledger/capability/ECS-facade, gameplay-host/mirror, despawn behavior, and Runtime 13 Cargo children. `runtime_13_script_binding_mirror_docs_match_structure_audit_counts` keeps the plan, runtime index, function ledger, M0 review, and interface-convergence mirror aligned; script package gates remain pending.

06 管 VM 插件生命周期（activate/空参数修复/热重载），10 管函数表 ABI 结构——本计划管两者之间的**语义层**：host function/module 注册面、`ZirconScriptType` 反射 marshalling、脚本对 ECS/资产/事件的能力面（capability）治理。

## 现状与证据（2026-06-13 实仓盘点）

- **绑定面三件**：反射宏家族 `zircon_host_function`/`zircon_host_module`/`ZirconScriptType`（`zircon_runtime_reflection_macros` crate 经 `lib.rs:32-34` 再导出）；使用点实测 4 文件——`script/vm/host/builtin_host_modules.rs`（内建宿主模块注册）、`core/framework/script.rs`（契约层）、`script/vm/tests.rs`、`lib.rs`。
- **VM 子系统形状**（`script/vm/` 实测 12 条目）：`backend/`（zr_vm 真实后端 + fallback，06 已细化）、`host/`（宿主函数注册）、`gameplay_host(.rs+/)`（玩法宿主面）、`capability_set.rs`（**能力门控已有雏形**）、`handles.rs`（脚本句柄）、`module/`、`plugin/`、`runtime/` + `runtime_context.rs`、`scene_hook.rs`。
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

- 2026-07-14 owner 硬切同步：`script_binding_boundary` 当前 `expected_source_file_count = 18`、`expected_test_file_count = 3`、`expected_guard_file_count = 9`、`missing_source_files = []`、`missing_guard_files = []`、`mirror_docs_guard_present = true`、`risks = []`。ZrVM `real_backend/host_modules.rs` 已归插件 crate，Runtime13 清单不再读取或计入插件实现 owner。
