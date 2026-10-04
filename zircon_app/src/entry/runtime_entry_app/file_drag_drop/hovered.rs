//! 文件拖拽路径的文本化 Runtime 通知；文本只用于同步事件调用。
//! 非 UTF-8 文件名经过 lossy 转换，消费端不得假定能无损还原原始路径。

use std::path::PathBuf;

use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::super::{converters::byte_slice, RuntimeEntryApp};

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_files_hovered(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        paths: Vec<PathBuf>,
    ) {
        for path in paths {
            let path_text = path.to_string_lossy();
            let event = ZrRuntimeEventV1::file_hovered(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                self.viewport,
                byte_slice(path_text.as_ref()),
            );
            if !self.dispatch_runtime_event(event_loop, event) {
                return;
            }
        }
    }
}
