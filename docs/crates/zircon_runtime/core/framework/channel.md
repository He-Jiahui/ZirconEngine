---
related_code:
  - zircon_runtime/src/core/framework/channel.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/runtime/events/subscribe.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/asset/facade/event.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_editor/src/ui/host/editor_asset_manager/api.rs
  - zircon_editor/src/ui/retained_host/app.rs
implementation_files:
  - zircon_runtime/src/core/framework/channel.rs
  - zircon_runtime/src/core/framework/mod.rs
plan_sources:
  - user: 2026-06-12 runtime architecture implementation from docs/plans/zircon_runtime/runtime
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/core/runtime/tests/channel.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Framework Channel Primitives

## Purpose

`zircon_runtime::core::framework::channel` owns neutral channel aliases and receive helpers shared by runtime contracts. The module was split out of the former core-root `channel_util.rs` and `types.rs` fragments so channel vocabulary lives with framework DTOs instead of the runtime service registry or the `core` root facade.

The public owner path is explicit: callers import `ChannelSender`, `ChannelReceiver`, `recv_latest`, and `wait_for` from `core::framework::channel`. The `core` root no longer re-exports these symbols, which keeps the root surface limited to curated engine runtime facades.

## Ownership Boundary

This module is contract-only. It may define crossbeam-backed aliases and small receive utilities, but it must not spawn threads, allocate runtime task pools, register services, or make lifecycle decisions.

Named thread creation belongs to `zircon_runtime::core::runtime::tasks`. Runtime service storage belongs to `zircon_runtime::core::runtime::ServiceObject`. Event fan-out still belongs to `EventBus`; it consumes `ChannelSender` from this module without making the channel module own event semantics.

## API

- `ChannelSender<T>` aliases `crossbeam_channel::Sender<T>`.
- `ChannelReceiver<T>` aliases `crossbeam_channel::Receiver<T>`.
- `recv_latest(...)` drains all currently available values and returns the last one.
- `wait_for(...)` wraps `recv_timeout(...)` and preserves `RecvTimeoutError`.

These helpers are intentionally small and deterministic. They exist so framework-facing contracts can name channel semantics without every consumer depending on the retired `core::types` or `core::channel_util` locations.

## Validation

`zircon_runtime/src/core/runtime/tests/channel.rs` covers `recv_latest(...)`. `zircon_runtime/src/tests/runtime_absorption/root_entries.rs` guards that `core/mod.rs` no longer declares `channel_util` or `types`, no longer re-exports channel helpers, and that this file exists as the framework owner.

The 2026-06-12 M2.1 migration evidence includes:

- `rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/root_entries.rs` passed with 4 tests.
- `cargo check -p zircon_runtime --lib --locked` passed with pre-existing warnings.
- downstream source scans found no remaining retired root channel imports under `zircon_app`, `zircon_editor`, `zircon_runtime_interface`, or `zircon_plugins`.
- `cargo check -p zircon_editor --lib --locked` passed with pre-existing warnings after editor imports moved to the framework channel path.
- `cargo test -p zircon_runtime --lib runtime_absorption --locked` is currently blocked during test build by an unrelated graphics test compile error: `partition_mesh_draws` is missing in `graphics/.../render.rs`.
