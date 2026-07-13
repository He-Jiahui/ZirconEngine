---
handoff_kind: fixed
status: fixed
created_at: 2026-07-11
resolved_at: 2026-07-11
summary_slug: animation-state-machine-infallible-conversion
origin_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
fixing_plan: docs/plans/zircon_plugins/04-animation.md
origin_child_dir: docs/plans/zircon_editor/editor/14
fixing_child_dir: docs/plans/zircon_plugins/04
related_code:
  - zircon_runtime/src/core/framework/animation/asset/binary.rs
  - zircon_runtime/src/core/framework/animation/asset/state_machine.rs
tests:
  - cargo test -p zircon_editor --locked --no-run --message-format=short
  - cargo test -p zircon_runtime --lib --locked asset::assets::animation
  - cargo +nightly test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --test animation_state_kind_asset_contract --offline --jobs 1
  - cargo +nightly test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --test animation_compiled_state_machine_contract --offline --jobs 1
---


# Plugins 04：Animation state-machine v1 转换错误类型失败

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 来源执行切片：Plan 14 M2 统一测试阶段
- 修复责任计划：`docs/plans/zircon_plugins/04-animation.md`
- 交接原因：错误发生在 Animation 二进制资产 v1 fallback 与 state-machine typed 转换边界，早于 Editor jobs 测试编译，最低共享原因归 Animation 资产合同。

## 失败现象与复现证据

受协调 lane `E:/targets/zircon-engine/lanes/test-e1b65c1437b84754a6cf511a1b948fba` 中，`cargo build -p zircon_editor --locked` 通过；随后 `cargo test -p zircon_editor --locked` 的测试目标编译失败。短格式复现稳定报：

```text
zircon_runtime/src/asset/assets/animation/state_machine.rs:177:13:
error[E0277]: the trait bound `AnimationAssetError: From<Infallible>` is not satisfied
```

诊断日志：`.codex/tmp/plan14-m2-editor-test-compile.stderr.log`。该错误发生在 Runtime/Animation 依赖编译，Plan 14 新增 jobs/export/viewport 测试尚未开始执行。

## 最低共享层根因

`decode_binary_asset_with_v1_payload_fallback<T,V1>` 要求 `<V1 as TryInto<T>>::Error: Into<AnimationAssetError>`；`AnimationStateMachineBinaryAssetV1` 当前通过 `From` 无失败转换为新 binary asset，因此 blanket `TryInto` 的错误类型为 `Infallible`，而 `AnimationAssetError` 没有对应转换。应由 Animation owner 收束 fallback 泛型合同或把 v1 升级改为显式 typed `TryFrom`，不能在 Editor 调用点吞掉类型错误。

## 架构修复验收

- Animation v1/new state-machine binary fallback 的错误类型合同唯一且可组合，不增加兼容 facade 或 Editor 特判。
- `cargo test -p zircon_runtime --lib --locked asset::assets::animation` 通过。
- 原始 `cargo test -p zircon_editor --locked --no-run --message-format=short` 不再出现该 E0277，Plan 14 M2 门禁可继续。

## 禁止临时方案

- 禁止在 Editor 增加 alias、兼容 shim、静默 fallback、重复解码或调用点例外。
- 禁止删除 v1 fallback、弱化序列化测试或用 `unwrap`/不可达分支掩盖 `Infallible` 合同。

## 修复结果与回传

- 根因：State-machine v1 fallback used blanket infallible conversion incompatible with typed animation asset errors.
- 架构修复：Hard-cut fallback DTO migration to explicit TryFrom<Error=AnimationAssetError> and current/v2/v1 typed decode chain.
- 验证：animation compiled state-machine and StateKind binary roundtrip focused tests passed; original E0277 removed.
- 回传：最低共享层错误类型合同已修复并回传 Plan 14；其余 Editor 门禁由上游范围外问题继续验证。
