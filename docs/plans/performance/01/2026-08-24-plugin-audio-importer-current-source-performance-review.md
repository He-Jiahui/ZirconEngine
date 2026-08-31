---
title: Plugin Audio Importer Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/audio_importer
  - zircon_runtime/src/asset
  - zircon_plugins/sound/runtime
status: static_complete_one_safe_fast_path_applied_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Source/Editor/AudioEditor/Private/Factories/SoundFactory.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Sound/SoundWave.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/SoundWave.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceBuffer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceDecode.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/SoundWaveDecoder.cpp
  - dev/Fyrox/fyrox-sound/src/buffer/mod.rs
  - dev/Fyrox/fyrox-sound/src/buffer/loader.rs
  - dev/Fyrox/fyrox-sound/src/buffer/streaming.rs
  - dev/Fyrox/fyrox-sound/src/decoder/mod.rs
---

# Plugin Audio Importer Current Source Performance Review

## 1. Coverage and evidence state

The primary package review covers **4/4 Rust files**, **1,196 physical / 1,082 non-empty lines**, **44,597 bytes**, **16 tests** and **3 ignored performance tests**. The package-relative `path + NUL + LF-normalized bytes + NUL` SHA-256 is `e26176ccc4aa8f31414ecadb20b393667a364ccac07b7ac65be47e4c3495c2cf`.

| Area | Rust files | Physical lines | Current execution truth |
|---|---:|---:|---|
| Dist | 1 | 108 | Exports registration/lifecycle ABI state, but `invoke_command` is absent; native import execution is not implemented. |
| Runtime capability | 1 | 35 | Advertises Stable source/library/native capability without a native callable import bridge. |
| Runtime importer | 2 | 1,053 | WAV and Symphonia codecs synchronously decode a complete source into interleaved resident `Vec<f32>` samples. |

The related product path was reviewed from `SoundAsset` construction through artifact staging, typed loading and Kira playback conversion. Two primary-package files, `runtime/src/lib.rs` and `runtime/src/plugin.rs`, already contained shared uncommitted work and were read but not edited or formatted. Per-file `rustfmt --check --edition 2021` passes **4/4** primary-package files.

Managed Rust tests, WPR/ETW and RenderDoc were not run. This session has no executable managed Windows validator identity and no launchable current-source engine/editor binary. RenderDoc is not a CPU audio-import profiler and is only relevant later if audio drives visible render-side debug products. The findings below prove data movement, ownership and scheduling defects from source; they do not claim measured latency, power or competitor parity.

## 2. Structural performance findings

### P0: the advertised importer is not selected by the first-party Runtime product

The importer registers WAV at priority 120, MP3/OGG/FLAC/AIFF at priority 90 and an Opus diagnostic entry at priority 80. The first-party Runtime catalog declares an `AudioImporter` row, but has no `RuntimePluginId::AudioImporter` provider branch and no dependency/feature for `zircon_plugin_audio_importer_runtime`. Normal product selection therefore cannot execute this provider even though capability metadata says Stable and native.

This is an authority defect, not startup micro-cost. First establish exactly one executable format-provider path and source/library/native parity; until then, capability and profile output must report the provider as unavailable. Dist lifecycle callbacks and a global epoch do not substitute for an import bridge.

### P0: every successful non-WAV import synchronously materializes the whole decoded stream

The Symphonia path clones the complete compressed `source_bytes`, probes it, then loops through every packet and appends converted samples to one resident `Vec<f32>`. One `SampleBuffer<f32>` scratch allocation is reused and initial reservation is capped, but there is no cap on total decoded samples, decoded bytes, channel count, duration or expansion ratio. WAV similarly ends as a fully decoded `SoundAsset`.

The current code has no import policy for static versus streaming clips, target codec, sample-rate conversion, quality, chunk size, seek data, loop/cue metadata, normalization or platform profile. Long music and tiny UI sounds are forced through the same product shape.

Unreal separates authoring raw/source data from platform compressed data. `SoundWave` owns streamed platform chunks, derived-data keys, asynchronous cache work, decompression policy and loading behavior such as retained, inline, on-demand or streaming. Fyrox independently exposes `SoundBufferImportOptions { stream: bool }` and distinct generic versus streaming buffers. Zircon needs the same explicit product decision before tuning a decoder loop.

### P0: decoded expansion has no admission budget or adversarial-input fence

Bounding only initial vector reservation does not bound the vector. A small compressed input can declare a long duration, excessive channel count/sample rate or highly compressible content and expand until memory pressure or OOM. Packet decode errors are skipped, so partial success can also hide corruption without a deterministic source diagnostic.

Admission must validate source bytes, stream count, sample rate, channels, duration, frame count and decoded byte ceiling before publication. Decode must carry cancellation/deadline, report rejected/corrupt packets, and stop when its budget is crossed. The artifact receipt must include declared versus produced frames and peak admitted bytes.

### P0: one canonical clip expands into multiple full PCM representations

The ownership chain amplifies a decoded clip:

1. `SoundAsset` owns interleaved `Vec<f32>` samples.
2. Artifact preparation clones the complete `SoundAsset` before bincode/zstd encoding.
3. Typed asset loading clones the value behind the resource handle.
4. `LoadedClip` retains that cloned `SoundAsset` in an `Arc`.
5. Kira conversion builds another full `Vec<Frame>`; mono is expanded to stereo and stereo still copies equivalent sample bytes.

A 10-minute, 48 kHz stereo f32 clip is `600 * 48,000 * 2 * 4 = 230,400,000` bytes, or **219.7 MiB**, per PCM-equivalent representation. Import artifact staging can therefore retain about **439.5 MiB** for the imported asset plus its cache clone, excluding source, decoder scratch and encoder buffers. First playback can retain roughly **659.2 MiB** across the resource asset, loaded-clip clone and Kira frames, again excluding compressed source and allocator overhead.

This is a lower-bound ownership model, not an RSS measurement. The target is one immutable canonical decoded payload for short clips, referenced by leases/`Arc`, and bounded compressed chunks plus a small decode ring for long clips. Artifact serialization must consume borrowed/moved payloads or a streaming encoder, never require an entire second PCM owner.

### P0: importer and playback channel contracts disagree

The importer accepts codec channel layouts beyond stereo, while `stereo_frames` rejects more than two channels with `UnsupportedAdvancedFeature`. This defers a deterministic platform/backend incompatibility until playback after import, cache and load work have already completed.

Cook policy must resolve source layout into a target-specific payload: preserve supported multichannel layouts, explicitly downmix with a versioned matrix, or reject during import/cook. Channel layout, target backend, sample format/rate and codec belong in the artifact key and diagnostics.

### P0: decode, conversion, artifact publication and first-play conversion lack a bounded task graph

Import runs on one synchronous call stack. There is no phase job for probe/header validation, decode/chunk cook, waveform/seek analysis, artifact encode or publication; no queue priority, cancellation, progress, byte admission or stale-generation rejection is visible in this package. First playback performs an O(sample count) Kira conversion synchronously on the load path.

Unreal's audio mixer uses asynchronous header/decode work, real-time or streaming decompression modes and bounded queued decode buffers. Zircon should submit admitted audio work to the shared Runtime/Editor job authorities, keep the main/editor thread to request/publication work, and make first-play readiness observable rather than hiding conversion in `load_clip`.

### P1: compressed source is cloned before decoding and full PCM is zstd-compressed as a generic artifact

The non-WAV path passes `context.source_bytes.clone()` into the decoder even though the import transaction already owns an immutable source snapshot. After full decode, generic artifact staging serializes PCM with bincode and zstd level 1. This pays a compressed-source copy, full decode allocation, full cache clone and generic compression pass without producing a seekable platform audio product.

The canonical recipe should retain one immutable source lease, derive platform codec/chunk/seek products under content-addressed keys, and only generate resident f32 for clips whose duration/budget policy selects static playback. Editor waveform/loudness products should be independent cached recipes rather than another reason to retain PCM in the base asset.

### P1: repeated clip loads rebuilt and discarded the full playback representation

Before this review, `load_clip_impl` checked `clip_ids_by_locator` only after resolving/loading the asset and constructing `LoadedClip`. A repeat hit therefore performed a resource lookup, full `SoundAsset` clone and O(samples) `Vec<Frame>` conversion, then discarded the result when the final map lookup found the existing ID.

The safe local fix adds a short locked cache lookup immediately after locator parsing and before project asset-manager resolution, while retaining the final check for concurrent misses. The regression `cached_clip_load_returns_before_project_asset_resolution` constructs a manager without `CoreRuntime`; the cached path succeeds only if it returns before resolution. Static ordering now proves cache lookup precedes manager/load/conversion. Dynamic timing is pending because the managed validator is unavailable.

Concurrent misses can still duplicate load/conversion work. The structural follow-up is an in-flight entry keyed by canonical asset identity and generation, with waiters sharing one terminal result and cancellation/refcount semantics.

### P1: current tests and ignored benchmarks do not cover the product bottleneck

The package tests cover codec fixtures and fragments such as registration, scratch reuse and reservation behavior. They do not measure source clone bytes, total decoded expansion, long-clip peak RSS, artifact copy/encode bytes, typed-load copy bytes, first-play conversion, queue/main-thread time, concurrent duplicate work, cancellation or energy.

Required fixtures include short UI clips, 10-second effects, 10-minute music, mono/stereo/surround, high-rate inputs, corrupt/truncated streams and compressed expansion bombs. Counters must record source/decoded/cooked/resident bytes, allocations, queue wait, worker/main CPU, chunks, cache result, duplicate-work suppression, underruns, cancellation latency and power per admitted/imported minute.

## 3. Reference-engine constraints

Unreal is the primary architectural constraint:

- `SoundFactory.cpp` imports authoring source into `SoundWave`, invalidates derived compressed data and interacts with streamed zeroth-chunk state rather than declaring one universal resident f32 product.
- `SoundWave.h/.cpp` defines compressed-data policy, per-platform formats, streamed chunks, derived-data keys, asynchronous cache tasks and loading behavior.
- `AudioMixerSourceBuffer.cpp`, `AudioMixerSourceDecode.cpp` and `SoundWaveDecoder.cpp` separate real-time/streaming decode from source management, use asynchronous decode/header work and refill bounded buffers.

Fyrox is a secondary Rust implementation reference. Its buffer loader makes streaming an explicit import option, generic buffers share heavy sample data, and streaming buffers retain a decoder while filling bounded blocks. Its decoder duration scan is not a target to copy; Zircon must avoid a mandatory whole-file scan when metadata/index products can provide duration and seek information.

The resulting constraint is: source authority, import settings, cooked platform payload, residency policy and playback decode are separate stages with separate identities. A synchronous importer returning universal f32 samples cannot remain the system boundary.

## 4. Dependency-ordered optimization plan

### M0: make provider and capability truth executable

Wire exactly one AudioImporter provider through the first-party catalog, dependency/features and profile readiness. Implement source/library/native execution parity or downgrade unsupported capability. Resolve WAV/MP3/OGG/FLAC/AIFF/Opus authority and remove diagnostic/provider shadows in one hard cutover.

### M1: define versioned audio source, import settings and cook products

Keep an immutable source lease and source provenance. Add settings for static/streaming policy, duration threshold, target codec/quality/rate/layout, chunk/seek configuration, loop/cue metadata and normalization. Define short resident clip, streamed platform chunks, seek/index and editor waveform/loudness products with independent artifact keys.

### M2: add admission, corruption and expansion budgets

Validate declared and produced frames/channels/rate/duration/bytes. Enforce configurable source, decoded, scratch and artifact budgets with typed diagnostics. Add cancellation/deadline and deterministic corrupt-packet policy; partial decode cannot silently publish as a healthy clip.

### M3: schedule bounded import and derived jobs

Split probe/header, chunk decode/cook, waveform/loudness, artifact encode and publication into shared scheduler jobs. Carry source revision, recipe key, target/profile, priority, byte reservation and cancellation. Publish only current-generation terminal results and preserve last-good artifacts on failure.

### M4: converge ownership to one payload per identity

Move or share immutable short-clip samples across import cache, resource handle and playback. Remove deep clones from artifact staging and typed load. Long clips retain compressed chunk leases and a bounded decoder ring whose memory is independent of clip duration. Coalesce concurrent same-key load/convert work.

### M5: cook backend-compatible playback data

Resolve channel layout, sample rate/format and codec during target cook. Support explicit multichannel/downmix policy. Feed Kira/backend from the canonical short payload or streamed decoder without rebuilding an equal-size frame array on every load. Keep callback work allocation-, lock- and I/O-free.

### M6: instrument and dynamically qualify the current-source product

Emit phase timings, bytes, allocations/peak RSS, queue/main CPU, cache/lease state, first-play latency, decode-buffer occupancy, underruns, cancellation and power receipts. Run fixed cold/warm/reimport/first-play/repeat-play/concurrent-load fixtures on a launchable current-source binary. Use WPR/ETW for CPU, I/O, scheduling, memory and power; use backend audio telemetry for buffer behavior. RenderDoc is not an audio acceptance tool.

## 5. Acceptance gates

| Gate | Required evidence |
|---|---|
| A1 | Exactly one product-selected importer executes every advertised format; capability cannot claim native without a callable bridge and parity receipt. |
| A2 | Import settings and target profile deterministically select resident or streaming products and participate in artifact keys. |
| A3 | Corrupt, truncated and expansion-bomb fixtures terminate within declared decoded-byte/frame/time budgets without partial healthy publication. |
| A4 | A short clip has at most one canonical PCM payload per asset generation; artifact/load/playback paths expose zero full-payload deep clones. |
| A5 | Long-clip steady-state memory is bounded by configured chunks/decode buffers, not proportional to total duration. |
| A6 | Repeat `load_clip` cache hit performs locator validation plus bounded map/lease work, with zero asset load, decode or frame conversion. |
| A7 | Concurrent same-key misses execute one load/cook/conversion and share one generation-bound terminal result. |
| A8 | Import/cook rejects or explicitly converts unsupported channel layouts before playback; source and backend layouts are recorded. |
| A9 | Decode/cook/waveform/artifact work runs on admitted workers with cancellation and stale-generation rejection; main/editor thread residency is within a declared budget. |
| A10 | Fixed fixtures report cold/warm import, artifact bytes, first/repeat play latency, peak RSS, worker/main CPU, underruns and energy on a current-source executable. |

## 6. Validation record

- Static package coverage: complete, 4/4 Rust files; related asset/cache/playback chain reviewed selectively to the terminal Kira representation.
- Formatting: primary package 4/4 per-file checks pass; the edited `clip_assets.rs` formats cleanly.
- Direct change: early repeated-load cache hit plus one regression test in `zircon_plugins/sound/runtime/src/service_types/clip_assets.rs`; final concurrent-miss check retained.
- Rust tests: not executed because the managed Windows validator identity is unavailable; no raw Cargo fallback was used.
- WPR/ETW/RenderDoc/power: pending until a launchable current-source product exists. No absolute performance or parity claim is accepted from this static review.
- Shared worktree: pre-existing audio-importer modifications were preserved and not reformatted.

