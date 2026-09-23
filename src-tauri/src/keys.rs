#[cfg(not(target_os = "macos"))]
use rdev::Key;

/// `.` は設定画面を開くための予約キー。割り当て対象からは常に除外する。
pub const RESERVED_KEY: &str = "Period";

/// 設定画面で割り当てできる物理キー。フロントエンドの `src/keys.ts` と対になる。
pub const ASSIGNABLE_KEYS: &[&str] = &[
    "KeyA", "KeyB", "KeyC", "KeyD", "KeyE", "KeyF", "KeyG", "KeyH", "KeyI", "KeyJ", "KeyK", "KeyL",
    "KeyM", "KeyN", "KeyO", "KeyP", "KeyQ", "KeyR", "KeyS", "KeyT", "KeyU", "KeyV", "KeyW", "KeyX",
    "KeyY", "KeyZ", "Digit0", "Digit1", "Digit2", "Digit3", "Digit4", "Digit5", "Digit6", "Digit7",
    "Digit8", "Digit9", "Backquote", "Minus", "Equal", "BracketLeft", "BracketRight", "Backslash",
    "Semicolon", "Quote", "Comma", "Slash", "IntlYen", "IntlRo", "IntlBackslash", "Escape", "Tab",
    "Enter", "Space", "Backspace", "ArrowLeft", "ArrowUp", "ArrowDown", "ArrowRight",
];

/// 旧バージョンの設定（LeftControl / LeftOption など）を正規名へ寄せる。
pub fn normalize_modifier(value: &str) -> Option<&'static str> {
    let compact: String = value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    Some(match compact.as_str() {
        "controlleft" | "leftcontrol" | "leftctrl" | "ctrlleft" => "ControlLeft",
        "controlright" | "rightcontrol" | "rightctrl" | "ctrlright" => "ControlRight",
        "altleft" | "leftalt" | "leftoption" | "optionleft" => "AltLeft",
        "altright" | "rightalt" | "rightoption" | "optionright" | "altgr" => "AltRight",
        "metaleft" | "leftmeta" | "leftcommand" | "commandleft" | "leftwin" | "winleft" => {
            "MetaLeft"
        }
        "metaright" | "rightmeta" | "rightcommand" | "commandright" | "rightwin" | "winright" => {
            "MetaRight"
        }
        "shiftleft" | "leftshift" => "ShiftLeft",
        "shiftright" | "rightshift" => "ShiftRight",
        "fn" | "function" => "Fn",
        _ => return None,
    })
}

/// 正規名を rdev のキーへ変換する。
#[cfg(not(target_os = "macos"))]
pub fn modifier_key(code: &str) -> Option<Key> {
    Some(match normalize_modifier(code)? {
        "ControlLeft" => Key::ControlLeft,
        "ControlRight" => Key::ControlRight,
        // macOSではOptionキーがAlt/AltGrとして報告される。
        "AltLeft" => Key::Alt,
        "AltRight" => Key::AltGr,
        "MetaLeft" => Key::MetaLeft,
        "MetaRight" => Key::MetaRight,
        "ShiftLeft" => Key::ShiftLeft,
        "ShiftRight" => Key::ShiftRight,
        "Fn" => Key::Function,
        _ => return None,
    })
}

/// 物理キーを設定ファイル上の正規名へ変換する。
#[cfg(not(target_os = "macos"))]
pub fn key_name(key: Key) -> Option<&'static str> {
    Some(match key {
        Key::KeyA => "KeyA",
        Key::KeyB => "KeyB",
        Key::KeyC => "KeyC",
        Key::KeyD => "KeyD",
        Key::KeyE => "KeyE",
        Key::KeyF => "KeyF",
        Key::KeyG => "KeyG",
        Key::KeyH => "KeyH",
        Key::KeyI => "KeyI",
        Key::KeyJ => "KeyJ",
        Key::KeyK => "KeyK",
        Key::KeyL => "KeyL",
        Key::KeyM => "KeyM",
        Key::KeyN => "KeyN",
        Key::KeyO => "KeyO",
        Key::KeyP => "KeyP",
        Key::KeyQ => "KeyQ",
        Key::KeyR => "KeyR",
        Key::KeyS => "KeyS",
        Key::KeyT => "KeyT",
        Key::KeyU => "KeyU",
        Key::KeyV => "KeyV",
        Key::KeyW => "KeyW",
        Key::KeyX => "KeyX",
        Key::KeyY => "KeyY",
        Key::KeyZ => "KeyZ",
        Key::Num0 => "Digit0",
        Key::Num1 => "Digit1",
        Key::Num2 => "Digit2",
        Key::Num3 => "Digit3",
        Key::Num4 => "Digit4",
        Key::Num5 => "Digit5",
        Key::Num6 => "Digit6",
        Key::Num7 => "Digit7",
        Key::Num8 => "Digit8",
        Key::Num9 => "Digit9",
        Key::BackQuote => "Backquote",
        Key::Minus => "Minus",
        Key::Equal => "Equal",
        Key::LeftBracket => "BracketLeft",
        Key::RightBracket => "BracketRight",
        Key::BackSlash => "Backslash",
        Key::SemiColon => "Semicolon",
        Key::Quote => "Quote",
        Key::Comma => "Comma",
        Key::Dot => RESERVED_KEY,
        Key::Slash => "Slash",
        Key::IntlBackslash => "IntlBackslash",
        Key::Escape => "Escape",
        Key::Tab => "Tab",
        Key::Return | Key::KpReturn => "Enter",
        Key::Space => "Space",
        Key::Backspace => "Backspace",
        Key::LeftArrow => "ArrowLeft",
        Key::UpArrow => "ArrowUp",
        Key::DownArrow => "ArrowDown",
        Key::RightArrow => "ArrowRight",
        // rdevがJIS固有キーを解決しないため、OS毎のスキャンコードで補う。
        #[cfg(target_os = "macos")]
        Key::Unknown(93) => "IntlYen",
        #[cfg(target_os = "macos")]
        Key::Unknown(94) => "IntlRo",
        #[cfg(target_os = "macos")]
        Key::Unknown(10) => "IntlBackslash",
        _ => return None,
    })
}

/// macOSのキーコードを正規名へ変換する。rdevを介さず自前のタップで受けるため。
#[cfg(target_os = "macos")]
pub fn key_name_from_macos_code(code: i64) -> Option<&'static str> {
    Some(match code {
        0 => "KeyA",
        1 => "KeyS",
        2 => "KeyD",
        3 => "KeyF",
        4 => "KeyH",
        5 => "KeyG",
        6 => "KeyZ",
        7 => "KeyX",
        8 => "KeyC",
        9 => "KeyV",
        10 => "IntlBackslash",
        11 => "KeyB",
        12 => "KeyQ",
        13 => "KeyW",
        14 => "KeyE",
        15 => "KeyR",
        16 => "KeyY",
        17 => "KeyT",
        18 => "Digit1",
        19 => "Digit2",
        20 => "Digit3",
        21 => "Digit4",
        22 => "Digit6",
        23 => "Digit5",
        24 => "Equal",
        25 => "Digit9",
        26 => "Digit7",
        27 => "Minus",
        28 => "Digit8",
        29 => "Digit0",
        30 => "BracketRight",
        31 => "KeyO",
        32 => "KeyU",
        33 => "BracketLeft",
        34 => "KeyI",
        35 => "KeyP",
        36 | 76 => "Enter",
        37 => "KeyL",
        38 => "KeyJ",
        39 => "Quote",
        40 => "KeyK",
        41 => "Semicolon",
        42 => "Backslash",
        43 => "Comma",
        44 => "Slash",
        45 => "KeyN",
        46 => "KeyM",
        47 => RESERVED_KEY,
        48 => "Tab",
        49 => "Space",
        50 => "Backquote",
        51 => "Backspace",
        53 => "Escape",
        93 => "IntlYen",
        94 => "IntlRo",
        123 => "ArrowLeft",
        124 => "ArrowRight",
        125 => "ArrowDown",
        126 => "ArrowUp",
        // 修飾キー。押下と解放はフラグで判別する。
        54 => "MetaRight",
        55 => "MetaLeft",
        56 => "ShiftLeft",
        57 => "CapsLock",
        58 => "AltLeft",
        59 => "ControlLeft",
        60 => "ShiftRight",
        61 => "AltRight",
        62 => "ControlRight",
        63 => "Fn",
        _ => return None,
    })
}

/// 修飾キーに対応するデバイス固有のフラグビット。
/// flagsChangedイベントでこのビットが立っていれば押下、落ちていれば解放。
#[cfg(target_os = "macos")]
pub fn macos_modifier_mask(name: &str) -> Option<u64> {
    Some(match name {
        "ControlLeft" => 0x0000_0001,
        "ShiftLeft" => 0x0000_0002,
        "ShiftRight" => 0x0000_0004,
        "MetaLeft" => 0x0000_0008,
        "MetaRight" => 0x0000_0010,
        "AltLeft" => 0x0000_0020,
        "AltRight" => 0x0000_0040,
        "ControlRight" => 0x0000_2000,
        "CapsLock" => 0x0001_0000,
        "Fn" => 0x0080_0000,
        _ => return None,
    })
}

/// JIS配列では同じ物理キーがOSによって別名で届くため、別名も一致とみなす。
pub fn key_matches(binding: &str, pressed: &str) -> bool {
    if binding == pressed {
        return true;
    }
    #[cfg(target_os = "windows")]
    {
        // Windows版のJISキーボードでは ¥ が BackSlash として報告される。
        if matches!(
            (binding, pressed),
            ("IntlYen", "Backslash") | ("Backslash", "IntlYen")
        ) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_key_is_not_assignable() {
        assert!(!ASSIGNABLE_KEYS.contains(&RESERVED_KEY));
    }

    #[test]
    fn normalizes_legacy_modifier_names() {
        assert_eq!(normalize_modifier("LeftControl"), Some("ControlLeft"));
        assert_eq!(normalize_modifier("LeftOption"), Some("AltLeft"));
        assert_eq!(normalize_modifier("LeftAlt"), Some("AltLeft"));
        assert_eq!(normalize_modifier("nope"), None);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn maps_macos_keycodes() {
        assert_eq!(key_name_from_macos_code(0), Some("KeyA"));
        assert_eq!(key_name_from_macos_code(26), Some("Digit7"));
        assert_eq!(key_name_from_macos_code(47), Some(RESERVED_KEY));
        assert_eq!(key_name_from_macos_code(36), Some("Enter"));
        assert_eq!(key_name_from_macos_code(124), Some("ArrowRight"));
        assert_eq!(key_name_from_macos_code(93), Some("IntlYen"));
        assert_eq!(key_name_from_macos_code(200), None);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn every_macos_key_is_assignable_or_a_modifier() {
        for code in 0..=127i64 {
            let Some(name) = key_name_from_macos_code(code) else {
                continue;
            };
            let known = ASSIGNABLE_KEYS.contains(&name)
                || name == RESERVED_KEY
                || macos_modifier_mask(name).is_some();
            assert!(known, "{name} (code {code}) is not a known key");
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn modifier_masks_are_distinct() {
        let masks: Vec<u64> = ["ControlLeft", "ShiftLeft", "AltLeft", "MetaLeft", "Fn"]
            .iter()
            .filter_map(|name| macos_modifier_mask(name))
            .collect();
        for (index, mask) in masks.iter().enumerate() {
            assert!(masks[index + 1..].iter().all(|other| other != mask));
        }
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn maps_physical_keys() {
        assert_eq!(key_name(Key::KeyA), Some("KeyA"));
        assert_eq!(key_name(Key::Num7), Some("Digit7"));
        assert_eq!(key_name(Key::Dot), Some(RESERVED_KEY));
        assert_eq!(key_name(Key::Return), Some("Enter"));
    }
}
