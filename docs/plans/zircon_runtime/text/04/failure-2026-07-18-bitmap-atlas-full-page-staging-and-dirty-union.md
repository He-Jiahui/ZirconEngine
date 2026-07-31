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
