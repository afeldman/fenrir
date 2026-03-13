//! Widget-Rendering für das Debug-Panel.

use egui::{Color32, Grid, RichText, ScrollArea, Window};
use fenrir_i18n::t;

use super::state::{DebugState, InputEvent, ServoEvent};

pub fn render(state: &mut DebugState, ctx: &egui::Context) {
    if !state.visible {
        return;
    }

    // egui-Fokus lesen
    state.egui_focus_widget = ctx
        .memory(|m| m.focused())
        .map(|id| format!("{id:?}"))
        .unwrap_or_else(|| "—".to_string());

    Window::new(&t("debug-panel-title"))
        .default_pos([10.0, 50.0])
        .default_size([440.0, 600.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.collapsing(&t("debug-section-rendering"), |ui| {
                render_section(ui, state);
            });
            ui.collapsing(&t("debug-section-input"), |ui| {
                input_section(ui, state);
            });
            ui.collapsing(&t("debug-section-webview"), |ui| {
                webview_section(ui, state);
            });
            ui.collapsing(&t("debug-section-servo"), |ui| {
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
        Some(true)  => t("debug-blit-some"),
        Some(false) => t("debug-blit-none"),
        None        => t("debug-blit-unknown"),
    };

    Grid::new("render_grid").num_columns(2).striped(true).show(ui, |ui| {
        ui.label(t("debug-blit-callback"));
        ui.label(RichText::new(blit_label).color(blit_color));
        ui.end_row();

        ui.label(t("debug-render-count"));
        ui.label(state.render_count.to_string());
        ui.end_row();

        ui.label(t("debug-blit-some-none"));
        ui.label(format!("{} / {}", state.blit_some_count, state.blit_none_count));
        ui.end_row();

        ui.label(t("debug-servo-spin"));
        ui.label(state.spin_count.to_string());
        ui.end_row();

        ui.label(t("debug-since-last-render"));
        let age = state.last_render
            .map(|t| format!("{:.0}ms", t.elapsed().as_millis()))
            .unwrap_or_else(|| "—".to_string());
        ui.label(age);
        ui.end_row();

        ui.label(t("debug-window-size"));
        ui.label(format!("{}×{}", state.window_size.0, state.window_size.1));
        ui.end_row();

        ui.label(t("debug-servo-size"));
        ui.label(format!("{}×{}", state.servo_size.0, state.servo_size.1));
        ui.end_row();

        ui.label(t("debug-scale-factor"));
        ui.label(format!("{:.2}", state.scale_factor));
        ui.end_row();

        ui.label(t("debug-egui-focus"));
        ui.label(RichText::new(&state.egui_focus_widget).monospace());
        ui.end_row();

        ui.label(t("debug-egui-consumed"));
        let consumed_color = if state.egui_consumed_last { Color32::YELLOW } else { Color32::GRAY };
        ui.label(RichText::new(state.egui_consumed_last.to_string()).color(consumed_color));
        ui.end_row();
    });
}

fn webview_section(ui: &mut egui::Ui, state: &DebugState) {
    Grid::new("webview_grid").num_columns(2).striped(true).show(ui, |ui| {
        ui.label(t("debug-load-status"));
        let color = match state.load_status.as_str() {
            "Complete"    => Color32::GREEN,
            "Loading"     => Color32::YELLOW,
            "Connecting"  => Color32::from_rgb(255, 165, 0),
            _             => Color32::GRAY,
        };
        ui.label(RichText::new(&state.load_status).color(color));
        ui.end_row();

        ui.label(t("debug-url"));
        ui.label(RichText::new(&state.current_url).monospace().small());
        ui.end_row();

        ui.label(t("debug-title"));
        ui.label(&state.page_title);
        ui.end_row();

        ui.label(t("debug-can-go-back"));
        ui.label(bool_label(state.can_go_back));
        ui.end_row();

        ui.label(t("debug-can-go-forward"));
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
