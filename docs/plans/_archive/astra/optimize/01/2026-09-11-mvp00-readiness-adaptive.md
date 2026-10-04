---
record_kind: milestone
status: implemented_pending_validation
created_at: 2026-09-11
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: W0/M0.1 isolated coordinator readiness fixture
session: mvp00-readiness-adaptive-20260911
---

# M0.1 isolated coordinator readiness fixture

## Scope and lowest broken layer

The original `Test-Kernel` launch omitted a port and used a fixed `50 * 100 ms`
health loop. The shared coordinator was already listening on `127.0.0.1:6518`
(PID 300, `python -m tools.session_coordinator --repo-root
E:\\Git\\ZirconEngine --port 6518 serve`). An isolated disposable-repository
reproduction using the old launch arguments exited with code 2 and
`[WinError 10048] ... address/port may only be used once`; it produced no
`runtime.json` or `startup-failure.json`. This establishes the fixture launch
contract and readiness window as the lowest broken layer, not coordinator
production behavior.

The same disposable launch with `--port 0` published a valid runtime descriptor
on an ephemeral port (61985 in the probe) and became healthy after approximately
8,843 ms under the current Windows load. The shared listener and any unrelated
Cargo work were left untouched.

## Owned implementation and regression

- `tools/tests/session-coordinator-smoke.Tests.ps1:18-19` defines a bounded
  30-second readiness budget and 100 ms poll interval.
- `:83-173` adds `Wait-CoordinatorHealthy`, which first waits for the
  server-owned runtime descriptor (avoiding repeated multi-second descriptor
  retries), checks process liveness, polls health, and includes bounded tails of
  `startup-failure.json`, stdout, and stderr in exit/timeout diagnostics.
- `:197-201` launches the isolated daemon with `--port 0` and redirects both
  output streams; `:204-210` consumes the adaptive readiness receipt.
- `:225-245` preserves the production lifecycle policy: the unscoped `stop`
  command must return exit code 2 and
  `lifecycle_global_shutdown_disabled`, after which only this disposable
  process tree is killed and the expected native exit code is cleared.
- `tools/tests/session-coordinator-smoke.Tests.ps1:598-709` applies the same
  isolated port, bounded readiness, diagnostic capture, and typed-stop cleanup
  contract to `Test-LeaseAndPatch`; its delayed patch assertion remains
  unchanged.
- `tools/tests/test_mvp00_session_coordinator_smoke_contract.py:12-116`
  locks both lane contracts, including ephemeral-port, bounded-readiness/
  diagnostics, and typed-stop rejection.

The pre-change Python contract was intentionally red (missing explicit port and
readiness helper). The final focused contract is green:

```text
python -m unittest tools.tests.test_mvp00_session_coordinator_smoke_contract -v
Ran 3 tests in 0.004s
OK
```

PowerShell parsing also passed with zero parser errors. The final Windows retry
was run twice; the measured wrapper run was:

```text
pwsh -NoProfile -ExecutionPolicy Bypass -Command '$sw=[Diagnostics.Stopwatch]::StartNew(); & .\\tools\\tests\\session-coordinator-smoke.Tests.ps1 -KernelOnly; $ok=$?; $childCode=$LASTEXITCODE; $sw.Stop(); Write-Output ("KERNEL_SUCCESS={0} KERNEL_LASTEXITCODE={1} KERNEL_ELAPSED_MS={2}" -f $ok,$childCode,[int]$sw.Elapsed.TotalMilliseconds); if(-not $ok){exit 1}else{exit 0}'
PASS: coordinator kernel smoke
KERNEL_SUCCESS=True KERNEL_LASTEXITCODE=0 KERNEL_ELAPSED_MS=27678
exit code: 0
```

The direct command `pwsh -NoProfile -ExecutionPolicy Bypass -File
tools/tests/session-coordinator-smoke.Tests.ps1 -KernelOnly` also returned
`PASS: coordinator kernel smoke` with exit code 0. A listener audit afterward
showed only the pre-existing shared coordinator on port 6518 (PID 300), and no
temporary smoke process remained.

The follow-up `-LeaseAndPatch` retry also returned `PASS: coordinator lease and
delayed patch smoke`, with wrapper success `True`, `$LASTEXITCODE=0`, and
26,560 ms elapsed. It exercised two active sessions, a queued patch behind a
lease, release/apply, and the same typed global-stop rejection before killing
only its disposable process. A post-run audit found no temporary listener; the
shared PID 300 listener was untouched. An unrelated pre-existing Python process
(PID 1632, WindowsApps launcher, no TCP listener) was observed and left
untouched as foreign state.

The complete smoke matrix was subsequently rerun in one invocation. Kernel,
lease/patch, managed Cargo/cleanup (including 115 coordinator unit tests),
temporary-repository finalize, legacy report/import/archive with two
maintenance ticks, JSON-client, strict-parser, and validator-dry-run lanes all
passed. The wrapper returned `M01_SUCCESS=True M01_LASTEXITCODE=0` after
`752371 ms`; an afterward listener/process audit found no temporary smoke
process. The long duration is retained as readiness/maintenance diagnostic
evidence, while the separate managed validation receipt remains required.

## Validation boundary and remaining gate

This is a focused fixture repair, not full W0 acceptance. The complete M0.1
declared Pester/Python batch (including cold/warm launcher, lease/patch,
legacy, and residual-job/lease checks) still requires a separate managed
validation receipt. No Cargo, native, Tauri, or external `E:\\Git\\zr_vm`
command was run. Status therefore remains `implemented_pending_validation`.

Final source fingerprints:

| File | SHA-256 |
|---|---|
| `tools/tests/session-coordinator-smoke.Tests.ps1` | `82FF580C81E68DEB158D98453949DB4BD1372CBEA510FA35D92866FDDFF77159` |
| `tools/tests/test_mvp00_session_coordinator_smoke_contract.py` | `E44D1DCE271302E1C6CC1C9029F9AA3CA0AC0CDC89DCE58B26AE1146C225E637` |

`git diff --check` passed for both owned source/test paths (only the existing
LF-to-CRLF normalization warning was emitted). No commit was created.
