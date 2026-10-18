# Editor CI validation

`CI / editor-compile-gate` runs on every pull request targeting `main` or `master`
using the GitHub-hosted `windows-2022` image. It checks the full
`target-editor-host` closure and plugin workspace, then executes the VM behavior
bridge lifecycle regression and editor `core::commands` tests. The test wrapper
rejects zero matching or zero passing tests. Its timeout is 60 minutes. It needs
no repository secret or GPU.

All Cargo, rustup, build and temporary directories are under a fresh
`D:\cargo-targets\zircon-local\ci\runs\<run>-<attempt>\e-0`.
The physical-volume, SUBST, ancestor reparse and previous-output checks run before
toolchain installation. Missing approved physical storage is a failure, with no
fallback directory. The image name is pinned; its concrete image version and
compiler identity are printed in the job log.

The short `e-0` lane and `c` Cargo-home leaf leave room for the pinned VM
dependency's 137-character relative paths under Windows libgit2's path limit.
Other Windows jobs use unique short lanes too; job/check names are unchanged.
The portable contract budgets every Cargo checkout for a 20-digit run ID,
three-digit attempt and three-digit matrix index without using path aliases.

The preparation shell starts with `runner.temp` for TEMP/TMP so PowerShell startup
cannot pre-create the managed run directory. `-BootstrapTemp` verifies that
identity and switches the script to the checked run paths. Subsequent commands
use the job's managed TEMP/TMP. The fresh-tree check remains enforced.

## Portable contract checks

From the repository root on Linux or Windows (Python 3.11+):

```sh
python -B .github/tests/test_ci_gates_contract.py
python -B .github/tests/test_runner_availability.py
git diff --check
```

These tests inspect workflow contracts and exercise runner inventory handling in
memory, including pagination, malformed data, offline/wrong-platform runners and
missing credentials. They do not compile Rust or establish Windows acceptance.

## Windows reproduction

Use a Windows checkout with VS x64 C++ tools, `rustup` on PATH and a physical D:
volume. Run in a fresh PowerShell process so the environment changes are scoped
to that process. Pick a new numeric run identity for each attempt:

```powershell
$env:GITHUB_RUN_ID = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds().ToString()
$env:GITHUB_RUN_ATTEMPT = '1'
$env:ZR_CI_OUTPUT_ROOT = "D:\cargo-targets\zircon-local\ci\runs\$env:GITHUB_RUN_ID-$env:GITHUB_RUN_ATTEMPT\e-0"
$env:CARGO_HOME = Join-Path $env:ZR_CI_OUTPUT_ROOT 'c'
$env:RUSTUP_HOME = Join-Path $env:ZR_CI_OUTPUT_ROOT 'rustup'
$env:CARGO_TARGET_DIR = Join-Path $env:ZR_CI_OUTPUT_ROOT 'target'
$env:CARGO_BUILD_BUILD_DIR = Join-Path $env:ZR_CI_OUTPUT_ROOT 'build'
$env:TEMP = Join-Path $env:ZR_CI_OUTPUT_ROOT 'tmp'
$env:TMP = $env:TEMP
$env:RUSTC_WRAPPER = ''
$env:RUSTC_WORKSPACE_WRAPPER = ''
$env:CARGO_INCREMENTAL = '0'
& .github/scripts/prepare-cargo-roots.ps1 -StorageRoot 'D:\cargo-targets\zircon-local'
& rustup toolchain install 1.94.1 --profile minimal
if ($LASTEXITCODE -ne 0) { throw 'Rust toolchain installation failed' }
python -B .github/tests/test_msvc_wrapper_arguments.py
if ($LASTEXITCODE -ne 0) { throw 'MSVC wrapper regression failed' }
& .github/scripts/invoke-cargo-msvc.ps1 cargo +1.94.1 check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked
if ($LASTEXITCODE -ne 0) { throw 'Editor closure check failed' }
& .github/scripts/invoke-cargo-msvc.ps1 cargo +1.94.1 check --manifest-path zircon_plugins/Cargo.toml --workspace --locked
if ($LASTEXITCODE -ne 0) { throw 'Plugin workspace check failed' }
& .github/scripts/invoke-cargo-msvc.ps1 -RequireMatchingTests cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --test behavior_bridge --locked
if ($LASTEXITCODE -ne 0) { throw 'Script behavior bridge lifecycle tests failed' }
& .github/scripts/invoke-cargo-msvc.ps1 -RequireMatchingTests cargo +1.94.1 test -p zircon_editor --lib core::commands --locked
if ($LASTEXITCODE -ne 0) { throw 'Editor command tests failed' }
```

This is independent command evidence. It supplies neither Jenkins acceptance nor
GUI/first-frame acceptance. Use [the launch guide](run-editor-locally.md) for
source-bound product startup and capture.

The wrapper regression uses a Python child to verify native argument forwarding
and non-vacuous test admission without compiling. Its three cases require
Windows and the VS environment; Linux discovery skips them explicitly.

## Hosted run and persistent runners

Inspect a PR run using `gh run list --workflow ci.yml --branch
fix/claude-review-20261017`, then `gh run view <run-id> --json jobs` and
`gh run view <run-id> --log-failed`. Require an executed, successful
`editor-compile-gate` with nonzero duration. A skipped, cancelled or pending job
does not count. `main` branch protection requires that exact GitHub Actions check
and an up-to-date branch. Verify it with:

```sh
gh api repos/He-Jiahui/ZirconEngine/branches/main/protection/required_status_checks
```

Trusted `main`/`master` pushes first run `runner-availability` on `ubuntu-24.04`.
All persistent Windows jobs depend on that result and have 120-minute execution
timeouts. The signal requires a repository secret named `RUNNER_STATUS_TOKEN`
containing a fine-grained token with **Administration:read** for this repository;
the normal `GITHUB_TOKEN` cannot list runners. Missing credentials, API errors or
no online runner with all `self-hosted`, `Windows`, `X64` labels fail visibly and
skip dependent jobs. A busy online runner is available but may still queue;
execution timeouts do not impose a queue deadline. PR source never receives this
secret and never runs on persistent runners.

The inventory probe is read-only:

```sh
python -B .github/scripts/check-runner-availability.py --repository He-Jiahui/ZirconEngine
```

Supply the token through the environment; do not place it in command arguments,
source, receipts or logs. Runner registration, token provisioning and actual
hosted/Windows run results remain separate from the source contract pass.

## MVP job boundaries

`MVP Editor Windows` separates `compile-gate`, `focused-tests`, and staged F5
acceptance. The guard resolves a lockfile/config hash after checkout, restores
only that job/toolchain's compiler target, and checks the restored tree for
reparse points before compilation. Each job retains its own evidence artifact;
F5 also downloads the preceding diagnostic artifacts. A compiler cache hit does
not replace any source, product or test receipt.

The existing F5 publisher still depends on retired local-coordinator entry
points, so product publication, cross-job binary reuse and relocating the F5
workspace-test receipt to the trusted wave job are **pending migration**. Its
current acceptance registry remains enforced. Run the Windows source contract
without compiling Rust:

```powershell
pwsh -NoProfile -File tools/tests/mvp_editor_windows_workflow.Tests.ps1
```

The portable CI contract also checks job dependencies, physical cache identity,
all existing F1–F4 selectors and the retained F5 inputs. When the publisher is
migrated, enable the MVP workflow and dispatch the authorized branch, then retain
per-job durations, cache results and the full F5 evidence artifact. Do not accept
a cached target or a diagnostic-only artifact as a staged product.
