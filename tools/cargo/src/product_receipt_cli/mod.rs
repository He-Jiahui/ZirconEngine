//! 产品收据命令入口与内部参数边界。
//! 主程序把收据子命令路由到此模块；输入和选项模块不成为库的公共 API。

mod input;
mod options;
mod run;

pub(crate) use run::run;
