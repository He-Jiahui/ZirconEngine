# Shader06 EC-M5 Current-Source Interactive Viewer Delivery

Plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
Milestone: EC-M5
Status: historical-baseline; current-source revalidation pending
Files: ["docs/plans/zircon_runtime/shader/06/2026-07-15-current-source-interactive-viewer-delivery.md", "docs/tests/runtime/shader/runtime_shader_pbr_interactive_viewer_current_source_20260715.png", "docs/tests/runtime/shader/zircon_shader_pbr_viewer_current_source_20260715_validation.md", "docs/tests/runtime/shader/zircon_shader_pbr_viewer_current_source_dx12_renderdoc_20260715_capture.rdc", "docs/zircon_runtime/tests/runtime_shader_pbr_hdri_export.md", "zircon_app/src/bin/zircon_shader_pbr_viewer/camera.rs", "zircon_app/src/bin/zircon_shader_pbr_viewer/hdri.rs", "zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs"]

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| EC-M5 | 当前源码交互查看器架构收束 | `completed` | 2026-07-15 | Viewer 通过真实 `TasksModule.descriptor()` 和 Foundation/Tasks/Asset 模块激活链取得 `ProjectAssetManagerAccess`，再创建生产 `SceneRenderer`。独立复审关闭 runtime/renderer 析构顺序风险后，又发现结构门禁会搜索测试自身 anchor；当前已将搜索限制到 `#[cfg(test)]` 前的生产源码，并以 production-anchor-removal 模拟确认会 RED。受管 job `03c5072d8e5b4214b146fd620027324d` 随后在修正后的当前源码上通过 viewer 18/18。 |
| EC-M5 | Windows production build 与手动控制 | `completed` | 2026-07-15 | 受管 job `1b86e228d43f4fad9edd9f0ef69d48df` 对 `zircon_app` build/test 均 exit 0。fresh EXE `--help` exit 0；Lakes 2K + source512 + PMREM512 DX12 窗口 26/26 个 5 秒样本保持响应并进入 Ready。原生滚轮令 446,585 像素发生变化，平均 RGB 绝对差 8.4627；343 像素左键拖动将标题从 yaw 0 改为 yaw 120。 |
| EC-M5 | 实际窗口截图与 RenderDoc | `completed` | 2026-07-15 | 1296x999 窗口截图 SHA256 `90D45BD3256C323275BCF112551264DD26DBFEA7D5550F24C7685C7B1D3A1354`；原尺寸复核确认 Lakes 道路、湖岸、树木、天空和球面镜像连续清晰，镜面球不是白球且无低分辨率马赛克。fresh DX12 RDC 为 48,462,499 bytes、SHA256 `1F12B9B03C0E3C2B8D1ED5068868C3FD589DB76578F639E06E277C11ABDBD0BC`，一次性捕获和 replay 均退出 0。 |
| M5 | 产物位置与范围审计 | `completed` | 2026-07-15 | PNG、RDC 和验证 Markdown 均位于 `docs/tests/runtime/shader`；仓库 `target` 与本次受管 Cargo target 的精确文件名扫描均为 0。生产 EXE、交互、截图、捕获和回放晚于全部生产修正；其后的测试专用搜索范围修正已由受管 job `03c5072d8e5b4214b146fd620027324d` 重新执行 viewer 18/18。 |

## Current-Source Status

The entries above remain immutable 2026-07-15 baseline evidence only. They do not close the current M5 gate after later runtime, importer, task-pool, and shader changes.

- The viewer now passes the `CoreRuntime` TasksModule compute pool into parallel HDRI staging, so equirectangular projection, source mip generation, and PMREM filtering do not consume the asset I/O worker.
- The viewer now reuses `ProjectAssetManager::current_project_manager()` after `open_project`; it no longer reopens and rescans the same temporary project before loading the scene.
- Viewer teardown now drops the world, renderer, and runtime watcher before attempting to remove its temporary project directory, preserving repeated Windows launch/capture runs.
- The HDRI loader now emits `Written` or `Reused` together with staging and total elapsed time, which the current-source first/second bake evidence must record.
- The ready window title consumes that immutable loader report, keeping the cache outcome plus staging/total timings visible while orbiting or zooming; loading-time interaction cannot falsely promote the title to `Ready`.
- The viewer IBL artifact cache now lives under the stable system-temporary `zircon_shader_pbr_viewer_ibl_cache` directory rather than the disposable per-launch project root, so the second process can actually exercise the `Reused` path.
- `--ibl-cache-dir <directory>` overrides that stable default for a controlled cold/hot pair; the invoking validation workflow must provide a caller-owned directory outside the repository and Cargo target directories.
- Viewer exposure/layout inspection and parallel staging now share one decoded RGBA32F HDR image, avoiding a second full source decode before either `Written` or `Reused` staging completes.
- The direct viewer path accepts Radiance `.hdr` input only and rejects decoded images that are not non-empty 2:1 equirectangular maps before cubemap or PMREM staging.
- The viewer remains a background scene loader; the UI event loop stays responsible for loading progress, orbit, zoom, and presentation.
- Current-source managed dev/release builds, dated DX12 screenshot, and RenderDoc capture/replay are recorded in `06/2026-07-27-m5-current-source-pbr-and-viewer-validation.md`. That record remains subject to coordinator validation, independent review, and a managed milestone commit; this 2026-07-15 record remains historical baseline evidence.

## Scope Delivered

- 当前源码 viewer 使用真实 runtime module lifecycle 和版本化 manager access，消除此前可执行文件已清理后无法从当前源码可靠重建的交付漂移。
- 可执行文件保留在协调器管理的外部 Cargo target，不向仓库 `target` 或未登记 `E:\ZirconBuilds` 目录复制。
- 手动窗口支持左键 orbit、滚轮 zoom、命令行初始 yaw/pitch、独立 source/PMREM face size 和一次性 RenderDoc 捕获。
- 512 面分辨率产品截图和 RenderDoc 捕获使用真实 Poly Haven Lakes 2K HDRI，验证天空盒与镜面反射的实际清晰度。

## Fresh Testing Evidence

- Managed Windows job `1b86e228d43f4fad9edd9f0ef69d48df`: `validate-matrix.ps1 -Package zircon_app`; build/test 均 exit 0，viewer 测试二进制 18/18。随后复审收紧测试自搜索范围；production-anchor-removal 模拟会 RED。Managed job `03c5072d8e5b4214b146fd620027324d` 在该修正后的当前源码上重新通过 viewer 18/18，并同时通过 M3 artifact 23/23 与 source staging 6/6（1 ignored）。
- 当前 EXE 为 72,369,664 bytes，SHA256 `F8EEAD721B125E9D4CAEF374E9A532F07EDE0909B4DCC724B173F3508994233A`；`--help` exit 0。
- Lakes 2K DX12 source512/PMREM512：26/26 响应样本通过；source 与 PMREM 都是 512x512、10 mip；原生滚轮改变 34.4932% 窗口像素，左键拖动后 yaw 精确到 120。
- 截图 1296x999、1,184,470 bytes；DX12 capture 48,462,499 bytes；RenderDoc capture/replay 均 exit 0。
- 详细命令、哈希、响应采样、视觉结论和产物扫描见 `docs/tests/runtime/shader/zircon_shader_pbr_viewer_current_source_20260715_validation.md`。

## Review

独立 reviewer 已对精确七文件清单完成只读复审，结果为 Critical `0`、Important `0`、Minor `0`。复审独立重跑 corrected current-source viewer 18/18，核对 EXE/PNG/RDC 的尺寸与 SHA256，确认 `--help` 和 RDC replay 均退出 0，并确认仓库与受管 target 的精确产物扫描为 0。该结论仍需由协调器写入 milestone review 服务记录；本段不替代受管审查门禁。
