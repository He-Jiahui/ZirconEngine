use std::io::{BufRead, Write};
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId, WindowLevel};

use super::{content::startup_text, paint, EditorStartupStage};

#[derive(Default)]
struct SplashState {
    stage: EditorStartupStage,
    closed: bool,
}

impl SplashState {
    fn receive(&mut self, message: &str) {
        if matches!(message, "close" | "ready") {
            self.closed = true;
        }
        if let Some(stage) = EditorStartupStage::parse(message) {
            self.stage = self.stage.max(stage);
        }
    }
}

struct SplashApp {
    state: Arc<Mutex<SplashState>>,
    window: Option<Arc<dyn Window>>,
    // The surface is dropped before its context/window on every exit path.
    surface: Option<Surface<Arc<dyn Window>, Arc<dyn Window>>>,
    _context: Option<Context<Arc<dyn Window>>>,
    project_requested: bool,
    visible: bool,
    started: Instant,
    next_tick: Instant,
    fonts: Arc<resvg::usvg::fontdb::Database>,
}

pub(super) fn run(project_requested: bool) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let proxy = event_loop.create_proxy();
    let state = Arc::new(Mutex::new(SplashState::default()));
    let input_state = state.clone();
    std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines().map_while(Result::ok) {
            let mut state = input_state
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            state.receive(&line);
            let closed = state.closed;
            drop(state);
            proxy.wake_up();
            if closed {
                return;
            }
        }
        // Losing the parent's pipe is terminal, including parent crash/forced exit.
        input_state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .closed = true;
        proxy.wake_up();
    });
    let mut fonts = resvg::usvg::fontdb::Database::new();
    fonts.load_font_data(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../zircon_runtime/assets/fonts/FiraSans-Regular.ttf"
        ))
        .to_vec(),
    );
    let now = Instant::now();
    let result = event_loop.run_app(SplashApp {
        state,
        window: None,
        surface: None,
        _context: None,
        project_requested,
        visible: false,
        started: now,
        next_tick: now,
        fonts: Arc::new(fonts),
    });
    println!("closed");
    let _ = std::io::stdout().flush();
    result.map_err(Into::into)
}

impl SplashApp {
    fn refresh_or_exit(&self, event_loop: &dyn ActiveEventLoop) {
        if self
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .closed
        {
            event_loop.exit();
        } else if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn present(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let Some(window) = self.window.as_ref() else {
            return Ok(());
        };
        let size = window.surface_size();
        let (Some(width), Some(height)) =
            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            return Ok(());
        };
        let stage = self
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .stage;
        let pixels = paint::render(
            size.width,
            size.height,
            window.scale_factor() as f32,
            stage,
            self.project_requested,
            self.started.elapsed(),
            self.fonts.clone(),
        )?;
        let Some(surface) = self.surface.as_mut() else {
            return Ok(());
        };
        surface.resize(width, height)?;
        let mut buffer = surface.buffer_mut()?;
        for (output, rgba) in buffer.iter_mut().zip(pixels.chunks_exact(4)) {
            *output = ((rgba[0] as u32) << 16) | ((rgba[1] as u32) << 8) | rgba[2] as u32;
        }
        window.pre_present_notify();
        buffer.present()?;
        if !self.visible {
            self.visible = true;
            println!("visible");
            let _ = std::io::stdout().flush();
        }
        Ok(())
    }
}

impl ApplicationHandler for SplashApp {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        // A real, resizable loading card; normal desktop decoration owns move/close.
        // Escape/close dismisses this informational card without canceling engine work.
        let attributes = WindowAttributes::default()
            .with_title(startup_text("editor.startup.window_title").as_ref())
            // Keep the loading card visible over an as-yet unpainted workbench window.
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_surface_size(LogicalSize::new(640.0, 440.0))
            .with_min_surface_size(LogicalSize::new(400.0, 360.0));
        let result = (|| -> Result<_, Box<dyn std::error::Error>> {
            let window: Arc<dyn Window> = event_loop.create_window(attributes)?.into();
            let context = Context::new(window.clone())?;
            let surface = Surface::new(&context, window.clone())?;
            Ok((window, context, surface))
        })();
        match result {
            Ok((window, context, surface)) => {
                window.request_redraw();
                self.window = Some(window);
                self._context = Some(context);
                self.surface = Some(surface);
            }
            Err(error) => {
                eprintln!("[zircon_editor] startup splash surface unavailable: {error}");
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. }
                if event.logical_key == Key::Named(NamedKey::Escape) =>
            {
                event_loop.exit()
            }
            WindowEvent::SurfaceResized(_) => {
                if self.visible {
                    println!("resized");
                    let _ = std::io::stdout().flush();
                }
                self.refresh_or_exit(event_loop);
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.present() {
                    eprintln!("[zircon_editor] startup splash presentation unavailable: {error}");
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.refresh_or_exit(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .closed
        {
            event_loop.exit();
            return;
        }
        let now = Instant::now();
        if now >= self.next_tick {
            self.next_tick = now + Duration::from_secs(1);
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
        // Elapsed time updates once a second. No continuous animation or engine ticks.
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_tick));
    }
}

#[cfg(test)]
#[path = "tests/child.rs"]
mod tests;
