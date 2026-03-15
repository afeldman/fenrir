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
use tracing::{info, warn, debug, error, trace};
use url::Url;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::ModifiersState;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

use fenrir_config;
use crate::debug_panel::{self, DebugState, ServoEvent};
use crate::input;
use crate::toolbar::{self, ToolbarAction, ToolbarState};
use crate::waker::FenrirWaker;
use fenrir_i18n::{t, t_with_args};
use fluent::FluentArgs;

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
    config: fenrir_config::FenrirConfig,
    // Frame-Rate-Limiting: Verhindert zu häufige Redraws
    last_redraw_time: Cell<std::time::Instant>,
    // Rendering-Kontrolle: Reduziert Rendering wenn Seite geladen ist
    page_loaded: Cell<bool>,
    last_frame_count: Cell<u32>,
    consecutive_static_frames: Cell<u32>,
}

impl BrowserState {
    pub fn new(event_loop: &ActiveEventLoop, waker: FenrirWaker) -> Rc<Self> {
        // Config laden
        let config = fenrir_config::load().unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Config-Fehler, nutze Defaults");
            fenrir_config::FenrirConfig::default()
        });
        
        tracing::debug!("Geladene Konfiguration: start_url = {}", config.ui.start_url);
        
        // Start-URL aus Konfiguration parsen
        let initial_url = match Url::parse(&config.ui.start_url) {
            Ok(url) => {
                tracing::debug!("URL erfolgreich geparsed: {}", url);
                url
            },
            Err(e) => {
                tracing::warn!("Ungültige Start-URL in Konfiguration: {}, Fehler: {}, verwende Standard", config.ui.start_url, e);
                // Verwende eine Standard-URL
                let default_url = Url::parse("file:///Users/anton.feldmann/Projects/priv/browser/test_minimal.html")
                    .expect("Standard-URL sollte gültig sein");
                tracing::debug!("Verwende Standard-URL: {}", default_url);
                default_url
            }
        };
        
        debug!("Erstelle BrowserState mit URL: {}", initial_url);
        
        // Icon für das Fenster laden
        let window_icon = load_window_icon();
        
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("Fenrir Browser")
                    .with_inner_size(PhysicalSize::new(config.ui.window_width, config.ui.window_height))
                    .with_window_icon(window_icon)
                    .with_visible(true)
                    .with_active(true),
            )
            .expect("Window konnte nicht erstellt werden");

        trace!("Fenster erstellt: {}x{}", window.inner_size().width, window.inner_size().height);

        let window_ctx = Rc::new(
            WindowRenderingContext::new(
                event_loop.display_handle().expect("display handle"),
                window.window_handle().expect("window handle"),
                window.inner_size(),
            )
            .expect("WindowRenderingContext"),
        );
        window_ctx.make_current().expect("make_current");

        let toolbar_offset = (config.ui.toolbar_height * window.scale_factor() as f32) as u32;
        let servo_size = PhysicalSize::new(
            window.inner_size().width,
            window.inner_size().height.saturating_sub(toolbar_offset),
        );
        debug!("Servo Größe: {}x{} (Toolbar: {}px)", servo_size.width, servo_size.height, toolbar_offset);
        
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
            config,
            // Frame-Rate-Limiting initialisieren
            last_redraw_time: Cell::new(std::time::Instant::now()),
            // Rendering-Kontrolle initialisieren
            page_loaded: Cell::new(false),
            last_frame_count: Cell::new(0),
            consecutive_static_frames: Cell::new(0),
        });

        state.servo.set_delegate(Rc::new(FenrirServoDelegate));

        // Servo-Kontext current setzen bevor WebView erstellt wird
        state.servo_ctx.make_current().expect("servo_ctx make_current vor WebView-Erstellung");
        trace!("Servo Context Größe: {:?}", state.servo_ctx.size());

        let webview = WebViewBuilder::new(
            &state.servo,
            state.servo_ctx.clone() as Rc<dyn RenderingContext>,
        )
        .url(initial_url.clone())
        .hidpi_scale_factor(Scale::new(state.window.scale_factor() as f32))
        .delegate(state.clone())
        .build();

        *state.webview.borrow_mut() = Some(webview);
        debug!("WebView erstellt, fordere initialen Redraw an");
        state.request_redraw();
        
        info!("Browser gestartet mit URL: {}", initial_url);
        trace!("BrowserState vollständig initialisiert");
        
        state
    }

    /// Request redraw immediately (for user interactions)
    /// This updates the last_redraw_time to prevent frame rate limiting from blocking user interactions
    pub fn request_redraw(&self) {
        self.last_redraw_time.set(std::time::Instant::now());
        self.window.request_redraw();
    }
    
    /// Request redraw with frame rate limiting (max 30 FPS for automatic updates)
    /// This is only used for automatic redraws from Servo's notify_new_frame_ready
    fn request_redraw_limited(&self) {
        const MIN_FRAME_TIME: std::time::Duration = std::time::Duration::from_millis(33); // ~30 FPS
        
        let now = std::time::Instant::now();
        let time_since_last_redraw = now.duration_since(self.last_redraw_time.get());
        
        if time_since_last_redraw >= MIN_FRAME_TIME {
            // Enough time has passed, request redraw immediately
            self.last_redraw_time.set(now);
            self.window.request_redraw();
        }
        // If too soon, we just skip this redraw request
        // This prevents too many redraws from Servo's notify_new_frame_ready
    }

    /// Compositing: Servo-Offscreen → Window, dann egui-Toolbar + Debug-Panel oben drauf.
    pub fn render(&self) {
        trace!("Beginne Render-Zyklus");

        // 1. GL-Kontext current setzen (gemeinsamer CGL-Kontext für Servo + egui).
        self.servo_ctx.make_current().expect("servo_ctx make_current");

        // 2. WebView in den Offscreen-FBO rendern.
        //    MUSS hier passieren: Intern ruft paint() prepare_for_rendering() auf und
        //    lässt WebRender in den FBO compositen. Ohne diesen Call bleibt der FBO leer.
        if let Some(wv) = self.webview.borrow().as_ref() {
            wv.paint();
        }

        // 3. Blit-Callback holen — FBO ist jetzt befüllt.
        type BlitFn = dyn Fn(&egui_glow::glow::Context, euclid::default::Rect<i32>) + Send + Sync;
        let blit_callback: Option<Arc<BlitFn>> = self.servo_ctx
            .render_to_parent_callback()
            .map(|b| Arc::from(b) as Arc<BlitFn>);

        if blit_callback.is_none() {
            debug!("Kein Blit-Callback — render_to_parent_callback returned None");
        }

        // 4. Window-Context für Darstellung vorbereiten.
        self.window_ctx.prepare_for_rendering();

        let mut nav_url: Option<String> = None;
        let mut nav_back = false;
        let mut nav_fwd = false;
        let mut nav_reload = false;
        let mut focus_url = false;

        let mut egui = self.egui.borrow_mut();

        {
            let mut toolbar = self.toolbar.borrow_mut();
            let mut dbg = self.debug.borrow_mut();
            let toolbar_height = self.config.ui.toolbar_height;

            egui.run(&self.window, |ctx| {
                // Servo-Inhalt im Hintergrund rendern (unter Toolbar und Debug-Panel).
                if let Some(blit) = &blit_callback {
                    let blit = blit.clone();
                    let screen = ctx.content_rect();
                    let webview_height = (screen.height() - toolbar_height).max(0.0);
                    let webview_rect = egui::Rect::from_min_size(
                        egui::pos2(0.0, toolbar_height),
                        egui::vec2(screen.width(), webview_height),
                    );
                    ctx.layer_painter(egui::LayerId::new(
                        egui::Order::Background,
                        egui::Id::new("webview_blit"),
                    ))
                    .add(egui::PaintCallback {
                        rect: webview_rect,
                        callback: Arc::new(egui_glow::CallbackFn::new(move |info, painter| {
                            let clip = info.viewport_in_pixels();
                            let target = euclid::default::Rect::new(
                                euclid::default::Point2D::new(
                                    clip.left_px.max(0),
                                    clip.from_bottom_px.max(0),
                                ),
                                euclid::default::Size2D::new(
                                    clip.width_px.max(0),
                                    clip.height_px.max(0),
                                ),
                            );
                            blit(painter.gl(), target);
                        })),
                    });
                }

                match toolbar::render(&mut toolbar, ctx, toolbar_height) {
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

        if focus_url {
            self.toolbar.borrow_mut().focus_url_bar();
            self.request_redraw();
        }

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
        let toolbar_px = (self.config.ui.toolbar_height * scale) as u32;
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
        let toolbar_px = self.config.ui.toolbar_height * scale;
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
            // Verwende die konfigurierte Suchmaschine
            self.config.privacy.search_engine.search_url(&raw)
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
        self.request_redraw_limited();
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
        let t = title.unwrap_or_else(|| t("app-name"));
        self.debug.borrow_mut().push_servo(ServoEvent::TitleChanged(t.clone()));
        self.debug.borrow_mut().page_title = t.clone();
        self.toolbar.borrow_mut().page_title = t.clone();
        
        // Use i18n for window title
        let mut args = FluentArgs::new();
        args.set("title", t.clone());
        let window_title = t_with_args("app-title", &args);
        self.window.set_title(&window_title);
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
            
            // Nachdem die Seite geladen ist, einen Redraw anfordern
            // Das könnte helfen, den Framebuffer zu initialisieren
            info!("Seite vollständig geladen! Fordere Redraw an...");
            self.request_redraw();
        } else if status == LoadStatus::Started {
            info!("Seite wird geladen...");
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

/// Lädt das Fenster-Icon für die aktuelle Plattform
fn load_window_icon() -> Option<winit::window::Icon> {
    use std::fs;
    use winit::window::Icon;
    
    // Versuche verschiedene Icon-Pfade (Priorität: ICO > PNG > Fallbacks)
    let icon_paths = [
        // Primäres ICO-Icon
        "resources/fenrir.ico",
        // PNG-Icons (die tatsächlich existieren)
        "resources/fenrir.png",
        "resources/logo.png",
        // Fallback: resources/ Ordner
        "resources/servo.ico",
        "resources/servo.icns",
        "resources/servo_64.png",
    ];
    
    for path in &icon_paths {
        if let Ok(bytes) = fs::read(path) {
            // Versuche zuerst mit spezifischen Parsern
            if path.ends_with(".png") {
                match load_png_file(path) {
                    Ok(icon) => {
                        tracing::debug!("PNG Icon geladen von: {}", path);
                        return Some(icon);
                    }
                    Err(_) => {
                        // PNG-Parser fehlgeschlagen, weiter zum nächsten
                    }
                }
            }
            
            if path.ends_with(".ico") {
                match load_ico_file(path) {
                    Ok(icon) => {
                        tracing::debug!("ICO Icon geladen von: {}", path);
                        return Some(icon);
                    }
                    Err(_) => {
                        // ICO-Parser fehlgeschlagen, weiter zum nächsten
                    }
                }
            }
            
            // Fallback: Versuche direkte Methode (nur für PNG mit 64x64)
            if path.ends_with(".png") {
                if let Ok(icon) = Icon::from_rgba(bytes, 64, 64) {
                    tracing::debug!("Icon direkt geladen von: {}", path);
                    return Some(icon);
                }
            }
        }
    }
    
    tracing::warn!("Kein Icon gefunden, verwende Standard");
    None
}

/// Lädt eine PNG-Datei und konvertiert sie in ein winit Icon
fn load_png_file(path: &str) -> Result<winit::window::Icon, Box<dyn std::error::Error>> {
    use image::ImageReader;
    use image::GenericImageView;
    
    tracing::debug!("Versuche PNG-Datei zu laden: {}", path);
    
    // Verwende ImageReader, der automatisch das Format erkennt
    let img = match ImageReader::open(path) {
        Ok(reader) => match reader.with_guessed_format() {
            Ok(reader_with_format) => match reader_with_format.decode() {
                Ok(img) => img,
                Err(e) => {
                    tracing::error!("Fehler beim Dekodieren von PNG: {}", e);
                    return Err(Box::new(e));
                }
            },
            Err(e) => {
                tracing::error!("Fehler beim Erraten des Formats: {}", e);
                return Err(Box::new(e));
            }
        },
        Err(e) => {
            tracing::error!("Fehler beim Öffnen der Datei: {}", e);
            return Err(Box::new(e));
        }
    };
    
    let (width, height) = img.dimensions();
    tracing::debug!("PNG Dimensionen: {}x{}", width, height);
    
    // Konvertiere zu RGBA
    let rgba = img.to_rgba8();
    tracing::debug!("PNG Daten konvertiert: {} Bytes", rgba.len());
    
    match winit::window::Icon::from_rgba(rgba.to_vec(), width, height) {
        Ok(icon) => {
            tracing::debug!("PNG Icon erfolgreich erstellt");
            Ok(icon)
        },
        Err(e) => {
            tracing::error!("Fehler beim Erstellen des Icons: {}", e);
            Err(Box::new(e))
        }
    }
}

/// Lädt eine ICO-Datei und konvertiert sie in ein winit Icon
fn load_ico_file(path: &str) -> Result<winit::window::Icon, Box<dyn std::error::Error>> {
    use image::ImageReader;
    use image::GenericImageView;
    
    tracing::debug!("Versuche ICO-Datei zu laden: {}", path);
    
    // Verwende ImageReader, der automatisch das Format erkennt
    let img = match ImageReader::open(path) {
        Ok(reader) => match reader.with_guessed_format() {
            Ok(reader_with_format) => match reader_with_format.decode() {
                Ok(img) => img,
                Err(e) => {
                    tracing::error!("Fehler beim Dekodieren von ICO: {}", e);
                    return Err(Box::new(e));
                }
            },
            Err(e) => {
                tracing::error!("Fehler beim Erraten des Formats: {}", e);
                return Err(Box::new(e));
            }
        },
        Err(e) => {
            tracing::error!("Fehler beim Öffnen der Datei: {}", e);
            return Err(Box::new(e));
        }
    };
    
    let (width, height) = img.dimensions();
    tracing::debug!("ICO Dimensionen: {}x{}", width, height);
    
    // Konvertiere zu RGBA
    let rgba = img.to_rgba8();
    tracing::debug!("ICO Daten konvertiert: {} Bytes", rgba.len());
    
    match winit::window::Icon::from_rgba(rgba.to_vec(), width, height) {
        Ok(icon) => {
            tracing::debug!("ICO Icon erfolgreich erstellt");
            Ok(icon)
        },
        Err(e) => {
            tracing::error!("Fehler beim Erstellen des Icons: {}", e);
            Err(Box::new(e))
        }
    }
}
