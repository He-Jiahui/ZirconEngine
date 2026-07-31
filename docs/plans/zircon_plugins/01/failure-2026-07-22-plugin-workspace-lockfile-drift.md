---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: plugin-workspace-lockfile-drift
origin_plan: docs/plans/zircon_plugins/08-zr-vm.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/zircon_plugins/08
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_plugins/plugin_sdk/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/native/Cargo.toml
tests:
  - cargo +1.94.1 metadata --manifest-path zircon_plugins/Cargo.toml --locked --format-version 1
  - cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --lib --features backend-zr-vm zr_vm_backend_has_one_plugin_owned_dense_production_path --locked --jobs 1 -- --nocapture --test-threads=1
---

# Plugins01: plugin workspace lockfile drift blocks locked consumers

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/08-zr-vm.md`
- 来源执行切片：Runtime13 + Plugin08 generation-owned `ScriptCallTable` atomic consumer gate
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：`zircon_plugins/Cargo.lock` 是整个 plugin workspace 的单一解析产物。Plugin08 只拥有 ZrVM consumer 源码，不能用一个局部测试刷新或手工改写共享 lockfile。

## 失败现象与复现证据

Coordinator reservation `960b69f5b7024610a022acdefdb17c59` 绑定 job `a3dbb677be3846ad9538e1edffc2dfaa` / run `afac3a9afdb0400489163c56f6bd7232`，执行 frontmatter 中的 Plugin08 exact command。作业约 5.82 秒后自然 released `exit 101`，live PIDs 为空；raw stdout 为 0 bytes，目标测试匹配数为 0。唯一 stderr 终止原因是：

```text
error: cannot update the lock file E:\Git\ZirconEngine\zircon_plugins\Cargo.lock because --locked was passed to prevent this
```

当前仓库内 plugin manifest diff 只显示 `zircon_plugins/native_dynamic_fixture/native/Cargo.toml` 新增三个 fixture feature flag；外部 `E:\Git\zr_vm` manifest/lockfile git 状态为空。但没有由 workspace owner 生成并审查候选 lock diff，因此本交接不把某一个 manifest 变更冒充为已证明的唯一根因。

## 最低共享层根因

当前 `zircon_plugins/Cargo.lock` 不能解析当前 plugin workspace manifest/path-dependency 集合，导致所有 `--locked` plugin consumers 在编译和测试前失败。Plugins01 已在自身计划中长期记录同一 workspace lock gate；最低共享修复必须由 plugin workspace owner 统一物化和审查，不能由 Runtime13 或单个 Plugin08 consumer 局部修补。

## 架构修复验收

- 在精确归属当前 plugin manifests、外部 path dependency source 与 canonical lockfile 后，由 Plugins01 owner 通过 managed offline resolution 生成候选 `zircon_plugins/Cargo.lock`，并审查 diff 不含非必要版本升级或网络漂移。
- `cargo +1.94.1 metadata --manifest-path zircon_plugins/Cargo.toml --locked --format-version 1` 以 managed source-bound job 通过。
- 重跑原始 Plugin08 exact command，raw output 证明目标测试实际执行；随后 Runtime13 继续自己的 focused gate 和 atomic review。

## 禁止临时方案

- 不得删除 `--locked`、改用 unlocked 结果冒充验收，或从 workspace 排除触发漂移的真实成员。
- 不得手工编辑 lockfile、复制 root `Cargo.lock`、固定一个 call site 的临时依赖，或吸收未归属的 manifest 变更。
- 不得用本次 0-test job 关闭 Plugin08 handoff，也不得把 lockfile 修复写成 ScriptCallTable 代码通过。

## 修复结果与回传

Open state: `Cargo-owned offline materialization complete / locked consumer validation pending`;
the failure remains open and no Plugin08 or real-fixture pass is claimed yet.

### 2026-07-22 Plugins01 broad 复现

current-source `tests::plugin_extensions` broad gate 中 7 个 real native fixture tests 全部在 fixture 编译前以
同一 `cannot update zircon_plugins/Cargo.lock because --locked was passed` 终止，证明该 lock drift
同时阻断 descriptor/entry、asset importer、load-manifest、missing-export、unknown-ABI 与 capability
negotiation 真实 fixture。静态归因确认 `zircon_plugins/Cargo.lock` 已有 `arc-swap` package，但
lockfile 中 `zircon_runtime` package dependencies 缺失根 `zircon_runtime/Cargo.toml` 新接入的
`arc-swap`；native fixture 的三个新 feature flag 不改变解析图。

按本 failure 禁令，未手改 lockfile。source-bound offline metadata reservation
`540a853ba834479c8a9ee59ba4f1f252`（fingerprint
`ea84b2f4d4a936782f8b05fb48135d7b47d77053cdbc24daa7008643992f785f`）已登记，将由
Cargo 自行物化候选差异；随后仍需 locked metadata 与原 Plugin08/real-fixture tests 复验。

前述 materialization 预约及其后继 `70b4cb59e16b414084f3d5ad6a8e7bfa` 均在 Windows FIFO
长队列中到期且从未绑定 job，因此不构成执行或失败证据，lockfile 仍未手工修改。

当前源码绑定的第一轮 offline metadata 预约 `630d505d7b4e42d9b7d4a6bdd0200156` 已由 job
`73ab02b070614fe0b4aba0f311702b38` / run `16f124a891f54d8b84adbf09c9526d16` 执行完成，
`exit 0`；但该命令带 `--no-deps`，Cargo 没有解析 path dependency 的完整依赖边，候选
`Cargo.lock` diff 为 0，`zircon_runtime` package dependencies 仍缺 `arc-swap`。因此这是一条
“命令不足以物化 drift”的反例，不是修复通过证据。

修正后的 source-bound offline resolution 预约 `b5c83a68db5a4b21be03e1bb74fba9f1`（build config
`plugins01-lockfile-r3`）已由 job `96008f430f804730b6bb3667ea9dbd2b` / run
`a979949683fe42f1b36e709a22068350` 执行不带 `--no-deps` 的完整 `cargo +1.94.1
metadata --manifest-path zircon_plugins/Cargo.toml --offline --format-version 1`，`exit 0`。Cargo
自行生成的候选 diff 仅在 `zircon_runtime` package dependencies 中新增 `"arc-swap"` 一行；没有
版本升级、package 增删或其他解析漂移，当前 lock SHA-256 为
`AEF339812B178636C2880991D68B7171C4DF2F7313BB7FD90ACF8645B5F0068A`。

新 lock 随后通过 source-bound reservation `ee34381afdc24a8fbd816657091b053c`、job
`0cd27b89d9394a45a5b2817ebae2e3ce` / run `70a4cbe0edbb4a69bdb8697bf109e836`
执行的 `cargo +1.94.1 metadata --manifest-path zircon_plugins/Cargo.toml --offline --locked
--format-version 1`，`exit 0`。这证明 canonical plugin workspace 已能在离线、锁定模式完整解析。

本 failure 继续保持 open，直到原 Plugin08/real-fixture tests 取得动态证据；两条 metadata 成功只证明
Cargo-owned 物化、候选差异与 locked resolution 正确，不冒充 consumer 行为通过。

### 2026-08-01 native-only SDK consumer 复现

Performance01 计划/代码复审以 `zircon_plugin_sdk` 的 `native` feature 验证 command-manifest V4
payload schema。受管 job `9ae1b12fc9714863b0146218a39e982f` / run
`50fa15b886044f3889213df7238e488c` 执行：

`cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --lib --no-default-features --features native native_command_manifest_v4_rejects_whitespace_only_payload_schema --locked --jobs 1 -- --nocapture --test-threads=1`

该 job 在编译和测试前以 `exit 101` 自然释放，唯一终止原因仍是 Cargo 拒绝在 `--locked` 下更新
`zircon_plugins/Cargo.lock`。同一 source-bound 输入的 validation-copy job
`3282248f9381464eb1ae0f3647ffeb77` 也在 `closure_planning` 以
`validation_copy_cargo_metadata_failed` 终止，未创建可运行副本。当前主 lock SHA-256 为
`70B559F8A8FE2102C2C0FA74E7A853A98DD3CF4436802D8E4AC96D51ADE498BD`，与本记录早先通过 locked
metadata 时的 `AEF339...` 不同；本 Session 未改写该 lockfile。

为区分代码行为与 workspace 解析失败，受管隔离 harness job
`65a96610695f4fd2b624fa1b4483c7fd` / run `4e1c0c49febb46c18fc3f9559a871886`
以独立 workspace/lock、`--offline` 和 native-only path dependency 编译当前生产
`command_manifest_v4_is_current_and_dense`，实际执行 1 个 whitespace-only payload schema
回归并得到 `1 passed; 0 failed`；主 lock 前后哈希不变。该 1/1 只证明 SDK 生产函数行为，不替代本
failure 的 canonical locked metadata 与原 consumer 门，也不把本记录改为 fixed。
