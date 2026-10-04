use std::fs;
use std::io;
use std::path::Path;

use crate::core::framework::script::ScriptHostModuleDescriptor;

use super::{render_script_host_modules_markdown, ScriptHostInterfaceMarkdownOptions};

/// CLI 用此入口把描述符投影落盘；调用方负责选择输出路径，写入会创建父目录并传播 I/O 错误。
pub fn write_script_host_modules_markdown(
    path: impl AsRef<Path>,
    modules: &[ScriptHostModuleDescriptor],
    options: &ScriptHostInterfaceMarkdownOptions,
) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, render_script_host_modules_markdown(modules, options))
}
