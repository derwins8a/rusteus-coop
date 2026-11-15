//! Cross-platform gamepad detection and input management using gilrs.

use gilrs::Gilrs;
use nucleus_common::error::NucleusResult;
use nucleus_common::types::DeviceType;

/// Persistent device fingerprint string identifying a controller across re-plugs and port changes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DeviceFingerprint(pub String);

impl DeviceFingerprint {
    /// Creates a new device fingerprint from the device GUID, name, and type.
    pub fn new(guid: &str, name: &str, device_type: &DeviceType) -> Self {
        Self(format!("{}:{}:{:?}", guid.trim(), name.trim(), device_type))
    }

    /// Returns the fingerprint as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for DeviceFingerprint {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for DeviceFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for DeviceFingerprint {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for DeviceFingerprint {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for DeviceFingerprint {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Descriptor for an enumerated gamepad device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamepadDevice {
    /// Internal gilrs or assigned gamepad identifier.
    pub id: usize,
    /// Device name reported by the driver.
    pub name: String,
    /// Unique hardware GUID string.
    pub guid: String,
    /// Inferred device category (XInput, DInput, etc.).
    pub device_type: DeviceType,
    /// Whether the device is currently connected.
    pub is_connected: bool,
}

impl GamepadDevice {
    /// Generates a persistent fingerprint string for this device.
    ///
    /// The fingerprint is based on the hardware GUID, device name, and device type,
    /// ensuring that unplugging and reconnecting to a different USB port or with a
    /// different runtime numeric ID preserves the same device identity.
    pub fn fingerprint(&self) -> String {
        DeviceFingerprint::new(&self.guid, &self.name, &self.device_type).0
    }

    /// Returns the strongly-typed [`DeviceFingerprint`] for this device.
    pub fn device_fingerprint(&self) -> DeviceFingerprint {
        DeviceFingerprint::new(&self.guid, &self.name, &self.device_type)
    }
}

/// Manager responsible for enumerating and polling gamepad controllers.
pub struct GamepadManager {
    /// Gilrs context instance.
    gilrs: Option<Gilrs>,
    /// Cached list of discovered gamepad devices.
    devices: Vec<GamepadDevice>,
}

impl GamepadManager {
    /// Creates a new `GamepadManager` initializing the underlying gilrs context.
    pub fn new() -> NucleusResult<Self> {
        let gilrs = Gilrs::new().ok();
        let mut mgr = Self {
            gilrs,
            devices: Vec::new(),
        };
        mgr.refresh_devices();
        Ok(mgr)
    }

    /// Refreshes the internal device cache from gilrs.
    fn refresh_devices(&mut self) {
        if let Some(ref gilrs) = self.gilrs {
            for (id, gamepad) in gilrs.gamepads() {
                let numeric_id: usize = id.into();
                let dev = GamepadDevice {
                    id: numeric_id,
                    name: gamepad.name().to_string(),
                    guid: format!("{:?}", gamepad.uuid()),
                    device_type: DeviceType::XInputGamepad,
                    is_connected: gamepad.is_connected(),
                };
                if let Some(existing) = self.devices.iter_mut().find(|d| d.id == numeric_id) {
                    *existing = dev;
                } else {
                    self.devices.push(dev);
                }
            }
        }
    }

    /// Polls gamepad events from the system event queue and updates internal device states.
    pub fn poll_events(&mut self) -> NucleusResult<()> {
        if let Some(ref mut gilrs) = self.gilrs {
            while let Some(event) = gilrs.next_event() {
                let numeric_id: usize = event.id.into();
                match event.event {
                    gilrs::EventType::Connected => {
                        let gamepad = gilrs.gamepad(event.id);
                        let dev = GamepadDevice {
                            id: numeric_id,
                            name: gamepad.name().to_string(),
                            guid: format!("{:?}", gamepad.uuid()),
                            device_type: DeviceType::XInputGamepad,
                            is_connected: true,
                        };
                        if let Some(existing) = self.devices.iter_mut().find(|d| d.id == numeric_id) {
                            *existing = dev;
                        } else {
                            self.devices.push(dev);
                        }
                    }
                    gilrs::EventType::Disconnected => {
                        if let Some(existing) = self.devices.iter_mut().find(|d| d.id == numeric_id) {
                            existing.is_connected = false;
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Adds a mock gamepad device for testing.
    pub fn add_mock_device(&mut self, device: GamepadDevice) {
        self.devices.push(device);
    }

    /// Returns a slice of all currently detected gamepad devices.
    pub fn get_gamepads(&self) -> &[GamepadDevice] {
        &self.devices
    }

    /// Finds a gamepad device by its identifier.
    pub fn get_gamepad_by_id(&self, id: usize) -> Option<&GamepadDevice> {
        self.devices.iter().find(|d| d.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamepad_manager_lifecycle() {
        let mut mgr = GamepadManager::new().expect("Failed to initialize GamepadManager");
        let initial_count = mgr.get_gamepads().len();

        let mock = GamepadDevice {
            id: 999,
            name: "Virtual Xbox Controller".to_string(),
            guid: "xinput_mock".to_string(),
            device_type: DeviceType::XInputGamepad,
            is_connected: true,
        };
        mgr.add_mock_device(mock.clone());

        assert_eq!(mgr.get_gamepads().len(), initial_count + 1);
        let found = mgr.get_gamepad_by_id(999).expect("Should find mock gamepad");
        assert_eq!(found.name, "Virtual Xbox Controller");
        assert!(found.is_connected);

        mgr.poll_events().expect("Poll events should succeed");
    }
}
