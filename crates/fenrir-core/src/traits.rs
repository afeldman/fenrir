use crate::error::FenrirResult;
use std::future::Future;
use std::pin::Pin;

/// Alle Fenrir-Module implementieren diesen Trait.
/// Ermöglicht einheitlichen Start/Stop-Lifecycle.
pub trait FenrirModule: Send + Sync {
    /// Modulname für Logging und Fehlerberichte.
    fn name(&self) -> &'static str;

    /// Modul hochfahren (async, kann fehlschlagen).
    fn start(&self) -> Pin<Box<dyn Future<Output = FenrirResult<()>> + Send>>;

    /// Modul sauber herunterfahren.
    fn stop(&self) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    /// Ist das Modul aktuell aktiv?
    fn is_running(&self) -> bool;
}

/// Typen die DSGVO-relevante Daten verwalten, implementieren diesen Trait.
pub trait Auditable {
    /// Beschreibung was dieser Typ an Daten hält und warum.
    fn audit_description(&self) -> &'static str;

    /// Alle gehaltenen Daten für DSGVO-Export serialisieren.
    fn export_data(&self) -> serde_json::Value;

    /// Alle Daten löschen (Recht auf Vergessenwerden).
    fn delete_all_data(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct TestModule {
        running: std::sync::atomic::AtomicBool,
    }

    impl TestModule {
        fn new() -> Self {
            Self {
                running: std::sync::atomic::AtomicBool::new(false),
            }
        }
    }

    impl FenrirModule for TestModule {
        fn name(&self) -> &'static str {
            "TestModule"
        }

        fn start(&self) -> Pin<Box<dyn Future<Output = FenrirResult<()>> + Send>> {
            let running = self.running.clone();
            Box::pin(async move {
                running.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            })
        }

        fn stop(&self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
            let running = self.running.clone();
            Box::pin(async move {
                running.store(false, std::sync::atomic::Ordering::SeqCst);
            })
        }

        fn is_running(&self) -> bool {
            self.running.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    struct TestAuditable {
        data: Vec<String>,
    }

    impl TestAuditable {
        fn new() -> Self {
            Self {
                data: vec!["test1".to_string(), "test2".to_string()],
            }
        }
    }

    impl Auditable for TestAuditable {
        fn audit_description(&self) -> &'static str {
            "Test data for auditing"
        }

        fn export_data(&self) -> serde_json::Value {
            json!({
                "data": self.data
            })
        }

        fn delete_all_data(&mut self) {
            self.data.clear();
        }
    }

    #[tokio::test]
    async fn test_fenrir_module_lifecycle() {
        let module = TestModule::new();
        assert_eq!(module.name(), "TestModule");
        assert!(!module.is_running());

        let result = module.start().await;
        assert!(result.is_ok());
        assert!(module.is_running());

        module.stop().await;
        assert!(!module.is_running());
    }

    #[test]
    fn test_auditable_trait() {
        let mut auditable = TestAuditable::new();
        assert_eq!(auditable.audit_description(), "Test data for auditing");

        let exported = auditable.export_data();
        assert_eq!(exported["data"].as_array().unwrap().len(), 2);

        auditable.delete_all_data();
        let exported_after_delete = auditable.export_data();
        assert_eq!(exported_after_delete["data"].as_array().unwrap().len(), 0);
    }
}
