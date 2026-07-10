---
related_code:
  - zircon_runtime/src/core/framework/error.rs
  - zircon_runtime/src/plugin/extension_registry_error.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/access.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/diagnostics.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin/plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin/feature.rs
  - zircon_runtime/src/plugin/runtime_plugin/capability_view.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_report/runtime.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report/runtime_merge.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/constructors.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/order.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/plugin.rs
  - zircon_runtime/src/core/runtime/descriptors/registry_name.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/first_party_runtime_catalog/src/tests/provider_snapshot.rs
  - tools/plugin_structure_audits/registration.py
  - tools/audit_plugin_structure.py
implementation_files:
  - zircon_runtime/src/core/framework/error.rs
  - zircon_runtime/src/plugin/extension_registry_error.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/access.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/diagnostics.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin/plugin.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin/feature.rs
  - zircon_runtime/src/plugin/runtime_plugin/capability_view.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/extension_report/runtime.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report/runtime_merge.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/constructors.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/order.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/plugin.rs
  - zircon_runtime/src/core/runtime/descriptors/registry_name.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - tools/plugin_structure_audits/registration.py
  - tools/audit_plugin_structure.py
plan_sources:
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-07-10 frameworks 基础架构新版硬切换目标
tests:
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_descriptor.rs
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_lifecycle.rs
  - zircon_runtime/tests/frameworks02_runtime_plugin_lifecycle.rs
  - zircon_runtime/src/core/runtime/tests/registry_name.rs
  - zircon_plugins/first_party_runtime_catalog/src/tests/provider_snapshot.rs
  - tools/tests/test_plugin_structure_audit_registration.py
doc_type: acceptance-evidence
status: validation_in_progress
---

# Frameworks 02 RuntimePlugin 模块排序硬切换验收证据

## 范围

本记录覆盖 Frameworks 02 M3 的 RuntimePlugin 排序硬切换、共享 kernel lifecycle 并轨和 embedded descriptor 单源切片，不声明整个 M3 或 Frameworks 02 已完成。feature-enabled first-party provider 顺序/模块身份快照已通过；插件工作区全量门已越过 shared Runtime/Editor 编译层，但被并发 Animation owner 的当前类型不匹配阻断。`native_dynamic_fixture` 的 ABI v3 实产物加载与行为边界已有此前 M3 记录，本切片不重复声明该成果。

## 基线问题

旧实现遇到插件模块重复、缺失依赖、跨 InitLevel 反向依赖或依赖环时，会记录诊断后退化为按插件名排序并继续注册或执行生命周期。该路径掩盖了错误依赖图，与新版“依赖图非法即拒绝启动/执行”的内核语义冲突。

## 硬切换结果

- 删除 `fallback_order_plugins`、`fallback_order_descriptors` 及对应字符串型旁路状态。
- descriptor 与 registration-report 扩展合并共用 `sort_module_activation_order` 结果。
- SDK/first-party/native 生成的 `RuntimePluginRegistrationReport` 在扩展合并前也从 package runtime-module 行重建 `ModuleDescriptor` 并进入同一排序器；真实 app/bootstrap 消费的 report 路径不再保留输入顺序旁路。
- `RuntimePlugin` 只保留 descriptor/manifest/selection、extension `register(...)` 和内嵌 `ModuleLifecycle` 访问；plugin-only ready/finish/activate/deactivate hooks、三套 context 和 catalog lifecycle dispatcher 已硬删除。
- `RuntimePluginRegistrationReport::from_plugin(...)` 自动且只注册内嵌 `ModuleDescriptor`；provider `register(...)` 仅注册 module-owned extensions。28 个首方 runtime plugins 全部以 `.with_module_descriptor(...)` 显式绑定模块，生产 `plugin.rs` 已清零重复 `register_module(...)`。
- SDK `RuntimePluginRegistrationBuilder::module(...)` 已硬切为只接收 module owner name；不再接受第二份 descriptor。particles/physics 的共享 manager 在 plugin 构造时建立，并由内嵌 descriptor 与 extension/interface 注册共同持有。
- 首方 embedded module identity 已硬切到 manifest 规范 `<package>.runtime`，旧 PascalCase `*Module` / `*PluginModule` 身份不保留 alias。因为 service registry 需要消费同一 module identity，`RegistryName` 改为从右侧解析 `.Driver|Manager|Plugin.<service>`，允许 namespaced module owner，同时继续拒绝空段、空白和带点 service 名。
- SDK 新增 `RuntimePluginDeclaration::with_module_descriptor(...)`，first-party 继续使用同一 descriptor builder；native ABI v3 从 manifest 投影 init-level/dependencies，并使用 no-op Rust lifecycle。
- 非法图保留为 `CoreError` 或 report fatal diagnostic；非法图发生后不合并任何扩展，不保留旧行为兼容分支。

## 回归断言

- descriptor catalog：非法依赖图不产生注册报告，并公开可检查的 typed module-order error。
- trait surface：plugin-only ready/finish/activate/deactivate 和 catalog dispatcher 不再存在。
- kernel lifecycle：插件内嵌 descriptor 的 build/ready/finish/cleanup 由 `CoreRuntime` 执行一次，deactivate 走反向 cleanup。
- registration-report 扩展合并：合法依赖图按统一顺序合并；非法图在首个扩展合并前产生 fatal diagnostic，返回空 registry。
- project-filtered 启动链：只对当前 target 真正 enabled 的 provider 建图；合法图按依赖顺序合并，缺少/禁用依赖或环在 app/bootstrap 使用任何扩展前 fatal。

## 验证记录

已通过的静态门：

- `rustfmt --edition 2021 --check`：本切片 Rust 文件通过。
- scoped `git diff --check`：无空白错误，仅有工作区 LF/CRLF 提示。
- 旧路径搜索：`fallback_order_plugins`、`fallback_order_descriptors`、`order_diagnostics` 无命中。
- 退役生命周期搜索：`PluginFinishContext`、`PluginReadyContext`、`PluginRuntimeContext`、`from_lifecycle_plugins`、`activate_lifecycle_plugins`、`deactivate_lifecycle_plugins` 在 Rust 源码无命中。
- 首方单一来源搜索：28 个 trait-backed runtime plugins 均恰好绑定一个 `.with_module_descriptor(...)`；首方生产声明 owner 无 `register_module(...)`；SDK module-owner builder 无 descriptor 参数。
- 插件结构审计：正式审计新增 Frameworks 02 单源 gate，只扫描 `src/plugin.rs` 与可选 `src/runtime_plugin/` 声明 owner，避免遍历整个子系统实现树；`python -m unittest tools.tests.test_plugin_structure_audit_registration` 通过 10/10，Windows 全量 JSON audit 返回 `runtime_plugin_descriptor_root_count=28`、`runtime_plugin_descriptor_single_source_violation_count=0` 与 `runtime-plugin-descriptor-single-source-clean`。
- WSL/Python 3.10 registration audit：`tomllib` 不可用时使用环境已有 `tomli`；同一 registration audit 通过，28 roots、0 violations，耗时 28.68s。
- 插件分发预检：`python -m tools.zircon_export plugin validate --all --repo-root . --json` 通过 39/39，`failed_count=0`、diagnostics 为空。

待完成的执行门：

- focused `invalid_runtime_plugin_module_order` 单元测试。
- `zircon_runtime` 包级测试门。
- WSL/Linux focused 与包级验证。

首次 WSL/Linux focused 尝试：

- 工具链：Ubuntu 22.04，`rustc 1.94.1`，`cargo 1.94.1`。
- 命令：`CARGO_TARGET_DIR=/tmp/zircon-frameworks02-m3-order-target cargo test -p zircon_runtime --lib invalid_runtime_plugin_module_order --no-default-features --features core-min --locked --jobs 2 -- --test-threads=1 --nocapture`。
- 结果：lib test target 在执行本切片测试前被并发工作区中未纳入本切片的 `zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt/tests/mod.rs:45` 编译错误阻断（`E0505`，`original` 已被 `Face` 借用后又被移动）。该文件与其目录均为未跟踪/外部活动改动，本切片未修改。
- 诊断层次：Cargo/manifest 与本切片类型定义已进入 `zircon_runtime` lib test 编译；最低已确认失败层是无关 font fixture 测试源码，focused 插件测试尚未运行。为保护并发所有权，不在本切片越权修复该文件。

第二次 WSL/Linux focused 尝试（font fixture 外部修复后）：

- 命令：`CARGO_TARGET_DIR=/tmp/zircon-frameworks02-m3-order-target cargo test -p zircon_runtime --lib registration_report_catalog_ --no-default-features --features core-min --locked --jobs 2 --message-format short --color never -- --test-threads=1 --nocapture`。
- 结果：再次在执行本切片测试前被另一项未跟踪活动源码阻断：`zircon_runtime/src/core/framework/animation/animation_target_id.rs:36` 的 `E0502`（对 `bytes` 的可变与不可变借用重叠）。
- 支持层判断：本切片新增 report sorter/extension gate 已通过 rustfmt 和生产 lib check；当前最低失败层仍是无关 lib-test 源码编译，不是插件排序断言或扩展合并行为。该活动文件未由本切片修改。

WSL/Linux 非测试编译门：

- 命令：`CARGO_TARGET_DIR=/tmp/zircon-frameworks02-m3-order-target cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 2 --message-format short --color never`。
- 结果：通过，`Finished dev profile`，耗时 10m49s；输出含 420 条当前工作区既有 warning。
- 结论：本切片生产代码和 typed error 路径在 Linux/core-min 下可编译；该结果不能替代被 font fixture 阻断的 lib-test 执行。

生命周期硬切换后的 WSL/Linux 编译门：

- 同一 `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked` 形态在删除 plugin-only lifecycle 后再次通过，耗时 11m02s，421 条当前工作区既有 warning。
- focused `runtime_plugin_lifecycle` lib-test 再次在执行断言前被无关活动源码阻断：`asset/project/manager/scan_and_import.rs` 引用尚未导出的 `stage_environment_ibl_source`；同一轮还暴露并修复了本切片测试中 `WeakBridge` 的显式类型标注问题。前者不属于本切片所有权。

首方单一 descriptor 来源硬切换后的 WSL/Linux 尝试：

- `zircon_runtime` lib-test no-run 已进入完整 lib-test 编译，但 `rustc 1.94.1` 在 early lint diagnostic rendering 中 ICE（`StyledBuffer::replace` slice index panic），未进入断言执行。
- plugin SDK locked check 已进入共享 `zircon_runtime` 编译，两次分别在 `dynamic_api::runtime_loop::present_extract` borrow-check 与增量关闭后的同一共享库分析阶段 ICE；没有产出 SDK 类型通过结论。
- core-min lib check 复跑在无关 `ui::text::layout_engine::visual_order::logical_text_clusters` typeck 中 ICE。三次均为编译器崩溃，不作为 pass，也不越权修改这些并发 owner。

WSL nightly 交叉验证：

- 安装 `rustc 1.99.0-nightly (2026-07-08)` 规避 stable 1.94.1 ICE；`cargo +nightly check -p zircon_runtime --lib --no-default-features --features core-min --locked` 通过（5m42s，既有 warnings）。
- plugin SDK 首次 `--offline` 检查刷新了缺少依赖项的 `zircon_plugins/Cargo.lock` 并通过；随后 `cargo +nightly check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked` 通过（6m46s，既有 warnings）。
- 新增独立 integration target；Cargo 首次编译并执行暴露 fixture crate-name validation，修复后 Cargo 重编译超时但无残留进程。使用同一 freshly built nightly `libzircon_runtime.rlib` 直接编译/执行该 integration target，1/1 通过，确认 report 唯一注册 embedded descriptor 且 `CoreRuntime` 执行 build/ready/finish/cleanup。
- feature-enabled first-party catalog 首轮执行暴露 `AiModule` 与 manifest `ai.runtime` 不一致；据此完成全部首方 module identity 硬切与 namespaced `RegistryName` 支持。修复后的全功能命令完成 11m41s 编译并执行 8 tests；`feature_enabled_first_party_provider_snapshot_reports_compiled_runtime_plugins` 通过，证明 13 个已链接 provider 顺序、diagnostics、manifest runtime module identity 与自动注册 descriptor 一致。其余 4 个旧结构审计测试首次仅因 WSL Python 3.10 缺少标准库 `tomllib` 而无法启动；新增 `tomli` fallback 后，WSL registration audit 已通过。直接执行同一 nightly 测试产物先取得 provider snapshot 1/1、exit 0；随后完整执行全部目录测试为 8/8 passed、0 failed、221.63s、exit 0，覆盖 manifest schema、capability single source、crate skeleton、dist boundary、descriptor/manifest parity 和 provider snapshot。
- exact-source `RegistryName` standalone harness 直接 `include!` 当前实现，验证 `weather.runtime.Manager.ClockManager` / `from_parts("weather.runtime", ...)` 解析成功，同时拒绝空 namespace segment 与 dotted service；compile/run 通过。

首方目录门通过后的 runtime 包级尝试：

- 命令：WSL nightly `cargo +nightly test -p zircon_runtime --lib registration_report_catalog_ --locked --jobs 1 --target-dir /home/hejiahui/zircon-targets/frameworks-first-party-catalog-featured-0710 -- --nocapture --test-threads=1`。
- 结果：lib-test target 在执行 focused tests 前，被 `graphics/scene/scene_renderer/post_process/resources/construct/construct/construct.rs:20` 的 `E0425` 阻断；该活动文件调用尚未由同一 render/HZB owner 提供的 `bind_group_layouts::hzb_msaa(...)`。
- HZB owner 随后补齐定义与 re-export；第三次同命令确认该错误消失，编译继续推进。新的最低失败层是活跃 text 硬切：runtime 已读取 `UiResolvedStyle.language`，而 `zircon_runtime_interface::ui::surface::UiResolvedStyle` 尚未包含该字段；同时多个 `ScreenSpaceUiTextBatch` 测试夹具还缺 `language`，合计 12 个 `E0609`/`E0560`/`E0063`。
- 边界判断：前后两次失败都位于明确的并发 WGPU/text owner，且相关文件已有非本会话脏改动；本切片未补临时 stub、未回退 HZB-MSAA 或 text-language API。owner 完成 interface/fixture 同仓硬切后，同一 nightly target 成功产出当前 `zircon_runtime` lib-test binary；Cargo 父进程在工具调用超时后无子进程等待，已仅终止该无工作父进程并保留测试产物。
- 直接执行当前 lib-test binary 的 Frameworks 02 focused groups：kernel module order 6/6、kernel module lifecycle/ready/finish/cleanup 6/6、builtin descriptor activation order 1/1、namespaced `RegistryName` 7/7、RuntimePlugin descriptor 12/12、RuntimePlugin lifecycle/report/native/capability/order 8/8；合计 40/40 passed、0 failed。完整 7399-test runtime lib suite 与 plugin workspace 全量 build/test 仍 pending。

插件工作区 locked gate：

- 标准 metadata 已补齐当前插件 lock；随后 `cargo +nightly metadata --manifest-path zircon_plugins/Cargo.toml --format-version 1 --locked --offline --no-deps` 通过，证明 lock 与 manifest 当前一致。
- `cargo +nightly build --manifest-path zircon_plugins/Cargo.toml --workspace --locked --jobs 1 --target-dir /home/hejiahui/zircon-targets/frameworks-first-party-catalog-featured-0710` 成功编译当前 `zircon_runtime`、`zircon_runtime_interface`、`zircon_editor` 及大部分 plugin runtime/dist crates。
- 首轮构建终止于 `animation/runtime/src/scene_hook/tick.rs:71` 的 E0308；并发 Animation owner 随后完成该借用硬切。原样重跑再次越过 shared Runtime/Editor，最终终止于 `animation/runtime/src/evaluation/pose_pool.rs` 对私有 `PoseBuffer::{with_capacity,joint_capacity}` 的 3 个 E0624。相关文件属于并发 Animation owner，本切片未增加兼容可见性或越权覆盖其活动改动。
- 阻断后重跑 Frameworks 02 持久化结构门：Python audit tests 10/10；全量 plugin audit 为 28 descriptor roots、0 single-source violations、0 registration compatibility shim sites。

完整 runtime lib-suite 探索：

- 直接执行同一当前 nightly lib-test binary 的 7399-test inventory，单线程运行 3604 秒后仍在执行大量全仓文件/计划镜像扫描；当时已有 4248 passed、587 个唯一 failed，其余未完成，已主动终止并保留 `/tmp/frameworks02-runtime-full-7399.log`。
- 失败类别包括 WSL 无 GPU adapter 的 project-render tests、活动 physics/animation wiring 文本守卫、Frameworks/Runtime 15/Render 计划状态镜像缺口，以及其他并发结构测试；因此不能把该探索记录为 full-suite pass。
- Frameworks 02 strict module graph / RuntimePlugin focused groups 仍为 40/40；当前全库最低剩余层是跨计划状态与活动 owner，后续只修复 Frameworks 02 所有权内失败，不通过恢复旧兼容路径规避。

## 当前判定

排序硬切换、shared kernel lifecycle 并轨、embedded descriptor 单源、持久化结构 gate、feature-enabled first-party catalog 8/8、runtime focused lib tests 40/40 及 WSL/Linux `zircon_runtime --lib core-min` 编译门已完成。独立 lifecycle integration target 和 provider snapshot 均有新鲜 1/1 成功证据；plugin workspace locked build 已两次越过 shared Runtime/Editor 编译层，当前被并发 Animation owner 的 PoseBuffer 可见性 E0624 阻断。完整 runtime lib suite 的 3604 秒探索取得 4248 pass/587 fail 后终止，明确暴露跨计划状态和并发 owner 剩余债务。完整 runtime lib suite 与插件工作区全量门仍未通过，因此本记录保持 `validation_in_progress`，不得把整个 M3 或 Frameworks 02 标记为 complete。
