---
handoff_kind: failure
status: open
created_at: 2026-10-02
summary_slug: runtime-script-feature-net-contract-dependency
origin_plan: docs/plans/astra/features/runtime/07-lifecycle-deadline-and-census.md
fixing_plan: docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
origin_child_dir: docs/plans/astra/features/runtime/07
fixing_child_dir: docs/plans/zircon_runtime/frameworks/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/framework/net/rpc.rs
  - zircon_runtime/src/script/vm/host_interface/descriptor.rs
  - zircon_runtime/src/script/vm/host_interface/registry.rs
tests:
  - python -m unittest tools.tests.test_frameworks_03_domain_feature_matrix tools.tests.test_frameworks_03_server_feature_boundary
  - cargo +1.94.1 check -p zircon_runtime --no-default-features --features script --locked --lib --tests
  - cargo +1.94.1 check -p zircon_runtime --no-default-features --features dynamic-api --locked --lib --tests --message-format short --color never
  - cargo +1.94.1 check -p zircon_runtime --no-default-features --features core-min,script --locked --lib
  - cargo +1.94.1 check -p zircon_runtime --no-default-features --features core-min,dynamic-api --locked --lib
  - cargo +1.94.1 test -p zircon_runtime --no-default-features --features script --locked --lib core::framework::net::tests::rpc_session_and_handshake_descriptors_are_runtime_mode_agnostic -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --no-default-features --features script --locked --lib script::vm::tests::host_interfaces -- --test-threads=1
  - cargo +1.94.1 test -p zircon_plugin_zr_vm_language_runtime --locked --lib tests::host_interface -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --no-default-features --features dynamic-api --locked --lib destroy_registry_deadline -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --no-default-features --features dynamic-api --locked --lib destroy_session_removes_registry_entry_only_after_event_mirror_quiescent_teardown -- --test-threads=1
---

# Frameworks03: script omits its required network contract feature

## 来源执行者

- 来源计划：`docs/plans/astra/features/runtime/07-lifecycle-deadline-and-census.md`
- 来源执行切片：independent Windows r3 compile gate before registry deadline and retry regressions.
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md`
- 交接原因：Frameworks03 owns Runtime feature dependencies; this missing lower contract prevents Runtime07 lifecycle tests from compiling.

The originating [registry deadline failure](../../../astra/features/runtime/07/failure-2026-09-29-destroy-registry-lock-exceeds-deadline.md) remains open. This child record carries the cross-plan status; global plan definitions remain read only.

## 失败现象与复现证据

Independent Windows r3 ran through `tools/dev/local-cargo.ps1` with compiler output under `D:/cargo-targets/zircon-local/windows/runtime07-r3/destroy-registry/target`:

```text
cargo +1.94.1 check -p zircon_runtime --no-default-features --features dynamic-api --locked --lib --tests --message-format short --color never
```

Exit code: **101**. Compilation reached `zircon_runtime`; no dynamic tests executed. `descriptor.rs:1` and `registry.rs:5` reported unresolved imports of `crate::core::framework::net`. The same run also reported separate `policy_index.rs:707:56` and `:711:56` macro syntax errors. Repairing this feature edge alone does not prove that the whole compile gate passes.

- Receipt: [.codex/tmp/failure-roll-20261002-runtime07-local-check-r3.json](../../../../../.codex/tmp/failure-roll-20261002-runtime07-local-check-r3.json).
- Log: [.codex/tmp/runtime07-check-r3.log](../../../../../.codex/tmp/runtime07-check-r3.log).
- Historical run log SHA256: `fe1e9db7be6f4696f6b983e459fb6855512dd0ff54c6e0fab28dc7f11be91983`.

This is actual local compile-failure evidence, not Jenkins acceptance. Historical run evidence is not a seal of the current shared source.

## 最低共享层根因

`dynamic-api` enables `script`. The current `script = ["diagnostic-log"]` omits `net-contracts`, while `core::framework::net` is gated by that feature. Script unconditionally publishes `VmRpcHandlerRegistration` using `RpcPayloadSchema` and stores it in registration and generation snapshots. Its registration methods and manager queries require that contract. The direct ZrVM plugin already explicitly requests `net-contracts` and `script`.

Declare `script = ["diagnostic-log", "net-contracts"]` at the feature owner. The additive matrix requests `core-min,script` and `core-min,dynamic-api`; these requests permit declared dependency closure. Keep the requests unchanged. Gating imports alone leaves required public types unresolved; making the whole RPC registration channel conditional would change the current API and lifecycle semantics.

The feature candidate was prepared before the serial installation. Its manifest preimage SHA256 is `0fb033553bbeb3dc4ad9e7ec85f7c1bbacf435bac01415de6f0774908f3faf75`; candidate SHA256 is `40a48d4ab315fdd401d94de52bf4a0cae3feded3d268aa6adba51acd99cd9cb2`. These identify candidate preparation inputs, not the historical r3 source snapshot. The root installer rechecked the exact preimage, current diff and empty compiler sample, then preserved every foreign baseline byte outside the one declared feature edge. Installation receipt: `.codex/tmp/failure-roll-20261002-shared-compile-install/script-feature/installed.json` (2026-10-02T18:29:32.497120+00:00).

## 架构修复验收

- Source guards must preserve the declared domain requests and Server exclusion contract. This does not assert executed Server acceptance.
- On independent Windows, compile isolated `script`, the exact original `dynamic-api` reproduction, and the declared `core-min` matrix requests. Resolve separate compile errors at their own lowest owners.
- Execute the shared RPC schema contract exact test (expected 1), Runtime host-interface authorization/publication/generation tests (expected 8), and direct ZrVM host-interface tests (expected 4). Report actual nonzero counts. These cover registration and query behavior; they do not establish a production VM-to-network dispatch adapter.
- Rerun the original Runtime07 deadline filter (expected 5) and event-mirror quiescent teardown test (expected 1) under `dynamic-api`, without manually adding `net-contracts`. Preserve retryable slot/census retention and lifecycle semantics.
- Keep all compilation products and compiler caches physically under drive-root `D:/cargo-targets`, `E:/cargo-targets`, or `F:/cargo-targets`. Capture current source attribution and command results. Acceptance remains open until required tests and independent review complete.

## 禁止临时方案

Do not add `net-contracts` manually to the original reproduction or matrix commands as a substitute for fixing the feature edge. Do not remove RPC registration, weaken tests, add aliases or compatibility shims, introduce test-only bypasses, or claim dynamic acceptance from static review. Preserve retired coordinator tickets and historical receipts; do not restore coordinator APIs, leases, or queues.

## 修复结果与回传

Open / 待验证. The one feature edge was installed at the exact boundary above. The independent four-candidate review `.codex/tmp/failure-roll-20261002-shared-compile-independent-review/review.json` reports Critical / Important / Moderate = 0 / 0 / 0; this is static review. No passing dynamic acceptance, Git commit, or notification is claimed. After lower-layer checks and the original upward gates pass, record results and return the same lifecycle artifact to Runtime07 as `fixed-2026-10-02-runtime-script-feature-net-contract-dependency.md`, preserving its provenance and status history.

## 2026-10-03 独立 Windows 下层验证（仍为 open）

现行 `script = ["diagnostic-log", "net-contracts"]` 与已安装的单条修复一致；本次没有重写源码。验证所读 Runtime manifest SHA256 为 `eb028b2a1f78102301f714f6357ea97c84f64a3770e0798be9f7c6b91080509a`。其中另一个任务的 `test-support` 变化被保留，未归属或采用到本 lifecycle。

本轮仅执行原始两个完整 Python source-guard 模块：

```text
python -B -m unittest tools.tests.test_frameworks_03_domain_feature_matrix tools.tests.test_frameworks_03_server_feature_boundary
```

Root 使用实际版本探针确认的原生 CPython 3.14.4 的 `-S -B -X utf8` 启动实际进程。两个原模块的实际测试 ID 与封印清单完全一致，**20/20 passed，errors=0、failures=0、skips=0**。原始测试正文没有替换或投影。测试进程 PID 32320（creation FILETIME `134355053931617891`）于 `2026-10-03T12:49:59.842641+00:00` 结束，exit 0；版本探针 PID 51160 也已 exit 0。二者实际 image SHA256 为 `7ca24f26d6e3f463419ee4f537ddd3acd312c38fe45e678cce08572f26a8bd1a`，没有 pending/unknown 自有子进程，测试内部没有启动子进程。

精确输入快照位于 `D:/cargo-targets/zircon-local/windows/failure-roll-20261003-framework03-python20-capture-r2/snapshot`，保留 `ZirconEngine` 与 `zr_vm` 的相对 sibling 布局。54 个命名文件包含 46 个 Cargo metadata 文件、两个完整测试模块、五个 Rust 文本输入和一个 PowerShell 文本输入；24 个声明的路径不存在守卫与两个精确派生字节码缓存不存在守卫均通过。外部 ZrVM 输入只有两个 binding manifests 和必要 workspace 祖先 manifest；没有 whole-tree archive、外部源码采用、编译或静默 hold 声明。捕获三轮守卫及版本探针前、原用例执行前、全部自有子进程结束后的三轮运行守卫均通过。当前输入与冻结副本各自的前后字节、成员和身份保持一致；二者的内容与相对成员映射匹配，未声称跨目录的文件身份相同。

原测试在物理 D 根目录下创建的 fixture 记录有 8 次 mkdir 事件，对应 **6 个唯一临时目录**；全部删除，临时根为空。实际用例读取了封印清单内的 48 个文件。标准 loader 的两次精确缺失缓存探测另行记录，未视为源码输入；没有生成或执行缓存文件。PowerShell 仅作为文本读取，没有执行其编译矩阵。

- [Root 实际结果接收](../../../../../.codex/tmp/failure-roll-20261003-framework03-python20-root-terminal-reception-r2/terminal.json)：SHA256 `4b07d471cb6603ca7e948bcc630d21c3e52625ecfa979756e49e21d0a888ca1f`。
- [保存证据独立审查](../../../../../.codex/tmp/failure-roll-20261003-framework03-python20-actual-reception-r2/review-final.json)：SHA256 `734fe851b9b9e52ec536d4fd0be40399fed40bd4a50c527a3c4b0cca675e3561`，Critical / Important / Moderate = 0 / 0 / 0。
- 实际 capture terminal SHA256 `187d5e631b3c5158f02293b02cb4a72e15bd74caadf33ed73559eb427edae6c1`；实际运行 terminal SHA256 `163bf8f56b97ff50d6a90811c85d1cf30eb8fff6d77b1e2c65b1e984aa0fd84a`，分别保存于上述物理 D capture 根与 `D:/cargo-targets/zircon-local/windows/failure-roll-20261003-framework03-python20-run-r2`。
- [r2 验证驱动独立实施审查](../../../../../.codex/tmp/failure-roll-20261003-framework03-python20-independent-review-r2/review.json)：SHA256 `b23314a9f1030bffb03ea97ab684bac132067d5fd2f10116ebd0dec9b08569e3`，Critical / Important / Moderate = 0 / 0 / 0。
- [r1 未执行候选的非零审查](../../../../../.codex/tmp/failure-roll-20261003-framework03-python20-independent-review-r1/review.json)：SHA256 `04e338ca1fa1e1f5d5235760213c7365d4431ddb3e6d47b6582577f15ac261cd`，Important=1 的缺失字节码探测误拒绝问题由新 r2 观察器修正。r1 文件、审查结果与历史 20 项结果原样保留；本次没有复用旧结果或修改原测试 loader。

**验收边界：本轮通过的是当前精确输入上的 Python source guards。四组原始 Cargo compile profiles 与 19 项 Rust 测试（1 RPC + 8 Runtime host interface + 4 ZrVM host interface + 5 deadline + 1 teardown）尚未执行，继续待验。** 没有 Runtime07、实际 Server/VM、DLL、Jenkins 或整个 MVP 的验收；没有 `fixed-*` 回传、Git commit、企微通知或 push。原 failure 与 lifecycle 保持 open。
