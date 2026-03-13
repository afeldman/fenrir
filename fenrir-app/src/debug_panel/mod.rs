//! Fenrir Debug Panel — nur in Debug-Builds aktiv.
//!
//! Zeigt Echtzeit-Zustand der Rendering-Pipeline, Servo-Events,
//! Input-Events, egui-Fokus und Netzwerk-Anfragen.
//!
//! Toggle: F12

mod state;
mod widgets;

pub use state::{DebugState, InputEvent, ServoEvent};
pub use widgets::render;

use std::time::{Duration, Instant};

const MAX_EVENTS: usize = 40;
