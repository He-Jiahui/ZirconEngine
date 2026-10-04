---
handoff_kind: failure
status: open
created_at: 2026-08-31
summary_slug: random-checkpoint-authority-generation-mismatch
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime/22
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_contracts/src/random/stream_checkpoint.rs
  - zircon_runtime/crates/zr_contracts/src/random/service_checkpoint.rs
  - zircon_runtime/crates/zr_contracts/src/random/checkpoint_error.rs
  - zircon_runtime/crates/zr_contracts/src/random/tests/checkpoint.rs
  - zircon_runtime/src/core/runtime/random/registry.rs
  - zircon_runtime/src/core/runtime/random/service.rs
  - zircon_runtime/src/core/runtime/random/tests/service.rs
  - zircon_runtime/src/core/runtime/random/tests/retention.rs
tests:
  - cargo test -p zr_contracts random::tests::checkpoint
  - cargo test -p zircon_runtime core::runtime::random::tests
---

# Runtime22: random checkpoint authority generation mismatch

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 来源执行切片：Frameworks01 random checkpoint contract hardening review
- 修复责任计划：`docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md`
- 交接原因：最低共享根因位于 Runtime22 拥有的 random checkpoint capture、restore 与 eviction 生命周期。

## 失败现象与复现证据

`RandomStreamCheckpoint` 只持有 `key + RandomState`，而
`RandomServiceCheckpoint::validate` 只校验格式、算法和 key 顺序。因此外部输入可将 generation
较新的 `RandomServiceState` 与旧 generation 的 stream progress 拼接为一个可反序列化 checkpoint。
恢复后，已注册 key 延续旧状态，未注册 key 则按新 seed/generation 派生；同一服务包含两个从未原子共存的
authority era。world/entity eviction 返回的独立 stream checkpoint 也缺少 generation，无法证明来源时代。

## 最低共享层根因

持久化 stream contract 没有绑定 `master_seed_generation`，service contract 因而没有足够信息拒绝跨时代
组合。先前 `registry -> seed` 原子 capture 修复只封闭了内部调度交错，无法验证构造器或 serde 输入。

## 架构修复验收

- `RandomStreamCheckpoint` 强制携带 authority generation，且 service checkpoint 拒绝任一 stream generation
  与 service generation 不一致。
- checkpoint 格式硬切到 v2；v1 缺字段或旧版本输入必须失败，不保留兼容反序列化、别名或 fallback。
- 完整 checkpoint 与 world/entity eviction 均在唯一 `registry -> seed` 锁序内捕获 generation。
- restore/replay 对已注册及新 key 保持同一 authority era，并复现已注册 key 的 exact next draw/index。
- `evict_stream` 的裸 `RandomState` 仅表示移除状态，不得被描述为可独立恢复 checkpoint。

## 禁止临时方案

- 不添加 v1 兼容字段默认值、serde fallback、旁路构造器、重复 generation 真相或调用点特判。
- 不在释放 registry lock 后读取 generation，也不引入 `seed -> registry` 的第二锁序。
- 不削弱 malformed checkpoint、restore/replay 或 eviction 回归来隐藏失败。

## 修复结果与回传

实现进行中；当前不声明测试、性能、提交、推送或回传通过。

当前修复侧哈希（2026-08-31）已记录如下：

- `stream_checkpoint.rs`: `a353e505a5b674c81f864ab13130acd10cd4f18e37f042048c592e06c1bde44c`
- `service_checkpoint.rs`: `34a8c276f4e2c969e2069d30dfd368d31009f7ee7b91ce09055f0c4fd0edd739`
- `checkpoint_error.rs`: `4c306bc801928cafdcf9ee934f3da1aa68316aef38c36d881eb27e75465d3480`
- `tests/checkpoint.rs`: `9b23ad6de7b5f34a657dad391c249a498d8db34181e290733c4620ede6c2b57b`
- `registry.rs`: `e40c79845fdc51c8a6e3177bd931d8b8166a55fb74c58a7a2aed61d90a955351`
- `service.rs`: `fe073099cf410e10765d848a1c8899dc11aca5affb09f9d1a9964ddc249681af`
- `tests/service.rs`: `3eeb0cdf0d28025cf48a23795385766595222d76b441f5db93086a7545e8de1c`
- `tests/retention.rs`: `e5010dac741f105c61a509fb802345330da18f81651a6d6a85de6634e37e068b`
- `registry/evict_matching_tests.rs`: `15b5cf62b28a0d984f39ea3b78a4e64388bd65d48402118d82376f0c2f83c0d5`

依赖快照哈希：`authority.rs`=`cdb5f29efcec25a84b6f2995fc4cd514f347a2dcc96fe27e799020dc7e4927e0`、
`service_state.rs`=`c10422917a2d8d1896c428c75173d6a7e9821adac2fdadb2af2f75aa93203f21`、
`service_checkpoint.rs`（本修复侧）=`34a8c276f4e2c969e2069d30dfd368d31009f7ee7b91ce09055f0c4fd0edd739`、
`checkpoint_error.rs`（transfer 前依赖快照）=`a8507f27ebdc23f5a7075697b490c257fee39f50e25aef6ed4ae848ff6b02843`。

The `checkpoint_error.rs` dependency was transferred into the Runtime22 immutable scope without
content drift. Runtime22 exact claim `749bc871a4be4e70a1798ab68dc4f3e1` and pre-edit attribution
`a0e8e1b7f99a433d80ffed97d329abc0` bound the transferred
`a8507f27ebdc23f5a7075697b490c257fee39f50e25aef6ed4ae848ff6b02843` source. The typed
`StreamAuthorityGenerationMismatch { index, service_generation, stream_generation }` variant is
now present at hash `4c306bc801928cafdcf9ee934f3da1aa68316aef38c36d881eb27e75465d3480`, closing the compile/scope
gap against the existing constructor and serde regressions. Focused static contract checks and
`git diff --check` pass; the committed checkpoint regression source remains byte-for-byte
unchanged.

The Runtime22 session base head `a832f974...` predates the complete random checkpoint dependency
tree. The owner therefore re-claimed and attributed the exact current contract, Runtime, regression,
eviction, and Failure-record paths before each aggregate attempt. Requests
`runtime22-random-generation-binding-aggregate-20260901-v1` and `-v2` were both rejected before
immutable validation admission with `validation_ticket_external_worktree_dirty` for external
repository `E:\Git\zr_vm`; neither attempt created a Cargo ticket or test result. The v2 lease and
attribution receipts are `c0d843572084480c9b1d14ecc79e0a36` and
`2d67a522021f4bbdb7ba03be4ffda775`; the dependency-specific receipts are
`f660af8fcf1a414684cf3657a4a19993` and `a259546f521749dbafea62253b0006ee`.

The Failure remains open until coordinator-managed Windows Cargo validation proves `zr_contracts`
constructor/serde rejection and Runtime restore/replay plus eviction behavior upward. No compile,
test, performance, commit, push, WeCom, or failure-return success is claimed yet.

### 2026-09-05 lower-layer admission receipt

Session `failure-roll-01a07160-runtime22` re-established the exact checkpoint contract, Runtime,
eviction regression, and Failure scope at baseline 599 / HEAD
`d3174741300b272774c56a37465f043a03debd6c`. Lease request
`ae9c912aac47433c8c9c83a8380b815a` and attribution request
`257abd0f747648cbb5a81df1802ae68a` bind that source. The pending error variant still has the
previously recorded SHA-256 `4c306bc801928cafdcf9ee934f3da1aa68316aef38c36d881eb27e75465d3480`.

The smallest lower-layer command, `cargo check -p zr_contracts --locked --lib`, was submitted as
`failure-roll-01a07160-runtime22-contract-check-v1` with Windows Rust/Cargo 1.94.1 and managed
reuse storage. Coordinator request `cabb9ff7b3c74b18b6d507261d433288` completed at
`2026-09-05T12:42:52.836923+00:00` with `validation_ticket_external_worktree_dirty` for
`E:\Git\zr_vm`, during admission. It created no validation ticket and ran no Cargo command.

The dependency was handed to its active owner task `01a0715f-bae0-7573-aaa7-fed4e781d095` for
notification after its own validated scoped commit leaves a sealable revision. The Engine Session
does not modify that repository or resubmit identical blocked requests. Constructor/serde tests,
Runtime restore/replay and eviction tests, independent closeout review, commit and SHA-deduplicated
WeCom notification remain pending. This lifecycle stays open.

### 2026-09-08 exact-source managed regression results

- The former fixing Session was archived by coordinator retention. Successor
  `failure-roll-01a07160-runtime22-r2` retains the same fixing plan and lifecycle;
  transfer `d9586f7c4dee409598521afcfd821f98` preserves all nine source hashes above.
  Source snapshot 3122 and pre-edit record snapshot 3123 preserve this boundary at
  baseline 601 / HEAD `8243c817d4a2a4a84bda206fcfd0a27320209d5d`.
  Earlier request and job identities remain unchanged.
- Windows managed job `f3c4cc4626fe413b9e3a86db8ee4ede6` executed
  `cargo test -p zr_contracts --no-default-features --locked --lib random::tests::checkpoint`:
  4 passed, 0 failed, 0 ignored. Constructor wrong-era rejection, duplicate-key rejection,
  v2 serde roundtrip and v1/unknown-version rejection all executed.
- Windows managed job `0391f109a80c4cb78763bf7e8500b1ae` executed
  `cargo test -p zircon_runtime --no-default-features --features target-server --locked --lib core::runtime::random::`:
  22 passed, 0 failed, 1 ignored. Capture/restore, exact next-draw replay, invalid restore
  atomicity, generation and registry eviction regressions executed. The ignored release
  eviction benchmark is not constructor/restore acceptance and was not run.
- Both static-link jobs used coordinator-granted reuse storage and the unchanged input
  `E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime25-project-paths-support-3095-20260908`,
  digest `32efa2999d8e8e6a86707ebecf8e10e53d6e68bf89287f60c2c02483b9f66870`.
  Its `results/runtime22-checkpoint-contract.{json,log}` and
  `results/runtime22-random-runtime.{json,log}` bind commands, jobs and actual test output.
  All nine selected source files match the tested input; no source repair was needed this round.
- State: `focused_contract_and_runtime_regressions_passed / independent_review_pending / closeout_binding_pending`.
  The operational `validate-matrix:failure-roll-01a07160-runtime22` jobs are not relabeled
  as fixing-Session validation tickets. Formal closeout binding, independent review,
  `failure return`, commit and SHA-deduplicated WeCom remain pending. This lifecycle stays open.

### 2026-09-08 independent review received

The existing coordinator-efficiency review task returned Critical 0 / Important 0 / Moderate 0
for source snapshot 3122 and record snapshot 3124. The result is preserved in
`.codex/tmp/runtime22-text-3122-review-20260908-result.txt`. It verified all nine Runtime22
current, attributed and ObjectStore source hashes before and after review, constructor/serde
validation, registry-to-seed lock order, restore/replay and eviction atomicity. The two managed
results above still contain 4 and 22 actual passing tests; the release benchmark remains ignored.
This supersedes `independent_review_pending` above. Formal fixing-Session command/source binding,
failure return, closeout and SHA-deduplicated notification remain pending; no operational job
identity has been changed or used as a substitute for that binding.

### 2026-09-19 rolling successor source reconciliation

- Successor Session `failure-roll-01a084c8-runtime22-r3` reclaimed the complete
  checkpoint contract, Runtime random service/registry regressions, dependency
  state, and this failure record at baseline epoch `611`. Ownership transfer
  fingerprint:
  `6559649feb52813017eb36cbce4e1869fdc3082c029d9556d1f08d99ff5b3b28`.
- Current source retains the generation-bound v2 checkpoint contract and the
  registry-to-seed atomic capture/restore paths. Several current hashes differ
  from the archived snapshot only through subsequent formatter/import edits
  (notably `checkpoint_error.rs`, checkpoint tests, registry, service, and
  eviction tests); those bytes are preserved and are not reverted or absorbed
  as new implementation work.
- The existing managed lower receipts remain scoped evidence: `f3c4cc...` ran
  four `zr_contracts` checkpoint tests and `0391f1...` ran 22 Runtime random
  tests plus one ignored benchmark. The independent review remains C0/I0/M0.
  Formal current-source binding, any required fresh validation, canonical
  failure return, closeout, and SHA-deduplicated notification are still
pending; no failure closure is claimed.

### 2026-09-19 current-source static validation

- Static request `failure-roll-01a084c8-runtime22-random-checkpoint-20260919-r1`
  admitted ticket `d4689e4186b741a5b8c88b3a9fe2cce2` with sealed manifest
  `dcffca32ffb40d166d0019b5d583557a1ea850ad5f027443edde99d19a7d46a2`.
  The checker covers generation-bearing stream/service checkpoints, v2 and
  legacy-version rejection, atomic registry capture, restore/retention, and
  eviction-test contracts. Status is `queued` pending terminal evidence.
- This is static-only; fresh managed Cargo reruns, formal current-source review
  binding, failure return, closeout, and the external `E:\\Git\\zr_vm`
  cleanliness prerequisite remain pending.

### 2026-09-19 static checker assertion correction

- Ticket `d4689e4186b741a5b8c88b3a9fe2cce2` ran job
  `1f1bfb10d28e43d9aa979c54a2c6a5b4` and failed at
  `2026-09-19T07:28:33.585483Z` with exit code `1`. The checker incorrectly
  required `format_version` in `stream_checkpoint.rs`; that field belongs to
  `service_checkpoint.rs`. The source contract itself was not changed and no
  Cargo command ran. The coordinator marked this validator assertion
  `failureCacheExcluded=true`; it is retained as non-acceptance evidence.
- A corrected checker will keep the stream generation assertion separate from
  the service format-version assertion and rerun against a fresh manifest.

### 2026-09-19 corrected static retry

- Corrected request `failure-roll-01a084c8-runtime22-random-checkpoint-20260919-r2`
  admitted ticket `565e572687534299aaf097830df62e0e` with sealed manifest
  `574b530f2367c32dafeaf1b4f71e26a89f890d09f21f5796d69488e4bb15828c`.
  The stream generation and service format-version assertions are now scoped to
  their owning files; status is `queued` pending terminal evidence.
- Static-only evidence remains insufficient for closure; managed Cargo reruns,
  current-source review binding, return, closeout, and clean `E:\\Git\\zr_vm`
  remain pending.

### 2026-09-19 third static retry

- Corrected request `failure-roll-01a084c8-runtime22-random-checkpoint-20260919-r3`
  admitted ticket `26bb0d11465a49d9b8e2ff29c8068e69` with sealed manifest
  `d2eec9bf01a2a1fa3faa6d5ace129a621560e1f239c4ba09b30802a43da28ca2`.
  It asserts the actual `version_one`/`unknown_versions` regression names and
  keeps all generation and eviction checks; status is `queued` pending terminal
  evidence.

### 2026-09-19 corrected static terminal result

- Ticket `26bb0d11465a49d9b8e2ff29c8068e69` passed at
  `2026-09-19T07:37:30.908145Z` in job
  `0a83b1e37133411ba248af8f76ccd8a4`, exit code `0`, with marker
  `RUNTIME22_RANDOM_CHECKPOINT_AUTHORITY_CURRENT_SOURCE_CONTRACT_PASS`.
  Cleanup event `10950` completed.
- This confirms the current generation-bound checkpoint, version hard-cut,
  registry capture, restore/retention, and eviction source contracts only. The
  prior checker-only failures (`d4689e...`, `565e57...`) remain preserved and
  excluded from reuse. Fresh managed Cargo/current binding, formal review,
  failure return, closeout, and clean `E:\\Git\\zr_vm` remain pending.

### 2026-09-19 second checker assertion correction

- Ticket `565e572687534299aaf097830df62e0e` ran job
  `2b3a5650a80d418d94c2e67548cd4849` and failed at
  `2026-09-19T07:33:48.067859Z` because the checker required the literal
  substring `v1`, while the current regression names the case
  `version_one` and `unknown_versions`. The source and tests are unchanged;
  no Cargo command ran, and the coordinator marked this validator assertion
  `failureCacheExcluded=true`.
- The next checker will assert the actual test/function names and version-field
  mutations rather than an invented shorthand.

### 2026-09-20 independent current-source review r3

Reviewer session: `review-runtime22-random-checkpoint-r3`, child of
`failure-roll-01a084c8-runtime22-r3`. The review covered the nine currently
attributed contract/runtime/regression paths; the adjacent authority and state
helpers were inspected as dependencies but were not re-owned or edited.

Result: **Critical=0 / Important=0 / Moderate=0**.

- The stream checkpoint carries `master_seed_generation`, while the service
  checkpoint hard-cuts `FORMAT_VERSION` to v2 and rejects a stream whose
  generation or algorithm differs from the service. Serde therefore cannot
  silently materialize a v1, unknown-version, or mixed-authority checkpoint.
- The registry captures the seed authority while holding the registry mutex;
  reseed and stream derivation use the same registry-then-seed order. Scope
  eviction captures generation under that lock and emits generation-bound
  checkpoints, while single-stream eviction deliberately returns only a
  `RandomState` observation.
- Runtime tests cover exact next-draw/index replay, generation exhaustion,
  active-lease atomicity, retention limits, reseeded eviction, duplicate/order
  rejection, and the lock-held capture boundary. The retained benchmark is
  explicitly ignored and is not treated as a passing performance result.
- The corrected static marker passed and scoped `git diff --check` passed.
  `rustfmt --check` reports pre-existing import/order and assertion-wrap drift
  in the current snapshots; no formatter-only rewrite was made during this
  review.

Current reviewed hashes:

```text
zircon_runtime/crates/zr_contracts/src/random/stream_checkpoint.rs
  a353e505a5b674c81f864ab13130acd10cd4f18e37f042048c592e06c1bde44c
zircon_runtime/crates/zr_contracts/src/random/service_checkpoint.rs
  34a8c276f4e2c969e2069d30dfd368d31009f7ee7b91ce09055f0c4fd0edd739
zircon_runtime/crates/zr_contracts/src/random/checkpoint_error.rs
  4c306bc801928cafdcf9ee934f3da1aa68316aef38c36d881eb27e75465d3480
zircon_runtime/crates/zr_contracts/src/random/tests/checkpoint.rs
  b6d364aa58267619e393c3fc2c307dfe6c77449c13c20a0dc9b5769e271aee45
zircon_runtime/src/core/runtime/random/registry.rs
  c274d3402ad8615740255d2220b22db673012ed7fa244ced601245d18e8bab3c
zircon_runtime/src/core/runtime/random/service.rs
  ed28b21c90b9276c04caee73c112a1d0a415cdc0a0d48a2f238685da93132516
zircon_runtime/src/core/runtime/random/tests/service.rs
  3eeb0cdf0d28025cf48a23795385766595222d76b441f5db93086a7545e8de1c
zircon_runtime/src/core/runtime/random/tests/retention.rs
  e5010dac741f105c61a509fb802345330da18f81651a6d6a85de6634e37e068b
zircon_runtime/src/core/runtime/random/registry/evict_matching_tests.rs
  a27ea6c09da8a3dc4eee2ec1160d9f92df141abd414f8b28db609222bf65acdb
```

Fresh managed `zr_contracts` and `zircon_runtime` Cargo runs, Frameworks01
determinism/replay upward evidence, canonical `fixed-*` return, and closeout
remain pending. The earlier checker-only failures are excluded from reuse and
remain historical validation evidence.
