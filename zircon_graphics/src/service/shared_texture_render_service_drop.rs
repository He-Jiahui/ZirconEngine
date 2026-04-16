use super::{
    render_thread_command::RenderThreadCommand,
    shared_texture_render_service::SharedTextureRenderService,
};

impl Drop for SharedTextureRenderService {
    fn drop(&mut self) {
        let _ = self.command_tx.send(RenderThreadCommand::Shutdown);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}
