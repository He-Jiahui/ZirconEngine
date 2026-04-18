use super::super::render_thread::RenderThreadCommand;
use super::render_service::RenderService;

impl Drop for RenderService {
    fn drop(&mut self) {
        let _ = self.command_tx.send(RenderThreadCommand::Shutdown);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}
