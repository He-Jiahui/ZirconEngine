---
status: in_progress
plan_sources:
  - docs/plans/astra/features/01-product-correctness.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
---

# Pack Promotion Recovery

## Historical Failure And Current Evidence

The original implementation forwarded the default `backup_pack = None` request through a
rename-and-cleanup path that could delete the working installed pack after a failed cross-volume
promotion. It also had no restart entry point after moving the old pack to an optional backup.
The current source no longer contains that path: the native hot-update entry calls
`ZrPackDeltaInstaller::install_delta`, which recovers the bounded promotion journal before
reading or rebuilding the base, publishes backup and installed bytes through the shared durable
transaction engine, and retires staging only under its captured digest. A committed publication
without a receipt resumes as `AlreadyInstalled`. The canonical finding is therefore
`implemented_pending_validation`; focused Cargo and product execution remain outstanding.

## Implementation

1. Add a regression using the existing injected rename-failure path with an existing installed
   pack and no requested backup. Preserve the installed bytes on failure.
2. Prepare the candidate beside the installed pack, validate its bytes before publication, and
   use the runtime atomic-write primitive to publish it without a missing-destination window.
   Retain requested backup bytes before replacement. Never remove an installed destination in
   response to a failure that did not publish a replacement.
3. Reuse `zr_resource`'s existing durable transaction engine, bounded append-only journal,
   digest-checked retirement and restart replay. Stage writes to the optional backup and the
   installed pack as one transaction; retire the staged source only under its captured digest.
   Recovery policy admits exactly the requested installed/backup/staged paths. Expose recovery
   on the installer and invoke it before staging in the native hot-update path, as well as
   before promotion. No new journal format or competing recovery state machine is introduced.
4. Add restart fixtures at prepared-journal, published-pack, and cleanup boundaries. Preserve
   old/new valid data on all failures and keep explicit backup semantics compatible. Exercise
   both no-backup and requested-backup paths and corrupt/inconsistent journal rejection.

## Ownership And Validation

Owner: asset implementation lane `astra-source-integrity-20260905`, parent
`astra-full-domain-20260905`. Scope is `asset/pack/install/{promotion,installer,error,mod}`,
coherent new promotion journal/recovery modules, and focused pack promotion tests. No World/ECS
edits. Existing dirty files are baselined and retained. Rustfmt/source checks run locally; all
Cargo/build/product regression execution goes through the parent's coordinator batch.

## Status

Source review confirmed the production default-None call chain. Exact six-file initial lease
receipt `11f97ae099b64ae6b8b901f44329766e` and report/tests lease
`1761237adb26491d9f8c798d7bc23ac1` acquired without conflicts. The existing auxiliary source
session registration still references the obsolete plan06 in coordinator state; new registration
request `c44f19588579404cbb96c5ff0a0d4166` was accepted without a terminal response. Parent has
the handoff. No Cargo or executable regression has run.

## Candidate Outcome

The implementation now uses the existing durable transaction engine for backup, installed target,
and digest-checked staged retirement. `install_delta` is the production composition entry:
recover pending publication before reading/rebuilding the base, or verify the installed target and
report `AlreadyInstalled` after a committed update. This allows a missing receipt or interrupted
plugin reload to resume without reapplying a delta to its new base or overwriting the prior backup.
Receipt writes are atomic and reject aliases of pack inputs/destinations. Canonical path and
hard-link aliases among staging, installed and backup destinations are rejected before mutation.

Native hot-update owner handoff received from `audit_plugin_closure`; exact production lease
`4c350dd560d34eeeb9d4c87b0d29d9e9` and test lease `e31fdf03ddf24f1582be7ca34cd4ea3e` acquired
without conflicts. Existing loader artifact-authority changes are preserved.

Focused tests cover failed commit with no backup, restart after staging/target replacement/source
retirement/per-file commit, restart after durable commit/cleanup, recovery-policy mismatch, corrupt
journal, aliases, automatic retry and receipt recreation. The real native hot-update regression
also retries after receipt removal and checks old backup bytes plus plugin discovery. Rustfmt and
scoped `git diff --check` passed. Independent source review and coordinator Cargo validation remain
pending; no test result is claimed.

The current continuation also closes a retry receipt inconsistency: an `AlreadyInstalled` result
with a requested backup now requires that backup to exist, decode as a valid pack, and match the
delta base manifest. Retries cannot claim a newly supplied, corrupt, or unrelated backup without
mutating the installed target. Focused source regressions cover missing, corrupt and wrong backup paths;
Cargo execution remains assigned to the parent validation lane.

## Source Candidate Hashes

```text
E:/Git/ZirconEngine/zircon_runtime/src/asset/pack/install/promotion.rs CBC91471EDC042A92D0533D5754E714CCD08832C8BA461D9352272A382F1125E
E:/Git/ZirconEngine/zircon_runtime/src/asset/pack/install/promotion_journal.rs 0DAC4C1047B70A0F4295AAE8433E47BD9D3217EBB2E4E06983C3F66FB51604EA
E:/Git/ZirconEngine/zircon_runtime/src/asset/pack/install/promotion_tests.rs D9518EDD25FCAE1F730C0C0486066CB4172710104F636C1E293FD53EA4AC5FD0
E:/Git/ZirconEngine/zircon_runtime/src/asset/pack/install/delta_workflow.rs AA4E034DAE958A9BA1A9C83D7F829D920AC0AF31AB8714E6C00147D84FB18613
E:/Git/ZirconEngine/zircon_runtime/src/asset/pack/install/installer.rs 702E0AE19FDC9D3C945A24243E806E9744D25840491B4DEFDA4AC28086C03AE2
E:/Git/ZirconEngine/zircon_runtime/src/asset/pack/install/mod.rs 8860E34BB4942A159E5B928612FE56A156059631276D5229D0D823BA75CFF1B4
E:/Git/ZirconEngine/zircon_runtime/src/asset/pack/install/error.rs 9A7E2D42A6F2542FF398121B330A3D8398E22331B6801CD1A8E96BB02E743312
E:/Git/ZirconEngine/zircon_runtime/src/asset/pack/install/promotion_report.rs 94F4AB09FDCB901C46E5BEB7B4A49FB449C7CF2858D91798333719B8090C118A
E:/Git/ZirconEngine/zircon_runtime/src/asset/pack/install/receipt_io.rs 757594733AA68DBE3754E34FACEF3800795FD374A656CAE52A99A196661231C6
E:/Git/ZirconEngine/zircon_runtime/src/asset/pack/install/file_io.rs 2844B87B11E2CF48EFE73BF7DEDF65D3A20B550495F9D903CADB4D04B832FA21
E:/Git/ZirconEngine/zircon_runtime/src/asset/pack/reader.rs 890D3D270B2401D1D7C9302DF02F41B6AE2B2D2D12588F383E47543E8BE52A22
E:/Git/ZirconEngine/zircon_runtime/src/asset/tests/pack/delta_installer.rs 96DA0E4833367E59E3AAF469BCDCEC42A8812F7D50C7C84CE583F6E2A9B2A2B1
E:/Git/ZirconEngine/zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_update_application.rs 1E85464118D26151AFDE86F044355009A4AE06D21A83F062E50F8ACE1F4802FC
E:/Git/ZirconEngine/zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_update_application.rs 96953927230E3D73989BE101B3C8D8C9CA428223D47CA732D923E145515720FF
E:/Git/ZirconEngine/docs/crates/zircon_runtime/asset/pack.md 0A963C28F928BAEFD43E68C924BB452F935089CB4549CA68C45B1020D75B7407
```
