//! 编译选项模块把默认值与链式修改方法分开，供管线编译入口构造稳定的编译契约。
#[path = "default.rs"]
mod default;
#[path = "methods.rs"]
mod methods;
