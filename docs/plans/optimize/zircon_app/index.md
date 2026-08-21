# `zircon_app` 差距审查

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

已完成产品 host/bootstrap/runtime-library/runtime-entry 首轮 E3 静态审查：117 个 production 文件、14,947 个物理行，覆盖 EntryConfig/module/plugin composition、runtime/editor runner、动态 DLL/session、Winit frame/window/surface owner、Play child report 和进程停机。详细 4 P0、27 P1、8 P2 见 [01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md](01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md)。

已完成 `zircon_shader_pbr_viewer` 与直接证据生产/消费链首轮 E3 静态审查：14 个 viewer production 文件、5,886 行，联读 7 个 evidence/profile/validator/receipt 工具与 624,277,393 bytes 历史 shader corpus。详细 4 P0、26 P1、8 P2 见 [02-pbr-viewer-tool-runtime-evidence-renderdoc-review.md](02-pbr-viewer-tool-runtime-evidence-renderdoc-review.md)。

已完成 `examples/woc` 多应用/多crate产品集成首轮E3审查：2,416个物理文件、98,353,987 bytes，覆盖四个role binary、8-crate native workspace、817个Zr module、ZrVM transaction/state identity、Client/Server、projection、assets与parity。四个binary当前都只打印identity后退出；clean clone缺29个入口可达module，native workspace有6个compile error，WOS83/113/118/117 authority互相矛盾，transaction接口又不能证明VM内部回滚。详细9 P0、66 P1、14 P2见[03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md](03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md)。

已完成WOC native client的61个production Rust文件逐文件E3审查：11,635行/361,156 bytes，联读47个测试文件/355个test，覆盖binary reachability、window/input、shell/auth、preferences、presentation与inventory/quest/HUD。binary不引用自己的library，manifest无window/GPU/UI/audio/network/async backend；`WocClientSession`也不组合windows/settings/device service。frame driver在timeline失败后仍消费命令并推进movement sequence，auth effect又允许Debug输出password/token/2FA。详细5 P0、88 P1、16 P2见[04-woc-native-client-window-input-shell-ui-presentation-frame-product-integration-review.md](04-woc-native-client-window-input-shell-ui-presentation-frame-product-integration-review.md)。

已完成WOC native server、bot与headless逐文件E3审查：三个manifest、5个production Rust文件/286行和唯一server test文件/6个test。三个binary仍只打印identity；server binary不引用同package library，driver也没有production caller。driver在VM提交前破坏性出队，fault batch没有可恢复journal并可被下一次空advance覆盖；queue又没有principal/session/world绑定、aggregate bytes/work budget或atomic outcome/replication/durability代际。详细4 P0、72 P1、16 P2见[05-woc-native-server-bot-headless-service-tick-replication-persistence-operations-product-integration-review.md](05-woc-native-server-bot-headless-service-tick-replication-persistence-operations-product-integration-review.md)。

已完成`examples/vampire`产品样例首轮E3审查：173个tracked文件/8,880,831 bytes，覆盖110个scene entity、52份`.zmeta`、24份GLB、ZrVM脚本/工件、玩法数据、34张PNG及asset/session测试。clean clone缺7份被ignore的模型源，required provider只在手工feature和test-only importer下闭合；全部玩法又集中在variable `onUpdate`，balance/behavior tree没有consumer。10个real-VM测试全部ignore，主import test与当前WGSL静态漂移，README声明的两张accepted图不存在且性能口径跨代冲突。详细5 P0、80 P1、16 P2见[06-vampire-roguelite-example-project-asset-script-gameplay-evidence-product-integration-review.md](06-vampire-roguelite-example-project-asset-script-gameplay-evidence-product-integration-review.md)。

已完成`templates/projects/renderable-empty`及其创建、导入、渲染、导出证据链首轮E3审查：17个模板文件/4,867 bytes逐项内嵌，联读Runtime Interface pack、Editor/Hub创建owner、101个非reference consumer与28个直接测试文件。模板ID没有version/content digest/engine compatibility，生成项目不记录provenance，也不声明provider/BuildSet；Editor与Hub共享bytes却复制不同事务且对“创建成功”定义不同。默认Windows release export又在Tooling03资格未通过时暴露。详细4 P0、72 P1、16 P2见[07-renderable-empty-project-template-create-import-render-export-evidence-product-integration-review.md](07-renderable-empty-project-template-create-import-render-export-evidence-product-integration-review.md)。

本分类现有7篇报告，合计35 P0、431 P1、94 P2。本轮确认 `target-server` 没有 binary 且 `run_headless()` 立即返回；Web/Android feature 无法从 required `target-client` 的 desktop/DLL bundle 形成独立产品；process-wide shutdown coordinator、多window/viewport/surface generation 和 typed Play child handshake 均未闭合。PBR 自动截图与 RenderDoc 又固定走 offscreen CPU readback、绕过 product host/native swapchain；ready validator 只检查非空颜色，运行期致命错误仍可能返回退出码 0。WOC进一步证明项目manifest、角色名、协议DTO、大量pure-model test和symbolic host effect不能替代真实ProductHost、VM adapter、network authority、transaction、native present、recoverable server commit与产品evidence。Vampire进一步证明开发机ignored source/cache、手工feature、test-only importer、跨代截图与按帧脚本不能替代clean-clone可构建、玩法权威和source-bound产品资格。Renderable Empty模板则证明embedded bytes、局部transaction和offscreen像素测试不能替代versioned template product、跨入口一致的Project Ready与可发布export资格。已有 DLL owner、GPU timing generation、RenderDoc replay、artifact hash、IBL 冷暖矩阵、staged project publish，以及WOC typed protocol/projection/golden、input、shell和fixed-tick局部状态基础应保留。

后续队列：

1. 继续审查其余example、顶层acceptance/fixture与未映射物理域，不重复App03-App07 finding。
2. 实施前与 `zircon_runtime_interface` 复核 ABI/FFI/handle/version/foreign ownership 全域。
3. 与 Plugins/Editor/Hub 复核 packaged runtime discovery、Play process、安装启动和跨进程停机。
4. 在 Tooling 实施轮次复核 evidence schema、CI、artifact store、retention 与同画质 paired benchmark。
