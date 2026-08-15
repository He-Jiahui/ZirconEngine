# `zircon_app` 差距审查

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

待审范围：entry/profile 组装、runtime/editor 产品循环、动态 runtime library、frame ownership、window/surface bridge、崩溃与正常停机、配置和产品模式。当前仅为支撑 Runtime01 而读取 bootstrap 与两条 teardown 路径，不构成 host 全域审查。

优先队列：统一 shutdown coordinator、runtime library 卸载安全、entry/run-loop 失败传播、多实例/多窗口和进程级诊断收尾。

