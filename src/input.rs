pub fn keycode_to_char(code: u8, shift: bool) -> Option<char> {
    let base = match code {
        10..=19 => "1234567890".chars().nth((code - 10) as usize),
        24..=33 => "qwertyuiop".chars().nth((code - 24) as usize),
        38..=46 => "asdfghjkl".chars().nth((code - 38) as usize),
        52..=58 => "zxcvbnm".chars().nth((code - 52) as usize),
        65 => Some(' '),
        20 => Some('-'),
        21 => Some('='),
        34 => Some('['),
        35 => Some(']'),
        51 => Some('\\'),
        47 => Some(';'),
        48 => Some('\''),
        59 => Some(','),
        60 => Some('.'),
        61 => Some('/'),
        _ => None,
    }?;

    Some(if shift { shift_char(base) } else { base })
}

fn shift_char(c: char) -> char {
    match c {
        '1' => '!',
        '2' => '@',
        '3' => '#',
        '4' => '$',
        '5' => '%',
        '6' => '^',
        '7' => '&',
        '8' => '*',
        '9' => '(',
        '0' => ')',
        '-' => '_',
        '=' => '+',
        '[' => '{',
        ']' => '}',
        '\\' => '|',
        ';' => ':',
        '\'' => '"',
        ',' => '<',
        '.' => '>',
        '/' => '?',
        c => c.to_ascii_uppercase(),
    }
}
