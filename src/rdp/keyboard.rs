use eframe::egui::{Event, Key};
use ironrdp::pdu::input::fast_path::{FastPathInputEvent, KeyboardFlags};

pub struct RDPKeyboardEvents {
    fastpath_events: Vec<FastPathInputEvent>,
}

impl RDPKeyboardEvents {
    pub fn maybe_from(event: &Event) -> Option<RDPKeyboardEvents> {
        match event {
            Event::Key {
                key,
                physical_key,
                pressed,
                repeat,
                modifiers,
            } => {
                let mut fastpath_events = Vec::<FastPathInputEvent>::default();

                // TODO process modifiers
                let r = get_scancode(physical_key.as_ref().unwrap_or(key));
                if let Err(e) = r {
                    log::error!("Key error: {}", e);
                    return None;
                } else {
                    let mut flags = if *pressed {
                        KeyboardFlags::empty()
                    } else {
                        KeyboardFlags::RELEASE
                    };

                    // TODO instead of sending modifiers on every keystroke, keep state and
                    // only send required deltas.
                    if modifiers.alt {
                        fastpath_events.push(FastPathInputEvent::KeyboardEvent(
                            flags,
                            SCANCODE_MODIFIER_ALT,
                        ));
                    }
                    if modifiers.shift {
                        fastpath_events.push(FastPathInputEvent::KeyboardEvent(
                            flags,
                            SCANCODE_MODIFIER_SHIFT,
                        ));
                    }
                    if modifiers.ctrl {
                        fastpath_events.push(FastPathInputEvent::KeyboardEvent(
                            flags,
                            SCANCODE_MODIFIER_CTRL,
                        ));
                    }

                    let (extended, scancode) = r.unwrap();
                    if extended {
                        flags |= KeyboardFlags::EXTENDED;
                    }

                    if *pressed {
                        fastpath_events.push(FastPathInputEvent::KeyboardEvent(flags, scancode));
                    } else {
                        fastpath_events
                            .insert(0, FastPathInputEvent::KeyboardEvent(flags, scancode));
                    }
                }

                Some(Self { fastpath_events })
            }
            _unsupported => None,
        }
    }

    pub fn as_fastpath_events(self) -> Vec<FastPathInputEvent> {
        self.fastpath_events
    }
}

pub static SCANCODE_MODIFIER_ALT: u8 = 0x38;
pub static SCANCODE_MODIFIER_CTRL: u8 = 0x1d;
pub static SCANCODE_MODIFIER_SHIFT: u8 = 0x2a;

/// Convert special keys to scan codes as given on
/// https://learn.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input
/// Returns a bool to indicate whether the scan code requires the
/// extended scancode prefix (0xE0)
pub fn get_scancode(key: &Key) -> anyhow::Result<(bool, u8)> {
    Ok(match key {
        Key::Space => (false, 0x39),
        Key::Delete => (false, 0xd3),
        Key::Backspace => (false, 0x0e),
        Key::Insert => (true, 0x52),
        Key::Tab => (false, 0x0f),
        Key::Enter => (false, 0x1c), // Same as enter for Win32?
        Key::Escape => (false, 0x01),
        Key::Home => (true, 0x47),
        Key::ArrowLeft => (true, 0x4b),  /* Move left, left arrow */
        Key::ArrowUp => (true, 0x48),    /* Move up, up arrow */
        Key::ArrowRight => (true, 0x4d), /* Move right, right arrow */
        Key::ArrowDown => (true, 0x50),  /* Move down, down arrow */
        Key::PageUp => (true, 0x49),
        Key::PageDown => (true, 0x51),
        Key::End => (true, 0x4f), /* EOL */
        Key::F1 => (false, 0x3b),
        Key::F2 => (false, 0x3c),
        Key::F3 => (false, 0x3d),
        Key::F4 => (false, 0x3e),
        Key::F5 => (false, 0x3f),
        Key::F6 => (false, 0x40),
        Key::F7 => (false, 0x41),
        Key::F8 => (false, 0x42),
        Key::F9 => (false, 0x43),
        Key::F10 => (false, 0x44),
        Key::F11 => (false, 0x57),
        Key::F12 => (false, 0x58),
        Key::F13 => (false, 0x64),
        Key::F14 => (false, 0x65),
        Key::F15 => (false, 0x66),
        Key::F16 => (false, 0x67),
        Key::F17 => (false, 0x68),
        Key::F18 => (false, 0x69),
        Key::F19 => (false, 0x6a),
        Key::F20 => (false, 0x6b),
        Key::F21 => (false, 0x6c),
        Key::F22 => (false, 0x6d),
        Key::F23 => (false, 0x6e),
        Key::F24 => (false, 0x76),
        Key::Colon => (false, 0x27),
        Key::Comma => (false, 0x33),
        Key::Backslash => (false, 0x2b), // Named backwards in egui?
        Key::Slash | Key::Pipe => (false, 0x35),
        Key::Backtick => (false, 0x29),
        Key::OpenBracket => (false, 0x1a),
        Key::CloseBracket => (false, 0x1b),
        Key::OpenCurlyBracket => (false, 0x1a),
        Key::CloseCurlyBracket => (false, 0x1b),
        Key::Equals => (false, 0x0d),
        Key::Exclamationmark => (false, 0x02),
        // Not sure what these would be mapped to in a Windows environment
        Key::Copy
        | Key::Cut
        | Key::Paste
        | Key::F25
        | Key::F26
        | Key::F27
        | Key::F28
        | Key::F29
        | Key::F30
        | Key::F31
        | Key::F32
        | Key::F33
        | Key::F34
        | Key::F35 => return Err(anyhow::anyhow!("Key {:?} is not mapped", key)),
        Key::Period => (false, 0x34),
        Key::Minus => (false, 0x0c),
        Key::Plus => (false, 0x4e), // Keypad
        Key::Quote => (false, 0x28),
        Key::Questionmark => (false, 0x35),
        Key::Semicolon => (false, 0x27),
        Key::A => (false, 0x1e),
        Key::B => (false, 0x30),
        Key::C => (false, 0x2e),
        Key::D => (false, 0x20),
        Key::E => (false, 0x12),
        Key::F => (false, 0x21),
        Key::G => (false, 0x22),
        Key::H => (false, 0x23),
        Key::I => (false, 0x17),
        Key::J => (false, 0x24),
        Key::K => (false, 0x25),
        Key::L => (false, 0x26),
        Key::M => (false, 0x32),
        Key::N => (false, 0x31),
        Key::O => (false, 0x18),
        Key::P => (false, 0x19),
        Key::Q => (false, 0x10),
        Key::R => (false, 0x13),
        Key::S => (false, 0x1f),
        Key::T => (false, 0x14),
        Key::U => (false, 0x16),
        Key::V => (false, 0x2f),
        Key::W => (false, 0x11),
        Key::X => (false, 0x2d),
        Key::Y => (false, 0x15),
        Key::Z => (false, 0x2c),
        Key::Num0 => (false, 0x0b),
        Key::Num1 => (false, 0x02),
        Key::Num2 => (false, 0x03),
        Key::Num3 => (false, 0x04),
        Key::Num4 => (false, 0x05),
        Key::Num5 => (false, 0x06),
        Key::Num6 => (false, 0x07),
        Key::Num7 => (false, 0x08),
        Key::Num8 => (false, 0x09),
        Key::Num9 => (false, 0x0a),
    })
}
