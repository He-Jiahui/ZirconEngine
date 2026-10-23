# Editor startup splash and welcome — Linux validation

Owner: John He. Branch: `fix/editor-splash-main-20261022`; base: `9e5918994`.

The editor owns a separate native splash process while runtime and GPU startup
block. Known stages advance monotonically at actual startup boundaries. The
first successful workbench presentation dismisses and reaps the child. Escape
hides the informational card while startup continues; startup errors and parent
EOF also close it. The welcome form keeps its existing project command bindings, with explicit
localized Open Project/Create Project labels.
The welcome probe schedules a lifecycle wake for debounce admission and result
collection, requests a document-region repaint when publishing its result, then
releases the wake when settled. Captions and fields share compact
rows; the standalone pane resolves its authored responsive display props before
measurement, collapsing recents below 900 logical pixels and optional intro/path
content below 600. The compact heading and project actions precede
supplementary guidance in wide, short panes. Composited caption pixels have regression coverage.

Design references: [App Dialog](https://pro.reactbits.dev/docs/app-ui/app-dialog),
[Agent Activity 3](https://pro.reactbits.dev/docs/app-ui/agent-activity/agent-activity-3),
[App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4), and
[Empty State](https://pro.reactbits.dev/docs/app-ui/empty-state). Shared Zircon
tokens, authored welcome flow, and existing command/first-present owners govern
the implementation; no external source or assets were copied.

## Reproducible scope

Fresh post-reboot evidence is retained under
`/workspace/ZirconEngine-review/evidence/splash-20261023/`. The detailed slice
record is `/workspace/ZirconEngine-review/editor-ui-splash-main-20261022.md`.
Earlier `/tmp` receipts and staged products were lost and supply no fresh pass.

Canonical compiler: unwrapped stable Rust 1.99.1, Linux x86_64. Cargo uses one job,
`CARGO_TARGET_DIR=/workspace/cargo-targets/ze-ui-splash`, and that target's `tmp`
directory for compiler temporary files. Dev/test profiles use debug=0 and disable
incremental compilation. Both compiler wrapper variables are unset. Compiler
processes preload system jemalloc; the test runner removes that preload before
native execution and mounts private application data at the unchanged `~/.zircon`.
Product runs use no allocator preload. The retained-model admission check remains
unchanged and reports the audited Linux model available.

```sh
cargo +stable build -p zircon_app --no-default-features \
  --features target-editor-host --bin zircon_editor --locked --jobs 1
cargo +stable rustc -p zircon_runtime --no-default-features \
  --features target-editor-host --lib --crate-type cdylib --locked --jobs 1
cargo +stable test -p zircon_editor --no-default-features --lib --locked --jobs 1 -- \
  startup_splash tests::ui::welcome::bootstrap_assets \
  editor_startup_session_document_welcome_pane_snapshot first_present::tests \
  project_chooser_preserves_first_run_guidance_without_opening_a_project \
  startup_session_defaults_to_the_project_chooser_without_recent_project \
  startup_session_keeps_valid_recent_projects_in_the_chooser \
  core::i18n::service::localized_text_tests welcome_session::project_probe \
  host_lifecycle::tests::tick welcome_form_captions_survive \
  app::tests::welcome_probe welcome_form_header compact_welcome_header \
  --test-threads=1
python3 -B -m unittest tools.tests.test_editor_zui_welcome_visual_density_contract -v
```

The editor library has no selected local features; its declared Runtime
dependency enables `target-editor-host`, and its test dependency adds
`test-support`. These commands cover the splash protocol/painter/locales,
welcome geometry, captions, probe deadlines and guidance, first-present
notification, and real chooser startup. They do not select a workspace or all-target build.

Native smoke uses X11 `DISPLAY=:3`, `WGPU_BACKEND=vulkan`, and
`ZIRCON_ALLOW_SOFTWARE_ADAPTER=1`. Each run has private application configuration,
UI and Mesa caches. Shared OS page cache is retained. The smoke harness captures
only an owned X11 drawable and sends input only to its launched windows.

## Results

The final focused Rust batch passes **49/49** tests; the Python welcome
structural suite passes **13/13**. Formatting, AST/TOML parsing and scoped
whitespace checks pass. The final Change Guard content scan covers 40 source
files with zero errors, warnings or suppressions; it makes no write-grant or
module-reachability claim. Receipt prefixes are `cargo-focused-tests-05`,
`welcome-structural-final-02`, `structural-checks-final-02`,
`structural-checks-present-handoff`, and `change-guard-delivery` under the durable
evidence root. The entry binary's unchanged baseline formatter drift is retained.
Existing compiler warnings remain visible in the logs.

The final host build (`cargo-build-host-04`) and runtime build
(`cargo-build-runtime-01`) pass. `stage-final-03` contains their canonical bytes.
`source-validation-reconciliation.json` verifies all 40 source hashes against
the build/test input manifest.

Native verification passes eight product cases, three child protocol cases and
two expected startup failures:

- Welcome Create and Open at 1672×941, 1672×420 and 560×384. A 640×420 capture
  also verifies the short layout. Captions remain visible, recents collapse,
  actions fit, and probe results repaint without another input or resize.
  Create asserts the actual project manifest; both actions assert the intended
  project in each fresh, isolated recent-project registry.
- CLI Create (`create-final-04`) and CLI Open of a welcome-created project
  (`open-escape-final-03`) reach real first workbench presentation and later
  visible scene pixels, then exit 0. The first-frame hook can capture the
  workbench before scene pixels arrive; `workbench.png` verifies the latter.
- Every product case observes the splash, resize acknowledgment, real GPU and
  project stages, closed acknowledgment and child reaping before quit. Open's
  Escape case hides/reaps the card while the parent is still loading and then
  reaches its workbench.
- The final child protocol passes stage/Ready, parent EOF and Escape, all exit 0.
  Missing runtime and invalid project return 1, close/reap their exact children,
  and never publish Ready.

`native-validation-summary.json` indexes those receipts and visual reviews.
`welcome_probe.py`, its JSONL input plans, `probe_failures.py`, command receipts,
native stderr and screenshots are retained in the evidence directory.

The first final CLI Create attempt (`create-final-03`) failed its 240-second
bound. Five-second resource samples jump from 15.00 to 332.55 seconds. The
harness terminated its owned launch, followed by a child broken-pipe diagnostic.
Its PIDs are gone. This attempt is excluded; a fresh run with unchanged bytes
and a 900-second bound passed. See `create-final-03-diagnostic.json`.

## Final measurements and artifacts

These are individual dev-build observations on shared software Vulkan/llvmpipe.
Fresh configuration and UI/Mesa caches do not evict the OS page cache. Native
markers and harness capture latency are separate; no paired speedup is claimed.

| Case | Splash visible after launch | First workbench after splash launch | Visible duration |
| --- | ---: | ---: | ---: |
| Normal welcome / Create | 91.01 ms | 5,573.77 ms | 5,482.77 ms |
| CLI Create | 189.45 ms | 30,071.25 ms | 29,881.81 ms |
| CLI Open / Escape | 114.83 ms | 27,397.99 ms | 305.54 ms |

Escape's card closed at 420.40 ms while loading continued. Across the six welcome
runs, resize acknowledgment took 3.76–24.05 ms; screenshot latency includes an
intentional 150 ms repaint wait. Normal welcome idle over 20.000 seconds observed
0 process CPU ticks (0.0% of one core at the counter's resolution), RSS
408.195 MiB and PSS 404.540 MiB. CLI Create's loading sample records parent
RSS/PSS 70,228/49,898 KiB and splash RSS/PSS 77,112/55,536 KiB separately.

`artifact-measurements.json` measures logical, unique-inode and allocated bytes.
Final product inodes are excluded from shared Cargo groups. Embedded symbols
are already included in product sizes; they are not additional package bytes.

| Group | Logical bytes | Unique-inode bytes |
| --- | ---: | ---: |
| Editor ELF | 488,979,464 | 488,979,464 |
| Runtime ELF | 256,970,720 | 256,970,720 |
| Staged package, including assets/manifest | 785,537,103 | 785,537,103 |
| Shared Cargo dependencies | 6,681,425,462 | 6,681,425,462 |
| Build-script products | 148,700,438 | 85,121,086 |
| Cargo fingerprints | 19,139,969 | 19,139,969 |
| Other Cargo root files | 1,947,188 | 1,947,188 |
| Incremental cache / compiler temporary files | 0 | 0 |

Editor static/dynamic symbol sections occupy 227,196,346/15,262 bytes; runtime
sections occupy 125,733,707/5,928 bytes. Both have zero DWARF bytes and no separate
symbol sidecars. Assets occupy 39,584,989 logical bytes. Nothing was stripped or
deleted. These are dev artifacts, not release-size acceptance.

Canonical SHA-256:

```text
zircon_editor       ee0ce2a14f39aff7328250c62127d355163b5cfda474b6f08f8bc49e2a55b08a
libzircon_runtime.so 236a64273fcc105e4ab6ec1db44d9a00302048685c4d12f77d3d7dc957339cbb
```

The existing projectless software-GPU presenter timeout remains visible. Welcome
actions activate real projects, but that fallback can leave their viewport blank;
CLI Create/Open separately verify scene rendering. Generic main-panel scrolling
beyond the tested minimum windows, GPU sharing and unrelated icon rendering are
not accepted by this slice. The primary project actions fit the tested windows.

These results supply Linux-local evidence only. Windows, Jenkins, integrated
engine acceptance, hardware-cold or release performance, and a controlled
before/after comparison remain outside this slice. Play controllers and layout
scheduling/coalescing are unchanged. `.codex/config.toml` remains uncommitted.
