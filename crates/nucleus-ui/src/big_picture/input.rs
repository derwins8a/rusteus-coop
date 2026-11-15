//! Gamepad input polling and couch action dispatching.
//!
//! Translates analog thumbsticks, D-pads, and controller buttons into standard GPUI
//! focus and navigation actions with configurable deadzones, debounce intervals, and repeat timers.

use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Cardinal direction for thumbstick or D-pad deflection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StickDirection {
    /// Upwards deflection (positive Y).
    Up,
    /// Downwards deflection (negative Y).
    Down,
    /// Leftwards deflection (negative X).
    Left,
    /// Rightwards deflection (positive X).
    Right,
}

impl StickDirection {
    /// Converts this stick direction into the corresponding GPUI action.
    pub fn to_action(&self) -> Box<dyn gpui::Action> {
        match self {
            Self::Up => Box::new(crate::actions::NavigateUp),
            Self::Down => Box::new(crate::actions::NavigateDown),
            Self::Left => Box::new(crate::actions::NavigateLeft),
            Self::Right => Box::new(crate::actions::NavigateRight),
        }
    }
}

/// Tracks directional repeat state and debounce timing for an independent directional input source
/// (such as the analog thumbstick or the digital D-pad).
#[derive(Debug, Clone, Default)]
pub struct DirectionalRepeatState {
    /// Currently active deflection direction.
    last_direction: Option<StickDirection>,
    /// Instant when the last directional action was fired.
    last_fire_time: Option<Instant>,
    /// Whether the next repeat fires after `initial_repeat_delay` (true) or `repeat_interval` (false).
    is_initial_hold: bool,
}

impl DirectionalRepeatState {
    /// Resets this directional state back to neutral.
    pub fn reset(&mut self) {
        self.last_direction = None;
        self.last_fire_time = None;
        self.is_initial_hold = false;
    }

    /// Evaluates direction and repeat timing given an optional current direction.
    pub fn update(
        &mut self,
        current_dir: Option<StickDirection>,
        now: Instant,
        initial_repeat_delay: Duration,
        repeat_interval: Duration,
    ) -> Option<StickDirection> {
        let Some(dir) = current_dir else {
            self.reset();
            return None;
        };

        match self.last_direction {
            Some(prev_dir) if prev_dir == dir => {
                let last_time = self.last_fire_time.unwrap_or(now);
                let elapsed = now.saturating_duration_since(last_time);
                let threshold = if self.is_initial_hold {
                    initial_repeat_delay
                } else {
                    repeat_interval
                };

                if elapsed >= threshold {
                    self.last_fire_time = Some(now);
                    self.is_initial_hold = false;
                    Some(dir)
                } else {
                    None
                }
            }
            _ => {
                // Direction changed or freshly deflected from neutral.
                self.last_direction = Some(dir);
                self.last_fire_time = Some(now);
                self.is_initial_hold = true;
                Some(dir)
            }
        }
    }
}

/// Translates raw gamepad thumbstick axes and button states into directional navigation
/// events and GPUI actions with configurable deadzone and debounce repeat timers.
#[derive(Debug, Clone)]
pub struct GamepadInputDispatcher {
    /// Radial deadzone threshold below which stick deflection is ignored (default 0.5).
    pub deadzone: f32,
    /// Delay before a held stick or button begins repeating (default 200ms).
    pub initial_repeat_delay: Duration,
    /// Interval between subsequent repeated directional triggers (default 100ms).
    pub repeat_interval: Duration,
    /// Separate repeat tracking state for analog thumbstick input.
    stick_state: DirectionalRepeatState,
    /// Separate repeat tracking state for digital D-pad input.
    dpad_state: DirectionalRepeatState,
    /// Set of buttons currently held down to enforce rising-edge triggers.
    pressed_buttons: HashSet<gilrs::Button>,
    /// Instant when Back + Start combo was first held down.
    osd_combo_start: Option<Instant>,
    /// Whether the OSD toggle action has already fired for this hold.
    osd_combo_fired: bool,
}

impl Default for GamepadInputDispatcher {
    fn default() -> Self {
        Self::new(0.5)
    }
}

impl GamepadInputDispatcher {
    /// Creates a new dispatcher with the specified deadzone and default timing constants
    /// (200ms initial repeat delay, 100ms repeat interval).
    pub fn new(deadzone: f32) -> Self {
        Self {
            deadzone,
            initial_repeat_delay: Duration::from_millis(200),
            repeat_interval: Duration::from_millis(100),
            stick_state: DirectionalRepeatState::default(),
            dpad_state: DirectionalRepeatState::default(),
            pressed_buttons: HashSet::new(),
            osd_combo_start: None,
            osd_combo_fired: false,
        }
    }

    /// Creates a new dispatcher with custom deadzone and timing parameters.
    pub fn with_timings(
        deadzone: f32,
        initial_repeat_delay: Duration,
        repeat_interval: Duration,
    ) -> Self {
        Self {
            deadzone,
            initial_repeat_delay,
            repeat_interval,
            stick_state: DirectionalRepeatState::default(),
            dpad_state: DirectionalRepeatState::default(),
            pressed_buttons: HashSet::new(),
            osd_combo_start: None,
            osd_combo_fired: false,
        }
    }

    /// Resets all internal state (active directions, repeat timers, and button hold trackers).
    pub fn reset(&mut self) {
        self.stick_state.reset();
        self.dpad_state.reset();
        self.pressed_buttons.clear();
        self.osd_combo_start = None;
        self.osd_combo_fired = false;
    }

    /// Polls analog stick deflection `(x, y)` at the given timestamp.
    ///
    /// - If the magnitude is below `deadzone`, returns `None` and resets stick repeat state.
    /// - If deflected beyond `deadzone`, fires immediately on direction change.
    /// - When held continuously in the same direction, suppresses repeats for `initial_repeat_delay` (200ms),
    ///   then fires repeated triggers every `repeat_interval` (100ms).
    /// - Operates independently from D-pad state; neutral stick will not clobber active D-pad repeats.
    pub fn poll_stick(&mut self, x: f32, y: f32, now: Instant) -> Option<StickDirection> {
        let magnitude = (x * x + y * y).sqrt();

        let current_dir = if magnitude < self.deadzone {
            None
        } else if x.abs() > y.abs() {
            if x > 0.0 {
                Some(StickDirection::Right)
            } else {
                Some(StickDirection::Left)
            }
        } else {
            if y > 0.0 {
                Some(StickDirection::Up)
            } else {
                Some(StickDirection::Down)
            }
        };

        self.stick_state.update(
            current_dir,
            now,
            self.initial_repeat_delay,
            self.repeat_interval,
        )
    }

    /// Polls D-pad digital inputs with decoupled repeat behavior matching the analog stick.
    ///
    /// When all directions are false, resets D-pad repeat state back to neutral.
    pub fn poll_dpad(
        &mut self,
        up: bool,
        down: bool,
        left: bool,
        right: bool,
        now: Instant,
    ) -> Option<StickDirection> {
        let mut x = 0.0f32;
        let mut y = 0.0f32;
        if right {
            x += 1.0;
        }
        if left {
            x -= 1.0;
        }
        if up {
            y += 1.0;
        }
        if down {
            y -= 1.0;
        }

        let current_dir = if x == 0.0 && y == 0.0 {
            None
        } else if x.abs() > y.abs() {
            if x > 0.0 {
                Some(StickDirection::Right)
            } else {
                Some(StickDirection::Left)
            }
        } else {
            if y > 0.0 {
                Some(StickDirection::Up)
            } else {
                Some(StickDirection::Down)
            }
        };

        self.dpad_state.update(
            current_dir,
            now,
            self.initial_repeat_delay,
            self.repeat_interval,
        )
    }

    /// Evaluates a button press or release event.
    ///
    /// Edge-triggered: returns `Some(action)` only on the rising edge (when pressed transitions
    /// from false to true). Returns `None` while held or released.
    pub fn poll_button(
        &mut self,
        button: gilrs::Button,
        pressed: bool,
        _now: Instant,
    ) -> Option<Box<dyn gpui::Action>> {
        if pressed {
            if self.pressed_buttons.insert(button) {
                // Rising edge: button was freshly pressed
                match button {
                    gilrs::Button::South => Some(Box::new(crate::actions::ConfirmAction)),
                    gilrs::Button::East => Some(Box::new(crate::actions::CancelAction)),
                    gilrs::Button::West => Some(Box::new(crate::actions::ContextAction1)),
                    gilrs::Button::North => Some(Box::new(crate::actions::ContextAction2)),
                    gilrs::Button::LeftTrigger => Some(Box::new(crate::actions::PrevTab)),
                    gilrs::Button::RightTrigger => Some(Box::new(crate::actions::NextTab)),
                    gilrs::Button::Start => Some(Box::new(crate::actions::QuickLaunch)),
                    gilrs::Button::Mode => Some(Box::new(crate::actions::ToggleBigPicture)),
                    gilrs::Button::DPadUp => Some(Box::new(crate::actions::NavigateUp)),
                    gilrs::Button::DPadDown => Some(Box::new(crate::actions::NavigateDown)),
                    gilrs::Button::DPadLeft => Some(Box::new(crate::actions::NavigateLeft)),
                    gilrs::Button::DPadRight => Some(Box::new(crate::actions::NavigateRight)),
                    _ => None,
                }
            } else {
                // Held button (no repeat for action buttons)
                None
            }
        } else {
            self.pressed_buttons.remove(&button);
            None
        }
    }

    /// Polls all gamepad buttons and analog sticks on the provided `gilrs::Gamepad`,
    /// returning any triggered GPUI actions for the current frame.
    ///
    /// Analog stick and D-pad maintain isolated repeat timers so neutral stick polling
    /// does not clobber held D-pad inputs.
    pub fn poll_buttons(
        &mut self,
        gamepad: &gilrs::Gamepad,
        now: Instant,
    ) -> Vec<Box<dyn gpui::Action>> {
        let mut actions: Vec<Box<dyn gpui::Action>> = Vec::new();

        // 1. Analog Stick
        let lx = gamepad.value(gilrs::Axis::LeftStickX);
        let ly = gamepad.value(gilrs::Axis::LeftStickY);
        let stick_action = self.poll_stick(lx, ly, now).map(|d| d.to_action());

        // 2. D-Pad (called unconditionally so releasing D-pad properly clears dpad_state)
        let dpad_up = gamepad.is_pressed(gilrs::Button::DPadUp);
        let dpad_down = gamepad.is_pressed(gilrs::Button::DPadDown);
        let dpad_left = gamepad.is_pressed(gilrs::Button::DPadLeft);
        let dpad_right = gamepad.is_pressed(gilrs::Button::DPadRight);
        let dpad_action = self
            .poll_dpad(dpad_up, dpad_down, dpad_left, dpad_right, now)
            .map(|d| d.to_action());

        // Stick takes precedence if both deflected simultaneously; otherwise D-pad fires
        if let Some(action) = stick_action {
            actions.push(action);
        } else if let Some(action) = dpad_action {
            actions.push(action);
        }

        // 3. Check Back (Select) + Start 1.5s Hold Combo for In-Game Couch OSD
        let back_pressed = gamepad.is_pressed(gilrs::Button::Select);
        let start_pressed = gamepad.is_pressed(gilrs::Button::Start);
        if let Some(combo_act) = self.poll_combo_osd(back_pressed, start_pressed, now) {
            actions.push(combo_act);
        }

        // 4. Monitored Action Buttons (Button::Mode toggles Big Picture, Button::Select reserved for OSD combo)
        const MONITORED: [gilrs::Button; 8] = [
            gilrs::Button::South,
            gilrs::Button::East,
            gilrs::Button::West,
            gilrs::Button::North,
            gilrs::Button::LeftTrigger,
            gilrs::Button::RightTrigger,
            gilrs::Button::Start,
            gilrs::Button::Mode,
        ];

        for &btn in &MONITORED {
            let is_down = gamepad.is_pressed(btn);
            // Suppress QuickLaunch when holding Select+Start combo
            if btn == gilrs::Button::Start && back_pressed {
                continue;
            }
            if let Some(act) = self.poll_button(btn, is_down, now) {
                actions.push(act);
            }
        }

        actions
    }

    /// Evaluates Back (Select) + Start button combo held for 1.5s (1500ms).
    /// Returns `Some(ToggleOsd)` once when threshold is reached, then suppresses until release.
    pub fn poll_combo_osd(
        &mut self,
        back: bool,
        start: bool,
        now: Instant,
    ) -> Option<Box<dyn gpui::Action>> {
        if back && start {
            let start_time = *self.osd_combo_start.get_or_insert(now);
            if !self.osd_combo_fired
                && now.saturating_duration_since(start_time) >= Duration::from_millis(1500)
            {
                self.osd_combo_fired = true;
                Some(Box::new(crate::actions::ToggleOsd))
            } else {
                None
            }
        } else {
            self.osd_combo_start = None;
            self.osd_combo_fired = false;
            None
        }
    }
}

/// Maps keyboard keystrokes and modifier combinations to canonical Big Picture UI action names.
pub fn map_key_to_couch_action_name(key: &str, ctrl: bool, shift: bool) -> Option<&'static str> {
    let lower = key.to_ascii_lowercase();
    let k = lower.as_str();

    // 1. OSD toggle shortcuts: Ctrl+Shift+O, F10, or Ctrl+Escape
    if (ctrl && shift && k == "o") || (!ctrl && k == "f10") || (ctrl && !shift && k == "escape") {
        return Some("ToggleOsd");
    }

    // If Ctrl is held down and not part of an OSD combo, suppress standard navigation/actions
    if ctrl {
        return None;
    }

    // 2. Big Picture toggle: F11
    if k == "f11" {
        return Some("ToggleBigPicture");
    }

    // 3. QuickLaunch: F5
    if k == "f5" {
        return Some("QuickLaunch");
    }

    // 4. Directional navigation
    match k {
        "arrowup" | "up" | "w" => return Some("NavigateUp"),
        "arrowdown" | "down" | "s" => return Some("NavigateDown"),
        "arrowleft" | "left" | "a" => return Some("NavigateLeft"),
        "arrowright" | "right" | "d" => return Some("NavigateRight"),
        _ => {}
    }

    // 5. Confirm: Enter, Space, Return
    if k == "enter" || k == "space" || k == "return" {
        return Some("Confirm");
    }

    // 6. Cancel: Escape, Backspace, B
    if k == "escape" || k == "backspace" || k == "b" {
        return Some("Cancel");
    }

    // 7. Tabs / Bumpers: Tab, Shift+Tab, E (NextTab), Q (PrevTab)
    if k == "tab" {
        return if shift {
            Some("PrevTab")
        } else {
            Some("NextTab")
        };
    }
    if k == "e" {
        return Some("NextTab");
    }
    if k == "q" {
        return Some("PrevTab");
    }

    // 8. Context actions: X (Context1), Y (Context2)
    if k == "x" {
        return Some("Context1");
    }
    if k == "y" {
        return Some("Context2");
    }

    None
}
