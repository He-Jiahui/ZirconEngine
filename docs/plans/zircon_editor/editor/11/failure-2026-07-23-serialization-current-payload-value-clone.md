---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: serialization-current-payload-value-clone
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/Cargo.toml
  - zircon_runtime_interface/src/serialization/load.rs
  - zircon_runtime_interface/src/serialization/migration/validate.rs
  - zircon_runtime_interface/src/serialization/write.rs
  - zircon_runtime_interface/src/serialization/text/canonical_writer.rs
  - zircon_runtime_interface/src/serialization/text/document.rs
  - zircon_runtime_interface/src/serialization/text/envelope.rs
  - zircon_runtime_interface/src/serialization/text/mod.rs
  - zircon_runtime_interface/src/serialization/text/read.rs
  - zircon_runtime_interface/src/serialization/tests/load_contract.rs
  - zircon_runtime_interface/src/serialization/tests/write_contract.rs
tests:
  - current text payload direct typed decode / legacy single Value migration contract
  - current text writer finite-float canonical single-owner contract
  - current and legacy 64MiB/128/2M/1M/16MiB serialization boundary fixtures
  - cargo test -p zircon_runtime_interface --locked --jobs 1 -- --test-threads=1
---

# Editor11：Serialization current payload Value clone热路径

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-570/571 serialization 41-file static review
- 修复责任计划：`docs/plans/zircon_editor/editor/11-serialization-and-versioning.md`
- 交接原因：`zircon_runtime_interface::serialization` 是 headless/runtime/editor 共用的 wire authority；current/legacy decode、header 和 canonical writer 不能由场景或编辑器 consumer 各自缓存/绕行。
- 生命周期键：`serialization-current-payload-value-clone`

## 失败现象与复现证据

当前 `load_versioned` 的 text 分支先将整份 bytes 解为 `serde_json::Value`，再将 envelope Value 反序列化，最后无论当前或旧版本都通过 migration `Value` 并 `from_value` 为 T。当前 `write_versioned_text` 先 `to_value(value)`，再创建 document Value、递归 canonicalize 整树，再 pretty-print；有限浮点 guard 又单独完整遍历一次。current payload 因而存在多次全树 materialize/clone，且 text 不具备与 binary 同等的硬 size budget。

## 最低共享层根因

版本壳没有区分“当前版本 payload 可直接 typed decode”与“旧版本必须进入单次 Value migration”的读取路径；text envelope 也没有 header-first borrowed payload representation。writer 以 JSON Value 同时承担 finite validation、canonical order 和 envelope 构造，导致同一事实多 owner 遍历。

## 架构修复验收

- Text reader 先以 borrowed header/payload 分离识别合法 `$zircon` envelope；schema/future version 在 payload decode 前 fail closed。
- Current version 的 payload 直接 decode 为 T，不创建 Value；只有无壳或旧 version 经一次 Value migration 后 decode T。业务对象含普通 `$zircon` 字段仍保持 v0 legacy 语义，不能因 envelope 探测误判。
- Text writer 用一次 finite-aware canonical serializer 直接构造 current envelope；删除 current writer 的 payload Value、document Value 和第二次完整 canonical clone，保持 canonical bytes、错误类型和尾换行。
- 为 text 加入与 binary 一致的显式输入/输出 budget；64MiB、128、2M、1M、16MiB 与 binary v1 golden、legacy fixtures、future/mismatch/malformed 均回归。
- 完成 focused interface Cargo、独立 review 与 current scene/editor upward tests 后，才返回此 failure；不接受静态扫描或 narrow fixture 替代规模门。

## 禁止临时方案

- Do not keep dual current readers/writers, consumer-local cache, or a compatibility Value fallback for current payloads.
- Do not decode unknown/future headers into T before validation, and do not classify arbitrary `$zircon` business fields as envelopes.
- Do not remove finite float, canonical text, legacy migration, binary golden, or size-boundary coverage to improve measurements.

## 修复结果与回传

Open state: `待修复`; the Editor11 plan remains the only owner of the direct current/legacy split and canonical writer hard cut.

## 产出记录与时间

- 2026-07-23 | PERF-MVP-570/571 source handoff | `open / routed-to-editor11` | Current `Value` materialization and duplicate canonical/finite traversals were established from source; no code change, Cargo result, or performance pass is claimed by this diagnostic record.
- 2026-07-23 | Text current-version direct decode | `open / source-and-static-review-complete` | Text now performs a borrowed `$zircon` header-signature probe, strictly validates claimed envelopes with raw payload references, validates schema/version and the complete migration chain, then decodes current payload directly into T. Only legacy/old versions materialize one `Value` for migration. Regression coverage fixes domain-owned `$zircon`, outer-field rejection, current-chain validation, malformed/future/schema behavior; scoped rustfmt and structural guards pass, independent review is `Critical/Important/Minor = 0/0/0`. Writer single-owner canonicalization, binary direct decode, size/performance gates and managed Cargo remain open.
- 2026-07-23 | Text current-version canonical writer | `open / source-and-static-review-complete` | Current text serialization now borrows T through the envelope and performs finite validation plus canonical object/key formatting in one serializer traversal; it no longer creates payload/document `serde_json::Value` instances or a second canonical clone. Struct/map keys remain lexically ordered, duplicate custom-map keys retain prior JSON `last-write-wins` semantics, numeric keys use JSON float spelling, and user serialization failures remain distinct from JSON encoding failures. Regression coverage includes duplicate, numeric, invalid map-key and custom-serialization cases; rustfmt, diff/static guards and final independent review are `0/0/0`. The writer remains a single responsibility at 923 lines; the next separate text-output responsibility must extract `text/canonical_writer.rs`. Managed Cargo, 64MiB boundary evidence, binary current direct decode and upward gates remain open.
- 2026-07-23 | Review-I3 contract hardening and owner extraction | `open / implementation-complete / managed-green-pending` | Independent exact-scope review rejected the first closeout at `C0/I3/M0`: duplicate reserved header fields could downgrade to legacy, wide integers split Text/Binary JSON domains, `RawValue` was silently rewritten, and `write.rs` had grown beyond the 1000-line production limit. Managed TDD RED job `2c337e6e39004e51aad199756800bee4` / run `ac4c1d1993fb4a74ad67777c433a5236` ran 48 serialization tests: 44 passed and the four expected contract/static guards failed. Current source propagates duplicate reserved-field errors as `InvalidEnvelope`, enforces the serde_json i64/u64 number domain, rejects the serde_json `RawValue` private marker consistently in Text/Binary, and hard-cuts the canonical serializer into `text/canonical_writer.rs`; `write.rs` is now about 345 lines and the new owner about 620 lines. rustfmt/diff-check pass. Snapshot 1074 / source fingerprint `c8c56655a31420fb0556781b52a2bf4c47cf65573ea1212a0665731ac90bb997` is queued for the same 48-test GREEN gate behind existing normal FIFO rows; no fixed return or commit is claimed yet.
- 2026-07-24 | Final formatting and current-source reservation refresh | `open / source-ready / managed-green-pending` | Two foreign Runtime04 full-lib runs compiled the Editor11 source without compiler errors but remained external RED (`21 passed / 22 failed`) on the Runtime04-owned asset-migration TOML-null contract. Their diagnostics exposed one Editor11-owned extraction warning (`Impossible` import) and a full Rust 1.94.1 edition-2021 rustfmt audit found three import-order differences. After both Runtime04 Rust jobs naturally released, the unused import was removed and all 12 owned Rust paths were formatted; scoped rustfmt check and `git diff --check` pass. Fresh exact11 reservation `607ba8f3af7a4063bf5800e57a6ea8cd` carries source-manifest fingerprint `b97d2b0370d95fd3b7f35e2098077b0f93a1b5ec01f4f7ff980384ac7c92d0c6`; full interface pre-attestation is 394 paths / `175220da80352393a0278813751baa7579d9530f70c000d7de7fe85488b20f4f`. Failure promotion remains correctly blocked until older normal Runtime04 reservation `59f3c41220a84d19bc8d844376e140c0` yields. No GREEN, fixed return, or commit is claimed yet.
- 2026-07-24 | Final current-source interface gate | `open / managed-green / independent-rereview-pending` | After the older normal FIFO barrier naturally released GREEN, reservation `607ba8f3af7a4063bf5800e57a6ea8cd` was promoted through the failure lifecycle and consumed as job `582c28106a8e4b34a582971895df201b` / run `1815a82090c24d9182f4da3d726bacc1`. Canonical Rust 1.94.1 ran the exact serialization filter: `48 passed / 0 failed / 262 filtered`, exit 0, with only the pre-existing `text/wire.rs::TEXT_ENVELOPE_KEY` dead-code warning. The exact11 hashes were unchanged after terminal, and the full 394-file interface compile-input attestation remained `175220da80352393a0278813751baa7579d9530f70c000d7de7fe85488b20f4f` from pre-start through terminal. Independent final rereview, failure return, and atomic commit remain pending; this record therefore does not claim fixed yet.
- 2026-07-24 | Final rereview acceptance gap | `open / C0-I1-M0-Minor1 / budget-tdd-red-queued` | Distinct reviewer Session/thread `019f8f4a-beb3-72d2-a3f5-d22837fdaf13` (no plan, scope, or leases) confirmed the ownership hard cut and Layout21 borrow repair, but correctly rejected failure closeout: the 48-test gate did not implement the required Text 64MiB input/output typed budget and did not prove current scene/editor upward gates. The Minor frontmatter owner inventory was updated for `migration/validate.rs`, `text/read.rs`, `text/canonical_writer.rs`, and `text/mod.rs`. Two boundary regressions now define the missing typed contracts before implementation; immutable exact14 reservation `f054089c4d414b89a895f155b04783f8` / fingerprint `e24825aa6f272062a4c517c2ad2a353ddd577079e94428c9104586d71420d55a` waits behind pre-existing FIFO rows for the managed RED compile. No implementation, fixed return, or commit is claimed yet.
- 2026-07-24 | Symmetric Text wire budget TDD | `open / focused-41-green / full-and-upward-gates-pending` | Managed RED reservation `f054089c4d414b89a895f155b04783f8`, job `4d1b6183afc5458c95dc12a9408ef022`, run `03a3bfd3b6ca4ee0a16ec7d66c468b1f` terminated exit 101 with the four expected missing-contract diagnostics: `MAX_TEXT_DOCUMENT_BYTES` E0432 x2 and `TextDocumentTooLarge` E0599 x2. The hard cut adds typed read/write errors, rejects Text input above 64 MiB before any parse, bounds canonical scalar writes and accumulated compound fragments during output construction, and accounts for the mandatory trailing newline; no post-hoc consumer limit or compatibility writer was added. Fresh exact14 reservation `2019918bab654278938afea1a8eaf6c1`, job `cca1b350d0014ef9bdf8a60ee6b23b44`, run `59e60977f00b46afb5e401d7c22b9e60` passed the `text_` focused gate `41/41`; both new 64 MiB contracts passed and all post-source hashes matched fingerprint `8cf12d1a86850be15af1f991fcf8ff6a6122578a55312254a9c84cc8db9c650c`. Full serialization and current scene/editor upward gates remain pending, so the failure stays open.
- 2026-07-29 | Full interface diagnostic and current-source recovery | `open / implementation-repaired / managed-rerun-pending` | Canonical Rust 1.94.1 managed job `3bb2a442c76748bab54b8fdb21408a9f` / run `73795d6cf598446f9e4e8e26dbf18ad2` compiled the complete current interface and ran 334 tests: `327 passed / 6 failed / 1 ignored`. The current-payload binary/text and bounded-streaming tests passed. Two failures were Editor11-owned: the source guard selected the owned adapter instead of `write_versioned_text_to`, and Binary accepted serde_json RawValue through its newtype marker. Current source fixes both without restoring a Value fallback, removes two compiler warnings, and records the bounded canonical sorting spool as an exact single-file/three-primitive boundary exception. Scoped rustfmt, diff-check and static contracts are green; fresh managed focused/full gates, independent review, upward Runtime04/Runtime11 gates and fixed return remain pending.
- 2026-07-30 | Fresh focused RawValue representation recovery | `open / implementation-repaired / fresh-rerun-queued` | Canonical Rust 1.94.1 job `8ee08a1a4f284da6af298ff588b39bf0` / run `2f90cac80bf24728adcd6615c5d4d756` compiled current source and ran 59 focused serialization tests with stable 401-input pre/post fingerprint `2b05711471b4faccb4cf913d87467049502fd4073b2b4b50f2418daa75519e5f`: `57 passed / 1 failed / 1 ignored`. The sole failure proved serde_json RawValue reaches canonical Text through its private struct marker; current source now rejects both its struct and newtype forms at the canonical serializer boundary. No current-payload Value fallback or compatibility path was restored. Fresh exact6 reservation `67ab96ef20e74ea79b9cf139e5d37fb6` waits behind the existing Shader06 row; full GREEN, upward scene/editor gates, independent review and fixed return remain open.
- 2026-07-30 | Fresh focused current-source GREEN | `open / focused-green / boundary-and-upward-pending` | reservation/job/run `67ab96ef20e74ea79b9cf139e5d37fb6` / `76fdfb36f35c413192a520565669c34a` / `c3bbf7083c3b4139849b91ec4cee5621` passed `58/58` active serialization tests with one explicit 512 MiB ignored gate. Full 401-input pre/post fingerprint remained `34c8e35654571d46d008c2ec56438f49015b1ff2a245b6af062e924277ea3326`; exact source hashes were unchanged. The atomic delivery scope is now the complete exact31 dirty serialization slice rather than an incomplete new-module subset. OS boundary, 512 MiB, scene/editor upward gates, independent review and fixed return remain open.
- 2026-07-30 | Current-source bounded writer acceptance evidence | `open / serialization-and-high-capacity-green / upward-pending` | The current exact32 source passed the broader serialization gate `62 passed / 0 failed / 1 ignored`, the extracted spool OS boundary `1/1`, and the explicit 512 MiB fixed-staging gate `1/1` in 17.92s. Their full 402-file compile-input pre/post fingerprint remained `fa51e11fded0881cd4c641fb5c41a0250265fd04afcbe710844037ecfae0aeaf`. Independent snapshot1313 interim review is `C0/I0/M0/Minor0` and confirms the direct current decode, legacy-only `Value` migration, pre-accounted map keys, chunked Display escaping, and private spool owner. Full interface and scene/editor upward gates plus a final post-document exact32 review remain open; no fixed return is claimed yet.
- 2026-07-30 | Full current Interface acceptance | `open / full-interface-green / upward-pending` | Fresh reservation/job/run `3b301ffebcb54fb881e02291fe40e2d4` / `ac76251fad174b18a80bc3dbaa1745a3` / `dd5e4239cf664f038f51934dc38292a4` passed the complete package: unit `338 passed / 0 failed / 1 ignored`, integration `3/3`, doc-tests `0`. Full 402-input pre/post fingerprint was `18a0b4f7ad2a3f6e0255e76a44f8080b9c067cb8e78d8bb189327cec732d962a`. Text09 and Dynamic Scene upward gates, final expanded-scope review, fixed return and atomic commit remain open.
