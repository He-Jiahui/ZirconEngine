# Runtime43：Session 测试 wake 构造编译阻断修复

日期：2026-10-11。状态：冻结 patch 已实际采用；原生测试、性能和里程碑验收保持开放。

归属：[Runtime43 Dynamic Runtime Session / Registry / FFI 原计划](../43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md)。本次修复该边界下既有 state / lifecycle 回归的编译阻断，不关闭 Runtime43 finding 或资格门，也不增加性能收益声明。

## 问题与修复

已结束的 `runtime-editor-source-repair-batch-20261010-04` 在首项三包 check 返回 `101`，其余 14 项未执行，原测试与性能执行数量为零。Root 已核对的[终态记录](../../../../../.codex/outbox/runtime-editor-batch04-terminal-checkpoint-v1/terminal-and-closed-log-audit.json) SHA256 为 `053e56ee8bf8ecc3b51225321f6c439d9627f509abfc8378180e59411db7bc09`。

[reload_source_settlement.rs](../../../../../zircon_runtime/src/dynamic_api/session/state/tests/reload_source_settlement.rs) 的两个既有用例从 `state::tests` 调用 registry 私有的 `RuntimeWakeRegistration::disabled()`，产生两处 E0624。冻结 patch 只改两处 import 和两处构造：使用已有 `#[cfg(test)] registry::insert_session(session)`。

实际调用链为 `insert_session -> try_insert_session -> try_insert_session_with_wake(session, RuntimeWakeRegistration::disabled())`。原路径与修复后的路径使用相同 disabled wake、检查过的 handle 分配、`SessionSlot::new`、失败清理和 `expect("test runtime session handle")`。生产文件、可见性、helper 和断言没有变化；该文件原五个测试及其 lifecycle、FFI 和 teardown 正文保留。

## 采用与结构证据

冻结 [candidate.patch](../../../../../.codex/outbox/runtime-dynamic-session-test-wake-construction-fix-v1/candidate.patch) SHA256 为 `7f41ef806b4b72ca61e93ba2c7c8fa7d01a0977ae53e8628f32b07f858f5c8e9`，四个 hunk 已一次应用。采用前 17 项 content / bytes / mtime / creation time / mode / symlink 守卫全部匹配；采用后其余 16 项源码及 controls、shared index、旧空 `index.lock` 元数据保持原样。

实际 canonical 文件的 Rustfmt `--edition 2021 --config skip_children=true --check` 与 scoped `git diff --check` 均为 `0`。该文件尚未进入 shared index，另核 canonical 文件与 `/dev/null` 的 no-index whitespace 检查无诊断（差异返回码 `1`），以及冻结 patch 的 reverse `apply --check --whitespace=error-all` 返回 `0`。未创建源码副本、stage 或操作 Git lock。

| 采用后的文件内容 | SHA256 |
| --- | --- |
| raw，58,424 bytes | `229d5b2cd099b83ceb06dd3aaf67889fed83866289e925671cae605001d6a8a1` |
| normalized LF，57,092 bytes | `1e036bffd0aacd0c452639f07cc1d63dc2f7fd962a2d0b40c1eaa1d8a26bc054` |

证据：[实际采用收据](../../../../../.codex/outbox/runtime-dynamic-session-test-wake-construction-adoption-v1/adoption-receipt.json)，SHA256 `9d86388f44e6bdce606d3cb0900145584d10358b129ca0c06b6d05e3d7a2639d`；[scoped 检查与原筛选交集](../../../../../.codex/outbox/runtime-dynamic-session-test-wake-construction-adoption-v1/scope-and-original-filter-receipt.json)，SHA256 `ec9c5e282e380a78619228b7f32d90a179ef98a3b807cca619dd40ee1baf9b2f`。

## 后续原测试批次

静态 V9 草案 `commands-draft-v9.json` SHA256 为 `da22f4fb8545ac3da29ee1e232d06bff4e5991ae58982fcc2dafa7b118d8a3f4`。其第 2 项 `runtime-scoped-regressions` 的 26 个 OR 子串筛选（剔除 `--test-threads=1` 旗）与以下两个原测试的交集均为零：

```text
dynamic_api::session::state::tests::reload_source_settlement::lifecycle_interruption::owned_events_reject_scene_writes_after_real_lifecycle_interruption
dynamic_api::session::state::tests::reload_source_settlement::lifecycle_interruption::owned_scene_writes_remain_live_after_ordinary_preflight_recovery
```

已将精确名称交给 Root，加入后续同一第 2 项；从该 V9 草案最低 20 的基准加入两条原用例，应至少提高到 22。新增测试和命令 arm 均为零，原 15 arms、14 performance producer、features / profiles 及原阈值保持。本次未修改 manifest，未运行 Cargo、原生测试、性能或协调器实时查询，未提交或推送。

完成状态见 [Runtime 完成列表](../../../astra/features/runtime/2026-10-11-runtime43-session-test-wake-completion-list.md)。
