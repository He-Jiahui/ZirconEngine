---
handoff_kind: failure
status: fixed
created_at: 2026-08-31
summary_slug: managed-cargo-sccache-temporary-path-lifecycle
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_app/08
plan_link_mode: child_record_only
related_code:
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
  - .codex/skills/zircon-dev/scripts/managed-cargo-storage.ps1
  - .codex/skills/zircon-dev/scripts/managed-cargo-storage.Tests.ps1
tests:
  - managed Cargo sccache compiler worker retains job TEMP through dependency-file publication
  - managed Cargo scratch cleanup waits for the complete compiler/cache process tree
  - released job leaves no Cargo/rustc process and removes only its exact scratch directory
---

# Frameworks01: managed Cargo removes the sccache temporary path during compilation

## Source executor

- Origin plan: `docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- Origin slice: ResourceManagement current-source release profile R4
- Fixing plan: `docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md`
- Fixing Session: `root-app08-runtime-artifact-reuse-20260830`
- Routing reason: the fixing Session owns the active `validate-matrix.ps1` runtime-artifact-reuse and
  compact-validation scope. Frameworks01 does not own or edit the managed Cargo storage lifecycle.

## Failure and exact receipt

Frameworks01 invoked the same ignored release profile after the previous foreign Runtime Interface
compile defects had advanced in current source. The managed command was:

```text
validate-matrix.ps1 -Package zr_resource -CargoProfile release -SkipBuild -LibTests
  -TestFilter resource_management_projection_current_source_profile -IgnoredTests
  -TargetDir E:\cargo-targets\frameworks01-resource-management-current-r4 -VerboseOutput
```

The report root was
`E:\zircon-profiles\frameworks01-resource-management-current-r4`; no output was written to C. The
coordinator receipt is:

- job `84f3507f1dee480184e94f5cbaf9fdb2`;
- child Session
  `validate-matrix:019ffe2b-296a-7023-9433-8654b9ea8f18:successor:e78b50911cc545f088400f1ee9abab3c`;
- start `2026-08-31T03:29:51.400449+08:00`;
- finish `2026-08-31T03:32:24.244899+08:00`, outer exit `1`;
- release `2026-08-31T03:32:31.974060+08:00`, cleanup status `retained`, no cleanup error;
- terminal job lookup request `949f999a4cf64dce8fecfc368d95cf3d`.

Compilation stopped in `zircon_runtime_interface`, before `zr_resource` or either profile scope was
compiled. Rustc was running through sccache when dependency-file publication failed:

```text
warning: ignoring -C extra-filename flag due to -o flag
error: failed to write dependency file to
\\?\E:\cargo-targets\zircon-engine\scratch\84f3507f1dee480184e94f5cbaf9fdb2\temporary\sccacheFuHIFi\deps.d:
The system cannot find the path specified. (os error 3)
```

Cargo returned 101. At terminal inspection the exact
`scratch\84f3507f1dee480184e94f5cbaf9fdb2` tree was absent. No Cargo or rustc process remained; only the
long-lived sccache server remained. The coordinator retained the target but does not expose an
inner managed run or a retained stdout/stderr artifact for this wrapper job, so this record preserves
the exact diagnostic and job receipt rather than inventing a log hash.

Current tooling source fingerprints at observation time are:

- `validate-matrix.ps1` SHA-256
  `94b2398c8097b07b3de577b38d3c4281b58eeb0ca7b7116a1533cf96687f7b49`;
- `managed-cargo-storage.ps1` SHA-256
  `7513108f7727a1d9da26e401958b61305054805c406b0d45c69745d3ad983805`.

## Lowest shared-layer diagnosis

`Push-ManagedCargoEnvironment` creates one job-scoped `temporary` directory and assigns it to
`TEMP`, `TMP`, and `TMPDIR`. `Pop-ManagedCargoEnvironment` restores the process environment and then
removes the complete job scratch tree. The observed error proves that a compiler/cache worker still
needed that directory when it was no longer present. The evidence does not yet prove whether cleanup
ran before the complete descendant process tree exited, whether the persistent sccache server kept a
job-scoped temporary path past its request lifetime, or whether another cleanup authority removed
the same directory. The fixing owner must distinguish those cases with process and directory-lifetime
evidence before changing the algorithm.

## Fix acceptance

- Add a deterministic RED that holds an sccache compiler request open through dependency-file
  publication while the validation wrapper reaches its cleanup boundary.
- Prove that one job cannot remove its scratch until Cargo, rustc, sccache compiler children, and any
  response-side dependency-file publication for that job are terminal.
- Do not keep a shared sccache daemon dependent on a deleted job-scoped `TEMP`; either give the daemon
  a stable non-C cache/temp authority or bind each request's temporary lifetime to an acknowledged
  compiler completion contract.
- Preserve exact-path, handle-bound scratch deletion after terminal completion. Do not weaken the
  approved-root and no-follow protections or retain unbounded scratch directories.
- Retain stdout/stderr or a typed terminal failure artifact for wrapper-managed jobs so a future
  infrastructure failure has a coordinator-addressable log hash.
- Rerun the exact Frameworks01 command. Acceptance requires that `zr_resource` compiles and both
  profile scopes emit their report artifacts; a tooling-only unit test is not sufficient return
  evidence.

## Forbidden temporary fixes

- Do not disable sccache and call the lifecycle fixed without measuring the intended reusable path.
- Do not add sleeps before cleanup or ignore OS error 3.
- Do not move TEMP or build artifacts to C.
- Do not leave every job scratch permanently retained.
- Do not attribute the 153-second failed build or any prior cold build time to Resource runtime
  performance.

## Rejected return R5

App08 returned a first tooling revision that sets `SCCACHE_CLIENT_SIDE=1` and
`SCCACHE_IGNORE_SERVER_IO_ERROR=1` whenever the wrapper is active. The returned source fingerprints
are:

- `managed-cargo-storage.ps1` SHA-256
  `8862823b6d00d934ff13e6bb981fb75907e19f2141e0cadde7fbaaab81155a41`;
- `managed-cargo-storage.Tests.ps1` SHA-256
  `1c440294736681fb2cb7cbf0d9c8048453cb9b71a958c230247080508b58ebd3`.

The owner reported Pester 5/5 and a two-request rustc metadata probe. Frameworks01 then reran the
exact acceptance command. Coordinator job `680c28eeb45f44ada781073ea28a3e50` started at
`2026-08-31T03:59:12.310595+08:00`, finished at `03:59:30.375280`, released at
`03:59:33.734781`, and returned outer exit 1. Its compatibility/reuse receipt names R4 job
`84f3507f1dee480184e94f5cbaf9fdb2` as `reused_from_job_id`.

R5 still failed before `zr_resource`. The fresh client reused the sccache server PID 1660, which was
started under R4 at 03:29:55 and remained live after both jobs. That server attempted to allocate a
new compiler temporary directory under the already retired R4 path, not the R5 path:

```text
sccache: encountered fatal error
sccache: error: Failed to create temp dir
path: \\?\E:\cargo-targets\zircon-engine\scratch\84f3507f1dee480184e94f5cbaf9fdb2\temporary\sccacheY8m77e
```

Cargo returned 101 while compiling `zircon_runtime_interface`; the wrapper returned 1. No profile
artifact was emitted. Terminal inspection found no Cargo or rustc process; the same sccache PID 1660
was still live. This disproves the first return as a complete fix: client-side mode does not rebind
the environment of an already-running server whose startup `TEMP` points at a deleted job scratch.
The metadata-only probe also did not reproduce Cargo's realistic `--emit=dep-info,metadata,link`
server-side temporary allocation.

The next return must deterministically start or reuse a server bound to a retired job TEMP, then
compile a dependency-file/link request through it. The accepted design must give the shared daemon
a stable non-C temporary authority or perform a controlled health/rebind transition before reuse;
setting client environment variables alone is insufficient. The exact Frameworks01 command remains
the terminal acceptance gate.

## Frameworks01 state

The failure was routed read-only to task `01a05272-935c-7001-aa4f-773e45ccd3ad`, whose active
coordinator Session is `root-app08-runtime-artifact-reuse-20260830`. R5 was rejected; the revised R6
storage lifecycle is accepted on exact origin evidence below. R9 completed the ResourceManagement
origin profile and closes this tooling Failure. The separate readiness profile, RSS/power evidence,
post-change comparison, production algorithm work, M1 acceptance, milestone commit, and WeCom
notification remain outside this fixed infrastructure record.

## R6 exact-origin return

App08's revised storage lifecycle assigns independent D/E/F sccache endpoints, binds the daemon to
the stable non-C `cache/sccache-temporary` authority with a PID/start-time marker, and permits a
stale-daemon restart only when no Cargo or rustc process is active. The returned source hashes are:

- `managed-cargo-storage.ps1`
  `4b0b7426b02c0f6e61e717f95c0a02250e05ed4e4ad608f3067283305ccc5f50`;
- `managed-cargo-storage.Tests.ps1`
  `d2882f0d7ecfd41f60ae8a5608ece3c852ef71731f87d78a66090bd9bd675391`;
- `validate-matrix.ps1`
  `79fe249f384e8eb8b333b1a121e92c72a2c7a3b595efbce98729b8448c538163`;
- `cargo_storage.py`
  `d9249bc2d74ac68483a777abba8c3d4e85734c62cad933c9d09e6db692789d32`;
- `artifact_governance.py`
  `b2efe9a78519aaa9392ebb622ba00ef0a9841f517783b9ba75046579fe414d6d`.

The fixing owner reported the deterministic retired-TEMP dep-info/metadata/link RED/GREEN, storage
Pester 7/7, affected validate-matrix lifecycle 7/7, artifact governance 35/35, and storage mapping
4/4. Frameworks01 then ran the exact command. Outer job
`96c7732d445d4596b5e86f662d8333ed`, successor Session
`validate-matrix:019ffe2b-296a-7023-9433-8654b9ea8f18:successor:c42b66e9f6944e31a39dc49798360252`,
started at `2026-08-31T04:57:00.893264+08:00`, finished at `05:00:13.738628`, released at
`05:00:25.025055`, and returned outer exit 1. It reused R5 job
`680c28eeb45f44ada781073ea28a3e50`; the wrapper exposes no inner run ID.

This exact run used sccache endpoint `127.0.0.1:42261`, daemon PID 31088, and stable server TEMP
`E:\cargo-targets\zircon-engine\cache\sccache-temporary`. Both `zircon_runtime_interface` and
`zr_resource` reached real dep-info/metadata/link compilation. No deleted-TEMP, `deps.d`, or OS error
3 failure recurred, so the sccache lifecycle defect itself is fixed on the origin command.

The run stopped before either profile scenario because Frameworks-owned ignored RED source read
`self_record.id` after `ResourceRecord::with_dependency_ids` consumed the value
(`manager/readiness_projection/tests/behavior_red.rs:7`, E0382). The construction now captures the
ID before the move; the repaired file SHA-256 is
`f7a2e749b105c7082f7e7e4078c176353653284c2c2d2a3f129ad076c2de7282`, rustfmt and whitespace checks
are green, and attribution request `423fe159221c4802884798269fd8c83d` succeeded.

The next exact submission request `b2b1a4b0e31a4f448a3c662cf721fada` was accepted but terminally
failed before job creation with `cargo_cpu_lane_reserved` for RuntimeInterface03 reservation
`eb7b006ec5ff464fa2f5102f9974f522`. It is a FIFO scheduling result, not a source or infrastructure
failure. The canonical record stays open until the identical command executes both profile scopes
and emits the report artifacts.

## R7 symmetric job-TEMP isolation and FIFO reconciliation

Origin R6 proved that the deleted daemon-TEMP defect was gone, but App08 found that its helper had
temporarily pointed the complete Cargo child environment at the stable daemon TEMP. The corrected
contract now uses stable `cache/sccache-temporary` only while initializing the long-lived sccache
daemon; Cargo, rustc, and build scripts again receive isolated
`scratch/<job>/temporary` `TEMP`/`TMP`/`TMPDIR`. The returned protected hashes are:

- `managed-cargo-storage.ps1`
  `7d1eb4fe2bad2fb7bc124efcac272c187226b9a6f52dbdf9c86e4cd5342f74d9`;
- `managed-cargo-storage.Tests.ps1`
  `4798293a9503186b1917aa5dc5074bbbc005dacd866868366f4eb529d1502cc9`.

The earlier `70858e9e...` / `fe3b98ba...` return was superseded after online use found that raw
string comparison treated Windows extended `\\?\E:\...` and display `E:\...` spellings as different
paths. Python and validate-matrix could therefore alternate unnecessary daemon rebinds. The current
helper compares Windows path identity. Frameworks01 verified the two replacement hashes above
exactly. App08 reported a pre-fix 6/7 RED and complete 7/7 GREEN, including extended-to-display and
display-to-extended reuse. Online ticket `8b0d0f42f3854ce28982fb3400d2583e` passed after about 56 seconds
of materialization and 10.63 seconds of Cargo/link; endpoint 42261 remained on PID 14596 throughout
and reported `Restarted=false`.

The exact origin command has not yet run to profile completion. The retry history is scheduling
evidence only:

- a transient unmanaged parent `E:\cargo-targets\zircon-engine\ephemeral\check` was traced to UI12
  managed job `3e36729ca8954949b8c07e1348a1c925`; it released with
  `cleanup_status=deleted`, and artifact audit `25e75f287cc147a29d7c4d7fe8acc626`
  returned `unmanaged=[]` without a Frameworks filesystem action;
- exact acquire request `4e8f7e19df8e4006b2b70230442f0e72` reconciled terminal failed with
  RuntimeInterface02 reservation `fddb926811924e0d98be1f863e7aec7d`; no job was created;
- after the symmetric helper return, artifact audit `a276efe3790a49589380e6b3d3941197`
  again returned `unmanaged=[]`, but the next exact acquire was rejected before job creation by
  Runtime04 reservation `b6aa834be4e8489bad9d217b1ff949a0`, since consumed as leased foreign job
  `98f39ba071014dad919ce54bcb974a4e`;
- after the Windows path-equivalence repair, Frameworks01 artifact audit request
  `0943b0b4b4e847029b52a894ae86a81b` completed with `unmanaged=[]`; the identical origin command's
  acquire request `16c82d55152a49f685c381263053016a` then terminally failed before job creation with
  `cargo_cpu_lane_reserved` for RuntimeEditor reservation `7551b566d19540438e8d2992d61b40e0`.

Frameworks01 did not cancel, consume, or bypass any reported reservation. The sccache lifecycle defect is
fixed on exact R6 compile evidence and its storage boundary is now symmetric, but this canonical
record remained open until R9 emitted the ResourceManagement artifacts below.

## Fixed return R9

The literal command recorded above omitted the environment variable that the ignored profile test
requires. Job `9acd911f7caf4aa583fa31000f66be0a` compiled `zircon_runtime_interface` and `zr_resource`
successfully through sccache, then returned Cargo 101 because
`ZR_RESOURCE_MANAGEMENT_PROFILE_DIR` was absent. A direct execution of the already-built E-drive
test binary reproduced the exact panic at `profile.rs:188` in 0.00 seconds. This was a Frameworks
profile invocation defect, not a recurrence of the sccache lifecycle failure.

The corrected origin preserves the package, release profile, filter, target directory, ignored-test
mode and managed validator, while explicitly binding the required report directory outside C:

```powershell
$env:ZR_RESOURCE_MANAGEMENT_PROFILE_DIR =
  'E:\cargo-targets\frameworks01-resource-management-current-r4\profiles\resource-management-current'
& '.codex/skills/zircon-dev/scripts/validate-matrix.ps1' `
  -Package zr_resource -CargoProfile release -SkipBuild -LibTests `
  -TestFilter resource_management_projection_current_source_profile -IgnoredTests `
  -TargetDir 'E:\cargo-targets\frameworks01-resource-management-current-r4' -VerboseOutput
```

Pre-run artifact audit request `b2559d6fdd2841d6894ec2f972f99189` completed with
`unmanaged=[]`. Managed job `f2f3280096d64ca699bdd9c9e4800e97` started at
`2026-08-31T07:03:07.175317+08:00`, finished at `07:07:19.454952`, released at
`07:07:25.510111`, and returned exit 0. It used E-drive scratch, target, Cargo cache, sccache cache
and stable daemon TEMP; endpoint `127.0.0.1:42261` remained PID 14596. Real
dep-info/metadata/link compilation and the ignored test completed without deleted TEMP, `deps.d`,
OS error 3, or daemon rebind failure.

The profile emitted 14 scenarios with 31 samples and 3 warmups each:

- metadata: 345 bytes / 10 lines / SHA-256
  `f7cc3e2bc196d316790d4f590cb16394c7d2b1135dab3407ecb2b0338ba79a07`;
- raw samples: 27,125 bytes / 435 lines / SHA-256
  `8c8bb282a3d3c051f85ff1b8f5198b66e4fb4d3ed69b79adae6bbbbad2701230`;
- summary: 3,126 bytes / 15 lines / SHA-256
  `1244bf20c9b30bf0fca4bd7f3bf850c7502a36c0f911823079718583923d05cd`.

The compiled source fingerprints recorded by the profile are management projection Blake3
`57c2d11a4c7f9d74e95c3999383037667bdf07906c339792a20a627a5e747cf5` and generation Blake3
`7caa2ed7d9cee4f7180bebfcc863af19319d0b913f25e3622413494935ac550f`.
Both files predate the run and did not drift during it. The result includes latency and allocation
evidence but explicitly records RSS and power as unavailable. The sccache TEMP lifecycle Failure is
therefore fixed; no broader Resource milestone is implied.
