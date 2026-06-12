---
related_code:
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/surface.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_runtime_interface/src/runtime_api/api_table.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/handles.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/version.rs
  - zircon_runtime_interface/src/ui
  - docs/engine-architecture/runtime-interface-cdylib-loader.md
  - docs/engine-architecture/runtime-interface-convergence.md
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/engine-architecture/runtime-interface-convergence.md
status: in_progress
last_refined: 2026-06-13
---

# 10 dynamic_api 与 runtime_interface 收敛线

子计划 06 显式排除的"另一条收敛线"落成计划：cdylib 函数表 ABI、session 生命周期出口、interface 契约纯净性与 UI 镜像契约的漂移治理。**native 插件 ABI（NativePluginAbiV3 族）归 06，不在本计划**；editor 客户端实现归 editor 计划。既有口径承接 `docs/engine-architecture/runtime-interface-convergence.md` 与 `runtime-interface-cdylib-loader.md`（2026-06-12 实测存在），本计划不另起口径，只把其目标态落成切片。

## 现状与证据（2026-06-12 实仓盘点）

- **C 出口单点**：`dynamic_api/exports.rs` 仅 1 个 `#[no_mangle] pub unsafe extern "C" fn zircon_runtime_get_api_v1(`（:25-26）——出口面已极窄（健康项）。session 级 C ABI 函数（如 `tick_frame` :301）经函数表分发而非独立符号。
- **函数表双族、版本不同步**：`runtime_api/api_table.rs` 有 `ZrHostApiV1`（:43，宿主回调面）与 `ZrRuntimeApiV1`（:63，runtime 服务面）；`plugin_api.rs` 有 `ZrHostApiV3`（:41）+ 子 API `ZrHostEcsApiV1`/`ZrHostAssetApiV1`/`ZrHostEventApiV1`/`ZrHostDiagnosticsApiV1`（:65/:83/:95/:111）+ `ZrPluginStateSnapshotApiV1`（:207）+ `ZrPluginApiV1`（:227）——**runtime 表 V1 与 plugin 宿主表 V3 并存，子 API 各自 V1**，版本演进规则（何时 bump、矩阵谁维护）未定稿。
- **interface 依赖面已纯净**（01 计划核实）：`zircon_runtime_interface` 依赖仅 glam/serde/serde_json/thiserror/toml/unicode-segmentation/uuid，无 wgpu/winit——守卫已由 01-M1 切片 1.4 锁定。
- **UI 镜像契约面巨大**：`zircon_runtime_interface/src/ui/` 22 条目（含 `v2/`、`template/asset/component_contract/api_version.rs` 的 `UiComponentApiVersion` :8 带 parse error 类型 :80），与 `zircon_runtime/src/ui/` 同构——共享 DTO 与重复定义的甄别、同步规则与漂移守卫缺失（09 计划的 runtime 侧形状收束后，移交清单落到本计划 M2）。
- **支撑件**：`handles.rs`（句柄）、`buffer.rs`（状态/缓冲契约）、`status.rs`、`version.rs`（版本常量）、`reflect/`、`resource/`、`profiling.rs`、`plugin_events.rs`、`plugin_diagnostics.rs`、`manifest.rs`、`math.rs`。
- 加载侧锚：`zircon_app` 经 libloading 动态加载 runtime 并经本 interface 对话（CLAUDE.md；`runtime-interface-cdylib-loader.md`）。
- 参考锚点（每点一行）：Fyrox dylib 插件函数边界 — `dev/Fyrox/fyrox-impl/src/plugin/dylib.rs`；本仓 native 插件 ABI 版本协商（06 计划已细化）作同构参照。

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
| M0 | 0.1 ABI 清册 | completed | 2026-06-12 | `docs/engine-architecture/runtime-interface-convergence.md#runtime-10-abi-inventory` 新增函数表族、跨界 DTO 域、session 操作面三张清册；源码扫描确认函数表 9 个、`ZrRuntimeApiV1` 13 字段，其中函数指针 11 个，session 操作面 11 项 |
| M0 | 0.2 版本矩阵 | completed | 2026-06-12 | 同一文档新增 Version Strategy：函数表任意字段增删/重排/类型/语义变化均 bump 新表版本；`size_bytes` 仅作显式协商/诊断字段；动态 runtime DTO 由 `ZIRCON_RUNTIME_ABI_VERSION_V1` 统管，plugin host 子表按窄表 bump |
| M1 | 1.1 repr(C) 与 ABI 清册守卫 | full_interface_package_passed | 2026-06-12 | 新增 `zircon_runtime_interface/src/tests/abi_safety_contracts.rs` 并接入 `tests/mod.rs`；`function_table_structs_are_all_repr_c` 锁定 9 个 `Zr*Api*` 函数表结构；`interface_public_signatures_stay_free_of_dynamic_object_exports` 扫描公开签名禁入词；`repr_c_guard_fails_on_missing_local_attribute` 与 `public_signature_guard_fails_on_dynamic_object_export` 提供负例自检；`function_table_field_counts_match_runtime_10_inventory` 锁定 M0 ABI 清册字段数矩阵（`ZrRuntimeApiV1` 13 字段及 plugin/host 表字段数），`runtime_api_session_operation_surface_matches_inventory` 锁定 `ZrRuntimeApiV1` 的 11 个 session 操作字段顺序，`runtime_10_version_strategy_rejects_in_place_table_shape_changes` 锁定保守版本策略与 `ZIRCON_RUNTIME_ABI_VERSION_V1` 常量 owner；`docs/engine-architecture/runtime-interface-convergence.md` 已同步命名字段数/操作面守卫；`rustfmt --edition 2021 --check zircon_runtime_interface\src\tests\abi_safety_contracts.rs` 通过；`cargo test -p zircon_runtime_interface inventory --locked --message-format short --color never` 通过 2/2；`cargo test -p zircon_runtime_interface version_strategy --locked --message-format short --color never` 通过 1/1；`cargo test -p zircon_runtime_interface abi_safety_contracts --locked --message-format short --color never` 通过 7/7；`cargo test -p zircon_runtime_interface --locked --message-format short --color never` 通过 165/165，doc-test 0/0 |
| M1 | 1.2 session 失败路径 | code_complete_static_passed_cargo_timeout | 2026-06-12 | `RuntimeDynamicSession.render_bridge` 改为 `Option<RuntimeRenderBridge>`；`minimal`/`headless` profile 通过 `uses_render_bridge()` 跳过 render bridge，capture 返回空帧，surface bind/present 为 no-op；`create_test_session` 现在显式创建 headless session；新增 `session_destroy_reports_explicit_not_found_after_headless_destroy`、`destroyed_headless_session_entry_points_reject_old_handle`、`create_session_accepts_named_headless_profile_without_render_bridge`、`minimal_and_headless_profiles_skip_render_bridge_bootstrap`，并保留缺失非零句柄/坏句柄/registry removal 守卫；`rustfmt --edition 2021 --check zircon_runtime\src\dynamic_api\session.rs zircon_runtime\src\dynamic_api\tests\session_lifecycle.rs zircon_runtime\src\dynamic_api\tests\support.rs` 通过；`git diff --check` 对动态 API 代码/计划/会话文档通过（仅 LF-to-CRLF warning）；冲突标记/尾随空白扫描通过；optional render bridge/source-token 扫描通过；首次 `cargo test -p zircon_runtime --lib destroy_session --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-10-headless-0612 --message-format short --color never -- --nocapture` 在另一条 `zircon_runtime` cargo lane 活跃时 904s 超时，孤立验证进程已停止；第二次在通道清空后启动同一命令，仍在测试二进制编译阶段 904s 超时，期间 render 检查 lane 又启动，本会话启动的孤立验证进程已停止；未声明 Cargo pass |
| M2 | 2.1 重复定义消化 | 待开始 | — | — |
| M2 | 2.2 v2 契约同步 | 待开始 | — | — |
| M3 | 3.1 重载失败注入 | scoped_cargo_passed_pending_full_package | 2026-06-12 | `LoadedRuntime` 新增私有 `validate_runtime_api_pointer(...)` 表校验入口；`runtime_api_pointer_rejects_null_from_entry_symbol`、`runtime_api_pointer_rejects_version_mismatch_before_session_creation`、`runtime_api_pointer_rejects_missing_required_functions_before_session_creation`、`runtime_library_loader_reports_missing_entry_symbol_source_path`、`runtime_library_loader_reports_missing_entry_symbol_from_dynamic_library`、`runtime_session_create_reports_first_call_failure_context` 已补入 `zircon_app/src/entry/runtime_library/tests.rs`；首次 `cargo test -p zircon_app --lib runtime_api_pointer_rejects --locked --message-format short --color never` 超时 304s 未返回结果，后续重跑通过 3/3；`cargo test -p zircon_app --lib runtime_library_loader_reports_missing_entry_symbol_source_path --locked --message-format short --color never` 通过 1/1；`cargo test -p zircon_app --lib runtime_library_loader_reports_missing_entry_symbol_from_dynamic_library --locked --message-format short --color never` 通过 1/1；`cargo test -p zircon_app --lib runtime_session_create_reports_first_call_failure_context --locked --message-format short --color never` 通过 1/1；完整 `cargo test -p zircon_app --locked` 仍待测试窗口 |

基线数值（开工首日记录）：

- C 出口基线：1（`zircon_runtime_get_api_v1`，exports.rs:25-26）
- 函数表基线：Zr*Api* 结构 9 个（api_table.rs 2 + plugin_api.rs 7）
- `repr(C)` 覆盖基线：14（`runtime_api/api_table.rs` + `plugin_api.rs`，其中 `Zr*Api*` 函数表结构 9/9 由 `abi_safety_contracts` 守卫）
- interface `ui/` 条目基线：22；重复定义候选数：__（M2 输入）
- `cargo test -p zircon_runtime_interface --locked` 通过数基线：__

## 风险与协调

- **与 06 的边界**：plugin_api.rs 的 `ZrHostApiV3` 族被 native loader 消费（06 地盘的宿主侧）——本计划只管表结构的 ABI-safe 与版本策略，函数语义与协商实现归 06；两计划的版本矩阵必须互引同口径。
- 既有收敛文档（`runtime-interface-convergence.md`）可能由 `zr-runtime-interface-convergence` skill 对应的会话维护——动工前确认无活跃会话在改该文档；有则并入其口径执行。
- M2 重复定义消化横跨 runtime/interface/editor 三 crate：每对独立切片、独立提交，三 crate 同测后再下一对。
- `dynamic_api/**` 是 10fps 会话触及区：切片前 `git status`，禁止回退。
- 08 计划若裁决补实体 generation（实体 ID 表示变化），`handles.rs` 的实体句柄 ABI 表示需联动——08/10 互为执行前检查项。
