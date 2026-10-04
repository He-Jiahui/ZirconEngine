---
handoff_kind: failure
status: open
created_at: 2026-10-03
summary_slug: native-sdk-owned-buffer-release-authority
origin_plan: docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/optimize/zircon_plugins/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
priority: 0
related_code:
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/Cargo.toml
  - zircon_plugins/gltf_importer/dist/src/lib.rs
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs
tests:
  - '.\tools\dev\local-cargo.ps1 +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --no-default-features --features native --lib native::owned_buffers::tests:: -- --test-threads=1'
  - .\tools\dev\local-cargo.ps1 +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --locked --no-default-features --features native --test native_owned_buffer_lower -- --test-threads=1
  - Native dist and fixture producer compilation; real Windows original-host save-state/free callback and retained image generation proof with exact artifacts.
---

# Plugins01: SDK V3 owned-buffer release authority

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md`
- 来源执行切片：P0-02 native SDK owned-buffer copied/forged release robustness, root rolling Failure continuation.
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：The SDK image owns the allocation and free callback. Host command sinks and RuntimeInterface allocation registries do not repair this V3 save-state buffer owner.

## 失败现象与复现证据

The pre-install reproduction `native.rs` SHA-256 was
`4c17ad181dfd9e9ac076245d6d158b81dd89eb37fd3ea2c180c81aaf22cade28`.
The reproduction SDK forgot the real Vec, derives a reversible owner token from
public metadata, and reconstructs a Vec using foreign carrier fields. There is
no authoritative consumed state. A copied descriptor can reach a second
reconstruction; altered metadata with a recalculated token can choose invalid
allocator parameters. This is static source evidence; no invalid free, crash
or unsafe deallocation was executed to reproduce it. The compliant host follows
the current unsafe function's unchanged-carrier and once-only preconditions.
This finding does not claim a demonstrated safe-Rust escape by that host.

The originating review used historical V2 language; the actual current carrier
is `NativePluginOwnedByteBufferV3`. A full current canonical scan found no exact
release-authority lifecycle. The existing
[native callback performance handoff](failure-2026-07-22-native-callback-per-call-lease-and-abi-copy.md)
addresses PERF541 leases/diagnostics and PERF542 output sinks, with different
acceptance. The separate
[entry unwind handoff](failure-2026-10-03-native-entry-ordinary-unwind-containment.md)
retains its own root and gates.

## 最低共享层根因

The authoritative live allocation must remain owned by the SDK image. A new
coherent `native/owned_buffers/` candidate retains the actual Vec under one
registry and replaces address-derived ownership with a checked, nonzero,
non-reused ID. Exact `(ID, address, len, capacity)` validation and one atomic
consume determine release; the actual Vec is dropped after the registry lock.
Unknown, consumed and mismatched carriers retain all live owners.

The Rust source API `owned_bytes` must return a Result without a second
infallible compatibility route. Registration errors retain the unpublished Vec
and expose the existing native failure status. Both actual producer callbacks
(glTF dist and native dynamic fixture) must publish only success or return that
failure. Descriptor ABI3, report layout5, behavior ABI4, carrier field order and
free callback routing remain unchanged. IDs are bearer capabilities, not
cryptographic secrets. IDs may restart after a new SDK image; no descriptor or
callback may outlive its original image.

## 架构修复验收

1. Keep the real allocation authoritative. No pointer decoding, foreign
   `Vec::from_raw_parts`, address-derived token or untracked forgotten Vec remains
   in this owner. Validate the exact tuple and consume once under one lock; drop
   outside it. Poison handling must retain records and the checked ID frontier.
2. Execute all ten real safe-core regressions: exact once-only consumption;
   copy/replay rejection; wrong ID/address/length/capacity preserving owners;
   legacy/unknown token rejection; non-reused IDs and exhaustion; real capacity
   overflow registration preserving the original Vec; competing releases;
   canonical and retained-capacity empties; malformed carriers; poisoned lock.
   These safety tests use the actual safe core, not invalid unsafe public free.
3. Execute three native-only downstream tests: valid callback payload lifetime,
   retained-capacity empty allocation and C field layout. Exact names/counts
   must be present; source availability, rustfmt and a zero-test filter do not
   pass this gate.
4. Compile both actual producer crates and current dist macros on Windows with
   Rust1.94.1 and Cargo `--locked`, bind complete manifests/locks/dependencies,
   toolchain/features/configuration and physical D/E/F cargo-targets outputs.
5. Validate original host save-state/free error propagation and the real Windows
   fixture DLL while the image generation is retained through the callback.
   Callback-vs-unload/reload, detached C layout/ABI and any property/sanitizer
   requirements in the originating audit remain separate actual gates.
6. Preserve all original business/comment bytes and the 56 audit-added lines.
   Review any combined entry/buffer postimage independently and retain separate
   lifecycle evidence; installing one full-file candidate must not overwrite
   the other's hunks or claim its acceptance.

## 禁止临时方案

Do not use the RuntimeInterface registry or V4 command sink as a substitute for
this SDK owner. Do not decode a token into allocator ownership, reconstruct a
Vec from foreign fields, silently free mismatched metadata, reuse allocation
IDs in one image, consume an owner on failure, add an infallible legacy wrapper,
or call a compatibility shim a repair. Do not execute invalid public frees or
UB to obtain a RED result. Do not infer DLL image retention, hostile-pointer
validation, concurrent read safety or OOM recovery from safe-core tests.

The source-comment receipt establishes comment authorship only; preserve
historical coverage and do not relabel product hashes as comment evidence.
Retired coordinator APIs and retained database rows do not grant adoption.

## 修复结果与回传

Status remains **open**. The unique
[private candidate receipt](../../../../.codex/tmp/failure-roll-20261003-sdk-buffer-lower-candidate/receipt.json)
SHA-256 is `f8cedf79e1115034f9c879801a16f79c20e434be48aa1730a71042764608d8f5`;
[design](../../../../.codex/tmp/failure-roll-20261003-sdk-buffer-lower-candidate/design.md)
records exact producers, scope and limitations. It covers seven existing and
six new paths. Rustfmt and author-side static checks are preparation evidence
only. This earlier candidate did not establish installation or dynamic acceptance.
The exact combined source installation and fresh source review are recorded below;
at that installation boundary the lower/producer/host/DLL gates were still open.
The current lower results are recorded in the next section.


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
