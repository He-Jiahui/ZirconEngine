---
handoff_kind: failure
status: open
created_at: 2026-09-07
summary_slug: app-runtime-library-verbatim-path
origin_plan: docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
fixing_plan: docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md
origin_child_dir: docs/plans/optimize/zircon_app/08
fixing_child_dir: docs/plans/optimize/zircon_runtime/25
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/project/paths.rs
  - zircon_runtime/src/asset/project/paths/tests.rs
  - zircon_app/src/entry/runtime_library/library_path.rs
  - zircon_app/src/entry/runtime_library/tests.rs
tests:
  - tools/dev/dev-fast-build.ps1 -Action test -Package zircon_app -FeatureOverride target-server -LibTests -TestFilter runtime_library_path_prefers_executable_sibling_when_present -LinkMode static
  - tools/dev/dev-fast-build.ps1 -Action test -Package zircon_app -FeatureOverride target-server -LibTests -TestFilter runtime_library_path_prefers_executable_sibling_when_present -LinkMode dev-dynamic
---

# Runtime25: Product library resolution fails for managed Windows paths

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md`
- 来源执行切片：App08 Windows server static/development-DLL correctness.
- 修复责任计划：`docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md`
- 交接原因：The failure reaches the shared ProjectPaths physical resolver before DLL loading.
  The current dirty paths.rs is attributed to `astra-full-domain-20260905`; coordinate its
  existing lexical-identity changes with that owner instead of overwriting them.

## 失败现象与复现证据

Managed dynamic suite `78159eadffa5401da1902ea46df3bdd8` failed both product-sibling and
Cargo-deps library lookup, plus relative-override tests, while resolving the product directory:
`could not resolve product executable directory for default runtime library`, `os error 1`.

Focused static job `0afd724815e94a528c67f929d63a2354` reproduced the same failure with
`runtime_library_path_prefers_executable_sibling_when_present`: 0 passed, 1 failed,
219 filtered, wrapper exit 1 and Cargo test exit 101. The test creates two real files
in a temporary product directory; it requires no junction or actual DLL load.
Sealed input `aae2aebb9e1c81ea7aa8ecdd3d1d8b9e66abc392413411a64265de39abee3e9b`
was unchanged: 10,708 files, 351 dependencies verified, zero repairs. The check and
test executable were reused; the static receipt contains no development DLL.
Queue/sync/check/compile-link/test times were 20.429/67.285/0.017/16.878/12.977 seconds.
This failed run is diagnostic evidence, not a successful performance sample.

Logs are retained under
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/app08-server-lib-assertions-20260907/results/path-sibling-static.log`
and `app08-server-lib-contracts-20260907/results/library-dynamic.log` in the same cache root.

## 最低共享层根因

Inspected candidates: App default/override selection, ProjectPaths ancestor discovery,
physical canonicalization, temporary-path representation, and development DLL loading.
Both link modes fail before library selection. The managed temporary paths use Windows
verbatim drive paths. `split_at_deepest_existing_project_ancestor` calls `fs::metadata`
after each individual component, including the standalone verbatim drive prefix before
its RootDir is appended. A direct read-only Win32 `GetFileAttributesExW` probe accepts
the complete verbatim drive root but rejects the prefix alone with error 87.
This identifies the root-prefix boundary for focused Rust verification; it does not
claim the Win32 probe reproduces Rust's exact error code.

At diagnosis, the live paths.rs SHA-256 was
`7c7671948fcb1fe6db812a9026db0075ab106e4009f3c4593ed3b936131a383a`;
the sealed version is `3a0c26d46bf7cc327bac945de838b30e3c00ca3ba54972da40a4eab9f1f260cc`.
Their difference adds lexical identity handling; the failing ancestor-scan and physical
canonicalization functions are unchanged. App08 made no edit to either shared function.

## 架构修复验收

- Add focused Windows ProjectPaths regressions for complete verbatim drive roots and
  ordinary/verbatim product directories, including an uncreated tail.
- Preserve physical junction identity, dot-segment semantics, long-path support,
  rooted/drive-relative rejection, and error propagation.
- Correct the lowest shared resolver boundary, then run the original App sibling,
  deps-sibling and relative-override tests in managed static and dynamic modes.
- Keep fixture-only junction command issues separate from this no-junction reproduction.

## 禁止临时方案

- Do not bypass ProjectPaths in the App loader, strip physical identity to compare strings,
  remove managed isolation, or ignore path failures for one link mode.
- Do not overwrite the active source owner's unrelated lexical-identity changes.

## 修复结果与回传

Open: `implemented_pending_validation`. The ancestor walk now waits for RootDir
after a Windows prefix before querying metadata. The two Windows regressions cover
a complete verbatim drive root, ordinary/verbatim product directories and a missing
tail. paths.rs SHA-256 is `F0B9532A187307D28C01535FD1900538D111973A22DC3A4F4AC61015A6B5D40D`;
paths/tests.rs is `CD38C2B3932594729589DC2F95645113F25065A80484744B8BB70FABD71F2577`.
Attribution is `c99323cf779948dea8e239306ce203a9`; App08 independent source review
reported C0/I0/M0 on those exact hashes.

App08 sealed input `5520daee052f89ea71e728059b5111e15c3312460fa4bb4035ec5bc59976e7f6`
passed all 20 original upward path cases: static sibling/deps job
`100b6ecee3b4410f903677cf135cf71f` (4), static override job
`ea0e5f7453eb4e7eb0e5e4b367b745ee` (6), dynamic sibling/deps job
`f3a1598de23a43f2a608e430a72c5445` (4), and dynamic override job
`6f1df40d26194cf49cf7dab8b48059a1` (6), all with zero failures.
Logs and full receipts are under
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/app08-server-path-fix-20260907/results`.
These prove upward correctness, not performance acceptance.

The lower Runtime group remains pending. Job `bf01bab4fc8e4b99bf92c70accdb63e2`
failed compilation before any tests, on the importer test enum import and the
package-service test's undeclared tempfile. Both test-support causes are repaired.
Subsequent requests `695c8dc2e6494d4897b51a6d6ec5b989` and
`cb4b76d393ab450ab79c709037407880` failed coordinator admission before Cargo.
The latter classified App08's concurrent registered scratch as unmanaged; App08
confirmed normal cleanup and owns that tooling diagnosis. Return this handoff as
fixed only after the focused lower ProjectPaths tests actually pass.

## 2026-09-19 rolling successor source reconciliation

The current-source successor Session is
`failure-roll-01a084c8-runtime25-library-path-r2`, registered against the
Runtime25 fixing plan at baseline epoch `611`. The five-path archived-owner
scope (this failure record, the shared ProjectPaths implementation and tests,
and the App runtime-library implementation and tests) was transferred under
fingerprint
`163da29624b50ceecb23a064d0a53d686cfd05ca12c972d9130ea4408efd726b` and then
leased to that Session. The transfer retained the existing source bytes and
foreign edits; no path was overwritten.

The frozen current hashes are:

- `zircon_runtime/src/asset/project/paths.rs`:
  `f0b9532a187307d28c01535fd1900538d111973a22dc3a4f4ac61015a6b5d40d`
- `zircon_runtime/src/asset/project/paths/tests.rs`:
  `cd38c2b3932594729589dc2f95645113f25065a80484744b8bb70fabd71f2577`
- `zircon_app/src/entry/runtime_library/library_path.rs`:
  `bd65ef4b2e50a7aa74b4ca1516199ca06bb410d65ffd6c3a564db53468155e0b`
- `zircon_app/src/entry/runtime_library/tests.rs`:
  `b3c314fdf95a9f5de21717aacd90e8d401cd53cc142f49256ca02851b225b805`

The current-source contract check passed locally and emitted
`RUNTIME25_APP_RUNTIME_LIBRARY_VERBATIM_PATH_CURRENT_SOURCE_CONTRACT_PASS`.
It verifies the RootDir boundary in the deepest-existing-ancestor walk, the
complete verbatim-drive and ordinary/verbatim uncreated-tail regressions, the
physical ProjectPaths lookup used by the App loader, and the executable-sibling,
Cargo-deps fallback, and physical-directory identity tests. This is source
evidence only; it is not a managed Cargo acceptance result.

The successor remains `resolving_failure` while the static ticket is prepared.
Fresh managed Runtime25 focused tests, the original App08 static/development
link-mode gates, independent C/I/M review, canonical `fixed-*` return, and
closeout remain pending. The existing external-worktree blocker
`validation_ticket_external_worktree_dirty:E:/Git/zr_vm` prevents a new
immutable dynamic Cargo ticket until that state changes.

## 2026-09-19 managed static successor ticket

The owned current-source overlay was submitted as ticket
`01c33f65e6574c0c8c0ddf678e7ed86d` (request
`failure-roll-01a084c8-runtime25-library-path-20260919-r1`) with source-manifest
hash `4c426af8bf2dd910f04777cecfb66b5390dd754e65998ab1da4ff2402852cf82`.
The managed command is a Python source-contract check that verifies the
RootDir-aware ancestor walk, the complete verbatim-drive and uncreated-tail
regressions, and the App sibling/deps/physical-identity tests; it must emit
`RUNTIME25_APP_RUNTIME_LIBRARY_VERBATIM_PATH_CURRENT_SOURCE_CONTRACT_PASS`.
The ticket is static-only (`staticParseOnly=true`, `upwardAcceptance=false`),
and its dependency roots are the two tracked implementation files so the
foreign untracked files under the surrounding source folders are not archived
as dependencies.

The ticket is queued and has not yet produced a managed job or terminal result.
This receipt does not claim dynamic Runtime25/App08 acceptance, review, fixed
return, or closeout.

The ticket subsequently executed as managed job
`c7474397c3ba4a7caadbd05b61324d2e` / run
`01c33f65e6574c0c8c0ddf678e7ed86d`, exited `0`, and emitted the exact marker
`RUNTIME25_APP_RUNTIME_LIBRARY_VERBATIM_PATH_CURRENT_SOURCE_CONTRACT_PASS`.
Coordinator cleanup completed. This terminal receipt proves only the frozen
current-source contract; it does not satisfy the dynamic Runtime25 ProjectPaths
Cargo tests, the App08 static/development link-mode gates, independent review,
canonical fixed return, or failure closeout.

## 2026-09-20 independent source review r2

Reviewer session: `review-runtime25-library-path-r2` (child of
`failure-roll-01a084c8-runtime25-library-path-r2`). The review was limited to the
five-path successor scope at the current baseline; no foreign source was edited or
absorbed.

The review found **Critical=0 / Important=0 / Moderate=0**:

- `ProjectPaths::resolve_path` retains a Windows prefix in the candidate path and
  defers the first metadata probe until `RootDir`, so a complete verbatim drive root
  is treated as a physical root while an uncreated tail remains available for the
  same identity-preserving resolution. Existing-file canonicalization, symlink
  error propagation, and lexical-vs-physical identity responsibilities remain
  explicit.
- `ProjectPaths::resolve_path_from` keeps product-relative requests anchored to the
  already resolved executable directory and rejects rooted or drive-relative input.
  The App loader therefore preserves the caller's request label while selecting the
  physical sibling/dependency path, including the fallback when the executable
  sibling is absent.
- The focused source-contract marker, scoped rustfmt, and scoped diff checks were
  independently reproduced. No implementation or test assertion in the owned
  files introduced a contract drift.

Current review hashes:

```text
zircon_runtime/src/asset/project/paths.rs
  f0b9532a187307d28c01535fd1900538d111973a22dc3a4f4ac61015a6b5d40d
zircon_runtime/src/asset/project/paths/tests.rs
  cd38c2b3932594729589dc2f95645113f25065a80484744b8bb70fabd71f2577
zircon_app/src/entry/runtime_library/library_path.rs
  bd65ef4b2e50a7aa74b4ca1516199ca06bb410d65ffd6c3a564db53468155e0b
zircon_app/src/entry/runtime_library/tests.rs
  b3c314fdf95a9f5de21717aacd90e8d401cd53cc142f49256ca02851b225b805
```

The adjacent untracked `zircon_runtime/src/asset/project/paths/lexical_identity.rs`
is attributed to another owner and is outside this review manifest. Its dynamic
compilation/closure must be coordinated with that owner; this review does not reuse
or claim it. Managed Runtime25 Cargo, the App08 link-mode gates, upward acceptance,
canonical `fixed-*` return, and closeout therefore remain pending.
