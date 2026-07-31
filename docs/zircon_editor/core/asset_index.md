# Editor Asset Index Projection

## Owner

`zircon_editor::core::asset::EditorAssetIndex` is the editor-only projection over the runtime
`AssetRegistryIndex`. Runtime registry entries remain the sole owners of asset UUIDs, locators,
types, tags, dependency edges, and source digests. `.zmeta` v7 `AssetMetaDocument` snapshots remain
the sole owners of source modification time, artifact locators, and persisted import state.

## Boundary

The editor projection may retain shared runtime registry and metadata snapshots plus transient UI
state such as dirty and importing flags. It must not scan project files, parse sidecars during row
queries, or maintain a second UUID/path/dependency registry. Watch events mark paths dirty and are
resolved through the current or next runtime registry snapshot.

Each metadata document owns a reverse set of its projected UUIDs. Re-ingesting one document validates
the complete replacement before removing only that document's retired members, so incremental refresh
cost is proportional to the document rather than the full project index.

## Query Contract

Rows are deterministic and path-sorted because they are projected from
`AssetRegistryIndex::entries`. UUID, path, type, tags, dependency, and digest accessors delegate to
the borrowed runtime `AssetRegistryEntry`. Missing or watch-dirty metadata yields `Stale`; a `.zmeta`
projection without its asset artifact locator yields `Broken`; an active import yields `Importing`;
otherwise a persisted artifact yields `Ready`. `PreviewState` remains outside this M2.1 import-validity
projection so the M3 thumbnail/cache owner can converge it without changing browser import semantics.
