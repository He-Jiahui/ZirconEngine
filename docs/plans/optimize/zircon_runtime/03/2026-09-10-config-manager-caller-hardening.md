---
title: Runtime03 Config Manager Caller Hardening
category: zircon_runtime
report_id: Runtime03-config-manager-caller-hardening-2026-09-10
date: 2026-09-10
session_id: root-astra-optimize-20260909
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime03 Config Manager Caller Hardening

## Scope

This slice addresses the persistent-caller portion of Runtime03 P1-5. `ConfigManager::set_value`
is the only caller path in this slice that advances the persistence generation; the public
`CoreHandle::store_config*` helpers remain available for explicit bootstrap/session overlays and
test seeding. The schema, layer, migration, and session-only API redesign in P1-6 is outside this
record, so P1-5 is not claimed as globally closed.

## Implementation

- Built-in and external animation managers retain generation-aware `ConfigManager` handles,
  declare the Foundation module/manager dependency, load typed playback settings through the
  manager, and persist writes with `set_value` before updating the in-memory projection.
- The Physics runtime manager follows the same contract for persistent physics settings,
  refreshing its handle when a shared manager is attached to a new runtime generation.
- The Editor capability transaction resolves a typed ConfigManager access object, reads and writes
  the capability list through that service, and propagates persistence failures through the
  capability and rollback paths.
- Foundation dependency registration is explicit in the affected module descriptors. This keeps
  activation ordering and stale-handle behavior visible to the existing manager resolver instead
  of relying on an implicit global lookup.

Startup `zircon_app` configuration, the dynamic session render-profile override, foundation's
internal store implementation, and test fixtures still use raw storage intentionally. They are
follow-up work for the typed schema/session split and are not hidden in this completion record.

## Regression coverage

- `animation_playback_settings_use_config_manager_persistence` writes to an isolated config file,
  asserts the typed value and dirty-generation advance, flushes, and reloads it in a second
  runtime.
- Runtime and plugin module-resolution tests register Foundation before Animation/Physics.
- The runtime manager contract guard checks Foundation dependencies, generation-aware handles,
  `set_value`, and the absence of the animation/physics raw write bypass.
- The Editor host boundary guard checks typed capability configuration and rejects
  `store_config_value` in the production host service.
- The focused Frameworks01/05 plus UI access-boundary batch passed `31/31`; the bounded
  Runtime/Editor source-contract batch passed `70/70`. These are static contract checks only.

## Local evidence

- Scoped `rustfmt --edition 2021 --check` passed for all 14 touched Rust owners/tests.
- The merged Runtime/Editor performance-contract discovery was run as one batch and passed
  `1723/1723` (`Runtime 1143/1143`, `Editor 580/580`). This is static/source evidence, not a
  device timing result.
- Scoped `git diff --check` and source-level caller guards passed.
- Managed Windows Cargo/WGPU execution, release CPU/allocation measurements, and product p50/p95/
  p99 evidence remain pending. No performance qualification is inferred from the static batch.

## Remaining parent work

Runtime03 P1-5 still needs the typed `ConfigKey<T>`/transaction contract, malformed-value
diagnostics, a hard split between persistent and session-only writes, migration of the remaining
app/session callers, and shutdown durability evidence. Those items belong to the parent plan or a
follow-up slice.

## Source fingerprints

- `zircon_runtime/src/animation/manager/mod.rs`: `242FCD12E65B3F156E44919693E76A222F00B04941DA21C52A9BD09BB66BAD64`
- `zircon_runtime/src/animation/module.rs`: `F3E0C80BE90D58B4A7F9E21EB1A26319ADB866C4348C2395DFB8BB8D16460806`
- `zircon_runtime/src/foundation/tests.rs`: `73E8095171813FCA3099EAA6BBC8A6E00C6F678E4A6465A0D127DACB4B09F6B3`
- `zircon_runtime/src/tests/extensions/manager_handles.rs`: `CB3168156D384ED04E3DA09A415FA43BEC26A624C3A3C8586C9BC96CE696737A`
- `zircon_plugins/animation/runtime/src/manager.rs`: `C5735BE551395B91C73A714D4A7981C9BBD32430AC92DA8DB21165618A7CF763`
- `zircon_plugins/animation/runtime/src/module.rs`: `3FC514C1EA4399A42DA95BD44CA6DDB1B5798E4AE36BFD40C282DF29218E0AC1`
- `zircon_plugins/animation/runtime/src/tests.rs`: `EA6A662D69A2215D21F63693836CC727387FF1319CAB174330B01E694099EB48`
- `zircon_plugins/physics/runtime/src/manager.rs`: `38CC14BA761C511E3EB22C3B1698263BFB3DA890A0C4290D390FB8EDBAD1B4F7`
- `zircon_plugins/physics/runtime/src/manager/settings.rs`: `D70AFF1369C5974103E1ACC9D9B7BA35273461583F4C247BE83C9CC41FC9A474`
- `zircon_plugins/physics/runtime/src/module.rs`: `FB099A965D6679E9731CC8DBA9E4BCECB184198E91516F6D69BC36B891781B0C`
- `zircon_plugins/physics/runtime/src/tests.rs`: `94B96CE86A5698756C5D7772A8D2F9F2BE0CF491ABDA7AFAEFF75F7B96B19F25`
- `zircon_editor/src/ui/host/runtime_services.rs`: `9222246F28B61288243EA5669878CD7E03EB7E28936963B56F6B878DB1ECE735`
- `zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/capabilities.rs`: `10AF17611150AAC20E01E0F62F31533D971B47A4CBC9A1A612C37DE074E0FFFD`
- `zircon_editor/src/tests/ui/boundary/host_cutover.rs`: `57D7F120F5D4A5480A2A058B33AC41FF18A9FB530D18BEAAA3450447669F9C11`
