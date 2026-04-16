use super::{render_service::RenderService, render_thread_command::RenderThreadCommand};

impl Drop for RenderService {
    fn drop(&mut self) {
        let _ = self.command_tx.send(RenderThreadCommand::Shutdown);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}
