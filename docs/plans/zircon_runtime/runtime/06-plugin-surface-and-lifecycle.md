---
related_code:
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/plugin/native.rs
  - zircon_runtime/src/plugin/native_plugin_loader
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs
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
last_refined: 2026-07-31
---

# 06 插件公开面与生命周期收束

## 现状与证据（2026-06-12 重核）

- **公开面实测**：`plugin/mod.rs:27-50` 把 native plugin loader 约 48 个类型/常量全量 `pub use` 到 `plugin` 根：三代 ABI 结构（`NativePluginAbiV1/V2/V3`）、Behavior/ByteSlice/CallbackStatus/EntryReport/HostFunctionTable/OwnedByteBuffer 的 V2/V3 双份、`NativePluginLiveHost` 全家（Command/LoadReport/Outcome）、Runtime 状态快照族（PlayModeSnapshot/StateSnapshot/StateRestoreReport 等）、4 个 STATUS 常量、4 个 ABI_VERSION 常量、4 个 DESCRIPTOR_SYMBOL 常量、2 个 PLAY_MODE 命令常量——与"native loader 退出 runtime 公共主路径"的既定决策（`docs/engine-architecture/native-plugin-boundary.md`，2026-06-12 实测存在）冲突。`plugin` 根的其余导出（manifest/catalog/profile/registration report/extension registry/scene hook/export 族，:17-26、:51-74）属"描述与报告"层，是目标公共面。
- **M1.2/M2/M3 当前状态（2026-07-31）**：M1 fallback focused evidence 已存在但完整 gate pending；M2 root-to-`plugin::native` 源码硬切已落地但 managed acceptance pending；M3 的精确 ABI 矩阵、public-surface 数量和剩余旧类型债以紧随其后的 second-review correction 为唯一当前判词。
- **2026-07-31 second-review ABI/public-surface correction（authoritative）**：上段 2026-07-31 简化成“entry/descriptor V3、behavior/host V4”的二分描述已被本条取代。精确矩阵是 descriptor/entry ABI V3、随 entry descriptor 交给插件的 `NativePluginHostFunctionTableV3` 仍为 first-party 当前合同、behavior callback table V4、runtime-interface host API 当前面为 `ZrHostApiV4`；`NativeHostApiV3RegistrationScope` 是待删除的旧 adapter public surface。V4 registration policy/scope 归独立 `native-host-api-adapter-public-debt` owner，不再误归 bridge-method。当前静态事实为 expected_source_file_count = 17、native root 0/0、native namespace 74/74、分类组 6/6、未分类 0/0、App 8/8、`risks = []`；root hard-cut scanner 已覆盖 direct loader、`native::{...}`、`self::native::*` 与 crate-qualified native re-export 语法。
- **Native public-surface renderer split（2026-06-21）**：`native_plugin_public_surface_markdown.py` 已承接 `render_native_plugin_public_surface_markdown(...)`；`native_plugin_public_surface.py` 保持 400 行 native public-surface scan / symbol classification / M4 gate owner，Markdown owner 为 63 行。Direct probe 当前报告 root re-export 0、native namespace re-export 64、symbol decision groups 5、migration debt 0、unclassified root/namespace symbol counts 0/0、root/native public re-export locations 0/1、M4 gate `classified-and-clear`、risks 0、rendered output 12 lines。该切片不关闭 Runtime 06；script VM / native plugin / app / plugin workspace Cargo-native gates 仍 pending。
- **Native hot-update/replay public-surface audit sync（2026-07-01）**：`NativePluginRuntimeDeltaHotUpdateReport`、`NativePluginRuntimeDeltaHotUpdateRequest`、`NativePluginRuntimeRegistrationReplayReport` 与 `NativePluginRuntimeRegistrationSystemReplay` 已归入既有 native live-host runtime public-surface group，未新增 `zircon_runtime::plugin` root re-export、compat module 或旧路径 shim。当前审计锚点：`root_reexport_count = 0`、`native_namespace_reexport_count = 64`、native root re-export 0/0、native namespace re-export 64/64、M4 gate `classified-and-clear`、debt groups 0/0、native namespace symbol groups 5/5、unclassified native root symbols 0/0、unclassified native namespace symbols 0/0、root public native re-export locations 0/0、public native namespace re-export locations 1/1、native loader test files 4/4、native test namespace import files 3/3、native test root import leaks 0/0、`last_refined = 2026-07-01`、`mirror_docs_guard_present = true`、`risks = []`；验证锚为 standalone plugin_surface_lifecycle 3/3，Cargo/native validation lane 仍 deferred。
- **Native host FFI panic guard（2026-06-22）**：承接 `engine-code-review-findings-2026-06.md` F1 与 `engine-code-structure-convention.md` E7，新增 `native_plugin_loader/ffi_panic_guard.rs`，`host_api_adapter.rs` 的 9 个 `ZrHostApiV3` 回调和 `host_callbacks.rs` 的 4 个 private native host callbacks 均在 `extern "C"` 边界内捕获 panic 并映射为状态码。静态 guard 扫描确认 13/13 callback 路由到 guard；focused Cargo 仍因 core-min 编译 1200s timeout 保持 pending。
- **F8 RuntimePluginDescriptor public-field convergence（2026-06-22）**：`RuntimePluginDescriptor` 的 15 个声明字段已从 public 字段硬切为私有字段，并新增 `descriptor/access.rs` 承接 15 个只读访问器。builtin catalog augmentation、registration validation 与 plugin extension 测试调用点改走 accessor；状态锚为 `runtime_plugin_descriptor_public_field_convergence_coremin_check_passed`。`review_f8_runtime_plugin_descriptor_fields_are_private_with_accessors` 锁定 `RuntimePluginDescriptor private fields 15/15`、访问器签名与文档状态；RuntimePluginDescriptor public-field convergence complete。broader Runtime 06 cargo/native gates 仍 pending。
- **F8 RuntimePluginDescriptor public constructor retired（2026-06-22）**：旧 `RuntimePluginDescriptor::new(...).with_*` 公开构造面已硬切退役，`descriptor/builder/construction.rs` retired，`descriptor/builder/fluent.rs` retired；`RuntimePluginDescriptorBuilder` 直接组装私有字段，builtin catalog 子树改为传递 `BuiltinCatalogDescriptorBuilder` 并在 catalog root 统一 `build()`，`zircon_plugins/plugin_sdk` 的 `RuntimePluginDeclaration` 改为持有 `RuntimePluginDescriptorBuilder`。状态锚为 `runtime_plugin_descriptor_public_constructor_retired_coremin_check_passed`，守卫为 `review_f8_runtime_plugin_descriptor_public_constructor_is_retired`；RuntimePluginDescriptor::new retired。
- **Runtime 15 F8 RuntimePluginDescriptor status mirror cleanup（2026-06-27）**：`runtime_15_runtime_plugin_descriptor_status_mirror_cleanup_static_passed_cargo_deferred` 已把 Runtime 06/index、review findings、结构规范、package-manifest docs 与 status-output 期望同步到当前完成态。新增 `review_f8_runtime_plugin_descriptor_status_mirrors_do_not_claim_public_field_pending`，锁定 RuntimePluginDescriptor private fields 15/15、RuntimePluginDescriptor public-field convergence complete 与 RuntimePluginDescriptor::new retired 不再被旧待办文字覆盖；2026-06-28 `f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred` 进一步把 F8 顶表状态列同步为 `convention + Runtime 04 + Runtime 06 + Runtime 15 / review closed`。不改 RuntimePluginDescriptor 行为。
- **ABI 策略当前态**：descriptor/entry 当前版为 V3，`NativePluginHostFunctionTableV3` 是该 entry 合同的当前 plugin-to-host callback table；behavior callback 当前版为 V4，runtime-interface host API 当前版为 `ZrHostApiV4`。V1/V2 entry/descriptor loader implementation files 0/0，unknown descriptor version 明确拒绝；`NativeHostApiV3RegistrationScope`、V2 byte-slice/buffer/callback-status 物理类型及 V3 alias 是仍待硬切的旧 public surface，禁止以 compatibility 名义冻结。
- **ZrVM 生命周期 owner 当前态（2026-07-14）**：真实实现已硬切到 `zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs`；`call_entry_lifecycle_export`、`activate` / `deactivate` / `saveState` / `restoreState` 和空参数 marshalling 由插件 crate 独占。Runtime06 只保留跨工作区生命周期清单，不恢复 Runtime 内旧 backend、re-export 或 shim。
- **热重载**：runtime/editor 入口归 `native_plugin_loader/native_plugin_live_host/lifecycle.rs`，bridge-lifecycle 变体归 `bridge_lifecycle.rs`，delta/export-root 变体归 `hot_update_application.rs`；所有路径共享 `NativePluginHotReloadState` 回滚语义。计划不再钉易漂移行号，执行时以函数名和 owner 文件核验。
- **下游调用面**：`zircon_app/src` 引用 `NativePlugin*` 当前共 8 文件（app NativePlugin current call-site files: 8）：7 个生产文件 `lib.rs`、`prelude.rs`、`entry/mod.rs`、`entry/export_bootstrap.rs`、`entry/entry_runner/mod.rs`、`entry/entry_runner/bootstrap.rs`、`entry/tests/profile_bootstrap.rs`，以及 1 个测试文件 `entry/entry_runner/editor/tests/gui_startup.rs`。8 个文件均属于 M2 namespace 硬切迁移面；任何新增调用点必须进入同一审计清单。
- Fyrox 锚点（每点一行）：`Plugin`（静态）/`DynamicPlugin`（dylib）双 trait + `PluginContainer` — `dev/Fyrox/fyrox-impl/src/plugin/mod.rs`；热重载"序列化状态→unload→重载→恢复" — `dev/Fyrox/fyrox-impl/src/plugin/dylib.rs`。

补充参考锚点（2026-06-13 实测核验，实现型切片动工前先读——index 公约 §7.9）：

- Godot GDExtension：C ABI 入口装载与符号协商的成熟实现（M2 收窄 facade、M3 版本协商失败路径对照）— `dev/godot/core/extension/gdextension_function_loader.{h,cpp}`、`gdextension.{h,cpp}`

## 目标

1. runtime 插件公共面收窄为"描述与报告"层（manifest/catalog/profile/registration report/extension registry/scene hook/export 族）；native loader 实现细节退出 `plugin` 根。
2. ABI 版本策略定稿：descriptor/entry 与其 plugin-to-host callback table 保持唯一 V3，behavior callback 与 runtime-interface host API 保持唯一 V4；彻底删除 V1/V2 entry/descriptor、V3 host-API registration scope 与 V2 byte DTO/旧 alias，不冻结、不保留 compatibility 名称。
3. VM plugin lifecycle 修复空参数空指针（最高优先，解锁 07 性能取证），失败路径补真实测试。

## 非目标

- 不改插件 framework DTO 的中性原则（行为留在 `zircon_plugins/*/runtime`）；不动 `zircon_runtime_interface` 的函数表 ABI（另一条收敛线）。
- 不在本计划做 VM 之外的脚本后端工作；fallback 后端行为仅在测试分层中涉及。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-runtime-plugin-surface-lifecycle",
  "goal": "收束 VM 生命周期、native loader 公共面与单一当前 ABI，并保留可审计的里程碑提交证据。",
  "milestones": [
    {"id": "M1", "title": "VM 生命周期阻塞修复", "depends_on": []},
    {"id": "M2", "title": "native loader 公共面收窄", "depends_on": ["M1"]},
    {"id": "M3", "title": "ABI 版本策略定稿", "depends_on": ["M2"]}
  ]
}
```

<!-- Workflow topology mirrors the existing M1-M3 plan headings and is maintained independently from milestone output records. -->

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

- 目标文件：`zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs`（及其 `call_module_export` 下行的 sys 调用段，执行时核验确切文件：Grep `call_module_export`，path `zircon_plugins/zr_vm_language/runtime/src/real_backend`）。
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

### M2 native loader 公共面收窄（源码硬切已落地，验收待关闭）

#### 切片 2.1 `plugin::native` 子命名空间收口

- 目标文件：`zircon_runtime/src/plugin/mod.rs`（删 :27-50 的 native 全量 `pub use`）；`plugin/native_plugin_loader/`（模块声明调整）。
- 改动形态（二选一，执行时定稿并记录理由）：
  - (a) native loader 全家改 `pub(crate)`，对外仅保留窄 facade（load report / live host 命令面：`NativePluginLoadReport`、`NativePluginLiveHostCommand`、`NativePluginLiveHostOutcome` 一类，不暴露 ABI 结构体/byte buffer/符号常量）；
  - (b) 收入 `plugin::native` 子命名空间，停止从 `plugin` 根 re-export。
  - 共同点：ABI 结构体、ByteSlice/ByteBuffer、STATUS/SYMBOL/VERSION 常量一律不出现在 `plugin` 根。
- 调用方迁移（当前实测全列，8 文件）：`zircon_app/src/{lib.rs, prelude.rs, entry/mod.rs, entry/export_bootstrap.rs, entry/entry_runner/mod.rs, entry/entry_runner/bootstrap.rs, entry/tests/profile_bootstrap.rs, entry/entry_runner/editor/tests/gui_startup.rs}` 改新路径或经窄 facade；runtime 内部调用方枚举：Grep `plugin::Native`，path `zircon_runtime/src`。同切片删除旧导出，不留桥（硬切换）。
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
- `cargo test -p zircon_runtime --lib world_runtime_extension --locked -- --nocapture --test-threads=1`，必须覆盖 `world_runtime_extension_callback_can_publish_a_new_generation` 与 `world_runtime_extension_callbacks_overlap_across_independent_worlds`，证明 callback 在短锁快照外执行且不同 World 可并发。
- 验收证据：公共面结构测试通过；`native-plugin-boundary.md` 口径一致。

### M3 ABI 版本策略定稿

#### 切片 3.1 V1/V2 全量硬切（entry/descriptor 已完成，V2 byte DTO/alias 待删除）

- 目标文件：`plugin/native_plugin_loader/`（V1/V2 描述符/EntryReport/协商分支所在文件，执行时枚举：Grep `AbiV1|AbiV2|_V1|_V2`，path `zircon_runtime/src/plugin/native_plugin_loader`）；`zircon_plugins/native_dynamic_fixture/native/src/lib.rs`（夹具同步改造）。
- 改动形态：V1/V2 entry/descriptor、EntryReport、旧 symbol/version 与 loader fallback 已直接删除，夹具保持“当前 V3 descriptor 成功 + 未知版本被拒”。`NativePluginHostFunctionTableV3` 属于当前 descriptor/entry callback 合同，不是待删的 V3 host-API adapter；剩余切片必须把三个 V2 byte DTO 迁成唯一 V3 物理类型并删除 alias，同时删除 `NativeHostApiV3RegistrationScope`，只保留 V4 behavior callback 与 `ZrHostApiV4` registration owner。发现仓外旧插件依赖只形成升级阻断记录，不得恢复旧类型、alias、shim 或双版本协商。
- 调用方迁移：夹具 1 文件 + loader 内协商分支；枚举命令同上。
- 验收：`native_plugin_loader_rejects_unknown_abi_version_with_explicit_report`、`native_plugin_loader_accepts_current_v3_descriptor`（loader 测试树）。
- DoD：生产 Rust 中 V1/V2 entry/descriptor、V2 byte DTO 和 `NativeHostApiV3RegistrationScope` 旧 public symbols 0 命中（failure 文档历史证据除外）；V3 descriptor/entry + host-function-table、V4 behavior + runtime-interface host-API 矩阵文档化；插件 workspace 全量 check 通过。

#### 切片 3.2 热重载回滚失败注入测试（对照 Fyrox）

- 目标文件：`native_plugin_loader/native_plugin_live_host/lifecycle.rs` 的 runtime/editor 入口、`bridge_lifecycle.rs` 的 bridge 变体、`hot_update_application.rs` 的 delta/export-root 变体及其测试 owner。
- 改动形态：补失败注入测试——"快照→重载→恢复/回滚"路径的两个失败点：重载后符号缺失 → 回滚成功；状态恢复失败 → 回滚成功且 `NativePluginHotReloadState` 报告一致（对照 Fyrox dylib.rs 的状态序列化重载语义）。
- 调用方迁移：无。
- 验收：`hot_reload_missing_symbol_after_reload_rolls_back_to_previous_instance`、`hot_reload_state_restore_failure_rolls_back_and_reports`。
- DoD：`cargo test -p zircon_runtime --lib native_plugin --locked -- --nocapture` 含上述测试全绿。

#### M3 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib native_plugin --locked -- --nocapture`（hot_reload/abi 过滤词）
- `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked`
- `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked`（夹具改造回归）
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk native_panic_guard --locked -- --nocapture --test-threads=1` 与 Runtime `native_plugin_host_callback_panic_guard` focused gate，必须覆盖并发 callback、process-global hook sentinel 不变以及 panic status 映射。
- 验收证据：ABI 支持矩阵文档（落 `native-plugin-boundary.md`）；回滚测试；被删旧版无残留引用。
- 文档：`docs/zircon_plugins/first_party_runtime_catalog.md`、export 契约文档刷新。

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`06/2026-07-09-plugin-surface-and-lifecycle-output-records.md`](06/2026-07-09-plugin-surface-and-lifecycle-output-records.md)
- 2026-07-18 bridge诊断性能交接：framework bridge 4/4确认debug/editor构建中weak typed、native host与script bridge每次调用都会对interface entry执行共享AtomicU64 RMW。Runtime06联动Runtime07增加off/sampled/sharded模式与snapshot聚合，普通debug产品路径off时不得有per-call RMW，not-enabled边沿仍须可诊断；见PERF-MVP-330及`docs/plans/performance/01/2026-07-18-runtime-core-framework-bridge-root-static-review.md`。
- 2026-07-22 world runtime extension callback锁交接：`WorldDriver`当前在`runtime_extensions` mutex guard内执行全部type-erased callback，慢插件会串行阻塞并发World初始化且重入可能自锁。Runtime06联动Plugins01发布immutable `Arc` registration generation，短锁snapshot、锁外callback，并冻结reload/unload quiescence；见PERF-MVP-451与`06/failure-2026-07-22-world-runtime-extension-callback-lock.md`。
- 2026-07-22 native callback stable-owner public-surface 同步：`NativePluginCallbackDiagnostics`、`NativePluginLiveHostDiagnostics` 归入 behavior/report 诊断组，`NativePluginLoadProjection` 归入 loader/discovery 组，`ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH` 归入 ABI contract 组；仍只经 `zircon_runtime::plugin::native` 暴露。当前审计锚点为 `root_reexport_count = 0`、`native_namespace_reexport_count = 68`、native root re-export 0/0、native namespace re-export 68/68、M4 gate `classified-and-clear`、debt groups 0/0、native namespace symbol groups 5/5、unclassified native root symbols 0/0、unclassified native namespace symbols 0/0、root public native re-export locations 0/0、public native namespace re-export locations 1/1、app NativePlugin current call-site files: 7、native loader V1/V2 implementation files 0/0、`zircon_plugins` V1/V2 usage files 0/0、export_build_plan V1/V2 usage 0/0、unknown ABI rejection、hot reload failure injection、native loader test files 4/4、native test namespace import files 3/3、native test root import leaks 0/0、fallback lifecycle failure tests 4/4、`runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed`、`runtime_06_native_loader_tests_use_isolated_plugin_native_namespace`、`mirror_docs_guard_present = true`、`risks = []`、`runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts`。该记录只同步公开面分类与镜像，不提前关闭 Runtime 06 的 Cargo/native 验证线。
- 2026-07-22 native SDK callback panic guard性能交接：Plugin SDK原先每callback交换process-global panic hook并分配空hook；静态修复已改为直接`catch_unwind`并保留panic status。Runtime06负责并发callback、hook sentinel与native loader动态回归，Runtime10共同确认宿主侧guard一致；见PERF-MVP-491与`06/failure-2026-07-22-native-sdk-callback-global-panic-hook.md`。
- 2026-07-22 extension/system性能补充：PERF-MVP-532确认`SystemRegistration`与`RuntimeSceneSystemRegistration`把唯一FnMut存在`Arc<Mutex<S>>`，所有World实例每次run共享锁；Runtime06联动Plugins01/Runtime11改为generation-owned per-World factory/state并定义reload/unload quiescence。PERF-MVP-533同时要求owner→slots与compiled world extension plan同代发布，避免每World复制全registration/closure；扩充既有`world-runtime-extension-callback-lock` failure共同验收。
- 2026-07-22 catalog mutation/project plan补充：Runtime06联动Plugins01按PERF-MVP-537把discover/register/feature/reload变更装入一个candidate transaction，成功只发布一代、失败保持last-good；按538让World/host消费`Arc<CompiledProjectPluginPlan>`与同代frozen extension handles，禁止每请求重做manifest completion/dependency resolution/registry merge。reload/unload必须保留旧代quiescence，短commit不得在主线程执行全catalog build。
- 2026-07-22 native callback generation补充：原global loaded-table长锁修复保留；PERF-MVP-541要求per-plugin stable callback acquire/drop也不获取Mutex，reload/unload以epoch transition关闭新lease并等待/拒绝旧代，diagnostics off近零、on时sharded/sampled。PERF-MVP-543把typed plugin identity、parsed registration/binding/method table与callback owner同代发布，避免lookup格式化key和单slot clone整manifest；见Plugins01新callback failure。
- 2026-07-22 native host context补充：PERF-MVP-544的capability probe零分配止损已落地；Runtime06联动Plugins01按545把现有ArcSwap flat slot Vec升级为chunked generational slab，避免scope批量创建O(H²)目录clone，并让bridge call只pin一个context generation后dense method dispatch。stale/reuse/wrap/drop/in-flight与callback library quiescence合同不得放宽。
- 2026-07-22 export plan generation补充：PERF-MVP-546已让export借用cached builtin catalog并复用已补全manifest，删除一次catalog rows深clone与一次completion；Runtime06仍须按538发布catalog generation + canonical project fingerprint对应的`Arc<CompiledProjectPluginPlan>`，供runtime World、Plugins09 export与Editor12共同借用。不得把本轮单caller快路扩展成多个consumer私有cache；stable请求plan build/manifest completion必须为0。
- 2026-07-31 V4 public-surface inventory 前向修复：V3/V4 registration scopes 归入独立 host-API adapter owner，bridge-method owner 不再吸收跨域 host API；当前审计为 expected_source_file_count = 17、`native_namespace_reexport_count = 74`、native namespace re-export 74/74、native namespace symbol groups 6/6、unclassified native namespace symbols 0/0、app NativePlugin current call-site files: 8、`risks = []`。root scanner 的三类替代 re-export 负例与 focused inventory 3/3 已纳入；Rust mirror 仍保留旧快照，V3 host adapter/V2 DTO hard cut 与 managed Cargo 仍 pending。

## Code Review findings disposition (2026-07-31)

### 已修订的现状漂移

- [x] 将 ABI 矩阵拆成 V3 descriptor/entry + `NativePluginHostFunctionTableV3`、V4 behavior + `ZrHostApiV4`；V3 host-API registration scope 明确保持 open debt。
- [x] V3/V4 host API registration policy/scope 归入独立 host-API adapter owner，不再误归 bridge-method。
- [x] source inventory 17/17 进入风险聚合与 focused 回归，消除 17/expected14 仍 `risks=[]` 的 false-green。
- [x] root hard-cut scanner 覆盖 `native::{...}`、`self::native::*` 与 crate-qualified native re-export 替代语法。
- [ ] V2 byte-slice/buffer/callback-status 物理类型与 V3 alias 仍是实际旧 API 债。由于总目标明确“不再兼容旧架构”，本计划拒绝把它们改写成可长期保留的规范类型；必须等 Runtime source quiet window 在源码中硬切为唯一 V3 类型后才可勾选。
- [x] 将热重载核验从漂移行号改为 lifecycle/bridge/delta 三个稳定 owner 路径。
- [x] 将 App 调用面更新为 7 个生产文件 + `gui_startup.rs` 1 个测试文件，共 8/8。

### 设计收口状态

- [x] M2 源码目标已落地：`plugin` 根只保留描述/报告层，native 实现类型只经 `plugin::native` 暴露，未恢复 shim/re-export。
- [ ] M2 accepted closeout 仍需 `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` managed Cargo/native 验证；pending 验证不阻塞其他可落地实现。

### 已补执行锚、仍待验证

- [x] M2 测试阶段已加入 world-runtime-extension callback 重入与跨 World 并发 focused gate。
- [x] M3 测试阶段已加入 plugin SDK/native host panic guard、process-global hook sentinel 与并发 callback focused gate。
- [ ] 以上新增执行锚以及 Rust mirror 74/8 更新仍需 managed current-source 证据；在此之前 Runtime 06 保持 `in_progress`。
