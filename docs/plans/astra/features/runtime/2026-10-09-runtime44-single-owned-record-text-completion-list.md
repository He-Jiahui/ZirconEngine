# Runtime44 · 单份 owned 日志文本完成列表

日期：2026-10-09。对应[优化记录](../../../optimize/zircon_runtime/44/2026-10-09-single-owned-record-text.md)。

## 已完成源码工作

- [x] 私有 record 以一份预留容量的 owned UTF-8 文本保存 scope/message，编码借用合法字节边界切片。
- [x] 保留公开 API、lazy preflight、owned 生命周期、enqueue 时间戳顺序、原字节估算、FIFO/flush/sync/metrics。
- [x] 仅适配既有测试 helper；两项用例正文与小写期望保持，没有新增镜像测试。
- [x] 独审接受后实际一次接入两文件，17 个 raw 守卫、实际后像、apply/fmt/diff 通过，共享 index/control 未变。
- [x] 复用原 Runtime #2 / Release #9 入口，维持 15 项批次和原 54 矩阵行、20 critical 样本及预算。

实际回执 SHA-256：`46ed74cf5ab75e412edab481a7576ff36604e11186c3e2f8c5ecc03dd296f249`。以上是源码完成项，未执行 native。

## 待实际验收

- [ ] 批量原生执行现有 Unicode、多记录、阈值/FIFO/durability/buffer/worker 生命周期用例，核真实名称与非零数量。
- [ ] 同来源同负载 before/after allocation requests/bytes、producer/worker CPU、完整 p50/p95/p99/max 达到原标准。
- [ ] 原 54 行风暴矩阵与 20 critical 样本完整成功；真实 NVMe/慢盘/console 与 conservation/drop/RSS 等原门通过。
- [ ] 完整 arena/callsite/producer clock 与 record/queue 硬字节 admission。
- [ ] G35/G36、真实产品 F0、milestone 及全 workspace 接受。

源码层减少一次独立 owned allocation 的机会，不代表实际总分配或性能已达标。
