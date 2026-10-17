use super::*;
impl BrowserEngine {
    pub async fn open(
        canvas: web_sys::HtmlCanvasElement,
        identity: u64,
        asset_root: String,
        animation_module: ModuleDescriptor,
        extensions: WorldRuntimeExtensionPlan,
    ) -> Result<Self, String> {
        if identity == 0 {
            return Err("canvas identity must be nonzero".into());
        }
        let core = CoreRuntime::try_new().map_err(error)?;
        let input = Arc::new(BrowserInput::default());
        for module in [
            TasksModule.descriptor(),
            TimeModule.descriptor(),
            FrameCountModule.descriptor(),
            crate::foundation::module_descriptor(),
            platform_module(Arc::clone(&input)),
        ] {
            core.register_module(module).map_err(error)?;
        }
        core.activate_registered_modules().map_err(error)?;
        let source =
            BrowserAssetSource::open(&core.handle(), asset_root, BrowserAssetLimits::default())
                .await
                .map_err(error)?;
        let bootstrap: Arc<Mutex<Option<Arc<dyn RenderFramework>>>> = Arc::new(Mutex::new(None));
        let mut graphics = crate::graphics::graphics_module_descriptor();
        let manager = graphics
            .managers
            .iter_mut()
            .find(|manager| manager.name.as_str() == RENDER_FRAMEWORK_NAME)
            .ok_or_else(|| "graphics descriptor has no render framework".to_string())?;
        manager.factory = factory({
            let bootstrap = Arc::clone(&bootstrap);
            move |_| {
                let renderer = bootstrap
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .take()
                    .ok_or_else(|| {
                        CoreError::Initialization(
                            RENDER_FRAMEWORK_NAME.into(),
                            "async Canvas bootstrap has not completed".into(),
                        )
                    })?;
                Ok(
                    Arc::new(RegisteredManagerService::<dyn RenderFramework>::new(
                        renderer,
                    )) as ServiceObject,
                )
            }
        });
        for module in [
            crate::asset::runtime::module_descriptor(source.manager()),
            crate::text::TextModule::default().descriptor(),
            crate::input::module_descriptor(),
            crate::scene::module_descriptor(),
            animation_module,
            graphics,
            crate::ui::module_descriptor(),
        ] {
            core.register_module(module).map_err(error)?;
        }
        core.activate_registered_modules().map_err(error)?;
        // Preload only the desktop-selected startup dependency closure.
        for id in source.startup_asset_ids() {
            source.preload(id).await.map_err(error)?;
        }
        let scene = source.load_scene().await.map_err(error)?;
        let mut world = World::empty();
        scene.spawn_into(&mut world).map_err(error)?;
        drop(scene);
        crate::scene::install_world_runtime_extension_plan(&core.handle(), extensions)
            .map_err(error)?;
        let level = crate::scene::create_level(
            &core.handle(),
            world,
            LevelMetadata {
                display_name: Some("Browser startup scene".into()),
                ..Default::default()
            },
        )
        .map_err(error)?;
        let ui = {
            let text = crate::text::text_runtime_context_for_core(&core.handle()).map_err(error)?;
            let handle = core.handle();
            let assets = resolve_manager_service(
                &handle,
                runtime_asset_manager_handle(&handle).map_err(error)?,
            )
            .map_err(error)?;
            ui::BrowserUi::load(&source, &text, &assets)?
        };
        let fonts = {
            let text = crate::text::text_runtime_context_for_core(&core.handle()).map_err(error)?;
            text.font_collection()
        };
        let assets = RuntimeAssetManagerAccess::new(
            core.handle(),
            runtime_asset_manager_handle(&core.handle()).map_err(error)?,
        );
        let renderer = Arc::new(
            WgpuRenderFramework::new_canvas(
                assets,
                canvas.clone(),
                identity,
                core.task_graph().worker_pool().clone(),
                fonts,
            )
            .await
            .map_err(error)?,
        );
        renderer
            .set_submission_config(RenderSubmissionConfig::default())
            .map_err(error)?;
        *bootstrap.lock().unwrap_or_else(|e| e.into_inner()) =
            Some(Arc::clone(&renderer) as Arc<dyn RenderFramework>);
        {
            // Publish the async-created instance through the canonical service.
            let handle = core.handle();
            let registered =
                resolve_manager_service(&handle, render_framework_handle(&handle).map_err(error)?)
                    .map_err(error)?;
            drop(registered);
        }
        let size = UVec2::new(canvas.width(), canvas.height());
        if size.x > 4096 || size.y > 4096 {
            return Err("browser viewport exceeds the 4096 pixel dimension limit".into());
        }
        let viewport = renderer
            .create_viewport(RenderViewportDescriptor::new(size.max(UVec2::ONE)))
            .map_err(error)?;
        // From here Drop owns cleanup for a failed surface bind as well.
        LIVE_ENGINES.fetch_add(1, Ordering::AcqRel);
        let engine = Self {
            core: Some(core),
            source,
            renderer: Some(renderer),
            level: Some(level),
            ui: Some(ui),
            viewport: Some(viewport),
            canvas,
            identity,
            input,
            frame_events: Vec::new(),
            last_timestamp: None,
            lifecycle_generation: 0,
            frame: 0,
            size,
            closing: false,
        };
        if size.x != 0 && size.y != 0 {
            engine.bind_surface(size)?;
        }
        Ok(engine)
    }
}
