---
related_code:
  - zircon_runtime/src/graphics/types
tests:
  - graphics types current source 21 of 21 Rust files and 1911 lines reviewed
  - all 24 tests read; one source regression added
  - production output-target plans changed from heap diagnostics to compact status values
  - scoped rustfmt, source contract and diff check passed
  - current-source Cargo, scale counters, F2 camera targets and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics types静态审查（2026-07-18）

## 当前源覆盖

`graphics/types/**`当前21/21个Rust文件、1,911行已逐文件静态阅读，24条测试已读。覆盖viewport frame/snapshot/extract builder、camera stack attachment/output policy、render region、texture/headless/surface target、graph-import/writeback plan、GPU/frame texture handles及错误面。

## 直接止损

`ViewportTextureWritebackPlan`与`ViewportTextureGraphImportPlan`原在每次纹理相机状态判断中保存texture handle并构造三份诊断`String`。生产调用只读取`target_kind/status/size`，格式与texture getter全部为`#[cfg(test)]`，因此ready、conversion、pending和错误路径都为不被消费的堆分配。

现将texture与三项格式诊断字段连同初始化表达式条件编译到测试构建；生产计划只保留Copy状态与尺寸，不改变现有测试诊断和值语义。新增源码回归要求两个计划的四类诊断字段均只在测试构建存在。RED时八个字段计数均为0，GREEN时每类均为2；rustfmt与scoped diff门禁通过。

## 剩余根因与责任计划

`ViewportRenderFrame`同时持有legacy `RenderSceneSnapshot`和`Arc<RenderFrameExtract>`。正常`from_shared_extract`仍构造一份空scene DTO，而synthetic `from_snapshot`先深clone scene再拆成extract；`RuntimePrepareCollectorContext::scene_snapshot`继续暴露双owner。该契约不能在types局部删除，归PERF-MVP-413/414的generation-owned submission/source artifact，由Runtime07收敛消费者后硬切单一权威。

多camera提交为恢复下一camera状态仍clone `FrameVisibility`、post volumes、stack与graph；camera plan本身也在preflight/submit间重复投影。归PERF-MVP-417，由Render09发布唯一`CompiledCameraSubmissionPlan`，Runtime07按camera slot借用generation handles。公开`ViewportFrame`的RGBA owned clone与capture查询继续归PERF-MVP-023/413。

graph-import readiness、final-target selection与skip-writeback会重复构造小计划；字符串分配已清零，但writeback仍创建独立encoder并额外submit。计划/状态计算先以counter量化，GPU提交合并继续归PERF-MVP-404，由Render01把外部纹理import/copy/conversion编入主graph。

本地Bevy `dev/bevy/crates/bevy_render/src/camera.rs`以单一`SortedCameras` resource投影排序相机/target；UE `dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphBuilder.cpp`把external texture注册及copy/execute纳入RDG owner。采用“单一generation camera artifact、外部目标进入主graph、诊断载荷按需”的原则，不复制其ECS/RDG API。

## 验收状态

21/21静态阅读、一条RED→GREEN源码回归、rustfmt、源码合同与diff门禁完成。Windows Cargo validator仍在启动前`ConvertFrom-Json`失败，24条测试没有current-source执行结果。仍需1/8/64 cameras、post volumes 0/1k、visibility 0/64MiB、targets 0/1/8下记录plan builds、snapshot/extract/post/visibility/RGBA clone bytes、String alloc、encoder/submit、CPU p95/RSS；补F2 surface/texture/headless、camera stack/resize/history parity、timestamp与DX12 RenderDoc。完成前保留`pending.md`，不进入`review.md`。
