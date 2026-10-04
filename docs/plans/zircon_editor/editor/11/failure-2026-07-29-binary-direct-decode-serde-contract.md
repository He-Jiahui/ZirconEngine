---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: binary-direct-decode-serde-contract
origin_plan: docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
fixing_plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
origin_child_dir: docs/plans/zircon_runtime/text/09
fixing_child_dir: docs/plans/zircon_editor/editor/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/serialization/binary/value/direct_decode.rs
  - zircon_runtime_interface/src/serialization/binary/value/mod.rs
  - zircon_runtime_interface/src/serialization/tests/binary_contract.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib text_cache_indexes_keep_hot_lookup_and_eviction_work_constant --locked --jobs 1 --color never -- --test-threads=1
  - cargo test -p zircon_runtime_interface --locked --jobs 1 -- --test-threads=1
---

# Editor11: binary direct decode Serde contract

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md`
- 来源执行切片：Text09 PF-M1 cache and font-handle focused validation
- 修复责任计划：`docs/plans/zircon_editor/editor/11-serialization-and-versioning.md`
- 交接原因：`zircon_runtime_interface::serialization` owns the current typed binary value decoder. Text09 cache, layout, font-handle, and rendering code cannot repair its Serde trait boundary without duplicating or bypassing the shared serialization contract.
- 生命周期键: `binary-direct-decode-serde-contract`

## 失败现象与复现证据

Managed Cargo job `def8ef76e1f846afba17a6bbf017371f`, run `1a68878b2fc044c390339cd80193939f`, executed the exact focused Text09 cache command below and exited with `101` before `zircon_runtime` Text09 code compiled:

```text
cargo +1.94.1 test -p zircon_runtime --lib text_cache_indexes_keep_hot_lookup_and_eviction_work_constant --locked --jobs 1 --color never -- --test-threads=1
```

The shared interface compilation reports two direct-decoder errors:

- `E0046`: `impl serde::Deserializer for &mut BinaryValueDeserializer` is missing `deserialize_bool` at `zircon_runtime_interface/src/serialization/binary/value/direct_decode.rs:189`.
- `E0499`: the enum-value branch takes another mutable borrow of `self` while `variant` is still borrowed at `direct_decode.rs:352`.

The same job also reports `CanonicalArray::end` ambiguity in the canonical text writer. That independent writer failure is already tracked by `failure-2026-07-29-canonical-text-streaming-output.md`; this record deliberately owns only the direct-decode Serde contract.

## 最低共享层根因

`BinaryValueDeserializer` does not implement the complete Serde `Deserializer` surface for the current binary representation, and its enum branch keeps a borrow derived from `self` alive while constructing `ValueEnumAccess` requires another mutable borrow. The shared decoder therefore cannot type-check for every downstream crate, independent of Text09's cache or layout implementation.

## 架构修复验收

- `direct_decode` implements the complete required Serde `Deserializer` contract, including `deserialize_bool`, with the correct binary value type semantics and typed errors.
- The enum-value path owns or clones the discriminant before releasing the `next_node` borrow, then constructs enum access without overlapping mutable borrows.
- Focused Editor11 binary serialization contract tests compile and pass, including bool and enum decoding coverage.
- The original exact Text09 managed cache command compiles and runs after the shared interface fixes; Text09 then reruns its cache, font-handle, parallel shaping, and WGPU product-frame gates before claiming completion.

## 禁止临时方案

- Do not add a Text09-local decoder, compatibility shim, trait workaround, or conditional compilation that hides the incomplete shared Serde implementation.
- Do not weaken bool, enum, typed-error, binary current-version, or serialization contract coverage.
- Do not fold this direct-decode repair into the already-open canonical text streaming record or suppress its compiler errors with unrelated writer changes.

## 修复结果与回传

Open state: `待修复`; Text09 remains active and continues its independently verifiable infrastructure work. No Text09 cache, font-handle, parallel shaping, or product-render acceptance is claimed until Editor11 restores the shared decoder contract and the affected managed gates are rerun.

## 产出记录与时间

- 2026-07-29 | `open / interface-diagnostic-green-for-owner-contract / rerun-pending` | `deserialize_bool` 已由 direct decoder 自身实现；enum discriminant 在再次借用 decoder 前转换为 owned `String`，没有兼容 shim。受管完整接口诊断 `3bb2a442c76748bab54b8fdb21408a9f` / `73795d6cf598446f9e4e8e26dbf18ad2` 中 bool/enum、numeric map key 与 direct current decode 回归均通过；当前后继仅移除编译器确认未使用的 `serde::Deserialize` import。完整诊断仍含其他 owner 失败，因此 fresh focused/full gate、独立复审、Text09 原命令回归和 fixed return 尚未完成。
- 2026-07-30 | `open / focused-owner-contract-green-inside-red-gate / upward-rerun-pending` | fresh focused job `8ee08a1a4f284da6af298ff588b39bf0` / run `2f90cac80bf24728adcd6615c5d4d756` 已重新编译当前 direct decoder，并通过 `binary_current_direct_decode_covers_bool_and_enum_variants`、numeric map key、current typed decode 与全部 Binary wire 回归；完整结果为 `57 passed / 1 failed / 1 ignored`，唯一失败属于 canonical Text `RawValue` struct-marker 分支。401 输入前后指纹同为 `2b05711471b4faccb4cf913d87467049502fd4073b2b4b50f2418daa75519e5f`。该证据确认本 owner 行为已绿，但 failure 仍等待 fresh 全 focused GREEN、Text09 原命令、独立复审和 fixed return。
- 2026-07-30 | `open / focused-green / text09-upward-pending` | 后继 job `76fdfb36f35c413192a520565669c34a` / run `c3bbf7083c3b4139849b91ec4cee5621` 已在 401 输入无竞态条件下 GREEN：`58 passed / 0 failed / 1 ignored`；所有 direct current decoder、binary wire 与 malformed contract 均通过。Text09 原 cache 命令、独立 exact31 复审和 failure fixed return 仍是关闭前置，故本记录不提前改为 fixed。
- 2026-07-30 | `open / broader-current-source-green / text09-upward-pending` | 当前 exact32 源的 broader serialization job/run `be60c135d75d4b57a8ff09d10e9ef21d` / `e581979bac5f4bfd8633c393cfd7982e` 重新运行 63 项并得到 `62 passed / 0 failed / 1 ignored`；Binary direct bool/enum/current decode、malformed 和 typed error 合同均在该 GREEN 集内。完整 402 输入前后保持 `fa51e11fded0881cd4c641fb5c41a0250265fd04afcbe710844037ecfae0aeaf`，snapshot1313 interim review 为 `C0/I0/M0/Minor0`。Text09 原命令、最终 fresh exact32 review、fixed return 与受管提交仍未完成，failure 保持 open。
- 2026-07-30 | `open / full-interface-green / text09-upward-pending` | Fresh full-package reservation/job/run `3b301ffebcb54fb881e02291fe40e2d4` / `ac76251fad174b18a80bc3dbaa1745a3` / `dd5e4239cf664f038f51934dc38292a4` passed unit `338/0/1`, integration `3/3`, and doc-tests `0`; complete 402-input pre/post fingerprint was `18a0b4f7ad2a3f6e0255e76a44f8080b9c067cb8e78d8bb189327cec732d962a`. The exact Text09 reproduction, final expanded-scope review, fixed return and managed commit remain required.
- 2026-08-28 | `open / text09-upward-rerun-reached-foreign-rhi-compile-debt` | Fresh managed job `e91cafa704a644878089247129b8c8dd` ran the exact `text_cache_indexes_keep_hot_lookup_and_eviction_work_constant` reproduction and released normally with exit `1` and no live process PIDs. `zircon_runtime_interface` compiled without a direct-decode diagnostic, but the target test executed `0`: `zr_rhi_wgpu` stopped first with three external E0308 errors at `production/device/diagnostics.rs:665/722/771`, where untracked current-source callers still pass `u32` to `DiagnosticTextureReadbackLayout::new(u64, u32)`. The two RHI paths had no attribution, lease, or active Session scope at the observation boundary, so this owner did not absorb or modify them. The failure remains open pending the RHI lower-owner return and a fresh execution of the same managed command.

### 2026-09-08 Exact Existing Repair And Current Gate

Stable fixing Session `failure-roll-01a07160-editor11` preserves the existing decoder,
module registration and binary contract tests, all unchanged from HEAD and archived
snapshot 2091. Transfer `6a6001c4b6584e6ea47b863730648761` establishes the current
ownership; source snapshot 3129 and pre-edit record snapshot 3130 retain the exact bytes.
`deserialize_bool` accepts only a binary Bool, and enum discriminants are owned before
the decoder is borrowed again. No new serializer implementation was introduced.

Managed Windows job `61f0ee23ee454d509a19f5a075dcd57a` used
`cargo test -p zircon_runtime_interface --no-default-features --locked --lib serialization::tests::`
with static linking and coordinator reuse storage. Input
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime25-project-paths-support-3095-20260908`
has digest `32efa2999d8e8e6a86707ebecf8e10e53d6e68bf89287f60c2c02483b9f66870`;
`results/editor11-serialization-contract.{json,log}` preserves the receipt and diagnostics.
All serialization files match this input. The attempt stopped at 23 unrelated interface
test-compilation errors and ran zero target tests. It is not a current serialization pass.

The UI-contract subset is now repaired by Interface03 source snapshot 3128 under its
separate owner; the linked
[ActivateLink failure](../../../optimize/zircon_runtime_interface/03/2026-09-09-runtime-interface-ui-activate-link-field-mismatch-return.md)
records that dependency. The remaining project and command-contract diagnostics retain
their respective owners. Fresh serialization execution, the exact Text09 reproduction,
independent review, formal closeout binding and fixed return remain pending.

### 2026-09-08 Serialization Regressions Executed

Managed Windows job `f95f64a6a06345d3940884140d9e3e50` used the exact derived input
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/interface-library-consumers-3137-20260908`,
digest `484b58e5512bb5619941864b4906bbfaaeef0cff78f89267586fe6e4b00d2b63`.
This input retains every serialization source byte and overlays Interface03, Editor08 and
App07 test-compilation repairs under their own snapshot owners. The full locked/static,
no-default-features interface library run compiled and executed 739 passing, 23 failing
and 101 ignored tests. Its `serialization::tests::*` subset executed 72 passing tests,
zero failures and one ignored 512 MiB streaming test. Direct Bool, enum variants,
numeric object keys, current typed payload and malformed binary tests all passed.
`results/interface-library-3137.{json,log}` retains names, commands and managed receipt.
The 23 failures are outside this serialization subset; the whole library is not green.
Text09 upward reproduction, independent source review and formal fixing-Session closeout
binding remain required. This lifecycle stays open.

The 2026-09-08 independent review of source 3129 and record 3145 returned
Critical 0 / Important 0 / Moderate 0. The preserved result is
`.codex/tmp/interface-app-editor-3137-review-20260908-result.txt`; it verified strict Bool
decoding, owned enum discriminants, map-key/payload boundaries and the original binary
regressions. Current/attribution/ObjectStore and immutable-input source hashes matched
before and after review, with no reviewer ownership conflict. Source review is complete;
Text09 upward reproduction, full-library failures and formal closeout binding remain pending.

### 2026-09-19 Rolling successor formal source binding

- Successor Session `failure-roll-01a084c8-editor11-binary-direct-decode-r1` reclaimed the
  archived exact-path ownership through coordinator transfer fingerprint
  `39b7feca8c2b84eebfdb09e02e09fae4336bd4ac5a86d3bac0a86aed375840bc` at baseline epoch
  `611`; no source bytes were changed during attribution.
- Formal non-Cargo source-contract ticket `77300940c68c460bb04b325bafbb8feb` was admitted
  from request `failure-roll-01a084c8-editor11-binary-direct-decode-20260919-r1` and is
  currently `queued`. Its sealed source-manifest hash is
  `a301ca95516dce7cc53c1680c586812f7ea34879012d5b55552c52962ba382ae`:

  | path | SHA-256 |
  | --- | --- |
  | `docs/plans/zircon_editor/editor/11/failure-2026-07-29-binary-direct-decode-serde-contract.md` | `b0b382eb870242002d1da6b29cb30655c6e7b977c4864a0ec430af7260686792` |
  | `zircon_runtime_interface/src/serialization/binary/value/direct_decode.rs` | `671ffedb482c4af713c8081e4c251b86ff3fa1ca1185c3541a293c252f6e1592` |
  | `zircon_runtime_interface/src/serialization/binary/value/mod.rs` | `3ee584707f2233ab607e135ced27c1b10640552a6a420a434773a04a98d5b41f` |
  | `zircon_runtime_interface/src/serialization/tests/binary_contract.rs` | `0f318e61b34bbba9cdf535d45b069016f18b67f7ad83166d3ff6f0db2f68a7f5` |

- The ticket executes a Windows PowerShell/rustfmt source-contract parse covering the full
  `Deserializer` bool implementation, owned enum discriminant, module export and binary
  regression anchors. It explicitly defers a fresh current-source interface Cargo gate, the
  exact Text09 cache reproduction, upward Editor11 gates, independent C/I/M review, canonical
  fixed return and closeout. Earlier serialization GREEN jobs remain supporting evidence only.
- Failure remains `open`; no fixed return, commit, or notification is claimed. The external
  `E:\Git\zr_vm` dirty-worktree blocker remains recorded for gates that require it.

### Ticket correction after source-contract assertion failure

- Prior ticket `77300940c68c460bb04b325bafbb8feb` reached its immutable command and failed only
  because the source parser expected an obsolete `direct_decode::decode` re-export. The current
  module exports `decode_binary_value_direct`; no source or behavior failure was observed.
- Corrected request `failure-roll-01a084c8-editor11-binary-direct-decode-20260919-r2` admits
  ticket `80b5c56a27e34c31be5820cb9895dd9f` with the actual export anchor. Its sealed manifest
  hash is `6690105bf28b110c06f6af070f7d4d659b42df0914c2d776d5b669f08e5897f6` and status is
  `queued`; the old ticket remains failed evidence and is not reused.

### Corrected source-contract ticket terminal result

- Ticket `77300940c68c460bb04b325bafbb8feb` remains failed evidence for the obsolete export
  assertion; it did not identify a source or behavior failure.
- Corrected ticket `80b5c56a27e34c31be5820cb9895dd9f` completed `passed` at
  `2026-09-19T05:01:00.869553Z` (exit code 0), with immutable Windows output
  `EDITOR11_BINARY_DIRECT_DECODE_SOURCE_CONTRACT_PARSE_PASS`. Its sealed manifest hash is
  `6690105bf28b110c06f6af070f7d4d659b42df0914c2d776d5b669f08e5897f6`. The result is limited
  to current-source parsing; interface Cargo, Text09/upward gates, independent C/I/M review,
  fixed return and closeout remain pending.

### 2026-09-21 Independent current-source review receipt

- Reviewer Session `review-editor11-binary-direct-decode-r1` claimed this failure document
  through coordinator request `bba3a57e0bcf41ce869ff88b87a62deb`; the three production/test
  files remained clean relative to the shared checkout and matched the corrected ticket
  manifest. The document is the only owned path with receipt drift.
- Current immutable source hashes at review time were:
  `direct_decode.rs` `671ffedb482c4af713c8081e4c251b86ff3fa1ca1185c3541a293c252f6e1592`,
  `value/mod.rs` `3ee584707f2233ab607e135ced27c1b10640552a6a420a434773a04a98d5b41f`, and
  `binary_contract.rs` `0f318e61b34bbba9cdf535d45b069016f18b67f7ad83166d3ff6f0db2f68a7f5`.
- Read-only checks passed: `rustfmt +1.94.1 --edition 2021 --config skip_children=true
  --check` over all three files; scoped `git diff --check`; and a source-contract probe
  confirming the `Deserializer` implementation, strict `deserialize_bool`, owned enum
  discriminant before the second decoder borrow, the current
  `decode_binary_value_direct` export, no `serde_json::Value` materialization, and the
  bool/enum, numeric-key, typed-current, wire-golden, and schema-error regression anchors.
  Probe terminal marker: `EDITOR11_BINARY_DIRECT_DECODE_SOURCE_REVIEW_PASS`.
- Independent findings: `Critical=0`, `Important=0`, `Moderate=0`. The review confirms the
  shared decoder is the lowest repair layer and found no compatibility shim, relaxed type
  acceptance, overlapping enum borrow, or stale module export in the current snapshot.
- This is a source-only review receipt. Fresh managed `zircon_runtime_interface` Cargo,
  exact Text09 cache reproduction, broader Editor11/upward gates, and the external
  `E:\Git\zr_vm` blocker remain unresolved. Historical green jobs and supervisor-only
  failures are not reused; canonical fixed return, review handoff, closeout, commit, and
  notification remain pending.
