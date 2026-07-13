---
related_code:
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/plugin/native.rs
  - zircon_runtime/src/plugin/native_plugin_loader
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/instance.rs
  - zircon_runtime/src/plugin/runtime_plugin
  - zircon_runtime/src/plugin/runtime_plugin/descriptor/builder/runtime_plugin_descriptor_builder.rs
  - zircon_runtime/src/plugin/runtime_profile
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - examples/vampire/assets/shaders/default_pbr.zmeta
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - zircon_runtime/src/tests/runtime_absorption/plugin_surface_lifecycle.rs
  - zircon_runtime/src/tests/runtime_absorption/plugin_surface_lifecycle/inventory.rs
  - zircon_runtime/src/tests/runtime_absorption/plugin_surface_lifecycle/lifecycle_fallback.rs
  - zircon_runtime/src/tests/runtime_absorption/plugin_surface_lifecycle/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/plugin_surface_lifecycle/native_loader_namespace.rs
  - zircon_runtime/src/tests/runtime_absorption/plugin_surface_lifecycle/split_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/plugin_surface_lifecycle/support.rs
  - docs/engine-architecture/native-plugin-boundary.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_markdown.py
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
status: in_progress
last_refined: 2026-07-12
---

# 06 插件公开面与生命周期收束

## 现状与证据（2026-06-12 重核）

- **公开面实测**：`plugin/mod.rs:27-50` 把 native plugin loader 约 48 个类型/常量全量 `pub use` 到 `plugin` 根：三代 ABI 结构（`NativePluginAbiV1/V2/V3`）、Behavior/ByteSlice/CallbackStatus/EntryReport/HostFunctionTable/OwnedByteBuffer 的 V2/V3 双份、`NativePluginLiveHost` 全家（Command/LoadReport/Outcome）、Runtime 状态快照族（PlayModeSnapshot/StateSnapshot/StateRestoreReport 等）、4 个 STATUS 常量、4 个 ABI_VERSION 常量、4 个 DESCRIPTOR_SYMBOL 常量、2 个 PLAY_MODE 命令常量——与"native loader 退出 runtime 公共主路径"的既定决策（`docs/engine-architecture/native-plugin-boundary.md`，2026-06-12 实测存在）冲突。`plugin` 根的其余导出（manifest/catalog/profile/registration report/extension registry/scene hook/export 族，:17-26、:51-74）属"描述与报告"层，是目标公共面。
- **M1.2/M2.1/M2.2/M3.1/M3.2 当前状态（2026-06-16）**：M1.2 fallback 层失败路径测试已 folder-backed 落地在 `zircon_runtime/src/script/vm/tests/lifecycle_failures.rs`，并由 `runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed` 锁定 `fallback lifecycle failure tests 4/4`；focused `vm_lifecycle_fallback` Cargo 已在 `--no-default-features --features core-min` 下通过 5/5，其中包含四个 fallback lifecycle 测试和结构守卫。real-backend 已确认 `ZR_VM_RUST_BINDING_LIB_DIR` 与 DLL 路径后越过 ZrVM 环境初始化；后续暴露的 shader `.zasset` bincode 反序列化错误已在 `ArtifactCacheShaderImportRedirectAsset` / `ArtifactCacheShaderTextureSlotAsset` 缓存线修复，`artifact_store_roundtrips_shader_assets_with_cache_safe_toml_metadata` 与 `project_manager_imports_compound_zshader_package_with_subassets` 均通过。Release ZrVM DLL 路径下，focused `vampire_project_session_starts_paused_until_start_button_click` 与 `vampire_project_session_game_over_menu_retries_to_playing` 已通过，覆盖 Start Game、Game Over、Retry 回到 playing；完整 `vampire_project_session`、plugin/native plugin、app 与 `zircon_plugins` gate 仍未关闭。native loader/ABI/bridge-method 类型已从 `zircon_runtime::plugin` 根硬切到 `zircon_runtime::plugin::native`；`plugin/mod.rs` 只保留 `pub mod native;`，不保留旧根 re-export。M2.2 已把 native loader 测试路径固化到隔离 namespace：`native loader test files 4/4`、`native test namespace import files 3/3`、`native test root import leaks 0/0`，并由 `runtime_06_native_loader_tests_use_isolated_plugin_native_namespace` 常驻守卫锁定。M3.1 已把 native plugin loader 收束为 V3-only：`NativePluginAbiV1/V2`、`NativePluginEntryReportV1/V2`、V2 host table、V1/V2 descriptor symbols 与 loader fallback 分支删除；`native_dynamic_fixture` 改为 V3 成功 + unknown ABI rejection 覆盖。M3.2 已新增 hot reload failure injection：`hot_reload_missing_symbol_after_reload_rolls_back_to_previous_instance` 与 `hot_reload_state_restore_failure_rolls_back_and_reports` 覆盖重载后缺符号回滚与 restore-state 失败回滚。`plugin_surface_lifecycle_boundary` 当前报告 `expected_source_file_count = 14`、`native_plugin_public_surface.m4_gate_status=classified-and-clear`、`root_reexport_count = 0`、`native_namespace_reexport_count = 64`、native root re-export 0/0、native namespace re-export 64/64、M4 gate `classified-and-clear`、debt groups 0/0、native namespace symbol groups 5/5、unclassified native root symbols 0/0、unclassified native namespace symbols 0/0、root public native re-export locations 0/0、public native namespace re-export locations 1/1。Runtime 06 仍保持 `in_progress`，因为 `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` Cargo/native 验证线未关闭。
- **Native public-surface renderer split（2026-06-21）**：`native_plugin_public_surface_markdown.py` 已承接 `render_native_plugin_public_surface_markdown(...)`；`native_plugin_public_surface.py` 保持 400 行 native public-surface scan / symbol classification / M4 gate owner，Markdown owner 为 63 行。Direct probe 当前报告 root re-export 0、native namespace re-export 64、symbol decision groups 5、migration debt 0、unclassified root/namespace symbol counts 0/0、root/native public re-export locations 0/1、M4 gate `classified-and-clear`、risks 0、rendered output 12 lines。该切片不关闭 Runtime 06；script VM / native plugin / app / plugin workspace Cargo-native gates 仍 pending。
- **Native hot-update/replay public-surface audit sync（2026-07-01）**：`NativePluginRuntimeDeltaHotUpdateReport`、`NativePluginRuntimeDeltaHotUpdateRequest`、`NativePluginRuntimeRegistrationReplayReport` 与 `NativePluginRuntimeRegistrationSystemReplay` 已归入既有 native live-host runtime public-surface group，未新增 `zircon_runtime::plugin` root re-export、compat module 或旧路径 shim。当前审计锚点：`root_reexport_count = 0`、`native_namespace_reexport_count = 64`、native root re-export 0/0、native namespace re-export 64/64、M4 gate `classified-and-clear`、debt groups 0/0、native namespace symbol groups 5/5、unclassified native root symbols 0/0、unclassified native namespace symbols 0/0、root public native re-export locations 0/0、public native namespace re-export locations 1/1、native loader test files 4/4、native test namespace import files 3/3、native test root import leaks 0/0、`last_refined = 2026-07-01`、`mirror_docs_guard_present = true`、`risks = []`；验证锚为 standalone plugin_surface_lifecycle 3/3，Cargo/native validation lane 仍 deferred。
- **Native host FFI panic guard（2026-06-22）**：承接 `engine-code-review-findings-2026-06.md` F1 与 `engine-code-structure-convention.md` E7，新增 `native_plugin_loader/ffi_panic_guard.rs`，`host_api_adapter.rs` 的 9 个 `ZrHostApiV3` 回调和 `host_callbacks.rs` 的 4 个 private native host callbacks 均在 `extern "C"` 边界内捕获 panic 并映射为状态码。静态 guard 扫描确认 13/13 callback 路由到 guard；focused Cargo 仍因 core-min 编译 1200s timeout 保持 pending。
- **F8 RuntimePluginDescriptor public-field convergence（2026-06-22）**：`RuntimePluginDescriptor` 的 15 个声明字段已从 public 字段硬切为私有字段，并新增 `descriptor/access.rs` 承接 15 个只读访问器。builtin catalog augmentation、registration validation 与 plugin extension 测试调用点改走 accessor；状态锚为 `runtime_plugin_descriptor_public_field_convergence_coremin_check_passed`。`review_f8_runtime_plugin_descriptor_fields_are_private_with_accessors` 锁定 `RuntimePluginDescriptor private fields 15/15`、访问器签名与文档状态；RuntimePluginDescriptor public-field convergence complete。broader Runtime 06 cargo/native gates 仍 pending。
- **F8 RuntimePluginDescriptor public constructor retired（2026-06-22）**：旧 `RuntimePluginDescriptor::new(...).with_*` 公开构造面已硬切退役，`descriptor/builder/construction.rs` retired，`descriptor/builder/fluent.rs` retired；`RuntimePluginDescriptorBuilder` 直接组装私有字段，builtin catalog 子树改为传递 `BuiltinCatalogDescriptorBuilder` 并在 catalog root 统一 `build()`，`zircon_plugins/plugin_sdk` 的 `RuntimePluginDeclaration` 改为持有 `RuntimePluginDescriptorBuilder`。状态锚为 `runtime_plugin_descriptor_public_constructor_retired_coremin_check_passed`，守卫为 `review_f8_runtime_plugin_descriptor_public_constructor_is_retired`；RuntimePluginDescriptor::new retired。
- **Runtime 15 F8 RuntimePluginDescriptor status mirror cleanup（2026-06-27）**：`runtime_15_runtime_plugin_descriptor_status_mirror_cleanup_static_passed_cargo_deferred` 已把 Runtime 06/index、review findings、结构规范、package-manifest docs 与 status-output 期望同步到当前完成态。新增 `review_f8_runtime_plugin_descriptor_status_mirrors_do_not_claim_public_field_pending`，锁定 RuntimePluginDescriptor private fields 15/15、RuntimePluginDescriptor public-field convergence complete 与 RuntimePluginDescriptor::new retired 不再被旧待办文字覆盖；2026-06-28 `f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred` 进一步把 F8 顶表状态列同步为 `convention + Runtime 04 + Runtime 06 + Runtime 15 / review closed`。不改 RuntimePluginDescriptor 行为。
- **ABI 策略当前态**：版本常量当前版 `ZIRCON_NATIVE_PLUGIN_ABI_VERSION = 3`；V1/V2 native plugin loader implementation files 0/0，`zircon_plugins` V1/V2 usage files 0/0，export_build_plan V1/V2 usage 0/0。旧版本协商失败路径不再靠 V2 fallback 覆盖，而由 `abi_unknown_version` fixture feature 生成 `abi_version = 99` 的 V3 descriptor，验证 loader 明确报告 `unsupported native plugin ABI version 99; expected 3`。
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
   - `grep -rln "NativePluginAbiV1\|DESCRIPTOR_SYMBOL_V1" zircon_plugins`（M3.1 后应为 0；失败则说明旧 ABI 复活）
4. ZrVM 环境确认：`../../zr_vm` checkout 存在且与 `backend-zr-vm` feature 可编译（M1 需要真实后端）。
5. 基线记录：`cargo test -p zircon_runtime --lib plugin --locked` 与 `--lib script --locked` 通过数记入状态节。

## 里程碑

### M1 VM 生命周期阻塞修复（最高优先，解锁 07 性能取证）

#### 切片 1.1 空参数 marshalling 修复

- 目标文件：`zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/instance.rs`（及其 `call_module_export` 下行的 sys 调用段，执行时核验确切文件：Grep `call_module_export`，path `zircon_runtime/src/script/vm/backend/zr_vm_project_backend`）。
- 改动形态（方案草案，执行时按 ZrVM C ABI 约定定稿）：空参数数组改为传"合法非空指针 + len=0"或 ZrVM 约定的显式空参数协议；实现为绑定侧防御（空 slice 分支换静态空槽指针，签名草案执行时定稿）。若必须 ZrVM 侧修，先落绑定侧防御并在 `docs/zircon_runtime/dynamic_api/session.md` 记录上游 issue 与版本配对要求（回写子计划 01 的 zr_vm 治理条目）。
- 调用方迁移：无公共面变化（`call_entry_lifecycle_export` 四个调用点 :83/:98/:104/:125 行为自动修复）。
- 验收：`examples/vampire` 真实启动不再触发 `function.c:1394` 断言（或上游 issue + 绑定防御双证据）。
- DoD：`cargo test -p zircon_runtime --lib vampire_project_session --features backend-zr-vm --locked -- --nocapture --test-threads=1` 不再因该断言失败。

#### 切片 1.2 生命周期失败路径测试分层

- 目标文件：`script/vm` 测试位（执行时核验既有测试树：`ls zircon_runtime/src/script/vm/`，`vm/tests.rs` 已存在）。
- 改动形态：四类路径补失败测试，按 feature 分层——`backend-zr-vm` 门控的真实后端测试与 fallback 路径测试分开：
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
2. **编译超时破解**：`--features backend-zr-vm` 组合曾实测 300s 编译超时（状态节证据）——用 `tools/dev-fast-build.ps1` 共享 `CARGO_TARGET_DIR` 预热 + 包内最小 feature 复跑；破解方法与 07 计划切片 0.2（profiling 构建超时）同源，结论双向回写；
  3. **real-backend 回归**：跑 M1 DoD 命令确认 `function.c:1394` 不再触发，同时确认 binding 侧 sentinel 修复（`../zr_vm/.../lib.rs`）与 runtime 侧行为一致。
- 调用方迁移：无。
- 验收：`vm_lifecycle_activate_with_empty_arguments_does_not_trip_native_assertion`（real-backend 门控）首次真实跑绿；fallback 层三测试绿。
- DoD：状态节 1.1/1.2 行的"runtime Cargo 待验证"翻转为"完成"，附命令输出摘要与编译耗时。

#### M1 测试阶段（milestone-first）

- 切片期：`cargo check -p zircon_runtime --lib --locked`
- 里程碑末：
  - `cargo test -p zircon_runtime --lib script::vm --locked -- --nocapture`
- 有真实 ZrVM 环境：`cargo test -p zircon_runtime --lib vampire_project_session --features backend-zr-vm --locked -- --nocapture --test-threads=1`
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

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`06/2026-07-09-plugin-surface-and-lifecycle-output-records.md`](06/2026-07-09-plugin-surface-and-lifecycle-output-records.md)
