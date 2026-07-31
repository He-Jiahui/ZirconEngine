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
