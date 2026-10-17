use super::*;
impl BrowserEngine {
    fn begin_close(&mut self) -> Result<(), String> {
        self.closing = true;
        self.input.close();
        self.source.close();
        self.frame_events.clear();
        self.ui.take();
        self.level.take();
        if let (Some(renderer), Some(viewport)) = (self.renderer.as_ref(), self.viewport.take()) {
            renderer.destroy_viewport(viewport).map_err(error)?;
        }
        self.renderer.take();
        Ok(())
    }
    pub async fn close(&mut self) -> Result<BrowserCloseReceipt, String> {
        let cleanup_error = self.begin_close().err();
        let deadline = web_time::Instant::now() + Duration::from_secs(5);
        loop {
            let Some(core) = self.core.as_ref() else {
                return Ok(BrowserCloseReceipt {
                    modules_closed: true,
                    task_graph_joined: true,
                    pending_assets: 0,
                });
            };
            let modules = core.shutdown_registered_modules_with_drain_timeout(Duration::ZERO);
            let tasks = core.shutdown_task_graph(Duration::ZERO);
            if modules.is_ok() && tasks.is_ok() && self.source.pending_requests() == 0 {
                self.core.take();
                return cleanup_error.map_or_else(
                    || {
                        Ok(BrowserCloseReceipt {
                            modules_closed: true,
                            task_graph_joined: true,
                            pending_assets: 0,
                        })
                    },
                    Err,
                );
            }
            if web_time::Instant::now() >= deadline {
                return Err(format!("browser close remains incomplete: modules={modules:?}; tasks={tasks:?}; asset_tasks={}",self.source.pending_requests()));
            }
            turn::next_turn().await?;
        }
    }
}
impl Drop for BrowserEngine {
    fn drop(&mut self) {
        let _ = self.begin_close();
        if let Some(core) = self.core.take() {
            let _ = core.shutdown_registered_modules_with_drain_timeout(Duration::ZERO);
            let _ = core.shutdown_task_graph(Duration::ZERO);
        }
        LIVE_ENGINES.fetch_sub(1, Ordering::AcqRel);
    }
}
