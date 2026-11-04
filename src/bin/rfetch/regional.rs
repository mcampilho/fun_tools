//! Time zone, locale and keyboard layout.

pub fn timezone() -> Option<String> {
    iana_time_zone::get_timezone().ok()
}

pub fn locale() -> Option<String> {
    sys_locale::get_locale()
}

/// Keyboard layout of the current input language, e.g. "pt-PT".
#[cfg(windows)]
pub fn keyboard() -> Option<String> {
    use windows_sys::Win32::Globalization::LCIDToLocaleName;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout;

    // SAFETY: plain Win32 calls with a valid, correctly sized buffer.
    unsafe {
        let hkl = GetKeyboardLayout(0) as usize;
        let lang_id = (hkl & 0xFFFF) as u32;
        let mut buf = [0u16; 85]; // LOCALE_NAME_MAX_LENGTH
        let n = LCIDToLocaleName(lang_id, buf.as_mut_ptr(), buf.len() as i32, 0);
        (n > 1).then(|| String::from_utf16_lossy(&buf[..(n - 1) as usize]))
    }
}

/// Keyboard layout from the usual configuration files.
#[cfg(not(windows))]
pub fn keyboard() -> Option<String> {
    let non_empty = |s: String| (!s.is_empty()).then_some(s);
    if let Some(v) = std::env::var("XKB_DEFAULT_LAYOUT").ok().and_then(non_empty) {
        return Some(v);
    }
    let read = |p: &str| std::fs::read_to_string(p).ok();
    let value_of = |text: &str, key: &str| {
        text.lines()
            .find_map(|l| l.trim().strip_prefix(key)?.strip_prefix('='))
            .map(|v| v.trim().trim_matches(['"', '\'']).to_string())
            .filter(|v| !v.is_empty())
    };
    // Debian/Ubuntu
    if let Some(v) = read("/etc/default/keyboard").and_then(|t| value_of(&t, "XKBLAYOUT")) {
        return Some(v);
    }
    // systemd-localed (X11), then the console keymap
    if let Some(t) = read("/etc/X11/xorg.conf.d/00-keyboard.conf") {
        let layout = t.lines().find_map(|l| {
            let l = l.trim();
            l.strip_prefix("Option")?
                .trim()
                .strip_prefix("\"XkbLayout\"")
                .map(|v| v.trim().trim_matches('"').to_string())
        });
        if let Some(v) = layout.and_then(non_empty) {
            return Some(v);
        }
    }
    read("/etc/vconsole.conf").and_then(|t| value_of(&t, "KEYMAP"))
}
