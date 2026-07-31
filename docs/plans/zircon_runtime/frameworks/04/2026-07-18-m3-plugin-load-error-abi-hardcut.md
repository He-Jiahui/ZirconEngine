---
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/plugin_load_error.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
implementation_files:
  - zircon_runtime/src/plugin/native_plugin_loader/plugin_load_error.rs
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
  - zircon_runtime/src/plugin/native_plugin_loader/abi_declarations.rs
  - zircon_runtime/src/plugin/native.rs
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/src/dist.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor/descriptor_abi.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor/entry_abi.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor/string_helpers.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/behavior_bridge.rs
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader/real_fixture.rs
  - zircon_plugins/native_dynamic_fixture/native/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - docs/engine-architecture/native-plugin-boundary.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - rustc --edition=2021 --test <descriptor_abi_guard> && <guard.exe> --nocapture
  - rustc --edition=2021 --test <entry_abi_guard> && <guard.exe> --nocapture
  - rustc --edition=2021 --test <string_helpers_guard> && <guard.exe> --nocapture
  - rustc --edition=2021 --test <behavior_bridge_guard> && <guard.exe> --nocapture
  - cargo test -p zircon_runtime --lib --locked native_loader_rejects_library -- --test-threads=1
  - cargo test -p zircon_runtime --lib --locked --jobs 1 plugin_load_error -- --test-threads=1
---

# Frameworks04 M3 PluginLoadError ABI 硬切

Status: exact17_static_review_green_cargo_blocked
Date: 2026-07-18
Session: `frameworks04-m3-plugin-load-error-capability-r3-20260718`

## 完成项目

- 新增 `plugin_load_error.rs` 作为 descriptor/entry ABI 加载错误的唯一 owner；`PluginLoadStage` 显式区分 library-open、descriptor-probe、runtime-entry、editor-entry。
- `PluginLoadError` 使用 `thiserror`，每个变体都携带插件 id、阶段、artifact path、结构化 expected/actual 和 repair hint；symbol、payload、library-open 错误继续保留 typed source。
- `probe_native_plugin_descriptor(...)` 与 `call_native_plugin_entry(...)` 直接返回 `PluginLoadResult<T>`，删除 `NativePluginDescriptorAbiError`、`NativePluginEntryAbiError`、`call_native_plugin_entry_result` 和 ABI 内部 String 转换。
- descriptor export 缺失、null、ABI 不匹配或 descriptor plugin id 不匹配时，library 不再进入 `NativePluginLoadReport.loaded`；requested runtime/editor entry 名称或导出缺失时同样拒绝整个 library，不保留 partial-loaded 兼容状态。
- `load_discovered.rs` 的 missing artifact、library open、descriptor 和 entry ABI 失败统一从 `PluginLoadError` 格式化到当前报告展示边界；旧局部前缀拼接已删除。
- Runtime15 descriptor/entry/string-helper/behavior/bridge typed-error guard 已硬切到统一 Frameworks04 owner，历史 guard 名保留，旧错误类型和旧字符串边界不再是 current assertion。
- `native_dynamic_fixture` 提供 descriptor-export-missing 与 runtime-entry-export-missing 故障注入；real-fixture 回归锁定坏 descriptor、缺 descriptor 和缺 requested entry 均不得进入 `loaded`。
- `NativePluginEntryReportV3` 在 SDK 与 Runtime 两侧同步硬切为显式 `required_capabilities` / `denied_capabilities` 字段；所有 SDK 宏在编译期生成 NUL 结尾的换行分隔声明，不从诊断文本反解析，也不保留旧布局 alias/shim。
- Runtime 以宿主实际 granted 集计算 `required - granted` 与 `denied intersect granted`，任一结果非空即返回 `PluginLoadError::CapabilityNegotiation`，携带结构化 `missing_required` / `denied` 并在 `loaded.push(...)` 前拒绝整个 library。
- runtime/editor 的 missing-host report 已按各自 capability 声明拆分；`NativePluginEntryReportV3.diagnostics` 改为必需 C 字符串，null 指针统一进入 typed invalid-payload 路径。
- `native_dynamic_fixture` 新增 `required_capability_missing` 故障注入，real-fixture 回归锁定缺必需 capability 时 `loaded` 为空且诊断保留精确 `missing_required` / `denied` 明细。
- 独立复审后新增 entry-report `layout_epoch=4`：descriptor/host/behavior 继续使用 V3 协议；loader 在创建完整新布局引用前只从 raw pointer 读取公共首字段，旧 report 的 `abi_version=3` 会被 typed contract mismatch 拒绝，消除旧动态库越界误读风险；不提供旧布局 fallback。
- `PluginLoadError::CapabilityNegotiation` 现在保留已解析的 entry diagnostics 与 host callback diagnostics，缺能力拒绝不再丢失 SDK `missing_host_diagnostics`。

## 验证

- 初始 TDD RED：descriptor/entry standalone guard 因统一 owner 尚未落地而失败；初始 GREEN 为 `2/2`。
- review repair RED：descriptor、entry、string-helper `0/3`，分别暴露 missing-symbol actual、requested-entry 早退和旧包装断言；behavior/bridge 当前契约 `2/2` 已为 GREEN。
- review repair GREEN：descriptor `1/1`、entry `1/1`、string-helper `1/1`、behavior/bridge `2/2`，合计 `5/5`。
- capability hard-cut TDD RED：扩展 entry ABI standalone guard 首次因缺少 `required_capabilities` 字段失败；real-fixture source guard 首次因缺少 `required_capability_missing` feature/test 失败。
- capability hard-cut static GREEN：SDK/Runtime V3 镜像字段顺序一致；12/12 个实际 `NativePluginEntryReportV3` 构造均填充 required/denied；80 个实际宏 capability 列表全部为 compile-time literal，非 literal 为 0；entry standalone guard 与 real-fixture source guard 均为 `1/1`。
- r3 首轮独立复审为 `Critical 1 / Important 1 / Minor 0`：分别定位旧 V3 report 可越过版本门并误读新指针、capability rejection 丢失 report/callback diagnostics。review-repair guard 首次 RED `0/1`，生产修复后 GREEN `1/1`；exact17 独立复审待重跑。
- r3 第二轮独立复审为 `Critical 1 / Important 0 / Minor 0`：diagnostics finding 已关闭，但复审证明在方法内检查 epoch 之前创建 `&NativePluginEntryReportV3` 仍要求旧短对象满足新布局。第二次 review-repair guard RED `0/1`，随后把 epoch gate 前移为 raw first-field read，GREEN `1/1`；exact17 最终复审待重跑。
- r3 第三轮 exact17 最终独立复审为 `Critical 0 / Important 0 / Minor 0`：raw first-field read 严格早于完整新布局引用及新 capability 指针读取；12 个 report initializer、descriptor/host/behavior V3 边界、behavior validation V3、diagnostics 保真与无 alias/shim/fallback 均复核通过。
- r2 独立复审结果为 `Critical 0 / Important 0 / Minor 0`。真实动态库行为测试已编写但尚未由受管 Cargo 实际编译/运行；`docs/plans/zircon_tooling/session_coordinator/01/failure-2026-07-18-full-compile-input-snapshot-barrier-missing.md` 返回 fixed 前不启动 Cargo，也不把本记录提升为 accepted。

## 里程碑判定

本切片已完成 Frameworks04 M3 的 descriptor/entry ABI 垂直硬切与 native-loader capability `missing_required/denied` 明细。manifest discovery、distribution compatibility、load-report structured failure collection、`export_bootstrap` 以及 App/Editor 展示边界仍需后续切片；因此 Frameworks04 M3 与计划 04 均不得标记完成。
