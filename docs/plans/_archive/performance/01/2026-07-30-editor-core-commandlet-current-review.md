# Editor core commandlet current-source review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Owners: Editor16 owns CLI/commandlet routing; Editor08 owns the canonical command registry; Editor10 owns asset migration; Editor12 owns plugin projection; Runtime11 owns bounded output/I/O policy.
- Accounting: keep `zircon_editor/src/core/commandlet/**` in `pending.md`. Headless execution intentionally does not instantiate the UI host, so synchronous task wall is not a GUI-frame defect; duplicate startup/report ownership still needs scale acceptance.
- Code disposition: no Rust source was changed. Existing tracked dirty source/tests were preserved.

## Exact scope

| module | files | physical lines | inline tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/commandlet/**` | 3/3 | 829 | 10 | `8e38d14dc5b45b0d011c7dc71e7687896963f9ed500bb5ebfc012567d000c817` |

All three files were read in full. Product reachability was traced through `EditorLaunchArgs`, editor entry routing, canonical command descriptors, asset migration and plugin catalog projection. The fingerprint streams each sorted native workspace-relative path, a zero byte, raw bytes and a zero byte into SHA256.

## Per-file review

| file | current-source performance result |
|---|---|
| `mod.rs` | Module wiring and exports only. |
| `runner.rs` | Headless routing correctly exits before GUI startup and plugin-list reuses a shared catalog projection. Remaining parse path copies all args in `EditorLaunchArgs`, clones them into a second Vec in `parse_commandlet_args`, then constructs the full default command registry; execution constructs the registry again and both name/route queries scan all descriptors. Asset migration maps every changed path/issue/message into a second owned report before entry code serializes the whole envelope into another String for stdout. |
| `tests.rs` | Ten tests cover canonical registration, shared plugin projection, capabilities, migration dry-run/apply, argument errors and runtime failure. No command-count/argument-width startup counter or 1M-row migration report/RSS/streaming-output test exists. |

## New tasks

### PERF-MVP-598: commandlet startup rebuilds stable input twice

One commandlet invocation creates two complete argument Vecs, builds `EditorCommandRegistry::default_workbench()` during parse, builds it again during execution and performs linear name/route scans. This is one-shot startup work, not a frame hot path, but plugin-contributed commandlets will amplify it. Parse once into a typed launch owner and resolve a generation-bound commandlet descriptor/token from the same canonical immutable registry projection consumed by execution.

### PERF-MVP-599: migration reporting owns the result three times

The runtime migration report owns all rows. `migration_report` creates another Vec/String representation for every changed file and issue, then `serde_json::to_string` creates a full envelope String before `println!`. Large migrations therefore retain runtime rows, commandlet rows and encoded JSON at once. Keep one shared report owner and stream a borrowed serialization projection to locked stdout with byte/error accounting; preserve the stable JSON envelope and exit code.

## Acceptance plan

- Startup: args 1/100/10K, commands 2/100/10K and contributed commandlets 0/100/10K. Count argument String owners/cloned bytes, registry builds/descriptors/validation, name/route visits and startup wall/RSS. Commandlet args have one owner, registry builds once per generation and resolution is direct.
- Migration output: changes/issues 0/1/1K/1M, paths/messages 16B/4KiB and stdout fast/10ms/blocked. Record runtime/report/JSON owners, cloned bytes, encoded bytes, peak RSS, first-byte/total wall and write errors. Row duplication and full encoded String must be zero; stable JSON bytes and exit codes remain exact.
- Run current-source managed commandlet/lib and app CLI lifecycle tests, then subprocess dry-run/apply/plugin-list with small/large fixtures. No rendering occurs, so RenderDoc is not applicable.

## Reference check

- Unreal `dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Commandlets/Commandlet.h` defines explicit client/editor/console/error-count policy for commandlets; Zircon similarly keeps one headless outcome contract instead of starting UI services.
- Godot `dev/godot/main/main.cpp` makes headless/export options explicit and avoids a display driver. Zircon's early commandlet routing already follows this principle and should retain it while removing duplicate parse/registry work.
- Bevy's shared task-pool ownership remains the reference for future long-running commandlets; a commandlet-specific private pool is not warranted.

## Static gates executed

- Read all current 3/3 Rust files and the listed production caller chain.
- `rustfmt --edition 2021 --check` passed all three files.
- `git diff --check -- zircon_editor/src/core/commandlet` passed with existing LF-to-CRLF warnings only.
- `review.md` remained unchanged. No managed Cargo, subprocess/scale/RSS/output trace or independent dynamic review ran.
