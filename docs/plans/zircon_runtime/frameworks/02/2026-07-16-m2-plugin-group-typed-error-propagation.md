# Frameworks 02 M2 Plugin Group Typed Error Propagation

> 本文件记录 `02-module-kernel-and-lifecycle-unification.md` 的 M2 当前修正；父计划仍持有里程碑定义与总状态。

| 里程碑 | 完成项目 | 状态锚 | 日期 | 当前状态与证据 |
|---|---|---|---|---|
| M2 内建模块与 profile 组装切换 | Nested plugin-group module-order error propagation | `frameworks_02_m2_nested_plugin_group_typed_error_focused_passed` | 2026-07-16 | **focused accepted**。`PluginGroupBuilder::add_group(...)` 已从 panic 型 `finish()` 硬切到 `try_finish()?`，嵌套组的缺依赖、依赖环和层级错误继续以既有 `PluginGroupError::ModuleOrder` 返回，不增加 alias、shim、fallback 或第二套排序路径。新增独立 integration contract `zircon_app/tests/plugin_group_error_contract.rs`，避免吸收 `zircon_app/src/plugins/tests.rs` 的其他会话改动。source RED 精确命中旧 `finish()` 调用并 exit 1；修改后 source GREEN、scoped rustfmt、`git diff --check` 通过；独立 review 为 0 Critical / 0 Important / 0 Low。Windows coordinator job `6ef17e9da8c9447d95558a87cfc19067` / run `56e1a9c189554c31ad72951ff7adf257` 执行 `cargo test -p zircon_app --test plugin_group_error_contract --locked`，编译 15m26s 后 1/1 passed、0 failed、test 0.03s、exit 0。首次 validation-copy 仅物化测试文件、因缺 `Cargo.toml` 退出 101，不计为 RED；后续复用池 unit 回归申请被 `cargo_reuse_pool_busy` 拒绝且未启动，因此本行只声明 exact focused gate，不声明全 App 宽门。 |

## 变更范围

- `zircon_app/src/plugins/builder.rs`
- `zircon_app/tests/plugin_group_error_contract.rs`
- `docs/zircon_app/plugins.md`

## 完成门与后续宽门

- 已完成受管 exact contract 1/1、scoped static gates 与独立 review 0/0/0。
- `zircon_app` plugin-group/full-package 宽门留在执行波次收口，不由本修正切片冒充。
- coordinator `milestone prepare --milestone M2` 返回 `workflow_topology_missing`：Plan02 父文件缺少 `zircon-workflow`，且当前父文件另有 7/7 行 foreign testing-policy diff；本 Session 不吸收该文件、不降级到 generic finalize，也不手工 stage/commit。精确 4 文件 manifest 等待独立 plan-topology maintenance 提交后重试。
