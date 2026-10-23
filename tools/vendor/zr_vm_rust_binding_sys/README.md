# Pinned ZrVM sys binding

This is the MIT-licensed sys crate from ZrVM revision
`3c1352ecb6ddfab02752d46682cc376558d5f481`. `UPSTREAM.json` records original
file hashes. The four `src/` ABI files are unchanged. The adjacent safe Rust
binding is pinned to the same revision with a narrow visitor-type repair.
All native sources are read from the unchanged pinned Git checkout.

The upstream build script admits Windows drive roots only. This build adds an
explicit Linux exception via `ZIRCON_LINUX_CARGO_TARGET_ROOT`, which must be a
physical direct child of `/workspace/cargo-targets`. Every compiler/cache/temp
output must remain inside that exact root. Linux `CARGO_HOME` is Cargo's
registry/Git source store, separate from compiler products in `OUT_DIR` and
`CARGO_TARGET_DIR`; the native build reads sources there without patching them.
Windows retains its original `CARGO_HOME` location check. Unset the exception
to retain the Windows policy. Symlinks and aliases remain rejected, including
case-only aliases on Linux.

Native compilation reads the already-fetched pinned Cargo checkout and writes
only under `OUT_DIR`. `ZR_VM_NATIVE_SOURCE_ROOT` can select a native source tree
explicitly; `ZR_VM_RUST_BINDING_LIB_DIR` retains the external SDK route.
`ZR_VM_RUST_BINDING_LINK_KIND=static` avoids adding native shared libraries to an
editor stage. Native builds remain serial. Do not edit Cargo's Git cache.

The Linux-only `cmake/` recipe reuses the pinned native module graph with its
Windows-only optional binding entry disabled, then adds the same binding
sources and definitions. It checks the physical output namespace again and
exports CMake's actual static/system dependency closure for Cargo, including
system library search directories. It does not create upstream's recursive
Cargo validation targets. Windows continues to use upstream's CMake entry.
