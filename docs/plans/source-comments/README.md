# Historical source-comment audits

Generated JSON/JSONL audits were removed from the tracked source set during M6
of the 2026-10-17 engineering review. Their exact bytes are preserved locally in
`.codex/outbox/claude-review-20261017/audit-archive`, with original paths, SHA-256
digests and lengths in `manifest.json`. Jenkins artifact-store import remains
pending; this relocation changes no historical receipt or acceptance result.

Fresh checkouts can recover the original objects from reviewed commit
`4f1ec91205d32c90a211b0ad060b6968eedd09a0` using `git cat-file -p
<commit>:<original-path>`. Route materialization to an approved Jenkins artifact
root. Do not restore generated audits into the source tree.

The repair script `tools/audits/repair_doc_paths.py` updates only document
front-matter path declarations. It rewrites unambiguous same-owner suffix and
file-to-module moves, and retires unresolved historical declarations. Historical
prose and test command descriptions remain unchanged; Git retains the original
metadata. The documentation gate rejects new invalid declarations.
