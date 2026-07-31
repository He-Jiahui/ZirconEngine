---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: canonical-text-streaming-output
origin_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
fixing_plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
origin_child_dir: docs/plans/zircon_runtime/runtime/11
fixing_child_dir: docs/plans/zircon_editor/editor/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/serialization/write.rs
  - zircon_runtime_interface/src/serialization/text/canonical_writer.rs
  - zircon_runtime/src/scene/dynamic_scene/session/construction/serialization.rs
  - zircon_runtime/src/scene/dynamic_scene/session/io/load_save/save.rs
tests:
  - canonical text streaming writer byte-equivalence and bounded-buffer contracts
  - cargo test -p zircon_runtime_interface --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_runtime --lib dynamic_scene_session --locked --jobs 1 -- --nocapture --test-threads=1
---

# Editor11: canonical text streaming output

## 来源执行者

- 来源计划: `docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 来源执行切片: Runtime11 dynamic scene session bounded asynchronous I/O prerequisite audit
- 修复责任计划: `docs/plans/zircon_editor/editor/11-serialization-and-versioning.md`
- 交接原因: `zircon_runtime_interface::serialization` owns canonical text wire construction. Runtime04 supplies a sealed archive artifact and Runtime11 owns the I/O lane, but neither can replace the shared canonical writer with a consumer-local serializer.
- 生命周期键: `canonical-text-streaming-output`

## 失败现象与复现证据

Current `write_versioned_text` returns a complete `String`, and `write_versioned(Format::Text)` converts that string into a complete `Vec<u8>`. The Dynamic Scene session save path therefore must first materialize the whole canonical payload before `fs::write` can begin. This violates the Runtime11 Dynamic Scene I/O requirement for bounded streaming writes at 1/64/512MiB and leaves Runtime04's sealed archive artifact unable to reach disk without a full-text resident copy.

The existing `serialization-current-payload-value-clone` handoff has already removed current-payload `Value` materialization and duplicate canonical traversals, but it intentionally retains the full-text return contract. It does not provide an `io::Write`-backed canonical output boundary for the Runtime11 file lane.

## 最低共享层根因

Editor11's canonical text writer has one output representation: an owned complete text document. Canonical ordering, finite-float validation, envelope framing, output-byte accounting, and sink ownership have not been separated behind a bounded streaming writer, so every downstream file writer is forced to inherit the full `String` allocation.

## 架构修复验收

- Editor11 exposes one finite-aware canonical text encoder that writes the versioned envelope directly to a bounded `io::Write` sink while preserving byte-for-byte canonical output, typed errors, the trailing newline, header-first validation, and legacy migration semantics.
- The writer keeps a fixed-size bounded staging buffer and reports write failures without retaining a second full document. `write_versioned_text` may remain only as an adapter over that one encoder for callers that explicitly require owned text; Runtime04/Runtime11 save paths must consume the streaming boundary.
- Focused interface coverage proves canonical byte equivalence, partial sink failures, output-budget rejection before unbounded accumulation, finite-float failures, duplicate/numeric map-key behavior, and 1/64/512MiB bounded-buffer behavior.
- Editor11's focused interface gate, Runtime04 immutable archive artifact gate, and Runtime11 Dynamic Scene session I/O gate all pass before this handoff returns fixed.

## 禁止临时方案

- Do not add a consumer-local JSON serializer, a `BufWriter` around an already complete `String`, a second canonical writer, or a compatibility fallback that restores the full payload for large writes.
- Do not weaken canonical byte, legacy migration, finite-float, typed-error, or size-boundary coverage.
- Do not move serialization, fsync, or file publication onto a caller/owner tick as a substitute for a streaming shared writer.

## 修复结果与回传

Open state: `待修复`; no Runtime04 archive or Runtime11 Dynamic Scene I/O pass is claimed until the Editor11 streaming writer and upward gates are accepted.

## 产出记录与时间

- 2026-07-29 | `open / implementation-repaired / managed-rerun-pending` | 单一 canonical writer 已使用 64 KiB 固定请求、磁盘排序 spool 和直接 `io::Write` 输出；owned String/Vec 仅保留为显式 adapter。受管完整接口诊断 `3bb2a442c76748bab54b8fdb21408a9f` / `73795d6cf598446f9e4e8e26dbf18ad2` 运行后，本 owner 的 streaming/size/canonical 回归除一个错误的静态函数切分锚点外均通过。当前源将锚点硬切到 `write_versioned_text_to`，补齐 Binary RawValue newtype 拒绝，并把排序 spool 的 `std::fs/std::process/std::sync` 使用限定为 `canonical_writer.rs` 的精确受审例外；全局接口边界禁令及反扩散扫描保持生效。fresh managed gate、512 MiB ignored gate、Runtime04/Runtime11 upward gates、独立复审和 fixed return仍待完成。
- 2026-07-30 | `open / lowest-owner-repaired / fresh-rerun-queued` | fresh focused job `8ee08a1a4f284da6af298ff588b39bf0` / run `2f90cac80bf24728adcd6615c5d4d756` 在 401 输入无竞态条件下运行 59 项：`57 passed / 1 failed / 1 ignored`。唯一失败是 Text `RawValue` 当前通过 serde `serialize_struct` 表示，而 canonical writer 仅拒绝同名 `serialize_newtype_struct`；当前源现以同一 typed `PayloadValidation` 拒绝两种私有表示，不引入第二 writer 或 Value materialization。fresh exact6 reservation `67ab96ef20e74ea79b9cf139e5d37fb6` / fingerprint `3702365a2b7cbc1e049a225605fa7a7d37bfbf7a7bf52ce83e362b80f00bb0b0` 已按 FIFO 排队；GREEN、512 MiB、OS 边界与 Runtime04/Runtime11 上行仍待真实证据。
- 2026-07-30 | `open / focused-green / boundary-and-upward-pending` | 后继 reservation/job/run `67ab96ef20e74ea79b9cf139e5d37fb6` / `76fdfb36f35c413192a520565669c34a` / `c3bbf7083c3b4139849b91ec4cee5621` 自然 GREEN：`58 passed / 0 failed / 1 ignored`，401 compile-input pre/post 均为 `34c8e35654571d46d008c2ec56438f49015b1ff2a245b6af062e924277ea3326`。唯一 ignored 是显式 512 MiB 高容量门；OS 精确例外 reservation `625c78b0139644f3bc0d436f1d912940` 已按 FIFO 排队。该 failure 仍等待 OS、512 MiB、Runtime04/Runtime11、独立复审与 fixed return。
- 2026-07-30 | `open / preaggregation-tdd-green / broader-and-upward-pending` | 独立复审的两个 Moderate 已按真实 RED→GREEN 闭环。job/run `26e6c31ba5a34376b574b6844936f1ba` / `8ee93db3dc2c4d4b8a04e0b071e01f34` 为 `1 passed / 3 failed`，证明旧实现对 RawValue map-key 使用错误类型且在预算拒绝前生成全部 Display 块/全部 map keys；完整 401 输入前后均为 `aba7afb92e8d5470d019b8225fe6d0edbd5f0a0439b9f8d119d6a11709dd5353`。修复把 Display escaping 直接接到 budget sink、map key 预记账并保持 duplicate last-write-wins，同时将 107 行磁盘 spool owner 硬切到 `canonical_spool.rs`。后继 job/run `e0da75c54cc54691ba18c340efb73f45` / `7ceac55b1a924bbf9078e8f3e6490b48` 为 `4 passed / 0 failed`，完整 402 输入前后同为 `fa51e11fded0881cd4c641fb5c41a0250265fd04afcbe710844037ecfae0aeaf`。failure 继续等待完整 serialization focused、OS、512 MiB、Runtime04/Runtime11、fresh review、fixed return 与受管提交。
- 2026-07-30 | `open / current-source-high-capacity-green / upward-pending` | 完整 serialization job/run `be60c135d75d4b57a8ff09d10e9ef21d` / `e581979bac5f4bfd8633c393cfd7982e` 为 `62 passed / 0 failed / 1 ignored`；OS owner 精确门 `9a3335b0bbcc4dfd88561610f95d4f4a` / `1b33af49ee944731bc504e1a2c48ad7b` 为 `1 passed / 0 failed`；显式 512 MiB job/run `9a7065ea22ed4b1e8e79f6fc9d1d79e6` / `d6ff0cdf9e2a4beca5e27f93e9160754` 为 `1 passed / 0 failed`。三项门禁的 402 路径 pre/post fingerprint 均为 `fa51e11fded0881cd4c641fb5c41a0250265fd04afcbe710844037ecfae0aeaf`。snapshot1313 interim review 为 `C0/I0/M0/Minor0`、`Ready`，但完整接口包、Runtime04/Runtime11 上行、最终 fresh exact32 review、fixed return 与受管提交仍未完成，failure 保持 open。
- 2026-07-30 | `open / full-interface-green / runtime-upward-queued` | Fresh full-package job/run `ac76251fad174b18a80bc3dbaa1745a3` / `dd5e4239cf664f038f51934dc38292a4` passed unit `338/0/1`, integration `3/3`, doc-tests `0`, with complete 402-input pre/post fingerprint `18a0b4f7ad2a3f6e0255e76a44f8080b9c067cb8e78d8bb189327cec732d962a`. Runtime consumer self-review then found and repaired a preservation regression: direct save now reuses the sole atomic staged writer, so a typed non-finite serialization error cannot truncate an existing archive; one focused regression locks target preservation and temp cleanup, and the duplicate 64 KiB policy constant is gone. Dynamic Scene reservation `4e72ea910ac14061b318c12620ba2d12` is queued; Runtime11 acceptance, final review, fixed return and commit remain open.
