# VG 的既有 GPU indexed adapter 用例在发出 adapter 之前失败

**范围：单独的测试夹具问题。** 不属于 [#228](https://github.com/He-Jiahui/ZirconEngine/pull/228) 的正式 Level / 自动 VG 交接，也不是 [#229](https://github.com/He-Jiahui/ZirconEngine/pull/229) 的 Model/Mesh UUID。此 Draft 只记录问题；不改生产编码、ECS、接口、WGSL、CI 或合并状态。

**实际观察：** R80 直接重放保全的 Windows x64 plugin libtest PE（生产源证据 `e27e84effc10285af2dacec6f5c8432020d2e9ad`，PE SHA256 `cf455e1bdc6d08382782205e6b994e96049f1b8d9998f97d8614e6c9c7df84`），精确选择 `render_pass_executors::indirect::indexed::tests::indexed_adapter_consumes_gpu_authority_and_preserves_run_identity`。单个用例真实退出 **101，0/1**，唯一失败 `stable_key.rs:14:5: entity id exceeds stable instance key packing range`。在 Rust author run 构造时 panic，尚未 dispatch/submit adapter，也没有读取任一五字命令。R80 两个独立控制（CPU 终态校验、只走 debug 颜色变体的 clip vertex-pull 实绘）另行通过，不能给这个 adapter 用例补记成功。选择器查找含 `RTX 3060` 的适配器，但没有记录它实际走哪个图形后端，不宣称此用例是 DX12 产品验收。

## 所有权和两个独立前置条件

1. **已执行失败**：fixture [identity](../../../zircon_plugins/virtual_geometry/runtime/src/executors/indexed_tests.rs#L62-L75) 写 `entity=0x8765_4321_0000_0000+row` 并配任意 `stable_instance_key`。生产 [packed stable key](../../../zircon_runtime/src/core/framework/render/scene_extract/mesh/stable_key.rs#L3-L18) 预留 16 位 primitive，48 位 entity 最大 `0x0000_ffff_ffff_ffff`。[author run admission](../../../zircon_plugins/virtual_geometry/runtime/src/executors/indexed.rs#L129-L149) 在本路径即使 `instance_index=Some` 也会计算 legacy key；从一开始就不应把超域值作为合法 author identity。保留生产的范围断言，不截断实体或拓宽 ABI。
2. **独立读源发现，未执行到**：fixture [覆盖第五个 run 的 stable key](../../../zircon_plugins/virtual_geometry/runtime/src/executors/indexed_tests.rs#L236-L264)，但 `#[cfg(test)]` 构造器[把输入身份当 canonical](../../../zircon_plugins/virtual_geometry/runtime/src/executors/indexed.rs#L90-L113)，而真实 [encode](../../../zircon_plugins/virtual_geometry/runtime/src/executors/indexed.rs#L265-L300) 要它逐值等于相应 source-row 的 canonical。只修 48 位范围仍会先得到 CPU `indexed run canonical identity differs from its source row`，不是当前测试后半段宣称的 GPU `status[4]=2`。WGSL 独立验证 [producer 14-word authority](../../../zircon_plugins/virtual_geometry/runtime/src/executors/indexed_args.wgsl#L33-L68)，它没有来源 stable-key 字段供这个负例替代该检查。

## 未来验证条件（未改动）

- 测试仍用原生产 producer/adapter WGSL 及原 Rust `encode`。合法输入由同一个受检查的 entity packer 生成 stable key，实体保留非零高 32 位和不同 primitive。重复源行的 disjoint spans、有效负 `base_vertex`、实际 GPUScene `first_instance`、不可见行、padding tail、读回 sidecar 与 authority 原值均保留，不通过重复复制 GPU 结果当预期。
- 分开两项负对照：真正的错 canonical key 应在 **CPU** 被拒，错误原文可辨；另构造 CPU 合法、但 GPU producer authority 的 candidate/cluster 字段与本 run 不符的测试，在真实 dispatch 后读回 **`status=2` 且 Args5 全零**。不把源码选择替换成 raw buffer 绕过 CPU owner 或删弱断言。
- 先冻结同一生产的测量合同、公开实际旧执行达到哪层，再做至多这个 fixture 的改动；同一 windows native GPU selector 真正提交并映射 GPU buffer 后才谈修复。确切图形 backend 要程序自己报告或明确锁定，不能用启动环境字符串当证据。stock #228 的同一提交深度/五字参数仍须独立测量。
