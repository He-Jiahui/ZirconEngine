# Runtime223 产品 journal 消费链事实复核

日期：2026-10-16。核心判断：gpt-6.1-sol max，NO_CONFIRMED_BUG（限定当前六个源码 owner）。本记录完成事实校正；RENDER223-P1-002 的完整功能、性能与原生验收继续 OPEN。

当前生产帧在 render_frame.rs:143/144 执行 admission 与 membership staging，在第 215 行通过 submission 校验后提交 journal。render_scene.rs:110 的 consumer plan 实际调用 apply_journal_membership；gpu_scene.rs:421–435 应用 retirements 与 resident writes。gpu_scene_sync.rs:120–123 仅在 journal 尚未接管成员关系的兼容路径清理完整 live set。原计划“没有产品 caller”的当前源码理由已过时，保留历史条目并追加本次更正。

同键、同实例数量的 register 在 gpu_scene.rs:332 复用；primitive 与 instance 更新分别在第 453、466 行比较 shadow 后标脏。逐 draw 的材质、变形、历史输出以及 consumer clone 仍可能有成本；本次没有性能测量可把这些成本判为缺陷，因此没有修改同步路径。

以下为本次限定读取的原始指纹；路径前缀为 zircon_runtime/src/graphics/scene/。全部无 CR，文件以 LF 结束。指纹只证明有限读取时的内容，不构成编译输入窗口稳定或原生通过证据。

| Owner | bytes | SHA-256 |
|---|---:|---|
| `renderer/core/render_scene.rs` | 7258 | `57b6dee47e324d5431f2c3e9b8ce6db02b2b8f340c781064963a590f3483c501` |
| `core/scene_renderer_render/render_frame.rs` | 11789 | `4412186e009e52fe5df113ca6dd28287e04f0d62743994e17f539ad12385d6ee` |
| `gpu_scene/journal_consumer.rs` | 22553 | `645edb580ea08ff39da54f8715ea23926f884fe347aabc885ec697fde7ab9086` |
| `gpu_scene/gpu_scene.rs` | 31861 | `407860c71f91e9073a9f22be8990860ea7e793111784002603d78a8e7918a075` |
| `renderer/mesh/build/gpu_scene_sync.rs` | 15647 | `b28ef30c487e640602e9a4270a6466b516af7d9f21b292114e6ead49bc213f79` |
| `renderer/mesh/build_mesh_draws/build/build.rs` | 33300 | `d90319380807285d4d7db5cdb853c7c36ecba85a8ea9b09e1ddbbbc3b89cda4c` |

已定位 renderer/core/tests/scene_admission.rs、gpu_scene/journal_consumer/tests/cases.rs、renderer/mesh/build/tests/gpu_scene_sync.rs 和 renderer/mesh/build/tests/perf.rs；未读取这些测试正文，未执行测试、benchmark、guard 或 CI。这些位置不是通过证据。

- [x] 确认生产 admission → staging → submission 校验 → journal commit 的调用事实。
- [x] 更正无 caller 与无条件完整 live-set 清理的旧理由。
- [ ] 验证完整增量语义及失败路径。
- [ ] 完成真实整帧性能矩阵与原生接受。

源码改动 0；构建、缓存、夹具、临时脚本和源码副本新增 0；释放 0 B。保留本事实记录及既有证据，不清理其他会话或未知归属产物。
