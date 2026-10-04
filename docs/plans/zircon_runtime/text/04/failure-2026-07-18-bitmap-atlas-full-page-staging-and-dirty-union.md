---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: bitmap-atlas-full-page-staging-and-dirty-union
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/atlas/bitmap_run/staging.rs
  - zircon_runtime/src/text/atlas/bitmap_run/tests/persistent_slots.rs
  - zircon_runtime/src/text/atlas/bitmap_run/staged_upload.rs
  - zircon_runtime/src/text/atlas/dirty.rs
  - zircon_runtime/src/text/atlas/page_shadow/commit.rs
  - zircon_runtime/src/text/atlas/page_shadow/patch.rs
  - zircon_runtime/src/text/atlas/page_shadow/store.rs
  - zircon_runtime/src/text/atlas/upload.rs
  - zircon_runtime/src/text/native_bitmap_atlas/storage.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/storage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/binding.rs
---

# Bitmap atlas整页staging与dirty union放大

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/atlas/{bitmap_run.rs,bitmap_run/**,dirty/**,page*,shelf_allocator.rs,upload/**}`当前源23/23 Rust文件及WGPU binding调用图
- 修复责任计划：`docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md`
- 联动责任：WGPU texture write layout联动Render17，stable slot/allocator回链PERF-MVP-231。
- 交接原因：dirty region、page shadow/staging与glyph slot属于Text04 atlas owner；WGPU row layout和实际write属于Render17，不能只在CPU planner改变stride而不验证backend约束。

## 失败现象与复现证据

PERF-MVP-243：512×512 page只要有一个小dirty glyph就新建并清零完整CPU staging Vec，R8为256 KiB、RGBA为1 MiB；partial request随后仍借用整页slice。dirty page只保存所有rect的外接union，未来persistent slots上的稀疏变化会上传大量未变区域。

本轮已直接删除`copy_upload_source_bytes`按glyph高度分配的row-range Vec：用末行常数上界检查保证失败前不部分写入，再直接逐行copy；源码RED→GREEN门禁、rustfmt和diff check通过。整页staging/dirty-region协议仍open。静态证据见`docs/plans/performance/01/2026-07-18-text-atlas-bitmap-upload-static-review.md`。

## 最低共享层根因

upload command使用page-wide `bytes_per_row/source_offset`，staging对象也按完整page建模；dirty tracking只有单一bounding rect，无法同时表达“多个小region”与write-count/byte成本。atlas又尚未持久保存glyph slot/allocator，所以CPU page bytes、dirty state和GPU residency没有统一generation owner。

## 架构修复验收

- 与PERF231一起建立generation-owned glyph slot、allocator及page residency；stable frame不得创建dirty/staging/upload work。
- Text04在明确CPU/GPU byte预算下选择persistent CPU page shadow或packed dirty-region staging；小rect每帧不得新分配/清零完整page。
- dirty page保留有界region集合，邻近rect按额外字节成本合并；write count、row alignment和upload bytes达到可配置阈值时才显式提升full page。
- Render17让binding/write消费新的region layout，验证`source_offset/bytes_per_row/rows_per_image`及WGPU限制；不得为适配接口再复制成整页Vec。
- 单个8×16 glyph分别在R8/RGBA页记录staging allocated/touched、upload bytes和write count；临时整页256 KiB/1 MiB清零为0。
- 同页两个对角rect记录payload sum、merged area和write count；上传必须选择可解释的multi-region或threshold full-page，不能无条件bounding union。
- 1/100/1k changed glyph与stable 300 frames记录dirty count/area、staging alloc/zero/copy、upload bytes/writes、CPU p50/p95；stable staging/write/upload=0。
- page rebuild/face invalidation允许明确full-page路径；generation requeue、Alpha/Color/Subpixel、clip/padding与Softbuffer/WGPU/RenderDoc像素等价。

## 禁止临时方案

- 不得只缩小atlas page来降低整页清零；page packing、draw count和GPU residency成本会转移。
- 不得强制每glyph一次`queue.write_texture`；必须有region合并与write-count预算。
- 不得只把整页Vec放进长期cache而缺少CPU/GPU byte hard cap和generation失效。
- 不得把staging allocation bytes与实际GPU upload bytes混成一个counter。

## 修复结果与回传

2026-07-31 前向收敛：当前 staging 在生成每个 upload target 后优先从 generation-matched CPU page shadow 回放目标矩形，再覆盖本帧 source copy；无 shadow 的持久页禁止普通 dirty threshold 升级为整页，eviction/rebuild 则同时失效 slot/shadow。新增 `render_text_atlas_full_page_replay_preserves_existing_persistent_slot`：已 commit 的 8x8 旧槽位与同页 56x64 新槽位触发 64x64 full-page command，断言 staging 保留旧槽位左上和右下像素。仅完成前向实现与 scoped rustfmt/diff 静态检查，未运行 Cargo 或 WGPU 产品渲染。

2026-07-31 mixed-storage 前向修复：连续 `AlphaMask → Color → AlphaMask` storage split 都从同一帧 atlas clone 构建，故 `zero_initialize_shadow_pages` 只是待合并的 commit，不能作为后一个 Alpha split 的 replay capability。`native_bitmap_atlas/storage.rs` 现在只认可 atlas 中已存在的 generation-matched shadow；没有它时，含另一 split persistent slot 的页保持 partial rect upload。新增 `native_bitmap_atlas_storage_split_does_not_promote_later_alpha_to_full_page`：8x8 Alpha、Color、56x64 Alpha 共用同一 Alpha page，锁定后一个 Alpha 为 partial command/compact staging，防止其整页零填充覆盖前一 split。仅完成前向实现与 scoped rustfmt/diff 静态检查，未运行 Cargo 或 WGPU 产品渲染。

2026-07-31 shadow replay follow-up：即使 atlas 有上一帧 committed shadow，它也不含同一帧、另一 storage split 刚写入的 Alpha slot。storage owner 现检查同页是否有当前 split 之外的 upload copy；存在时禁止 full-page replay，仍保留有界 dirty region 合并而非逐 glyph write。新增 `native_bitmap_atlas_storage_split_does_not_replay_stale_shadow_over_new_alpha`：先提交 cached 8x8 Alpha shadow，下一帧执行 `cached Alpha → new Alpha 8x8 → Color → later Alpha 64x48`，后一个 Alpha 覆盖 75% page 仍必须生成 compact partial staging，不能以旧 shadow 擦除前一 split 的新 slot。仅完成前向实现与 scoped rustfmt/diff 静态检查，未运行 Cargo 或 WGPU 产品渲染。

2026-07-31 独立静态复审：fresh page pending-zero-init 与 committed-shadow sibling split 两个覆盖路径均复核为 P0/P1/P2=0；renderer 按 atlas format 复用 texture-array，partial shadow patches 按 split 顺序合并，不能再覆盖 sibling 像素。该结论不替代受管 Cargo/WGPU 和产品像素证据。

Open state: `等待Text04联动Render17/PERF231受管 current-source Cargo 与产品 WGPU/像素证据；在收到成功回执前，本 failure 不转 fixed。`。

## 2026-09-25 current-source rolling reconciliation (bitmap staging r1)

Coordinator Session `failure-roll-01a084c8-text04-bitmap-staging-r1` reclaimed the exact
failure record and the eleven related atlas/upload paths. No Rust source was edited in this
session. The current source probe emitted
`TEXT04_BITMAP_ATLAS_CURRENT_SOURCE_PASS 11 files`, covering page-shadow replay and generation
guards, retained dirty-region merge/full-page threshold policy, compact row staging, storage
split handling, persistent-slot regressions, and WGPU source-offset/row-layout bounds.

The immutable current manifest captured before this record update is:

| Path | SHA-256 |
| --- | --- |
| `zircon_runtime/src/text/atlas/bitmap_run/staging.rs` | `3506cd400510b962d784cb57ef86c468a03f8dede086101a2c01a062e3c7ed08` |
| `zircon_runtime/src/text/atlas/bitmap_run/tests/persistent_slots.rs` | `cc88a00eb0b02f9df9d19339e36ad849606d9449cbc457f14dcbdc4a39f03dc3` |
| `zircon_runtime/src/text/atlas/bitmap_run/staged_upload.rs` | `22f3218a03a26274d3ce80d968d2c4e33c8428c6f83e0439bbc6beca477ec536` |
| `zircon_runtime/src/text/atlas/dirty.rs` | `e30c76fe72a1adfa5fd205ecb06f040b96d6e6a606941df6189ee16ade82f6cf` |
| `zircon_runtime/src/text/atlas/page_shadow/commit.rs` | `80d7e3d2ad8f2d1d68013f2e19f1207a6dcfb69fd2540cda27fb4c5763927018` |
| `zircon_runtime/src/text/atlas/page_shadow/patch.rs` | `a25ddc5b4882840dcbf1045620fbb45adecaf0e080d863a67c2bd2403b1a41e9` |
| `zircon_runtime/src/text/atlas/page_shadow/store.rs` | `32a4cb0cf0fd9a0fb07b977b4174e3ad050251b5c42c75fdb2d11da3cac200cc` |
| `zircon_runtime/src/text/atlas/upload.rs` | `a8d248f11eea985c8abfe59d9fb00044360e17f948ac0b1ec8cd38d66623333a` |
| `zircon_runtime/src/text/native_bitmap_atlas/storage.rs` | `368f0798da3c071d51104acf780114b6b10717afd81b4d34b69504f0c4e6631a` |
| `zircon_runtime/src/text/native_bitmap_atlas/tests/storage.rs` | `7a4eb7e585f0d44da298810c3ee0fc8f8045d529016307b45a28434ba8c7bde5` |
| `zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/binding.rs` | `41cc3c99b86b03083007274fe22eb49985ebc0f7f54c5b7f47ffc74df1fc626f` |

The working-tree edits in `staged_upload.rs`, `persistent_slots.rs`, `page_shadow/commit.rs`,
`native_bitmap_atlas/tests/storage.rs`, and `atlas_texture_upload/binding.rs` predate this
Session (the binding/staged-upload capacity changes are foreign current-source edits); they are
recorded as unowned drift and are not absorbed. `git diff --check` is clean. Strict
`rustfmt +1.94.1 --edition 2024 --config skip_children=true --check` remains non-green only
on the documented pre-existing formatting drift in five dirty files; no formatter result is
reused as dynamic acceptance. The authoritative post-reconciliation document/source manifest
is coordinator snapshot `3835` (superseding the wording-only intermediate snapshot `3834`).

This is static handoff evidence only. Managed Windows Text04 Cargo, dirty-region/stable-frame
scale, Render17 row-layout/WGPU and RenderDoc/pixel evidence, PERF-MVP-231 slot coupling,
independent review, canonical fixed return, closeout and WeCom remain pending; historical jobs
and prior static receipts are not reused as current dynamic acceptance.

Independent review receipt (2026-09-25): the reviewer rechecked coordinator snapshot `3835`
and confirmed all eleven source hashes, the five-file dirty provenance/count, the static source
probe, clean scoped diff check, rustfmt caveat, and explicit pending Cargo/WGPU/RenderDoc/perf
gates. Review result is Critical `0`, Important `0`, Moderate `0`. This receipt is carried
forward in the post-review coordinator manifest snapshot `3836`; snapshot `3835` remains the
reviewed source state and `3834` is wording-only intermediate evidence.

### 2026-09-08 Shared Upload Bytes Test Repair

Stable fixing Session `failure-roll-01a07160-text04` preserves this lifecycle.
The text-only managed job `9e3f8ac2a6564bd48fbeab45874342e3` reported three
test comparisons between the current `Arc<[u8]>` staging storage and obsolete
owned `Vec` expectations. Source `3091`, request
`a1e848c73e1042c7b9264a60a433e073`, compares borrowed slices instead, keeping
every expected byte, target rectangle, packed row stride and failure check:

- `zircon_runtime/src/text/atlas/bitmap_run/tests.rs`, hash
  `0c34b57af01fd73a477bdaf28172d340c674db35993f4167d7b3e7cbfa811ceb`.
- `zircon_runtime/src/text/raster/swash/tests.rs`, hash
  `c34a0fc6286da14683db2f67a510dc180c1b2a1f34b8bb020249173bb482fe34`.

Their pre-edit hashes are in snapshot `3086`, with exact archived attribution
and audited transfer `4d260401693b4376983c7fd585acbf72`. Existing production
and prior regression changes remain intact. Formatting and whitespace checks
pass; no dynamic pass is inferred. A fourth byte-comparison diagnostic in
`atlas/bitmap_run/tests/dirty_upload.rs` belongs to a currently unproven dirty
revision: hash `971c6c7eca190fbf2acdc7f90c809c700711c15befc06f547a718f09d588dc34`
matches neither HEAD nor its retained attribution, and no source snapshot
contains those bytes. That path is suspended pending provenance; it was not
edited or included in Text04 ownership. The original dirty-region scale,
stable-frame, Render17 upload and product-pixel gates remain open.

The subsequent text-only managed job `1f242fb1789e44ce87d7a5cacc569d5f`
compiled source `3091` without diagnostics in either edited byte-comparison
file. Its exact input/receipt is recorded in the sibling persistent-slot
failure: `runtime-text-owner-test-support-3095-20260908`, digest
`4e5f800cd19e5748464ac759b4e1efbfae073ecf1c05d419327a6d6739cbe9e2`.
Three unrelated/unowned fixture errors remain, including the suspended
dirty-upload path above, and no tests ran. The source review and original
dynamic acceptance remain pending.

### 2026-09-08 Dirty Upload Provenance Resolved

The existing coordinator-efficiency task completed the six-file 3091 review at
C0 / I0 / M0 (`.codex/tmp/text-3100-review-20260908-result.txt`). This supersedes
the source-review-pending statement above; dynamic atlas acceptance is still open.

The suspended `dirty_upload.rs` preimage was compared with HEAD through the
repository's Rust 2021 / style-edition 2024 rustfmt, using stdout only. Both
canonical outputs were identical with zero differing lines: the unexpected
preimage hash was due solely to import ordering and formatting. No semantic
foreign change was present. Transfer preview `81a450e1bc514547b7281233b98ceb47`
was eligible, and apply `96a474ee9db24c1abeb6d2ca8e21d36a` assigned the exact
bytes to the stable Text04 Session. Pre-edit snapshot 3117 preserves the original
`971c6c7eca190fbf2acdc7f90c809c700711c15befc06f547a718f09d588dc34` bytes
and this record; the archived attribution remains visible in the transfer.

Source snapshot 3118, request `15de81f26efd49308de240b615366393`, changes only
the semantic assertion `staging.pages[0].bytes` to `.bytes.as_ref()`. The exact
128-byte array, off-origin rectangle, packed row stride, zero source offset,
rows-per-image and failure assertions are preserved. Current source hash:
`665d619ce3fb1c5ed8f45691e24953caadcefba715b387122e096bc4f275ddff`.
Repository rustfmt and scoped whitespace checks pass. This source was not part of
the earlier 3091 review or 3095 managed input; its own compilation and incremental
review remain required. The original scale, stable-frame and product gates remain.

### 2026-09-08 incremental review and compiler result

The existing coordinator-efficiency task reviewed source 3118 and record 3121 at
Critical 0 / Important 0 / Moderate 0. Its preserved result is
`.codex/tmp/runtime22-text-3122-review-20260908-result.txt`; the preimage provenance,
128-byte packed upload, off-origin rectangle, stride and failure assertions were checked,
and selected hashes remained unchanged before and after review.

Managed graphics job `77612283b03b47af9d69d7c58500c323` used the exact derived input
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime-graphics-text-consumers-3120-20260908`,
digest `e6d232251b629d6bf665204dcc7bd222c8f958ed13a04854490f48c6ee442a00`.
Its `results/graphics-library-3120.{json,log}` and compiler-diagnostics JSON preserve
25 external compiler errors and zero tests. No diagnostic remains in the 3118 path.
Source review is complete; dirty-region scale, stable-frame, Render17 upload, WGPU/product pixel
and formal fixing-Session acceptance remain pending. This lifecycle stays open.
