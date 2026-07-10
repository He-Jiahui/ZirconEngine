# Frameworks 03 Sound Contract Feature Boundary Acceptance

## Scope

- Add independent Runtime/App `sound-contracts` features.
- Include the contract in Client and Editor presets while keeping Server free of implicit Sound services.
- Gate the Sound framework and manager trait/name/holder/resolver at declaration and assembly boundaries.
- Require direct Sound runtime/editor consumers to request the contract explicitly.
- Hard-migrate channel topology from the optional Sound service to the neutral `core::framework::audio` owner.
- Remove the old `SoundChannelLayout`, `SoundSpeakerChannel`, and `core/framework/sound/channel_layout.rs` architecture without aliases.

## Static Evidence

- `python tools/tests/test_frameworks_03_contract_feature_boundary.py`: 12/12 passed.
- Old symbol and old public-path scan: zero matches in production Rust.
- `git diff --check` over the scoped implementation/doc set: passed with line-ending warnings only.

## WSL Nightly Compile Evidence

- Runtime contract alone: `cargo check -p zircon_runtime --lib --no-default-features --features sound-contracts --locked --offline --jobs 1` passed in 7m30s.
- Server exclusion: `cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --offline --jobs 1` passed in 3m39s.
- App Server exclusion: `cargo check -p zircon_app --lib --no-default-features --features target-server --locked --offline --jobs 1` passed in 6m49s.
- App default Client: `cargo check -p zircon_app --lib --locked --offline --jobs 1` passed in 9m19s with an unprivileged temporary `libudev-dev` sysroot. The first attempt stopped before Zircon compilation because WSL did not provide `libudev.pc`.
- Sound runtime plugin: `cargo check -p zircon_plugin_sound_runtime --locked --offline --jobs 1` passed in 3m53s.
- Sound editor plugin: `cargo check --manifest-path zircon_plugins/sound/editor/Cargo.toml --locked --offline --jobs 1` passed in 18m34s after the locked editor dependencies were fetched.
- Audio importer without Sound: `cargo check --manifest-path zircon_plugins/audio_importer/runtime/Cargo.toml --locked --offline --jobs 1` passed in 4m05s.
- Sound runtime full test target: `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --no-run --locked --offline --jobs 1` compiled successfully in 15m50s.
- Sound runtime channel-layout focus: the same manifest with filter `channel_layout` passed 2/2 in 6m36s; 366 unrelated tests were filtered out.
- Sound runtime full suite: the same manifest without a filter passed 368/368 with zero failures; test execution took 0.18s and the full Cargo command took 8m12s after concurrent workspace-manifest changes triggered recompilation.

The first editor attempt stopped before compilation because `atk 0.18.2` was missing from the offline cache. A locked online retry fetched dependencies but showed that the root workspace does not register the editor package; the package-manifest command above then performed and passed the actual source check. These pre-source environment/command failures are not counted as code passes or failures.

## Remaining Scope

- Physics contract/data ownership and feature gate remain pending because an active plugin session owns the required Physics manager import files.
- Frameworks 03 M1 full Runtime/App tests and the complete per-domain matrix remain pending.
