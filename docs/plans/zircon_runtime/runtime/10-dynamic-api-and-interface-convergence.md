---
related_code:
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/resolve_asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager_handle.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/surface.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/headless_profiles.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/event_split.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/test_owner_split.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/ffi_panic_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/mirror_docs.rs
  - zircon_runtime_interface/src/runtime_api/api_table.rs
  - zircon_runtime_interface/src/runtime_api/host_requests.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/cursor/request.rs
  - zircon_runtime_interface/src/handles.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/version.rs
  - zircon_runtime_interface/src/ui
  - docs/engine-architecture/runtime-interface-cdylib-loader.md
  - docs/engine-architecture/runtime-interface-convergence.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_abi_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_diagnostics_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_failure_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_host_request_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_session_lifecycle_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_ui_contract_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_validation_inventory.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/engine-architecture/runtime-interface-convergence.md
status: in_progress
last_refined: 2026-06-22
---

# 10 dynamic_api 与 runtime_interface 收敛线

子计划 06 显式排除的"另一条收敛线"落成计划：cdylib 函数表 ABI、session 生命周期出口、interface 契约纯净性与 UI 镜像契约的漂移治理。**native 插件 ABI（NativePluginAbiV3 族）归 06，不在本计划**；editor 客户端实现归 editor 计划。既有口径承接 `docs/engine-architecture/runtime-interface-convergence.md` 与 `runtime-interface-cdylib-loader.md`（2026-06-12 实测存在），本计划不另起口径，只把其目标态落成切片。

## 现状与证据（2026-06-12 实仓盘点）

- **C 出口单点**：`dynamic_api/exports.rs` 仅 1 个 `#[no_mangle] pub unsafe extern "C" fn zircon_runtime_get_api_v1(`（:25-26）——出口面已极窄（健康项）。2026-06-13 M1.3 后，函数表实际指向 `exports.rs` 的 `_ffi` wrappers；`session.rs` owner 函数保持私有 Rust ABI `unsafe fn`，避免 panic 先跨 `extern "C"` 边界再被捕获。
- **函数表双族、版本不同步**：`runtime_api/api_table.rs` 有 `ZrHostApiV1`（宿主回调面）与 `ZrRuntimeApiV1`（runtime 服务面）；`plugin_api.rs` 有 `ZrHostApiV3` + 子 API `ZrHostEcsApiV1`/`ZrHostAssetApiV1`/`ZrHostEventApiV1`/`ZrHostBridgeApiV1`/`ZrHostDiagnosticsApiV1` + `ZrPluginStateSnapshotApiV1` + `ZrPluginApiV1`——**runtime 表 V1 与 plugin 宿主表 V3 并存，子 API 各自 V1**，版本演进规则已在 M0 定稿，当前机器清册由 `dynamic_runtime_api_boundary` 复核为函数表 10/10、字段数漂移 0、缺失 `#[repr(C)]` 0。
- **interface 依赖面已纯净**（01 计划核实）：`zircon_runtime_interface` 依赖仅 glam/serde/serde_json/thiserror/toml/unicode-segmentation/uuid，无 wgpu/winit——守卫已由 01-M1 切片 1.4 锁定。
- **UI 镜像契约面巨大**：`zircon_runtime_interface/src/ui/` 22 条目（含 `v2/`、`template/asset/component_contract/api_version.rs` 的 `UiComponentApiVersion` :8 带 parse error 类型 :80），与 `zircon_runtime/src/ui/` 同构——共享 DTO 与重复定义的甄别、同步规则与漂移守卫缺失（09 计划的 runtime 侧形状收束后，移交清单落到本计划 M2）。
- **支撑件**：`handles.rs`（句柄）、`buffer.rs`（状态/缓冲契约）、`status.rs`、`version.rs`（版本常量）、`reflect/`、`resource/`、`profiling.rs`、`plugin_events.rs`、`plugin_diagnostics.rs`、`manifest.rs`、`math.rs`。
- 加载侧锚：`zircon_app` 经 libloading 动态加载 runtime 并经本 interface 对话（CLAUDE.md；`runtime-interface-cdylib-loader.md`）。
- 参考锚点（每点一行）：Fyrox dylib 插件函数边界 — `dev/Fyrox/fyrox-impl/src/plugin/dylib.rs`；本仓 native 插件 ABI 版本协商（06 计划已细化）作同构参照。

补充参考锚点（2026-06-13 实测核验，实现型切片动工前先读——index 公约 §7.9）：

- Godot GDExtension：C ABI 函数表注册/装载/版本协商的成熟实现（M0 版本矩阵、M3 装载失败路径对照）— `dev/godot/core/extension/{gdextension.{h,cpp},gdextension_function_loader.{h,cpp}}`；API 面 dump/版本化 — `extension_api_dump.cpp`

## 目标

1. 函数表版本策略定稿：`ZrRuntimeApiV1`/`ZrHostApiV1`/`ZrHostApiV3` 及子 API 的版本矩阵、bump 规则（加字段=新版尾追 vs 破坏=新表）、协商失败路径——与 06 的 native ABI 策略同口径但分表治理。
2. ABI-safe 结构守卫：函数表与跨界 DTO 全部 `#[repr(C)]`/ABI-safe，违例可被结构测试拒绝。
3. UI 镜像契约漂移治理：共享 DTO 单一来源判定规则 + 同步守卫，消化 09 移交的重复定义清单。
4. session 生命周期出口收口：经函数表的 session 操作面盘点成册，失败路径（坏句柄、双 destroy、tick 在 destroy 后）测试完备。

## 非目标

- 不改 native 插件 ABI（06 地盘）；不改 editor 客户端；不动 wgpu/渲染对象跨界禁令的实现侧（已有守卫，只引用）。
- 不在本计划做函数表的功能扩张（新增 API 归各 feature owner）。

### 全局硬约束（继承总计划 §4，违反即返工）

- 动态边界只传 ABI-safe 值与序列化负载（本计划即此条的执法计划）；硬切换不留兼容层；不新增 crate；非网络语义 server 命名是 blocker。

## 执行前检查清单

1. 既有口径精读：`docs/engine-architecture/runtime-interface-convergence.md` 与 `runtime-interface-cdylib-loader.md` 全文——本计划切片与其目标态逐条对表，冲突处以既有口径为准并记录。
2. 活动会话对齐：`dynamic_api/**` 被 10fps 会话与 wgpu 主链触及——`git status --porcelain -- zircon_runtime/src/dynamic_api/ zircon_runtime_interface/src/`，脏区避让。
3. 事实重核：
   - `grep -rn "no_mangle" zircon_runtime/src/dynamic_api/`（核出口仍单点）
   - `grep -n "pub struct Zr" zircon_runtime_interface/src/runtime_api/api_table.rs zircon_runtime_interface/src/plugin_api.rs`（核表清单与行号）
   - `grep -c "repr(C)" zircon_runtime_interface/src/runtime_api/api_table.rs zircon_runtime_interface/src/plugin_api.rs`（repr 覆盖基线）
4. 基线记录：`cargo test -p zircon_runtime_interface --locked` 通过数、`cargo test -p zircon_runtime --lib dynamic_api --locked` 通过数记入状态节。

## 里程碑

### M0 ABI 面盘点（先证据后守卫）

#### 切片 0.1 函数表与跨界类型清册

- 目标文件：`docs/engine-architecture/runtime-interface-convergence.md`（扩展"现状清册"节，不另起新文件）。
- 改动形态：纯文档。清册三表：(a) 函数表族（7+ 个 Zr*Api* 结构，逐表列字段数、版本、消费方 crate）；(b) 跨界 DTO 域（handles/buffer/status/manifest/reflect/resource/ui/...，逐域列 `#[repr(C)]` vs serde-序列化两类传输形态）；(c) session 操作面（经 `ZrRuntimeApiV1` 可达的全部函数指针，从 api_table.rs:63 起逐项，含 `tick_frame`/`create`/`destroy` 族）。
- 验收：三表齐备；每个函数表有版本与消费方两列。
- DoD：清册落文档；后续守卫以清册为白名单源。

#### 切片 0.2 版本矩阵与 bump 规则定稿

- 目标文件：同 0.1 文档（"版本策略"节）。
- 改动形态：决策记录——矩阵现状（RuntimeApi V1 / HostApi V1 / plugin 宿主 HostApi V3 / 子 API V1×4 / PluginApi V1 / StateSnapshotApi V1）+ 规则定稿：尾追字段不 bump（C 布局前缀兼容）还是任何变更即 bump（候选二选一，与 06 的 native ABI"单一当前版 + 显式协商失败"口径对齐）；`version.rs` 常量与表版本的对应关系单点化。
- 验收：规则判词 + 矩阵表；与 06-M3 的 ABI 支持矩阵互引。
- DoD：策略落文档且 `version.rs` 引用关系明确。

#### M0 测试阶段（milestone-first）

- 纯审计：`git status --porcelain` 仅 docs 变更。

### M1 ABI-safe 结构守卫

#### 切片 1.1 repr(C) 与禁入类型守卫

- 目标文件：`zircon_runtime_interface/src/tests/`（既有测试树，执行时核验：`ls zircon_runtime_interface/src/tests/`）。
- 改动形态：新增结构守卫（签名草案，执行时定稿）：

  ```rust
  #[test]
  fn function_table_structs_are_all_repr_c() { /* 按 M0 清册扫描 api_table.rs/plugin_api.rs 源文本，每个 pub struct Zr*Api* 前必须有 #[repr(C)] */ }
  #[test]
  fn interface_sources_stay_free_of_non_abi_safe_exports() { /* 禁入词根：Box<dyn、Rc<、Arc<dyn、impl Trait（公共签名中）；白名单经清册 */ }
  ```

- 调用方迁移：无（纯新增守卫；若守卫揪出违例，违例修复列独立切片）。
- 验收：两守卫 + 违例清单（可为空）；负例自检（注入违规样本断言守卫报错）。
- DoD：`cargo test -p zircon_runtime_interface --locked` 含新守卫全绿。

#### 切片 1.2 session 失败路径测试

- 目标文件：`zircon_runtime/src/dynamic_api/tests/`（既有 `session_lifecycle.rs`，扩展）。
- 改动形态：补三类失败路径（既有 `tick_frame_rejects_unknown_session` 是范本）：缺失/已移除 session 显式错误、坏句柄跨全部 session 操作面（按 M0 清册 (c) 逐函数）、destroy 删除注册表条目的结构守卫；`minimal`/`headless` profile 不创建 `RuntimeRenderBridge`，为生命周期测试提供真实 session 但不依赖 WGPU 设备能力。
- 验收（测试名草案）：`destroy_session_reports_explicit_not_found_for_missing_nonzero_handle`、`session_destroy_reports_explicit_not_found_after_headless_destroy`、`destroyed_headless_session_entry_points_reject_old_handle`、`all_session_entry_points_reject_invalid_handle`、`missing_session_entry_points_reject_nonzero_handle`。
- DoD：清册 (c) 的每个入口至少有一条坏句柄或销毁后旧句柄测试覆盖；destroy 入口的 registry removal 契约有守卫；headless/minimal profile 明确跳过 render bridge。

#### 切片 1.3 FFI panic 边界

- 目标文件：`zircon_runtime/src/dynamic_api/exports.rs`、`zircon_runtime/src/dynamic_api/tests/api_table.rs`、`docs/zircon_runtime/dynamic_api/session.md`。
- 改动形态：`zircon_runtime_get_api_v1` 与 `ZrRuntimeApiV1` 函数表入口增加最终 `catch_unwind` containment；函数表仍只暴露同一批 ABI entry points，但每个 entry point 先经过 `exports.rs` 的 `_ffi` wrapper，再委派到 `session.rs` 内部 Rust-ABI owner 函数。
- 调用方迁移：无 ABI 字段变化；加载方仍按 `zircon_runtime_get_api_v1` 获取同一 `ZrRuntimeApiV1` 表。
- 验收：`runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary` 锁定函数表不得直接指向 session owner 函数；panic 被转换为 `ZrStatusCode::Panic` 与稳定诊断；`zircon_runtime_get_api_v1` 获取表期间的 panic 返回 null 指针。
- DoD：panic containment 不扩大公共 Rust API，不移动 session 正常错误校验 owner；session owner 函数不得保留 `extern "C"`；rustfmt、源码锚点、冲突标记/尾随空白、scoped diff 检查通过；Cargo 在编译通道空闲后补跑 `dynamic_api`。

#### M1 测试阶段（milestone-first）

- `cargo check -p zircon_runtime --lib --locked`；`cargo test -p zircon_runtime_interface --locked`
- `cargo test -p zircon_runtime --lib dynamic_api --locked -- --nocapture`
- `cargo test -p zircon_app --locked`（加载方回归）

### M2 UI 镜像契约漂移治理

#### 切片 2.1 单一来源判定与重复定义消化

- 目标文件：`zircon_runtime_interface/src/ui/**` 与 `zircon_runtime/src/ui/**` 的重复定义对（输入 = 09-M0 移交清单；自查命令：对两树跑 `grep -rh "pub struct\|pub enum" | sort | uniq -d` 取候选）。
- 改动形态：判定规则定稿——跨界传输的 DTO 单一来源在 interface，runtime 侧只 `use`；runtime-only 行为类型不得进 interface。重复对逐个硬切换：保留 interface 定义、删 runtime 副本、调用方改 use（或反向，按判定规则）。
- 调用方迁移：逐重复对枚举（清单驱动，每对 ≤10 调用方全列于执行记录）。
- 验收：`ui_contract_types_have_single_definition_across_interface_and_runtime`（结构守卫：两树无同名 pub struct/enum 重复，白名单除外）。
- DoD：重复清单清零或白名单化；守卫进常驻树。

#### 切片 2.2 v2 契约同步规则

- 目标文件：同 0.1 文档 +（按 09-M0 切片 0.2 的 v2 裁决结果联动）。
- 改动形态：`interface/ui/v2` 与 `runtime/ui/v2` 的同步规则按 09 的 v2 判词写入；`UiComponentApiVersion`（component_contract/api_version.rs:8）的版本协商失败路径补测试：`ui_component_api_version_mismatch_is_rejected_with_parse_error`。
- 调用方迁移：无。
- 验收：同步规则 + 版本协商测试。
- DoD：v2 契约规则与 09 判词一致且互引。

#### M2 测试阶段（milestone-first）

- `cargo test -p zircon_runtime_interface --locked`
- `cargo test -p zircon_runtime --lib ui --locked`（重复消化的 runtime 侧回归）
- `cargo check -p zircon_editor --lib --locked`（镜像契约消费方回归）

### M3 cdylib 重载路径收尾

#### 切片 3.1 runtime 库重载失败注入

- 目标文件：加载侧 `zircon_app`（libloading 装载点，执行时核验：Grep `libloading|zircon_runtime_get_api_v1`，path `zircon_app/src`）+ `runtime-interface-cdylib-loader.md` 口径刷新。
- 改动形态：对照 06-M3 的热重载回滚测试模式，补 runtime cdylib 自身的装载失败路径：符号缺失（`zircon_runtime_get_api_v1` 不存在）、版本协商失败（表版本不匹配）、装载后首调用失败——三类各一测试，宿主侧行为定稿（报错退出 vs 回退，判词）。
- 调用方迁移：无公共面变化。
- 验收（测试名草案）：`runtime_library_missing_entry_symbol_fails_load_with_explicit_report`、`runtime_api_version_mismatch_is_rejected_before_session_creation`。
- DoD：`cargo test -p zircon_app --locked` 含新测试全绿；loader 文档与行为一致。

#### M3 测试阶段（milestone-first）

- `cargo test -p zircon_app --locked`
- `cargo test -p zircon_runtime_interface --locked`；`cargo test -p zircon_runtime --lib dynamic_api --locked`
- 验收证据：三类失败注入测试；`runtime-interface-cdylib-loader.md` 刷新。

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M0 | 0.1 ABI 清册 | completed | 2026-06-12 | `docs/engine-architecture/runtime-interface-convergence.md#runtime-10-abi-inventory` 新增函数表族、跨界 DTO 域、session 操作面三张清册；2026-06-14 `dynamic_runtime_api_boundary` 复核源码清册为函数表 10 个（api_table.rs 2 + plugin_api.rs 8，含 `ZrHostBridgeApiV1`）、`ZrRuntimeApiV1` 13 字段，其中函数指针 11 个，session 操作面 11 项 |
| M0 | 0.2 版本矩阵 | completed | 2026-06-12 | 同一文档新增 Version Strategy：函数表任意字段增删/重排/类型/语义变化均 bump 新表版本；`size_bytes` 仅作显式协商/诊断字段；动态 runtime DTO 由 `ZIRCON_RUNTIME_ABI_VERSION_V1` 统管，plugin host 子表按窄表 bump |
| M1 | 1.1 repr(C) 与 ABI 清册守卫 | full_interface_package_passed | 2026-06-12 | 新增 `zircon_runtime_interface/src/tests/abi_safety_contracts.rs` 并接入 `tests/mod.rs`；`function_table_structs_are_all_repr_c` 锁定 10 个 `Zr*Api*` 函数表结构；`interface_public_signatures_stay_free_of_dynamic_object_exports` 扫描公开签名禁入词；`repr_c_guard_fails_on_missing_local_attribute` 与 `public_signature_guard_fails_on_dynamic_object_export` 提供负例自检；`function_table_field_counts_match_runtime_10_inventory` 锁定 M0 ABI 清册字段数矩阵（`ZrRuntimeApiV1` 13 字段及 plugin/host 表字段数），`runtime_api_session_operation_surface_matches_inventory` 锁定 `ZrRuntimeApiV1` 的 11 个 session 操作字段顺序，`runtime_10_version_strategy_rejects_in_place_table_shape_changes` 锁定保守版本策略与 `ZIRCON_RUNTIME_ABI_VERSION_V1` 常量 owner；`docs/engine-architecture/runtime-interface-convergence.md` 已同步命名字段数/操作面守卫；`rustfmt --edition 2021 --check zircon_runtime_interface\src\tests\abi_safety_contracts.rs` 通过；`cargo test -p zircon_runtime_interface inventory --locked --message-format short --color never` 通过 2/2；`cargo test -p zircon_runtime_interface version_strategy --locked --message-format short --color never` 通过 1/1；`cargo test -p zircon_runtime_interface abi_safety_contracts --locked --message-format short --color never` 通过 7/7；`cargo test -p zircon_runtime_interface --locked --message-format short --color never` 通过 165/165，doc-test 0/0 |
| M1 | 1.2 session 失败路径 | code_complete_static_passed_cargo_timeout | 2026-06-12 | `RuntimeDynamicSession.render_bridge` 改为 `Option<RuntimeRenderBridge>`；`minimal`/`headless` profile 通过 `uses_render_bridge()` 跳过 render bridge，capture 返回空帧，surface bind/unbind/present 为 no-op；`create_test_session` 现在显式创建 headless session；新增 `session_destroy_reports_explicit_not_found_after_headless_destroy`、`destroyed_headless_session_entry_points_reject_old_handle`、`create_session_accepts_named_headless_profile_without_render_bridge`、`minimal_and_headless_profiles_skip_render_bridge_bootstrap`，并保留缺失非零句柄/坏句柄/registry removal 守卫；2026-06-13 追加 `runtime_absorption::dynamic_api_session::runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces`，以源码/测试/模块文档/Runtime 10 计划/总表五点锁定 optional render bridge、headless/minimal 跳过 bridge、capture 空帧 fallback 与 bind/unbind/present no-op；新增守卫的 `rustfmt --edition 2021 --check zircon_runtime\src\tests\runtime_absorption\dynamic_api_session.rs zircon_runtime\src\tests\runtime_absorption\mod.rs` 通过，冲突标记/尾随空白/锚点扫描与 scoped `git diff --check` 通过（仅 LF-to-CRLF warning）；`rustfmt --edition 2021 --check zircon_runtime\src\dynamic_api\session.rs zircon_runtime\src\dynamic_api\tests\session_lifecycle.rs zircon_runtime\src\dynamic_api\tests\support.rs` 通过；`git diff --check` 对动态 API 代码/计划/会话文档通过（仅 LF-to-CRLF warning）；冲突标记/尾随空白扫描通过；optional render bridge/source-token 扫描通过；首次 `cargo test -p zircon_runtime --lib destroy_session --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-10-headless-0612 --message-format short --color never -- --nocapture` 在另一条 `zircon_runtime` cargo lane 活跃时 904s 超时，孤立验证进程已停止；第二次在通道清空后启动同一命令，仍在测试二进制编译阶段 904s 超时，期间 render 检查 lane 又启动，本会话启动的孤立验证进程已停止；未声明 Cargo pass |
| M1 | 1.2 Dynamic API 测试边界拆分 | structure_audit_static_passed_cargo_pending | 2026-06-13 | `session_lifecycle.rs` 拆出 `session_entry_points.rs`（跨入口坏句柄/旧句柄/缺失句柄覆盖）与 `session_profiles.rs`（headless/minimal/profile/source-shape 守卫），共享 ABI 请求构造器提升到 `support.rs`；`dynamic_api/tests/structure.rs` 与 `dynamic_api_test_boundary.py` 已更新到 11 个 owner modules。定向结构审计事实：`expected_module_count = 11`、`session_entry_points.rs = 145`、`session_lifecycle.rs = 136`、`session_profiles.rs = 112`、`oversized_modules = []`、`risks = []`；`docs/zircon_runtime/dynamic_api/session.md`、`runtime-interface-convergence.md` 与 M0 review 同步。Cargo 仍待 active lanes 清空后随 M1.2/M1.3 `dynamic_api` gate 补跑。 |
| M1 | Runtime 10 Dynamic API test boundary Markdown renderer split | dynamic_api_test_markdown_split_static_passed_cargo_deferred_tests_deferred | 2026-06-21 | `runtime_structure_audits/dynamic_api_test_markdown.py` 新增为 `render_dynamic_api_test_boundary_markdown(...)` owner，`dynamic_api_test_boundary.py` 保持 test-tree 审计/风险入口并降到 89 行，Markdown owner 为 35 行。direct `dynamic_api_test_boundary_audit` 继续报告 folder-backed owner modules 11/11、legacy `zircon_runtime/src/dynamic_api/tests.rs` absent、missing modules/declarations/oversized modules 全部为空、`risks = []`；验证：Python py_compile、direct Dynamic API test boundary audit。Package-level Cargo stayed deferred while external compile lanes remain active；no `dynamic_api` / app loader / UI gate promoted. |
| M1 | 1.3 FFI panic 边界 | code_static_passed_cargo_pending | 2026-06-13 | `zircon_runtime/src/dynamic_api/exports.rs` 将 `zircon_runtime_get_api_v1` 与 11 个 `ZrRuntimeApiV1` 函数表入口收束到 `catch_unwind` 边界；函数表指向 `_ffi` wrappers，wrappers 将意外 unwind 转为 `ZrStatusCode::Panic` 与 `runtime dynamic API panic caught at FFI boundary`，表获取期间 panic 返回 null；`session.rs` owner 函数改为私有 Rust ABI `unsafe fn`，避免 panic 先跨 `extern "C"` 边界；新增 `runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary` 源码守卫并拒绝 private session owner 重新声明 `extern "C"`；新增 `runtime_absorption::dynamic_api_session::runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge`，把 exports/session/API-table test/模块文档/Runtime 10/总索引串成常驻架构守卫；`docs/zircon_runtime/dynamic_api/session.md` 已同步 FFI Panic Boundary 分工；`rustfmt --edition 2021 --check zircon_runtime\src\tests\runtime_absorption\dynamic_api_session.rs zircon_runtime\src\tests\runtime_absorption\plan_status.rs` 通过，FFI wrapper 源码锚点扫描、冲突标记/尾随空白扫描与 scoped `git diff --check` 通过（仅 LF-to-CRLF warning）；Cargo 待当前 runtime 编译通道空闲后补跑 `dynamic_api` |
| M1 | 1.4 Dynamic Session Event Split | folder_split_static_passed_cargo_deferred_active_lanes | 2026-06-14 | `zircon_runtime/src/dynamic_api/session.rs` 保留私有 Rust-ABI session owner、registry、profile、lifecycle/frame/surface/profile/host-request 编排，并通过 `mod events;` 委托事件处理；`zircon_runtime/src/dynamic_api/session/events.rs` 新增为 pointer/mouse/touch/keyboard/IME/file-drag/window/gamepad/accessibility dispatch、camera/menu action owner；新增 `runtime_10_dynamic_session_event_split_keeps_abi_owner_and_event_router` 与 `runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner`，`dynamic_runtime_api_boundary` 同步到 `expected_source_file_count = 21`，`performance_hotpath_boundary` 同步到 `large_file_hotspot_count = 38`、`runtime-other=14`、`hotspot_guard_anchor_count = 20`。验证：rustfmt check、Python py_compile、direct dynamic/performance/status audits、aggregate audit JSON assertions、standalone `dynamic_api_session.rs` 4/4、standalone `performance_hotspots.rs` 6/6、standalone `status_output` 2/2；包级 `dynamic_api`/Runtime 07 Cargo gates 因 active compile lanes deferred。 |
| M1 | 1.5 Dynamic Session Test Owner Split | folder_split_static_passed_cargo_deferred_active_lanes | 2026-06-14 | 删除 `zircon_runtime/src/dynamic_api/session/tests.rs`，改为 folder-backed `session/tests/{mod,vampire_runtime_support,vampire_gameplay,vampire_menu,vampire_hud,frame_diagnostics,runtime_errors,lock_poison}.rs`；`mod.rs` 只保留声明，`vampire_runtime_support.rs` 474 行，`frame_diagnostics.rs` 127 行，`vampire_gameplay.rs` 179 行，`vampire_hud.rs` 172 行，其他 owner 文件更低；新增 `runtime_10_dynamic_session_test_owner_split_keeps_focused_modules`，`performance_hotpath_boundary` 的 Runtime 07 extract/FPS 证据路径改为 `session/tests/frame_diagnostics.rs`，`performance_hotspots.rs` 同步 include。2026-06-25 的 Runtime 15 M2 后续命名硬切已把 shared vampire support owner 从 `helpers.rs` 收束到 `vampire_runtime_support.rs`，该 Runtime 10 守卫同步读取新 owner。验证：rustfmt check、Python py_compile、direct performance/status audits、standalone `dynamic_api_session.rs` 5/5、standalone `performance_hotspots.rs` 6/6、standalone `status_output` 2/2；包级 `dynamic_api` / Runtime 07 Cargo gates 因 active compile lanes deferred。 |
| M2 | 2.1 重复定义消化 | runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending | 2026-06-17 | `UiBindingCodec` 与 `UiAssetSchemaVersionPolicy` 的 runtime-local duplicate definitions 已删除：`zircon_runtime/src/ui/event_ui/codec.rs`、`zircon_runtime/src/ui/template/asset/schema/policy.rs`；runtime event_ui 只导出 `UiEventManager`，template asset schema 只导出 `UiAssetSchemaMigrator`，interface 继续拥有 `zircon_runtime_interface::ui::event_ui::UiBindingCodec` 与 `zircon_runtime_interface::ui::template::asset::schema::UiAssetSchemaVersionPolicy`；新增 `runtime_10_ui_contract_types_have_single_definition_across_interface_and_runtime`，`dynamic_runtime_api_boundary` 当前记录 `expected_source_file_count = 29`、`host_request_payload_anchors = 38/38`、`ui_contract_single_source_anchors = 7/7`、`ui_contract_duplicate_public_types = 0`、`ui_v2_contract_sync_anchors = 9/9`。验证：rustfmt check、Python py_compile、direct `dynamic_runtime_api_boundary_audit`、M2.1 当时 standalone `dynamic_api_session.rs` 6/6，本轮 M2.2 后 standalone `dynamic_api_session.rs` 9/9、standalone `plan_status.rs` 32/32、direct duplicate scan 0/0、conflict/diff checks 通过（仅 LF/CRLF warnings）；focused Cargo `cargo test -p zircon_runtime --lib dynamic_api_session --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-10-ui-contract-0617 --message-format short --color never -- --nocapture` 304s 超时无测试结果，未声明通过；2026-06-17 interface package Cargo gate 已 168/168 通过，runtime ui/editor gates 仍 pending。 |
| M2 | 2.2 v2 契约同步 | runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending | 2026-06-17 | 新增 `runtime_10_ui_v2_contract_sync_matches_runtime_09_verdict_and_interface_owner`，锁定 Runtime 09 `v2-replacement-mainline` 判词、interface `ui/v2` DTO owner、runtime `ui/v2` consumer、`UiComponentApiVersion` owner 与 runtime component-contract validation 的 `actual.is_compatible_with(required)` 路径；`zircon_runtime_interface/src/tests/ui_v2_contracts.rs` 新增命名守卫 `ui_component_api_version_mismatch_is_rejected_with_parse_error`，覆盖 major mismatch 不兼容与非法 `2.0` 解析错误；`dynamic_runtime_api_boundary` 当前记录 `expected_source_file_count = 29`、`host_request_payload_anchors = 38/38`、`ui_v2_contract_sync_anchors = 9/9`。验证：rustfmt check、Python py_compile、direct `dynamic_runtime_api_boundary_audit` risks=[] / source 29/29 / host-request payload 38/38 / UI single-source 7/7 / UI v2 9/9 / duplicate public types 0、standalone `dynamic_api_session.rs` 9/9、standalone `plan_status.rs` 32/32、scoped conflict-marker scan 通过；首次并行 rustc/link 因 Windows 页文件/链接器内存不足失败，改为串行后通过；2026-06-17 interface package Cargo gate 已 168/168 通过，M2 Cargo lane 仍等待 runtime ui/editor。 |
| 横切 | M2 UI 镜像契约 pending gate | code_static_pending_owner_cargo | 2026-06-17 | `runtime_absorption::plan_status::cargo_gates::runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff` 继续锁定 Runtime 10 M2 UI 镜像契约在 `cargo test -p zircon_runtime_interface --locked`、`cargo test -p zircon_runtime --lib ui --locked` 与 `cargo check -p zircon_editor --lib --locked` 全部通过前保持 pending；2.1 静态硬切已删除两个 runtime-local duplicate contract types，2.2 v2 契约同步已静态落地并记录 `runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending` / `ui_v2_contract_sync_anchors = 9/9`；2026-06-17 interface package gate 已通过，runtime ui/editor Cargo gates 仍 pending。 |
| M2 | interface 契约 Cargo 验证 | interface_package_passed_ui_editor_pending | 2026-06-17 | `cargo test -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-10-interface-0617 --message-format short --color never` 通过：168 passed；0 failed；doc-test 0/0；覆盖 `abi_safety_contracts`、`ui_component_api_version_mismatch_is_rejected_with_parse_error`、layout/render/window/UI contract tests；M2 broader gate 仍等待 `cargo test -p zircon_runtime --lib ui --locked` 与 `cargo check -p zircon_editor --lib --locked`。 |
| M3 | 3.1 重载失败注入 | scoped_cargo_passed_pending_full_package | 2026-06-12 | `LoadedRuntime` 新增私有 `validate_runtime_api_pointer(...)` 表校验入口；`runtime_api_pointer_rejects_null_from_entry_symbol`、`runtime_api_pointer_rejects_version_mismatch_before_session_creation`、`runtime_api_pointer_rejects_missing_required_functions_before_session_creation`、`runtime_library_loader_reports_missing_entry_symbol_source_path`、`runtime_library_loader_reports_missing_entry_symbol_from_dynamic_library`、`runtime_session_create_reports_first_call_failure_context` 已补入 `zircon_app/src/entry/runtime_library/tests.rs`；首次 `cargo test -p zircon_app --lib runtime_api_pointer_rejects --locked --message-format short --color never` 超时 304s 未返回结果，后续重跑通过 3/3；`cargo test -p zircon_app --lib runtime_library_loader_reports_missing_entry_symbol_source_path --locked --message-format short --color never` 通过 1/1；`cargo test -p zircon_app --lib runtime_library_loader_reports_missing_entry_symbol_from_dynamic_library --locked --message-format short --color never` 通过 1/1；`cargo test -p zircon_app --lib runtime_session_create_reports_first_call_failure_context --locked --message-format short --color never` 通过 1/1；完整 `cargo test -p zircon_app --locked` 仍待测试窗口 |
| 横切 | Dynamic runtime API 结构镜像 | structure_audit_static_passed_cargo_pending | 2026-06-20 | `runtime_structure_audits/dynamic_runtime_api_boundary.py` 当前静态事实：`expected_source_file_count = 33`、`function_table_structs = 10/10`、`field_count_mismatches = 0`、`missing_repr_c_tables = 0`、`runtime_session_ffi_wrappers = 11/11`、`direct_session_table_entry_bypasses = 0`、`session_owner_extern_c_present = false`、`headless_lifecycle_anchors = 12/12`、`ffi_panic_anchors = 9/9`、`loader_failure_anchors = 10/10`、`behavior_test_anchor_count = 16`、`missing_behavior_test_anchors = []`、`runtime_diagnostics_anchors = 15/15`、`missing_runtime_diagnostics_anchors = []`、`host_request_payload_anchors = 38/38`、`missing_host_request_payload_anchors = []`、`ui_pending_gate_anchors = 8/8`、`ui_contract_single_source_anchors = 7/7`、`ui_contract_duplicate_public_types = 0`、`ui_v2_contract_sync_anchors = 9/9`、`pending_cargo_gate_anchors = 5/5`、`doc_anchors = 13/13`、`mirror_docs_guard_present = true`、`risks = []`；这仍是静态结构证据，`dynamic_api`、完整 app loader 与 UI contract Cargo gates 按实现优先策略后续补跑 |
| 横切 | Dynamic runtime API 镜像文档守卫 | mirror_docs_static_passed_cargo_pending | 2026-06-20 | `runtime_absorption::dynamic_api_session::runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts` 锁定 33 个 Runtime 10 source owner、10 个 `#[repr(C)]` function table field-count、11 个 runtime session FFI wrapper、private Rust ABI session owner、6 份镜像文档字段，以及 `dynamic_runtime_api_boundary` 的 `behavior_test_anchor_count = 16`、`missing_behavior_test_anchors = []`、`runtime_diagnostics_anchors = 15/15`、`missing_runtime_diagnostics_anchors = []`、`host_request_payload_anchors = 38/38`、`missing_host_request_payload_anchors = []`、`ui_contract_single_source_anchors = 7/7`、`ui_contract_duplicate_public_types = 0`、`ui_v2_contract_sync_anchors = 9/9`、`doc_anchors = 13/13` 与 `mirror_docs_guard_present = true`；同时把 runtime diagnostics profile-control snapshot、UI contract single-source、v2 contract sync guard 与 host-request payload ABI boundary 纳入拆分模块。验证：Python py_compile、direct `dynamic_runtime_api_boundary_audit` risks=[]、standalone `dynamic_api_session.rs` 9/9；Cargo gates 仍 pending。 |
| 横切 | Runtime 10 host-request payload ABI boundary | host_request_payload_boundary_static_passed_cargo_pending | 2026-06-20 | `dynamic_runtime_api_boundary` 现在把 `drain_host_requests` 的 payload owner chain 纳入 Runtime 10 Dynamic API 边界清册：interface `runtime_api/host_requests.rs` owns `ZrRuntimeHostRequestBatchV1` / `ZrRuntimeHostRequestV1::{Ime,GamepadRumble,Cursor}` DTOs；`zircon_runtime/src/dynamic_api/session/host_requests.rs` owns runtime conversion from IME/gamepad rumble/cursor drains into ABI payloads；`zircon_runtime/src/dynamic_api/tests/host_requests.rs` owns encoding/free owner-token behavior tests；`zircon_app/src/entry/runtime_entry_app/host_requests/{routing,cursor/request}.rs` owns app-side application. Current audit records `expected_source_file_count = 33`, `host_request_payload_anchors = 38/38`, `missing_host_request_payload_anchors = []`, `doc_anchors = 13/13`, `mirror_docs_guard_present = true`, `risks = []`; validation: Python py_compile/direct dynamic audit, standalone `dynamic_api_session.rs` and status-output/rustfmt lightweight gates; `dynamic_api/app/UI Cargo gates pending`。 |
| 横切 | Runtime 10 F18 asset manager resolution return shape | runtime_10_asset_manager_resolution_handle_shape_coremin_check_passed | 2026-06-22 | F18 manager 解析返回形态已硬切到 registered handle：`resolve_asset_manager(core)` 现在返回 `Result<Arc<AssetManagerHandle>, CoreError>`，直接沿用 `CoreHandle::resolve_manager::<AssetManagerHandle>(ASSET_MANAGER_NAME)`，不再在 helper 内隐藏 `.map(|holder| holder.shared())` 或直接返回 `Arc<dyn AssetManager>`。动态项目启动的 `open_project_assets(...)` 在边界执行 `let asset_manager = asset_manager.shared();`，即 dynamic project boundary calls `.shared()` 后再调用 `open_project(...)`。新增 `review_f18_asset_manager_resolution_returns_registered_handle` 结构守卫，并同步 module docs、review findings、结构规范、runtime index 与 status-output expectations。验证：scoped rustfmt --check 通过；F18 structure guard 1/1、status-output 2/2 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime10-f18-manager-resolution-0622` 通过（既有 141 warnings）；`dynamic_api/app/UI Cargo gates pending`。 |
| 横切 | Runtime 10 runtime diagnostics profile-control snapshot | runtime_diagnostics_profile_control_static_passed_cargo_deferred_tests_deferred | 2026-06-20 | 状态锚 `runtime_diagnostics_profile_control_static_passed_cargo_deferred_tests_deferred`；`ProfileControlCommand::RuntimeDiagnosticsSnapshot` 复用既有 `profile_control` JSON ABI，返回 `ProfileControlResponse.runtime_diagnostics` / `RuntimeDiagnosticsSnapshot` / `RuntimeSceneAssetReloadDiagnostics`，由 `session/diagnostics.rs` 从 `collect_runtime_diagnostics(...)` 和 last scene-asset reload report 投影；no new `ZrRuntimeApiV1` function pointer。`dynamic_runtime_api_boundary` 当前同步 `expected_source_file_count = 33`、`behavior_test_anchor_count = 16`、`runtime_diagnostics_anchors = 15/15`、`missing_runtime_diagnostics_anchors = []`、`doc_anchors = 13/13`；Cargo/focused tests 按实现优先策略 deferred。 |
| 横切 | Runtime 10 scene-asset reload diagnostic path guard | runtime_10_scene_asset_reload_diagnostic_paths_static_guard_rustfmt_passed_cargo_deferred_tests_deferred | 2026-06-20 | `runtime_absorption/dynamic_api_session/runtime_diagnostics.rs` 新增 `runtime_10_scene_asset_reload_frame_diagnostics_keep_stable_store_paths`，并由 `shared.rs` 的 `EXPECTED_RUNTIME_10_SCENE_ASSET_RELOAD_DIAGNOSTIC_PATH_ANCHORS` 锁定 21 个动态 session 场景资产热重载诊断锚点：`RuntimeDynamicSession::tick_scene_asset_reload` 必须通过 `DynamicSceneAssetReloadQueue::tick_into_level` 产出 frame report、调用 `record_scene_asset_reload_frame_report(&self.runtime, &report)`、缓存 `last_scene_asset_reload_report`，`session/scene_asset_reload_diagnostics.rs` 必须继续记录 `scene.asset_reload.events_drained` / `scheduled` / `skipped` / `skipped_removed` / `skipped_reload_failed` / `skipped_missing_locator` / `skipped_stale_revision` / `superseded_pending` / `applied` / `failed` / `stale` / `pending` / `receiver_disconnected`，并保留 `["scene", "asset_reload"]` subsystem tags。该守卫只固定现有诊断路径和文档锚点，不新增 ABI 表项、不改变 frame tick/apply policy、不触碰 render/editor/plugin owner。验证：`rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/runtime_diagnostics.rs zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared.rs` passed；standalone `dynamic_api_session.rs` 10/10 passed；standalone `plan_status.rs` 33/33 passed；Cargo/focused tests deferred because an external render Cargo lane is active. |
| 横切 | Runtime 10 dynamic diagnostics inventory split | runtime_10_dynamic_api_diagnostics_inventory_split_static_passed_cargo_deferred_tests_deferred | 2026-06-20 | `runtime_structure_audits/dynamic_runtime_api_diagnostics_inventory.py` 新增为 Runtime 10 diagnostics anchor inventory owner：既有 `runtime_diagnostics_anchors = 15/15` 清单从 `dynamic_runtime_api_boundary.py` 抽出，新接入 `scene_asset_reload_diagnostic_path_anchors = 21/21` 与 `missing_scene_asset_reload_diagnostic_path_anchors = []`；`dynamic_runtime_api_boundary.py` 保持审计入口职责，新增风险项只在 scene-asset reload diagnostic path anchor 缺失时触发；`runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts` 要求六份镜像文档同步这两个字段。验证目标：Python py_compile、direct dynamic runtime API audit、standalone `dynamic_api_session.rs`、standalone `plan_status.rs`；package-level Cargo deferred because external runtime render Cargo lanes are active. |
| 横切 | Runtime 10 host-request inventory split | runtime_10_host_request_payload_inventory_split_static_passed_cargo_deferred_tests_deferred | 2026-06-20 | `runtime_structure_audits/dynamic_runtime_api_host_request_inventory.py` 新增为 Runtime 10 host-request payload anchor inventory owner：既有 38 项 `host_request_payload_anchors` 清单从 933 行 `dynamic_runtime_api_boundary.py` 抽出，边界入口降到 781 行并继续报告 `host_request_payload_anchors = 38/38`、`missing_host_request_payload_anchors = []` 与 `risks = []`。验证：Python py_compile；direct `dynamic_runtime_api_boundary_audit` source 33/33、runtime diagnostics 15/15、scene-asset reload 21/21、host-request payload 38/38、risks=[]；package-level Cargo deferred because an external render Cargo lane is active. |
| 横切 | Runtime 10 UI contract inventory split | runtime_10_ui_contract_inventory_split_static_passed_cargo_deferred_tests_deferred | 2026-06-20 | `runtime_structure_audits/dynamic_runtime_api_ui_contract_inventory.py` 新增为 Runtime 10 UI pending/single-source/v2 contract anchor inventory owner：既有 `ui_pending_gate_anchors`、`ui_contract_single_source_anchors` 与 `ui_v2_contract_sync_anchors` 清单从 `dynamic_runtime_api_boundary.py` 抽出，边界入口降到 681 行并继续报告 `ui_pending_gate_anchors = 8/8`、`ui_contract_single_source_anchors = 7/7`、`ui_v2_contract_sync_anchors = 9/9` 与 `risks = []`。验证：Python py_compile；direct `dynamic_runtime_api_boundary_audit` source 33/33、host-request payload 38/38、UI pending 8/8、single-source 7/7、v2 9/9、risks=[]；package-level Cargo deferred because an external render Cargo lane is active. |
| 横切 | Runtime 10 validation inventory split | runtime_10_dynamic_api_validation_inventory_split_static_passed_cargo_deferred_tests_deferred | 2026-06-20 | `runtime_structure_audits/dynamic_runtime_api_validation_inventory.py` 新增为 Runtime 10 behavior-test / pending Cargo gate / mirror-doc anchor inventory owner：既有 `behavior_test_anchor_count = 16`、`pending_cargo_gate_anchors = 5/5`、`doc_anchors = 13/13` 与 `runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts` 清单从 `dynamic_runtime_api_boundary.py` 抽出，边界入口降到 545 行并继续报告 `missing_behavior_test_anchors = []`、`missing_cargo_gate_anchors = []`、`missing_doc_anchors = []` 与 `risks = []`。验证：Python py_compile；direct `dynamic_runtime_api_boundary_audit` source 33/33、behavior-test 16/16、pending Cargo gates 5/5、doc anchors 13/13、risks=[]；rustfmt touched plan-status Rust files；standalone `dynamic_api_session.rs` 10/10；standalone `plan_status.rs` 33/33；package-level Cargo deferred because external render Cargo lanes are active. |
| 横切 | Runtime 10 session lifecycle inventory split | runtime_10_session_lifecycle_inventory_split_static_passed_cargo_deferred_tests_deferred | 2026-06-20 | `runtime_structure_audits/dynamic_runtime_api_session_lifecycle_inventory.py` 新增为 Runtime 10 headless/minimal lifecycle anchor inventory owner：既有 `headless_lifecycle_anchors = 12/12` 清单从 `dynamic_runtime_api_boundary.py` 抽出，边界入口降到 509 行并继续报告 `missing_headless_lifecycle_anchors = []` 与 `risks = []`。验证：Python py_compile；direct `dynamic_runtime_api_boundary_audit` source 33/33、headless lifecycle 12/12、risks=[]；rustfmt touched plan-status Rust files；standalone `dynamic_api_session.rs` 10/10；standalone `plan_status.rs` 33/33；package-level Cargo deferred because external render Cargo lanes are active. |
| 横切 | Runtime 10 failure boundary inventory split | runtime_10_failure_boundary_inventory_split_static_passed_cargo_deferred_tests_deferred | 2026-06-20 | `runtime_structure_audits/dynamic_runtime_api_failure_inventory.py` 新增为 Runtime 10 FFI panic / loader failure anchor inventory owner：既有 `ffi_panic_anchors = 9/9` 与 `loader_failure_anchors = 10/10` 清单从 `dynamic_runtime_api_boundary.py` 抽出，边界入口降到 449 行并继续报告 `missing_ffi_panic_anchors = []`、`missing_loader_failure_anchors = []` 与 `risks = []`。验证：Python py_compile；direct `dynamic_runtime_api_boundary_audit` source 33/33、FFI panic 9/9、loader failure 10/10、risks=[]；rustfmt touched plan-status Rust files；standalone `dynamic_api_session.rs` 10/10；standalone `plan_status.rs` 33/33；package-level Cargo deferred because external render Cargo lanes are active. |
| 横切 | Runtime 10 ABI source inventory split | runtime_10_dynamic_api_abi_inventory_split_static_passed_cargo_timeout_no_result_tests_deferred | 2026-06-21 | `runtime_structure_audits/dynamic_runtime_api_abi_inventory.py` 新增为 Runtime 10 source owner、function-table shape、session operation 清册 owner：既有 `expected_source_file_count = 33`、`expected_function_table_count = 10` 与 `runtime_session_operation_count = 11` 清单从 `dynamic_runtime_api_boundary.py` 抽出，边界入口降到 391 行并继续报告 function table 缺失/field mismatch/direct table bypass/session owner drift 全部为空，`session_owner_extern_c_present = false`、`risks = []`。验证：Python py_compile；direct `dynamic_runtime_api_boundary_audit` source 33/33、function-table 10/10、runtime session operation 11/11、risks=[]；rustfmt touched plan-status Rust files；standalone `dynamic_api_session.rs` 10/10；standalone `plan_status.rs` 33/33；`CARGO_TARGET_DIR=E:\cargo-targets\zircon-shared cargo test -p zircon_runtime --lib dynamic_api_session --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture` 904s timeout no result，post-timeout process scan found no residual cargo/rustc/rustdoc；no `dynamic_api` / app loader / UI gate promoted. |
| 横切 | Runtime 10 runtime API Markdown renderer split | runtime_api_markdown_split_static_passed_cargo_deferred_tests_deferred | 2026-06-21 | `runtime_structure_audits/runtime_api_markdown.py` 新增为 `render_runtime_api_boundary_markdown(...)` owner，`runtime_api_boundary.py` 保持 interface ABI facade 审计/风险入口并降到 143 行，Markdown owner 为 39 行。direct `runtime_api_boundary_audit` 继续报告 folder-backed owner modules 6/6、`runtime_api.rs` facade 12/20 non-empty lines、missing modules/declarations/re-exports/forbidden facade declarations/oversized modules 全部为空、`risks = []`；验证：Python py_compile、direct runtime API boundary audit。Package-level Cargo stayed deferred while external compile lanes remain active；no `dynamic_api` / app loader / UI gate promoted. |
| 横切 | Runtime 10 dynamic runtime API Markdown renderer split | dynamic_runtime_api_markdown_split_static_passed_cargo_deferred_tests_deferred | 2026-06-21 | `runtime_structure_audits/dynamic_runtime_api_markdown.py` 新增为 `render_dynamic_runtime_api_boundary_markdown(...)` owner，`audit_runtime_structure.py` now imports the renderer from that Markdown owner instead of `dynamic_runtime_api_boundary.py`; `dynamic_runtime_api_boundary.py` remains the 330-line audit/risk owner and the Markdown owner is 65 lines. Direct audit continues to report source 33/33, function tables 10/10, field-count mismatches 0, missing `#[repr(C)]` tables 0, runtime session wrappers 11/11, direct table-entry bypasses 0, `session_owner_extern_c_present = false`, headless 12/12, FFI panic 9/9, loader failure 10/10, behavior 16/16, runtime diagnostics 15/15, scene-asset reload diagnostics 21/21, host-request payload 38/38, UI pending 8/8, single-source 7/7, UI duplicates 0, v2 9/9, pending Cargo gates 5/5, docs 13/13, mirror-doc guard present, and `risks = []`. Validation: Python py_compile and direct `dynamic_runtime_api_boundary_audit`; package-level Cargo stays deferred while external compile lanes remain active, so no `dynamic_api`, app loader, or UI gate is promoted. |
| 横切 | Runtime 10 dynamic input mouse-wheel event owner guard | dynamic_input_mouse_wheel_event_owner_guard_focused_cargo_passed_broader_input_pending | 2026-06-21 | 状态锚 `dynamic_input_mouse_wheel_event_owner_guard_focused_cargo_passed_broader_input_pending`；`zircon_runtime/src/dynamic_api/tests/input_events.rs` 的 `mouse_wheel_at_events_decode_delta_bits_for_dynamic_session` 源码守卫现在同时拼接 `include_str!("../session.rs")` 与 `include_str!("../session/events.rs")`，跟随 M1.4 后 `handle_mouse_wheel` 的真实 owner 文件，继续锁定 `ZR_RUNTIME_MOUSE_WHEEL_COORDS_PRESENT_V1`、`f32::from_bits(event.key_code)` / `event.scan_code` 与 `MouseWheelEvent::new(unit, delta_x, delta_y)` 的解码顺序。验证：`rustfmt --edition 2021 --check zircon_runtime\src\dynamic_api\tests\input_events.rs` 通过；`cargo test -p zircon_runtime --lib mouse_wheel_at_events_decode_delta_bits_for_dynamic_session --locked --jobs 1 --target-dir target\codex-runtime11-default-tasks-0621 --message-format short --color never -- --test-threads=1 --nocapture` 日志 `target\codex-runtime12-logs\dynamic_input_mouse_wheel_event_owner_default_20260621.log` 记录 `1 passed; 0 failed; 4704 filtered out`。随后 Runtime 12 broader `input` 过滤门更新为 `342 passed; 12 failed; 4353 filtered out`（日志 `target\codex-runtime12-logs\input_default_after_mouse_wheel_guard_20260621.log`）；该行只关闭动态输入事件 owner 守卫，不提升 broader `dynamic_api`、app loader、runtime UI/editor 或 Runtime 12 input/action_map/gamepad/app gates。 |
| 横切 | Runtime 10 Vampire W input real-backend gate | dynamic_vampire_w_input_real_backend_gate_ignored_without_zr_vm_remaining_ui_input_pending | 2026-06-21 | 状态锚 `dynamic_vampire_w_input_real_backend_gate_ignored_without_zr_vm_remaining_ui_input_pending`；`zircon_runtime/src/dynamic_api/session/tests/vampire_gameplay.rs` 中的 `vampire_project_session_w_key_moves_player_before_input_clear` 现在在未启用 `zr-vm-real-backend` 时使用 `#[cfg_attr(not(feature = "zr-vm-real-backend"), ignore = "requires zr-vm-real-backend and ZR_VM_RUST_BINDING_LIB_DIR")]` 明确标注真实 VM 后端依赖，避免默认配置把环境能力缺失误报成输入路由失败。验证：`rustfmt --edition 2021 --check zircon_runtime\src\dynamic_api\session\tests\vampire_gameplay.rs` 通过；focused default Cargo `vampire_project_session_w_key_moves_player_before_input_clear` 日志 `target\codex-runtime12-logs\vampire_w_input_default_real_backend_gate_20260621.log` 记录 `0 passed; 0 failed; 1 ignored; 4706 filtered out`；随后 Runtime 12 broader `input` 过滤门在 `target\codex-runtime12-logs\input_default_after_vampire_real_backend_gate_20260621.log` 记录 `342 passed; 11 failed; 1 ignored; 4353 filtered out`，剩余失败全在 UI input/text routing owners。本行只关闭默认配置 real-backend gate 误报，不提升真实后端 Vampire 行为、broader `dynamic_api`、app loader、runtime UI/editor 或 Runtime 12 input/action_map/gamepad/app gates。 |
| 横切 | Runtime 10 Dynamic API current audit recheck | dynamic_api_current_audit_static_passed_cargo_pending | 2026-06-20 | 本轮复核 Runtime 10 当前 Dynamic API / interface 边界结构事实：`dynamic_runtime_api_boundary_audit` 报告 source files 33/33、function tables 10/10、field-count mismatches 0、missing `#[repr(C)]` tables 0、runtime session FFI wrappers 11/11、direct session table-entry bypasses 0、session owner extern C false、headless lifecycle anchors 12/12、FFI panic anchors 9/9、loader failure anchors 10/10、behavior-test anchors 16/16、runtime diagnostics anchors 15/15、host-request payload anchors 38/38、UI pending-gate anchors 8/8、UI contract single-source anchors 7/7、duplicate public UI contract types 0、UI v2 contract sync anchors 9/9、pending Cargo gate anchors 5/5、doc anchors 13/13、`mirror_docs_guard_present = true`、`risks = []`。验证：Python py_compile、direct `dynamic_runtime_api_boundary_audit` risks=[]、standalone `dynamic_api_session.rs` 9/9；`dynamic_api`、完整 app loader 与 runtime UI/editor Cargo gates 仍 pending。 |
| 横切 | Runtime 10 dynamic_api_session Cargo 验证窗口探测 | cargo_recheck_timeout_static_guards_passed | 2026-06-20 | 尝试 `cargo test -p zircon_runtime --lib dynamic_api_session --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-10-dynamic-current-0620 --message-format short --color never -- --test-threads=1 --nocapture`，604s 工具窗口超时，未生成 `zircon_runtime` 测试二进制或测试结果；进程检查未发现残留 cargo/rustc/rustdoc。当前轻量验证已通过：Python py_compile、direct `dynamic_runtime_api_boundary_audit` risks=[]、standalone `dynamic_api_session.rs` 9/9；不得据此提升 `dynamic_api` / app loader / runtime UI/editor Cargo gates。 |
| 横切 | Dynamic runtime API 行为测试锚审计同步 | mirror_docs_static_passed_cargo_pending | 2026-06-15 | `dynamic_runtime_api_boundary` 与 `runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts` 现在锁定 16 个行为测试锚：动态 API session 缺失/销毁句柄失败路径、headless/minimal profile、FFI panic table wrapper、app loader 失败注入，以及 profile-control runtime diagnostics snapshot；当前 `behavior_test_anchor_count = 16`、`missing_behavior_test_anchors = []`。验证：Python py_compile、direct `dynamic_runtime_api_boundary_audit` risks=[]、standalone dynamic_api_session 9/9；`dynamic_api`、完整 app loader 与 UI contract owner/Cargo gates 仍 pending。 |
| 横切 | dynamic_api_session 吸收守卫拆分 | focused_cargo_passed_broader_gates_pending | 2026-06-17 | `runtime_absorption/dynamic_api_session.rs` now mounts `dynamic_api_session/{shared,headless_profiles,event_split,test_owner_split,ffi_panic_boundary,runtime_diagnostics,ui_contract,v2_contract,mirror_docs}.rs`; Runtime 10 mirror counts now report `expected_source_file_count = 33`、`behavior_test_anchor_count = 16`、`missing_behavior_test_anchors = []`、`runtime_diagnostics_anchors = 15/15`、`host_request_payload_anchors = 38/38`、`ui_contract_duplicate_public_types = 0`、`ui_v2_contract_sync_anchors = 9/9`. Validation: 2026-06-15 focused Cargo `cargo test -p zircon_runtime --lib dynamic_api_session --locked` 已 5 passed / 4231 filtered out；本轮新增 runtime diagnostics / UI contract / v2 contract guard 后 package Cargo 仍按实现优先策略后续补跑；broader `dynamic_api`、完整 app loader 与 UI contract Cargo gates 仍 pending。 |

基线数值（开工首日记录）：

- C 出口基线：1（`zircon_runtime_get_api_v1`，exports.rs:25-26）
- 函数表基线：Zr*Api* 结构 10 个（api_table.rs 2 + plugin_api.rs 8，含 `ZrHostBridgeApiV1`）
- `repr(C)` 覆盖基线：14（`runtime_api/api_table.rs` + `plugin_api.rs`，其中 `Zr*Api*` 函数表结构 10/10 由 `abi_safety_contracts` 与 `dynamic_runtime_api_boundary` 守卫）
- interface `ui/` 条目基线：22；重复定义候选数：2 -> 0（M2.1 已删除 runtime-local `UiBindingCodec` / `UiAssetSchemaVersionPolicy` 重复定义；守卫 `ui_contract_duplicate_public_types = 0`）
- `cargo test -p zircon_runtime_interface --locked` 通过数基线：168 passed；0 failed；doc-test 0/0（2026-06-17，`E:\cargo-targets\zircon-runtime-10-interface-0617`）

## 风险与协调

- **与 06 的边界**：plugin_api.rs 的 `ZrHostApiV3` 族被 native loader 消费（06 地盘的宿主侧）——本计划只管表结构的 ABI-safe 与版本策略，函数语义与协商实现归 06；两计划的版本矩阵必须互引同口径。
- 既有收敛文档（`runtime-interface-convergence.md`）可能由 `zr-runtime-interface-convergence` skill 对应的会话维护——动工前确认无活跃会话在改该文档；有则并入其口径执行。
- M2 重复定义消化横跨 runtime/interface/editor 三 crate：每对独立切片、独立提交，三 crate 同测后再下一对。
- `dynamic_api/**` 是 10fps 会话触及区：切片前 `git status`，禁止回退。
- 08 计划若裁决补实体 generation（实体 ID 表示变化），`handles.rs` 的实体句柄 ABI 表示需联动——08/10 互为执行前检查项。
