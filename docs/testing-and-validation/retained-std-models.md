# Admitted retained allocation models

`zircon_runtime_interface/build.rs` pins the compiler commit, target triple and
SHA-256 of all three target `liballoc`, `libcore` and `libstd` rlibs. Unknown
identities, compiler wrappers and unstable layout flags remain unavailable.
`RETAINED_MODEL_ID_V1` reports the actual selected model. Admission still requires
the normal tree-backed serde_json map and ordinary number representation.

The original Rust 1.94.1 Windows MSVC distribution remains admitted unchanged.
The Linux editor additionally admits the rustup Rust 1.99.1
`x86_64-unknown-linux-gnu` distribution, commit
`58cf8f9c35b355199727627d07ae34a4e2846d67`. Exact library hashes are beside the
admission predicate rather than duplicated here.

The Linux audit checked the installed `rust-src` at that commit:

- `alloc/src/collections/btree/node.rs`: B = 6, eleven keys and values per leaf,
  an 8-byte parent pointer, two u16 fields and twelve internal edges. The existing
  conservative node bound covers field alignment and internal edges. A retained
  empty tree can keep one node, already covered by `len.max(1)`.
- `alloc/src/sync.rs`: the C-layout Arc header has two pointer-sized atomic counts,
  followed by an aligned payload and final alignment padding.
- `std/Cargo.toml` uses hashbrown 0.17.1. Its `raw.rs` capacity rounding and
  `TableLayout::calculate_layout_for` match the existing certificate: 16-byte SSE2
  groups, 7/8 load for large tables, element-size-dependent small capacities,
  aligned bucket data, one control byte per bucket and a trailing group.
- Vec, VecDeque, Unix OsString and PathBuf bill their actual capacities. These
  bounds cover requested allocation layouts, not allocator overhead or RSS.

`tests/retained_allocation_model.rs` tracks real requests to the linked global
allocator and checks bounds for tree splits/removals, aligned payloads, Arc,
capacity-bearing buffers and fresh hash tables across small/large capacities.
`tests/build_contract.rs` exercises the production admission predicate, including
altered/missing library identities, unknown commits/targets and layout refusal.
These regressions support the source audit; they do not admit arbitrary future
stdlib revisions. Audit and pin a new distribution before extending the list.
