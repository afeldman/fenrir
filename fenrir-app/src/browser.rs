//! BrowserState — zentraler Zustand des Fenrir Browser Fensters.
//!
//! Hält Servo, Rendering-Contexts, egui, Toolbar-Zustand und WebView.
//! Implementiert `WebViewDelegate` für Servo-Callbacks.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use egui_glow::EguiGlow;
use euclid::Scale;
use servo::{
    InputEvent, LoadStatus, MouseLeftViewportEvent, NavigationRequest, OffscreenRenderingContext,
    PermissionRequest, RenderingContext, Servo, ServoBuilder, WebView, WebViewBuilder,
    WebViewDelegate, WindowRenderingContext,
};
use tracing::{info, warn};
use url::Url;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::ModifiersState;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

use crate::debug_panel::{self, DebugState, ServoEvent};
use crate::input;
use crate::toolbar::{self, ToolbarAction, ToolbarState, TOOLBAR_HEIGHT_PX};
use crate::waker::FenrirWaker;

pub struct BrowserState {
    pub window: Window,
    pub(crate) toolbar: RefCell<ToolbarState>,
    servo: Servo,
    window_ctx: Rc<WindowRenderingContext>,
    servo_ctx: Rc<OffscreenRenderingContext>,
    webview: RefCell<Option<WebView>>,
    egui: RefCell<EguiGlow>,
    mouse_pos: Cell<(f32, f32)>,
    modifiers: Cell<ModifiersState>,
    pub debug: RefCell<DebugState>,
}

impl BrowserState {
    pub fn new(event_loop: &ActiveEventLoop, waker: FenrirWaker, initial_url: Url) -> Rc<Self> {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("Fenrir")
                    .with_inner_size(PhysicalSize::new(1280u32, 800u32)),
            )
            .expect("Window konnte nicht erstellt werden");

        let window_ctx = Rc::new(
            WindowRenderingContext::new(
                event_loop.display_handle().expect("display handle"),
                window.window_handle().expect("window handle"),
                window.inner_size(),
            )
            .expect("WindowRenderingContext"),
        );
        window_ctx.make_current().expect("make_current");

        let toolbar_offset = (TOOLBAR_HEIGHT_PX * window.scale_factor() as f32) as u32;
        let servo_size = PhysicalSize::new(
            window.inner_size().width,
            window.inner_size().height.saturating_sub(toolbar_offset),
        );
        let servo_ctx = Rc::new(window_ctx.offscreen_context(servo_size));
        let egui = EguiGlow::new(event_loop, window_ctx.glow_gl_api(), None, None, true);

        let servo = ServoBuilder::default()
            .event_loop_waker(Box::new(waker))
            .build();

        let mut debug = DebugState::default();
        debug.window_size = (window.inner_size().width, window.inner_size().height);
        debug.servo_size = (servo_size.width, servo_size.height);
        debug.scale_factor = window.scale_factor();

        let state = Rc::new(Self {
            window,
            servo,
            window_ctx,
            servo_ctx,
            webview: RefCell::new(None),
            egui: RefCell::new(egui),
            toolbar: RefCell::new(ToolbarState::new(initial_url.as_str())),
            mouse_pos: Cell::new((0.0, 0.0)),
            modifiers: Cell::new(ModifiersState::empty()),
            debug: RefCell::new(debug),
        });

        state.servo.set_delegate(Rc::new(FenrirServoDelegate));

        let webview = WebViewBuilder::new(
            &state.servo,
            state.servo_ctx.clone() as Rc<dyn RenderingContext>,
        )
        .url(initial_url)
        .hidpi_scale_factor(Scale::new(state.window.scale_factor() as f32))
        .delegate(state.clone())
        .build();

        *state.webview.borrow_mut() = Some(webview);
        info!("Browser gestartet");
        state
    }

    /// Compositing: Servo-Offscreen → Window, dann egui-Toolbar + Debug-Panel oben drauf.
    pub fn render(&self) {
        // Offscreen-Kontext current setzen (Servo rendert dorthin),
        // dann Window-Kontext für egui/blit vorbereiten — Reihenfolge wie in servoshell.
        self.servo_ctx.make_current().expect("servo_ctx make_current");
        self.window_ctx.prepare_for_rendering();

        let mut nav_url: Option<String> = None;
        let mut nav_back = false;
        let mut nav_fwd = false;
        let mut nav_reload = false;
        let mut focus_url = false;

        let servo_ctx = self.servo_ctx.clone();
        let mut egui = self.egui.borrow_mut();

        {
            let mut toolbar = self.toolbar.borrow_mut();
            let mut dbg = self.debug.borrow_mut();

            // Box<dyn Fn> in Arc wrappen damit der Callback in egui-Closures geclont werden kann
            type BlitFn = dyn Fn(&egui_glow::glow::Context, euclid::default::Rect<i32>) + Send + Sync;
            let blit_callback: Option<Arc<BlitFn>> = servo_ctx
                .render_to_parent_callback()
                .map(|b| Arc::from(b) as Arc<BlitFn>);
            let blit_is_some = blit_callback.is_some();
            dbg.record_render(blit_is_some);

            egui.run(&self.window, |ctx| {
                if let Some(blit) = &blit_callback {
                    let blit = blit.clone(); // Arc::clone — billig
                    let screen = ctx.screen_rect();
                    let webview_rect = egui::Rect::from_min_size(
                        egui::pos2(0.0, TOOLBAR_HEIGHT_PX),
                        egui::vec2(screen.width(), screen.height() - TOOLBAR_HEIGHT_PX),
                    );
                    ctx.layer_painter(egui::LayerId::background()).add(
                        egui::PaintCallback {
                            rect: webview_rect,
                            callback: Arc::new(egui_glow::CallbackFn::new(
                                move |info, painter| {
                                    let clip = info.viewport_in_pixels();
                                    let target = euclid::default::Rect::new(
                                        euclid::default::Point2D::new(clip.left_px, clip.from_bottom_px),
                                        euclid::default::Size2D::new(clip.width_px, clip.height_px),
                                    );
                                    blit(painter.gl(), target);
                                },
                            )),
                        },
                    );
                }

                match toolbar::render(&mut toolbar, ctx) {
                    ToolbarAction::Navigate(url) => nav_url = Some(url),
                    ToolbarAction::Back          => nav_back = true,
                    ToolbarAction::Forward       => nav_fwd = true,
                    ToolbarAction::Reload        => nav_reload = true,
                    ToolbarAction::FocusUrl      => focus_url = true,
                    ToolbarAction::None          => {}
                }

                debug_panel::render(&mut dbg, ctx);
            });
        }

        if focus_url { self.toolbar.borrow_mut().focus_url_bar(); }

        egui.paint(&self.window);
        self.window_ctx.present();

        if let Some(url) = nav_url { self.navigate_string(url); }
        if nav_back    { self.go_back(); }
        if nav_fwd     { self.go_forward(); }
        if nav_reload  { self.reload(); }
    }

    pub fn spin(&self) {
        self.debug.borrow_mut().spin_count += 1;
        self.servo.spin_event_loop();
    }

    pub fn resize(&self, new_size: PhysicalSize<u32>) {
        let scale = self.window.scale_factor() as f32;
        let toolbar_px = (TOOLBAR_HEIGHT_PX * scale) as u32;
        let servo_size = PhysicalSize::new(
            new_size.width,
            new_size.height.saturating_sub(toolbar_px),
        );
        self.window_ctx.resize(new_size);
        self.servo_ctx.resize(servo_size);
        if let Some(wv) = self.webview.borrow().as_ref() {
            wv.resize(servo_size);
        }
        let mut dbg = self.debug.borrow_mut();
        dbg.window_size = (new_size.width, new_size.height);
        dbg.servo_size = (servo_size.width, servo_size.height);
    }

    pub fn on_window_event(&self, event: &winit::event::WindowEvent) -> bool {
        let consumed = self
            .egui
            .borrow_mut()
            .on_window_event(&self.window, event)
            .consumed;
        self.debug.borrow_mut().egui_consumed_last = consumed;
        consumed
    }

    pub fn handle_cursor_moved(&self, position: PhysicalPosition<f64>) {
        let scale = self.window.scale_factor() as f32;
        let toolbar_px = TOOLBAR_HEIGHT_PX * scale;
        let x = position.x as f32;
        let y = position.y as f32 - toolbar_px;
        self.mouse_pos.set((x, y));

        if y < 0.0 {
            if let Some(wv) = self.webview.borrow().as_ref() {
                wv.notify_input_event(InputEvent::MouseLeftViewport(
                    MouseLeftViewportEvent::default(),
                ));
            }
            self.debug.borrow_mut().push_input(crate::debug_panel::InputEvent::MouseLeftViewport);
            return;
        }
        if let Some(wv) = self.webview.borrow().as_ref() {
            wv.notify_input_event(input::mouse_move(x, y));
        }
        self.debug.borrow_mut().push_input(crate::debug_panel::InputEvent::MouseMove(x, y));
    }

    pub fn handle_mouse_button(&self, button: MouseButton, state: ElementState) {
        let (x, y) = self.mouse_pos.get();
        if y < 0.0 { return; }
        let label = format!("{button:?} {state:?} @ ({x:.0},{y:.0})");
        if let Some(wv) = self.webview.borrow().as_ref() {
            wv.notify_input_event(input::mouse_button(button, state, x, y));
        }
        self.debug.borrow_mut().push_input(crate::debug_panel::InputEvent::MouseButton(label));
    }

    pub fn handle_scroll(&self, delta: MouseScrollDelta) {
        let (x, y) = self.mouse_pos.get();
        let (dx, dy) = match delta {
            MouseScrollDelta::LineDelta(a, b) => (a, b),
            MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
        };
        if let Some(wv) = self.webview.borrow().as_ref() {
            wv.notify_input_event(input::scroll(delta, x, y));
        }
        self.debug.borrow_mut().push_input(crate::debug_panel::InputEvent::Scroll(dx, dy));
    }

    pub fn handle_keyboard(&self, event: &winit::event::KeyEvent) {
        let label = format!("{:?}", event.logical_key);
        if let Some(ev) = input::keyboard(event, self.modifiers.get()) {
            if let Some(wv) = self.webview.borrow().as_ref() {
                wv.notify_input_event(ev);
            }
            self.debug.borrow_mut().push_input(crate::debug_panel::InputEvent::Keyboard(label));
        }
    }

    pub fn set_modifiers(&self, state: ModifiersState) {
        self.modifiers.set(state);
    }

    pub fn get_modifiers(&self) -> ModifiersState {
        self.modifiers.get()
    }

    pub fn toggle_debug(&self) {
        let mut dbg = self.debug.borrow_mut();
        dbg.visible = !dbg.visible;
    }

    // ── Navigation ────────────────────────────────────────────────────────────

    pub fn navigate_string(&self, raw: String) {
        let url_str = if raw.contains("://") || raw.starts_with("about:") {
            raw.clone()
        } else if raw.contains('.') && !raw.contains(' ') {
            format!("https://{raw}")
        } else {
            format!("https://duckduckgo.com/?q={}", urlencoding::encode(&raw))
        };
        match Url::parse(&url_str) {
            Ok(url) => {
                if let Some(wv) = self.webview.borrow().as_ref() {
                    wv.load(url);
                }
            }
            Err(_) => warn!(url = %url_str, "Ungültige URL"),
        }
    }

    pub fn go_back(&self) {
        if let Some(wv) = self.webview.borrow().as_ref() {
            wv.go_back(1);
        }
    }

    pub fn go_forward(&self) {
        if let Some(wv) = self.webview.borrow().as_ref() {
            wv.go_forward(1);
        }
    }

    pub fn reload(&self) {
        if let Some(wv) = self.webview.borrow().as_ref() {
            wv.reload();
        }
    }
}

impl WebViewDelegate for BrowserState {
    fn notify_new_frame_ready(&self, _wv: WebView) {
        self.debug.borrow_mut().push_servo(ServoEvent::FrameReady);
        self.window.request_redraw();
    }

    fn notify_url_changed(&self, wv: WebView, url: Url) {
        let url_str = url.to_string();
        self.debug.borrow_mut().push_servo(ServoEvent::UrlChanged(url_str.clone()));
        self.debug.borrow_mut().current_url = url_str.clone();
        let mut t = self.toolbar.borrow_mut();
        t.sync_url_from_browser(&url_str);
        t.can_go_back = wv.can_go_back();
        t.can_go_forward = wv.can_go_forward();
        let mut dbg = self.debug.borrow_mut();
        dbg.can_go_back = wv.can_go_back();
        dbg.can_go_forward = wv.can_go_forward();
    }

    fn notify_page_title_changed(&self, _wv: WebView, title: Option<String>) {
        let t = title.unwrap_or_else(|| "Fenrir".to_string());
        self.debug.borrow_mut().push_servo(ServoEvent::TitleChanged(t.clone()));
        self.debug.borrow_mut().page_title = t.clone();
        self.toolbar.borrow_mut().page_title = t.clone();
        self.window.set_title(&format!("Fenrir — {t}"));
    }

    fn notify_load_status_changed(&self, wv: WebView, status: LoadStatus) {
        let label = format!("{status:?}");
        self.debug.borrow_mut().push_servo(ServoEvent::LoadStatus(label.clone()));
        self.debug.borrow_mut().load_status = label;
        let mut toolbar = self.toolbar.borrow_mut();
        toolbar.load_status = status;
        if status == LoadStatus::Complete {
            toolbar.can_go_back = wv.can_go_back();
            toolbar.can_go_forward = wv.can_go_forward();
        }
    }

    fn request_navigation(&self, _wv: WebView, req: NavigationRequest) {
        self.debug.borrow_mut().push_servo(ServoEvent::Navigation(req.url.to_string()));
        // Alle erlaubt — fenrir-security kommt in Phase 2
    }

    fn request_permission(&self, _wv: WebView, req: PermissionRequest) {
        self.debug.borrow_mut().push_servo(ServoEvent::Permission(
            format!("{:?}", req.feature())
        ));
        req.deny();
    }
}

/// Globaler Servo-Delegate (Browser-Level Events)
struct FenrirServoDelegate;
impl servo::ServoDelegate for FenrirServoDelegate {}
