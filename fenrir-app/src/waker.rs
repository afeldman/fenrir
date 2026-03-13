//! winit-basierter EventLoopWaker für Servo.
//!
//! Servo ruft `wake()` auf wenn es Arbeit hat (Netzwerkantwort, Layout, Animation).
//! Wir schicken ein User-Event in den winit Event-Loop um ihn aufzuwecken.

use servo::EventLoopWaker;
use winit::event_loop::EventLoopProxy;

#[derive(Clone, Debug)]
pub struct WakerEvent;

#[derive(Clone)]
pub struct FenrirWaker(EventLoopProxy<WakerEvent>);

impl FenrirWaker {
    pub fn new(proxy: EventLoopProxy<WakerEvent>) -> Self {
        Self(proxy)
    }
}

impl EventLoopWaker for FenrirWaker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(self.clone())
    }

    fn wake(&self) {
        // Fehler ignorieren — window könnte schon geschlossen sein
        let _ = self.0.send_event(WakerEvent);
    }
}
