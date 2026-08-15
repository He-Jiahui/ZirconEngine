---
handoff_kind: failure
status: open
created_at: 2026-07-30
summary_slug: native-discovery-compile-boundary
origin_plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/zircon_editor/editor/11
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/discover/authority.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discovery_refresh/contract.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover_load_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discovery_refresh/tests/admission.rs
tests:
  - bounded_load_manifest_rejects_an_entry_before_unbounded_vector_growth
  - bounded_root_admission_rejects_new_roots_without_evicting_a_snapshot
  - load_manifest_discovery_is_an_authority_owned_refresh_input
  - Editor11 dynamic_scene_session managed upward gate
---

# Plugins01: native discovery compile boundary

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/11-serialization-and-versioning.md`
- 来源执行切片：Editor11 canonical streaming 的 Runtime Dynamic Scene 上行门
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：两个编译错误均位于 native plugin discovery authority 与 bounded load-manifest parser，最低共享原因属于 Plugins01；Editor11 不应在 serialization owner 内增加绕过。

## 失败现象与复现证据

共享 Runtime15 受管 job `a4eb0c99d0af45ee8b8394c0a7517855` / run `5ca8e9f1d1654ad9bca00e148a81a3a1` 于 2026-07-30 自然终止，`exit101`、live PIDs `[]`，测试执行数为 0。编译只有两个 error：

- `discover/authority.rs:118` E0624：authority sibling 调用 `NativePluginDiscoveryRefreshInput::root_scan()`，但 constructor 仍是 `pub(super)`。
- `discover_load_manifest.rs:195` E0277：TOML 1.1 的 `Deserializer::new` 已返回 `Result`，调用点却把该 `Result` 直接传给 `DeserializeSeed::deserialize`。

原始 stderr 保存在 `.codex/state/session-coordinator/cargo-runs/a4eb0c99d0af45ee8b8394c0a7517855/5ca8e9f1d1654ad9bca00e148a81a3a1/stderr.log`。该失败会在任何 `zircon_runtime --lib` 门进入测试前复现，因此 Editor11 不能把现有 Dynamic Scene reservation 当作可接受运行。

最低两行修复后的 r2 job `4ebc76252c4b4655912a40983c2001f0` / run `efbb0b3a1e3e477aa389fca468199489` 已越过上述两个 error，随后自然 `exit101`、live PIDs `[]`；唯一 error 为 `discovery_refresh/tests/admission.rs:185` E0382，测试把 `collector: Arc<_>` 移入 `with_pool` 后仍需调用 `collector.release()`。该结果证明生产编译边界已恢复，同时把最低剩余问题收窄到同一 owner 的 test harness。

## 最低共享层根因

Discovery authority 已成为同一 `native_plugin_loader` owner 下的唯一生产构造者，但 `root_scan` 仍保留旧 child-module 可见性。与此同时 bounded seed parser 在 workspace 升级 TOML 1.1 后未硬切到 fallible `Deserializer::parse`，导致 parse error 与 seed deserialize 的 typed error 边界没有接合。

同一 refactor 的 test harness 未在把 collector 交给 service 时保留共享 `Arc`。自审还发现 load-manifest scratch helper 在 helper 内消费 admission token，未让 token 的词法作用域覆盖实际 TOML parse；累计峰值数值虽已写入 sink，owner 证明仍不完整。

## 架构修复验收

- `root_scan` 只允许 `crate::plugin::native_plugin_loader` 内部访问，不升级为 public API，不复制 constructor。
- Bounded load-manifest parser 使用 TOML 1.1 fallible parser，并保留原 `toml::de::Error`；容量拒绝回归必须通过。
- Admission test 必须通过 clone 保留控制 `Arc`；parse scratch token 必须由 caller 持有到 parser 返回，不能在 helper 内提前结束 owner 证明。
- Runtime lib 必须重新编译，原 Editor11 Dynamic Scene 上行门必须在同一 current-source 输入下通过。

## 禁止临时方案

- 禁止 public re-export、alias、compatibility shim、旧 TOML constructor fallback 或测试条件绕过。
- 禁止弱化 bounded parser 的 `DeserializeSeed` / candidate budget，或把 native plugin 错误吞进 Editor11 serialization 路径。

## 修复结果与回传

Open state: `implementation_complete / current-source static review complete / managed focused-upward, independent review, failure return, and atomic commit pending`; no pass is claimed.
