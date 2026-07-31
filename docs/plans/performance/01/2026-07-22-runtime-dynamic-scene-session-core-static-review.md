---
related_code:
  - zircon_runtime/src/scene/dynamic_scene/session
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/bevy/crates/bevy_scene/src/spawn.rs
  - dev/bevy/crates/bevy_asset/src/io/file/file_asset.rs
  - dev/godot/scene/resources/packed_scene.cpp
tests:
  - zircon_runtime/src/scene/tests/dynamic_scene/archive_manifest.rs
  - zircon_runtime/src/scene/tests/dynamic_scene_session
  - current-source Windows zircon_runtime session tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime dynamic scene session核心逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/scene/dynamic_scene/session/**`当前 **563/563** 个Rust文件、**7,573** 行已逐文件阅读；外部`scene/tests/dynamic_scene_session/**`当前16文件、68 tests已纳入动态验收清单。覆盖root/construction/manifest/metadata/reports/slot/slot_store/validation、capture_retention/io/merge/query/restore/retention、slot/selected操作，以及facade/path API/capture/export/merge/mutation/query/restore/retention/transfer/target-path全组合。

## 已直接修复

`RuntimeSessionArchiveManifest::{slots_with_tag,slots_matching_display_name}`原来每次查询都把trim后的输入复制成新`String`。现让返回iterator显式借用self与查询切片，filter全程使用`&str`；行为测试继续覆盖trim/empty/match，新增源码守卫先RED后GREEN，rustfmt/diff通过，归PERF-MVP-473。

## 归档构造与序列化放大

`to_versioned_json_pretty`先深clone完整archive，再normalize/sort/validate/pretty serialize；load先parse完整`serde_json::Value`，embedded scene header检查又将每个scene Value序列化为String并解析DynamicScene，slot scene-document路径随后继续DynamicScene→pretty String→Value→archive serialize和反向往返。Level capture/diff还先`level.snapshot()`深clone World，再capture完整DynamicScene；restore为取得entity count构造宽`node_records()`。

PERF-MVP-474交接Runtime04：scene/session由一次capture/compile发布generation-owned immutable archive artifact，typed serde直接读写，不以String/Value作为内部中转；canonical order、schema/error位置和atomic publish合同由同一artifact负责，preview/query只借用摘要/index。

## 同步路径I/O与无界提交

load/save使用`read_to_string`、完整parse、完整pretty String与同步`fs::write`；atomic save也先把全部payload驻留内存再写temp/rename。任意path mutation均加载整个archive、修改并重写整个文件，调用线程承担I/O、parse、serialize和replace；没有single-flight、byte/time budget、取消、shutdown flush或queue诊断。

PERF-MVP-475交接Runtime11：bounded I/O lane消费Runtime04 immutable artifact，按path+generation合并/取消旧写，streaming writer写temp并flush/fsync/rename；caller只提交ticket/观察结果，队列发布bytes/depth/age/drop/cancel/latency，shutdown有明确flush/cancel策略。

## 查找、预览与批量事务放大

archive/manifest/slot store普遍线性查找且每次push/upsert/rename全量sort。selection先构造完整owned manifest（clone所有summary/metadata/tags并normalize/sort），随后commit又按selected id查找；preview多次`ensure_supported`验证所有embedded scenes，commit再次lookup/validate。merge先preview，再逐incoming重复contains并逐项push/sort；retention/capture preview深clone整个archive，重复构造slot-id set/vector并排序。组合路径可接近O(I*T + I*T logT)，并产生payload深clone。

PERF-MVP-476交接Runtime08：归档采用canonical slot-id index+dense stable order和generation validation ticket；selection直接返回borrowed handle/index，preview生成轻量mutation plan，commit一次验证并batch merge/prune/sort/publish。失败只能丢弃plan，不复制整个archive或留下部分写。

## Facade与path组合核对

最终368个facade/path相关文件逐一确认均为共享底层转发或组合，没有独立的第三套算法；但preview与commit是两个互不关联的调用，path层会分别完整load/validate archive，commit再完整save，无法消费preview ticket。capture/transfer/export/merge的named/selected/metadata/retention组合均放大同一474/475/476根因，因此不按包装函数重复编号；最终修复必须让这些入口汇聚到同一generation artifact、I/O ticket和mutation plan。

## 参考引擎对照

Bevy scene路径把dependency register/resolve与World spawn分层，queued scene持asset handle，ready后才进入World；其file asset backend暴露async reader/writer并对文件资源施加限制。Godot PackedScene实例化消费预打包node/property state。Zircon采用相同原则：prepare产物只生成一次、I/O不占调用线程、apply消费索引化计划；不复制其ECS或对象模型。

## 动态验收

1. current-source Cargo覆盖16个session测试文件、68 tests：archive v0/current roundtrip/canonical errors、manifest query、capture/copy/import/export/merge/retention/selection/restore/path atomic mutation。
2. slots/entities/payload 1/1k/100k及1/64/512MiB记录archive/World/scene/String/Value/metadata clone bytes、validation passes、lookup probes、sort comparisons、main/worker/I/O wall、queue peak/age/RSS。
3. 474要求每generation capture/normalize/serialize至多一次且内部JSON roundtrip为0；475要求caller blocking I/O=0、pending bytes有界、superseded write不发布；476要求lookup O(1)或O(logN)、batch只sort/publish一次、preview payload deep clone=0、失败authority零变化。

受管Cargo、规模counter、F2/F4保存/恢复产品trace未完成，因此该范围继续保留在`pending.md`，不进入`review.md`。
