---
handoff_kind: fixed
status: fixed
created_at: 2026-08-29
resolved_at: 2026-08-29
summary_slug: zr-contracts-random-seed-receipt-const-compile
origin_plan: docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
fixing_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
origin_child_dir: docs/plans/optimize/zircon_runtime/22
fixing_child_dir: docs/plans/zircon_runtime/frameworks/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_contracts/src/random/service_state.rs
  - zircon_runtime/crates/zr_contracts/src/random/tests/state.rs
tests:
  - rustc +1.94.1 const Option equality RED probe
  - rustc +1.94.1 zircon_runtime/crates/zr_contracts/src/lib.rs rlib compile
  - python -B -m unittest tools.tests.test_frameworks_01_random_contract_kernel_boundary -v
---

# Frameworks01: `RandomSeedReceipt` const constructor does not compile on Rust 1.94.1

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md`
- 来源执行切片：Runtime22 random checkpoint atomicity TDD RED
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 交接原因：失败位于 Frameworks01 物理 owner `zr_contracts::random` 的公共 const constructor，低于
  Runtime22 registry/checkpoint 执行层。

## 失败现象与复现证据

Runtime22 在批准的 Windows isolated rustc boundary 中以 Rust 1.94.1 编译当前
`zr_contracts/src/lib.rs`，唯一错误为 `service_state.rs:80` 的 E0658：
`previous_generation.checked_add(1) != Some(generation)` 在 `pub const fn try_new` 内调用了尚未
const-stable 的 `Option<u64>::PartialEq`。

Frameworks01 使用同一 rustc 1.94.1 和 D 盘 `TEMP`/`TMP`/metadata 输出运行最小 probe，稳定得到两条
E0658：conditionally-const operator 不可调用，且 `PartialEq` 尚不是 stable const trait。该 probe 只包含
`checked_add` 与 `Option` equality，排除了 serde、thiserror、Runtime22 checkpoint 和 registry 的影响。

## 最低共享层根因

receipt 的 successor-generation 语义正确，但其 const 实现形态不兼容项目锁定编译器。最低修复是在
`RandomSeedReceipt::try_new` 内对 `checked_add` 结果做 pattern match，再以 const-stable 的原始 `u64`
比较验证 generation。public const API、typed error、manual `Deserialize -> try_new` 和 overflow fail-closed
语义都必须保留。

## 架构修复验收

- Rust 1.94.1 必须编译保留 `pub const fn try_new` 的真实 `zr_contracts` rlib。
- generation jump 与 `u64::MAX` overflow 必须继续返回 typed
  `RandomSeedReceiptError::NonSuccessorGeneration`，合法单 successor 继续构造 receipt。
- manual Deserialize 必须继续通过同一 `try_new`，不能增加 unchecked wire constructor。
- Runtime22 在新 contracts hash 上恢复其原 checkpoint RED/GREEN；本 Failure 不替代 checkpoint 验收。

## 禁止临时方案

- 不得移除 `const` 以隐藏编译错误，也不得新增非 const overload、alias、compatibility shim 或 call-site
  特判。
- 不得恢复 unchecked `new`、derived `Deserialize` 或绕过 typed error。
- 不得由 Runtime22 复制 receipt invariant，或在 checkpoint 层吞掉 contracts 编译失败。

## 修复结果与回传

- 根因：RandomSeedReceipt::try_new used non-const-stable Option PartialEq inside a public const fn on Rust 1.94.1.
- 架构修复：Preserved the const API and matched checked_add before comparing primitive u64 values; typed error and manual Deserialize remain on the same invariant.
- 验证：Rust 1.94.1 real zr_contracts rlib compile exit 0; const and serde harness exit 0; rustfmt and diff checks green; Random owner guard 13/14 with only the existing foreign root-manifest RED.
- 回传：Frameworks01 contracts compile blocker fixed at service_state hash c10422917a2d8d1896c428c75173d6a7e9821adac2fdadb2af2f75aa93203f21; Runtime22 may resume checkpoint validation.
