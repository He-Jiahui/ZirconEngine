---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: project-catalog-input-generation
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/project/manager/mod.rs
  - zircon_runtime/src/asset/project/manager/registry_access.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/registry/asset_registry_index.rs
  - zircon_runtime_interface/src/resource/resource_record.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/source_generation.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs
tests:
  - cargo test -p zircon_runtime project_catalog_input_generation --locked
  - cargo test -p zircon_editor unchanged_project_catalog_generation --locked
---

# Runtime04：ProjectManager 缺少完整 catalog input generation

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：Editor09 immutable catalog generation / unchanged refresh fast path
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：ProjectManager、ResourceRecord、AssetMetaDocument、artifact revision 与 package roots 均由 Runtime04 持有；Editor09 只能消费该 authority，不能再扫描并复制第二份 project truth。

## 失败现象与复现证据

Editor09 已建立 `ProjectManifest + PackageAssetRegistry + ResourceRecord` typed delta，并把并发 full sync 线性化为 latest-started winner；但该 delta 不能安全证明 catalog 输入 unchanged。`sync_from_project` 实际还读取 source filesystem mtime、完整 `AssetMetaDocument` 的 preview/unit/included-files/entries 等投影，以及 ready artifact 的 direct references。上述输入没有完整进入 `ResourceRecord` 或当前 `AssetRegistryIndex`。

独立复审给出具体反例：只 touch 内容不变的 source 后，source hash 与完整 ResourceRecord 可保持相同，但 `.zmeta.source_mtime_unix_ms` 和 Editor catalog 的 `source_mtime_unix_ms` 必须变化；若 Editor09仅比较 ResourceRecord 就会永久保留旧值。外部 meta 字段变化或 artifact reference generation 变化也存在同类漏判。Editor09 因此已撤掉错误的 unchanged 早退，只保留 typed resource delta 指标；当前仍会执行 meta/artifact load、reference/folder rebuild、shader IDE write 与全 generation publish。

## 最低共享层根因

Runtime project authority 没有发布一个 immutable、单调且覆盖全部 catalog projection 输入的 generation，也没有从两个 generation 生成 added/modified/removed/renamed typed delta。Editor 不能用路径 mtime 拼接、重复 meta hash 或重新加载 artifact 来伪造该 generation，否则会形成第二 authority，并重新引入本 Failure 要消除的全量 I/O。

## 架构修复验收

- ProjectManager/asset pipeline 发布 immutable catalog-input generation；generation identity 至少覆盖完整 ProjectManifest、PackageAssetRegistry、ResourceRecord、source mtime、catalog-relevant AssetMetaDocument 投影和 artifact direct-reference revision/payload。
- 相同 generation 查询为 O(1) identity 比较；不得重新读取 source/meta/artifact，也不得重写 shader IDE environment。
- generation delta 是 typed `added/modified/removed/renamed`，rename 保留 previous/current locator；同 ID 的 meta/artifact/reference 变化必须进入 modified。
- watcher/import/migration/package-root 更新与 generation publish 共享同一线性化边界；旧扫描结果不得覆盖新 generation。
- Runtime 测试覆盖 source touch但hash不变、meta unit/included/entries变化、artifact direct refs变化、package root变化与完全 unchanged；Editor09 消费后连续 unchanged refresh 保持同一 catalog Arc，artifact load/reference build/folder build/shader IDE write计数均为0。

## 禁止临时方案

- 不得让 Editor09 每次遍历并 hash 全部 `.zmeta` 或加载全部 artifact 来判断 unchanged。
- 不得只比较 ResourceRecord/source hash，或用 filesystem mtime 作为唯一 generation authority。
- 不得增加 UI caller catalog cache、第二 registry、兼容旧 channel 或 test-only bypass。

## 修复结果与回传

Open state: `待 Runtime04 提供完整 project catalog-input generation 与 typed delta；Editor09 当前安全保留 full rebuild，不声明 unchanged fast path、10k规模门或本 Failure 已完成。`
