---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: preference-quota-error-kind-toolchain-drift
origin_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_plugins/01
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/platform/preferences/atomic_file.rs
tests:
  - cargo test -p zircon_runtime --lib platform::preferences::atomic_file::tests::platform_preference_storage_maps_host_io_error_categories --locked --jobs 1 --message-format short --color never -- --exact --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib native_callback_can_reenter_live_host_descriptor_without_deadlock --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib active_animation_tick_emits_immediate_then_paused_tick_resets_to_idle --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1
---

# Frameworks05：偏好存储 quota ErrorKind 与固定工具链漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 来源执行切片：Plugins01 native-callback current-source focused Windows 1.94.1 gate（job `acb2896d1cd24b61a84050784dc3f69e` / run `32a8189d1d854365b3f4aa8d9b0438c7`）。
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：Plugins01 的 current-source Windows 回归已越过 native callback 源码，但被 platform preference owner 新增的未跟踪实现阻断；错误位于跨平台 I/O 分类，不属于插件回调或 native loader 边界。

## 失败现象与复现证据

固定 Windows 工具链为 `rustc 1.94.1 (e408947bf 2026-03-25)`。以下两个独立受管编译均在同一两处失败：

- Plugins01 job `acb2896d1cd24b61a84050784dc3f69e` / run `32a8189d1d854365b3f4aa8d9b0438c7`，exit `101`；
- Runtime10 job `04edf97fb8d74fa988ef7639137314b3` / run `7cb8bb602497464486c2344d6ec5c697`，exit `101`。

错误均为：

- `zircon_runtime/src/platform/preferences/atomic_file.rs:109`：`std::io::ErrorKind::FilesystemQuotaExceeded` 在 1.94.1 中不存在；
- `zircon_runtime/src/platform/preferences/atomic_file.rs:142`：同一不存在变体被测试 fixture 再次引用。

Plugins01 run 还暴露了 feature projection 测试观测函数的最低层重导出可见性错误；该支撑错误由来源计划单独修复，不归入本节点。

## 最低共享层根因

Frameworks05 新增的 atomic-file backend 直接绑定了固定 stable 工具链未提供的 `ErrorKind` 变体，生产映射和测试又双录该名称。因此任何编译 `zircon_runtime` lib tests 的上层计划都会在目标测试执行前失败。

## 架构修复验收

- 仅使用仓库固定 stable 工具链公开支持的 I/O 分类，仍把存储容量/配额错误投影为 `PreferenceStorageErrorKind::CapacityExceeded`；不得把所有 I/O 错误一律降级为 transient。
- 生产映射与测试使用同一受支持的分类 authority，避免再次双录一个尚不可用的标准库变体。
- frontmatter 中 Frameworks05 focused test、Plugins01 current-source focused test均须在 Windows 1.94.1 上越过编译并执行目标断言。
- 修复必须保留 denied、capacity、corrupt backend、transient I/O 四类公开语义。

## 禁止临时方案

- 不得删除 quota/capacity 语义、吞掉写入错误或让测试跳过固定工具链。
- 不得在 Plugins01/native loader 中增加条件编译来绕开 platform preference 模块。
- 不得把未通过编译的旧产物或 validation copy 当作 current-source GREEN。

## 修复结果与回传

2026-07-22 current-source candidate：生产映射和 inline category test 已从不存在的 `io::ErrorKind::FilesystemQuotaExceeded` 硬切为 Rust 1.94 稳定的 `io::ErrorKind::QuotaExceeded`。`StorageFull`、`FileTooLarge` 与 quota 继续统一投影为 `CapacityExceeded`；未新增平台魔数、依赖、兼容分支或上层绕过。固定工具链 rustfmt、旧名称 absence scan、exact source diff-check 已通过；managed focused/upward Cargo 尚未取得 terminal pass，不得返回 fixed。

受管 focused retry job `282d191c8da14fb38a3edd5804464424` / run `2206a8fe67b9462489bbb403439baf2a` 自然终止并释放，exit `101`。该 run 在执行目标断言前被 current-source `zircon_runtime` lib-test 的三个外部编译错误阻断：`graphics/scene/scene_renderer/ui/text/tests/prepare_report.rs` 两处继续构造已移除的 `font_faces_changed` 字段，`tests/plugin_extensions/plugin_workspace_shape.rs` 一处在 move `runtime_id` 后再次借用。因此本 run 仅证明 Frameworks05 原有 `FilesystemQuotaExceeded` 编译错误已越过，不能作为 focused test red/green 或 fixed return 证据；外部 owner 修复后必须重新申请 source-bound focused gate。

Open state: `Frameworks05 owner source fix and fresh managed validation pending`。Plugins01 可继续修复自身 feature-projection 支撑错误，但在本节点回传前不能宣称 current-source focused test 通过。

## 2026-07-31 current-source owner recovery

- Successor session `frameworks05-preference-quota-error-kind-r1-20260731` owns the exact source,
  failure, return, and fixed-artifact scope. The previous Frameworks05 preference sessions are
  cancelled and no active foreign lease owns `atomic_file.rs`.
- Current production and inline test code use stable Rust 1.94.1 `io::ErrorKind::QuotaExceeded`;
  `StorageFull` and `FileTooLarge` remain in the same `CapacityExceeded` projection. The retired
  `FilesystemQuotaExceeded` spelling is absent, and denied, corrupt-backend, and transient-I/O
  mappings remain explicit.
- The Frameworks05 preference boundary suite is 7/7 GREEN on current source. The exact preference
  Rust files pass scoped Rust 1.94.1 rustfmt, and scoped `git diff --check` is clean apart from line
  ending warnings. These are static/current-source checks only.
- Independent review of exact4 snapshot 1371 is C0/I0/M0. Both future artifacts remain exact
  tombstones; the source/failure hashes match and the ordinal tombstone-aware fingerprint is
  `39af02b313269935aa27e4b0567ac971406c38301787cae6167dad02b4c905a1`.
  A direct no-output Rust 1.94.1 probe also confirms every mapped `ErrorKind` variant is supported.
- Reviewed exact4 snapshot 1372 binds the final source bytes and review record. Full Cargo-closure
  validation-copy job `9097e7dfea204e298a0d5c476c43e01f` was durably accepted for the canonical
  focused Rust command; its worker is materializing a frozen compile-input copy asynchronously.
- State is `fix_implemented_review_green_validation_receipt_accepted`. Pending materialization or
  validation delays only accepted closeout. The focused terminal result, Plugins01 upward gate,
  commit, and fixed return are still required; this node remains open and does not claim acceptance.
