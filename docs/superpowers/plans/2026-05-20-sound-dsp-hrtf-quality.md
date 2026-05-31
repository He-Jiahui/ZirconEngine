# Sound DSP/HRTF Quality Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Improve deterministic sound runtime quality with stateful biquad filters and cross-block loaded HRTF FIR continuity.

**Architecture:** Neutral sound contracts stay unchanged. Concrete DSP/HRTF state lives in `zircon_plugins/sound/runtime`, with focused `engine/filter.rs` and `engine/hrtf.rs` modules to keep render orchestration from growing another responsibility.

**Tech Stack:** Rust, Cargo, existing sound runtime manager/tests, current `SoundFilterEffect` and `SoundHrtfProfileDescriptor` DTOs.

---

## Source Map

- Create `zircon_plugins/sound/runtime/src/engine/filter.rs`: biquad coefficient/state math and filter block processing.
- Create `zircon_plugins/sound/runtime/src/engine/hrtf.rs`: loaded-profile FIR rendering and HRTF render-state key/state.
- Modify `zircon_plugins/sound/runtime/src/engine/mod.rs`: register/re-export new engine modules as needed.
- Modify `zircon_plugins/sound/runtime/src/engine/dsp_state.rs`: add filter state storage to `SoundEffectRuntimeState`.
- Modify `zircon_plugins/sound/runtime/src/engine/dsp.rs`: call stateful biquad filtering instead of the old local filter helper.
- Modify `zircon_plugins/sound/runtime/src/engine/state.rs`: store HRTF render state.
- Modify `zircon_plugins/sound/runtime/src/engine/render/mod.rs`: pass source/listener/profile identity into the HRTF module and prune stale HRTF state.
- Modify `zircon_plugins/sound/runtime/src/tests/dsp_state.rs`: add filter continuity and mode tests.
- Modify `zircon_plugins/sound/runtime/src/tests/spatial.rs`: add loaded-HRTF cross-block continuity test.
- Update `docs/engine-architecture/runtime-sound-extension.md` and `.codex/sessions/20260503-0228-sound-mixer-graph-continuation.md`.

## Milestone 1: Stateful Filter DSP

- [x] Add `SoundBiquadFilterState` and `SoundBiquadChannelState` in `engine/filter.rs`.
- [x] Implement Audio EQ Cookbook coefficient tuning for low-pass, high-pass, band-pass, notch, low-shelf, and high-shelf modes.
- [x] Add `filter_state: SoundBiquadFilterState` to `SoundEffectRuntimeState`.
- [x] Replace `filter_block(...)` in `engine/dsp.rs` with `apply_biquad_filter_block(...)`.
- [x] Add tests proving low-pass continuity, high-pass DC rejection, and shelf gain behavior.

## Milestone 2: Loaded HRTF Continuity

- [x] Add `SoundHrtfRenderStateKey` and `SoundHrtfRenderState` in `engine/hrtf.rs`.
- [x] Move loaded profile convolution into `engine/hrtf.rs` and remember previous interleaved source samples up to max kernel length.
- [x] Add `hrtf_states: HashMap<SoundHrtfRenderStateKey, SoundHrtfRenderState>` to `SoundEngineState`.
- [x] Prune stale HRTF states to active source/listener/profile keys during render-state synchronization.
- [x] Keep missing profiles routed to preview fallback and do not mutate loaded-profile state.
- [x] Add tests proving FIR tails survive across one-frame render blocks and missing profile fallback remains deterministic.

## Milestone 3: Docs And Testing Stage

- [x] Update `docs/engine-architecture/runtime-sound-extension.md` with the filter/HRTF behavior, implementation files, plan sources, tests, and remaining production DSP/HRTF gaps.
- [x] Update `.codex/sessions/20260503-0228-sound-mixer-graph-continuation.md` with current status, touched modules, and validation evidence.
- [x] Run formatting:

```powershell
cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_runtime
rustfmt --check "zircon_plugins\sound\runtime\src\engine\filter.rs" "zircon_plugins\sound\runtime\src\engine\hrtf.rs" "zircon_plugins\sound\runtime\src\engine\dsp.rs" "zircon_plugins\sound\runtime\src\engine\dsp_state.rs" "zircon_plugins\sound\runtime\src\engine\render\mod.rs" "zircon_plugins\sound\runtime\src\engine\state.rs"
```

- [x] Run focused tests:

```powershell
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" dsp_state --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-dsp-hrtf-quality" --message-format short --color never
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" spatial --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-dsp-hrtf-quality" --message-format short --color never
```

- [x] Run whitespace check:

```powershell
git diff --check -- "zircon_plugins\sound\runtime\src\engine" "zircon_plugins\sound\runtime\src\tests" "docs\engine-architecture\runtime-sound-extension.md" ".codex\sessions\20260503-0228-sound-mixer-graph-continuation.md" "docs\superpowers\specs\2026-05-20-sound-dsp-hrtf-quality-design.md" "docs\superpowers\plans\2026-05-20-sound-dsp-hrtf-quality.md"
```

## Acceptance Criteria

- Filter effects keep deterministic state across render blocks.
- Filter modes use the existing `cutoff_hz`, `resonance`, and `gain_db` descriptor fields instead of ignoring resonance/gain.
- Loaded HRTF profiles keep FIR tails across render blocks.
- Missing HRTF profiles still use the preview fallback.
- Render orchestration delegates loaded HRTF rendering to a focused module instead of growing another inline DSP section.
- Docs and session notes identify remaining production DSP/HRTF gaps.
