---
handoff_kind: failure
status: open
created_at: 2026-10-03
summary_slug: native-entry-ordinary-unwind-containment
origin_plan: docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/optimize/zircon_plugins/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
priority: 0
related_code:
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/Cargo.toml
  - zircon_plugins/plugin_sdk/src/dist.rs
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
tests:
  - .\tools\dev\local-cargo.ps1 +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --no-default-features --features native --test native_entry_guard -- --nocapture --test-threads=1
  - Windows real native fixture DLL runtime/editor entry-panic rejection followed by normal entry, callback and teardown; bind SDK, fixture, host and artifact hashes.
---

# Plugins01: ordinary Rust unwind at the exported entry boundary

This canonical lifecycle is open. The reviewed source repair is installed;
matching dynamic validation remains pending.
Its stable slug retains the existing private draft identity. The canonical
creation date is October 3, 2026; the author audit observation on October 2
remains historical evidence. Other Plugins01 failures retain their own lifecycles and acceptance
conditions; they do not substitute for this entry-unwind boundary.

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md`
- 来源执行切片：P0-04 source audit, SDK native runtime/editor entry-point panic boundary; root rolling Failure continuation.
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：The exported SDK C entry is the lowest shared owner of ordinary Rust ready-hook and entry-expression unwind containment; the host cannot catch a panic that crosses this non-unwinding boundary first.

## 失败现象与复现证据

The originating optimize review records P0-04. Current
`zircon_plugins/plugin_sdk/src/native.rs` has SHA-256
`4c17ad181dfd9e9ac076245d6d158b81dd89eb37fd3ea2c180c81aaf22cade28`.
`NativePluginEntryPointV3::entry_report` calls the ordinary Rust `on_host_ready`
hook at lines 449-450. `export_native_plugin_entry_v3!` directly evaluates the
entry expression and calls that method inside a non-unwinding C function at
lines 716-720. The existing behavior callback guard has a different call path.
A caller-side catch cannot contain a panic that already crosses this C boundary.
These statements are source evidence; no abort or DLL execution was performed
by this design task.

The actual producers are SDK dist macros, which export both runtime and editor
entrypoints through this macro. The real native fixture uses nonempty ready
hooks. The host calls the Library symbol at
`native_plugin_abi.rs:172` and rejects a null entry report at lines 174-181 before
dereferencing it. V3 has no entry status field. This existing null rejection is
the minimal failure contract; a diagnostic-looking nonnull report must not
stand in for rejection.

## 最低共享层根因

The minimal repair catches evaluation of the entry expression and the complete
`entry_report/on_host_ready` call inside the exported C body. The public,
doc-hidden SDK helper is visible to downstream macro expansions. A normal
report pointer is returned unchanged; an ordinary unwind returns null. Keep
descriptor/entry ABI 3, report layout epoch 5, behavior ABI 4 and their layouts.
Do not use C-unwind or replace the process-global panic hook.

The prepared entry-only patch and actual downstream test draft remain at
`.codex/tmp/failure-roll-20261001-plugins01-entry-guard-candidate-sol/`.
Their prior independent static review is C/I/M 0/0/0. The current preimages still
match that candidate; it has not been installed or dynamically validated.

## 架构修复验收

Acceptance requires the root validation lane to preserve the source/command/
toolchain/profile/artifact identity and actually execute the regression. Under
the original macro the combined panic test reaches the runtime ready hook
first, so a single abort proves only that path. Editor hook and expression
coverage are demonstrated by GREEN or by separately filtered subprocess RED
cases, not inferred from the first abort. The native-only target must have
explicit required-features metadata and run two tests, without ignored or zero
test success. Unwind profile is required and abort configuration must be
reported as unsupported containment. The real Windows DLL follow-up must
exercise runtime and editor exports through the actual loader, observe typed
null rejection and prove a later ordinary load/callback/teardown succeeds.

Panic payload disposal is also a boundary: the current candidate contains a
second unwind while dropping the caught payload and explicitly aborts on a
panicking destructor. That exceptional payload is outside ordinary recoverable
panic containment. It must never be logged as a successful recovered entry.
Containment does not roll back ready-hook side effects. Panic=abort, a panicking
panic hook, OOM abort, foreign faults, invalid pointers and unloaded-image calls
remain outside this repair's promise.

## 禁止临时方案

Do not replace the process-global panic hook, use C-unwind, change ABI layouts,
return a nonnull diagnostic report as successful admission, remove current
source-audit comments, or treat static review as an executed test. Do not widen
this lifecycle to claim owned-buffer, signed-package or DLL lifetime acceptance.

Ownership must be settled using active chat/diff evidence and an explicit exact
source handoff. The current native whole-file bytes include preexisting changes
and source-audit comments. Historical attribution and an empty old lease sample
do not establish their business author. Existing audit provenance and comment
bytes must be retained. The previously accepted author-side boundary message
01a0f90c-1591-7ec0-8de4-0b080c7c3826 remains a request receipt, not consent.

The local coordinator is retired. Its registration, lease, attribution, ticket
and closeout APIs are not execution prerequisites and must not be restored.
Independent local Windows command evidence uses the current local-cargo wrapper;
compilation products and caches must physically reside below drive-root
D/E/F:\cargo-targets. Local results do not grant Jenkins acceptance; the actual
regression, independent review and product gates remain open until executed.

## 修复结果与回传

This entry lifecycle does not close owned-buffer release robustness, borrowed
slice safety, signed-package authority, native callback performance or ABI
inventory. Signed project admission remains a distinct fail-closed availability
follow-up under its owning plan. No failure return, commit or notification has been performed for this lifecycle.
The exact source installation is recorded below. This record publishes the proven static root cause and open
acceptance gates; it does not promote static evidence to a dynamic pass.


## Exact audit boundary reception

Root received the author-side terminal boundary at `2026-10-03T01:44:28+00:00`
(October 2 in America/Los_Angeles). The [terminal audit receipt](../../../../.codex/tmp/source-audit-20260930/sdk-entry-source-owner-natural-boundary-20261002-r1.json)
has SHA-256 `882492217d1fa28d4806300c5bfdae6fb858ca3e3f5e40c58be0227a46d1b733`.
The [independent reception review](../../../../.codex/tmp/failure-roll-20261003-sdk-entry-boundary-review/review.json)
has SHA-256 `b6820143dd578c6332ea6986a927edcaba91baa5131bc7587cded4557b19aec6`.

All 56 added documentation lines (4,786 bytes) match recorded raw bytes and
positions. Removing only those lines recovers the immutable pre-audit SHA-256
`1def13ffee21a260fa2878278d0c9c647c861896c9bf06beea12e67c31ec219a`.
Fifty lines contain Han characters; six are blank documentation or Safety
heading lines. All preexisting business and comment bytes are preserved.
The manifest has zero audit delta. Coverage rows and child review/result hashes
match their historical receipts.

This receipt establishes comment authorship and audit completion only. It grants
no whole-file business adoption or DLL/test acceptance. Root will preserve that
provenance, recheck exact preimages before an authorized repair, and record only
the precise product delta. The old coordinator remains retired.


## 2026-10-03 independent source installation; validation remains open

The rolling Failure task installed the exact reviewed combined fourteen-file
product delta on October 3, 2026. The [Root installation receipt](../../../../.codex/tmp/failure-roll-20261003-sdk-combined-root-install-r1/terminal.json)
has SHA-256 `f31b96b664747778304f7c25be32a5d81c8407cc9c9c718b47975bfb216fc58c`.
The [combined independent source review](../../../../.codex/tmp/failure-roll-20261003-sdk-combined-independent-review/review.json)
has SHA-256 `c4d3bc70502474f6cba8d60409aeddbfa046d046c21291a56ae77068eb658acd`
and Critical/Important/Moderate = 0/0/0. This qualification is source-only.

Seven existing paths changed and seven new paths were created. Root owns only
the recorded business delta, including both direct producer adaptations;
unknown baseline authorship and all historical comment coverage remain separate.
All 56 audit lines (4,786 bytes) are retained exactly. The current SDK native
postimage is `d2a8061205f32f8107a75fb623a81665434c24f3d76dc3e076680fb1e199f327`;
its manifest is `07cfb028d83cef73b8c7b57565b867aa0b9de8b1a1f4672edc1af1aadefd4f25`.
The root Cargo.lock remains byte-for-byte at
`c1016294d0e616a7ae9dcee930480a9e68cfc73d8128b8ea38533e1019f4722c`.
The pre-install reproduction and earlier candidate records above are historical
inputs; they are not current source or validation passes.

The [existing-root-lock lower validation plan](../../../../.codex/tmp/failure-roll-20261003-sdk-existing-lock-lower-validation-plan-r1/plan.json)
separates ten owned-buffer core tests, three public valid-buffer tests, two entry
panic tests and one actual distribution-macro test. Its current independent
[entry route diagnosis](../../../../.codex/tmp/failure-roll-20261003-sdk-current-validation-entry-route-r1/receipt.json)
preserves Rust 1.94.1, Cargo --locked, exact filters and features. At the
installation boundary, no Rust test had executed on that snapshot. The approved
D/E/F roots were all below
the local Cargo 35 GiB free-space reserve at the
[storage preflight sample](../../../../.codex/tmp/failure-roll-20261003-neural12-original-audit-root-preflight-failure-r1/terminal.json).
Existing artifacts were preserved. The separate fixture lock decision remains
pending; it is not a blanket dependency of these existing-root-lock tests.

This lifecycle remains **open**. Both direct producer checks, original-host
save/free and error propagation, real Windows DLL/ABI/image retention, and any
originating property/performance gates still require actual matching evidence.
No canonical fixed return, commit SHA, notification, push, Jenkins milestone or
whole-workspace acceptance is claimed.
## 2026-10-03 independent Windows lower batch: check and 16 tests passed

The [actual batch receipt](../../../../.codex/tmp/failure-roll-20261003-sdk-root-lock16-actual-batch-r2/terminal.json)
binds the existing Root workspace/lock, the installed SDK sources, the native
feature profile and all five commands. Windows Rust/Cargo 1.94.1 used
`--locked --no-default-features --features native --jobs 1` throughout.
The package `check --all-targets` exited 0. The focused tests actually executed
10 owned-buffer core cases, 3 public valid-buffer cases, 2 ordinary entry-panic
cases and the original `dist::tests::dist_plugin_one_file_export_compiles` case:
**16 passed, 0 failed, 0 ignored**. Exact test IDs and raw logs are retained.

The [independent actual evidence review](../../../../.codex/tmp/failure-roll-20261003-sdk-root-lock16-independent-actual-review-r2/review.json)
has Critical/Important/Moderate = **0/0/0**. It accepts this lower batch only.
The [frozen input receipt](../../../../.codex/tmp/failure-roll-20261003-sdk-root-lock16-source-capture-r2/terminal.json)
binds 1,007 files, including the 17 actual embedded project templates. All
private hashes and original input bookends matched after execution; the Root
lock remains `c1016294d0e616a7ae9dcee930480a9e68cfc73d8128b8ea38533e1019f4722c`.
Compiler products, build directories, Cargo caches and temporary outputs use
physical D:\cargo-targets paths. No whole-tree quiet or transient-ABA exclusion
is inferred. Named Rust tool images matched before/after; Python's prelaunch
and post-batch content hashes matched, with no claim of a recorded initial
Python file identity or per-stage image observation.

The [first check failure](../../../../.codex/tmp/failure-roll-20261003-sdk-root-lock16-package-check-r1/terminal.json)
remains preserved: Cargo exit 101 reported 17 missing template inputs in the
first private copy. The second capture corrected that validation input closure,
without changing production code, a workspace manifest or either lock. Native
Cargo locks and compatible compiler caches were retained; old failure receipts
were not reused as passes.

This lifecycle stays **open**. The [current direct-producer plan](../../../../.codex/tmp/failure-roll-20261003-sdk-direct-producer-current-preparation-r1/plan.json)
keeps both producer checks and the original glTF state test on their real
plugins workspace route; neither producer is in the actual Root member set.
Those checks, original host save/free/error propagation, real Windows DLL
image retention and the originating unsafe/ABI/product gates still require
matching execution evidence. The specific standalone DLL fixture lock decision
remains pending. This lower pass grants no fixed return, full fourteen-file
compiled claim, commit, notification, push or Jenkins acceptance.
