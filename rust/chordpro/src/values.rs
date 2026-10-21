// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Reads the values of `{tempo}`, `{time}`, `{duration}` and `{capo}`, written by hand without a regex engine.

/// The value of `{tempo}`, which people write every way a score does: `120`, `120 bpm`, `♩ = 96`, `97.5`. The first number
/// in it counts, rounded to a whole one; a value with no number, or one that is not positive, is no tempo.
pub fn parse_tempo(text: &str) -> Option<i32> {
    let chars: Vec<char> = text.chars().collect();
    let start = chars.iter().position(char::is_ascii_digit)?;
    let mut end = start;
    while chars.get(end).is_some_and(char::is_ascii_digit) {
        end += 1;
    }
    // `97.5` or `97,5`, the fraction only where digits follow the separator.
    if matches!(chars.get(end), Some('.' | ',')) && chars.get(end + 1).is_some_and(char::is_ascii_digit) {
        end += 1;
        while chars.get(end).is_some_and(char::is_ascii_digit) {
            end += 1;
        }
    }
    let number: String = chars[start..end].iter().map(|&c| if c == ',' { '.' } else { c }).collect();
    let value: f64 = number.parse().ok()?;
    // Kotlin's `roundToInt` rounds half up.
    let rounded = (value + 0.5).floor() as i32;
    (rounded > 0).then_some(rounded)
}

/// The value of `{time}`: a fraction (`3/4`, `6/8`), or the common time and cut time marks a score uses (`C` for 4/4, `C|`
/// for 2/2). A bar of 1 to 16 beats of a whole, half, quarter, eighth or sixteenth note is a signature; anything else is none.
/// Returns the beats of a bar and the note each beat is.
pub fn parse_time(text: &str) -> Option<(i32, i32)> {
    let value = text.trim();
    match value {
        "C" => return Some((4, 4)),
        "C|" | "¢" => return Some((2, 2)),
        _ => {}
    }
    let (beats, unit) = value.split_once('/')?;
    let (beats, unit) = (beats.trim_end(), unit.trim_start());
    let is_number = |text: &str| (1..=2).contains(&text.len()) && text.chars().all(|c| c.is_ascii_digit());
    if !is_number(beats) || !is_number(unit) {
        return None;
    }
    let (beats, unit): (i32, i32) = (beats.parse().ok()?, unit.parse().ok()?);
    ((1..=16).contains(&beats) && [1, 2, 4, 8, 16].contains(&unit)).then_some((beats, unit))
}

/// The seconds `{duration}` declares: seconds (`268`), minutes and seconds (`4:28`) or hours, minutes and seconds
/// (`1:02:03`). Anything else (`about 4 min`, `4:5`, `0`) is no duration at all, never guessed at.
pub fn parse_duration(text: &str) -> Option<u64> {
    let value = text.trim();
    let parts: Vec<&str> = value.split(':').collect();
    let digits = |part: &str, min: usize, max: usize| (min..=max).contains(&part.len()) && part.chars().all(|c| c.is_ascii_digit());
    let seconds_part = |part: &str| digits(part, 2, 2) && part.parse::<u64>().is_ok_and(|n| n < 60);
    let seconds = match parts.as_slice() {
        [seconds] if digits(seconds, 1, 6) => seconds.parse::<u64>().ok()?,
        [minutes, seconds] if digits(minutes, 1, 4) && seconds_part(seconds) => minutes.parse::<u64>().ok()? * 60 + seconds.parse::<u64>().ok()?,
        [hours, minutes, seconds] if digits(hours, 1, 3) && seconds_part(minutes) && seconds_part(seconds) => {
            hours.parse::<u64>().ok()? * 3600 + minutes.parse::<u64>().ok()? * 60 + seconds.parse::<u64>().ok()?
        }
        _ => return None,
    };
    (seconds > 0).then_some(seconds)
}

/// `seconds` as `m:ss`, or `h:mm:ss` from an hour on.
pub fn format_duration(seconds: u64) -> String {
    let (hours, minutes, seconds) = (seconds / 3600, seconds % 3600 / 60, seconds % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tempo_is_the_first_number() {
        assert_eq!(parse_tempo("120"), Some(120));
        assert_eq!(parse_tempo("120 bpm"), Some(120));
        assert_eq!(parse_tempo("♩ = 96"), Some(96));
        assert_eq!(parse_tempo("97.5"), Some(98));
        assert_eq!(parse_tempo("97,4"), Some(97));
        assert_eq!(parse_tempo("fast"), None);
        assert_eq!(parse_tempo("0"), None);
    }

    #[test]
    fn a_time_signature_is_a_fraction_or_a_mark() {
        assert_eq!(parse_time("3/4"), Some((3, 4)));
        assert_eq!(parse_time("6 / 8"), Some((6, 8)));
        assert_eq!(parse_time("C"), Some((4, 4)));
        assert_eq!(parse_time("C|"), Some((2, 2)));
        assert_eq!(parse_time("17/4"), None);
        assert_eq!(parse_time("4/3"), None);
        assert_eq!(parse_time("waltz"), None);
    }

    #[test]
    fn a_duration_is_seconds_or_a_clock() {
        assert_eq!(parse_duration("268"), Some(268));
        assert_eq!(parse_duration("4:28"), Some(268));
        assert_eq!(parse_duration("1:02:03"), Some(3723));
        assert_eq!(parse_duration("4:5"), None);
        assert_eq!(parse_duration("0"), None);
        assert_eq!(parse_duration("about 4 min"), None);
        assert_eq!(format_duration(268), "4:28");
        assert_eq!(format_duration(3723), "1:02:03");
    }
}
