# 05-subsystem-decoupling-contracts 产出记录归档

> 来源：[`05-subsystem-decoupling-contracts.md`](../05-subsystem-decoupling-contracts.md) 的 `## 6. 状态与产出记录`。

## 6. 状态与产出记录

执行时逐切片填写；完成一个切片更新一行，不许批量补记。

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M1 接缝普查与契约定稿 | Production runtime domain dependency matrix and S1–S4 signature baseline | `frameworks_05_m1_domain_dependency_matrix_2399_refs_80_edges_contract_signatures_locked` | 2026-07-10 | 新增 `tools/runtime_domain_dependency_audit.py`，只扫描 production domain owners，输出稳定 80-edge matrix 与 2399 条 `{source_domain,target_domain,path,line,source}` 逐行证据；测试 owner `tools/tests/test_runtime_domain_dependency_audit.py` 2/2 通过，覆盖跨域发现、自引用排除、root/test owner 排除。机器基线：`05/baselines/2026-07-10-runtime-domain-dependencies.json`；签名基线：`05/baselines/2026-07-10-contract-signatures.md`，锁定 S1 TextLayoutService、S2 AssetLoaderRegistry、S3 RenderSceneExtract、S4 generation handle。当前关键边：asset→ui 3、graphics→ui 4、ui→graphics 19、graphics→scene 13。M2–M4 未执行，不声明接缝清零。 |
| M2 S2/S5 | asset→ui `.zui` loader ownership + crate-root order hard cutover | `frameworks_05_m2_asset_ui_edge_3_to_0_check_ui_18_and_font_cache_contract_passed_full_lib_test_pending` | 2026-07-11 | 复用既有 `AssetImporterHandler` / `AssetImporterRegistry`，不新增同义 registry；硬删除 asset builtin `zircon.builtin.ui_document.zui`、`import_ui_zui_asset.rs`、`ui_v2_document_import.rs`，`.zui` 只由 `ui_document_importer` runtime plugin 注册；asset DTO wrapper 使用 `asset/assets/ui/{document_loader,resource_references}.rs` 本域 helper，crate root 声明顺序注释删除。机器基线复测为 2401 refs / 79 edges，asset→ui 3→0；Python 静态/扫描测试 5/5 通过。focused Rust 首轮 16/18 暴露两个默认 importer 旧 fixture，改为显式安装 plugin fixture 后，默认 feature lib-test binary 精确执行 `asset::tests::assets::ui` 为 18/18、0 failed、7433 filtered、测试体 0.24s；完整 Runtime lib check 同日通过（5m24s，418 warnings）。扩大 `asset::tests` 首轮为 393/395：字体 artifact 因 authoring skip 形状进入 bincode wire 而 `UnexpectedEof`，Vampire shader 断言属于活动 Shader/PBR owner。字体 cache 已硬切到独立完整字段 DTO，无 legacy reader/compat path；`core-min` production lib check 4m59s 通过，公开 `.zasset` 磁盘往返 contract 覆盖 composite/variable/metadata/metrics/cmap 并 1/1 通过。当前默认 lib-test 被活动 rich-text 迁移的 6 个外部编译错误阻断，修复后 asset 全组与计划要求的全量 Runtime lib-test 仍 pending，不声明 M2 完成。 |
