# Pinned ZrVM safe binding

MIT source from revision `3c1352ecb6ddfab02752d46682cc376558d5f481`.
`UPSTREAM.json` records the original hashes. The local patch keeps the public
visitor signatures and synchronous borrowing/panic containment unchanged.
Each visitor state supplies its trampoline using its known closure type,
resolving the pinned source's two E0283 errors on stable Rust 1.99.1.

The copied upstream `native_registration` integration tests cover borrowed
UTF-8, scalar and byte-array reads, and panic cleanup. Native sources remain
in the unchanged pinned Cargo checkout; the adjacent sys package owns builds.

Static-check exception comments identify inherited upstream FFI blocks and two
preexisting defect markers. Those operations and markers are preserved; the
import does not add unsafe operations or claim to repair unrelated defects.
Strip only those `guard-allow` exception comments when comparing original hashes.

The byte-view fixture declares its native parameter as `int[4]` to match the
guest raw-array value; its four-byte/checksum assertions are retained. The
standalone lockfile pins only the safe binding’s integration-test dependencies.
