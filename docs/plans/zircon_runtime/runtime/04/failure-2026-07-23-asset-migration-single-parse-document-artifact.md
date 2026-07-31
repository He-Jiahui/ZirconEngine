---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: asset-migration-single-parse-document-artifact
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/assets/material/mod.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/asset/assets/project_document.rs
  - zircon_runtime/src/asset/assets/project_document/codec.rs
  - zircon_runtime/src/asset/assets/project_document/material.rs
  - zircon_runtime/src/asset/assets/project_document/model.rs
  - zircon_runtime/src/asset/assets/project_document/scene.rs
  - zircon_runtime/src/asset/migration/document.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/document_migration.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib asset::assets::project_document::material::tests::public_serializer_rejects_unsupported_material_version_before_resolution --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib asset::tests::migration::project_commandlet::document_migration::retired_project_reference_without_subasset_omits_toml_null_and_is_idempotent --locked --jobs 1 -- --nocapture --test-threads=1
---

# Runtime04：asset migration single-parse document artifact缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime asset migration 性能审查 PERF-MVP-511；经批准从 single-inventory lifecycle 拆分。
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`

## 失败现象与复现证据

Single-parse/null-omission focused job 已 1/1 green，但独立 closeout review 为 C0/I1/M0：public `ZMaterialDocument::to_project_toml_string` 对公开构造的 `version = 1` 不再返回 `ProjectDocumentError`，而是输出 unsupported document。

## 最低共享层根因

一次 typed artifact 改造删除了 serializer 末端的 reparsing validation，却没有把 version contract 下沉到借用型 serialization boundary；deserializer 仍校验 version，public serializer 因此出现不对称回归。

## 架构修复验收

- 一个 canonical scalar/borrowed version validator 同时服务 serialize、deserialize 与 test-only TOML reader，不 clone `ZMaterialDocument`、不 reparse String。
- Public serializer 在调用 reference resolver 和生成 bytes 前拒绝 unsupported version，并保留原 typed `ProjectDocumentError` 链。
- Existing null omission、labeled subasset、public AssetRef JSON 与 clone-free typed artifact tests 保持 green。

## 禁止临时方案

- 禁止恢复 serialize 后 TOML reparse、clone 完整 material document、call-site 特判、test-only bypass 或放宽 version contract。

## 修复结果与回传

Open state: `snapshot 1075 的 null-omission focused gate 1/1 green；closeout 1707235a7e8e447db1324a8507ecbe41 因独立 review C0/I1/M0 未登记 accepted review。managed job e91e54e602824524bf56a48360bcd38e/run 7ac9642321014a77adbf3460ba5a918d 在 snapshot 1084 上完成真实 RED：running 1 test，目标 public serializer contract 0 passed/1 failed/8897 filtered；其 unwrap_err 收到 version = 1 的 Ok TOML，证实 serializer 在 resolver/bytes 前缺少 version gate。第一次 GREEN 尝试 job 92f8362cd055456ca26a85af22536c32/run 8dc8066f034440b1a56595ef12308994 在 snapshot 1090 自然 released exit 101；lib-test 编译在 material.rs:50 以 E0282 终止，目标测试未运行。这是新增 scalar validation 后 decode_document 的结果类型不再被后续推断的 Runtime04 编译阻断，不是 contract RED。修复将该局部变量显式标注为 ZMaterialDocument，保留唯一 scalar validator 和原有 resolver 顺序。第二次 GREEN 尝试 job 06edb98ec3d1418e8fdde637e293d2c4/run 35df6d34a91d40df8627afaeaa3abede 在 snapshot 1091 自然 released exit 101，目标测试仍未运行；当前 shared native-plugin loader 编译阻断为 native_plugin_load_report/tests.rs:388、409 缺少 NativePluginLoadProjection（E0425），以及 discover/authority.rs:431 以 struct literal 构造新增私有 projection 字段的 NativePluginLoadReport。该三项不在 Runtime04 路径所有权内；必须由 native-plugin owner 修复后，再以新的 FIFO reservation 重试 Runtime04 GREEN。`

## 2026-07-27 recovery status

- 原始 document artifact source/test scope 已恢复并通过 exact `rustfmt --check`。migration production path 只构造一个 `ProjectDocumentArtifact`，在变更时才写出 pretty bytes，并把同一 artifact 移交 formal reader；`original.clone()`、`artifact.clone()`、`artifact.value().clone()`、`self.value.clone()` 与 legacy reparse helper 只出现在测试的负向源码守卫字符串中，不存在于 production migration path。
- material serializer 在 `encode_document` 和 reference resolver 前调用唯一的 `validate_zmaterial_version`；artifact deserializer 与 test-only TOML reader 使用同一 validator。局部 decode 类型已显式为 `ZMaterialDocument`，保留 typed error conversion。
- 尚未为当前源码申请新的 Cargo reservation：indexed-resolver 的 source snapshot admission 刚刚 timeout，且没有返回 snapshot ID/job。此处不把静态检查或旧 snapshot 的编译阻断误记为 GREEN；状态保持 `open`，等待控制面恢复和 shared native-plugin compile owner 的修复后，按声明的两个 focused gates 重试。
