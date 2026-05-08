# Remote BRP World API Reflection Substrate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the missing Remote / BRP foundation by landing a safe runtime world reflection substrate first, then adding the protocol shell and local in-process roundtrip without starting network transport work.

**Architecture:** The first owner is `zircon_runtime::scene`: it must expose remote-readable schema and value access for entities, components, resources, and queries without depending on editor-only state. The protocol contract then lives under `zircon_runtime::core::framework::remote` as DTOs, method descriptors, permission metadata, and local dispatch; network transports remain out of scope until the world API and protocol semantics are testable.

**Tech Stack:** Rust 2021, `serde`, `serde_json`, existing `zircon_runtime` scene ECS/dynamic component APIs, existing diagnostics/plugin metadata, Cargo workspace validation through Zircon milestone testing stages.

---

## Current Baseline

- The workspace root is the fixed `zircon_app`, `zircon_runtime`, `zircon_editor`, `zircon_runtime_interface` shape in `Cargo.toml`.
- `.codex/plans/runtime-picking-gizmos-camera-remote-bevy-completion-plan.md` already assigns Remote Protocol to M7 and names `zircon_runtime::core::framework::remote` as the protocol owner.
- `zircon_editor/src/core/editor_operation.rs` has a `callable_from_remote` flag, but that is editor operation control, not Bevy Remote Protocol parity.
- `zircon_runtime/src/scene/ecs/component_registry.rs` and `resource_registry.rs` already list runtime component/resource descriptors, but they currently expose only Rust type names and dynamic component identifiers, not remote schema, permissions, or JSON value access.
- `zircon_runtime/src/scene/world/dynamic_components.rs` already stores dynamic component payloads as `serde_json::Value` and validates plugin component descriptors. This is the safest first bridge for remote component payloads.
- `zircon_runtime/src/core/diagnostics/*` and `zircon_runtime/src/plugin/*` already expose diagnostics/plugin status data that can later be wired as remote read-only APIs.
- Bevy reference evidence: `dev/bevy/crates/bevy_remote/src/lib.rs` uses JSON-RPC 2.0 DTOs, `RemoteMethods`, `BrpRequest`, `BrpResponse`, `BrpError`, instant and watching handlers; `dev/bevy/crates/bevy_remote/src/builtin_methods.rs` defines `world.*`, `resource.*`, `registry.schema`, `schedule.*`, and `rpc.discover`; `dev/bevy/crates/bevy_remote/src/schemas/mod.rs` and `dev/bevy/crates/bevy_world_serialization/src/dynamic_world_builder.rs` show that world remote access depends on reflected component/resource type metadata.

## Out Of Scope

- TCP, HTTP, WebSocket, stdio, SSE, and editor-host route transports.
- Full Bevy `Reflect` equivalent for every Rust component type.
- Editor operation invocation over BRP.
- Long-lived watch streaming beyond versioned change tokens and local pollable subscriptions.
- ABI export through `zircon_runtime_interface`; that belongs after local protocol semantics stabilize.

## File Structure

- Create `zircon_runtime/src/scene/remote/mod.rs`: navigational module for world remote API.
- Create `zircon_runtime/src/scene/remote/schema.rs`: remote schema DTOs for component/resource/entity fields and permission flags.
- Create `zircon_runtime/src/scene/remote/value.rs`: JSON value DTO helpers and typed error conversion for component/resource value access.
- Create `zircon_runtime/src/scene/remote/access.rs`: `WorldRemoteAccess` facade methods for list/get/query/spawn/despawn wrappers.
- Create `zircon_runtime/src/scene/remote/query.rs`: remote query request/response DTOs and query matching against available component descriptors.
- Modify `zircon_runtime/src/scene/mod.rs`: export the new remote API module without making root files behavioral.
- Modify `zircon_runtime/src/scene/world/mod.rs`: include the remote access module only as structure.
- Modify `zircon_runtime/src/scene/ecs/component_registry.rs`: add remote descriptor projection helpers without replacing ECS storage behavior.
- Modify `zircon_runtime/src/scene/ecs/resource_registry.rs`: add remote descriptor projection helpers without creating direct mutable resource serialization.
- Create `zircon_runtime/src/core/framework/remote/mod.rs`: navigational module for BRP DTOs and registry.
- Create `zircon_runtime/src/core/framework/remote/protocol.rs`: `RemoteRequest`, `RemoteResponse`, `RemotePayload`, `RemoteError`, protocol version, JSON-RPC compatible serialization.
- Create `zircon_runtime/src/core/framework/remote/capability.rs`: method capability flags and access table types.
- Create `zircon_runtime/src/core/framework/remote/method.rs`: `RemoteMethodDescriptor`, `RemoteMethodRegistry`, method ids, params/result schema ids.
- Create `zircon_runtime/src/core/framework/remote/builtin.rs`: built-in method descriptors and dispatch adapters into `WorldRemoteAccess`.
- Create `zircon_runtime/src/core/framework/remote/local.rs`: in-process local channel/session roundtrip with no network framing.
- Modify `zircon_runtime/src/core/framework/mod.rs`: export `remote` structurally.
- Add tests under `zircon_runtime/src/tests/remote/` with `mod.rs`, `world_schema.rs`, `world_query.rs`, `protocol.rs`, `permissions.rs`, and `local_roundtrip.rs`.
- Add docs `docs/zircon_runtime/scene/remote.md` and `docs/zircon_runtime/core/framework/remote.md` with required machine-readable headers.
- Update `.codex/plans/runtime-picking-gizmos-camera-remote-bevy-completion-plan.md` only after implementation if milestone status needs a concrete acceptance note; do not rewrite the roadmap during initial implementation.

## Milestone 1: World Remote Schema Substrate

- Goal: expose remote-stable world schema metadata for entity, component, and resource inspection without adding protocol dispatch.
- In-scope behaviors: list known component schemas, list known resource schemas, identify dynamic plugin components, classify read/write capabilities, expose entity IDs and per-entity component ids, return structured missing-type errors.
- Dependencies: existing `World`, `ComponentRegistry`, `ResourceRegistry`, `ComponentTypeRegistry`, and dynamic component descriptor behavior.

### Implementation Slices

- [ ] Create `zircon_runtime/src/scene/remote/schema.rs` with these public DTOs: `RemoteWorldSchema`, `RemoteComponentSchema`, `RemoteResourceSchema`, `RemoteEntitySchema`, `RemoteFieldSchema`, `RemoteSchemaSource`, `RemoteSchemaAccess`, and `RemoteSchemaError`. All DTOs that cross protocol boundaries derive `Clone`, `Debug`, `PartialEq`, `Serialize`, and `Deserialize` where stable.
- [ ] Implement `RemoteSchemaAccess` as explicit booleans: `read`, `write`, `query`, `spawn`, `despawn`, `debug`. Do not infer permissions from method names.
- [ ] Add component projection helpers in `component_registry.rs` that produce schema source, type id, type name, and storage type for registered Rust and dynamic components. Keep the ECS registry storage private.
- [ ] Add resource projection helpers in `resource_registry.rs` that produce type id/type name and read-only default access. Do not add resource mutation yet.
- [ ] Create `zircon_runtime/src/scene/remote/access.rs` with `WorldRemoteAccess<'w>` wrapping `&'w World` and methods `schema()`, `list_entities()`, `entity_schema(entity)`, `list_components()`, and `list_resources()`.
- [ ] Wire `zircon_runtime/src/scene/remote/mod.rs`, `zircon_runtime/src/scene/mod.rs`, and `zircon_runtime/src/scene/world/mod.rs` as navigational exports only.
- [ ] Add tests in `zircon_runtime/src/tests/remote/world_schema.rs` proving dynamic component descriptors appear with JSON schema-like fields, unknown entities return structured errors, and resource/component lists are sorted deterministically.
- [ ] Add docs `docs/zircon_runtime/scene/remote.md` with required frontmatter and a section explaining that this milestone intentionally exposes schema and read-only enumeration before BRP method dispatch.

### Testing Stage

- Run: `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib remote::world_schema --locked --target-dir target\codex-shared-a -- --nocapture`
- Run: `$env:CARGO_INCREMENTAL='0'; cargo check -p zircon_runtime --lib --locked --target-dir target\codex-shared-a`
- Debug loop: if tests fail in upper remote tests, inspect `ComponentRegistry`, `ResourceRegistry`, and `WorldRemoteAccess` before changing protocol-facing DTOs.
- Acceptance evidence: schema tests pass, `cargo check` passes, docs list touched code and tests.

## Milestone 2: World Remote Values And Query API

- Goal: provide read-first remote world value access and query semantics over the schema substrate.
- In-scope behaviors: get dynamic component payloads, list component ids on entity, lenient vs strict get semantics, query entities by required/optional/has/without component ids, basic spawn/despawn wrappers, mutation API descriptors that remain permission-gated until schema validation is sufficient.
- Dependencies: Milestone 1 schema substrate.

### Implementation Slices

- [ ] Create `zircon_runtime/src/scene/remote/value.rs` with `RemoteEntityId`, `RemoteComponentValue`, `RemoteResourceValue`, `RemoteValueError`, and conversion from `serde_json::Value` for dynamic components.
- [ ] Create `zircon_runtime/src/scene/remote/query.rs` with `RemoteQueryRequest`, `RemoteQueryData`, `RemoteQueryFilter`, `RemoteQueryRow`, `RemoteGetComponentsRequest`, and `RemoteGetComponentsResponse`.
- [ ] Implement `WorldRemoteAccess::get_components(request)` supporting dynamic component JSON payloads and fixed component presence reporting. Fixed Rust components without remote serialization return a per-component `RemoteValueError::NotSerializable` in lenient mode and fail the whole request in strict mode.
- [ ] Implement `WorldRemoteAccess::query(request)` using entity iteration plus `contains_component_id` / `registered_dynamic_component_id`. Query result rows include `entity`, `components`, `missing`, and `has` maps.
- [ ] Implement `WorldRemoteAccess::spawn_empty()` and `WorldRemoteAccess::despawn_entity(entity)` as foundation for later BRP write methods. Do not allow arbitrary component insertion yet except dynamic component insertion through existing descriptor validation.
- [ ] Add `WorldRemoteAccess::set_dynamic_component(entity, component_id, value)` with explicit write access validation and existing `World::set_dynamic_component` reuse.
- [ ] Add tests in `zircon_runtime/src/tests/remote/world_query.rs` covering lenient get, strict get, query with/without, optional component results, missing component error data, dynamic component set/get, and despawned entity rejection.
- [ ] Update `docs/zircon_runtime/scene/remote.md` with the value/query semantics and explain the intentional divergence from Bevy: Rust fixed components are presence-queryable until Zircon has a full reflect serializer.

### Testing Stage

- Run: `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib remote::world_query --locked --target-dir target\codex-shared-a -- --nocapture`
- Run: `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib remote::world_schema --locked --target-dir target\codex-shared-a -- --nocapture`
- Run: `$env:CARGO_INCREMENTAL='0'; cargo check -p zircon_runtime --lib --locked --target-dir target\codex-shared-a`
- Debug loop: if query semantics disagree with schema output, fix the schema/query substrate before adding protocol adapters.
- Acceptance evidence: strict/lenient behavior and dynamic component payloads are covered by targeted tests.

## Milestone 3: BRP Protocol DTOs, Method Registry, And Permissions

- Goal: define the BRP-compatible protocol shell and capability table without network transport.
- In-scope behaviors: JSON-RPC 2.0 compatible request/response/error, method id validation, params/result schema ids, protocol version, method discovery, capability classification, permission denial errors.
- Dependencies: Milestone 1 and 2 world remote substrate.

### Implementation Slices

- [ ] Create `zircon_runtime/src/core/framework/remote/protocol.rs` with `RemoteProtocolVersion`, `RemoteRequest`, `RemoteResponse`, `RemotePayload`, `RemoteError`, `RemoteErrorCode`, and JSON serialization matching JSON-RPC field names: `jsonrpc`, `id`, `method`, `params`, `result`, `error`.
- [ ] Use stable error codes: parse error `-32700`, invalid request `-32600`, method not found `-32601`, invalid params `-32602`, internal error `-32603`, plus Zircon domain codes `-32010` permission denied, `-32020` world access error, `-32030` schema unavailable, `-32040` subscription unavailable.
- [ ] Create `capability.rs` with `RemoteCapability::{ReadWorld, WriteWorld, ReadResources, WriteResources, Diagnostics, EditorAuthoring, PluginAccess, Debug}` and `RemoteCapabilitySet`.
- [ ] Create `method.rs` with `RemoteMethodId`, `RemoteMethodDescriptor`, `RemoteMethodKind::{Instant, Subscription}`, `RemoteMethodRegistry`, duplicate id validation, and permission lookup.
- [ ] Create `builtin.rs` with built-in method descriptors for `rpc.discover`, `registry.schema`, `world.list_entities`, `world.list_components`, `world.get_components`, `world.query`, `world.spawn_entity`, `world.despawn_entity`, `world.set_dynamic_component`, `resource.list`, `diagnostics.snapshot`, and `plugin.status`. Methods not implemented in dispatch still appear as descriptors only if they return `RemoteErrorCode::MethodNotFound` is not possible; otherwise keep them out of the initial registry.
- [ ] Implement registry discovery result with protocol version, method id, kind, params schema id, result schema id, required capabilities, and stability string.
- [ ] Add tests in `zircon_runtime/src/tests/remote/protocol.rs` for request/response/error JSON roundtrip, invalid method ids, duplicate registry entries, discovery output, and error code serialization.
- [ ] Add tests in `zircon_runtime/src/tests/remote/permissions.rs` for capability denial on write methods and read-only access success on read methods.
- [ ] Create `docs/zircon_runtime/core/framework/remote.md` with required frontmatter, Bevy reference files, protocol field mapping, error codes, method capability table, and transport out-of-scope note.

### Testing Stage

- Run: `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib remote::protocol --locked --target-dir target\codex-shared-a -- --nocapture`
- Run: `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib remote::permissions --locked --target-dir target\codex-shared-a -- --nocapture`
- Run: `$env:CARGO_INCREMENTAL='0'; cargo check -p zircon_runtime --lib --locked --target-dir target\codex-shared-a`
- Debug loop: if protocol tests fail because world data is missing, do not special-case protocol; return to Milestone 1 or 2 substrate.
- Acceptance evidence: protocol DTOs roundtrip and method permission gates reject unauthorized methods before handler execution.

## Milestone 4: Local Roundtrip Dispatch

- Goal: connect protocol DTOs to world remote methods through an in-process local session.
- In-scope behaviors: dispatch `rpc.discover`, `registry.schema`, `world.list_entities`, `world.list_components`, `world.get_components`, `world.query`, `world.spawn_entity`, `world.despawn_entity`, `world.set_dynamic_component`, and `resource.list` over a local channel/session API.
- Dependencies: Milestone 2 world value/query behavior and Milestone 3 protocol registry.

### Implementation Slices

- [ ] Create `zircon_runtime/src/core/framework/remote/local.rs` with `LocalRemoteSession`, `LocalRemoteRequestContext`, and `LocalRemoteDispatchResult`.
- [ ] Implement local dispatch as direct function calls over `&mut World` plus `RemoteCapabilitySet`; do not create threads, sockets, async runtimes, or network framing.
- [ ] Add builtin handler adapters in `builtin.rs` that parse `serde_json::Value` params into the world remote request DTOs and convert results into `RemoteResponse`.
- [ ] Ensure every handler returns structured `RemoteError` on invalid params, missing entity, missing component, not serializable component, permission denied, and unimplemented subscription.
- [ ] Add minimal subscription token DTOs but make subscription methods return `RemoteErrorCode::SubscriptionUnavailable` until a later event-stream milestone defines watch semantics.
- [ ] Add tests in `zircon_runtime/src/tests/remote/local_roundtrip.rs` that serialize request JSON, dispatch through `LocalRemoteSession`, deserialize response JSON, and assert results for discovery, schema, list entities, get dynamic component, query, denied write, allowed dynamic write, and post-write readback.
- [ ] Update both remote docs with local roundtrip examples and the exact list of methods implemented vs reserved.

### Testing Stage

- Run: `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib remote::local_roundtrip --locked --target-dir target\codex-shared-a -- --nocapture`
- Run: `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib remote --locked --target-dir target\codex-shared-a -- --nocapture`
- Run: `$env:CARGO_INCREMENTAL='0'; cargo check -p zircon_runtime --lib --locked --target-dir target\codex-shared-a`
- Debug loop: when a local roundtrip fails, first isolate whether JSON DTO, registry permission, or world substrate failed; fix the lowest layer.
- Acceptance evidence: local roundtrip proves protocol shell can operate runtime world data without editor-only APIs or network transport.

## Milestone 5: Diagnostics, Plugin Status, And Acceptance Sweep

- Goal: add basic read-only engine status remote methods and run the broader validation stage before claiming BRP foundation completion.
- In-scope behaviors: diagnostics snapshot method, plugin status method, docs finalization, acceptance matrix, workspace-level validation.
- Dependencies: local protocol dispatch from Milestone 4.

### Implementation Slices

- [ ] Add `diagnostics.snapshot` handler using existing `collect_runtime_devtools_snapshot` or `collect_runtime_diagnostics` data where a `World`-local context can supply required inputs; if the current API requires unavailable runtime state, register the method as descriptor-only with `RemoteErrorCode::SchemaUnavailable` and document the missing runtime handle requirement.
- [ ] Add `plugin.status` handler using existing runtime plugin descriptor/catalog data if reachable without constructing editor state; otherwise expose the built-in catalog summary only and document the limitation.
- [ ] Add tests that these methods either return structured snapshots or structured unavailable errors, never panic and never return untyped strings.
- [ ] Update docs with an acceptance table covering request/response DTOs, method registry, permissions, world access, query, component/resource schema, diagnostics/plugin status, and subscription gap.
- [ ] Update `.codex/plans/runtime-picking-gizmos-camera-remote-bevy-completion-plan.md` with a short M7 progress note only after tests pass.

### Testing Stage

- Run: `$env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib remote --locked --target-dir target\codex-shared-a -- --nocapture`
- Run: `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -TargetDir target\codex-shared-a`
- If shared framework exports or workspace manifests changed, run: `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir target\codex-shared-a`
- Debug loop: if workspace validation fails outside remote, classify whether failure is from this branch of work or pre-existing dirty worktree state before changing unrelated files.
- Acceptance evidence: command output, docs, and tests together show BRP foundation is partial only for network/watch/ABI, not for world schema/local roundtrip.

## Required Boundary Coverage

- Empty world returns empty entity/query results and non-empty schema only for registered component/resource types.
- Missing entity returns structured `WorldAccess` error.
- Missing component in lenient mode appears in per-component errors; in strict mode fails the request.
- Non-serializable fixed Rust components are presence-queryable but not value-readable until reflect serialization exists.
- Dynamic component JSON values roundtrip exactly.
- Duplicate method registration fails deterministically.
- Permission denied happens before handler mutation.
- Subscription/watch methods return a typed unavailable error until the event stream milestone.
- Protocol rejects malformed JSON-RPC version and missing method.

## Self-Review

- Spec coverage: The plan covers the user-confirmed order: World API / Reflection substrate, then BRP protocol shell, then local roundtrip. Network transports and watch streams are explicitly deferred but represented by typed errors and descriptors.
- Placeholder scan: No `TBD`, `TODO`, or unspecified edge handling remains. Every milestone names files, behavior, tests, commands, and docs.
- Type consistency: `WorldRemoteAccess`, `RemoteRequest`, `RemoteResponse`, `RemoteError`, `RemoteMethodRegistry`, `RemoteCapabilitySet`, and `LocalRemoteSession` are introduced before later milestones use them.
