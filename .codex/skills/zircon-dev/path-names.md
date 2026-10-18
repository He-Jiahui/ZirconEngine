# Engine path names

Use for first-party engine source, tests and resources. Keep the complete
repository-relative path, including separators and extension, at most **80
UTF-16 code units**. The complete checkout path is at most **128**; reserve at
least **48** for the checkout prefix and its separator.

Name files and folders by a recognizable responsibility. Avoid repeating the
parent domain, task IDs and optimization batch labels. Prefer complete words or
familiar abbreviations. When shortening layers, keep subsystem ownership and
test/resource distinctions clear; resolve collisions with a responsibility
word, never by overwriting a sibling.

Run the read-only guard from the actual checkout:

```powershell
python -B -m tools.audits.path_names --summary
```

The guard covers app, editor, runtime, interface, host, hub, derive and plugin
roots. Third-party vendor trees, ignored products/caches and noncanonical
temporary files are excluded. Expand that scope only when the user requests it.

For renames, record exact old/new paths and current hashes, then update Rust
mounts, include payloads, Cargo entries, resource IDs and active consumers.
Physical renames may use ordinary `#[path]` mounts to retain Rust API identities.
Preserve cfg, visibility, signed fixtures and unrelated working changes. Do not
keep duplicate sources or compatibility files. Recheck the path budget,
collisions, hashes and module/resource references after the move.
