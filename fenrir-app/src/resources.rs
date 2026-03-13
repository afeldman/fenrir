//! Servo ResourceReader — findet und liest Servo-interne Ressourcen.
//!
//! Die resources/ liegen im fenrir-Workspace (fenrir/resources/) und werden
//! dort versioniert. Sie stammen ursprünglich aus dem Servo-Repo und werden
//! beim Servo-Upgrade manuell aktualisiert.

use std::path::PathBuf;
use std::sync::Mutex;
use std::{env, fs};

use servo::resources::{self, Resource, ResourceReaderMethods};

static RESOURCES_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

struct FenrirResourceReader;

/// Muss vor `ServoBuilder::build()` aufgerufen werden.
pub fn init() {
    resources::set(Box::new(FenrirResourceReader));
}

fn resources_dir() -> PathBuf {
    let mut cache = RESOURCES_DIR.lock().unwrap();
    if let Some(ref p) = *cache {
        return p.clone();
    }

    // 1. FENRIR_RESOURCES env-var (CI / Custom-Installations)
    if let Ok(path) = env::var("FENRIR_RESOURCES") {
        let p = PathBuf::from(path);
        if p.is_dir() {
            *cache = Some(p.clone());
            return p;
        }
    }

    // 2. resources/ relativ zum Executable (Release-Build / App Bundle)
    if let Ok(mut exe) = env::current_exe().and_then(|p| p.canonicalize()) {
        while exe.pop() {
            for name in &["resources", "Resources"] {
                let candidate = exe.join(name);
                if candidate.is_dir() {
                    *cache = Some(candidate.clone());
                    return candidate;
                }
            }
        }
    }

    // 3. resources/ relativ zum Arbeitsverzeichnis (dev: `cargo run` aus fenrir/)
    let cwd = env::current_dir().unwrap_or_default();
    let candidate = cwd.join("resources");
    if candidate.is_dir() {
        *cache = Some(candidate.clone());
        return candidate;
    }

    panic!(
        "Servo resources/ Verzeichnis nicht gefunden.\n\
         Optionen:\n\
         - `cargo run` aus dem fenrir/ Workspace-Root starten\n\
         - FENRIR_RESOURCES=/pfad/zu/resources setzen"
    )
}

impl ResourceReaderMethods for FenrirResourceReader {
    fn read(&self, res: Resource) -> Vec<u8> {
        let path = resources_dir().join(res.filename());
        fs::read(&path).unwrap_or_else(|e| panic!("Resource {:?} nicht lesbar: {e}", path))
    }

    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
        vec![resources_dir()]
    }

    fn sandbox_access_files(&self) -> Vec<PathBuf> {
        vec![]
    }
}
