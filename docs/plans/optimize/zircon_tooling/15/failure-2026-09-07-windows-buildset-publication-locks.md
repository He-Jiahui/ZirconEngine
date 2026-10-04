---
handoff_kind: failure
status: open
created_at: 2026-09-07
summary_slug: windows-buildset-publication-locks
plan_link_mode: child_record_only
origin_plan: docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
fixing_plan: docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md
origin_child_dir: docs/plans/optimize/zircon_app/08
fixing_child_dir: docs/plans/optimize/zircon_tooling/15
related_code:
  - tools/cargo/src/build/product_build/build_set.rs
  - tools/cargo/src/build/product_build/build_set/behavior_tests.rs
  - tools/cargo/src/build/product_build/build_set/namespace_lock.rs
  - tools/cargo/src/build/product_build/build_set/namespace_lock/journal.rs
  - tools/cargo/src/build/product_build/batch.rs
  - tools/cargo/src/build/receipt/receipt_writer.rs
  - tools/cargo/src/build/receipt/receipt_writer/windows_publication.rs
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package cargo-zircon -LibTests -SkipBuild -LinkMode static
---

# Tooling15: Windows BuildSet 与凭据发布路径保护失败

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md`
- 来源执行切片：M2 显式 Runtime 产品 DLL 入口及现有产物凭据回归。
- 修复责任计划：`docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md`
- 交接原因：失败位于已有产品 BuildSet 与凭据文件发布实现；App08 的产品 feature guard、开发 DLL 和编译池租约不拥有此路径保护机制。本轮未修改以下两项测试或其锁实现。

## 失败现象与复现证据

在 Windows 11 / Rust 1.94.1 MSVC、受管静态链接模式下执行：

```powershell
./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package cargo-zircon -LibTests -SkipBuild -LinkMode static
```

受管作业 `d3e7e0d48d104d5ba47e5312353ed011` 的源码清单为 `1b737a0f08688b2ba5ed097ddde7dd02439ca30debbcdb7ea19d05c2fd9dbe47`。检查通过，测试结果为 93 passed、2 failed、60 ignored；Cargo 测试退出 101，入口退出 1。忽略项是现有 release-only 性能测试。

| 测试 | 实际结果 | 原有契约 |
| --- | --- | --- |
| `build::product_build::build_set::behavior_tests::locks_the_snapshot_namespace_against_absent_input_and_a_b_a_mutation` | `behavior_tests.rs:159` 对新增 `source/build.rs` 的写入成功，`is_err()` 断言失败 | `ValidatedBuildSet` 存活期间不允许插入原清单缺失的输入 |
| `build::receipt::receipt_writer::windows_tests::publication_file_denies_replacement_until_its_handle_is_released` | `receipt_writer.rs:235` 删除新发布硬链接成功，`is_err()` 断言失败 | 原临时文件保护句柄存活期间，发布路径也不可替换 |

作业阶段耗时：排队 20.11 s、副本同步 24.38 s、检查 4.53 s、编译链接 7.30 s、测试执行 1.39 s。351 个依赖包全部通过完整性校验，无修复。机器观测峰值新增占用 11,780,096 bytes，包含可能的并发活动；不以失败作业作提速证据。

结构化凭据位于 `E:/cargo-targets/zircon-engine/pool/a7c726fcdd457d7fdb7ca72b3e112234afdbb64adbf582a15546e2d1366ad3b3/.zircon-compile/metrics/validation-d3e7e0d48d104d5ba47e5312353ed011.json`。作业已终态释放，无存活子进程。

同源码的 `product_build_owner` 集成测试已由作业 `67c4895d5e4146048a1baa2ee0098a74` 通过 6/6；它使用 fake Cargo，不覆盖这两项 Windows 路径保护行为，也不证明真实 Runtime C ABI 加载。

## 最低共享层根因

`build_set.rs` 的 Windows 目录打开方式使用 `FILE_SHARE_READ`、`FILE_FLAG_BACKUP_SEMANTICS` 与 `FILE_FLAG_OPEN_REPARSE_POINT`。持有该目录句柄未阻止新增子文件，已枚举文件的句柄保护与编译前后清单校验不足以证明缺失输入在整个构建期间不曾出现。

`receipt_writer.rs` 先持有临时文件句柄，再用硬链接发布输出路径。当前句柄保护了原临时路径，但测试证明新链接仍可被删除。文件内容句柄与发布路径身份需要分别建立可验证的保护；不能从原路径不可删除推导新路径不可替换。

这两项产品输入/发布契约与 coordinator 编译池的独占租约、内容同步和前后验证不同。修复应落在 Tooling15 的产品路径所有权与生命周期机制，并保持跨入口一致。

## 架构修复验收

- 保持现有两项测试，验证保护期间缺失输入插入、已有输入修改及 A/B/A 修改均被拒绝；保护释放后正常写入恢复。
- 验证凭据发布期间输出路径的删除/替换被拒绝，发布成功后凭据字节、路径身份与 receipt digest 一致；覆盖竞争、失败清理和占用恢复。
- 先运行对应 `-TestFilter` 的受管库测试，再重跑原始完整库命令和 `-TestTarget product_build_owner` 的受管集成测试。
- 修复回传后，App08 重新执行产品入口回归。真实 Runtime C ABI 导出/加载仍需客户端源码交接修复后另行通过。

## 禁止临时方案

- 不得忽略、删除、改写预期或仅在测试路径绕过这两项失败。
- 不得仅增加构建后扫描或重试来声称阻止了运行期间的 A/B/A 修改。
- 不得用临时文件句柄或硬链接内容相同替代发布路径保护证据，也不得把 fake Cargo 通过等同于产品 C ABI 验收。
- 不增加兼容别名、静默回退、重复权威数据或单一调用点特例。

## 修复结果与回传

修复 Session 为 `failure-roll-01a07160-tooling15`，状态仍为 open。原始失败和两项测试预期保留；新源码快照 `3044` 修复了 `3002` 独立审查发现的两项问题，并通过下层、完整库及产品构建集成测试。独立终审已通过 Critical 0 / Important 0 / Moderate 0，跨计划支持归属与 closeout 证据绑定仍待完成。

### 2026-09-08 修复快照与审查

- 初始修复快照 `2984` 在枚举前持有目录身份并设置禁止新增、删除和重命名的非继承 DACL；凭据通过原保护句柄执行不覆盖目标的原子重命名。单次及批量产品构建显式结束目录保护，失败路径也恢复原权限。
- 独立审查对 `2984` 报告 Critical 0 / Important 2 / Moderate 0：异常终止后 DACL 缺少持久恢复，以及原 null DACL 被误转换成 deny-only 权限。两项意见均保留为本次修复验收内容。
- 修订快照 `2991` 精确冻结本 Session 的七个源码和测试文件。新增恢复日志在权限变更前同步原 SDDL 和卷/目录文件 ID；正常入口独占日志并核验所有目录身份后逆序恢复。正常释放通过日志原句柄删除日志；恢复失败保留日志。原 null DACL 在任何权限修改前明确拒绝。
- 原命名空间失败测试新增无 Drop 子进程退出、再次正常打开恢复、原 SDDL 对比和日志清理断言；另覆盖 null DACL 权限不变和枚举失败后的恢复。`2991` 已交独立审查，尚未收到终审结论。
- App08 拥有的 `product_build.rs` 单次构建消费者修复独立冻结于 `2981`；产品 DLL 支持文件分别取自 App08 的 `2868` / `2986`。Cargo Windows 依赖由 Astra 原 owner 更新。验证可复用这些精确支持字节，跨计划源码提交仍由各自 owner 处理。

### 受管验证记录

1. 初始下层作业 `6b08f54d09fb43db878cc7c0dc7e3788` 使用输入摘要 `3b8ab670a348bf9098945e84803da9355fff7af20d2cbd32a3dfb5a9c0de47ef`，在 `--locked` 检查阶段因 Cargo.lock 与父快照依赖图不匹配退出；没有执行修复源码或测试。日志：`E:/cargo-targets/zircon-engine/cache/build-benchmarks/tooling15-path-locks-2984-lower-20260908/results/namespace-lower.log`。
2. 新完整验证输入 `E:/cargo-targets/zircon-engine/cache/build-benchmarks/tooling15-path-locks-full-20260908` 封存 `10792` 个文件，摘要 `5c334a0b0eabbb8e045746a3706ab627eccf918f57076f3c420a73dcb5a5a1e5`。包含 `2991` 七文件和 App08 精确支持；使用父快照 Cargo.lock，仅为 `cargo-zircon` 加入已授权的 `windows-sys 0.61.2` 依赖，TOML 结构比较确认无其他锁文件差异。
3. 对上述新输入的首次申请在进入编译前返回 `cargo_reuse_pool_busy`，占用作业为 `94924f658d164ab5a299830cd6aa7b11`；没有产生新验证票或测试结果。日志：`E:/cargo-targets/zircon-engine/cache/build-benchmarks/tooling15-path-locks-full-20260908/results/namespace-lower-r2.log`。占用作业后来由协调器确认 orphaned 且进程树为空，原 owner 通过请求 `9fa877d07bb04a71aa032cadec19c638` 释放租约；未伪造退出码或通过证据。
4. 同一输入摘要的下层作业 `a10e55adaa2747b8b24aeda45bfa6841` 实际执行 `-TestFilter build::product_build::build_set`：检查 18.977 秒、编译链接 11.065 秒、测试阶段 1.173 秒；11 passed / 2 failed / 14 ignored，退出 1。两项失败均为新恢复路径遇到 SDDL 中的 NUL：原失败测试已进入无 Drop 退出后的恢复，null DACL 测试已进入原权限恢复。完整日志和嵌入式受管凭据保留在 `E:/cargo-targets/zircon-engine/cache/build-benchmarks/tooling15-path-locks-full-20260908/results/namespace-lower-r3.log`。
5. 根因是 `ConvertSecurityDescriptorToStringSecurityDescriptorW` 返回缓冲区容量而非有效字符串长度，旧转换只移除最后一个 NUL。按 [Windows API 契约](https://learn.microsoft.com/en-us/windows/win32/api/sddl/nf-sddl-convertsecuritydescriptortostringsecuritydescriptorw) 改为在容量内定位首个终止符，只解码其前缀；不读取或保存缓冲区尾部。源码快照 `2998` 仅相对 `2991` 修改 `namespace_lock/journal.rs`，新哈希为 `291523d72356e158f733de3495f84cddd45180cb0e031985f524d31f69de1df9`。
6. 新输入 `E:/cargo-targets/zircon-engine/cache/build-benchmarks/tooling15-path-locks-sddl-2998-20260908` 封存 10792 文件，摘要 `4cf30b2f5aad6215866607e613f5ff61e7c4bf9c811a7cac197cb3906d8a2dae`；完整清单核对确认只有上述 journal 文件改变。作业 `78ec84aeebbb42cea19ecc62dfa5568d` 执行同一下层命令，12 passed / 1 failed / 14 ignored；原异常退出恢复已成功，剩余失败是新增测试严格比较 `D:AI` 与 `D:` 的自动继承信息标记。
7. 最终快照 `3002` 将恢复断言明确为逐个 ACE 原始字节及顺序、访问掩码、继承标志和 DACL 保护状态一致；Windows 的自动继承信息标记不代替权限语义。另增加完整目录扫描遇到子目录 null DACL 的拒绝及原权限不变回归。原始两个失败测试预期未削弱。参考 [自动 ACE 传播](https://learn.microsoft.com/en-us/windows/win32/secauthz/automatic-propagation-of-inheritable-aces) 与 [安全描述符控制位](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-control)。
8. 下层输入 `tooling15-path-locks-acl-3002-20260908` 摘要为 `d9ca0b5086b10ad31da520bdb3525ab7a66fab5c63808bcd051ca8b6ce9fd9c0`。作业 `ea7cb446cf554e6ea4d1ab3a7ce6ad96` 通过 BuildSet 13 项，14 项原有性能测试忽略；作业 `70aeb47b59e242418319b231a0f13fe0` 通过 receipt 4 项，1 项原有性能测试忽略。实际覆盖无 Drop 异常退出恢复、根及子目录 null DACL、发布占用、竞争和失败清理。完整库作业 `cb4eccf775d04fcba54469e565ebf835` 为 99 passed / 0 failed / 60 ignored。
9. 同输入产品集成作业 `6cc67b116899404da0055452b0c00060` 为 5 passed / 1 failed。四产品批量测试的旧 fake Cargo 对 App08 新增的 Runtime DLL `cargo rustc` 命令返回 9。现行已有夹具 `tools/cargo/tests/product_build_owner/batch.rs` 支持该协议，哈希 `d3c18e9885f068687295d97ff37bc5f02e57f97e14cef2d114f82f19253d8b74`，以只读支持快照 `3004` 冻结；它尚无可证明的源码归属，本 Session 未编辑、归属或纳入提交范围。
10. 最终完整输入 `E:/cargo-targets/zircon-engine/cache/build-benchmarks/tooling15-path-locks-batch-support-20260908` 仍为 10792 文件，摘要 `5121a4607004d3ce764a8c8a2b63fb552587b95f903e04adf1e5e298dacd8cc6`。完整清单比较确认相对前一输入仅增加该夹具修复字节。作业 `169da6533390454f98d059833a985fb5` 实际通过 `product_build_owner` 6/6；作业 `22cd236a3e77474f8507894102efe693` 实际通过完整库 99 项，0 failed / 60 ignored。两者均保留 `--locked`，入口及受管凭据 exit 0；日志分别为该目录下 `results/product-build-owner.log` 与 `results/full-library.log`。这些集成测试使用 fake Cargo，不证明真实 Runtime C ABI 加载。

旧失败作业 `a10e55adaa2747b8b24aeda45bfa6841` 的三个测试临时目录残留权限曾阻止产物清理。核对绝对路径均位于该作业 scratch、不是重解析点后，仅对这三个测试目录恢复继承权限；协调器清理请求 `151841ce35444809b5ec5d8128680ff5` 随后删除该作业 scratch，失败及残留列表均为空。未修改用户配置、其他作业目录或协调器持久状态。

最终源码快照 `3002` 的七个文件哈希：

| 路径（相对 `tools/cargo/src/build/`） | SHA-256 |
| --- | --- |
| `product_build/batch.rs` | `d8cd58084fbaaea6c3e153a23a20c67bbdd69d483ebc7caeb6754a2e2be38bd8` |
| `product_build/build_set.rs` | `0fea1118455cf64a908ba046bdc5ef5fe34c4f2feb0e364e944f394e6474807c` |
| `product_build/build_set/behavior_tests.rs` | `f875a030fd313fd0b1b53319feea309eb17b9b91e6f27fe6c7d0b942322b7551` |
| `product_build/build_set/namespace_lock.rs` | `f239b126579f991cf04637ce2f89a0a32553439a85b506918e31f9fc5fa81dfe` |
| `product_build/build_set/namespace_lock/journal.rs` | `291523d72356e158f733de3495f84cddd45180cb0e031985f524d31f69de1df9` |
| `receipt/receipt_writer.rs` | `e06d3072bdfc27061927a56b599008700c1f264013af00b1c816fbedb8271ebe` |
| `receipt/receipt_writer/windows_publication.rs` | `f091af8349045c0826a2823f53646a587fc1ed0b75b0e02ad101824d9b33870a` |

下一步由独立审查任务核对最终 `3002` 与验证支持闭包，处理跨计划支持归属，并取得 Critical / Important / Moderate 全零结论。上述 `validate-matrix` 作业属于 `validate-matrix:failure-roll-01a07160-tooling15` 操作 Session；真实动态日志不能冒充 fixing Session 的 `cargo_job_runs` 或验证票据。closeout 必须通过现行协调器绑定精确源码、命令和验证身份，之后才回传唯一 `fixed-*`、提交 `fix(failure): windows-buildset-publication-locks` 并核对按 SHA 去重的企微结果。真实产品 C ABI 仍按其原责任链独立验收。

### Final Review Dispatch

The final 3002 source and 3005 evidence record were submitted to the user's
existing coordinator-efficiency review task
`01a07063-6f03-7803-a12d-13ea015ca645` through the local Codex queue, message
`01a07ead-e449-7d83-b645-668e76e8cd0b`. This updates the obsolete 2991 review
context with the final hashes and completed lower/full/integration results.
The request requires read-only before/after hash verification, findings by
severity, and no ownership claim over App08, Astra or the unowned batch fixture.
Delivery is queued; no final review conclusion is implied. The original
diagnostics author also received the existing Coordinator01 provenance next
step in message `01a07eae-17cf-7e80-b53e-9469474eab50`; pending validation ticket
identities and the root's promise to wait for that support response are intact.

### Review Findings Repaired In Source 3044

The existing review task completed the resumed `3002` review with Critical 0,
Important 1, Moderate 1. The complete report is retained at
`E:/Git/ZirconEngine/.codex/tmp/tooling15-review-resume-20260908-r2-result.txt`.
It found that journal recovery accepted a present but null DACL, and that
receipt cleanup missed unwind/process-exit paths and ignored deletion errors.
The source/current/object hashes matched all seven `3002` paths throughout
that independent review. Its result supersedes the preceding pending dispatch;
it was not an acceptance result.

Source `3041` added four reproductions without changing production. Its input
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/tooling15-review-regressions-3041-20260908`
has 10,792 files and digest
`6c768a4339241d1a8c59f5dfb70149d03199eea2f5326314318d5e0fe97a8aa5`.
Managed job `102827b130a84636aade70f76feb9e67` ran BuildSet: 13 passed, the new
null-DACL recovery case failed, 14 existing performance cases ignored. Job
`31293adedbbc43c3bfb44de081833324` ran receipt tests: 4 passed, the three new
panic/process-exit/deletion-error cases failed, 1 performance case ignored.
Both had real test execution and exit 1. Earlier jobs `193091479fd34dd3949c603b45718695`
and `c4ab2dbe3b274ace8c741b482a140cb7` failed before tests because the convenience
entry added the unsupported `target-client` feature; their logs are preserved
as command-configuration failures, not regression reproductions.

Snapshot `3044`, request `dbcce5b02b3f4422ba50564d80382e9a`, retains the same
seven owned paths and changes only these three hashes from `3002`:

| Path under `tools/cargo/src/build/` | SHA-256 |
| --- | --- |
| `product_build/build_set/namespace_lock/journal.rs` | `9a1c547b81862330b73a71f900672e522cc1496eae7fa10929bc4f734b429495` |
| `receipt/receipt_writer.rs` | `dbf07752fd6d73305e78a52a277173cd371f9cb82f1a8789f693d82f087ceda3` |
| `receipt/receipt_writer/windows_publication.rs` | `3ae743c6cb7be3c6887057ab3b420225ee1670a9fc38c1380d8783b983c1caf5` |

Recovery now validates all directory identities and parsed DACLs before any
permission write. Missing/null DACLs fail while retaining the original journal.
The recovery regression also proves a later valid child record cannot mutate
permissions before an earlier invalid record is rejected. This follows the
[Windows DACL API contract](https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-getsecuritydescriptordacl).
Temporary receipt ownership is held by an RAII guard. Windows cleanup marks the
same open file identity for deletion, propagates ordinary cleanup errors, and
reports unwind cleanup failures. A bounded scan recovers only matching reserved
temporary names; active handles are preserved, reparse/non-file inputs rejected,
and exact-name collisions get their own bounded recovery attempt. Publication
still performs a non-overwriting rename on the original locked handle.

Final immutable input:
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/tooling15-review-repair-3044-20260908`,
10,792 files, digest `976ca495b70a47997f06a16ac4b200eea79c10f685d71585564d9986bbd10043`.
Only the three reviewed files differ from the preceding regression input.
All four managed commands use the original `validate-matrix.ps1` package path,
`-SkipBuild -LinkMode static`, Cargo `--locked`, and this exact input digest:

| Gate | Job | Result |
| --- | --- | --- |
| BuildSet lower regression | `3f20c11e286b443ea2ddcea61794d7c8` | 14 passed, 0 failed, 14 ignored |
| Receipt lower regression | `4d5a258d8e2842b6af56c630d4a4dd81` | 8 passed, 0 failed, 1 ignored |
| Original full library | `bc6e774bebec4a32a8b6c4707c1b6fcd` | 104 passed, 0 failed, 60 ignored |
| `product_build_owner` | `520da2c1da854580af2ce7bf2a301279` | 6 passed, 0 failed |

All commands and embedded managed receipts returned exit 0. Their exact logs,
commands, results and post-run manifest checks are in `results/green-managed-*.json`
and matching logs. The original ignored performance tests were not reclassified.
The added live-writer recovery regression preserves occupied and unrelated files.
Product integration still uses fake Cargo and does not certify Runtime C ABI.

App08's existing task completed the requested read-only authorship handback:
the batch fixture at `d3c18e9885f068687295d97ff37bc5f02e57f97e14cef2d114f82f19253d8b74`
cannot be attributed to that author from the available evidence. Source `2981`
only proves its `product_build.rs` complete-boundary change. Preserve `3004`
as read-only unassigned support; the fixture is not part of this seven-file
submission. The exact handback is in
`E:/Git/ZirconEngine/.codex/tmp/app08-support-resume-20260908-result.txt`.
Independent review of `3044` subsequently completed with Critical 0, Important 0,
Moderate 0. The report is retained at
`E:/Git/ZirconEngine/.codex/tmp/tooling15-3044-review-resume-20260908-result.txt`.
It verified all seven source hashes before and after review, the exact `3046`
evidence record, both repaired findings, and the four actual GREEN job results.
It did not mutate source, run validation, claim leases or create Git evidence.

Support ownership and formal closeout identity binding remain required.
The official `validation_copy.materialize_cargo` path enforces same-Session
ownership for overlays; `validation_copy.run` then creates the actual managed
Cargo run. Current HEAD still lacks the Windows dependency declaration and the
App08 consumer/fixture changes used by the successful immutable input. These
foreign changes must be integrated by their respective owners before an owned
seven-file overlay can reproduce the accepted input through that formal path.
Do not relabel the operational jobs, replace the stable fixing Session, or copy
unassigned fixture changes into this commit. This item is suspended for that
dependency while independent failures continue. No new return, commit or WeCom
delivery is claimed by the completed source review and operational jobs.

### 2026-09-12 rolling-closeout reconciliation

The archived implementation owner was not reused. Successor Session
`failure-roll-01a084c8-tooling15-buildset-r1` received the exact seven source
paths, this open record, the future child return receipt, and the future
origin-side fixed record through ownership-transfer fingerprint
`12ad80e92d8fc2b488786bd7c6062fe94b457bbc40f836f0b9733509b91a4b8a`.
All transferred source hashes still match the reviewed 3044 values. A fresh
current-worktree `rustfmt --edition 2021 --check` over the seven paths and
scoped `git diff --check` both returned exit 0.

This is a provenance and snapshot check only. It does not turn the old
`validate-matrix:failure-roll-01a07160-tooling15` operational jobs into
same-Session closeout evidence. Formal validation-copy materialization still
cannot reproduce the accepted 3044 input from current HEAD because the needed
Windows dependency declaration and App08 consumer/fixture support remain in
their own ownership chains. The lifecycle therefore remains open in
`waiting_validation`; no `fixed-*` return, closeout commit, or WeCom delivery
is claimed.

### 2026-09-19 rolling successor source reconciliation

The archived Tooling15 owner was replaced by successor Session
`failure-roll-01a084c8-tooling15-buildset-r2` at baseline epoch `611`. The open
failure record and the seven reviewed BuildSet/receipt source files transferred
under fingerprint
`1c8708d2d2273e6e806abe9856c5ff6c55128f89ebe34b36db8137b1fbf86273` and are
leased to the successor. The transfer preserved the reviewed current bytes and
did not absorb the separately owned App08 consumer, Windows dependency
declaration, or unassigned batch fixture.

Current source hashes remain the reviewed 3044 values:

- `product_build/batch.rs`:
  `d8cd58084fbaaea6c3e153a23a20c67bbdd69d483ebc7caeb6754a2e2be38bd8`
- `product_build/build_set.rs`:
  `0fea1118455cf64a908ba046bdc5ef5fe34c4f2feb0e364e944f394e6474807c`
- `product_build/build_set/behavior_tests.rs`:
  `f875a030fd313fd0b1b53319feea309eb17b9b91e6f27fe6c7d0b942322b7551`
- `product_build/build_set/namespace_lock.rs`:
  `f239b126579f991cf04637ce2f89a0a32553439a85b506918e31f9fc5fa81dfe`
- `product_build/build_set/namespace_lock/journal.rs`:
  `9a1c547b81862330b73a71f900672e522cc1496eae7fa10929bc4f734b429495`
- `receipt/receipt_writer.rs`:
  `dbf07752fd6d73305e78a52a277173cd371f9cb82f1a8789f693d82f087ceda3`
- `receipt/receipt_writer/windows_publication.rs`:
  `3ae743c6cb7be3c6887057ab3b420225ee1670a9fc38c1380d8783b983c1caf5`

Current-source rustfmt and scoped diff checks pass, and the source-contract
checker emits `TOOLING15_BUILDSET_PUBLICATION_LOCK_CURRENT_SOURCE_CONTRACT_PASS`.
Formal same-Session validation-copy reproduction remains required; the prior
operational green jobs are retained as historical evidence only.

The successor submitted static ticket `fba814f9b0bd4202a30ca4dcf980bfe9`
(request `failure-roll-01a084c8-tooling15-buildset-20260919-r1`) with
source-manifest hash
`c31214435302143ed099115316a12fd96ea66b4833a9c77b1e0ec5ca47ae0801`.
The managed source-contract command checks the reviewed BuildSet namespace and
receipt publication/cleanup anchors and emits
`TOOLING15_BUILDSET_PUBLICATION_LOCK_CURRENT_SOURCE_CONTRACT_PASS`.
It is static-only (`staticParseOnly=true`, `upwardAcceptance=false`) and is
queued. Dynamic same-Session Cargo reproduction, App08 support integration,
review refresh, canonical fixed return, and closeout remain pending.

### 2026-09-19 static ticket materialization retry

The first successor static ticket reached the coordinator but failed before
running its source-contract command. Ticket
`fba814f9b0bd4202a30ca4dcf980bfe9` linked materialization job
`3446b6d6c7544a4d91eb80c5a385b947` and terminated with coordinator evidence
`validation_copy_dependency_archive_failed` at `template_dependencies`.
The command did not start and no test or source-contract result is claimed.

The failure was caused by the ticket's `coverage.dependencyRoots` including
three current-session files that are intentionally untracked overlays
(`build_set/namespace_lock.rs`, `build_set/namespace_lock/journal.rs`, and
`receipt_writer/windows_publication.rs`). The pinned `git archive` cannot
materialize those paths as template dependencies; they must remain overlay
paths while the tracked roots are archived from the sealed baseline. A
corrected retry will retain the identical eight-file source manifest and
exclude only those untracked overlays from `dependencyRoots`, with a forced
rerun reason preserving this coordinator diagnostic. This is a validation
materialization correction, not a source repair; the failure remains open.

Corrected retry ticket `57c1a1463e1b4c73adfa111be9686da1` (request
`failure-roll-01a084c8-tooling15-buildset-20260919-r2`) was admitted with the
same eight-file source manifest, now hashed
`fc515ac2ec0d81b2c7a0c7de333ccb5577f73ecae683d5fdd94c2a75191ac1e5`. Its
`dependencyRoots` contain only the four tracked Rust files; the three
untracked namespace/publication files remain explicit same-Session overlays.
The retry is queued with `forceRerunReason` recording ticket
`fba814f9b0bd4202a30ca4dcf980bfe9` and the archive-path diagnosis. Until a
terminal managed receipt exists, this remains static validation pending and
does not alter the dynamic Cargo, review, return, or closeout requirements.

The corrected ticket passed on the same managed wrapper. Ticket
`57c1a1463e1b4c73adfa111be9686da1` linked job/run
`f14a3c3f4388410aa75c296ea440fdb9` and returned exit code 0 with
`TOOLING15_BUILDSET_PUBLICATION_LOCK_CURRENT_SOURCE_CONTRACT_PASS`; the
coordinator cleanup receipt is complete. This terminal result supersedes
neither the retained first materialization failure nor the required dynamic
Cargo/App08 gates: it is a static current-source contract check only. The
session remains open for independent C/I/M review, canonical fixed return and
closeout after the external dependency owner supplies a clean validation
revision.

### 2026-09-21 Independent current-source review receipt

- Reviewer Session `review-tooling15-buildset-r2` claimed this failure document
  through coordinator request `520f9edba9f94eb3b8b8065cf98ad397`. The reviewer made
  no source edits and did not absorb the separately owned App08 consumer, Windows
  dependency declaration, or unassigned batch fixture. The seven source files are
  intentionally dirty/untracked in the shared checkout, but each current byte
  matches the passed ticket `57c1a1463e1b4c73adfa111be9686da1` manifest.
- Current source hashes were verified as follows: `batch.rs`
  `d8cd58084fbaaea6c3e153a23a20c67bbdd69d483ebc7caeb6754a2e2be38bd8`,
  `build_set.rs` `0fea1118455cf64a908ba046bdc5ef5fe34c4f2feb0e364e944f394e6474807c`,
  `behavior_tests.rs` `f875a030fd313fd0b1b53319feea309eb17b9b91e6f27fe6c7d0b942322b7551`,
  `namespace_lock.rs` `f239b126579f991cf04637ce2f89a0a32553439a85b506918e31f9fc5fa81dfe`,
  `journal.rs` `9a1c547b81862330b73a71f900672e522cc1496eae7fa10929bc4f734b429495`,
  `receipt_writer.rs` `dbf07752fd6d73305e78a52a277173cd371f9cb82f1a8789f693d82f087ceda3`,
  and `windows_publication.rs`
  `3ae743c6cb7be3c6887057ab3b420225ee1670a9fc38c1380d8783b983c1caf5`.
- Read-only checks passed: `rustfmt +1.94.1 --edition 2021 --config
  skip_children=true --check` over all seven files; scoped `git diff --check`;
  and a source-contract probe emitting
  `TOOLING15_BUILDSET_PUBLICATION_LOCK_SOURCE_REVIEW_PASS`. The probe verified
  `ValidatedBuildSet`/`NamespaceJournal` admission, final inventory verification
  in the product-build consumer, directory identity and durable recovery records,
  null-DACL rejection, RAII temporary-receipt cleanup and error propagation,
  bounded temporary recovery, and non-replacing Windows publication by the locked
  file identity. It also found the absent-input/A-B-A, null-DACL, publication,
  competition, panic/process-exit recovery, and cleanup-error regression anchors.
- Independent findings: `Critical=0`, `Important=0`, `Moderate=0`. The current
  snapshot preserves the lowest-layer ownership boundary: directory ACL freeze
  and journal recovery protect the BuildSet namespace, while receipt publication
  protects the destination identity separately; no test weakening, retry-only
  workaround, compatibility alias, or unbounded recovery path was found.
- This is a source-only review receipt. Fresh same-Session managed `cargo-zircon`
  focused/full validation, App08 product-staging integration, and formal support
  ownership are still pending; the external `E:\Git\zr_vm` dirty-worktree blocker
  remains active. Historical operational green jobs and supervisor failures are
  not reused. Canonical fixed return, closeout, commit, and notification remain
  pending.
