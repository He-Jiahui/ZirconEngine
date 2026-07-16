---
record_kind: milestone_slice
plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
milestone: S15.4/S15.5
slice: blend-space-preview-toolbar-chip-density
status: implementation-complete-runtime-interface-build-blocked
date: 2026-07-14
---

# Layout 15：Blend Space Preview 工具栏组件密度

## 目标与根因

1260×780 的现有生产截图保持了三档响应式边界，但 Preview 区只把 `Perspective` 与 `Lit` 绘制成 20–24px Caption，视觉层级和工具栏节奏弱于 Unreal 参考。根因在 L4 workspace 的 Preview toolbar 未消费既有标准控件，并非截图、native painter 或 Runtime Text shaping 的问题。

## 实现

- Preview toolbar 改为共享 28–32 logical-pixel control rhythm。
- `Perspective` 与 `Lit` 直接消费 `WorkbenchChip`；它们继续使用共享 token、Runtime Text 和相对 HorizontalBox 布局。
- Preview viewport 高度从 132–184 调整为 164–216 logical pixels，内部图片继续通过 Stretch 获取剩余空间；没有加入绝对窗口坐标、局部 RGB/字体覆写或 feature-specific painter。
- 新增 production-bridge 合同，要求两个共享 Chip、标准工具栏高度、可读的最小 viewport 高度，以及图像至少 110px 的有效高度。资源结构断言遵循现有 `.zui` 合同的源码级检查；`.zui` 数组末尾逗号由引擎资源加载器接受，通用严格 TOML 解析器不能作为该资源格式的验收器。

## 当前验证

- 通过：此前受管 `Cargo build -p zircon_editor` 当前源码构建成功；`verify-native-extension-module-contract.mjs` 通过（44 modules、886 routed/unique bindings）。
- 基线失败（不在本切片范围）：`verify-web-native-handoff.mjs` 仍报告 6 项 generated-bottom-panel matrix 问题；该脚本不覆盖 Blend Space preview asset，本切片未触碰对应 native-web 交接面。
- 受管 coordinator 阻塞已修复：creation identity、动态 loopback 端口以及 supervisor 的非-Cargo 控制子进程过滤已落地；Cargo lifecycle 39/39、server 24/24、Pester 1/1 都已通过。`validate-matrix.ps1 -Package zircon_editor -SkipBuild` 已能进入 Cargo test，而不是被旧 PID 复用阻断。
- 先前确定性阻塞：`ScreenSpaceUiTextSystem::new` 的调用点传入 `ProjectAssetManagerAccess` 并使用 `?`，当前构造器要求 `Arc<ProjectAssetManager>` 且返回非 `Result`，产生 E0308/E0277。最新受管重试已越过该编译位置，但 Runtime Text 01 尚未按 handoff 回传验收；失败交接仍保持 open，见 [`Runtime Text 01 constructor/access drift`](../../../zircon_runtime/text/01/failure-2026-07-14-runtime-text-ui-system-constructor-drift.md)。
- 当前确定性阻塞：最新受管重试在 `zircon_runtime_interface::serialization::binary` 停止；`BinaryValue`/`BinaryValueError` 叶子 `pub(super)` 与父 owner re-export 的可见域不匹配，产生 11 个 E0364/E0365/E0603。该 Binary versioned payload 属 Editor11 M3.1，而非 Layout/Text；失败交接见 [`Editor11 BinaryValue visibility compilation`](../../editor/11/failure-2026-07-14-binary-value-visibility-compilation.md)。
- 共享路径诊断（不作为验收）：最后可运行的同一 editor test binary 中，`builtin_v2_template_file_cache_is_reused_across_runtime_instances` 在 1.52s 通过，说明 24 个 startup documents 的 V2 cache load/reuse 不是停顿点；`startup_template_runtime_loads_componentized_workbench_window_bridge_source` 在其后的 Workbench surface/bridge 构建阶段连续占用约 58.9 CPU 秒、60 秒仍未返回，已按验证进程身份终止。Workbench root 传递闭包为 122 个 `.zui`、2,155 个声明节点，展开后约 2,035 个 surface 节点并合并 809 条 stylesheet rules；恢复本切片 Preview Chip/高度约束前后均复现，故不能归因于本次资源改动。Runtime Interface 编译恢复后必须先用边界计时定位 V2 surface/style/layout/projection 的最低共享层，再向上重跑截图。
- 截图状态：未复用旧截图作为新证据，未把图片写入 `target`；待 Runtime Text 编译恢复且共享模板加载收敛后，只刷新 `docs/tests/editor/editor-window-m3-blend-space-workbench-{640x520,900x620,1260x780}.png`。

## 后续

等待 Editor11 M3.1 收束 Binary owner 可见性并回传；同时等待 Runtime Text 01 完成既有 handoff 的验收回传。之后重跑受管包验证、当前源码合同和 ignored production-window capture。只有三者通过并完成三档截图目视复核后，才把本切片状态改为验收通过。
