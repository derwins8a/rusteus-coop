//! Platform services initialization and factory dispatch.

use std::sync::Arc;
use nucleus_platform::PlatformServices;

/// Creates concrete platform services based on target operating system.
pub fn create_platform_services() -> Arc<dyn PlatformServices> {
    #[cfg(windows)]
    {
        Arc::new(nucleus_platform_windows::WindowsPlatform::new())
    }

    #[cfg(target_os = "linux")]
    {
        Arc::new(nucleus_platform_linux::LinuxPlatform::new())
    }
}
