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
last_refined: 2026-07-05
---

# 13 脚本绑定面与反射收束

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

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M0 | 0.1 宿主面清册 | completed_static_passed | 2026-06-13 | 新增 `docs/zircon_runtime/script/vm/host/function_ledger.md`；复核 `builtin_host_modules.rs`、`gameplay_host.rs`、`bridge_host_module.rs`、`host_export_registry.rs`、`reflection_docs/*`，记录固定 6 模块 / 52 函数 / 2 类型描述符和 bridge 动态模块形状；源码计数 `builtin_callbacks=11` / `gameplay_callbacks=39` / `macro_host_functions=2`，合计 52；已读锚点 `dev/godot/core/extension/extension_api_dump.cpp`、`dev/Fyrox/fyrox-impl/src/script/mod.rs`、`dev/Piccolo/engine/source/runtime/core/meta/reflection/reflection.h`。Cargo 未跑，M0 为文档清册切片。 |
| M0 | 0.2 marshalling 判词 | completed_static_passed | 2026-06-13 | `function_ledger.md` 记录 value descriptors / host handles / serialized payloads 三分类，声明 `zircon_runtime_interface::reflect` 为 editor/remote schema 面，VM host calls 仍走 `ScriptHostValue` 描述符；裁决当前 ECS 访问路径为 `zr.zircon.gameplay` gameplay facade，`ZrHostEcsApiV1` 保持 native/plugin ABI 层；anchor scan、冲突标记与尾随空白扫描为空，`git diff --check` 仅 index LF/CRLF 提示。Cargo 未跑，M0 为文档判词切片。 |
| M1 | 1.1 清册守卫 | code_static_pending_cargo | 2026-06-13 | 新增 `zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs::host_function_registry_matches_documented_ledger` 与负例 `host_function_registry_ledger_guard_rejects_missing_entry`，并在 `runtime_absorption/mod.rs` 挂接；守卫固定 6 模块 / 52 函数 / 2 类型描述符、capability 清册、bridge 动态模块形状、M0 marshalling/ECS 判词和 Runtime 13 状态锚点。2026-06-13 本轮重核：`rustfmt --edition 2021 --check` 覆盖 `script_host_ledger.rs`、`runtime_absorption/mod.rs`、`gameplay_host/tests.rs`；anchor scan、冲突标记与尾随空白扫描为空；`git diff --check` 仅 `gameplay_host/tests.rs` 与 `runtime_absorption/mod.rs` LF/CRLF 提示。Cargo/独立 rustc 待编译通道空闲。 |
| M1 | 1.2 capability 测试 | code_static_pending_cargo | 2026-06-13 | 新增 `zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs::host_capability_representatives_are_declared_on_registered_modules` 与 `host_function_without_required_capability_is_rejected_with_explicit_error`；固定 capability 类走真实 `register_builtin_host_modules` 描述符，`bridge.call` 走最小动态 bridge 描述符；正向守卫检查模块声明与函数 required capability，反向守卫以空 `CapabilitySet` 要求显式 `missing capability ...` 拒绝。本轮重核：Runtime 13 anchor scan 找到 capability 正反守卫，`rustfmt --edition 2021 --check` 通过，冲突标记/尾随空白扫描为空，`git diff --check` 仅 LF/CRLF 提示。Cargo 待编译通道空闲。 |
| M2 | 2.1 句柄失效语义 | code_static_pending_cargo | 2026-06-13 | 新增 `zircon_runtime/src/script/vm/gameplay_host/tests.rs::script_held_entity_handle_reports_invalid_after_despawn`；测试用真实 `LevelSystem` + `register_gameplay_host_module` + `with_script_runtime_call_context` 覆盖脚本持有实体 id 的 live read、`despawn`、post-despawn `position_json == "null"`，以及 stale `set_position` 写访问必须由 world 层显式拒绝。本轮重核：anchor scan 找到该测试锚，`rustfmt --edition 2021 --check` 通过，冲突标记/尾随空白扫描为空，`git diff --check` 仅 LF/CRLF 提示。Cargo 待编译通道空闲。 |
| M2 | 2.1 句柄失效语义 typed 诊断同步 | static_updated_cargo_blocked_by_render_pbr_compile_drift | 2026-07-04 | `frameworks_02_m3_gameplay_host_stale_entity_typed_diagnostic_static_updated_cargo_blocked_render_pbr_helpers`：复核 `script_held_entity_handle_reports_invalid_after_despawn` 的当前失败根因不是 stale entity 被写访问接受，而是测试仍断言旧 `missing node` 文案；底层 `World::update_transform` 已通过 `SceneError::MissingEntity` 拒绝缺失实体，VM host-call 边界字符串化为 `cannot update transform for missing entity ...`。本轮只同步 `script/vm/gameplay_host/tests/combat_lifecycle.rs`，锁定新版 typed-error 诊断并显式拒绝旧 node wording 回流；不恢复 fallback entity、旧 String-error path、compat shim 或 gameplay host 专用旁路。验证：touched-file rustfmt check 通过；exact Cargo 过滤器 `cargo test -p zircon_runtime --lib script_held_entity_handle_reports_invalid_after_despawn --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-extension-registry-dotted-coremin-0704 --message-format short --color never -- --nocapture --test-threads=1` 在当前 worktree lib-test 编译期被外部 render/Plan08 `project_scenes.rs` PBR helper 缺失阻断，未执行目标测试、不计 Cargo pass。 |
| M2 | 2.2 访问路径收束 | code_static_pending_cargo | 2026-06-13 | 新增 `zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs::script_ecs_access_path_stays_on_gameplay_facade_not_native_ecs_abi`；守卫 `function_ledger.md` 的 `zr.zircon.gameplay` / `ScriptRuntimeCallContext` / `ZrHostEcsApiV1` 分层判词，并遍历 `zircon_runtime/src/script/**/*.rs` 禁止脚本层直接出现 native ECS ABI 符号。本轮重核：anchor scan 找到 gameplay facade / native ECS ABI 分层锚，`rustfmt --edition 2021 --check` 通过，冲突标记/尾随空白扫描为空，`git diff --check` 仅 LF/CRLF 提示。Cargo 待编译通道空闲。 |
| M3 | 验证门守卫 | cargo_validation_pending_guarded | 2026-06-13 | 新增 `runtime_absorption::plan_status::runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass`，要求 Runtime 13 在 `cargo test -p zircon_runtime --lib script --locked -- --nocapture` 有真实通过证据前保持 `in_progress`，并锁定 Runtime 13 计划、总索引 P16/子计划行、`function_ledger.md` 与 M0 评审里的 host ledger、capability、脚本持有实体失效和 gameplay facade/native ECS ABI 分层锚点。2026-06-14 复跑：scoped `cargo test -p zircon_runtime --lib --locked script::vm -- --nocapture` 48/48 passed；broader `cargo test -p zircon_runtime --lib script --locked -- --nocapture` 仍 pending，当前失败在非本切片 scene/vampire/UI 测试。 |
| 横切 | Script binding 结构审计 owner | structure_audit_static_passed_cargo_pending | 2026-06-13 | 新增 `.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/script_binding_boundary.py` 并接入 `audit_runtime_structure.py`，把 Runtime 13 的 host ledger、builtin/gameplay/macro callback 计数、capability 清册、bridge 动态模块形状、`zr.zircon.gameplay` facade、native ECS ABI 旁路禁用、Rust guard anchors 与 pending Cargo gate 统一纳入结构审计。Targeted audit facts：`expected_source_file_count = 19`、`expected_test_file_count = 3`、`fixed_host_module_count = 6`、`fixed_host_function_count = 52`、`type_descriptor_count = 2`、`builtin_callback_count = 11`、`gameplay_callback_count = 39`、`macro_host_function_count = 2`、`host_capability_count = 11`、`guard_anchor_count = 9`、`native_ecs_abi_references = []`、`oversized_test_files = []`、`mirror_docs_guard_present = true`、`risks = []`。Cargo 仍按 M3 gate 保持 pending。 |
| 横切 | Script binding 镜像文档守卫 | mirror_docs_static_passed_cargo_pending | 2026-06-14 | 新增 `runtime_absorption::script_binding::runtime_13_script_binding_mirror_docs_match_structure_audit_counts` 并在 `runtime_absorption/mod.rs` 挂接，锁定 Runtime 13 计划、runtime index、M0 review、runtime-interface convergence 与 `function_ledger.md` 必须同步 `script_binding_boundary` 的 `expected_source_file_count = 19`、`expected_test_file_count = 3`、`fixed_host_module_count = 6`、`fixed_host_function_count = 52`、`type_descriptor_count = 2`、`builtin_callback_count = 11`、`gameplay_callback_count = 39`、`macro_host_function_count = 2`、`host_capability_count = 11`、`guard_anchor_count = 9`、`native_ecs_abi_references = []`、`oversized_test_files = []`、`mirror_docs_guard_present = true` 与 `risks = []`。Cargo script filters 仍 pending。 |
| 横切 | Gameplay host owner split | folder_split_static_passed_script_vm_cargo_broader_gate_pending | 2026-06-14 | `zircon_runtime/src/script/vm/gameplay_host.rs` 保留 `zr.zircon.gameplay` descriptor 与 39 个 `HostExportFunction::new` 注册锚，具体回调拆入 `gameplay_host/{combat,components,input,lifecycle,navigation,script_bindings,transform,values}.rs`；新增 `runtime_13_gameplay_host_owner_split_keeps_domain_files` 锁定 domain files、主注册文件/子 owner 400 行预算、代表回调注册锚、value helper owner 与 navigation-aware movement 依赖。同步 `script_binding_boundary` 为 `expected_source_file_count = 19`、`guard_anchor_count = 9`，`builtin_callback_count = 11`、`gameplay_callback_count = 39`、`macro_host_function_count = 2` 不变。验证：rustfmt check、Python py_compile、direct `script_binding_boundary_audit` risks=0/source 19/19/guards 9/9、aggregate `audit_runtime_structure.py --json` Runtime 13/plan-status assertions、standalone `script_binding.rs` 2/2、standalone status-output 2/2、`cargo test -p zircon_runtime --lib --locked script::vm -- --nocapture` 48/48 passed；broader `cargo test -p zircon_runtime --lib script --locked -- --nocapture` 仍为 script Cargo filters pending，当前失败在非本切片测试 `scene_assets_keep_script_only_entities_as_empty_nodes`、`vampire_example_manifest_scene_and_scripts_are_importable`、`material_virtualized_descriptors_expose_mui_web_aliases`。 |
| 横切 | Gameplay host predicate functions for real ZR VM | focused_behavior_passed_broader_script_gate_pending | 2026-06-16 | 新增并验证 `gameplay.entity_exists(entity)` 与 `gameplay.script_number_at_most(entity, property, threshold, fallback)`，让 Vampire 脚本把实体存在性与数值阈值比较留在 Rust host facade 内执行，避免 real ZR VM 对 host-returned entity/numeric 直接比较路径不稳定。`docs/zircon_runtime/script/vm/host/function_ledger.md`、`script_binding_boundary.py`、`script_binding.rs` 与 `script_host_ledger.rs` 同步保持 6 模块 / 52 固定函数 / 39 gameplay callbacks；复用真实后端测试二进制验证 `gameplay_host_script_property_match_and_heal_update_bindings` 1/1 与 `host_function_registry_matches_documented_ledger` 1/1 通过。Broader `cargo test -p zircon_runtime --lib script --locked` 仍按 M3 gate pending。 |
| 横切 | Script binding current audit recheck | script_binding_current_audit_static_passed_cargo_pending | 2026-06-20 | 本轮只复核 Runtime 13 当前脚本绑定/反射结构事实，生产代码未改：`script_binding_boundary_audit` 报告 source files 19/19、guard/test files 3/3、fixed host modules/functions/types 6/52/2、callback counts 11/39/2、host capabilities 11/11、Runtime 13 guard anchors 9/9、missing fixed modules/capabilities/ledger doc anchors/bridge anchors/gameplay facade anchors/Cargo gate anchors 均为空、native ECS ABI references []、oversized test files []、`mirror_docs_guard_present = true`、`risks = []`。验证通过：Python py_compile、direct `script_binding_boundary_audit` risks=[]、standalone `script_binding.rs` 2/2；broader `cargo test -p zircon_runtime --lib script --locked` 仍按 Runtime 13 gate pending。 |
| 横切 | Runtime 13 script binding Markdown renderer split | script_binding_markdown_split_static_passed_cargo_deferred_tests_deferred | 2026-06-21 | 状态锚 `script_binding_markdown_split_static_passed_cargo_deferred_tests_deferred`；新增 `script_binding_markdown.py` 承接 `render_script_binding_boundary_markdown(...)`，`script_binding_boundary.py remains the 352-line audit/risk owner`，`Markdown owner is 106 lines`；总审计入口改从新 renderer owner 导入，直接审计仍报告 source files 19/19、guard/test files 3/3、fixed host modules/functions/types 6/52/2、callback counts 11/39/2、host capabilities 11/11、Runtime 13 guard anchors 9/9、missing fixed modules/capabilities/ledger doc anchors/bridge anchors/gameplay facade anchors/Cargo gate anchors 均为空、native ECS ABI references []、oversized test files []、`mirror_docs_guard_present = true`、`risks = []`，renderer module 为 `runtime_structure_audits.script_binding_markdown`；验证：Python py_compile、direct audit、standalone script_binding 2/2、standalone plan-status 33/33 通过，broader script Cargo gate 仍 deferred。 |
| 横切 | Runtime 13 Script binding 2026-07-01 current audit recheck | script_binding_20260701_current_audit_static_passed_cargo_deferred | 2026-07-01 | 状态锚 `script_binding_20260701_current_audit_static_passed_cargo_deferred`；复核当前脚本绑定/反射结构事实，生产代码未改：`script_binding_boundary_audit` 报告 `expected_source_file_count = 19`、source files 19/19、guard/test files 3/3、fixed host modules/functions/types 6/52/2、callback counts 11/39/2、host capabilities 11/11、Runtime 13 guard anchors 9/9、missing fixed modules/capabilities/ledger doc anchors/bridge anchors/gameplay facade anchors/Cargo gate anchors 均为空、native ECS ABI references []、oversized test files []、`mirror_docs_guard_present = true`、`risks = []`。同轮 full `audit_runtime_structure.py --json` 风险汇总为 `{}`；standalone `plan_status.rs` 41/41 通过。broader script Cargo gate 仍 deferred，因为外部 Cargo/rustc 通道 active。 |
| 横切 | Runtime 15 M3 script host ledger guard folder-backed split | runtime_15_script_host_ledger_guard_folder_backed_static_passed_cargo_deferred | 2026-07-05 | `Runtime 15 M3 script host ledger guard folder-backed split` 将 `runtime_absorption/script_host_ledger.rs` 收束为 route owner；清册常量与 ledger helper 移入 `script_host_ledger/catalog.rs`，清册正反守卫移入 `script_host_ledger/ledger.rs`，capability fixture 与 bridge fixture 移入 `script_host_ledger/capability_fixture.rs`，capability 正反守卫移入 `script_host_ledger/capability.rs`，gameplay facade/native ECS ABI 边界移入 `script_host_ledger/ecs_facade.rs`，结构守卫为 `script_host_ledger/split_layout.rs::runtime_15_script_host_ledger_guard_is_folder_backed`；该切片只整理 Runtime 13 absorption guard 测试 owner，不改 script VM/host 生产行为；Cargo gate deferred。 |
| 横切 | Runtime 15 M3 script-binding route-owner split | runtime_15_script_binding_route_owner_split_static_passed_cargo_deferred | 2026-07-05 | `Runtime 15 M3 script-binding route-owner split` 将 `runtime_absorption/script_binding.rs` 收束为 route owner；Runtime 13 source/test/gameplay owner 清单移入 `script_binding/inventory.rs`，镜像文档守卫 `runtime_13_script_binding_mirror_docs_match_structure_audit_counts` 移入 `script_binding/mirror_docs.rs`，gameplay host owner 守卫 `runtime_13_gameplay_host_owner_split_keeps_domain_files` 移入 `script_binding/gameplay_host.rs`，文件存在/行数/计数 helper 移入 `script_binding/support.rs`，结构守卫为 `script_binding/split_layout.rs::runtime_15_script_binding_route_owner_is_folder_backed`；该切片只整理 Runtime 13 absorption mirror 测试 owner，不改 script VM/host 生产行为；Cargo gate deferred active lanes。 |
| 横切 | VM plugin management policy typed validation errors | runtime_15_vm_plugin_management_policy_typed_errors_static_passed_cargo_deferred | 2026-06-27 | Runtime 15 F5 follow-up：`script/vm/plugin/management_policy/error.rs` 新增 `VmPluginManagementPolicyError` / `VmPluginManagementPolicyResult`，并把 garbage-collection、memory 与 aggregate management policy validation 从 `Result<(), String>` 收敛到 typed variants；`script/vm/mod.rs` 与 `script/mod.rs` 导出 typed surface，management TOML schema、hot reload policy、GC/memory vocabulary 和 slot lifecycle 状态投影不变。新增 `review_f5_vm_plugin_management_policy_uses_typed_validation_errors` 锁定无 String-error 回流；scoped rustfmt/static scans 通过，broader script Cargo gate 仍 deferred。 |

基线：反射宏使用点 4 文件；`script/vm` 条目已在 M0.1 复核为 backend/host/gameplay/runtime/plugin 等分区，固定 host ledger 为 6 模块 / 52 函数 / 2 类型描述符；`capability_set` 已存在（门控雏形）。`script_binding_boundary` 进一步锁定 audited source files 19/19、guard/test files 3/3、callback counts 11/39/2、host capability anchors 11/11、Runtime 13 guard anchors 9/9、native ECS ABI references 0、`mirror_docs_guard_present = true`、risks = []；`gameplay_host.rs` 已降为注册/descriptor owner，玩法输入、transform、component、combat、lifecycle、navigation 与 value helper 分别由 folder-backed domain files 承载。它不替代 large-file owner 对 `script/vm` 大文件的独立治理。

## 风险与协调

- 与 06 共享 `script/vm`（其 M1 收尾在飞）、与 10 共享 ABI 口径（marshalling 规则必须引用其 M0 清册）、与 08 共享实体句柄语义——三处互引，先开工者登记。
- 宿主面清册若揭出无 capability 门控的函数，逐个补门是行为变化：按"先文档现状、再独立切片收紧"两步走，禁止清册切片夹带收紧。
