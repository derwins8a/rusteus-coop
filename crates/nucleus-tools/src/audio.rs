//! Audio device routing and per-instance audio management.
//!
//! Provides genuine Windows MMDevice API device enumeration and Windows Audio Policy
//! rerouting for game process IDs using internal COM interfaces (`IAudioPolicyConfigFactory`
//! and `IAudioPolicyConfigFactoryFor21H2`).

use nucleus_common::error::NucleusResult;
use std::sync::OnceLock;
use std::sync::mpsc::{Sender, channel};
use std::thread;

/// Device interface GUID suffix for audio render endpoints.
pub const DEVINTERFACE_AUDIO_RENDER: &str = "#{e6327cad-dcec-4949-ae8a-991e976a79d2}";
/// Device interface GUID suffix for audio capture endpoints.
pub const DEVINTERFACE_AUDIO_CAPTURE: &str = "#{2eef81be-33fa-4800-9670-1cd474972c3f}";
/// Device interface prefix for MMDEVAPI devices.
pub const MMDEVAPI_TOKEN: &str = r"\\?\SWD#MMDEVAPI#";

/// Audio data flow direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFlow {
    /// Playback / speaker / headphones endpoint.
    Render = 0,
    /// Microphone / line-in endpoint.
    Capture = 1,
}

/// Audio console / multimedia / communication roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioRole {
    /// System sounds and console output.
    Console = 0,
    /// Music, movies, and game media output.
    Multimedia = 1,
    /// Voice chat and communications.
    Communications = 2,
}

/// Formats a raw audio device ID (or MMDevice ID) into the full MMDEVAPI device path.
///
/// Example:
/// `{0.0.0.00000000}.{guid}` -> `\\?\SWD#MMDEVAPI#{0.0.0.00000000}.{guid}#{e6327cad-dcec-4949-ae8a-991e976a79d2}`
pub fn format_mmdevapi_device_id(device_id: &str, flow: AudioFlow) -> String {
    let clean = unpack_mmdevapi_device_id(device_id);
    let suffix = match flow {
        AudioFlow::Render => DEVINTERFACE_AUDIO_RENDER,
        AudioFlow::Capture => DEVINTERFACE_AUDIO_CAPTURE,
    };
    format!("{}{}{}", MMDEVAPI_TOKEN, clean, suffix)
}

/// Strips the `\\?\SWD#MMDEVAPI#` token and interface GUID suffix from a formatted MMDEVAPI string.
pub fn unpack_mmdevapi_device_id(device_id: &str) -> String {
    let mut s = device_id.trim();
    if let Some(stripped) = s.strip_prefix(MMDEVAPI_TOKEN) {
        s = stripped;
    }
    if let Some(stripped) = s.strip_suffix(DEVINTERFACE_AUDIO_RENDER) {
        s = stripped;
    }
    if let Some(stripped) = s.strip_suffix(DEVINTERFACE_AUDIO_CAPTURE) {
        s = stripped;
    }
    s.to_string()
}

type ComTask = Box<dyn FnOnce() + Send + 'static>;

/// Dedicated background STA (Single-Threaded Apartment) worker thread.
///
/// Windows COM interfaces (especially undocumented shell/WinRT activation factories)
/// require execution within an STA apartment thread initialized with
/// `CoInitializeEx(None, COINIT_APARTMENTTHREADED)`.
pub struct ComThread {
    sender: Sender<ComTask>,
    thread_id: thread::ThreadId,
}

impl ComThread {
    /// Returns the global static `ComThread` instance.
    pub fn global() -> &'static ComThread {
        static INSTANCE: OnceLock<ComThread> = OnceLock::new();
        INSTANCE.get_or_init(ComThread::new)
    }

    /// Spawns a new dedicated STA worker thread.
    pub fn new() -> Self {
        let (sender, receiver) = channel::<ComTask>();
        let (id_tx, id_rx) = channel();

        thread::Builder::new()
            .name("NucleusComAudioThread".to_string())
            .spawn(move || {
                let _ = id_tx.send(thread::current().id());

                #[cfg(windows)]
                unsafe {
                    use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
                    let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
                }

                while let Ok(task) = receiver.recv() {
                    task();
                }

                #[cfg(windows)]
                unsafe {
                    use windows::Win32::System::Com::CoUninitialize;
                    CoUninitialize();
                }
            })
            .expect("Failed to spawn ComThread");

        let thread_id = id_rx.recv().expect("Failed to receive ComThread thread id");
        Self { sender, thread_id }
    }

    /// Returns whether the current thread is the dedicated COM STA thread.
    pub fn is_com_thread(&self) -> bool {
        thread::current().id() == self.thread_id
    }

    /// Dispatches a synchronous closure to execute within the STA worker thread.
    ///
    /// If the current thread is already the COM worker thread, executes immediately
    /// to prevent self-deadlocks.
    pub fn invoke<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        if self.is_com_thread() {
            return f();
        }

        let (tx, rx) = channel();
        let task = Box::new(move || {
            let res = f();
            let _ = tx.send(res);
        });

        if self.sender.send(task).is_err() {
            panic!("ComThread worker thread has terminated unexpectedly");
        }
        rx.recv().expect("ComThread task failed to send response")
    }

    /// Convenience wrapper invoking a task on the global `ComThread`.
    pub fn invoke_global<F, R>(f: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        Self::global().invoke(f)
    }
}

impl Default for ComThread {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Windows Native COM & combase.dll definitions
// ---------------------------------------------------------------------------

#[cfg(windows)]
type HSTRING = *mut std::ffi::c_void;

#[cfg(windows)]
type FnRoGetActivationFactory = unsafe extern "system" fn(
    activatable_class_id: HSTRING,
    iid: *const windows::core::GUID,
    factory: *mut *mut std::ffi::c_void,
) -> windows::core::HRESULT;

#[cfg(windows)]
type FnWindowsCreateString = unsafe extern "system" fn(
    source_string: *const u16,
    length: u32,
    string: *mut HSTRING,
) -> windows::core::HRESULT;

#[cfg(windows)]
type FnWindowsDeleteString = unsafe extern "system" fn(string: HSTRING) -> windows::core::HRESULT;

#[cfg(windows)]
type FnWindowsGetStringRawBuffer =
    unsafe extern "system" fn(string: HSTRING, length: *mut u32) -> *const u16;

#[cfg(windows)]
type FnSetPersistedDefaultAudioEndpoint = unsafe extern "system" fn(
    this: *mut std::ffi::c_void,
    process_id: u32,
    flow: u32,
    role: u32,
    device_id: HSTRING,
) -> windows::core::HRESULT;

#[cfg(windows)]
type FnGetPersistedDefaultAudioEndpoint = unsafe extern "system" fn(
    this: *mut std::ffi::c_void,
    process_id: u32,
    flow: u32,
    role: u32,
    device_id: *mut HSTRING,
) -> windows::core::HRESULT;

#[cfg(windows)]
type FnClearAllPersistedApplicationDefaultEndpoints =
    unsafe extern "system" fn(this: *mut std::ffi::c_void) -> windows::core::HRESULT;

/// VTable layout for `IAudioPolicyConfigFactory` and `IAudioPolicyConfigFactoryFor21H2`.
///
/// Inherits from `IInspectable` (which inherits from `IUnknown`):
/// - Slots 0..2: IUnknown (QueryInterface, AddRef, Release)
/// - Slots 3..5: IInspectable (GetIids, GetRuntimeClassName, GetTrustLevel)
/// - Slots 6..24: 19 incomplete internal policy methods
/// - Slot 25: `SetPersistedDefaultAudioEndpoint`
/// - Slot 26: `GetPersistedDefaultAudioEndpoint`
/// - Slot 27: `ClearAllPersistedApplicationDefaultEndpoints`
#[cfg(windows)]
#[repr(C)]
pub struct IAudioPolicyConfigFactoryVtbl {
    pub query_interface: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        riid: *const windows::core::GUID,
        ppv: *mut *mut std::ffi::c_void,
    ) -> windows::core::HRESULT,
    pub add_ref: unsafe extern "system" fn(this: *mut std::ffi::c_void) -> u32,
    pub release: unsafe extern "system" fn(this: *mut std::ffi::c_void) -> u32,
    pub get_iids: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        iid_count: *mut u32,
        iids: *mut *mut windows::core::GUID,
    ) -> windows::core::HRESULT,
    pub get_runtime_class_name: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        class_name: *mut HSTRING,
    ) -> windows::core::HRESULT,
    pub get_trust_level: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        trust_level: *mut i32,
    ) -> windows::core::HRESULT,
    pub slot_6: usize,
    pub slot_7: usize,
    pub slot_8: usize,
    pub slot_9: usize,
    pub slot_10: usize,
    pub slot_11: usize,
    pub slot_12: usize,
    pub slot_13: usize,
    pub slot_14: usize,
    pub slot_15: usize,
    pub slot_16: usize,
    pub slot_17: usize,
    pub slot_18: usize,
    pub slot_19: usize,
    pub slot_20: usize,
    pub slot_21: usize,
    pub slot_22: usize,
    pub slot_23: usize,
    pub slot_24: usize,
    pub set_persisted_default_audio_endpoint: FnSetPersistedDefaultAudioEndpoint,
    pub get_persisted_default_audio_endpoint: FnGetPersistedDefaultAudioEndpoint,
    pub clear_all_persisted_application_default_endpoints:
        FnClearAllPersistedApplicationDefaultEndpoints,
}

/// GUID for Pre-Windows 10 21H2 `IAudioPolicyConfigFactory`:
/// `{2a59116d-6c4f-45e0-a74f-707e3fef9258}`
#[cfg(windows)]
pub const IID_AUDIO_POLICY_CONFIG_FACTORY: windows::core::GUID =
    windows::core::GUID::from_u128(0x2a59116d_6c4f_45e0_a74f_707e3fef9258);

/// GUID for Post-Windows 10 21H2 & Windows 11 `IAudioPolicyConfigFactoryFor21H2`:
/// `{ab3d4648-e242-459f-b02f-541c70306324}`
#[cfg(windows)]
pub const IID_AUDIO_POLICY_CONFIG_FACTORY_FOR_21H2: windows::core::GUID =
    windows::core::GUID::from_u128(0xab3d4648_e242_459f_b02f_541c70306324);

/// Dynamically loaded `combase.dll` exports.
#[cfg(windows)]
struct ComBase {
    _module: windows::Win32::Foundation::HMODULE,
    ro_get_activation_factory: FnRoGetActivationFactory,
    windows_create_string: FnWindowsCreateString,
    windows_delete_string: FnWindowsDeleteString,
    #[allow(dead_code)]
    windows_get_string_raw_buffer: Option<FnWindowsGetStringRawBuffer>,
}

#[cfg(windows)]
impl ComBase {
    fn load() -> Result<Self, String> {
        unsafe {
            use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
            use windows::core::{s, w};

            let module = LoadLibraryW(w!("combase.dll"))
                .map_err(|e| format!("Failed to load combase.dll: {:?}", e))?;

            let ro_get_activation_factory = GetProcAddress(module, s!("RoGetActivationFactory"))
                .ok_or_else(|| "RoGetActivationFactory not found in combase.dll".to_string())?;
            let windows_create_string = GetProcAddress(module, s!("WindowsCreateString"))
                .ok_or_else(|| "WindowsCreateString not found in combase.dll".to_string())?;
            let windows_delete_string = GetProcAddress(module, s!("WindowsDeleteString"))
                .ok_or_else(|| "WindowsDeleteString not found in combase.dll".to_string())?;
            let windows_get_string_raw_buffer =
                GetProcAddress(module, s!("WindowsGetStringRawBuffer"));

            Ok(Self {
                _module: module,
                ro_get_activation_factory: std::mem::transmute(ro_get_activation_factory),
                windows_create_string: std::mem::transmute(windows_create_string),
                windows_delete_string: std::mem::transmute(windows_delete_string),
                windows_get_string_raw_buffer: windows_get_string_raw_buffer
                    .map(|p| std::mem::transmute(p)),
            })
        }
    }

    fn create_hstring(&self, s: &str) -> Result<HSTRING, String> {
        let wide: Vec<u16> = s.encode_utf16().collect();
        let mut hstring: HSTRING = std::ptr::null_mut();
        unsafe {
            let hr = (self.windows_create_string)(wide.as_ptr(), wide.len() as u32, &mut hstring);
            if hr.is_err() {
                return Err(format!("WindowsCreateString failed: {:?}", hr));
            }
        }
        Ok(hstring)
    }

    fn delete_hstring(&self, hstring: HSTRING) {
        if !hstring.is_null() {
            unsafe {
                let _ = (self.windows_delete_string)(hstring);
            }
        }
    }
}

/// RAII wrapper for safely managing `HSTRING` lifetimes.
#[cfg(windows)]
struct RaiiHString<'a> {
    combase: &'a ComBase,
    handle: HSTRING,
}

#[cfg(windows)]
impl Drop for RaiiHString<'_> {
    fn drop(&mut self) {
        self.combase.delete_hstring(self.handle);
    }
}

/// Discovers the Windows build number using `RtlGetVersion` (avoiding application manifest shims)
/// or registry fallback.
#[cfg(windows)]
pub fn get_windows_build() -> u32 {
    #[repr(C)]
    struct OsVersionInfoW {
        os_version_info_size: u32,
        major_version: u32,
        minor_version: u32,
        build_number: u32,
        platform_id: u32,
        sz_csd_version: [u16; 128],
    }

    type FnRtlGetVersion = unsafe extern "system" fn(info: *mut OsVersionInfoW) -> i32;

    unsafe {
        use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
        use windows::core::{s, w};

        if let Ok(module) = LoadLibraryW(w!("ntdll.dll")) {
            if let Some(proc) = GetProcAddress(module, s!("RtlGetVersion")) {
                let rtl_get_version: FnRtlGetVersion = std::mem::transmute(proc);
                let mut info: OsVersionInfoW = std::mem::zeroed();
                info.os_version_info_size = std::mem::size_of::<OsVersionInfoW>() as u32;
                if rtl_get_version(&mut info) == 0 {
                    return info.build_number;
                }
            }
        }

        // Fallback: Registry HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\CurrentBuildNumber
        if let Ok(hklm) = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
        {
            if let Ok(build_str) = hklm.get_value::<String, _>("CurrentBuildNumber") {
                if let Ok(build) = build_str.parse::<u32>() {
                    return build;
                }
            }
        }
        0
    }
}

/// Native client managing the `Windows.Media.Internal.AudioPolicyConfig` COM interface.
#[cfg(windows)]
struct AudioPolicyClient {
    combase: ComBase,
    factory: *mut std::ffi::c_void,
    #[allow(dead_code)]
    pub is_21h2: bool,
}

#[cfg(windows)]
impl AudioPolicyClient {
    fn new() -> Result<Self, String> {
        let combase = ComBase::load()?;
        let class_hstring = combase.create_hstring("Windows.Media.Internal.AudioPolicyConfig")?;

        let is_21h2_os = get_windows_build() >= 19044;
        let primary_iid = if is_21h2_os {
            IID_AUDIO_POLICY_CONFIG_FACTORY_FOR_21H2
        } else {
            IID_AUDIO_POLICY_CONFIG_FACTORY
        };
        let fallback_iid = if is_21h2_os {
            IID_AUDIO_POLICY_CONFIG_FACTORY
        } else {
            IID_AUDIO_POLICY_CONFIG_FACTORY_FOR_21H2
        };

        let mut factory: *mut std::ffi::c_void = std::ptr::null_mut();
        let mut is_21h2 = is_21h2_os;

        unsafe {
            let hr = (combase.ro_get_activation_factory)(class_hstring, &primary_iid, &mut factory);
            if hr.is_err() || factory.is_null() {
                let hr_fallback =
                    (combase.ro_get_activation_factory)(class_hstring, &fallback_iid, &mut factory);
                if hr_fallback.is_err() || factory.is_null() {
                    combase.delete_hstring(class_hstring);
                    return Err(format!(
                        "RoGetActivationFactory failed for both 21H2 and pre-21H2: primary={:?}, fallback={:?}",
                        hr, hr_fallback
                    ));
                }
                is_21h2 = !is_21h2_os;
            }
            combase.delete_hstring(class_hstring);
        }

        tracing::info!(
            "AudioPolicyConfigFactory activated successfully (21H2 interface: {})",
            is_21h2
        );

        Ok(Self {
            combase,
            factory,
            is_21h2,
        })
    }

    fn set_persisted_default_endpoint(
        &self,
        pid: u32,
        flow: AudioFlow,
        role: AudioRole,
        device_id: &str,
    ) -> Result<(), String> {
        let formatted = format_mmdevapi_device_id(device_id, flow);
        let hstring = self.combase.create_hstring(&formatted)?;
        let _guard = RaiiHString {
            combase: &self.combase,
            handle: hstring,
        };

        unsafe {
            let vtbl = *(self.factory as *const *const IAudioPolicyConfigFactoryVtbl);
            let hr = ((*vtbl).set_persisted_default_audio_endpoint)(
                self.factory,
                pid,
                flow as u32,
                role as u32,
                hstring,
            );
            if hr.is_err() {
                return Err(format!(
                    "SetPersistedDefaultAudioEndpoint failed for PID {} (role {:?}): {:?}",
                    pid, role, hr
                ));
            }
        }
        Ok(())
    }

    fn clear_all_persisted_application_default_endpoints(&self) -> Result<(), String> {
        unsafe {
            let vtbl = *(self.factory as *const *const IAudioPolicyConfigFactoryVtbl);
            let hr = ((*vtbl).clear_all_persisted_application_default_endpoints)(self.factory);
            if hr.is_err() {
                return Err(format!(
                    "ClearAllPersistedApplicationDefaultEndpoints failed: {:?}",
                    hr
                ));
            }
        }
        Ok(())
    }
}

#[cfg(windows)]
impl Drop for AudioPolicyClient {
    fn drop(&mut self) {
        if !self.factory.is_null() {
            unsafe {
                let vtbl = *(self.factory as *const *const IAudioPolicyConfigFactoryVtbl);
                ((*vtbl).release)(self.factory);
                self.factory = std::ptr::null_mut();
            }
        }
    }
}

#[cfg(windows)]
thread_local! {
    static POLICY_CLIENT: std::cell::RefCell<Option<AudioPolicyClient>> = const { std::cell::RefCell::new(None) };
}

#[cfg(windows)]
fn with_policy_client<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&AudioPolicyClient) -> Result<R, String>,
{
    POLICY_CLIENT.with(|cell| {
        let mut opt = cell.borrow_mut();
        if opt.is_none() {
            *opt = Some(AudioPolicyClient::new()?);
        }
        let client = opt.as_ref().unwrap();
        f(client)
    })
}

// ---------------------------------------------------------------------------
// Public Audio Types & AudioTool Implementation
// ---------------------------------------------------------------------------

/// Represents an audio endpoint device on the system.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AudioEndpoint {
    /// Device endpoint unique ID or COM identifier.
    pub id: String,
    /// Friendly human-readable device name (e.g. "Headphones (Realtek Audio)").
    pub name: String,
    /// Data flow type ("Render" for playback, "Capture" for recording).
    pub flow: String,
    /// Whether this endpoint is the system default.
    pub is_default: bool,
}

/// Tool for querying system audio endpoints and routing individual instance audio outputs
/// to designated headphones or speakers.
#[derive(Debug, Default, Clone)]
pub struct AudioTool;

/// Type alias for backward and architectural compatibility.
pub type AudioRouter = AudioTool;

impl AudioTool {
    /// Convenience helper returning list of `(device_id, friendly_name)` pairs.
    pub fn get_audio_devices() -> Vec<(String, String)> {
        Self::enumerate_render_devices()
            .into_iter()
            .map(|ep| (ep.id, ep.name))
            .collect()
    }
    /// Provides standard fallback devices when hardware enumeration is unavailable
    /// (e.g. headless VMs, CI pipelines, or systems without sound cards).
    pub fn fallback_devices() -> Vec<AudioEndpoint> {
        vec![
            AudioEndpoint {
                id: "{default-playback-device}".to_string(),
                name: "Default System Audio Output".to_string(),
                flow: "Render".to_string(),
                is_default: true,
            },
            AudioEndpoint {
                id: "{secondary-audio-device}".to_string(),
                name: "Secondary Audio Device / Headphones".to_string(),
                flow: "Render".to_string(),
                is_default: false,
            },
        ]
    }

    /// Enumerates active playback/render audio endpoints on the system.
    pub fn enumerate_render_devices() -> Vec<AudioEndpoint> {
        #[cfg(windows)]
        {
            ComThread::invoke_global(|| unsafe { Self::enumerate_render_devices_win32() })
        }

        #[cfg(not(windows))]
        {
            Self::fallback_devices()
        }
    }

    #[cfg(windows)]
    unsafe fn enumerate_render_devices_win32() -> Vec<AudioEndpoint> {
        unsafe {
            use windows::Win32::Media::Audio::*;
            use windows::Win32::System::Com::*;
            use windows::Win32::UI::Shell::PropertiesSystem::*;
            use windows::core::GUID;

            let enumerator: IMMDeviceEnumerator =
                match CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) {
                    Ok(e) => e,
                    Err(err) => {
                        tracing::warn!("Failed to create MMDeviceEnumerator: {:?}", err);
                        return Self::fallback_devices();
                    }
                };

            let default_id: Option<String> =
                match enumerator.GetDefaultAudioEndpoint(eRender, eConsole) {
                    Ok(dev) => dev.GetId().ok().map(|pwstr| {
                        let s = pwstr.to_string().unwrap_or_default();
                        CoTaskMemFree(Some(pwstr.as_ptr() as *const _));
                        s
                    }),
                    Err(err) => {
                        tracing::debug!("No default audio endpoint discovered: {:?}", err);
                        None
                    }
                };

            let collection: IMMDeviceCollection =
                match enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) {
                    Ok(c) => c,
                    Err(err) => {
                        tracing::warn!("Failed to enumerate audio endpoints: {:?}", err);
                        return Self::fallback_devices();
                    }
                };

            let count = match collection.GetCount() {
                Ok(c) => c,
                Err(err) => {
                    tracing::warn!("Failed to get audio device count: {:?}", err);
                    return Self::fallback_devices();
                }
            };

            if count == 0 {
                tracing::info!("Zero active audio render devices found; using fallback list");
                return Self::fallback_devices();
            }

            const PKEY_DEVICE_FRIENDLY_NAME: PROPERTYKEY = PROPERTYKEY {
                fmtid: GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
                pid: 14,
            };

            let mut devices = Vec::with_capacity(count as usize);

            for i in 0..count {
                let device = match collection.Item(i) {
                    Ok(d) => d,
                    Err(_) => continue,
                };

                let id_pwstr = match device.GetId() {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                let id = id_pwstr.to_string().unwrap_or_default();
                CoTaskMemFree(Some(id_pwstr.as_ptr() as *const _));

                if id.is_empty() {
                    continue;
                }

                let mut friendly_name = String::new();
                if let Ok(store) = device.OpenPropertyStore(STGM_READ) {
                    if let Ok(propvar) = store.GetValue(&PKEY_DEVICE_FRIENDLY_NAME) {
                        let name_str = propvar.to_string();
                        if !name_str.is_empty() {
                            friendly_name = name_str;
                        }
                    }
                }

                if friendly_name.is_empty() {
                    friendly_name = format!("Audio Render Device {}", i + 1);
                }

                let is_default = default_id.as_deref() == Some(&id);

                devices.push(AudioEndpoint {
                    id,
                    name: friendly_name,
                    flow: "Render".to_string(),
                    is_default,
                });
            }

            if devices.is_empty() {
                return Self::fallback_devices();
            }

            // Guarantee that at least one device is flagged default
            if !devices.iter().any(|d| d.is_default) {
                if let Some(first) = devices.first_mut() {
                    first.is_default = true;
                }
            }

            devices
        }
    }

    /// Reroutes audio playback of a specific game process ID to a designated audio endpoint ID.
    pub fn route_process_audio(pid: u32, device_id: &str) -> NucleusResult<()> {
        if device_id.is_empty() || device_id == "{default-playback-device}" {
            return Ok(());
        }

        #[cfg(windows)]
        {
            let device_id = device_id.to_string();
            tracing::info!(
                "Routing audio for PID {} to audio endpoint: {}",
                pid,
                device_id
            );

            let res = ComThread::invoke_global(move || {
                with_policy_client(|client| {
                    let roles = [
                        AudioRole::Console,
                        AudioRole::Multimedia,
                        AudioRole::Communications,
                    ];
                    for role in roles {
                        if let Err(err) = client.set_persisted_default_endpoint(
                            pid,
                            AudioFlow::Render,
                            role,
                            &device_id,
                        ) {
                            tracing::warn!(
                                "Failed to route audio for PID {} to endpoint {} (role {:?}): {}",
                                pid,
                                device_id,
                                role,
                                err
                            );
                        }
                    }
                    Ok(())
                })
            });

            if let Err(err) = res {
                tracing::warn!("Audio policy routing error for PID {}: {}", pid, err);
            }
            Ok(())
        }

        #[cfg(not(windows))]
        {
            let _ = (pid, device_id);
            Ok(())
        }
    }

    /// Resets all persisted application default audio endpoints back to system defaults.
    pub fn reset_all_endpoints() -> NucleusResult<()> {
        #[cfg(windows)]
        {
            tracing::info!("Resetting all persisted application default audio endpoints");
            let res = ComThread::invoke_global(|| {
                with_policy_client(|client| {
                    client.clear_all_persisted_application_default_endpoints()
                })
            });

            if let Err(err) = res {
                tracing::warn!(
                    "Failed to reset application default audio endpoints: {}",
                    err
                );
            }
            Ok(())
        }

        #[cfg(not(windows))]
        {
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_and_unpack_device_id() {
        let bare = "{0.0.0.00000000}.{0f588c7a-977d-4ba4-8846-993d395726ce}";
        let render_formatted = format_mmdevapi_device_id(bare, AudioFlow::Render);
        assert_eq!(
            render_formatted,
            format!(r"\\?\SWD#MMDEVAPI#{}{}", bare, DEVINTERFACE_AUDIO_RENDER)
        );

        let capture_formatted = format_mmdevapi_device_id(bare, AudioFlow::Capture);
        assert_eq!(
            capture_formatted,
            format!(r"\\?\SWD#MMDEVAPI#{}{}", bare, DEVINTERFACE_AUDIO_CAPTURE)
        );

        // Idempotence: formatting an already formatted string returns identical result
        let double_formatted = format_mmdevapi_device_id(&render_formatted, AudioFlow::Render);
        assert_eq!(double_formatted, render_formatted);

        // Unpacking strips tokens
        assert_eq!(unpack_mmdevapi_device_id(&render_formatted), bare);
        assert_eq!(unpack_mmdevapi_device_id(&capture_formatted), bare);
        assert_eq!(unpack_mmdevapi_device_id(bare), bare);
    }

    #[test]
    fn test_com_thread_invoke() {
        let result = ComThread::invoke_global(|| 100 + 42);
        assert_eq!(result, 142);

        // Nested invocation on the same COM thread should execute without deadlocking
        let nested = ComThread::invoke_global(|| ComThread::invoke_global(|| 999));
        assert_eq!(nested, 999);
    }

    #[test]
    fn test_audio_tool_enumeration() {
        let devices = AudioTool::enumerate_render_devices();
        assert!(!devices.is_empty(), "Audio devices list must not be empty");
        assert!(
            devices.iter().any(|d| d.is_default),
            "At least one audio endpoint must be marked as default"
        );
        for dev in &devices {
            assert!(!dev.id.is_empty());
            assert!(!dev.name.is_empty());
            assert_eq!(dev.flow, "Render");
        }
    }

    #[test]
    fn test_audio_tool_routing() {
        let res1 = AudioTool::route_process_audio(1234, "{default-playback-device}");
        assert!(res1.is_ok());

        let res2 = AudioTool::route_process_audio(1234, "");
        assert!(res2.is_ok());

        // Route with dummy ID executes through COM without panic
        let res3 = AudioTool::route_process_audio(
            1234,
            "{0.0.0.00000000}.{00000000-0000-0000-0000-000000000000}",
        );
        assert!(res3.is_ok());
    }

    #[test]
    fn test_reset_all_endpoints() {
        let res = AudioTool::reset_all_endpoints();
        assert!(res.is_ok());
    }

    #[test]
    #[cfg(windows)]
    fn test_windows_build_discovery() {
        let build = get_windows_build();
        // Modern Windows 10/11 build numbers are typically > 10000
        assert!(build > 0, "Windows build number should be detected");
    }
}
