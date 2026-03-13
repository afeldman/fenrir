//! Konvertierung winit-Events → Servo InputEvents.

use keyboard_types::{Code, Key, KeyState, Location, Modifiers, NamedKey};
use servo::{
    DevicePoint, InputEvent, KeyboardEvent, MouseButton as ServoButton, MouseButtonAction,
    MouseButtonEvent, MouseMoveEvent, WebViewPoint, WheelDelta, WheelEvent, WheelMode,
};
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta};
use winit::keyboard::{Key as WinitKey, ModifiersState, NamedKey as WN, PhysicalKey, KeyCode};

fn device_point(x: f32, y: f32) -> WebViewPoint {
    WebViewPoint::Device(DevicePoint::new(x, y))
}

/// Maus-Koordinate (relativ zur WebView) → MouseMoveEvent.
pub fn mouse_move(x: f32, y: f32) -> InputEvent {
    InputEvent::MouseMove(MouseMoveEvent::new(device_point(x, y)))
}

/// Mausklick → MouseButtonEvent.
pub fn mouse_button(button: MouseButton, state: ElementState, x: f32, y: f32) -> InputEvent {
    let btn = match button {
        MouseButton::Left     => ServoButton::Left,
        MouseButton::Right    => ServoButton::Right,
        MouseButton::Middle   => ServoButton::Middle,
        MouseButton::Back     => ServoButton::Back,
        MouseButton::Forward  => ServoButton::Forward,
        MouseButton::Other(n) => ServoButton::Other(n),
    };
    let action = match state {
        ElementState::Pressed  => MouseButtonAction::Down,
        ElementState::Released => MouseButtonAction::Up,
    };
    InputEvent::MouseButton(MouseButtonEvent::new(action, btn, device_point(x, y)))
}

/// Scroll → WheelEvent.
pub fn scroll(delta: MouseScrollDelta, x: f32, y: f32) -> InputEvent {
    let (dx, dy, mode) = match delta {
        MouseScrollDelta::LineDelta(dx, dy) => ((dx * 76.0) as f64, (dy * 76.0) as f64, WheelMode::DeltaLine),
        MouseScrollDelta::PixelDelta(pos)   => (pos.x, pos.y, WheelMode::DeltaPixel),
    };
    InputEvent::Wheel(WheelEvent::new(WheelDelta { x: dx, y: dy, z: 0.0, mode }, device_point(x, y)))
}

/// winit-Tastaturevent → Servo KeyboardEvent (vereinfacht).
pub fn keyboard(event: &KeyEvent, modifiers: ModifiersState) -> Option<InputEvent> {
    let state = match event.state {
        ElementState::Pressed  => KeyState::Down,
        ElementState::Released => KeyState::Up,
    };

    let key = match &event.logical_key {
        WinitKey::Named(n) => Key::Named(named_key(*n)?),
        WinitKey::Character(s) => Key::Character(s.to_string()),
        _ => return None,
    };

    let code = physical_code(event.physical_key);
    let mods = winit_mods(modifiers);

    let kb_event = keyboard_types::KeyboardEvent {
        state,
        key,
        code,
        modifiers: mods,
        location: Location::Standard,
        repeat: event.repeat,
        is_composing: false,
    };

    Some(InputEvent::Keyboard(KeyboardEvent::new(kb_event)))
}

fn named_key(k: WN) -> Option<NamedKey> {
    Some(match k {
        WN::Enter       => NamedKey::Enter,
        WN::Tab         => NamedKey::Tab,
        WN::Escape      => NamedKey::Escape,
        WN::Backspace   => NamedKey::Backspace,
        WN::Delete      => NamedKey::Delete,
        WN::ArrowLeft   => NamedKey::ArrowLeft,
        WN::ArrowRight  => NamedKey::ArrowRight,
        WN::ArrowUp     => NamedKey::ArrowUp,
        WN::ArrowDown   => NamedKey::ArrowDown,
        WN::Home        => NamedKey::Home,
        WN::End         => NamedKey::End,
        WN::PageUp      => NamedKey::PageUp,
        WN::PageDown    => NamedKey::PageDown,
        WN::F1  => NamedKey::F1,  WN::F2  => NamedKey::F2,  WN::F3  => NamedKey::F3,
        WN::F4  => NamedKey::F4,  WN::F5  => NamedKey::F5,  WN::F6  => NamedKey::F6,
        WN::F7  => NamedKey::F7,  WN::F8  => NamedKey::F8,  WN::F9  => NamedKey::F9,
        WN::F10 => NamedKey::F10, WN::F11 => NamedKey::F11, WN::F12 => NamedKey::F12,
        WN::Shift | WN::Control | WN::Alt | WN::Super | WN::Meta => return None,
        WN::Copy  => NamedKey::Copy,
        WN::Paste => NamedKey::Paste,
        WN::Cut   => NamedKey::Cut,
        _         => NamedKey::Unidentified,
    })
}

fn physical_code(key: PhysicalKey) -> Code {
    match key {
        PhysicalKey::Code(c) => match c {
            KeyCode::KeyA => Code::KeyA, KeyCode::KeyB => Code::KeyB,
            KeyCode::KeyC => Code::KeyC, KeyCode::KeyD => Code::KeyD,
            KeyCode::KeyE => Code::KeyE, KeyCode::KeyF => Code::KeyF,
            KeyCode::KeyG => Code::KeyG, KeyCode::KeyH => Code::KeyH,
            KeyCode::KeyI => Code::KeyI, KeyCode::KeyJ => Code::KeyJ,
            KeyCode::KeyK => Code::KeyK, KeyCode::KeyL => Code::KeyL,
            KeyCode::KeyM => Code::KeyM, KeyCode::KeyN => Code::KeyN,
            KeyCode::KeyO => Code::KeyO, KeyCode::KeyP => Code::KeyP,
            KeyCode::KeyQ => Code::KeyQ, KeyCode::KeyR => Code::KeyR,
            KeyCode::KeyS => Code::KeyS, KeyCode::KeyT => Code::KeyT,
            KeyCode::KeyU => Code::KeyU, KeyCode::KeyV => Code::KeyV,
            KeyCode::KeyW => Code::KeyW, KeyCode::KeyX => Code::KeyX,
            KeyCode::KeyY => Code::KeyY, KeyCode::KeyZ => Code::KeyZ,
            KeyCode::Enter => Code::Enter, KeyCode::Tab => Code::Tab,
            KeyCode::Space => Code::Space, KeyCode::Escape => Code::Escape,
            KeyCode::Backspace => Code::Backspace, KeyCode::Delete => Code::Delete,
            KeyCode::ArrowLeft => Code::ArrowLeft, KeyCode::ArrowRight => Code::ArrowRight,
            KeyCode::ArrowUp => Code::ArrowUp, KeyCode::ArrowDown => Code::ArrowDown,
            KeyCode::Home => Code::Home, KeyCode::End => Code::End,
            KeyCode::PageUp => Code::PageUp, KeyCode::PageDown => Code::PageDown,
            _ => Code::Unidentified,
        },
        _ => Code::Unidentified,
    }
}

fn winit_mods(m: ModifiersState) -> Modifiers {
    let mut mods = Modifiers::empty();
    if m.shift_key()   { mods |= Modifiers::SHIFT; }
    if m.control_key() { mods |= Modifiers::CONTROL; }
    if m.alt_key()     { mods |= Modifiers::ALT; }
    if m.super_key()   { mods |= Modifiers::META; }
    mods
}
