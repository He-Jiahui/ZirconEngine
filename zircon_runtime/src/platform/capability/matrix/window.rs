//! 窗口相关能力按后端、光标/指针、拖放/输入法和生命周期拆分；各子模块都
//! 在排除 headless 等直接返回路径后，通过 window_backend 传播窗口开关或不可用原因。
mod backend;
mod cursor;
mod drag_drop;
mod ime;
mod lifecycle;
