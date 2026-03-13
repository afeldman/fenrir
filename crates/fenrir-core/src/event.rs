use crate::types::{FenrirUrl, TabId, Timestamp};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// Alle Browser-weiten Events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FenrirEvent {
    /// Tab wurde geöffnet.
    TabOpened { id: TabId, url: FenrirUrl, at: Timestamp },
    /// Tab wurde geschlossen.
    TabClosed { id: TabId, at: Timestamp },
    /// Navigation in einem Tab.
    TabNavigated { id: TabId, url: FenrirUrl, at: Timestamp },
    /// Tab-Titel geändert.
    TabTitleChanged { id: TabId, title: String },
    /// Netzwerkanfrage blockiert (Tracker, Phishing, etc.).
    RequestBlocked { tab: TabId, url: FenrirUrl, reason: BlockReason, at: Timestamp },
    /// Browser wird beendet.
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockReason {
    Tracker,
    Phishing,
    VpnKillSwitch,
    UserRule,
}

/// Kanal-Kapazität für den globalen Event-Bus.
const EVENT_BUS_CAPACITY: usize = 256;

/// Globaler Event-Bus (tokio broadcast).
pub struct EventBus {
    sender: broadcast::Sender<FenrirEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(EVENT_BUS_CAPACITY);
        Self { sender }
    }

    /// Event publizieren. Ignoriert Fehler wenn keine Subscriber.
    pub fn publish(&self, event: FenrirEvent) {
        let _ = self.sender.send(event);
    }

    /// Subscriber erstellen.
    pub fn subscribe(&self) -> broadcast::Receiver<FenrirEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[test]
    fn test_event_variants() {
        let tab_id = TabId::new();
        let url = FenrirUrl::parse("https://example.com").unwrap();
        let timestamp = Timestamp::now();

        let opened = FenrirEvent::TabOpened {
            id: tab_id,
            url: url.clone(),
            at: timestamp,
        };

        let closed = FenrirEvent::TabClosed {
            id: tab_id,
            at: timestamp,
        };

        let navigated = FenrirEvent::TabNavigated {
            id: tab_id,
            url: url.clone(),
            at: timestamp,
        };

        let title_changed = FenrirEvent::TabTitleChanged {
            id: tab_id,
            title: "New Title".to_string(),
        };

        let blocked = FenrirEvent::RequestBlocked {
            tab: tab_id,
            url,
            reason: BlockReason::Tracker,
            at: timestamp,
        };

        let shutdown = FenrirEvent::Shutdown;

        // Just ensure they can be created and debug printed
        println!("{:?}", opened);
        println!("{:?}", closed);
        println!("{:?}", navigated);
        println!("{:?}", title_changed);
        println!("{:?}", blocked);
        println!("{:?}", shutdown);
    }

    #[test]
    fn test_block_reason_variants() {
        let tracker = BlockReason::Tracker;
        let phishing = BlockReason::Phishing;
        let vpn = BlockReason::VpnKillSwitch;
        let user = BlockReason::UserRule;

        println!("{:?}", tracker);
        println!("{:?}", phishing);
        println!("{:?}", vpn);
        println!("{:?}", user);
    }

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let event_bus = EventBus::new();
        let mut receiver1 = event_bus.subscribe();
        let mut receiver2 = event_bus.subscribe();

        let tab_id = TabId::new();
        let url = FenrirUrl::parse("https://example.com").unwrap();
        let timestamp = Timestamp::now();

        let event = FenrirEvent::TabOpened {
            id: tab_id,
            url,
            at: timestamp,
        };

        event_bus.publish(event.clone());

        // Receiver 1 should get the event
        let received1 = timeout(Duration::from_millis(100), receiver1.recv()).await;
        assert!(received1.is_ok());
        let received_event1 = received1.unwrap();
        assert!(received_event1.is_ok());
        assert!(matches!(received_event1.unwrap(), FenrirEvent::TabOpened { .. }));

        // Receiver 2 should also get the event
        let received2 = timeout(Duration::from_millis(100), receiver2.recv()).await;
        assert!(received2.is_ok());
        let received_event2 = received2.unwrap();
        assert!(received_event2.is_ok());
        assert!(matches!(received_event2.unwrap(), FenrirEvent::TabOpened { .. }));
    }

    #[tokio::test]
    async fn test_event_bus_multiple_subscribers() {
        let event_bus = EventBus::new();
        let mut receivers = Vec::new();

        // Create 5 subscribers
        for _ in 0..5 {
            receivers.push(event_bus.subscribe());
        }

        let tab_id = TabId::new();
        let _url = FenrirUrl::parse("https://example.com").unwrap();
        let timestamp = Timestamp::now();

        let event = FenrirEvent::TabClosed {
            id: tab_id,
            at: timestamp,
        };

        event_bus.publish(event);

        // All 5 receivers should get the event
        for mut receiver in receivers {
            let received = timeout(Duration::from_millis(100), receiver.recv()).await;
            assert!(received.is_ok());
            let received_event = received.unwrap();
            assert!(received_event.is_ok());
            assert!(matches!(received_event.unwrap(), FenrirEvent::TabClosed { .. }));
        }
    }

    #[tokio::test]
    async fn test_event_bus_no_subscribers() {
        let event_bus = EventBus::new();

        // Publishing with no subscribers should not panic
        let event = FenrirEvent::Shutdown;
        event_bus.publish(event);

        // Now create a subscriber and publish again
        let mut receiver = event_bus.subscribe();
        let event2 = FenrirEvent::Shutdown;
        event_bus.publish(event2);

        let received = timeout(Duration::from_millis(100), receiver.recv()).await;
        assert!(received.is_ok());
        let received_event = received.unwrap();
        assert!(received_event.is_ok());
        assert!(matches!(received_event.unwrap(), FenrirEvent::Shutdown));
    }

    #[test]
    fn test_event_bus_default() {
        let event_bus = EventBus::default();
        let receiver = event_bus.subscribe();
        // Should be able to create a receiver without issues
        drop(receiver);
    }
}
