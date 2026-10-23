# Scripted editor Play

Build `zircon_app`'s `zircon_editor` binary with the opt-in `editor-scripting`
feature in addition to the editor/profiling features required by the task.
It includes `backend-zr-vm` and the first-party ZrVM runtime provider. The
default editor feature set retains its existing dynamic Play route.

The project must select `zr_vm_language` for `client_runtime`, declare its
`[scripts]` package roots/startup packages, and attach scene script bindings.
Selecting the provider for `editor_host` alone does not enable it in Play.
Required providers that cannot be resolved still reject Play admission.
Resolved providers use the project's enabled selection instead of the plugin's
disabled-by-default descriptor value; their supported targets remain unchanged.

Scripted Play resolves the current project's runtime selections on each launch.
The host supplies those registrations to an in-process runtime session using
the editor's materialized scene snapshot. Rust registrations never cross the
dynamic runtime ABI. All gateway operations and allocation releases use that
same linked API table/registry. The normal session owner verifies the module
composition receipt and retains failures for creator-thread cleanup. Staged
editor/runtime BuildSet preflight remains required for the authoring gateway.

The existing EmbeddedPlayBackend still owns snapshot retention and Stop/retire.
Gameplay mutations affect the Play world; stopping restores the editor world.
An external gateway reference prevents retirement until that owner releases it.
The built-in `script.bindings` reflection adapter preserves enabled/update flags
and property JSON in Play snapshots. Scene TOML continues to use its existing
script-binding representation. Structured GUI attachment remains a separate gap.

## Linux build exception

The pinned ZrVM sys build originally accepts Windows output roots only. The
local MIT sys package in `tools/vendor/zr_vm_rust_binding_sys` preserves the
upstream ABI sources and adds an explicit Linux output-root exception.
`UPSTREAM.json` records provenance and original hashes. The root Cargo patch
substitutes the sys package and its small safe binding. The safe binding's
visitor states provide explicit trampoline types to repair upstream E0283
errors on stable 1.99.1 while preserving public signatures. Both packages have
original hash records; the native source revision remains pinned.

Set `ZIRCON_LINUX_CARGO_TARGET_ROOT` only under a user-authorized Linux task;
it must name a physical direct child of `/workspace/cargo-targets`. Set
`CARGO_TARGET_DIR` to that root and compiler temporary paths below it. All
compiler-output containment/alias checks remain active. Linux's Cargo source
registry (`CARGO_HOME`) remains separate from compiler products; Windows keeps
its original source-registry location gate. Without this opt-in, the
original Windows policy applies. `ZR_VM_RUST_BINDING_LINK_KIND=static` builds
the serial native dependency into `OUT_DIR` and avoids an extra runtime shared
library in the editor stage. Native source is read from the pinned Cargo Git
checkout; that checkout is never patched.
The local Linux CMake recipe adds only the native binding target to upstream's
module graph, validates its physical output namespace, and exports the target's
static/system dependency closure. It does not run recursive Cargo workspaces.

## Focused regression

After building and staging the editor, declare and run the `zircon_app` library
test `scripted_play_runs_materialized_snapshot_and_retires` with defaults
disabled and the same editor/scripting/profiling features. Pass `--ignored
--test-threads=1` explicitly, with `ZIRCON_RUNTIME_LIBRARY` and
`ZIRCON_ASSET_ROOT` selecting that admitted stage and `TMPDIR` in the task's
target directory. It copies the supplied runtime into a private test stage, pairs its manifest
with the actual test executable hash, and runs normal preflight validation.
The original editor stage is untouched. It creates its own project, proves `onStart` and `onUpdate`
execute on a snapshot differing from the persisted scene, checks two session
identities, verifies outstanding gateway ownership blocks retirement, and
checks the persisted scene remains unchanged.

The sys output-policy test is a standalone Rust test target rooted at its
`build.rs`; compile it with `rustc --test` into the authorized target directory.
It checks the permitted physical root and rejects outside/lookalike paths and
an in-root symlink. These scoped results do not constitute Jenkins acceptance.
