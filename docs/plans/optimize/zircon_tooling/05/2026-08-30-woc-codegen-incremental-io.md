# WOC codegen incremental I/O optimization

- Date: 2026-08-30 Asia/Shanghai
- Owner: Tooling05
- Plan item: `TOOL-CODEGEN-P1-024`
- Validation: managed coordinator batch pending

## Change

- Reference inventory validation now reads each catalog once and reuses the UTF-8 bytes for
  SHA-256 verification and JSON parsing.
- Projection publication reports `Written` or `Unchanged`; identical Rust/ZrVM outputs preserve
  their file modification time and do not invalidate the downstream incremental build chain.
- SHA-256 projection formatting writes into one 64-byte-capacity string instead of allocating one
  temporary string per digest byte.

## Performance evidence

The Windows release model used the six tracked `reference/current-head` catalogs and 13 samples:

- catalog input: `7,954,150 B`; admitted reads `15,908,300 B -> 7,954,150 B` (50% lower);
- catalog validation P95: `249,659,000 ns -> 183,222,300 ns` (26.61% lower);
- unchanged projection: current generated output is `11,555 B`, admitted writes
  `11,555 B -> 0 B` (100% lower), with mtime preserved;
- 11.28 MiB unchanged-output scale model P95: `1,575,574,600 ns -> 726,714,700 ns`
  (53.87% lower).

The timing numbers are conservative local model evidence, not a product-runtime claim. Final
acceptance requires the focused WOC contract-codegen test lane and the static performance contract
to pass in one coordinator-owned validation batch.
