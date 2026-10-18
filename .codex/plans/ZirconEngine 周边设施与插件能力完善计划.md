# ZirconEngine 周边设施与插件能力完善计划

## Summary

本轮按“基础设施优先”推进：把 Native 动态插件提升为正式产品 ABI，同时保留 VM 作为沙盒/脚本后端；开发工具、诊断、log 先收束成统一基础设施；动画和物理作为首批能力插件验证新接口。

参考取舍：

- Bevy `bevy_dylib` 用于动态链接加速，不等同于插件 ABI；Zircon 将保留正式 C ABI Native 插件。
- Bevy `bevy_diagnostic`/`bevy_log`/`bevy_dev_tools` 的核心模式是：诊断 store、tracing filter、overlay、schedule dump、截图/调试工具。
- Bevy 当前没有内置完整物理引擎；可借鉴的是 `FixedUpdate`、固定步长、物理状态与视觉插值分离。
- Bevy animation 可借鉴：target id、曲线 evaluator、graph blend/add/mask、events、播放状态和调度顺序。

## Key Changes

- 插件系统：
  - 在 `zircon_runtime/src/plugin/native_plugin_loader` 上新增 ABI v3，保留 v1/v2 兼容加载。
  - ABI v3 继续使用稳定 C ABI 和 byte payload，不暴露 Rust trait object；新增 host log、host diagnostic、capability query、state schema version、command/event manifest schema 字段。
  - 统一 Native 与 VM 的 manifest、capability、diagnostic、export packaging 模型；`NativeDynamic` 成为正式策略，VM 仍是并列后端。
  - 对 physics/animation 这类能力插件采用 host-side adapter：动态库提供 command/state/manifest，运行时侧创建稳定 manager handle 和 scene hook 适配层。

- 诊断与 log：
  - 将 `zircon_runtime::core::diagnostics` 从静态 snapshot 扩展为 Bevy 式 rolling diagnostics store：`DiagnosticPath`、measurement history、smoothed/current/min/max、unit 和 subsystem tags。
  - 保留现有 `diagnostic_log` 文件/console sink，并桥接 `tracing` filter；CLI 保留 `--log-level`，新增 `--log-filter` 用于模块级过滤。
  - Native/VM 插件通过 host diagnostic/log callback 写入同一 store 和 log 文件，插件加载、能力协商、热重载、命令调用、状态迁移都产生结构化诊断。

- 开发工具：
  - 增加 runtime devtools snapshot：模块/service 图、scene hook 调度顺序、插件 catalog、Native/VM 后端状态、diagnostic store 摘要。
  - 在现有 editor runtime diagnostics pane 上扩展插件、调度、log 摘要视图；不要新建与 `zircon_editor` 根结构冲突的独立 editor root。
  - 增加 schedule/export dump，输出类似 Bevy `schedule_data` 的调度和依赖数据，供回归测试和调试面板读取。

- 动画基础设施：
  - 在现有 animation plugin 基础上补齐 target-id 与 mask/additive graph 能力；保留当前 sequence/property track 和 state machine。
  - 新增动画事件 track 与播放事件诊断，scene hook 仍在 `LateUpdate`，并在 transform/render extract 前完成 pose/writeback。
  - 不把具体动画 runtime 放回 `zircon_runtime`；`zircon_runtime::core::framework::animation` 只放中性 contract/DTO。

- 物理接口：
  - 保持 physics 作为插件，runtime framework 只保留中性 `PhysicsManager`、step/query/contact DTO。
  - 明确固定步长模型：`FixedUpdate` 执行物理，step result 暴露 substeps、remaining time、interpolation alpha；视觉插值由 runtime/editor 消费，不由物理插件直接写 editor state。
  - 本轮不接入真实 Jolt/Rapier 后端；保留 `builtin` fallback 和 `jolt` unavailable slot，新增 backend descriptor/diagnostics 以便后续 native backend 插件接入。

## Test Plan

- Runtime/plugin:
  - `cargo test -p zircon_runtime --lib plugin_extensions native_plugin_loader runtime_diagnostics --locked`
  - `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --tests --locked`
  - `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --tests --locked`
  - 新增覆盖：ABI v3 probe、v2 fallback、capability denied、host log callback、diagnostic callback、state save/restore、hot reload failure rollback。

- Devtools/diagnostics/log:
  - 测 rolling diagnostics history、log filter precedence、plugin diagnostics merge、schedule dump deterministic output。
  - Editor pane tests覆盖 runtime diagnostics payload、plugin catalog payload、schedule payload、missing-service fallback。

- Animation/physics:
  - Animation：graph blend/add/mask、event track、loop/reverse event boundary、missing target diagnostics、state transition continuation。
  - Physics：fixed-step zero/one/many substeps、query-only mode、disabled/unavailable backend、contact drain、interpolation alpha、without-plugin no-op behavior。

- Milestone acceptance:
  - `cargo check -p zircon_runtime --lib --tests --locked`
  - `cargo check -p zircon_app --all-targets --locked`
  - `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets`
  - Update docs under `docs/zircon_runtime`, `docs/zircon_plugins`, and `docs/engine-architecture`.

## Assumptions

- User-selected defaults are binding: Native 动态插件是正式产品接口；VM 插件路径保留为双后端。
- 本轮不追求完整第三方物理引擎实现，只完成可接入后端的稳定接口与诊断。
- 不创建新 root package；继续遵守 `zircon_app`、`zircon_runtime`、`zircon_editor` 主结构和 `zircon_runtime::core` spine。
- 所有 animation/physics 具体行为继续属于 `zircon_plugins/*/runtime`，不是 `zircon_runtime` 具体模块。
