---
related_code:
  - zircon_app/src/entry/entry_runner
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
tests:
  - inline and editor parser tests: 65
  - rustfmt check: blocked by two current-source formatting diffs
  - current-source managed Windows Cargo pending
  - F0 editor GUI, CLI, and composition startup counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# App entry runner当前源码复核（2026-07-30）

## 范围与当前基线

`zircon_app/src/entry/entry_runner/**`当前源码 **13/13** 个Rust文件、**3,082** 行、**65** 条`#[test]`已逐文件阅读；path+raw-content SHA-256为`2f2d2fd416a9d635e2bb852ffc0a47e784320de76fec309d111ca73e57a3388e`。6个tracked文件与`editor/**`外部未提交内容只读纳入，本轮未修改Rust。

| 模块 | 文件 | 行 | 测试 | 结论 |
|---|---:|---:|---:|---|
| `entry_runner`根生产/参数文件 | 7/7 | 2,353 | 37 | 单次线性参数解析、runtime event loop/session一次创建；无新增稳态热点 |
| `entry_runner/editor/composition.rs` | 1/1 | 99 | 1 | prepared project复用成立；runtime registration report整组深clone仍在 |
| `entry_runner/editor/tests/**` | 5/5 | 630 | 27 | 覆盖GUI/CLI/prepared startup与1/100/1000 plugin规模静态计数 |

## 当前结论

- **PERF-MVP-427已部分落地**：GUI与CLI operation都先构造一个`EditorStartupPreparation`；产品GUI对project只执行一次`ProjectAuthority::open_project`，随后把`ProjectManager` move到`EditorHostRunConfig`，host调用`open_prepared_project`完成资产激活而不按路径重开或重解析manifest。first-party runtime/editor registrations各构造一次并move给bootstrap/host，产品GUI的gateway runtime显式保持projectless。
- **剩余P1深克隆**：`prepare_editor_startup`仍把`project.manifest().plugins`深clone到`EntryConfig`，现有测试明确记录`project_manifest_clone_count=1`和估算heap bytes；公开的`EditorApplicationComposition::open_project`又为core bootstrap执行`runtime_plugin_registrations.clone()`，再把原Vec交给linked runtime。单个`RuntimePluginRegistrationReport`拥有`PluginPackageManifest`、`ProjectPluginSelection`、`RuntimeExtensionRegistry`和diagnostics，后者包含多组typed extension tables，因此clone bytes随插件与贡献规模增长。
- **native plugin仍重复I/O/加载**：entry在`selected_native_editor_plugin_registration_reports`内调用`NativePluginLoader.load_discovered_editor`，host激活后又在`apply_project_plugin_manifest`调用同一loader。两边各自只消费一次本地load report，但同一project generation仍发生两次目录发现、manifest/load/entry与贡献物化；capability collect/sort/dedup只是第一次结果的后续投影，不能抵消第二次load。
- GUI/operation模式探测只clone一次受OS命令行长度约束的`Vec<String>`；diagnostic/runtime session参数均为单次线性扫描，runtime runner只建立一次event loop、wake registration和dynamic session，当前不列为MVP热点。
- 当前13文件`rustfmt --check`只报告`runtime.rs`断言换行和`runtime_session_args.rs` import排序两个外部源码差异；`git diff --check`无新增本轮问题。未运行Cargo或产品启动，因此不能把静态计数测试当作动态GREEN。

## 参考引擎与目标

- Bevy `dev/bevy/crates/bevy_app/src/app.rs:232-245`在plugin readiness阶段以`mem::take`暂时取得唯一registry owner，检查后放回，说明多consumer/可重入阶段可以围绕单一owner传递而不是复制完整registry。
- Unreal `dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp:6951-6956`把plugin module loading作为显式boot timing阶段；Zircon应同样把manifest parse、registration build/clone、core bootstrap、runtime session与host activation分段计时，而不是只记录总启动时间。

Editor01联动Editor12把现有`EditorStartupPreparation`收敛成按entry/project/plugin generation冻结的共享startup artifact：manifest/plugin selections、native load report/projection与runtime/editor registration payload只封存一次，core、runtime session、capability和host消费共享handle或borrowed view；不得为composition另建第二缓存，也不得跨generation保留旧动态库贡献。

## 动态验收

对0/1/100/1,000 plugins分别运行cold/warm产品GUI、CLI operation和`EditorApplicationComposition`，记录project open/manifest parse、native directory/manifest discovery、DLL load/entry call、runtime/editor registration build、extension/catalog/descriptor projection、deep-clone count/bytes、allocation bytes、阶段wall/p95与诊断顺序。验收要求：

- 每project generation按路径open/manifest parse不超过1，prepared asset activation恰好1；
- 每entry/project generation native discovery/load/entry call、runtime/editor registrations、capability/catalog/module descriptor build各不超过1；
- project manifest与composition registration deep clone count/bytes均为0，多consumer只增加共享owner引用；
- 失败回滚、native selection、诊断与加载顺序等价，旧generation可卸载且无悬挂callable；
- current-source managed Cargo、F0产品trace与上述规模counter全部通过后，才从`pending.md`移入`review.md`。
