//! Global application paths and constants.

use std::path::PathBuf;
use std::sync::OnceLock;

static GLOBALS: OnceLock<Globals> = OnceLock::new();

/// Global application paths and constants.
#[derive(Debug, Clone)]
pub struct Globals {
    /// NucleusCoop installation root directory.
    pub install_root: PathBuf,
    /// User environment root (e.g. C:\Users\username).
    pub user_environment_root: PathBuf,
    /// User documents directory.
    pub user_documents_root: PathBuf,
    /// Application data directory (profiles, settings).
    pub app_data_path: PathBuf,
    /// Handlers directory.
    pub handlers_path: PathBuf,
    /// Utils directory (bundled tools).
    pub utils_path: PathBuf,
    /// GUI assets directory (covers, screenshots).
    pub gui_path: PathBuf,
}

impl Globals {
    /// Initializes global application paths with the given installation root.
    pub fn init(install_root: PathBuf) -> &'static Globals {
        GLOBALS.get_or_init(|| {
            let user_environment_root = dirs::home_dir().unwrap_or_default();
            let user_documents_root = dirs::document_dir().unwrap_or_default();
            let app_data_path = dirs::data_dir().unwrap_or_default().join("Nucleus Coop");
            let handlers_path = install_root.join("handlers");
            let utils_path = install_root.join("utils");
            let gui_path = install_root.join("gui");

            Globals {
                install_root,
                user_environment_root,
                user_documents_root,
                app_data_path,
                handlers_path,
                utils_path,
                gui_path,
            }
        })
    }

    /// Gets the global application paths instance.
    ///
    /// # Panics
    /// Panics if [`Globals::init`] has not been called.
    pub fn get() -> &'static Globals {
        GLOBALS
            .get()
            .expect("Globals not initialized. Call Globals::init() first.")
    }

    /// Gets the global application paths instance if initialized, or None.
    pub fn try_get() -> Option<&'static Globals> {
        GLOBALS.get()
    }
}
