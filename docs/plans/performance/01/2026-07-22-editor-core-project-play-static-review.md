---
related_code:
  - zircon_editor/src/core/project
  - zircon_editor/src/core/play
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/godot/editor/editor_node.cpp
  - dev/godot/editor/editor_undo_redo_manager.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorPlaySettings.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/PlayLevel.h
  - dev/bevy/crates/bevy_tasks/src/usages.rs
tests:
  - zircon_editor/src/core/project/authority.rs::performance_source_guards
  - zircon_editor/src/core/play/controller.rs::performance_source_guards
  - zircon_editor/src/core/play/plugin_activation/native.rs::performance_source_guards
  - zircon_editor/src/core/play/process_backend/mod.rs::performance_source_guards
  - zircon_editor/src/core/play/process_backend/output.rs::performance_source_guards
  - current-source Windows Cargo and project/play product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor core project与play逐文件性能静态审查（2026-07-22）

## 范围与覆盖

已逐文件阅读`zircon_editor/src/core/project`生产13/13、`play`生产31/31，共 **44/44生产文件**；外部project/play tests 10个文件只作为既有合同索引，尚未逐文件验收。`zircon_editor/src/core`累计完成 **188/257** 文件静态阅读，剩余69个含外部tests继续留在`pending.md`。

受管Cargo lane仍由其他Session预约，没有运行raw Cargo。Play路径会启动runtime preview，但当前未取得可运行产品session；未伪造进程trace或RenderDoc capture。

## 已确认的性能形状

- ProjectAuthority的canonical root已逐ancestor拒绝link/reparse并canonicalize，随后open/probe/recent validation过去又重复同一ancestor metadata walk。本轮把post-canonical structural validation拆出并复用，保留manifest/file错误语义。更大的PERF-MVP-075/100仍是ProjectAuthority、Runtime AssetManager与EditorProjectDocument重复open/scan，以及welcome/probe稳定轮询文件系统。
- project template creation在调用线程同步create directories、逐entry write、manifest load+canonical save和directory rename；每entry还重复`create_dir_all(parent)`。这是低频显式操作，先归Runtime11有界I/O job与进度，不以无依据并发直接改写原子commit顺序。
- `PlaySceneSource::from_world`同步执行完整World→DynamicScene→pretty JSON，snapshot store随后在调用线程创建目录、写整文档、`sync_all`、rename；Process backend的`start`还持active mutex跨materialize与process spawn。大场景进入Play会直接阻塞主线程并产生完整scene+JSON+文件多份峰值，登记PERF-MVP-550。
- PendingEditQueue是无上限`VecDeque<EditorOperationInvocation>`；Play期间任意允许排队的workspace/document edits可持续常驻宽JSON参数，snapshot又深clone全部intent。登记PERF-MVP-551，必须按operation/target语义定义lossless/latest/bounded，而不是统一静默截断。
- Process output queue虽有1024-line entry cap，但reader使用`read_until('\n')`，单行bytes无上限；每tick poll原全量drain并format最多1024行。此次把live drain收为64 lines/poll，terminal仍完整drain；最终还需max-line/max-bytes/oldest-age和Runtime11 blocking-I/O owner。两条reader是每play session手建threads，回链Editor14。
- terminal poll过去持active mutex跨reader join与snapshot directory cleanup，本轮在finish前释放；inactive poll也只读一次mode。Controller自己的transition gate仍跨plugin activate/deactivate及backend start/stop/poll foreign work，慢I/O/callback会阻塞stop/edit route和其他状态查询，登记PERF-MVP-553。
- NativePluginBridgeActivation deactivate过去先clone完整play-mode snapshot再restore；本轮先move snapshot，失败才放回，成功路径full blob clone=0，补充PERF-MVP-540。

## 参考引擎核对

- Unreal PIE把play settings、launch request和process/session lifecycle分离；Zircon应把snapshot build/materialize/spawn作为有界job ticket并在安全点提交，而不是在controller transition mutex内串行执行全部阶段。
- Godot editor把scene/play状态切换集中在EditorNode生命周期，但资源扫描与导入有独立filesystem/cache owner；Zircon保留单一transition authority，同时把大I/O/serialization移出主线程并用generation验证commit。
- Bevy按AsyncCompute/Io池区分可跨帧CPU与阻塞I/O；Zircon使用Runtime11统一预算，不能为每次Play无限新增私有worker或把stdout/stderr无上限行缓冲留在专线程。

## 本轮直接止损与动态验收

直接止损：canonical project path不重复ancestor metadata walk；inactive backend poll单mode read；native play snapshot成功deactivate零深clone；terminal process finish释放active锁；live output每poll最多64行。源码守卫、scoped rustfmt与diff check通过；Cargo/产品trace待办。

以scene **1/64MiB/1GiB**、pending edits **1/1k/100k**、output lines **1/1k/1M**、line bytes **64B/1MiB/1GiB**、poll 30/60/120Hz测试。记录World/DynamicScene/JSON owners、serialize/write/fsync/main-thread wall、active/transition lock wait、queue entries/bytes/oldest age、drained/formatted lines、reader threads与RSS；要求snapshot主线程serialization/I/O=0、队列bytes硬有界、poll obey count+time budget、foreign callback不在controller mutex内、stop/crash/rollback/cleanup无泄漏。F0/F4 project open/create、Play start/stop/crash、pending apply/discard、plugin lifecycle、output UTF-8与RenderDoc首帧通过后方可进入`review.md`。
