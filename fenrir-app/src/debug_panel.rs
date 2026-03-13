//! Fenrir Debug Panel — nur in Debug-Builds aktiv.
//!
//! Zeigt Echtzeit-Zustand der Rendering-Pipeline, Servo-Events,
//! Input-Events, egui-Fokus und Netzwerk-Anfragen.
//!
//! Toggle: F12

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use egui::{Color32, Grid, RichText, ScrollArea, Window};

const MAX_EVENTS: usize = 40;

// ── Event-Typen ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum ServoEvent {
    FrameReady,
    UrlChanged(String),
    TitleChanged(String),
    LoadStatus(String),
    Navigation(String),
    Permission(String),
    ConstellationLost,
}

impl ServoEvent {
    fn label(&self) -> (&str, Color32) {
        match self {
            Self::FrameReady           => ("frame_ready",        Color32::from_rgb(100, 200, 100)),
            Self::UrlChanged(_)        => ("url_changed",        Color32::from_rgb(100, 180, 255)),
            Self::TitleChanged(_)      => ("title_changed",      Color32::from_rgb(180, 180, 100)),
            Self::LoadStatus(_)        => ("load_status",        Color32::from_rgb(200, 150, 100)),
            Self::Navigation(_)        => ("navigation",         Color32::from_rgb(200, 100, 200)),
            Self::Permission(_)        => ("permission",         Color32::from_rgb(255, 100, 100)),
            Self::ConstellationLost    => ("CONSTELLATION_LOST", Color32::RED),
        }
    }

    fn detail(&self) -> Option<&str> {
        match self {
            Self::UrlChanged(s) | Self::TitleChanged(s)
            | Self::LoadStatus(s) | Self::Navigation(s)
            | Self::Permission(s) => Some(s),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum InputEvent {
    MouseMove(f32, f32),
    MouseButton(String),
    Scroll(f32, f32),
    Keyboard(String),
    EguiConsumed(String),
    MouseLeftViewport,
}

impl InputEvent {
    fn label(&self) -> (&str, Color32) {
        match self {
            Self::MouseMove(..)       => ("mouse_move",        Color32::GRAY),
            Self::MouseButton(_)      => ("mouse_button",      Color32::from_rgb(200, 200, 100)),
            Self::Scroll(..)          => ("scroll",            Color32::from_rgb(180, 180, 180)),
            Self::Keyboard(_)         => ("keyboard",          Color32::from_rgb(100, 200, 255)),
            Self::EguiConsumed(_)     => ("egui_consumed",     Color32::from_rgb(255, 180, 50)),
            Self::MouseLeftViewport   => ("mouse_left_vp",     Color32::from_rgb(150, 150, 255)),
        }
    }

    fn detail(&self) -> Option<String> {
        match self {
            Self::MouseMove(x, y)   => Some(format!("({x:.0}, {y:.0})")),
            Self::MouseButton(s)    => Some(s.clone()),
            Self::Scroll(x, y)      => Some(format!("({x:.1}, {y:.1})")),
            Self::Keyboard(s)       => Some(s.clone()),
            Self::EguiConsumed(s)   => Some(s.clone()),
            Self::MouseLeftViewport => None,
        }
    }
}

// ── DebugState ────────────────────────────────────────────────────────────────

pub struct DebugState {
    pub visible: bool,

    // Rendering
    pub render_count: u64,
    pub blit_some_count: u64,
    pub blit_none_count: u64,
    pub last_blit_was_some: Option<bool>,
    pub spin_count: u64,
    pub last_render: Option<Instant>,

    // egui
    pub egui_focus_widget: String,
    pub egui_consumed_last: bool,

    // Servo
    pub servo_events: VecDeque<(Instant, ServoEvent)>,

    // Input
    pub input_events: VecDeque<(Instant, InputEvent)>,

    // WebView Zustand
    pub load_status: String,
    pub current_url: String,
    pub page_title: String,
    pub can_go_back: bool,
    pub can_go_forward: bool,

    // Window
    pub window_size: (u32, u32),
    pub servo_size: (u32, u32),
    pub scale_factor: f64,
}

impl Default for DebugState {
    fn default() -> Self {
        Self {
            visible: false,
            render_count: 0,
            blit_some_count: 0,
            blit_none_count: 0,
            last_blit_was_some: None,
            spin_count: 0,
            last_render: None,
            egui_focus_widget: "—".to_string(),
            egui_consumed_last: false,
            servo_events: VecDeque::with_capacity(MAX_EVENTS),
            input_events: VecDeque::with_capacity(MAX_EVENTS),
            load_status: "—".to_string(),
            current_url: "—".to_string(),
            page_title: "—".to_string(),
            can_go_back: false,
            can_go_forward: false,
            window_size: (0, 0),
            servo_size: (0, 0),
            scale_factor: 1.0,
        }
    }
}

impl DebugState {
    pub fn push_servo(&mut self, ev: ServoEvent) {
        if self.servo_events.len() >= MAX_EVENTS {
            self.servo_events.pop_front();
        }
        self.servo_events.push_back((Instant::now(), ev));
    }

    pub fn push_input(&mut self, ev: InputEvent) {
        // Mouse-Move nicht jedes Mal loggen — nur wenn Position sich stark ändert
        if let InputEvent::MouseMove(..) = &ev {
            if let Some((_, InputEvent::MouseMove(..))) = self.input_events.back() {
                // letztes event war auch move → überschreiben statt pushen
                self.input_events.pop_back();
            }
        }
        if self.input_events.len() >= MAX_EVENTS {
            self.input_events.pop_front();
        }
        self.input_events.push_back((Instant::now(), ev));
    }

    pub fn record_render(&mut self, blit_was_some: bool) {
        self.render_count += 1;
        self.last_render = Some(Instant::now());
        self.last_blit_was_some = Some(blit_was_some);
        if blit_was_some {
            self.blit_some_count += 1;
        } else {
            self.blit_none_count += 1;
        }
    }
}

// ── Render ────────────────────────────────────────────────────────────────────

pub fn render(state: &mut DebugState, ctx: &egui::Context) {
    if !state.visible {
        return;
    }

    // egui-Fokus lesen
    state.egui_focus_widget = ctx
        .memory(|m| m.focused())
        .map(|id| format!("{id:?}"))
        .unwrap_or_else(|| "—".to_string());

    Window::new("🔧 Fenrir Debug")
        .default_pos([10.0, 50.0])
        .default_size([440.0, 600.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.collapsing("📐 Rendering Pipeline", |ui| {
                render_section(ui, state);
            });
            ui.collapsing("🎮 Input Events", |ui| {
                input_section(ui, state);
            });
            ui.collapsing("🌐 WebView Zustand", |ui| {
                webview_section(ui, state);
            });
            ui.collapsing("⚡ Servo Events", |ui| {
                servo_events_section(ui, state);
            });
        });
}

fn render_section(ui: &mut egui::Ui, state: &DebugState) {
    let blit_color = match state.last_blit_was_some {
        Some(true)  => Color32::GREEN,
        Some(false) => Color32::RED,
        None        => Color32::GRAY,
    };
    let blit_label = match state.last_blit_was_some {
        Some(true)  => "✅ Some (Servo rendert)",
        Some(false) => "❌ None (kein Frame)",
        None        => "? (noch kein Render)",
    };

    Grid::new("render_grid").num_columns(2).striped(true).show(ui, |ui| {
        ui.label("blit callback");
        ui.label(RichText::new(blit_label).color(blit_color));
        ui.end_row();

        ui.label("renders gesamt");
        ui.label(state.render_count.to_string());
        ui.end_row();

        ui.label("blit Some / None");
        ui.label(format!("{} / {}", state.blit_some_count, state.blit_none_count));
        ui.end_row();

        ui.label("servo spin()");
        ui.label(state.spin_count.to_string());
        ui.end_row();

        ui.label("seit letztem render");
        let age = state.last_render
            .map(|t| format!("{:.0}ms", t.elapsed().as_millis()))
            .unwrap_or_else(|| "—".to_string());
        ui.label(age);
        ui.end_row();

        ui.label("window size");
        ui.label(format!("{}×{}", state.window_size.0, state.window_size.1));
        ui.end_row();

        ui.label("servo size");
        ui.label(format!("{}×{}", state.servo_size.0, state.servo_size.1));
        ui.end_row();

        ui.label("scale factor");
        ui.label(format!("{:.2}", state.scale_factor));
        ui.end_row();

        ui.label("egui focus");
        ui.label(RichText::new(&state.egui_focus_widget).monospace());
        ui.end_row();

        ui.label("egui consumed");
        let consumed_color = if state.egui_consumed_last { Color32::YELLOW } else { Color32::GRAY };
        ui.label(RichText::new(state.egui_consumed_last.to_string()).color(consumed_color));
        ui.end_row();
    });
}

fn webview_section(ui: &mut egui::Ui, state: &DebugState) {
    Grid::new("webview_grid").num_columns(2).striped(true).show(ui, |ui| {
        ui.label("load status");
        let color = match state.load_status.as_str() {
            "Complete"    => Color32::GREEN,
            "Loading"     => Color32::YELLOW,
            "Connecting"  => Color32::from_rgb(255, 165, 0),
            _             => Color32::GRAY,
        };
        ui.label(RichText::new(&state.load_status).color(color));
        ui.end_row();

        ui.label("url");
        ui.label(RichText::new(&state.current_url).monospace().small());
        ui.end_row();

        ui.label("title");
        ui.label(&state.page_title);
        ui.end_row();

        ui.label("can go back");
        ui.label(bool_label(state.can_go_back));
        ui.end_row();

        ui.label("can go fwd");
        ui.label(bool_label(state.can_go_forward));
        ui.end_row();
    });
}

fn servo_events_section(ui: &mut egui::Ui, state: &DebugState) {
    ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
        for (ts, ev) in state.servo_events.iter().rev() {
            let (name, color) = ev.label();
            let age_ms = ts.elapsed().as_millis();
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{age_ms:>5}ms")).small().color(Color32::GRAY));
                ui.label(RichText::new(name).color(color).monospace());
                if let Some(detail) = ev.detail() {
                    ui.label(RichText::new(detail).small().monospace());
                }
            });
        }
    });
}

fn input_section(ui: &mut egui::Ui, state: &DebugState) {
    ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
        for (ts, ev) in state.input_events.iter().rev() {
            let (name, color) = ev.label();
            let age_ms = ts.elapsed().as_millis();
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{age_ms:>5}ms")).small().color(Color32::GRAY));
                ui.label(RichText::new(name).color(color).monospace());
                if let Some(detail) = ev.detail() {
                    ui.label(RichText::new(detail).small().monospace());
                }
            });
        }
    });
}

fn bool_label(v: bool) -> RichText {
    if v {
        RichText::new("✅").color(Color32::GREEN)
    } else {
        RichText::new("✗").color(Color32::GRAY)
    }
}

fn _age(ts: Instant) -> String {
    let d = ts.elapsed();
    if d < Duration::from_secs(60) {
        format!("{:.1}s", d.as_secs_f32())
    } else {
        format!("{:.0}min", d.as_secs_f32() / 60.0)
    }
}
