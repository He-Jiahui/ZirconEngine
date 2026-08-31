---
title: Plugin Opus Importer Current Source Product Gap Review
date: 2026-08-23
scope:
  - zircon_plugins/opus_importer
status: static_complete_product_missing
canonical_owners:
  - docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md
references:
  - dev/UnrealEngine/Engine/Source/Developer/AudioFormatOpus/Private/AudioFormatOpus.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/OpusAudioDecoder/Module/Private/OpusAudioInfo.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/OpusAudioDecoder/Module/Public/Decoders/OpusAudioInfo.h
---

# Plugin Opus Importer Current Source Product Gap Review

## 1. Coverage

The current `zircon_plugins/opus_importer` Rust surface is **4/4 files**, **493 physical / 439 non-empty lines**, **18,588 bytes**, and **8 test markers**. Its workspace-relative `path + NUL + raw bytes + NUL` SHA-256 is `279dd13422636666ff969f4e69c490d0acf53375fa669dd6094f51332e0bf20c`. Runtime capability/declaration, plugin, inline tests, native dist entry, Cargo manifests, generated plugin manifest and first-party catalog projection were read directly. The folder is clean in the shared worktree.

## 2. Primary finding

There is no Opus import or decode algorithm to optimize. The runtime registers `DiagnosticOnlyAssetImporter`; every import returns the stable error that a NativeDynamic libopus backend is required. The dist ABI advertises a registration manifest but has no command invocation, bridge method, decode function, state or unload hook. Its diagnostic explicitly says the importer remains hosted by the runtime module, while that runtime module contains only the diagnostic placeholder.

The plugin manifest advertises `.opus -> Sound`, native-dynamic packaging and `runtime.asset.importer.audio.opus`. `RuntimePluginId` recognizes Opus, and generated manifest tests include it, but `first_party_runtime_catalog` has no Opus dependency or provider slot. A project can therefore name a valid first-party ID whose source runtime is not selected by the compiled first-party catalog; loading the standalone dist still cannot decode. Capability/schema existence is being confused with product availability.

This is a correctness/readiness P0, not a measured performance bottleneck. Caching registration reports, parallelizing the diagnostic importer or tuning lookup branches would make the missing product harder to see without importing one byte of audio.

## 3. Unreal source constraints

Unreal separates cook-time Opus formatting from runtime decoding. `AudioFormatOpus.cpp` declares parallel cook support, derives a supported sample rate, uses 20 ms frames, records true samples/channel/rates/frame count/pre-skip/silence in a versioned header, encodes bounded frames and stores each compressed frame length. `OpusAudioInfo.cpp` validates/owns decoder state, consumes compressed chunks incrementally, returns compressed bytes consumed and PCM frames/bytes produced, retains only unused decoded output, supports streaming and seek, and resets pre-skip/leftover state on loop.

The transferable constraints are: explicit format/version/header, bounded frame/chunk work, stateful decoder ownership, incremental consumption, streaming/seek semantics and measured parallel offline work. Zircon must not copy Unreal's custom container or C++ allocation patterns; source `.opus` container handling and derived asset format must be defined for Zircon.

## 4. Dependency-ordered implementation plan

### M0: capability truth and fail-closed selection

Until a backend is loaded and validated, report the Opus importer as `Unavailable(MissingBackend)` and do not publish a Ready importer contribution. First-party catalog/profile projection must return a typed missing-provider result for an enabled Opus selection rather than silently omitting it. Native dist negotiation must prove the decode capability it provides.

### M1: proven codec/container backend

Use a maintained libopus plus the appropriate `.opus` container parser through one backend owner. Define ABI/version, supported platforms, channel/sample-rate limits, pre-skip/gain/granule semantics, malformed/truncated limits and terminal errors. Decoder instances are stateful leases and are never shared concurrently without an explicit synchronization contract.

### M2: asynchronous bounded import pipeline

Asset import reads under byte/time quotas, parses metadata, decodes or validates in worker operations, and emits progress/cancel/terminal receipts. No decode, resample, waveform analysis, compression or artifact write runs on the editor/frame thread. Derived artifacts contain versioned metadata and bounded chunk/index tables; cache keys include source hash, importer/codec version and settings.

### M3: streaming playback artifact

Separate source import from runtime playback. Large audio remains chunked; playback consumes compressed chunks incrementally into a bounded PCM ring with seek/loop/pre-roll state. Decode scheduling uses audio deadlines and underrun/backpressure telemetry rather than full-file PCM materialization.

### M4: lifecycle and product integration

One provider generation owns native library/module lifetime. Unload waits for decoder leases and import operations or returns Busy; reload creates a new generation without invalidating active streams. Editor import, packaged client playback and dynamic plugin paths consume the same validated artifact/codec contract.

## 5. Quantified acceptance

1. Corpus: valid/corrupt/truncated `.opus`, mono/stereo/multichannel, supported sample rates, silence/impulse/music/voice, `1 s/60 s/1 h`, small/large metadata and seek/loop boundaries. Compare decoded duration/channel/rate/pre-skip/gain and deterministic artifact hashes.
2. Import: `1/4/16` concurrent jobs and `64 KiB/1 MiB/16 MiB` chunks. Record wall/CPU p50/p95/p99, compressed and PCM throughput, allocations/bytes, peak RSS, queue wait, cancellation latency, artifact bytes and cache hit/miss. Main/editor thread decode time must be zero.
3. Playback: `1/16/64` streams, seek/loop and I/O jitter. Record decode deadline misses, ring fill, underruns, worker CPU, latency, RSS and energy; retained PCM is bounded by configured rings, not source duration.
4. Fault/lifecycle: missing library/symbol/version, denied capability, malformed packet, allocation limit, cancel, plugin unload/reload and process shutdown. Every accepted operation/stream reaches one terminal state and no callback enters unloaded code.
5. WPR/ETW profiles worker CPU, main-thread work, locks/waits, file I/O, RSS and energy on a launchable current-source editor/client. RenderDoc is not applicable to audio codec performance.
6. Unreal comparison uses matched source duration/channels/rate, build mode, hardware and sampling window. Compare architecture/work scale and report absolute values; do not claim parity from source inspection.

## 6. Current status

- Static review is complete for **4/4** Rust files.
- No production edit was made because a real codec/container/operation contract is required; the current implementation is a diagnostic placeholder.
- Cargo, native libopus, import/playback product, WPR/ETW, power, fault and soak validation remain pending. The plugin is not eligible for review-ledger acceptance, milestone commit or WeCom completion notification.
