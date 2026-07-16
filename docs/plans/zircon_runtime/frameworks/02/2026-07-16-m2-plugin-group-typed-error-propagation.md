# Frameworks 02 M2 Plugin Group Typed Error Propagation

> 本文件记录 `02-module-kernel-and-lifecycle-unification.md` 的 M2 当前修正；父计划仍持有里程碑定义与总状态。

| 里程碑 | 完成项目 | 状态锚 | 日期 | 当前状态与证据 |
|---|---|---|---|---|
| M2 内建模块与 profile 组装切换 | Nested plugin-group module-order error propagation | `frameworks_02_m2_source_committed_focused_passed_native_pending_m1` | 2026-07-17 | **source committed; focused accepted; native milestone pending M1**。`PluginGroupBuilder::add_group(...)` 已从 panic 型 `finish()` 硬切到 `try_finish()?`，嵌套组的缺依赖、依赖环和层级错误继续以既有 `PluginGroupError::ModuleOrder` 返回，不增加 alias、shim、fallback 或第二套排序路径。新增独立 integration contract `zircon_app/tests/plugin_group_error_contract.rs`，避免吸收 `zircon_app/src/plugins/tests.rs` 的其他会话改动。source RED 精确命中旧 `finish()` 调用并 exit 1；修改后 source GREEN、scoped rustfmt、`git diff --check` 通过；独立 review 为 0 Critical / 0 Important / 0 Low。Windows coordinator job `6ef17e9da8c9447d95558a87cfc19067` / run `56e1a9c189554c31ad72951ff7adf257` 执行 `cargo test -p zircon_app --test plugin_group_error_contract --locked`，编译 15m26s 后 1/1 passed、0 failed、test 0.03s、exit 0。四文件 focused owner 已随协调器提交 `ad2c6f989cfff927ff5679467ca0cc71e2e20c0e` 进入 Git 历史；父计划 workflow topology 已由 `caba8b043bf90b1db86b5b02dc16e1564f64e672` 落盘。首次 validation-copy 仅物化测试文件、因缺 `Cargo.toml` 退出 101，不计为 RED；后续复用池 unit 回归申请被 `cargo_reuse_pool_busy` 拒绝且未启动，因此本行只声明 exact focused gate，不声明全 App 宽门，也不越过 M1 依赖声明 M2 native accepted。 |

## 变更范围

- `zircon_app/src/plugins/builder.rs`
- `zircon_app/tests/plugin_group_error_contract.rs`
- `docs/zircon_app/plugins.md`

## 完成门与后续宽门

- 已完成受管 exact contract 1/1、scoped static gates 与独立 review 0/0/0。
- 四文件 focused owner `zircon_app/src/plugins/builder.rs`、`zircon_app/tests/plugin_group_error_contract.rs`、`docs/zircon_app/plugins.md` 与本记录已由 `ad2c6f989cfff927ff5679467ca0cc71e2e20c0e` 提交；不再把当前源码写成待提交工作树。
- `caba8b043bf90b1db86b5b02dc16e1564f64e672` 已为父计划补入 canonical `zircon-workflow` 的 M1 → M2 → M3 依赖拓扑；旧 `workflow_topology_missing` 是已解决的历史控制面结果。
- `zircon_app` plugin-group/full-package 宽门留在执行波次收口，不由本修正切片冒充。
- M2 native milestone 仍必须等待 M1 native milestone 完成后，在最新 HEAD 的独立 Session 中执行 prepare/validate/review/commit；不降级到 generic finalize，也不手工 stage/commit。
