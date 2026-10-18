# Defect classes

Each class below recurs in the repository's own `BUG: [CR-...]` markers. When a change touches the trigger, apply the rule and add the required test. Evidence IDs and examples come from the supplied 2026-10-12 review; they are historical leads, not an inventory of currently open bugs. Recheck each marker and file in the actual checkout. "Guard" names the `change_guard.py` rule that catches part of the class; classes without a guard rule rely on review.

## D1. Recursive deletion and path containment

- **Trigger:** `shutil.rmtree`, `remove_dir_all`, `Remove-Item -Recurse`, or a path assembled from a profile, preset, project file or CLI argument.
- **Rule:** For profile/preset names used as directory components, require a single component matching `^[A-Za-z0-9][A-Za-z0-9_-]*$`. Resolve the final target and confirm it is inside the managed root before deleting. Refuse, never fall back, when the check fails.
- **Test:** `../x`, an absolute path, `C:\x`, `C:x` and a symlink leaving the root are refused and nothing outside a temporary root is removed.
- **Evidence:** `CR-EXPORT-0001` (`zircon_runtime_interface/src/export/preset.rs`); `tools/export/platform_bundle.py` joins `out_root / "bundle" / args.profile` and deletes the result.
- **Guard:** `fs.unbounded-delete` (no containment call in the 25 preceding lines).

## D2. Project-relative paths

- **Trigger:** parsing, joining or deserializing project, asset or plugin-package paths.
- **Rule:** Validate every segment, not only the first: reject `:`, `\`, `..`, `.`, empty segments, Windows device names (`CON`, `PRN`, `AUX`, `NUL`, `COM1`-`COM9`, `LPT1`-`LPT9`, with or without extension) and trailing dots or spaces. Construct and deserialize only through the parser.
- **Test:** `safe/C:/outside`, `a/../b`, `NUL.txt`, `a./b` and `a /b` are rejected on every platform.
- **Evidence:** `CR-PROJECT-0001` (`zircon_runtime_interface/src/project/rel_path/parse.rs` in the supplied review; verify its current owner).

## D3. Raw pointers and `unsafe`

- **Trigger:** `unsafe` blocks or impls, FFI, a public function taking `*const`/`*mut`.
- **Rule:** A safe `pub fn` does not dereference a caller's raw pointer. Make it `unsafe fn` with a `/// # Safety` section, or take a reference or a validated handle. Every new `unsafe` block or impl carries a `// SAFETY:` comment naming the invariant it relies on.
- **Test:** safe wrappers reject null pointers or invalid handles; exercise unsafe entry points only with inputs satisfying their safety contract. Never dereference a fabricated dangling pointer as a regression fixture. Run Miri on the affected module when Jenkins offers it.
- **Evidence:** `CR-R02-public_sdk_reflect-0001` and `-0002` (`zircon_plugins/plugin_sdk/src/native.rs`).
- **Guard:** `rust.safe-fn-raw-pointer`, `rust.unsafe-without-safety`, `rust.unsafe-fn-safety-doc`. Patch mode checks added unsafe uses and changed public parameter lists; pointer-typed callback parameters and returns do not count as caller-owned raw pointers.

## D4. Panic containment

- **Trigger:** `catch_unwind`, FFI entry points and callbacks, schedulers, task systems.
- **Rule:** The root `Cargo.toml` `shipping` profile sets `panic = "abort"`, so `catch_unwind` contains nothing in shipping builds. Do not rely on it for shipping correctness until the panic policy is decided. Drop a caught panic payload inside a second guard; a payload whose `Drop` panics escapes `Err(_)`.
- **Test:** a callback panicking with a payload whose `Drop` also panics.
- **Evidence:** `CR-PLUGIN-NATIVE-0101` (`zircon_runtime/src/plugin/loader/ffi_panic_guard.rs`).
- **Guard:** `rust.catch-unwind-under-abort` (warning).

## D5. Bounds on untrusted data

- **Trigger:** cooked assets, network packets, configuration numbers, public fields.
- **Rule:** Slice with `get(start..end)` after checking `start <= end <= len`. Validate configured divisors as nonzero before division or remainder, for example with `NonZero*`. Use checked conversion for `u64` to `u32` and checked arithmetic for sizes. Reject NaN and infinity at construction.
- **Test:** a malformed fixture for each rejected case returns an error, without panicking.
- **Evidence:** `CR-VIRTUAL-GEOMETRY-0007` (`cluster_start` past the header), `CR-PLUGIN-NET-0022` (`drop_every_nth_packet = Some(0)`).

## D6. Graph traversal

- **Trigger:** parent chains, dependency graphs, hierarchy or page walks.
- **Rule:** Track visited nodes or bound the depth by the node count. A cycle is an error, never a hang.
- **Test:** a cyclic fixture terminates with an error.
- **Evidence:** `CR-PLUGIN-ANIMATION-0006`, `CR-VIRTUAL-GEOMETRY-0003`, `CR-VIRTUAL-GEOMETRY-0005`.

## D7. Validation bypassed by deserialization or public fields

- **Trigger:** `#[derive(Deserialize)]` or `pub` fields on a type with a validating constructor or setter.
- **Rule:** Keep fields private and deserialize through the validator with `#[serde(try_from = "Raw")]`. `RelPath` already shows the pattern.
- **Test:** deserializing each invalid value fails.
- **Evidence:** `CR-PLUGIN-NET-0022` (serde carries the value the setter rejects).
- **Guard:** `rust.serde-bypass` (warning).

## D8. Network authorization and dispatch

- **Trigger:** RPC or message dispatch, validators, timeouts.
- **Rule:** Check session state for every client-originated direction, including bidirectional descriptors. Do not call user callbacks while holding a global lock. Decide a timeout before side effects commit, or make the operation idempotent.
- **Test:** an unauthenticated bidirectional call is refused; a validator that calls back into the manager does not deadlock.
- **Evidence:** `CR-PLUGIN-NET-0016`, `CR-PLUGIN-NET-0017` (`zircon_plugins/net/features/rpc/runtime/src/manager/dispatch.rs`).

## D9. Coordinate spaces and contract values

- **Trigger:** transform reads and writes; render-graph flags checked against executor contracts.
- **Rule:** Read and write in the same space; convert world to local through the parent's inverse. Assert a static contract and its compiled value in one test, not in two tests that each pin one side.
- **Test:** a child entity under a translated parent; compile the render graph and execute one frame.
- **Evidence:** `CR-SCRIPT-AUDIT-0002` (`zircon_runtime/src/script/vm/gameplay_host/transform.rs`), `CR-PLUGIN-RENDERING-0008` (contact shadow never executes).

## D10. Test reachability and integrity

- **Trigger:** new or moved test files, a removed `mod` line, edits to assertions.
- **Rule:** Executable Rust source must have a complete path from a Cargo target through literal `mod`, `#[path]` or `include!` edges. A declaration in an orphan parent does not connect its child. Respect Cargo auto-discovery flags; document source used only as fixture data. Do not ignore or skip a test without a marker. A test asserting stale text must be fixed with the code, not kept failing. File-based fixtures use permitted, owned output namespaces. This guard's regression checks use a synthetic file tree in memory and forbid filesystem mutation.
- **Test:** the guard reports no `rust.unreachable-module` for the patch, and the affected test is compiled and executed by the applicable validation lane. Literal reachability does not prove feature activation or macro expansion. Module path semantics follow the [Rust Reference](https://doc.rust-lang.org/reference/items/modules.html); target discovery follows the [Cargo Book](https://doc.rust-lang.org/cargo/reference/cargo-targets.html).
- **Evidence:** about 19,000 unwired virtual-geometry test lines (`virtual_geometry_renderer_test_promotion_guard.rs`); orphaned `segment_cache/run_index.rs`, `template_icon_button_glyph_kind.rs` and a duplicate `hybrid_gi/.../bind_group_layout/bind_group_layout.rs`; failing-by-design tests `CR-SCRIPT-AUDIT-0004`, `CR-UI-TEST-0208`; the committed `zircon_editor/src/tests/workbench/project/.zircon-task-*.tmp`.
- **Guard:** `rust.unreachable-module`, `rust.uncompiled-source`, `test.weakened`, `scope.backup-artifact`.

## D11. Build output location

- **Trigger:** defaults, settings, scripts or CI that choose an output directory.
- **Rule:** Only drive-root `D:\cargo-targets`, `E:\cargo-targets` or `F:\cargo-targets` namespaces. Do not commit per-user paths.
- **Test:** the default output setting passes `tools/zircon_build.py` root validation.
- **Evidence:** `CR-HUBCORE-0001`: `zircon_hub/src/settings/paths.rs` defaults to `%USERPROFILE%\ZirconBuilds` and `zircon_hub/hub.toml` commits `C:\Users\HeJiahui\ZirconBuilds`, the supplied review reported a rejected first Hub build. Verify the current settings and marker before treating this as an open defect.
- **Guard:** `storage.output-root`.

## Guard rule index

The 22 rules below are static diagnostics. Recursive-delete containment and serde validation are heuristics; review the actual target, validator and data flow. Storage checks cover literal output assignments and legacy roots; dynamic values and physical aliases require the runtime storage gate. Test-integrity warnings count removed tests and inserted skip markers, and do not prove assertion strength. Config/read/parse errors exit 2 instead of producing a passing receipt.

| Rule | Severity | Suppressible | Meaning |
| --- | --- | --- | --- |
| `scope.unlisted-path` | error | no | Path outside the declared `--path` list |
| `scope.write-grant` | error | no | No `write` grant in the supplied permission profile; this does not prove active sandbox enforcement |
| `scope.backup-artifact` | error | no | Backup, scratch or build-output file in the patch |
| `scope.control-plane` | warn | no | Control-plane file; record the user's authorization |
| `scope.generated-adapter` | warn | no | `.opencode/skills` edited instead of `.codex/skills` |
| `scope.lockfile` | warn | no | Lockfile changed; record the dependency reason |
| `diff.conflict-marker` | error | no | Unresolved merge conflict marker |
| `marker.format` | error | yes | Malformed `BUG:` marker |
| `marker.removed-without-fix` | error | no | Marker removed by a comment-only patch |
| `marker.fix-elsewhere` | warn | yes | Marker removed from a file without a code change |
| `marker.closed-without-test` | warn | yes | Marker closed without a test change |
| `rust.unreachable-module` | error | yes | Rust file not reachable from a crate root |
| `rust.uncompiled-source` | warn | yes | Uncompiled Rust file referenced only by a path string |
| `rust.unsafe-without-safety` | error | yes | New `unsafe` without `// SAFETY:` |
| `rust.unsafe-fn-safety-doc` | warn | yes | `pub unsafe fn` without `# Safety` |
| `rust.safe-fn-raw-pointer` | error | yes | Safe `pub fn` takes a raw pointer |
| `rust.catch-unwind-under-abort` | warn | yes | `catch_unwind` under an abort profile |
| `rust.serde-bypass` | warn | yes | Derived `Deserialize` skips a validating constructor |
| `fs.unbounded-delete` | error | yes | Recursive delete without visible containment |
| `storage.output-root` | error | yes | Output outside approved `cargo-targets` roots |
| `test.weakened` | warn | yes | Tests removed, ignored or skipped |
| `input.worktree-mismatch` | error | no | Missing/deleted/mismatched source in the final patched checkout |

A suppression is a code comment containing `guard-allow(<rule>): <reason of at least eight characters>` on the flagged line or up to three lines above it; findings without a line number accept it in the file's first 40 lines.
