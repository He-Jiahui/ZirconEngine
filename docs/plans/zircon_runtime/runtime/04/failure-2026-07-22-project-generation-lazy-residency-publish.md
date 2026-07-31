---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: project-generation-lazy-residency-publish
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/open_project.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/resource_sync.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/watcher.rs
tests:
  - cargo test -p zircon_runtime --lib asset::tests::pipeline --locked --jobs 1 -- --nocapture --test-threads=1
  - cold, warm, one-percent change, large artifact, concurrent query and rollback matrices
---

# Runtime04：project generation锁外构造与懒驻留缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime asset pipeline逐Rust文件性能审查，PERF-MVP-499
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：候选generation、registry/resource authority和resident payload发布必须由Runtime04统一拥有，查询端或Editor不能绕过锁域另建缓存。
- 生命周期键：`project-generation-lazy-residency-publish`

## 失败现象与复现证据

project open在generation写锁内clone importer registry、spawn watcher、全量scan/import、同步读取并prepare全部artifact后发布。watch/import/reimport也持generation读锁与project写锁构造深候选、执行全scan、全resource prepare/load/clear/commit。虽然存在`ensure_resident`，每次open和单文件watch仍会把整个project重新驻留，锁时长、I/O与RSS随全项目增长。

## 最低共享层根因

metadata/reference generation和resident payload没有分离，candidate prepare与authority commit也没有阶段边界；昂贵I/O/hash/decode在共享锁内执行。

## 架构修复验收

- 在锁外由Runtime11有界jobs构造immutable candidate inventory/registry/metadata delta；提交只做generation CAS/authority swap并保留last-good。
- metadata/reference发布不读取全部artifact；MVP启动只按startup working set或显式请求single-flight驻留。
- watch按source index与reverse dependency closure准备affected delta，unchanged asset不scan/hash/read/prepare。
- projects/assets 1/1k/100k、artifact 4KiB/256MiB、cold/warm/1% change记录锁wait+hold、reads、resident bytes和worker overlap；发布锁持有近常数，warm read=0，changed近closure。
- 保留rollback、duplicate/GUID/import diagnostics、watch ordering、shutdown flush和并发查询单generation一致性。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止在Editor/scene/render建立第二套project或resident cache truth。
- 禁止仅缩小一层mutex但继续在另一层project/generation锁内全量I/O。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.

2026-07-30 retained-host current-source caller证据：Runtime project activation返回后Editor host先执行一次`EditorAssetManager::refresh_from_runtime_project`，retained-host `finalize_startup_host`又在首帧前无条件通过`sync_asset_workspace`执行第二次；缺少可比较的project/catalog-input generation token迫使consumer用全量重建确认新鲜度。Runtime04交付必须让一次candidate commit同时发布immutable catalog-input generation与consumer watermark，Editor09/retained host可证明每generation只投影一次；不得靠caller跳过refresh或私有cache猜测一致性。验收增加`project/catalog generation id`、refresh/build次数、meta/artifact/reference reads与commit-to-first-frame wall；1/1K/100K资产下每activation构建/发布不超过1，warm unchanged为0。证据：`docs/plans/performance/01/2026-07-30-editor-retained-host-startup-current-review.md`；无动态pass声明。
