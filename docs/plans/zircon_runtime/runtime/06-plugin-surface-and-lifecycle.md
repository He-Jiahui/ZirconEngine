---
related_code:
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/instance.rs
  - ../zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding/src/lib.rs
  - zircon_runtime/src/plugin/runtime_plugin
  - zircon_runtime/src/plugin/runtime_profile
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - docs/engine-architecture/native-plugin-boundary.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_boundary.py
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
status: in_progress
last_refined: 2026-06-14
---

# 06 插件公开面与生命周期收束

## 现状与证据（2026-06-12 重核）

- **公开面实测**：`plugin/mod.rs:27-50` 把 native plugin loader 约 48 个类型/常量全量 `pub use` 到 `plugin` 根：三代 ABI 结构（`NativePluginAbiV1/V2/V3`）、Behavior/ByteSlice/CallbackStatus/EntryReport/HostFunctionTable/OwnedByteBuffer 的 V2/V3 双份、`NativePluginLiveHost` 全家（Command/LoadReport/Outcome）、Runtime 状态快照族（PlayModeSnapshot/StateSnapshot/StateRestoreReport 等）、4 个 STATUS 常量、4 个 ABI_VERSION 常量、4 个 DESCRIPTOR_SYMBOL 常量、2 个 PLAY_MODE 命令常量——与"native loader 退出 runtime 公共主路径"的既定决策（`docs/engine-architecture/native-plugin-boundary.md`，2026-06-12 实测存在）冲突。`plugin` 根的其余导出（manifest/catalog/profile/registration report/extension registry/scene hook/export 族，:17-26、:51-74）属"描述与报告"层，是目标公共面。
- **ABI 三代并存**：版本常量当前版 `ZIRCON_NATIVE_PLUGIN_ABI_VERSION = 3`；V1/V2 在仓内的真实使用者**仅 1 个文件**——`zircon_plugins/native_dynamic_fixture/native/src/lib.rs`（测试夹具，自带本地常量副本 :7-10，并实现 `SyncDescriptorV1/V2` :285-286 以覆测三代协商）；`plugin/export_build_plan/` 生成的宿主文件**不引用**任何版本符号（grep `DESCRIPTOR_SYMBOL_V\d|ABI_VERSION_V\d` 0 命中）。淘汰决策的真实迁移面比旧文预估小得多。
- **ZrVM 空指针现场（精确定位）**：`script/vm/backend/zr_vm_project_backend/real_backend/instance.rs`——`fn call_entry_lifecycle_export(&mut self, export_name: &str, arguments: &[zrvm::Value])`（:58-65）委托 `call_optional_export`（:50-56 经 `call_module_export`）；`activate()`（:73-94）以 `call_entry_lifecycle_export("activate", &[])` 传**空 slice**，deactivate（:98）/saveState（:104）同口径——空 slice 在 sys 边界成为非法指针，触发 `zr_vm_core.dll function.c:1394` 断言（`.codex/sessions/20260611-0416` 根因记录）。修复点在 `call_module_export` 到 `zr_vm_rust_binding_sys` 的参数 marshalling 段。
- **热重载**：`native_plugin_loader/native_plugin_live_host/lifecycle.rs` 的 `pub fn hot_reload_runtime_plugin(`（:32）/`pub fn hot_reload_editor_plugin(`（:40）+ `NativePluginHotReloadState` 回滚（状态字段执行时核验：Grep `NativePluginHotReloadState`，path 同目录）。
- **下游调用面**：`zircon_app/src` 引用 `NativePlugin*` 当前共 7 文件（2026-06-14 实测全列；app NativePlugin current call-site files: 7）：`lib.rs`、`prelude.rs`、`entry/mod.rs`、`entry/export_bootstrap.rs`、`entry/entry_runner/mod.rs`、`entry/entry_runner/bootstrap.rs`、`entry/tests/profile_bootstrap.rs`——M2 收窄的迁移面；`entry/export_bootstrap.rs` 是 Runtime 02 generated/export 收束后的新增调用面，不改变 M2 硬切换要求。
- Fyrox 锚点（每点一行）：`Plugin`（静态）/`DynamicPlugin`（dylib）双 trait + `PluginContainer` — `dev/Fyrox/fyrox-impl/src/plugin/mod.rs`；热重载"序列化状态→unload→重载→恢复" — `dev/Fyrox/fyrox-impl/src/plugin/dylib.rs`。

补充参考锚点（2026-06-13 实测核验，实现型切片动工前先读——index 公约 §7.9）：

- Godot GDExtension：C ABI 入口装载与符号协商的成熟实现（M2 收窄 facade、M3 版本协商失败路径对照）— `dev/godot/core/extension/gdextension_function_loader.{h,cpp}`、`gdextension.{h,cpp}`

## 目标

1. runtime 插件公共面收窄为"描述与报告"层（manifest/catalog/profile/registration report/extension registry/scene hook/export 族）；native loader 实现细节退出 `plugin` 根。
2. ABI 版本策略定稿：单一当前版 V3 + 显式支持矩阵；V1/V2 按"仅夹具使用"的实测结论处置（删除或冻结文档化，带删除条件）。
3. VM plugin lifecycle 修复空参数空指针（最高优先，解锁 07 性能取证），失败路径补真实测试。

## 非目标

- 不改插件 framework DTO 的中性原则（行为留在 `zircon_plugins/*/runtime`）；不动 `zircon_runtime_interface` 的函数表 ABI（另一条收敛线）。
- 不在本计划做 VM 之外的脚本后端工作；fallback 后端行为仅在测试分层中涉及。

### 全局硬约束（继承总计划 §4，违反即返工）

- 硬切换：native 类型迁出 `plugin` 根后不留 re-export；不新增 crate。
- 动态边界只传 ABI-safe 值与序列化负载；非网络语义 server 命名是 blocker。

## 执行前检查清单

1. 活动会话对齐：重读 `.codex/sessions/20260603-2304-plugin-ecosystem-continuation.md` 最新状态（插件注册/导出校验区活跃）；`20260611-0416`（ZrVM 根因所在会话）状态复核。
2. worktree 脏文件检查：`git status --porcelain -- zircon_runtime/src/plugin/ zircon_runtime/src/script/ zircon_app/src/entry/`。
3. 事实重核：
   - `grep -n "pub use native_plugin_loader" zircon_runtime/src/plugin/mod.rs`（核 :27-50 块）
   - `grep -rn "call_entry_lifecycle_export" zircon_runtime/src/script`
   - `grep -rln "NativePlugin" zircon_app/src`（核 6 文件清单）
   - `grep -rln "NativePluginAbiV1\|DESCRIPTOR_SYMBOL_V1" zircon_plugins`（核"仅夹具"仍成立）
4. ZrVM 环境确认：`../../zr_vm` checkout 存在且与 `zr-vm-real-backend` feature 可编译（M1 需要真实后端）。
5. 基线记录：`cargo test -p zircon_runtime --lib plugin --locked` 与 `--lib script --locked` 通过数记入状态节。

## 里程碑

### M1 VM 生命周期阻塞修复（最高优先，解锁 07 性能取证）

#### 切片 1.1 空参数 marshalling 修复

- 目标文件：`zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/instance.rs`（及其 `call_module_export` 下行的 sys 调用段，执行时核验确切文件：Grep `call_module_export`，path `zircon_runtime/src/script/vm/backend/zr_vm_project_backend`）。
- 改动形态（方案草案，执行时按 ZrVM C ABI 约定定稿）：空参数数组改为传"合法非空指针 + len=0"或 ZrVM 约定的显式空参数协议；实现为绑定侧防御（空 slice 分支换静态空槽指针，签名草案执行时定稿）。若必须 ZrVM 侧修，先落绑定侧防御并在 `docs/zircon_runtime/dynamic_api/session.md` 记录上游 issue 与版本配对要求（回写子计划 01 的 zr_vm 治理条目）。
- 调用方迁移：无公共面变化（`call_entry_lifecycle_export` 四个调用点 :83/:98/:104/:125 行为自动修复）。
- 验收：`examples/vampire` 真实启动不再触发 `function.c:1394` 断言（或上游 issue + 绑定防御双证据）。
- DoD：`cargo test -p zircon_runtime --lib vampire_project_session --features zr-vm-real-backend --locked -- --nocapture --test-threads=1` 不再因该断言失败。

#### 切片 1.2 生命周期失败路径测试分层

- 目标文件：`script/vm` 测试位（执行时核验既有测试树：`ls zircon_runtime/src/script/vm/`，`vm/tests.rs` 已存在）。
- 改动形态：四类路径补失败测试，按 feature 分层——`zr-vm-real-backend` 门控的真实后端测试与 fallback 路径测试分开：
  - activate 成功/失败（坏入口模块）
  - deactivate 幂等
  - 空参数路径（修复回归锚）
  - 坏符号（不存在的 export → `is_optional_export_missing` 分支 :53 的正反例）
- 调用方迁移：无。
- 验收（测试名草案）：`vm_lifecycle_activate_with_empty_arguments_does_not_trip_native_assertion`（real-backend 门控）、`vm_lifecycle_missing_optional_export_returns_none_not_error`、`vm_lifecycle_bad_entry_module_surfaces_vm_error`。
- DoD：四类路径各有测试；real/fallback 两层命令均绿。

#### 切片 1.3 real-backend 验证收尾（2026-06-12 二次细化新增，承接 1.1/1.2 的"runtime Cargo 待验证"状态）

- 目标文件：无新代码改动（验证 + 1.2 剩余测试分层补齐）。
- 改动形态（按依赖序三步）：
  1. **fallback 层测试先行**：1.2 中不需 real backend 的失败路径测试（坏入口模块、坏符号、deactivate 幂等）先落地并跑绿——不受 real-backend 编译超时影响；
  2. **编译超时破解**：`--features zr-vm-real-backend` 组合实测 300s 编译超时（状态节证据）——用 `tools/dev-fast-build.ps1` 共享 `CARGO_TARGET_DIR` 预热 + 包内最小 feature 复跑；破解方法与 07 计划切片 0.2（profiling 构建超时）同源，结论双向回写；
  3. **real-backend 回归**：跑 M1 DoD 命令确认 `function.c:1394` 不再触发，同时确认 binding 侧 sentinel 修复（`../zr_vm/.../lib.rs`）与 runtime 侧行为一致。
- 调用方迁移：无。
- 验收：`vm_lifecycle_activate_with_empty_arguments_does_not_trip_native_assertion`（real-backend 门控）首次真实跑绿；fallback 层三测试绿。
- DoD：状态节 1.1/1.2 行的"runtime Cargo 待验证"翻转为"完成"，附命令输出摘要与编译耗时。

#### M1 测试阶段（milestone-first）

- 切片期：`cargo check -p zircon_runtime --lib --locked`
- 里程碑末：
  - `cargo test -p zircon_runtime --lib script::vm --locked -- --nocapture`
  - 有真实 ZrVM 环境：`cargo test -p zircon_runtime --lib vampire_project_session --features zr-vm-real-backend --locked -- --nocapture --test-threads=1`
- 验收证据：断言不再触发的运行记录；文档 `docs/zircon_runtime/dynamic_api/session.md` 刷新。

### M2 native loader 公共面收窄

#### 切片 2.1 `plugin::native` 子命名空间收口

- 目标文件：`zircon_runtime/src/plugin/mod.rs`（删 :27-50 的 native 全量 `pub use`）；`plugin/native_plugin_loader/`（模块声明调整）。
- 改动形态（二选一，执行时定稿并记录理由）：
  - (a) native loader 全家改 `pub(crate)`，对外仅保留窄 facade（load report / live host 命令面：`NativePluginLoadReport`、`NativePluginLiveHostCommand`、`NativePluginLiveHostOutcome` 一类，不暴露 ABI 结构体/byte buffer/符号常量）；
  - (b) 收入 `plugin::native` 子命名空间，停止从 `plugin` 根 re-export。
  - 共同点：ABI 结构体、ByteSlice/ByteBuffer、STATUS/SYMBOL/VERSION 常量一律不出现在 `plugin` 根。
- 调用方迁移（实测全列，6 文件）：`zircon_app/src/{lib.rs, prelude.rs, entry/mod.rs, entry/entry_runner/mod.rs, entry/entry_runner/bootstrap.rs, entry/tests/profile_bootstrap.rs}` 改新路径或经窄 facade；runtime 内部调用方枚举：Grep `plugin::Native`，path `zircon_runtime/src`。同切片删除旧导出，不留桥（硬切换）。
- 验收：`plugin_root_exports_stay_descriptive_layer_only`（结构测试，归属 `zircon_runtime/src/tests/plugin_extensions/` 既有树）——断言 `plugin/mod.rs` 源文本不再含 `NativePluginAbi`/`ByteBuffer`/`DESCRIPTOR_SYMBOL` 等实现词根的根级导出。
- DoD：`cargo check -p zircon_app --locked` 通过且 `plugin` 根无 native 实现类型。

#### 切片 2.2 测试与文档迁移

- 目标文件：native loader 相关测试迁到隔离 namespace（执行时枚举：Grep `NativePluginAbi`，path `zircon_runtime/src/tests`）；`docs/engine-architecture/native-plugin-boundary.md` 口径刷新。
- 改动形态：测试 `use` 路径批量改新 namespace；文档把"目标态"改写为"现状"。
- 调用方迁移：仅测试文件。
- 验收：boundary 文档与代码一致；结构测试进常驻树。
- DoD：`cargo test -p zircon_runtime --lib plugin --locked` 全绿。

#### M2 测试阶段（milestone-first）

- `cargo check -p zircon_runtime --lib --locked`
- `cargo test -p zircon_runtime --lib plugin --locked -- --nocapture`
- `cargo test -p zircon_app --locked`
- `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked`
- 验收证据：公共面结构测试通过；`native-plugin-boundary.md` 口径一致。

### M3 ABI 版本策略定稿

#### 切片 3.1 V1/V2 处置（按"仅夹具使用"实测结论）

- 目标文件：`plugin/native_plugin_loader/`（V1/V2 描述符/EntryReport/协商分支所在文件，执行时枚举：Grep `AbiV1|AbiV2|_V1|_V2`，path `zircon_runtime/src/plugin/native_plugin_loader`）；`zircon_plugins/native_dynamic_fixture/native/src/lib.rs`（夹具同步改造）。
- 改动形态：实测 V1/V2 无生产使用者（export 宿主文件不引用版本符号；唯一使用者是协商测试夹具）→ 默认决策**直接删除 V1/V2**（硬切换）：删 `NativePluginAbiV1/V2`、`EntryReportV1/V2`、`SyncDescriptorV1/V2` 适配段、`ABI_VERSION_V1/V2`、`DESCRIPTOR_SYMBOL_V1/V2`；夹具改为"V3 成功 + 未知版本被拒"两用例（保留版本协商失败路径覆盖，不保留旧版实现）。若执行时发现仓外生产插件依赖 V1/V2（决策门槛：`zircon_plugins` 之外的发布物清单），转冻结文档化并写明删除条件。
- 调用方迁移：夹具 1 文件 + loader 内协商分支；枚举命令同上。
- 验收：`native_plugin_loader_rejects_unknown_abi_version_with_explicit_report`、`native_plugin_loader_accepts_current_v3_descriptor`（loader 测试树）。
- DoD：Grep `AbiV1|AbiV2` 全仓 0 命中（或冻结文档化判词落地）；插件 workspace 全量 check 通过。

#### 切片 3.2 热重载回滚失败注入测试（对照 Fyrox）

- 目标文件：`native_plugin_loader/native_plugin_live_host/lifecycle.rs`（:32/:40 两入口）+ 同目录测试位。
- 改动形态：补失败注入测试——"快照→重载→恢复/回滚"路径的两个失败点：重载后符号缺失 → 回滚成功；状态恢复失败 → 回滚成功且 `NativePluginHotReloadState` 报告一致（对照 Fyrox dylib.rs 的状态序列化重载语义）。
- 调用方迁移：无。
- 验收：`hot_reload_missing_symbol_after_reload_rolls_back_to_previous_instance`、`hot_reload_state_restore_failure_rolls_back_and_reports`。
- DoD：`cargo test -p zircon_runtime --lib native_plugin --locked -- --nocapture` 含上述测试全绿。

#### M3 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib native_plugin --locked -- --nocapture`（hot_reload/abi 过滤词）
- `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked`
- `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked`（夹具改造回归）
- 验收证据：ABI 支持矩阵文档（落 `native-plugin-boundary.md`）；回滚测试；被删旧版无残留引用。
- 文档：`docs/zircon_plugins/first_party_runtime_catalog.md`、export 契约文档刷新。

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M1 | 1.1 空参数修复 | 代码完成，runtime Cargo 待验证 | 2026-06-12 | 修复最低共享层 `../zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding/src/lib.rs`：空参数导出调用传合法 sentinel pointer + len=0；`ZrVmPluginInstance` 保持 `call_entry_lifecycle_export(..., &[])` 正常路径；binding `call_module_export_accepts_empty_argument_slice` 通过 |
| M1 | 1.2 失败路径测试 | 部分完成，runtime Cargo 待验证 | 2026-06-12 | 新增 binding 侧 `call_module_export_accepts_empty_argument_slice` 并通过；既有 `project_session_preserves_module_state_between_export_calls` 通过并覆盖 session 空参数生命周期；`vampire_project_session_starts_paused_until_start_button_click --features zr-vm-real-backend` 300s 编译超时，runtime real-backend/fallback 分层测试仍待补齐 |
| M2 | 2.1 native 收口 | 待开始 | — | — |
| M2 | 2.2 测试/文档迁移 | 待开始 | — | — |
| M3 | 3.1 V1/V2 处置 | 待开始 | — | — |
| M3 | 3.2 回滚失败注入 | 待开始 | — | — |
| 横切 | Cargo/plugin pending gate | code_static_pending_cargo | 2026-06-13 | 新增 `runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation`，保持 Runtime 06 在 `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` 验证线通过前为 `in_progress`：M1 空参数修复只记录 binding 与 session 静态/局部证据，真实后端 `vampire_project_session`、runtime fallback、plugin/native plugin、app 与 `zircon_plugins` workspace Cargo 仍待 clean lane 重跑；M2/M3 native 收口、测试/文档迁移、V1/V2 处置、回滚失败注入仍为待开始；M0 评审同步引用 `native_plugin_public_surface` 与 `root_reexport_count = 70`，以固定当前 public-surface debt 证据名。 |
| 横切 | plugin surface/lifecycle 结构镜像 | structure_audit_static_passed_cargo_pending | 2026-06-14 | 新增 `plugin_surface_lifecycle_boundary` 并接入总审计；镜像 Runtime 06 source 10/10、doc 5/5、frontmatter `in_progress`、`last_refined = 2026-06-14`、`native_plugin_public_surface.m4_gate_status = migration-debt-present`、`root_reexport_count = 70`、native root re-export 70/70、public native re-export locations 1/1、debt groups 5/5、unclassified native symbols 0/0、native loader V1/V2 implementation files 6/6、`zircon_plugins` V1/V2 usage files 1/1（仅 `native_dynamic_fixture`）、export_build_plan V1/V2 usage 0/0、app NativePlugin current call-site files: 7；本切片只做静态镜像和计划同步，不改 plugin/native/script 生产代码，Cargo 仍等待 active lanes 清空后按本计划验证线重跑。 |
| 横切 | plugin surface/lifecycle 静态验证 | static_validation_passed_cargo_blocked | 2026-06-14 | `python -m py_compile` 覆盖 `native_plugin_public_surface.py`、`plugin_surface_lifecycle_boundary.py` 与总审计脚本通过；直接运行 `plugin_surface_lifecycle_boundary_audit` 报告 `risks = []`；聚合 `audit_runtime_structure.py --json` 中 Runtime 06 镜像断言通过，`native_plugin_public_surface` 当前按设计保留迁移债风险且 root count/debt/unclassified 断言为 70/5/0；Markdown 输出含 Runtime 06 边界、`native-bridge-method-public-debt` 与 7 个 app NativePlugin 文件；`rustfmt --edition 2021 --check` 覆盖 plan-status early/recent guards 通过；冲突标记、尾随空白、`git diff --check` 通过（文档 LF→CRLF 警告 only）；Cargo/rustc 通道仍被其他 `cargo check -p zircon_runtime --lib --no-default...` lane 占用，因此未声明 Runtime 06 Cargo 通过。 |
| 横切 | plugin surface/lifecycle 镜像文档守卫 | mirror_docs_static_passed_cargo_pending | 2026-06-14 | 新增 `runtime_absorption::plugin_surface_lifecycle::runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts` 并接入 `runtime_absorption/mod.rs` 与 recent static guard 清单；Runtime 06 mirror-doc guard anchors: `plugin_surface_lifecycle_boundary`; `expected_source_file_count = 10`; `expected_doc_file_count = 5`; native root re-export 70/70; M4 gate `migration-debt-present`; debt groups 5/5; unclassified native symbols 0/0; public native re-export locations 1/1; app NativePlugin current call-site files: 7; native loader V1/V2 implementation files 6/6; `zircon_plugins` V1/V2 usage files 1/1; export_build_plan V1/V2 usage 0/0; `mirror_docs_guard_present = true`; `risks = []`; `runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts`。验证：rustfmt check、Python py_compile、direct `plugin_surface_lifecycle_boundary_audit`、aggregate `audit_runtime_structure.py --json` Runtime 06 assertions、standalone rustc 1/1、stale old-count scan、conflict marker scan 通过。本切片只新增静态守卫、Python 审计锚点和镜像文档写回，不改 plugin/native/script 生产代码；`script::vm/vampire_project_session/plugin/native_plugin/app/plugins` Cargo/native gate 仍 pending。 |

基线数值（开工首日记录）：

- `plugin/mod.rs` native 导出基线：约 48 类型/常量（:27-50；重核命令见执行前检查清单）
- V1/V2 使用者基线：1 文件（native_dynamic_fixture）；export 宿主文件引用：0
- `zircon_app` NativePlugin 引用基线：6 文件
- `function.c:1394` 断言复现命令：`cargo test -p zircon_runtime --lib vampire_project_session --features zr-vm-real-backend --locked -- --nocapture --test-threads=1`
- `cargo test -p zircon_runtime --lib plugin --locked` 通过数基线：__

## 风险与协调

- `20260603-2304-plugin-ecosystem-continuation` 会话活跃于插件注册/导出校验区，M2/M3 执行前强制重读其笔记；`20260611-0416` 是 ZrVM 根因记录所在，M1 前复读，**禁止回退其 worktree 改动**。
- M2 改动触及 `zircon_app` 入口装配（6 文件含 `entry_runner/bootstrap.rs`），与 `zircon_app::entry` 的源断言测试（活动会话 touched）对齐；prelude/lib.rs 的再导出若被外部工具消费，收窄前确认无隐藏调用方。
- ZrVM 是仓外依赖（`../../zr_vm`）：M1 若需上游修复，版本配对要求记入子计划 01 的 zr_vm 治理条目（`runtime-tech-stack.md`），双计划交叉引用。
- M3 删除 V1/V2 是硬切换：夹具与 loader 协商分支必须同一变更内闭合，禁止"先删 loader 后改夹具"的中间态提交。
