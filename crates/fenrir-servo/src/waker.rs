//! EventLoopWaker: verbindet Servo's internen Wakeup-Mechanismus mit tokio.
//!
//! Servo ruft `wake()` auf wenn es Arbeit hat (Netzwerkantwort, Animation, etc.).
//! Wir leiten das an eine tokio `Notify` weiter, die FenrirHost::wait_for_work()
//! aufweckt.

use servo::EventLoopWaker;
use std::sync::Arc;
use tokio::sync::Notify;

pub struct FenrirEventLoopWaker {
    notify: Arc<Notify>,
}

impl FenrirEventLoopWaker {
    pub fn new(notify: Arc<Notify>) -> Self {
        Self { notify }
    }
}

impl EventLoopWaker for FenrirEventLoopWaker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(Self {
            notify: Arc::clone(&self.notify),
        })
    }

    fn wake(&self) {
        self.notify.notify_one();
    }
}
