---
scope:
  - zircon_runtime/src/asset/project/paths.rs
  - zircon_runtime/src/asset/project/package_asset_registry.rs
  - zircon_runtime/src/asset/project/manager/source_uri_for_path.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs
reference:
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/PathTree.h
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/PathTree.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistryState.cpp
status: research_complete_implementation_not_approved
date: 2026-08-14
---

# Project Virtual URI Resolution Algorithm Audit

## Decision

`res://` is the only project virtual path required by the MVP. It is a persisted,
platform-independent identity formed from the path relative to a manifest-declared asset root.
The physical project root, cache root, release/staging root, and Windows path representation are
resolver inputs, not additional persisted URI prefixes. No `windows://`, drive, cache, or release
namespace is proposed.

Windows-specific behavior belongs to the resolver boundary only: resolve an external project or
asset-root input once, retain its physical operation path, and derive a diagnostic display path
without changing the logical URI. `ProjectPaths` and `PackageAssetRegistry` already provide that
boundary: the former resolves aliases and rejects drive-relative/root-relative input; the latter
publishes canonical project roots after containment validation. `AssetUri` must continue to retain
only `res://relative/path` or its existing package identity.

## Current Control Flow

1. `ProjectManager::open` resolves a project root and `PackageAssetRegistry::register_project_roots`
   resolves each manifest `RelPath` root once, checks containment, rejects aliases that escape the
   project, and stores the physical roots.
2. `collect_files` walks each registered root and rejects symlink/reparse entries before a source
   is admitted to the scanner.
3. Full scans call `source_uri_for_asset_root_path(root, None, path)`. For a project source this
   calls the public `project_uri_for_source_path(path)` boundary, which resolves the source again
   and resolves every configured project root again before selecting a unique containing root.
4. The public boundary is correct for arbitrary editor and external paths: it must discover aliases,
   reject sources outside roots, and report overlapping roots as ambiguous. The full scanner is
   different: it already owns a canonical root and has produced each path below that root after
   rejecting reparse entries.

The shared public resolver should remain authoritative for untrusted or arbitrary paths. The scan
loop must not silently weaken its invariants; it should use a separate, owner-private fast path
only when its provenance is already proven by the scanner.

## Cost Model

Let `N` be admitted source files, `R` be project asset roots, and `C(path)` one filesystem
canonicalization. Current full project scanning performs the collection traversal and, in addition,
approximately `N * (1 + R)` path canonicalizations for project source-to-URI conversion. This is
not a Rust allocation micro-cost: on Windows it can become repeated filesystem/reparse queries.

The F1 template normally has one root and only a few sources, so this is not a claim that it is the
current MVP wall-time bottleneck. It becomes structurally material for large asset roots because
the source collector already establishes `(registered_root, descendant_path)` provenance.

The proposed internal fast path has `O(N * L)` lexical component work, where `L` is the relative
path length, and no added root canonicalization during the scan. Registration remains `O(R * C)`;
external reverse lookup remains `O(R * C)`. No cache is allowed to mask a changed root or permit a
link/reparse escape.

## Reference Comparison

Unreal's AssetRegistry keeps logical package-path ownership inside the registry. `FPathTree`
inserts a path once, records parent/child relationships, and performs path existence/subpath
queries from maps. `FAssetRegistryState` also builds `CachedAssetsByPath` while loading its asset
state. The relevant lesson is ownership, not copying Unreal's entire hierarchy: validated logical
paths are indexed by the registry/scan owner rather than re-derived from an external filesystem
boundary for every registry entry.

For Zircon MVP, `PackageAssetRegistry` is already the correct owner of the validated physical
roots and the scanner already owns traversal provenance. A full Unreal-style general path tree is
premature. The minimal equivalent is a private helper accepting the registered canonical root and
the collector-produced descendant, converting the checked relative components directly to the
same canonical `res://` URI.

## Required Invariants For Any Implementation

- Persisted `AssetUri` remains relative and portable. No absolute path, Windows prefix, cache
  directory, or process working directory may enter the URI.
- Project root and manifest roots are resolved once at registration and remain containment-checked.
- The fast path is callable only from a collection path that rejects symlinks/reparse points and
  passes the exact registered root used for traversal.
- The public `project_uri_for_source_path` keeps its physical canonicalization and ambiguity
  behavior for arbitrary editor, watcher, and external inputs.
- Multiple manifest roots with the same logical relative source remain rejected by the existing
  duplicate-URI check; the fast path may not choose an implicit winner.
- Package URI handling remains separate and unchanged.

## Candidate Implementation Shape

Do not add a prefix registry or platform branch. Keep the existing `PackageAssetRegistry` root
storage. Add an owner-private conversion used only by full scan and compound-member collection:

1. Verify `path.strip_prefix(registered_root)` succeeds.
2. Convert its checked components to forward-slash logical text.
3. Construct the existing `AssetUri::parse("res://...")` identity.

The public `project_uri_for_source_path` continues to resolve physical identity and can delegate to
the same final relative-component-to-URI formatter after it has resolved a root and path. This
avoids duplicated URI formatting while preserving the security boundary.

## Measurement Plan Before Approval

Run only after the current UI12 validation window is released. Use a Windows coordinator copy and
write all artifacts below `D:\ZirconBuilds\mvp-path-resolution-<run-id>`; never use `C:`.

| Scenario | Data | Measure | Required comparison |
|---|---:|---|---|
| Template baseline | canonical F1 template | scan/import elapsed, CPU, disk I/O | current vs candidate |
| Medium source root | 1,000 flat/nested sources | elapsed; resolve count; allocs | current vs candidate |
| Large source root | 100,000 accepted sources | elapsed; CPU; I/O; peak set | current vs candidate |
| Multiple roots | 2 and 8 declared roots | elapsed and URI/duplicate behavior | current vs candidate |
| Alias/security | junction/SUBST outer root; inner reparse source | typed errors and URI identity | no regression |

The medium resolve count is `ProjectPaths::resolve_existing`; allocations are
measured separately. The large case records CPU samples, disk I/O, and peak
working set separately from elapsed time.

## Runtime Capture Output Boundary

`ZIRCON_RUNTIME_CAPTURE_FRAME_PNG` is a runtime output setting, not persisted
project data and not a second virtual-path prefix. Its physical destination is
represented as `ResolvedProjectPath`; the project identity remains the sole
portable `res://` URI.

`runtime_frame_capture_path_from_value` resolves an unrooted output below the
already-open physical project root with `ProjectPaths::resolve_path_from`. An
absolute output is resolved with `ProjectPaths::resolve_path`; with no open
project, an unrooted output is likewise resolved at the ordinary resolver
boundary. `resolve_path_from` preserves the selected physical base and rejects
both rooted and Windows drive-relative inputs. `resolve_path` resolves existing
junction/SUBST/symlink identity and retains only an uncreated tail for a new
target.

Consequently, cache, stage, release, and capture destinations stay physical
resolver inputs. Validation policy may require an approved `D:`, `E:`, or `F:`
output root, but that policy must not become a persisted URI scheme or a second
prefix parser.

## Reference Alignment

Unreal keeps project, saved, and user directories in its Core `FPaths` utility,
alongside normalization and relative-to-full conversion. This supports one
central physical-path boundary rather than allowing callers to retain arbitrary
working-directory-relative strings.

Bevy separates `AssetPath` identity from its `AssetSource` reader/writer. Its
named-source grammar is appropriate for an engine that explicitly supports
multiple asset source identities. Zircon deliberately does not adopt that
grammar here: the approved persistent identity is only `res://`, while the
project root, cache root, staging root, release root, and capture destination
are resolver inputs. That preserves portability without creating a prefix
parser for every physical storage location.

The existing `ProjectPaths` and `ResolvedProjectPath` boundary is therefore the
correct landing zone. No new crate, facade, persisted URI form, or compatibility
alias is needed; the remaining work is owner-scoped fixture-root enforcement and
the later Windows measurements already listed above.

Use Windows Performance Recorder/Analyzer or an equivalent approved CPU and file-I/O profiler for
the 1,000 and 100,000 source cases. Record wall time separately, and do not infer power from elapsed
time. Power comparison requires a stable hardware/power-plan run with an actual energy counter; no
cross-engine efficiency claim is valid without equivalent workloads and hardware.

Approval requires identical registry logical identities and typed failures for every correctness
case, plus profiler evidence that source-to-URI canonicalization is a material scan cost. If it is
not material, retain the simpler current implementation and record the measured result rather than
shipping an unproven optimization.

## Test Artifact Root Compliance

Current `source_uri_for_path.rs` test support derives its alias fixture parent from
`std::env::temp_dir()`. `frame_capture.rs` derives its capture fixture parent
from the current test binary directory. Either can resolve under `C:` on
Windows, which is not an approved Zircon artifact root. This is a test-fixture
output concern, not a reason to change the `res://` resolver or add another URI
scheme.

Before the focused Rust alias test is run again, its fixture-root helper must require an explicitly
approved `D:`, `E:`, or `F:` test-artifact root, resolve that root through
`ProjectPaths`, and create only a uniquely named child below it. The cleanup,
alias/reparse, and capture atomic-write assertions remain the same. A missing or
disallowed root must fail before any fixture directory is created; falling back
to the process temp directory or test-binary directory is not permitted.
