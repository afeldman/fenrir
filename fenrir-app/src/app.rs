//! winit ApplicationHandler — Haupt-Event-Loop des Fenrir Browsers.

use std::rc::Rc;

use url::Url;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::NamedKey;

use crate::browser::BrowserState;
use crate::waker::{FenrirWaker, WakerEvent};

pub enum BrowserApp {
    Initial(FenrirWaker),
    Running(Rc<BrowserState>),
}

impl BrowserApp {
    pub fn new(waker: FenrirWaker) -> Self {
        Self::Initial(waker)
    }
}

impl ApplicationHandler<WakerEvent> for BrowserApp {
    /// Fenster erstellen wenn Plattform bereit ist.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Self::Initial(waker) = self {
            let state = BrowserState::new(event_loop, waker.clone());
            *self = Self::Running(state);
        }
    }

    /// Servo hat Arbeit — Event-Loop pumpen.
    fn user_event(&mut self, _: &ActiveEventLoop, _: WakerEvent) {
        if let Self::Running(state) = self {
            state.spin();
            state.window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Self::Running(state) = self else { return };

        // Servo bei jedem Event pumpen
        state.spin();

        // egui bekommt Events zuerst (für Toolbar-Interaktion)
        let egui_consumed = state.on_window_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                state.render();
            }

            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    state.resize(new_size);
                    state.window.request_redraw();
                }
            }

            // ── Maus ──────────────────────────────────────────────────────────
            WindowEvent::CursorMoved { position, .. } if !egui_consumed => {
                state.handle_cursor_moved(position);
            }

            WindowEvent::MouseInput { button, state: btn_state, .. } if !egui_consumed => {
                state.handle_mouse_button(button, btn_state);
            }

            WindowEvent::MouseWheel { delta, .. } if !egui_consumed => {
                state.handle_scroll(delta);
            }

            // ── Tastatur ──────────────────────────────────────────────────────
            WindowEvent::ModifiersChanged(mods) => {
                state.set_modifiers(mods.state());
            }

            WindowEvent::KeyboardInput { event: key_event, .. }
                if key_event.state == ElementState::Pressed =>
            {
                if egui_consumed {
                    return;
                }

                let mods = state.get_modifiers();
                let ctrl_or_cmd = mods.control_key() || mods.super_key();

                // Browser-Shortcuts
                if ctrl_or_cmd {
                    if let winit::keyboard::Key::Character(c) = &key_event.logical_key {
                        match c.as_str() {
                            "l" | "L" => {
                                state.toolbar.borrow_mut().focus_url_bar();
                                state.window.request_redraw();
                                return;
                            }
                            "r" | "R" => { state.reload(); return; }
                            _ => {}
                        }
                    }
                }

                if let winit::keyboard::Key::Named(named) = &key_event.logical_key {
                    match named {
                        NamedKey::F5 => { state.reload(); return; }
                        NamedKey::F12 => { state.toggle_debug(); state.window.request_redraw(); return; }
                        NamedKey::BrowserBack => { state.go_back(); return; }
                        NamedKey::BrowserForward => { state.go_forward(); return; }
                        NamedKey::BrowserRefresh => { state.reload(); return; }
                        _ => {}
                    }
                }

                if mods.alt_key() {
                    if let winit::keyboard::Key::Named(named) = &key_event.logical_key {
                        match named {
                            NamedKey::ArrowLeft  => { state.go_back(); return; }
                            NamedKey::ArrowRight => { state.go_forward(); return; }
                            _ => {}
                        }
                    }
                }

                // Rest an Servo
                state.handle_keyboard(&key_event);
            }

            _ => {}
        }
    }
}
