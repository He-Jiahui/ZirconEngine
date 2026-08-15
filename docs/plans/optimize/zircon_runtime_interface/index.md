# `zircon_runtime_interface` 差距审查

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

待审范围：Rust/C ABI、结构版本、capability negotiation、句柄 generation、字符串/缓冲区所有权、callback/threading、panic 隔离、错误分类和向前/向后兼容。必须与 `zircon_app` 动态 runtime library 和 `zircon_runtime` session owner 纵向联审。

