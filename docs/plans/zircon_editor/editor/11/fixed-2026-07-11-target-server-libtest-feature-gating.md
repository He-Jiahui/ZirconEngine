---
handoff_kind: fixed
status: fixed
created_at: 2026-07-11
summary_slug: target-server-libtest-feature-gating
origin_plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
fixing_plan: docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
origin_child_dir: docs/plans/zircon_editor/editor/11
fixing_child_dir: docs/plans/zircon_runtime/frameworks/03
related_code:
  - zircon_runtime/src/asset/tests
  - zircon_runtime/src/scene/tests
  - zircon_runtime/src/tests/plugin_extensions
  - zircon_runtime/src/tests/prelude.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_runtime --lib --no-default-features --features target-server --locked versioned_json
resolved_at: 2026-07-11
---


# Frameworks 03：target-server lib-test feature 门控失败

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/11-serialization-and-versioning.md`
- 来源执行切片：Plan11 M1.2 场景版本壳与 v0→v1 迁移的 server-profile 聚焦验证
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md`
- 交接原因：`target-server` 生产 lib check 已通过，但 lib-test crate 无条件编译依赖已关闭 feature 的既有测试 owner，导致 Plan11 聚焦测试尚未执行。

## 失败现象与复现证据

2026-07-11 在受管 lane `D:\targets\zircon-engine\lanes\workspace-40b427506d8f48a89c26f69f57c9dbec` 执行：

`cargo test -p zircon_runtime --lib --no-default-features --features target-server --locked --target-dir D:\targets\zircon-engine\lanes\workspace-40b427506d8f48a89c26f69f57c9dbec versioned_json -- --test-threads=1`

命令 exit 101，汇总 73 个编译错误，测试进程未启动。代表性最低错误：

- E0433：`asset/tests/project/asset_flow_sample.rs:22-23` 无条件引用已关闭的 `crate::graphics::{backend::RenderBackend, scene::ResourceStreamer}`。
- E0432：`asset/tests/project/example_vampire/manifest_scene_imports.rs:12` 无条件引用已关闭的 `crate::script::discover_vm_plugin_packages`。
- E0432：`scene/tests/dynamic_scene.rs:8` 在未启用 `physics-contracts` 时引用 `PhysicsWorldStepPlan`。
- E0432：`tests/plugin_extensions/extension_registry.rs:11` 在无 graphics 时引用 `RenderFeaturePassDescriptor`。
- E0433：shader property-layout/asset-flow 测试在无 graphics 时仍直接引用未链接的 `naga`、`wgpu`。

同一 lane 的 `cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked` exit 0，证明失败属于 lib-test feature/profile 编排，不是 Plan11 生产场景代码。

Plan11 随后把 5 项场景迁移合同收敛到独立 integration owner，并成功编译、直接执行为 5/5；但通过 Cargo 选择该 integration target 时，Cargo 仍会先编译 `graphics`、`script`、`dynamic-api` 等未按 `target-server` profile 门控的既有可执行目标并 exit 101。该新增信号与上述 lib-test 73 错同属 Frameworks 03 的 target/profile target-selection 边界，不能由 Plan11 重新开启禁用 feature 规避。

Plan10 M2.2 的受管 `cargo check -p zircon_runtime --no-default-features --features target-server --locked` 再次证明 library 本体可成功产出，随后 production bins 在相同 profile 边界 exit 101：`src/bin/zircon_shader_ide_env/run.rs:7` 无条件导入已关闭的 `zircon_runtime::graphics`；`src/bin/zircon_shader_prewarm/run.rs:9-13` 无条件导入已关闭的 `zircon_runtime::dynamic_api` wgpu/prewarm API（3 个 unresolved）。该信号归本 failure 的 bin target feature gating，不允许 Plan10 用重启 graphics/dynamic-api 规避。

## 最低共享层根因

Frameworks 03 已裁剪 `target-server` 的 production feature 集，但 Runtime lib-test 根模块仍把 graphics/UI/script/dynamic-api/physics-contract 测试 owner 当作常驻模块编译。测试选择器 `versioned_json` 只能过滤运行，不能阻止 Rust 编译未门控测试模块，因此任何 server-profile 聚焦测试都会被不相关域提前阻断。

## 架构修复验收

- 在测试模块声明边界按真实 feature/profile 门控 graphics、UI、script、dynamic-api、physics-contract owner；不要在测试函数内部做运行时跳过。
- 共享 scene/serialization 测试支持必须能在 `target-server` 的实际 production feature 集下独立编译。
- 先复跑 `target-server --lib check`，再复跑上述 `versioned_json` 精确测试，最后执行 Frameworks 03 声明的 profile matrix。
- 保留 client/editor profile 下各域测试覆盖；门控不能静默删除其正常组合中的测试。

## 禁止临时方案

- 禁止为 server profile 重新启用 graphics/UI/script/dynamic-api 或 `physics-contracts` 来掩盖测试边界错误。
- 禁止给退役/禁用域添加空实现、兼容 re-export 或假 `wgpu/naga` 类型。
- 禁止把 production check 通过冒充 lib-test 通过。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Frameworks 03 M1 | target-server lib-test、integration 与 bin owner 门控 | `未通过-生产lib通过-独立场景合同5/5-全目标仍阻断` | 2026-07-11 | `target-server --lib check` exit 0；独立 Plan11 integration binary 5/5；`versioned_json` lib-test exit 101（73 个未门控引用）；Plan10 受管 package check 再证 library 产出成功，随后 shader IDE/prewarm bins 因无条件 graphics/dynamic-api import exit 101。 |

## 修复结果与回传

- 根因：Test module declarations did not express their graphics UI script physics and text feature requirements, so target-server compiled disabled-domain owners before filtering tests.
- 架构修复：Gated test owners at module declarations by their real features and rejected non-finite reflected scalars at the typed JSON write boundary; server features remain minimal.
- 验证：Target-server lib-test no-run exited 0 with 3841 tests; versioned_json passed 4/4.
- 回传：Plan11 M1.2 server-profile serialization tests can resume without re-enabling disabled domains.
