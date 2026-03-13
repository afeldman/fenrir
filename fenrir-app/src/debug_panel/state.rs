//! DebugState und Event-Typen für das Debug-Panel.

use std::collections::VecDeque;
use std::time::Instant;

use egui::Color32;

use super::MAX_EVENTS;

// ── Event-Typen ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum ServoEvent {
    FrameReady,
    UrlChanged(String),
    TitleChanged(String),
    LoadStatus(String),
    Navigation(String),
    Permission(String),
    Console(String),
}

impl ServoEvent {
    pub fn label(&self) -> (&str, Color32) {
        match self {
            Self::FrameReady           => ("frame_ready",        Color32::from_rgb(100, 200, 100)),
            Self::UrlChanged(_)        => ("url_changed",        Color32::from_rgb(100, 180, 255)),
            Self::TitleChanged(_)      => ("title_changed",      Color32::from_rgb(180, 180, 100)),
            Self::LoadStatus(_)        => ("load_status",        Color32::from_rgb(200, 150, 100)),
            Self::Navigation(_)        => ("navigation",         Color32::from_rgb(200, 100, 200)),
            Self::Permission(_)        => ("permission",         Color32::from_rgb(255, 100, 100)),
            Self::Console(_)           => ("console",            Color32::from_rgb(150, 150, 150)),
        }
    }

    pub fn detail(&self) -> Option<&str> {
        match self {
            Self::UrlChanged(s) | Self::TitleChanged(s)
            | Self::LoadStatus(s) | Self::Navigation(s)
            | Self::Permission(s) | Self::Console(s) => Some(s),
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
    MouseLeftViewport,
}

impl InputEvent {
    pub fn label(&self) -> (&str, Color32) {
        match self {
            Self::MouseMove(..)       => ("mouse_move",        Color32::GRAY),
            Self::MouseButton(_)      => ("mouse_button",      Color32::from_rgb(200, 200, 100)),
            Self::Scroll(..)          => ("scroll",            Color32::from_rgb(180, 180, 180)),
            Self::Keyboard(_)         => ("keyboard",          Color32::from_rgb(100, 200, 255)),
            Self::MouseLeftViewport   => ("mouse_left_vp",     Color32::from_rgb(150, 150, 255)),
        }
    }

    pub fn detail(&self) -> Option<String> {
        match self {
            Self::MouseMove(x, y)   => Some(format!("({x:.0}, {y:.0})")),
            Self::MouseButton(s)    => Some(s.clone()),
            Self::Scroll(x, y)      => Some(format!("({x:.1}, {y:.1})")),
            Self::Keyboard(s)       => Some(s.clone()),
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
