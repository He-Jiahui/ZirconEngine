---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: asset-artifact-chunked-generation-store
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/artifact
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs
  - zircon_runtime/src/core/framework/render
tests:
  - cargo test -p zircon_runtime --lib asset::tests::assets::artifact_store --locked --jobs 1 -- --nocapture --test-threads=1
  - artifact size, cold/warm, interrupted-write, corruption, IBL candidate and chunk residency matrices
---

# Runtime04：asset artifact分块generation store缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime asset artifact 16/16逐Rust文件性能审查，PERF-MVP-506
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：artifact schema、content identity、last-good publish与resident generation必须由Runtime04统一拥有，Render/Editor不得建立第二套cache truth。
- 生命周期键：`asset-artifact-chunked-generation-store`

## 失败现象与复现证据

普通cache写入同时持业务资产、深拷贝wire DTO、bincode bytes与zstd payload，读取整文件后整块解压/反序列化；UI document在bincode外又做TOML String往返。IBL runtime/asset-derived/staging store整blob读写，dispatch把完整blob clone进candidate Vec，environment再复制全部source texels。同步非原子文件I/O使caller stall、峰值RSS和损坏窗口随最大资产增长。PERF-MVP-505只删除了独立compressed Vec，未解决其余所有权和I/O边界。

## 最低共享层根因

当前artifact是单体路径文件而非versioned manifest引用的content-addressed chunks；metadata验证、payload residency、serde/compression和文件发布没有独立生命周期，也没有Runtime11有界I/O lane与共享borrowed candidate边界。

## 架构修复验收

- manifest独立记录schema、kind、content/revision hash、raw/compressed sizes和ordered chunk ids；payload按immutable content id共享，旧schema明确迁移或失效。
- serializer/zstd流式写临时文件并atomic replace manifest；中断、磁盘满和损坏保持last-good generation可读。
- reader先验证小header，只按请求chunk懒读/解码并以`Arc`共享；UI typed DTO不再TOML String中转。
- IBL derived/runtime/staging candidate借用/shared blob，source texels不复制；Render13只消费requested upload-ready mip/face chunks。
- Runtime11统一encode/decode/I/O的entry、bytes、age、RSS与shutdown预算；Editor15/export复用同一manifest/chunk inventory。
- 参考Bevy `AssetWriter`的异步writer/rename边界和Unreal DDC `FIoHash` + `FCompressedBuffer` shared value；不复制其与Zircon schema/generation不匹配的接口。
- 4KiB/256MiB/1GiB、1/1k assets、cold/warm/1% change及中断/损坏矩阵记录owner/clone/copy/read/write/decode bytes、caller blocked、queue/RSS：大payload owner=1，额外峰值按bounded chunk，warm payload I/O/decode=0，stable write=0。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止仅把整块Vec包进`Arc`而保留多次serialize/decode/copy。
- 禁止Editor、Render或IBL store维护独立content index/generation。

## 修复结果与回传

Open state: `部分前向修复，待受管验证`; no pass is claimed.

### 2026-07-31 Runtime04 source repair

- `artifact/store.rs` now publishes a schema-v3 `ZRARTM03` manifest only after
  streaming the compressed payload through immutable BLAKE3-addressed chunks.
  The manifest records kind, the owning `ResourceRecord` revision,
  raw/compressed sizes, ordered chunk ids, and the complete compressed-content
  hash. The prior `ZRARTM02` and older `ZRARTZ01` formats are invalidated, so
  the existing project recovery path reimports them rather than retaining a
  compatibility decoder.
- Reads validate the small manifest before streaming chunks through zstd. They
  reject non-BLAKE3 identifiers before forming chunk paths, then verify every
  chunk size/hash, the complete compressed hash, decoded raw byte count, and
  payload kind. Manifest input is capped at 4 MiB before bincode decode, and
  manifest serialization applies the same cap before atomic publication.
  Decode completion explicitly drains/verifies every declared chunk. Reimport
  rechecks an existing chunk's BLAKE3 digest and atomically repairs a corrupt
  entry before publishing a manifest. Manifest publication uses the shared
  atomic writer, so a failed replacement retains the prior manifest generation.
- Focused regression coverage now checks shared chunks across distinct
  manifests, multi-chunk streaming round trips, corrupted-chunk rejection and
  repair through reimport, oversized-manifest rejection, traversal-id rejection,
  final-chunk verification after the main payload is decoded, v2-format
  reimport, zero and over-budget raw-payload `Parse` rejection, and manifest
  persistence of a non-default resource revision. A metadata/payload-kind
  conflict is also rejected as `Parse` while the prior generation remains
  readable. The corrupt-chunk repair fixture now replaces the cache entry with a
  128 KiB file and locks the metadata-length short circuit, so reimport repair
  cannot regress to reading an arbitrarily large existing chunk. `rustfmt
  +1.94.1 --check` and scoped `git diff --check` pass; no Cargo command was run
  outside the coordinator.

Independent review round 1 reported three Important issues: unbounded manifest
read/decode, unvalidated hash identifiers in chunk paths, and an existing
corrupted chunk surviving reimport. All three received the forward repairs above.
Independent review round 2 reported 0 Critical / 1 Important: the malicious
chunk-id test also made the top-level manifest hash invalid, so it returned
before exercising the chunk predicate. The test now keeps a valid BLAKE3
manifest hash and makes only the chunk id malicious. `rustfmt +1.94.1 --check`
and scoped `git diff --check` pass after that repair; managed Cargo evidence
remains pending.

Follow-up source repair: the original summary omitted the resource revision
required by this failure's manifest identity contract. The production manifest,
test fixture, and cache assertions now hard-cut from `ZRARTM02`/schema 2 to
`ZRARTM03`/schema 3 and serialize `ResourceRecord.revision`. The project
restart fixture now supplies the exact v2 magic and proves it is reimported as
v3; no decoder for either older magic remains. The non-default revision fixture
deserializes the published manifest and asserts the persisted value. `rustfmt
+1.94.1 --check` and scoped `git diff --check` both pass after this change.
Managed Cargo evidence remains pending and is not inferred from these static
checks.

The revision regression additionally republishes the same resource with a
changed revision. It asserts that the artifact URI remains stable while the
replacement `ZRARTM03` manifest records the new revision, preserving one cache
location and one current generation rather than allocating a parallel cache
truth.

The focused hard-cut regression writes a valid v3 artifact, changes only its
magic to `ZRARTM02`, and asserts that `ArtifactStore::read` rejects it. This
keeps legacy detection at the canonical store boundary; project restart recovery
remains responsible for reimport rather than a compatibility decoder.

The read boundary now validates a nonzero raw payload size no larger than 2 GiB
before opening chunks, then passes that validated size to fixed-int bincode's
`with_limit`. This admits the required 1 GiB asset matrix with serialization
headroom while preventing a malformed manifest from requesting unbounded bincode
allocation. The regression writes a schema-v3 manifest with `u64::MAX` raw
bytes and asserts rejection before any chunk is read.

Independent review then found the write boundary lacked the same check before
staging, which could publish a generation its reader rejects. The repair
reuses the raw-size validation immediately after serialized-size calculation
and before staging-file creation; the source guard fixes that ordering without
allocating a multi-gigabyte test payload.

The post-repair independent review reported `0 Critical / 0 Important / 0
Minor`: the shared validator is also called before read-side chunk construction,
and the new contract failure remains a typed `Parse` error without masking
serialization or I/O sources.

The follow-up regression review also reported `0 Critical / 0 Important / 0
Minor`: oversized, zero-length, and over-budget manifests each reach their
intended `Parse` boundary, while a rejected kind conflict leaves the prior data
generation readable.

The manifest-capacity review initially reported `0 Critical / 0 Important / 0
Minor`: the maximum 2 GiB generation needs about 32,768 64 KiB chunk entries,
whose fixed-int manifest inventory is about 2.38 MiB before fixed fields. The
4 MiB read/decode cap therefore admits writer output with margin while
retaining a bounded malformed-input allocation; the 4 MiB plus one-byte fixture
locks the rejection boundary. A follow-up review correctly found one important
test-only arithmetic gap: the Zstd conservative compression bound is
`raw_bytes + raw_bytes / 256`, so 2 GiB requires 32,896 64 KiB chunk entries,
not 32,769. The wire-fixture regression now derives that bound without
allocating its payload and asserts the full conservative inventory remains
inside the same 4 MiB limit. Independent re-review reported `0 Critical / 0
Important / 0 Minor`: the corrected fixture derives exactly 32,896 entries,
matches the production fixed-int wire format, and remains below the 4 MiB cap.
Managed Cargo evidence remains pending.

The final publication-boundary review reported `0 Critical / 0 Important / 0
Minor`: serialization failures remain `ArtifactCacheSerialize`, the shared
capacity contract is `Parse`, and atomic publication preserves typed I/O
failure. An oversized manifest is rejected before it can replace the last-good
generation.

Independent review round 3 of the v3 follow-up reported `0 Critical / 0
Important / 0 Minor`. It verified that the production and test manifest wire
order place `revision` directly after `kind`, the written value is
`metadata.revision`, and the `ZRARTM02` restart fixture reimports through the
sole `ZRARTM03` decoder without a compatibility path. No F7 typed-error source
regression was found.

Structure review under `engine-code-structure-convention.md` found the current
artifact owner remains within the production-file budget: `artifact/store.rs`
and its primary test owner are both below the applicable 800-line structure
guard. The public store surface is limited to `write` and `read`; the
implementation has no production panic, TODO, or whole-artifact `fs::read`
path.

The coordinator created generic test job `81f078f49bd746ce8837a8f0b27df37f`
without a compatibility descriptor or source copy. It was released before start
(no command, PID, or Cargo evidence) rather than used as a substitute for the
declared source-bound focused gate.

The public module document `docs/zircon_runtime/asset/artifact.md` still
describes the retired single-file magic-plus-compressed-payload store and its
old structural-audit counts. It must be updated by the documentation owner to
describe the v3 manifest, immutable chunks, revision identity, and current
audit state; this record does not edit that unleased public contract.

A follow-up source audit found that `validate_manifest` previously checked the
manifest's declared compressed total only against its chunk sum. A corrupt
manifest could therefore make the reader traverse more compressed bytes than
this store's Zstd writer can produce for an admitted raw payload. The repair
mirrors Zstd 1.5.7's `ZSTD_COMPRESSBOUND` formula: `raw + raw / 256`, plus its
sub-128 KiB margin. The writer validates the finished encoder output before
sync and publication; the reader validates the aggregate before constructing
`ChunkReader`. The `raw = 1`, `compressed = 65` manifest regression exceeds
the 64-byte bound and proves rejection is typed `Parse` before chunk I/O.

Independent review of the compressed-payload bound reported `0 Critical / 0
Important / 0 Minor`. It confirmed the formula agrees with Zstd 1.5.7, the
streaming writer does not flush a partial frame before the finished-output
check, the reader rejects before any chunk open, and existing writer/read tests
continue to cover admitted output. The preceding r4 source-bound ticket and
copy remain non-terminal historical evidence; this follow-up requires a fresh
r5 source snapshot and managed ticket. No Cargo terminal evidence is claimed
by this source and review record.

The r5 snapshot is now sealed as `1378` with source-manifest hash
`d64819d24bb6cab1cb1d0bcdfa087374f916d40520bcd241e2972aebefa9c570`.
Its focused Windows ticket is
`813a36f0955949dc8ebfcc611f8da91e`, bound to
`cargo +1.94.1 test -p zircon_runtime --lib
asset::tests::assets::artifact_store --locked --jobs 1 -- --nocapture
--test-threads=1`. The matching isolated source-copy request
`aacac92d5a6d47a4b420a0fdfc0b8b3e` was accepted for materialization. These
are receipt facts only: neither a queued ticket nor a materializing copy is
test execution or acceptance, and their eventual terminal evidence must be
read without substituting the preceding r4 source snapshot.

Remaining acceptance is deliberately open: managed focused Cargo evidence,
interrupted-write and benchmark matrices, lazy requested-chunk residency, and
the Runtime11/IBL shared I/O and ownership work remain separate required plan
layers.

### 2026-08-01 requested-chunk residency and structure convergence

- `ArtifactStore` is now a cloneable shared-residency owner rather than a unit
  value. `open_chunk_inventory(...)` validates and exposes immutable generation
  metadata without payload I/O; `read_compressed_chunk(...)` admits one
  inventory index and returns a shared `Arc<[u8]>`. Full artifact decoding uses
  the same reader/cache path. The default residency is a bounded 64 MiB LRU and
  publishes resident bytes/chunks, cache hits, successful disk reads/bytes, and
  evictions. Corrupt chunks are rejected before residency publication.
- The API deliberately calls these compressed chunks: schema v3 stores ordered
  pieces of one zstd frame, not independently uploadable mip/face sections.
  Runtime11/Render13 sectioning must build on this inventory rather than create
  a second index. The last-good fixture prepares and publishes immutable chunks
  for revision 2, drops before manifest replacement, and proves revision 1
  remains readable.
- `store.rs`/`chunk_residency.rs` are 710/437 lines. Runtime04 structure mirrors
  now agree on 25 source owners, 22 guard owners, 28 test anchors, and 24
  behavior anchors. `python -B -m unittest
  tools.tests.test_runtime_asset_pipeline_audit` passed 2/2; direct boundary
  audit reports all missing lists empty and `risks = []`.
- Fresh source-bound focused Cargo receipt: ticket
  `7b90f810f9354ebfa906b247829a8397`, request
  `runtime04-artifact-chunk-residency-20260801-3e7f9bab5f68`, manifest
  `e485e4e5a8916f2a905b1aaea99242528038d79c88dbd1b30762602ead6b1ab0`.
  Its receipt is `queued`; no terminal result or pass is inferred.

Remaining acceptance stays open for that managed terminal evidence, quantitative
4 KiB/256 MiB/1 GiB cold/warm/1% matrices, and the Runtime11/IBL/Render13 shared
I/O and semantic-section ownership layer.
