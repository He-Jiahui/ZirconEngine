---
handoff_kind: fixed
status: fixed
created_at: 2026-07-18
summary_slug: sound-automation-nonfinite-preflight
origin_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
fixing_plan: docs/plans/zircon_plugins/02-sound.md
origin_child_dir: docs/plans/zircon_runtime/render/01
fixing_child_dir: docs/plans/zircon_plugins/02
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/sound/runtime/src/automation/target/apply.rs
  - zircon_plugins/sound/runtime/src/automation/values.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation/non_finite_values.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation/unsupported_parameter/track_delay.rs
tests:
  - cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_runtime --lib --locked --jobs 1 tests::automation_binding::validation::non_finite_values:: -- --nocapture --test-threads=1
resolved_at: 2026-07-18
---


# Plugins02：Sound automation 非有限值缺少统一前置校验

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 来源执行切片：Render01 F2 basic-scene product acceptance 的 Sound M1 Kira hard-cutover 上行门
- 修复责任计划：`docs/plans/zircon_plugins/02-sound.md`
- 交接原因：自动化目标值的合法性与副作用顺序由 Sound runtime 统一入口拥有；Render01
  不拥有或修改 Sound/Kira 路径。

## 失败现象与复现证据

受管 tests-only RED reservation `d58189a1aa7f426da5933d8b034a1e1e` 执行为 job
`770baa08b4f24797b1ca0db228e4b2c9` / run `b7448e7c3d0f41ea92b012ed467a00b8`，
Rust 1.94.1 下 `3 executed / 0 passed / 3 failed / 339 filtered`。Track mute 与 Volume
priority 接受 `NaN` 并返回成功；Effect chorus voices 在校验输入前先查找 effect，返回
`UnknownEffect`，没有返回 typed `SoundError::InvalidParameter`。

## 最低共享层根因

`apply_automation_target` 只在 `SynthParameter` 分支调用 `ensure_finite_value`。Track、Effect、
Source、Listener 与 Volume 分支会先克隆/查找资源并进入各自参数转换，导致非有限值的诊断类型、
顺序和副作用边界不一致。有限值约束属于所有 Sound automation target 的共享输入契约，应在
Kira active-state gate 后、任何目标查找或图/描述符复制前执行一次。

## 架构修复验收

- 统一入口在任何 target 分派前拒绝非有限值，并返回 typed `SoundError::InvalidParameter`。
- Track、Volume、Effect 三条受管 focused 回归全部通过，Effect 必须证明校验早于资源查找。
- 既有 delay `NaN` 行为同步为统一 `finite` 诊断，Sound route、plugin broad 与 Render01 F2
  上行门必须在最终 immutable Sound SHA 后重新验收。

## 禁止临时方案

- 不得在各 target 分支复制有限值校验，不得增加 silent clamp、兼容 fallback 或 call-site 特例。
- 不得删除/忽略 NaN 测试，不得让 Render01 修改或吸收 Sound runtime 路径。

## 修复结果与回传

- 根因：Only the SynthParameter automation branch preflighted finite values, so Track and Volume accepted NaN and Effect lookup could fail before the shared input contract was checked.
- 架构修复：Moved finite-value preflight into apply_automation_target after the active-state gate and before every target dispatch, resource lookup, graph copy or descriptor mutation; all target kinds now share one typed error boundary.
- 验证：Automation RED job 770baa08b4f24797b1ca0db228e4b2c9 failed 3/3 and GREEN job da450ccadd43494991f956627704041f passed 3/3; route 8/8, final broad 344/344, package check and both locked metadata gates exited 0.
- 回传：Sound automation non-finite preflight is fixed at the shared lowest layer; immutable M1 milestone SHA remains the Render01 F2/Shader06 acceptance boundary.
