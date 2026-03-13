//! Fenrir Browser Toolbar — URL-Bar, Navigation-Buttons, Load-Indicator.
//!
//! Rendert die obere Browser-Chrome via egui. Kommuniziert Nutzeraktionen
//! über `ToolbarAction`-Rückgabewert zurück an BrowserState.

use egui::{Frame, TopBottomPanel};
use fenrir_i18n::t;
use servo::LoadStatus;

/// Aktionen die die Toolbar an den Browser zurückgibt.
#[derive(Debug)]
pub enum ToolbarAction {
    Navigate(String),
    Back,
    Forward,
    Reload,
    FocusUrl,
    None,
}

/// Zustand der Toolbar (geteilt mit BrowserState via RefCell).
pub struct ToolbarState {
    pub url_input: String,
    pub current_url: String,
    pub page_title: String,
    pub load_status: LoadStatus,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    url_focused: bool,
}

impl ToolbarState {
    pub fn new(initial_url: &str) -> Self {
        Self {
            url_input: initial_url.to_string(),
            current_url: initial_url.to_string(),
            page_title: t("app-name"),
            load_status: LoadStatus::Complete,
            can_go_back: false,
            can_go_forward: false,
            url_focused: false,
        }
    }

    pub fn sync_url_from_browser(&mut self, url: &str) {
        if !self.url_focused {
            self.url_input = url.to_string();
        }
        self.current_url = url.to_string();
    }

    pub fn focus_url_bar(&mut self) {
        self.url_focused = true;
    }
}

/// Rendert die Toolbar. Gibt ToolbarAction zurück wenn der Nutzer etwas getan hat.
pub fn render(ui_state: &mut ToolbarState, ctx: &egui::Context, toolbar_height: f32) -> ToolbarAction {
    let mut action = ToolbarAction::None;

    let frame = Frame::default()
        .fill(ctx.style().visuals.window_fill)
        .inner_margin(6.0);

    TopBottomPanel::top("fenrir_toolbar")
        .frame(frame)
        .exact_height(toolbar_height)
        .show(ctx, |ui| {            
            ui.horizontal_centered(|ui| {
                // ← Back
                let back = ui.add_enabled(
                    ui_state.can_go_back,
                    egui::Button::new("◀").min_size(egui::vec2(28.0, 28.0)),
                );
                if back.clicked() {
                    action = ToolbarAction::Back;
                }

                // → Forward
                let fwd = ui.add_enabled(
                    ui_state.can_go_forward,
                    egui::Button::new("▶").min_size(egui::vec2(28.0, 28.0)),
                );
                if fwd.clicked() {
                    action = ToolbarAction::Forward;
                }

                // ↻ Reload / ✕ Stop
                let reload_label = match ui_state.load_status {
                    LoadStatus::Complete => "↻",
                    _ => "✕",
                };
                if ui.button(reload_label).clicked() {
                    action = ToolbarAction::Reload;
                }

                // URL-Bar
                let url_id = egui::Id::new("url_bar");
                let text_edit = egui::TextEdit::singleline(&mut ui_state.url_input)
                    .desired_width(ui.available_width() - 4.0)
                    .font(egui::TextStyle::Monospace)
                    .hint_text(t("toolbar-url-hint"));

                let resp = ui.add(text_edit);

                // URL-Bar fokussiert?
                ui_state.url_focused = resp.has_focus();

                // Setzt Fokus wenn extern angefordert
                if matches!(action, ToolbarAction::None) && ui_state.url_focused {
                    ctx.memory_mut(|m| m.request_focus(url_id));
                }

                // Enter → navigieren
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let url = ui_state.url_input.trim().to_string();
                    if !url.is_empty() {
                        action = ToolbarAction::Navigate(url);
                    }
                }
            });
        });

    action
}
