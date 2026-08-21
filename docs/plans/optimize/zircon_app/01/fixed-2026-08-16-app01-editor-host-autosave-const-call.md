---
handoff_kind: fixed
status: fixed
created_at: 2026-08-16
summary_slug: app01-editor-host-autosave-const-call
origin_plan: docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
fixing_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
origin_child_dir: docs/plans/optimize/zircon_app/01
fixing_child_dir: docs/plans/zircon_editor/editor/14
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/recovery/autosave.rs
  - zircon_editor/src/core/recovery/autosave_adapter.rs
tests:
  - cargo build -p zircon_app --no-default-features --features target-editor-host --locked --bin zircon_editor
resolved_at: 2026-08-16
---


# Editor14: App01 Editor-host validation is blocked by an autosave const call

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md`
- 来源执行切片：App01 M2/P1-12 host foreign-output budget 的 Editor-host compatibility gate
- 修复责任计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 交接原因：最低失败层是 Editor14 所有的 autosave scheduler/adapter API，不属于 App01 动态运行时宿主边界。

## 失败现象与复现证据

在 Windows 受管 Cargo lane 执行：

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 `
  -Package zircon_app `
  -NoDefaultFeatures `
  -Features target-editor-host `
  -Bin zircon_editor `
  -SkipTest
```

`zircon_runtime` 和 App01 前置依赖完成编译后，`zircon_editor` 以 E0015 失败：

```text
zircon_editor/src/core/recovery/autosave_adapter.rs:178:24
cannot call non-const method `AutosaveScheduler::is_due` in constant functions
```

因此 App01 的 `target-editor-host` 产品构建未完成，后续同配置 `foreign_output` 测试也未执行；`target-client` 下 foreign-output `17/17` 与相邻 operation/profile 协议合同 `2/2` 已通过，本记录不把 Editor-host gate 标记为通过。

## 最低共享层根因

`AutosaveJobAdapter::is_due` 在提交 `7a20f921` 中成为 `pub(crate) const fn`，其函数体调用 `AutosaveScheduler::is_due(now)`；后者不是 const。该合同在任何启用 `zircon_editor` 的构建中均不成立，与 App01 foreign-output 实现无关。

## 架构修复验收

- 在 autosave scheduler/adapter 权威边界统一 `is_due` 的 const 语义；若无编译期调用需求，应移除不真实的 const 承诺，而不是增加条件编译或调用点绕过。
- Editor14 autosave focused tests通过，且 `zircon_editor` library 可在当前 Windows stable toolchain 编译。
- 原始 `target-editor-host` App01 `foreign_output` 受管测试执行并通过。

## 禁止临时方案

- 不得增加别名、兼容 shim、静默 fallback、重复状态源、test-only bypass 或 App01 调用点特例。
- 不得禁用 Editor-host 测试或降低计划验收标准来隐藏 E0015。

## 修复结果与回传

- 根因：AutosaveJobAdapter::is_due was declared const even though it is a runtime clock-and-state predicate delegating to the intentionally non-const AutosaveScheduler::is_due.
- 架构修复：Removed the false const API contract from the adapter and retained AutosaveScheduler as the single runtime authority; no scheduling state or behavior changed.
- 验证：validate-matrix -Package zircon_editor -SkipTest: exit 0; App target-editor-host build compiled past zircon_editor and autosave, then failed in foreign zircon_plugin_hybrid_gi_runtime with 42 independent errors.
- 回传：Editor14 autosave const-call blocker is fixed and returned to App01; the original product build now advances to a separate Hybrid GI owner failure.
