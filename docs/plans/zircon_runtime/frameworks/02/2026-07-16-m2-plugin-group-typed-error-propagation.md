# Frameworks 02 M2 Plugin Group Typed Error Propagation

Plan: docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
Milestone: M2
Status: completed
Files: ["docs/plans/zircon_runtime/frameworks/02/2026-07-16-m2-plugin-group-typed-error-propagation.md"]

> 本文件记录 `02-module-kernel-and-lifecycle-unification.md` 的 M2 当前修正；父计划仍持有里程碑定义与总状态。

| 里程碑 | 完成项目 | 状态锚 | 日期 | 当前状态与证据 |
|---|---|---|---|---|
| M2 内建模块与 profile 组装切换 | Nested plugin-group module-order error propagation | `frameworks_02_m2_source_committed_focused_passed_m1_accepted_native_ready` | 2026-07-17 | **source committed; focused gate passed; M1 prerequisite accepted; native M2 acceptance ready**。`PluginGroupBuilder::add_group(...)` 已从 panic 型 `finish()` 硬切到 `try_finish()?`，嵌套组的缺依赖、依赖环和层级错误继续以既有 `PluginGroupError::ModuleOrder` 返回，不增加 alias、shim、fallback 或第二套排序路径。新增独立 integration contract `zircon_app/tests/plugin_group_error_contract.rs`，避免吸收 `zircon_app/src/plugins/tests.rs` 的其他会话改动。source RED 精确命中旧 `finish()` 调用并 exit 1；修改后 source GREEN、scoped rustfmt、`git diff --check` 通过；独立 review 为 0 Critical / 0 Important / 0 Minor。Windows coordinator job `6ef17e9da8c9447d95558a87cfc19067` / run `56e1a9c189554c31ad72951ff7adf257` 执行 `cargo test -p zircon_app --test plugin_group_error_contract --locked`，编译 15m26s 后 1/1 passed、0 failed、test 0.03s、exit 0。四文件 focused owner 已随协调器提交 `ad2c6f989cfff927ff5679467ca0cc71e2e20c0e` 进入 Git 历史；父计划 workflow topology 已由 `caba8b043bf90b1db86b5b02dc16e1564f64e672` 落盘；M1 已由原生 milestone commit `83eb1074098471c7e08ff996bd81916c5578cb5e` 接受。首次 validation-copy 仅物化测试文件、因缺 `Cargo.toml` 退出 101，不计为 RED；后续复用池 unit 回归申请被 `cargo_reuse_pool_busy` 拒绝且未启动，因此本行只声明 exact focused gate，不声明全 App 宽门，也不把 native-ready 冒充已提交的 M2 workflow acceptance。 |

## Scope Delivered

- `zircon_app/src/plugins/builder.rs`
- `zircon_app/tests/plugin_group_error_contract.rs`
- `docs/zircon_app/plugins.md`

## Fresh Testing Evidence

- 已完成受管 exact contract 1/1、scoped static gates 与独立 review 0/0/0。
- 四文件 focused owner `zircon_app/src/plugins/builder.rs`、`zircon_app/tests/plugin_group_error_contract.rs`、`docs/zircon_app/plugins.md` 与本记录已由 `ad2c6f989cfff927ff5679467ca0cc71e2e20c0e` 提交；不再把当前源码写成待提交工作树。
- `caba8b043bf90b1db86b5b02dc16e1564f64e672` 已为父计划补入 canonical `zircon-workflow` 的 M1 → M2 → M3 依赖拓扑；旧 `workflow_topology_missing` 是已解决的历史控制面结果。
- `zircon_app` plugin-group/full-package 宽门留在执行波次收口，不由本修正切片冒充。
- M1 已由原生 workflow commit `83eb1074098471c7e08ff996bd81916c5578cb5e` 接受；M2 当前只待在同一 workflow run 中执行 prepare/validate/review/commit，不降级到 generic finalize，也不手工 stage/commit。

## Review

- 既有独立 review 为 Critical 0 / Important 0 / Minor 0；确认 `try_finish()?` 是嵌套 plugin-group module-order 错误的唯一传播路径，`add_group` 不再调用显式 `finish()` panic wrapper，也没有 compat alias、shim 或第二套排序器。
- M2 只接受 plugin-group 类型化错误传播和已提交 focused contract；不把 1/1 integration gate冒充全 `zircon_app` 宽回归，也不提前声明 M3 RuntimePlugin 生命周期完成。
