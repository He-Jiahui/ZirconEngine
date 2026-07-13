---
handoff_kind: fixed
status: fixed
created_at: 2026-07-11
summary_slug: wgpu-hal-windows-version-split
origin_plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
fixing_plan: docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
origin_child_dir: docs/plans/zircon_editor/editor/11
fixing_child_dir: docs/plans/zircon_runtime/runtime/01
related_code:
  - Cargo.lock
  - zircon_runtime/Cargo.toml
plan_sources:
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo build -p zircon_runtime --locked
resolved_at: 2026-07-12
---


# Runtime 01：wgpu-hal Windows 类型版本分裂失败

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/11-serialization-and-versioning.md`
- 来源执行切片：Plan11 M1.2 场景反射版本壳与 canonical writer 测试阶段
- 修复责任计划：`docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md`
- 交接原因：Runtime 生产构建在编译 `wgpu-hal` 时失败，尚未进入 Zircon 场景代码，不能把依赖图失配记为 Plan11 行为失败。

## 失败现象与复现证据

2026-07-11 受管命令 `.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipTest` 使用 coordinator lane `D:\targets\zircon-engine\lanes\check-b480c5afac0047a78cd970581a98cbdd`，在 `wgpu-hal 29.0.4` 的 DX12 子分配实现编译失败：

- `wgpu-hal-29.0.4/src/dx12/suballocation.rs:83`：E0308，`gpu_allocator::d3d12::ID3D12DeviceVersion::Device` 需要 `windows 0.54` 的 `ID3D12Device`，实际传入 `windows 0.62` 的同名类型。
- `suballocation.rs:299/306、338/345、377/384`：E0277，`ResourceCategory: From<&D3D12_RESOURCE_DESC>` 与 `Param<ID3D12Heap>` 均因 `windows-core 0.54/0.62` 类型及 trait 身份不同而不成立。
- 当前 `Cargo.lock` 同时把 `gpu-allocator 0.28` 的 Windows 依赖收敛到 `windows 0.54.0`，而 `wgpu-hal 29.0.4` 的 DX12 面使用 `windows 0.62.2`；错误发生在注册表依赖源码，不在 `zircon_runtime/src/scene`。

## 最低共享层根因

最低失败层是 Runtime 01 管辖的图形依赖版本收敛：`wgpu 29.0.4`、`gpu-allocator 0.28` 与 `windows/windows-core` 形成两个不兼容的 COM 类型世界。Rust 同名类型跨 crate 版本不相等，所以上层场景、编辑器或测试代码无法修复该编译失败。

## 架构修复验收

- 在单一、可解释的依赖组合中统一 `wgpu-hal` 与 `gpu-allocator` 的 Windows 类型版本；修复必须落在 manifest/lock/依赖选型，不得在 Zircon 业务代码中转指针或复制 COM wrapper。
- 重新生成并审计 `Cargo.lock`，确认不再由该组合同时引入不兼容的 `windows 0.54/0.62` DX12 类型路径。
- 受管复跑 `cargo build -p zircon_runtime --locked`，随后通知 Plan11 owner 复跑 Runtime 场景门禁。
- 若选择升级或降级 `wgpu`/`gpu-allocator`，同步 Runtime 01 的技术栈基线与依赖守卫，禁止只修改 lockfile 漂移版本。

## 禁止临时方案

- 禁止在场景、渲染调用方或编辑器层添加类型转换 shim、裸指针桥接或条件编译绕过 DX12。
- 禁止跳过 Windows backend 来冒充 Runtime 全平台构建通过。
- 禁止把未执行到 Zircon 源码的依赖编译失败记为 Plan11 功能失败。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Runtime 01 | Windows 图形依赖版本收敛 | `未通过-待-runtime01-owner-修复` | 2026-07-11 | 受管 `zircon_runtime -SkipTest` 在 `wgpu-hal 29.0.4` 报 E0308×1、E0277×9；错误显示 `windows 0.54` 与 `0.62` 的 DX12 类型/trait 不相容。 |
| Runtime 01 | WGPU 29.0.3 单补丁线复验 | `实现完成-生产构建超时-仍开放` | 2026-07-11 | 当前 `Cargo.lock` 的 `wgpu`/`wgpu-core`/Windows-Linux-Android deps/`wgpu-hal`/naga bridge/`wgpu-types` 均为 29.0.3,`gpu-allocator 0.28.0` 使用 `windows 0.62.2`,`cpal` 独立保留 `windows 0.54.0`;locked/offline metadata 与 Editor `cargo check --tests` 已通过,Editor lib-test 亦完成链接并执行。但受管 `cargo build -p zircon_runtime --locked --offline --jobs 1` 在 904.1s 外层上限时仍活跃编译 Runtime 本体,全程未重现 wgpu-hal COM 类型错误但未取得 exit 0,故不得关闭本 failure。owned 3 个进程已停止且 coordinator job 以 124 结束并释放。 |

## 修复结果与回传

- 根因：wgpu-hal 29.0.4 and gpu-allocator 0.28 resolved incompatible Windows COM crate versions; the dependency graph required a single compatible wgpu 29.0.3 patch line with gpu-allocator on windows 0.62.
- 架构修复：Cargo.lock and Runtime01 dependency governance converge wgpu, wgpu-core, wgpu-hal and wgpu-types on 29.0.3 while the Runtime01 static guard prevents patch-line drift; no business-layer COM shim was added.
- 验证：Fresh current-source zircon_runtime lib-test build completed successfully on the coordinator-managed D drive; the resulting binary passed structure_convention 1304/1304, code_review_findings 298/298, tech_stack 15/15 and focused rich-link input regressions 4/4 without reproducing any wgpu-hal Windows type error.
- 回传：Runtime production compilation is restored on the unified WGPU 29.0.3 dependency line; Editor11 may resume its scene serialization gate.
