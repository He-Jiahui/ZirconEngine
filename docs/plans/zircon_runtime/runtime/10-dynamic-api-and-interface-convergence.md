---
related_code:
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/profile.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/resolve_asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager_handle.rs
  - zircon_runtime/src/dynamic_api/tests/host_request_payloads.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/surface.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/abi.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/behavior.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/diagnostics.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/docs.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/host_requests.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/slices.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/source_inventory.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared/split_layout.rs
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
last_refined: 2026-07-17
---

# 10 dynamic_api 与 runtime_interface 收敛线

子计划 06 显式排除的"另一条收敛线"落成计划：cdylib 函数表 ABI、session 生命周期出口、interface 契约纯净性与 UI 镜像契约的漂移治理。**native 插件 ABI（NativePluginAbiV3 族）归 06，不在本计划**；editor 客户端实现归 editor 计划。既有口径承接 `docs/engine-architecture/runtime-interface-convergence.md` 与 `runtime-interface-cdylib-loader.md`（2026-06-12 实测存在），本计划不另起口径，只把其目标态落成切片。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-runtime-dynamic-api-interface-convergence",
  "goal": "收敛动态 runtime session、版本化服务句柄、cdylib ABI 与 runtime_interface 契约边界。",
  "milestones": [
    {"id": "M1", "title": "ABI 面盘点（正文 M0）", "depends_on": []},
    {"id": "M2", "title": "ABI-safe 结构守卫（正文 M1）", "depends_on": []},
    {"id": "M3", "title": "UI 镜像契约漂移治理（正文 M2）", "depends_on": ["M2"]},
    {"id": "M4", "title": "cdylib 重载路径收尾（正文 M3）", "depends_on": ["M3"]}
  ]
}
```

<!-- workflow schema 不接受 M0，因此机器节点 M1-M4 映射正文 M0-M3；正文 M0/M1 的实现与证据早于协调器节点，机器 M2 作为 late-adoption 独立可提交切片，正文任务顺序仍为权威。 -->

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

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`10/2026-07-09-dynamic-api-and-interface-convergence-output-records.md`](10/2026-07-09-dynamic-api-and-interface-convergence-output-records.md)
- fixed 已修复：[dynamic-runtime-v1-fallback-reintroduced](../../zircon_editor/editor/03/fixed-2026-07-16-dynamic-runtime-v1-fallback-reintroduced.md)
- 2026-07-17 F0/F1 项目启动单 prepared owner：`RuntimeProjectConfig::prepare` 在插件选择前只打开一次 `ProjectManager`，`RuntimePreparedProject` 持有同一 manifest 快照并通过抽象 `AssetManager::open_prepared_project` 移交原 manager；具体激活仍归 Runtime04 `ProjectAssetManager`。Scene 通过 `AssetManager::current_project_snapshot` 在短读锁内取得已扫描快照，释放锁后加载默认场景，不再二次 `ProjectManager::open + scan_and_import`，也不在 manager 锁内执行外部 I/O；路径式 `AssetManager::open_project` 同样委托该激活 owner。强化源码合同已按“具体 resolver / 二次 open-scan / 锁内回调”真实 RED→GREEN。受管 production check job `cc5e2e75a7c447c092b6afe72954e42d` / run `f24239e0baf34216bea0ed95d955bf56` 执行 `cargo check -p zircon_runtime --lib --locked`，exit 0（5m17s）。受管 focused lib-test job `1c78ff96a67d44f4ad80f47704e720fc` / run `7878fa0242cd48b5a67eee0f0b6e62bb` 执行 `cargo test -p zircon_runtime --lib --locked project_startup_snapshot_survives_disk_manifest_rewrite -- --test-threads=1`，原始输出确认两个目标均 `ok`，`2 passed / 0 failed / 8185 filtered`，exit 0；独立规范复审 `ACCEPTED`，质量复审 `critical=0 / important=0`。父计划保持 `in_progress`，不得据此宣告 F0/F1 或 Runtime10 完成。
- 2026-07-17 P0 空 host-request ABI fast-path：`drain_host_requests` 在请求集合为空时直接写出 canonical `ZrOwnedByteBuffer::empty()`，跳过 JSON 编码、分配和释放 owner；非空 batch 的 schema、owned-buffer 所有权和释放路径保持不变，既有 `zircon_app` consumer 在解码前已显式接受 empty buffer。受管 Windows lib-only job `b0ea82ad0943466794e3af3c5333816b` / run `4b9e4151d39f4cd9b95de28b2c0ee261` 执行 `cargo test -p zircon_runtime --lib --locked dynamic_session_drains_runtime_ime_cursor_area_and_surrounding_text_requests_once -- --test-threads=1`，原始输出为 `1 passed / 0 failed / 8190 filtered`、exit 0；Performance01 failure 已 canonical return 为 [`../../performance/01/fixed-2026-07-17-empty-host-request-batch.md`](../../performance/01/fixed-2026-07-17-empty-host-request-batch.md)，独立规范与质量复审均为 `critical=0 / important=0`。该项只证明 Runtime10 ABI 空批次契约，不宣告全 runtime 性能或 MVP 完成。
- open 待修复：[editor-selection-state-runtime-session-boundary](10/failure-2026-07-17-editor-selection-state-runtime-session-boundary.md)；Editor01 M2.3 已证明 `selected_node` 是 construction-only 默认 orbit anchor 的过期 authoring 命名，Runtime10 负责删除字段、事件同步 helper 与固化旧边界的结构锚，不得用改名字段或兼容 setter 保留第二份选择真相。
- open 待修复：[dynamic-api-owner-status-anchor-loss](15/failure-2026-07-17-dynamic-api-owner-status-anchor-loss.md)；Runtime10 `dynamic_api` 上行门暴露 Runtime15 current parent/index 与既有 Dynamic API child-owner/status records 的五组镜像断链，修复责任归 Runtime15，不得由 Runtime10 放宽结构守卫。
- 2026-07-18 M2 reactive wake V3 interface owner：`zircon_runtime_interface` 已静态实现 V3-only 函数表、`ZrRuntimeSessionConfigV2`/`ZrRuntimeWakeSinkV1`、raw-`u32` `ZrRuntimeFrameDemandV1` 与 V2 retirement/layout/version guards；旧 V2 表/符号/get-api type、ConfigV1、CreateSessionFnV1、TickFrameFnV1 已从 interface 生产面硬删除，无 alias/forwarder。当前状态为 `implemented_static_pending_atomic_runtime_app_migration`：rustfmt 与精确范围 `git diff --check` 通过；按批准设计，Runtime10 runtime export/lifecycle 与 Runtime03 app loader/cadence/真实 producer 全部迁移并取得受管 Cargo 证据前，本项不得独立提交或宣告 M2 完成。

## 性能审阅交接

- 2026-07-22 plugin event scene-producer补充：`scene/event_mirror` typed cursor当前已把全部未读事件逐条转`serde_json::Value`并一次性collect，dynamic session随后再复制descriptor/per-delivery String、构造第二Vec并整批encode；bounded budget必须从typed read开始，不能只在ABI尾端截断。Scene成功drain的无用event-id clone已止损，其余继续归PERF-MVP-432与open [`10/failure-2026-07-19-plugin-event-bounded-delivery.md`](10/failure-2026-07-19-plugin-event-bounded-delivery.md)。
- 2026-07-22 native callback panic边界联动：Plugin SDK已静态删除per-callback process-global panic-hook交换，直接`catch_unwind`并映射ABI panic status；Runtime10验收宿主`ffi_panic_guard`与插件侧状态/diagnostics一致，Runtime06拥有最终native lifecycle与并发回归。见PERF-MVP-491及`../06/failure-2026-07-22-native-sdk-callback-global-panic-hook.md`。
- 2026-07-22 native command/output ABI性能补充：PERF-MVP-542要求load generation预编译dense command slot与稳定NUL-safe identity，stable callback不再每次构造`CString`；大output改caller-provided bounded sink/buffer或明确统一allocator transfer合同，Windows跨CRT禁止直接接管未知Vec内存。Runtime10负责ABI version/layout/status/free/panic协商与旧版硬切，Plugins01负责generation/lifecycle；0/1KiB/1MiB/256MiB输出记录alloc/copy/RSS/caller wall。
- 2026-07-22 editor gateway/consumer补充：owned ABI output重复validate已在Editor侧删除；Runtime10必须让tick返回完整frame demand而非consumer恒true，并为plugin event drain提供cursor+`max_events/max_bytes/deadline`、remaining/oldest-age，预算从typed producer开始。viewport正常帧以GPU/generation handle跨边界，foreign RGBA Vec copy只保留显式跨进程capture/fallback；对应PERF-MVP-424/069/023。
- 2026-07-23 runtime-interface ABI foundation补充：`buffer.rs`、`handles.rs`、`manifest.rs`、`runtime_api.rs`、`status.rs`、`version.rs`、`lib.rs` 7/7静态审查确认Copy/POD基础无独立热点；`ZrOwnedByteBuffer`虽可零额外复制地移交runtime原Vec，但Editor `capture_frame`仍`to_vec()`整份foreign RGBA后free，继续归PERF-MVP-023，native command owned output双owner继续归542。profile/operation/host-request/plugin-event consumer直接从foreign bytes serde decode，没有第二次raw Vec copy，但producer非空page仍每次`to_vec` allocation，继续受既有page/bytes/deadline门禁。V3 re-export/version三文件为current foreign dirty cutover，本交接只读保留，不作为atomic runtime/app migration或Cargo验收。
- 2026-07-23 runtime-interface `runtime_api/**`补充：10/10静态审查确认V3 `FrameDemand`在App entry已映射到cadence，但Editor gateway仍只校验后恒返true，继续按PERF-MVP-424完成所有host的demand传递；host-request无budget归425，plugin drain无request budget/cursor归069/432，operation poll owned String+JSON归435/430，event逐条ABI归426/314。新增PERF-MVP-565不由Runtime10建立第二capability truth；Runtime10只需让tick/demand和bounded ABI transport不触发Editor每帧全量control-plane工作。`api_table.rs`与新增`frame_demand.rs`/`session.rs`为current foreign dirty，本轮只读保留。
- 2026-07-23 profile-control ABI容量补充：`ProfileCaptureConfig`的非零max entries和wide String当前无hard byte ceiling，`ProfileControlResponse`也可携带多个宽snapshot/report。Runtime10按PERF-MVP-566在decode/encode边界验证finite/effective config与最大output page bytes，但只消费Runtime07唯一recorder budget/generation；不得建立动态库专用第二ring，也不得以截断最终JSON替代producer端eviction/drop诊断。
- 2026-07-23 App runtime-library owned-output/teardown补充：PERF-MVP-574要求先冻结error-after-output所有权，再让App对frame/host request/plugin event/operation out-param从status与decode前建立RAII exactly-once free；cleanup错误组合诊断但不覆盖primary status。`destroy_session`失败不得以可重复的永久forget作为常规终态，需显式wake detach+destroy retry，或count/bytes/age硬有界且可观测的quarantine。fake FFI覆盖0/1KiB/64MiB output、success/error/invalid JSON/wrong ABI/free failure；1/1k/100k failed destroy证明无UAF，leaked bytes=0且registry/proxy为0或硬有界。继续使用[`10/failure-2026-07-19-app-entry-host-request-and-wake-boundary.md`](10/failure-2026-07-19-app-entry-host-request-and-wake-boundary.md)，不得创建重复failure。

## Code Review 建议 (2026-07-30)

### 与代码现状不符，需修订

- 正文 M0/M1/M3 全篇以 `ZrRuntimeApiV1` / `zircon_runtime_get_api_v1` 为对象，但当前代码已硬切到 V3-only，且状态节 2026-07-18「reactive wake V3 interface owner」已明确 V2 表/符号/get-api type「已从 interface 生产面硬删除，无 alias/forwarder」。正文与状态节自相矛盾，建议整体把 V1 口径改写为 V3：
  - §「现状与证据」第 1 点「`dynamic_api/exports.rs` 仅 1 个 `#[no_mangle] ... zircon_runtime_get_api_v1(`（:25-26）」已过时。实测 `zircon_runtime/src/dynamic_api/exports.rs:41-51` 唯一 `#[no_mangle]` 出口是 `zircon_runtime_get_api_v3(host: *const ZrHostApiV1) -> *const ZrRuntimeApiV3`，已带 `catch_unwind` 顶层 containment；表实例为 `RUNTIME_API_V3`（:19-39），含 18 个 `_ffi` wrapper 入口且全部经 `catch_ffi_panic`（:70-78）委派到 `session.rs` 的私有 Rust-ABI owner。
  - §「现状与证据」第 2 点「runtime 表 V1 与 plugin 宿主表 V3 并存」已过时。实测 `zircon_runtime_interface/src/runtime_api/api_table.rs` 只有 `ZrHostApiV1`（:48）与 `ZrRuntimeApiV3`（:72），load 符号常量为 `ZR_RUNTIME_GET_API_SYMBOL_V3 = b"zircon_runtime_get_api_v3\0"`（:15），版本常量 `ZIRCON_RUNTIME_API_VERSION_V3`（:97）。`ZrRuntimeApiV1` 已不存在。
  - M1 切片 1.3 DoD/验收把守卫锚在 `zircon_runtime_get_api_v1` 与 `ZrRuntimeApiV1` 函数表；当前 FFI panic 边界已在 V3 出口与 18 个 `_ffi` wrapper 上实现（`exports.rs:41-51,70-78,80-202`）。建议把切片 1.3 的入口名、表名、`runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary` 的对象全部改为 V3，并标注「已落地」。
  - M3 切片 3.1「符号缺失（`zircon_runtime_get_api_v1` 不存在）」「版本协商失败」测试名草案 `runtime_api_version_mismatch_is_rejected_before_session_creation` 需改为 V3 符号 `zircon_runtime_get_api_v3` 与 V3 版本常量，否则失败注入测试会锚在不存在的符号上。

### 设计优化建议

- 状态节 2026-07-18 项自评为 `implemented_static_pending_atomic_runtime_app_migration`，但 `exports.rs` 已实实在在导出 V3-only 表且无 V1/V2 残留。建议复核该状态标签是否仍准确：若 runtime export 侧迁移已完成（代码证据支持），应把「pending atomic runtime/app migration」收窄为仅 app loader/cadence 侧，并在 M0 版本矩阵节把「RuntimeApi V1 / plugin HostApi V3 并存」更新为「RuntimeApi V3 单当前版 + HostApi V1 宿主回调面」的实际现状。

### 验证缺口

- `host_abi_is_supported`（`exports.rs:63-68`）对 `host.is_null()` 直接返回 `true`（放行 null host），仅在非 null 时校验 `abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1`。M3 切片 3.1 的「版本协商失败」失败注入应显式覆盖：(a) 非 null host 且 abi_version 不匹配 → 返回 null 表；(b) null host 的放行语义是否为有意契约。当前计划未把「null host 放行」列为待验证契约，建议在 M3 补一条守卫锚定该分支语义，避免后续被误判为漏校验。
