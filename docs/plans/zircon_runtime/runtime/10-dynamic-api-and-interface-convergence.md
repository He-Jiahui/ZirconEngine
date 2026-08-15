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
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/handle.rs
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
last_refined: 2026-08-01
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

- **C 出口单点**：`dynamic_api/exports.rs` 当前唯一 `#[no_mangle]` 出口是 `zircon_runtime_get_api_v3(host: *const ZrHostApiV1) -> *const ZrRuntimeApiV3`；入口用 `catch_unwind` containment，`RUNTIME_API_V3` 的 18 个函数指针全部指向 `_ffi` wrappers，再委派到 `session.rs` 私有 Rust-ABI owner。V1/V2 runtime 表与符号已硬删除。
- **函数表版本面已收敛**：`runtime_api/api_table.rs` 的宿主回调面为 `ZrHostApiV1`，runtime 服务面只有冻结的 `ZrRuntimeApiV3`；加载符号为 `ZR_RUNTIME_GET_API_SYMBOL_V3`。`plugin_api.rs` 的 native-plugin `ZrHostApiV3` 与各 V1 子 API 属于独立版本域，不再描述为 runtime V1/V3 双族漂移。当前机器清册继续由 `dynamic_runtime_api_boundary` 复核 repr、字段与禁入类型。
- **interface 依赖面已纯净**（01 计划核实）：`zircon_runtime_interface` 依赖仅 glam/serde/serde_json/thiserror/toml/unicode-segmentation/uuid，无 wgpu/winit——守卫已由 01-M1 切片 1.4 锁定。
- **UI 镜像契约面巨大**：`zircon_runtime_interface/src/ui/` 22 条目（含 `v2/`、`template/asset/component_contract/api_version.rs` 的 `UiComponentApiVersion` :8 带 parse error 类型 :80），与 `zircon_runtime/src/ui/` 同构——共享 DTO 与重复定义的甄别、同步规则与漂移守卫缺失（09 计划的 runtime 侧形状收束后，移交清单落到本计划 M2）。
- **支撑件**：`handles.rs`（句柄）、`buffer.rs`（状态/缓冲契约）、`status.rs`、`version.rs`（版本常量）、`reflect/`、`resource/`、`profiling.rs`、`plugin_events.rs`、`plugin_diagnostics.rs`、`manifest.rs`、`math.rs`。
- 加载侧锚：`zircon_app` 经 libloading 动态加载 runtime 并经本 interface 对话（CLAUDE.md；`runtime-interface-cdylib-loader.md`）。
- 参考锚点（每点一行）：Fyrox dylib 插件函数边界 — `dev/Fyrox/fyrox-impl/src/plugin/dylib.rs`；本仓 native 插件 ABI 版本协商（06 计划已细化）作同构参照。

补充参考锚点（2026-06-13 实测核验，实现型切片动工前先读——index 公约 §7.9）：

- Godot GDExtension：C ABI 函数表注册/装载/版本协商的成熟实现（M0 版本矩阵、M3 装载失败路径对照）— `dev/godot/core/extension/{gdextension.{h,cpp},gdextension_function_loader.{h,cpp}}`；API 面 dump/版本化 — `extension_api_dump.cpp`

## 目标

1. 函数表版本策略维持：`ZrRuntimeApiV3` 是 runtime 单一当前表，任何字段变化必须新建版本并原子硬切；`ZrHostApiV1` 宿主回调面与 native-plugin `ZrHostApiV3`/子 API 各自独立演进，协商失败必须显式。
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
- 改动形态：纯文档。清册三表：(a) 函数表族（逐表列字段数、版本、消费方 crate）；(b) 跨界 DTO 域（handles/buffer/status/manifest/reflect/resource/ui/...，逐域列 `#[repr(C)]` vs serde-序列化两类传输形态）；(c) session 操作面（经 `ZrRuntimeApiV3` 可达的 18 个函数指针，含 create/destroy、event/frame/surface、profile/tick、host/plugin drains 与 operation 族）。
- 验收：三表齐备；每个函数表有版本与消费方两列。
- DoD：清册落文档；后续守卫以清册为白名单源。

#### 切片 0.2 版本矩阵与 bump 规则定稿

- 目标文件：同 0.1 文档（"版本策略"节）。
- 改动形态：决策记录——矩阵现状（RuntimeApi V3 / HostApi V1 / plugin 宿主 HostApi V3 / 各版本化子 API）+ 已冻结规则：`ZrRuntimeApiV3` 任何字段变化都要求新表版本与所有 dynamic host 原子硬切，不做尾追兼容；`version.rs` 与 API table 的版本对应保持单点。
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

#### 切片 1.3 FFI panic 边界（已落地，保留 current-source 验收）

- 目标文件：`zircon_runtime/src/dynamic_api/exports.rs`、`zircon_runtime/src/dynamic_api/tests/api_table.rs`、`docs/zircon_runtime/dynamic_api/session.md`。
- 当前实现：`zircon_runtime_get_api_v3` 已有顶层 `catch_unwind`，18 个 `ZrRuntimeApiV3` entry points 全部经过 `catch_ffi_panic` `_ffi` wrapper 后委派给 `session.rs` owner；panic 映射为 `ZrStatusCode::Panic`，获取表期间 panic 返回 null。
- 调用方迁移：`zircon_app` 已按 `ZR_RUNTIME_GET_API_SYMBOL_V3` 加载并校验 `ZIRCON_RUNTIME_API_VERSION_V3`、alignment、size 与必需函数字段；不保留 V1/V2 fallback。
- 剩余验收：current-source dynamic API 与 app loader Cargo；显式锁定非 null 错 ABI 返回 null，以及 null host 放行是否继续作为内嵌加载契约。
- DoD：函数表不得直接指向 session owner；session owner 不恢复 `extern "C"`；无 V1/V2 symbol/table/alias；focused tests 与 app loader 失败注入通过。

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

- 目标文件：加载侧 `zircon_app`（libloading 装载点，执行时核验 `libloading|zircon_runtime_get_api_v3|ZR_RUNTIME_GET_API_SYMBOL_V3`）+ `runtime-interface-cdylib-loader.md` 口径刷新。
- 改动形态：对照 06-M3 的热重载回滚测试模式，保持 runtime cdylib 三类装载失败路径：V3 符号缺失、`ZIRCON_RUNTIME_API_VERSION_V3`/table size 协商失败、装载后首调用失败；不得回退 V1/V2 表。
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
- 2026-07-18 M2 reactive wake V3 interface owner（2026-08-10 current-source 对齐）：interface、runtime export 与 `zircon_app` loader 均已硬切 V3-only；旧 V1/V2 表/符号/type 与 fallback 为 0。Runtime03 current-source 已把 session wake sink 接入项目 generation change producer，并在 scene reload pending 期间保持 `Immediate` demand 直至收敛；当前状态为 `source_repair_complete_pending_managed_cargo_wpr_and_product_validation`，不再把已完成的 API migration 或 producer 源码写成 pending。
- open 待修复：[zrvm-vampire-behavior-test-ownership-gap](../../zircon_plugins/08/failure-2026-08-01-zrvm-vampire-behavior-test-ownership-gap.md)；10 个 RuntimeDynamicSession Vampire tests 被永久 ignore 并声称真实覆盖已迁至 Plugins08，但插件树没有等价 Vampire 行为测试。Plugins08 必须先建立 real-backend gameplay/HUD/menu/diagnostics 覆盖，Runtime10/Runtime15 才能删除 1,263 行旧 test/support owner。

## 性能审阅交接

- 2026-07-22 plugin event scene-producer补充：`scene/event_mirror` typed cursor当前已把全部未读事件逐条转`serde_json::Value`并一次性collect，dynamic session随后再复制descriptor/per-delivery String、构造第二Vec并整批encode；bounded budget必须从typed read开始，不能只在ABI尾端截断。Scene成功drain的无用event-id clone已止损，其余继续归PERF-MVP-432与open [`10/failure-2026-07-19-plugin-event-bounded-delivery.md`](10/failure-2026-07-19-plugin-event-bounded-delivery.md)。
- 2026-07-22 native callback panic边界联动：Plugin SDK已静态删除per-callback process-global panic-hook交换，直接`catch_unwind`并映射ABI panic status；Runtime10验收宿主`ffi_panic_guard`与插件侧状态/diagnostics一致，Runtime06拥有最终native lifecycle与并发回归。见PERF-MVP-491及`../06/failure-2026-07-22-native-sdk-callback-global-panic-hook.md`。
- 2026-07-22 native command/output ABI性能补充：PERF-MVP-542要求load generation预编译dense command slot与稳定NUL-safe identity，stable callback不再每次构造`CString`；大output改caller-provided bounded sink/buffer或明确统一allocator transfer合同，Windows跨CRT禁止直接接管未知Vec内存。Runtime10负责ABI version/layout/status/free/panic协商与旧版硬切，Plugins01负责generation/lifecycle；0/1KiB/1MiB/256MiB输出记录alloc/copy/RSS/caller wall。
- 2026-07-22 editor gateway/consumer补充：owned ABI output重复validate已在Editor侧删除；Runtime10必须让tick返回完整frame demand而非consumer恒true，并为plugin event drain提供cursor+`max_events/max_bytes/deadline`、remaining/oldest-age，预算从typed producer开始。viewport正常帧以GPU/generation handle跨边界，foreign RGBA Vec copy只保留显式跨进程capture/fallback；对应PERF-MVP-424/069/023。
- 2026-07-23 runtime-interface ABI foundation补充：`buffer.rs`、`handles.rs`、`manifest.rs`、`runtime_api.rs`、`status.rs`、`version.rs`、`lib.rs` 7/7静态审查确认Copy/POD基础无独立热点；`ZrOwnedByteBuffer`虽可零额外复制地移交runtime原Vec，但Editor `capture_frame`仍`to_vec()`整份foreign RGBA后free，继续归PERF-MVP-023，native command owned output双owner继续归542。profile/operation/host-request/plugin-event consumer直接从foreign bytes serde decode，没有第二次raw Vec copy，但producer非空page仍每次`to_vec` allocation，继续受既有page/bytes/deadline门禁。V3 re-export/version三文件为current foreign dirty cutover，本交接只读保留，不作为atomic runtime/app migration或Cargo验收。
- 2026-07-23 runtime-interface `runtime_api/**`补充：10/10静态审查确认V3 `FrameDemand`在App entry已映射到cadence，但Editor gateway仍只校验后恒返true，继续按PERF-MVP-424完成所有host的demand传递；host-request无budget归425，plugin drain无request budget/cursor归069/432，operation poll owned String+JSON归435/430，event逐条ABI归426/314。新增PERF-MVP-565不由Runtime10建立第二capability truth；Runtime10只需让tick/demand和bounded ABI transport不触发Editor每帧全量control-plane工作。`api_table.rs`与新增`frame_demand.rs`/`session.rs`为current foreign dirty，本轮只读保留。
- 2026-07-23 profile-control ABI容量补充：`ProfileCaptureConfig`的非零max entries和wide String当前无hard byte ceiling，`ProfileControlResponse`也可携带多个宽snapshot/report。Runtime10按PERF-MVP-566在decode/encode边界验证finite/effective config与最大output page bytes，但只消费Runtime07唯一recorder budget/generation；不得建立动态库专用第二ring，也不得以截断最终JSON替代producer端eviction/drop诊断。
- 2026-07-23 App runtime-library owned-output/teardown补充：PERF-MVP-574要求先冻结error-after-output所有权，再让App对frame/host request/plugin event/operation out-param从status与decode前建立RAII exactly-once free；cleanup错误组合诊断但不覆盖primary status。`destroy_session`失败不得以可重复的永久forget作为常规终态，需显式wake detach+destroy retry，或count/bytes/age硬有界且可观测的quarantine。fake FFI覆盖0/1KiB/64MiB output、success/error/invalid JSON/wrong ABI/free failure；1/1k/100k failed destroy证明无UAF，leaked bytes=0且registry/proxy为0或硬有界。继续使用[`10/failure-2026-07-19-app-entry-host-request-and-wake-boundary.md`](10/failure-2026-07-19-app-entry-host-request-and-wake-boundary.md)，不得创建重复failure。

## Code Review 同步结论 (2026-07-30，2026-08-01 落实)

- 旧 V1 runtime 表、符号与 pending atomic runtime/app migration 叙述已按 current source 修正为 V3-only；本节不再保留已完成的“建议修订”清单。
- `host_abi_is_supported` 对非 null host 校验 `ZIRCON_RUNTIME_ABI_VERSION_V1`，对 null host 放行。该分支仍需由 M1.3/M3.1 验收明确为内嵌加载契约或收紧，不能由计划文字推定正确。
- Runtime10 仍为 `in_progress`；V3 API 对齐不替代 cadence、failure injection、current-source Cargo 与产品验证。
