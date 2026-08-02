---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: runtime04-gltf-importer-current-compile-errors
origin_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/zircon_plugins/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets.rs
tests:
  - cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --lib --features backend-zr-vm zr_vm_backend_has_one_plugin_owned_dense_production_path --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib asset::tests::assets::gltf_importer --locked --jobs 1 -- --nocapture --test-threads=1
---

# Runtime 04: current glTF importer changes do not compile

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 来源执行切片：Plugins08 ZR VM language runtime exact locked consumer validation
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：最低共享根因位于 Runtime04 正在实现的 glTF importer，而不是插件工作区锁文件、ZR VM 原生绑定或 Plugins08 运行时。

## 失败现象与复现证据

当前源码受管运行证据：

- reservation：`966b51b8fe094b6dbd0f8fc744d0d167`；
- job：`3ddd0739370d488a9920f1480540bd6b`；
- run：`05276475daf343f9a479b22bd04b3c75`；
- 终态：`completed`，exit code `101`，目标测试执行数 `0`；
- 已跨过：`meshopt 0.6.2`、`float-cmp 0.10.0`、`zircon_runtime` 依赖解析，以及 `zr_vm_rust_binding_sys` 原生链接环境发现。

编译在目标 Plugins08 测试启动前暴露两个 Runtime04 当前源码错误：

1. `zircon_runtime/src/asset/importer/ingest/import_gltf.rs:117`：E0422，使用
   `ModelPrimitiveAsset` 但当前 import list 未导入该公开类型；
2. `zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets.rs:262`：E0282，
   `vec![None; nodes.len()]` 在第 266 行 closure 使用前不能推断 `Option` 的元素类型。

这两个文件属于尚未完成受管验证的 Runtime04 WOC glTF/meshopt/WebP importer 工作。已有 canonical lifecycle：
`docs/plans/zircon_runtime/runtime/04/failure-2026-07-17-woc-gltf-meshopt-webp-import.md`，当前状态为
“实现完成，受管验证待回执”。本记录只回传从 Plugins01/Plugins08 exact consumer 暴露的当前编译边界，不另建并行实现生命周期。

## 最低共享层根因

Runtime04 的 hard-cut importer 实现已开始构造 reference-only `ModelPrimitiveAsset`，但声明未进入
`import_gltf.rs` 的实际作用域；同时 generation hierarchy index 的 `skin_by_joint` 初始化缺少足以在 closure
前完成推断的显式 `Vec<Option<usize>>` 类型。两处均是当前 Runtime04 实现的内部编译完整性问题。

## 架构修复验收

- 在 Runtime04 所有权下完成最小 hard-cut 修复：导入 canonical `ModelPrimitiveAsset`，并明确
  `skin_by_joint` 的 `Vec<Option<usize>>` 类型；
- Runtime04 focused glTF importer gate 在相同 current-source manifest 上实际运行并 GREEN；
- 原 Plugins08 exact locked consumer 命令随后在相同 current-source manifest 上实际运行目标测试并 GREEN；
- 独立 review 确认没有恢复重复 primitive payload、没有回退 WOC meshopt/WebP 优化，也没有引入兼容 shim。

## 禁止临时方案

- 不得从 Plugins01 会话跨计划修改 Runtime04 源码；
- 不得删除、屏蔽或 feature-gate 掉当前 WOC importer 实现以绕过编译；
- 不得移除 `meshopt`、手工回退插件工作区 lockfile，或跳过 `--locked`；
- 不得把“编译推进到 Runtime04”记录为 Plugins08 测试通过。

## 修复结果与回传

Open state: `编译修复已存在，受管验证与独立审查待终态`; no pass is claimed.

- `import_gltf.rs` 已导入 canonical `ModelPrimitiveAsset`；`gltf_animation_subassets.rs` 已将
  `skin_by_joint` 明确为 `Vec<Option<usize>>`。两处均为 hard-cut 编译完整性修复，没有恢复重复 primitive payload、移除 meshopt/WebP 路径或增加兼容 shim。
- 精确 `rustfmt` 与 `git diff --check` 已通过。
- Windows focused validation receipt：ticket `f8fe64c14b324e238691970284a44c35`，request
  `runtime04-gltf-current-compile-fix-20260801-0f1d7f304a10`，4-path source manifest
  `0f1d7f304a10e57a9d2ead2b22fc68ea652bb50930e8a3fecd81172571f19a58`，command
  `cargo +1.94.1 test -p zircon_runtime --lib asset::tests::assets::gltf_importer --locked --jobs 1 -- --nocapture --test-threads=1`；receipt状态为`queued`，不等于GREEN。
- current-source focused GREEN 与受管提交证据返回后，Plugins01 才能在协调器FIFO下消费原 exact validation；不得重建已经终态的validation copy或重放旧请求。
