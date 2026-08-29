/// Maps Soundcore device_color byte values to hex colors.
///
/// The device_color from Bluetooth is the ASCII character code of a color string.
/// For example, byte 0x31 = character '1' = "black".
///
/// Color mapping reverse-engineered from Soundcore Android APK:
/// `com.oceanwing.serverResManager.serverResource.ServerResourceUrlUtil.b()`

/// Maps a device_color byte to a hex color string (e.g., "#1a1a1a").
/// Returns None if the byte doesn't map to a known color.
#[allow(dead_code)]
pub fn color_from_byte(color_byte: u8) -> Option<&'static str> {
    let ch = color_byte as char;
    match ch {
        '1' => Some("#a0a0a0"),   // black → lightened for dark bg
        '2' => Some("#f5f5f5"),   // white
        '3' => Some("#5a8fc5"),   // blue → lightened
        '4' => Some("#c0c0c0"),   // silver
        '5' => Some("#e8a0b4"),   // pink
        '6' => Some("#6aaa62"),   // green → lightened
        '7' => Some("#d4a017"),   // yellow
        '8' => Some("#ab8453"),   // brown → lightened
        '9' => Some("#e45050"),   // red → lightened
        'a' | 'A' => Some("#808080"),  // gray
        'b' | 'B' => Some("#c9a84c"),  // golden
        'c' | 'C' => Some("#f5f0e1"),  // sand_white
        'd' | 'D' => Some("#4a6a94"),  // navy_blue → lightened
        'e' | 'E' => Some("#e8735a"),  // coral
        'f' | 'F' => Some("#87ceeb"),  // sky_blue
        'g' | 'G' => Some("#e84057"),  // watermelonred
        'h' | 'H' => Some("#c3b091"),  // khaki
        'i' | 'I' => Some("#89cff0"),  // babyblue
        'k' | 'K' => Some("#d3d3d3"),  // lightgray
        'l' | 'L' => Some("#7fb5d5"),  // bluesilver
        'm' | 'M' => Some("#c93756"),  // redgolden
        'n' | 'N' => Some("#add8e6"),  // lightblue
        'o' | 'O' => Some("#800080"),  // purple
        'p' | 'P' => Some("#4a5d23"),  // camouflage
        'q' | 'Q' => Some("#fffdd0"),  // creamywhite
        'r' | 'R' => Some("#1a1a1a"),  // black_se
        's' | 'S' => Some("#f0f0f0"),  // white_red
        't' | 'T' => Some("#9b59b6"),  // purple_yellow
        'u' | 'U' => Some("#2d2d2d"),  // black_golden
        'v' | 'V' => Some("#9acd32"),  // yellow_green
        'w' | 'W' => Some("#0d0d0d"),  // black_pure
        'x' | 'X' => Some("#006400"),  // green_pure
        'y' | 'Y' => Some("#0000cd"),  // blue_pure
        _ => Some("#2d2d2d"),           // default black
    }
}

/// Maps a color name (from Soundcore) to a hex color string.
#[allow(dead_code)]
pub fn color_from_name(name: &str) -> Option<&'static str> {
    match name.to_lowercase().as_str() {
        "black" => Some("#2d2d2d"),
        "white" => Some("#f5f5f5"),
        "blue" => Some("#1a3a5c"),
        "silver" => Some("#c0c0c0"),
        "pink" => Some("#e8a0b4"),
        "green" => Some("#2d5a27"),
        "yellow" => Some("#d4a017"),
        "brown" => Some("#6b4423"),
        "red" => Some("#8b1a1a"),
        "gray" => Some("#808080"),
        "golden" => Some("#c9a84c"),
        "sand_white" => Some("#f5f0e1"),
        "navy_blue" => Some("#1a2744"),
        "coral" => Some("#e8735a"),
        "sky_blue" => Some("#87ceeb"),
        "watermelonred" => Some("#e84057"),
        "khaki" => Some("#c3b091"),
        "babyblue" => Some("#89cff0"),
        "lightgray" => Some("#d3d3d3"),
        "bluesilver" => Some("#7fb5d5"),
        "redgolden" => Some("#c93756"),
        "lightblue" => Some("#add8e6"),
        "purple" => Some("#800080"),
        "camouflage" => Some("#4a5d23"),
        "creamywhite" => Some("#fffdd0"),
        "black_se" => Some("#1a1a1a"),
        "white_red" => Some("#f0f0f0"),
        "purple_yellow" => Some("#9b59b6"),
        "black_golden" => Some("#2d2d2d"),
        "yellow_green" => Some("#9acd32"),
        "black_pure" => Some("#0d0d0d"),
        "green_pure" => Some("#006400"),
        "blue_pure" => Some("#0000cd"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_mapping() {
        assert_eq!(color_from_byte(b'1'), Some("#2d2d2d"));
        assert_eq!(color_from_byte(b'2'), Some("#f5f5f5"));
        assert_eq!(color_from_byte(b'3'), Some("#1a3a5c"));
    }

    #[test]
    fn test_name_mapping() {
        assert_eq!(color_from_name("black"), Some("#2d2d2d"));
        assert_eq!(color_from_name("white"), Some("#f5f5f5"));
        assert_eq!(color_from_name("unknown"), None);
    }
}
