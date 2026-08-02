---
handoff_kind: failure
status: open
created_at: 2026-07-31
summary_slug: authoring-world-test-concrete-level-manager
origin_plan: docs/plans/zircon_plugins/04-animation.md
fixing_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
origin_child_dir: docs/plans/zircon_plugins/04
fixing_child_dir: docs/plans/zircon_editor/editor/01
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/editing/authoring_world.rs
  - zircon_editor/src/tests/editing/authoring_world.rs
  - zircon_editor/src/tests/editing/mod.rs
tests:
  - python -m unittest tools.tests.test_frameworks_05_layer_direction.Frameworks05LayerDirectionTests.test_asset_manager_consumers_use_versioned_handles_at_use_points -v
  - python -m unittest tools.tests.test_frameworks_05_layer_direction -v
  - cargo +1.94.1 test -p zircon_editor --lib tests::editing::authoring_world::authoring_facade_replaces_and_clears_the_stable_gateway --locked --jobs 1 -- --exact --nocapture --test-threads=1
---

# Editor01: authoring-world test owns a concrete LevelManager in production source

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/04-animation.md`
- 来源执行切片：Plugins04 animation sequence caller hard-cut upward Frameworks05 layer-direction gate
- 修复责任计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 交接原因：具体 `DefaultLevelManager` 测试构造位于 Editor01 production module，最低修复 owner 是 Editor01 的 test placement boundary。

## 失败现象与复现证据

The current-source Frameworks05 layer-direction suite ran 28 tests. Its animation sequence owner
test passed, but the suite ended 27/28 because
`zircon_editor/src/core/editing/authoring_world.rs` imported and constructed
`DefaultLevelManager` inside an inline `cfg(test)` module. The guard intentionally scans production
module files as a whole and rejects cross-domain concrete manager consumers.

## 最低共享层根因

The production authoring-world facade already depends only on `LevelSystem` plus the editor runtime
gateway. Its behavior regression was placed in the production module and introduced the concrete
manager token there. This is an Editor01 test-owner placement defect, not a runtime scene contract or
Plugins04 animation failure.

## 架构修复验收

- Keep the production authoring-world owner free of `DefaultLevelManager` and other concrete manager
  construction.
- Preserve the same replace, clear, stable gateway, and detached-access behavior in the canonical
  `zircon_editor/src/tests/editing` tree.
- Keep the Frameworks05 concrete-manager guard unchanged and rerun its focused and complete suite.
- Run the focused Editor Rust behavior gate through the managed coordinator before fixed return.

## 禁止临时方案

- Do not weaken the Frameworks05 scan, strip `cfg(test)` text in the guard, or add an allowlist.
- Do not add a concrete manager alias, helper, facade, or compatibility constructor to production
  Editor code.
- Do not delete the behavior regression or treat the 27/28 run as a Plugins04 source failure.

## 修复结果与回传

Open state: `待修复`; `fix_implemented_static_green_managed_validation_pending`: the behavior test has moved to the existing Editor
test tree without changing production gateway behavior. The focused consumer guard is 1/1 GREEN and
the complete Frameworks05 layer-direction suite is 28/28 GREEN on current source; Rust 1.94.1 scoped
rustfmt and diff-check also pass. Independent review of snapshot 1370 is C0/I0/M0; all exact4 hashes
match and the ordinal fingerprint is
`c0960fd6af95e70f933da20848bb958387a567b286a5b6bd92785ffcde8e7fc5`. Managed Editor Rust evidence,
commit, and fixed return remain pending.

The managed command uses the test's full post-move module path. A bare function filter combined
with `--exact` would select zero tests and is not acceptance evidence.

Full Cargo-closure validation-copy job `fcd25d7748244c54b28704fefab73b2f` was durably accepted
for that canonical full-path command. Its worker is materializing a frozen compile-input copy
asynchronously; this receipt delays only accepted closeout and is not a Rust GREEN claim.
