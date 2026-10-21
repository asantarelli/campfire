// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Reading the bytes of a library file as text: UTF-8, else Windows-1252, a BOM stripped.

const WINDOWS_1252_HIGH: [char; 32] = [
    '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8d}', 'Ž', '\u{8f}', '\u{90}', '‘', '’', '“', '”', '•', '–', '—',
    '˜', '™', 'š', '›', 'œ', '\u{9d}', 'ž', 'Ÿ',
];

/// Decodes the bytes of a library file. Files are written as UTF-8; one written by another program in the code page of
/// a Western European Windows is read as Windows-1252, which is what its bytes most likely mean.
pub fn decode_library_text(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    match std::str::from_utf8(bytes) {
        Ok(text) => strip_bom(text).to_string(),
        Err(_) => bytes
            .iter()
            .map(|&byte| match byte {
                0x80..=0x9F => WINDOWS_1252_HIGH[(byte - 0x80) as usize],
                other => other as char,
            })
            .collect(),
    }
}

/// A BOM anywhere would read as lyrics in front of a `{title}`.
fn strip_bom(text: &str) -> &str {
    text.strip_prefix('\u{feff}').unwrap_or(text)
}

/// Whether the start of a file looks like text rather than a picture or a program: no more than one control character in
/// fifty bytes, a NUL never.
pub fn is_text_like(prefix: &[u8]) -> bool {
    if prefix.contains(&0) {
        return false;
    }
    let controls = prefix.iter().filter(|&&byte| byte < 0x20 && !matches!(byte, b'\t' | b'\n' | b'\r' | 0x0c)).count();
    controls * 50 <= prefix.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_is_read_as_it_is_and_the_bom_is_dropped() {
        assert_eq!(decode_library_text("Canción".as_bytes()), "Canción");
        assert_eq!(decode_library_text(b"\xEF\xBB\xBF{title: x}"), "{title: x}");
    }

    #[test]
    fn a_western_european_file_is_read_as_windows_1252() {
        assert_eq!(decode_library_text(b"Canci\xF3n \x93hola\x94 \x80"), "Canción “hola” €");
    }

    #[test]
    fn text_is_told_from_binary() {
        assert!(is_text_like(b"{title: x}\n[Am]hola\r\n"));
        assert!(!is_text_like(b"PK\x03\x04\x00\x00"));
        assert!(!is_text_like(&[1, 2, 3, 4, 5, 6, 7, 8, 9]));
    }
}
