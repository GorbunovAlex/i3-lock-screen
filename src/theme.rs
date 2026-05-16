#[derive(Clone, Copy)]
pub struct Theme {
    pub base: u32,
    pub mantle: u32,
    pub teal: u32,
    pub sky: u32,
    pub text: u32,
    pub subtext: u32,
    pub red: u32,
    pub peach: u32,
}

impl Theme {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "mocha" => Some(Self::mocha()),
            "macchiato" => Some(Self::macchiato()),
            "frappe" | "frappé" => Some(Self::frappe()),
            "latte" => Some(Self::latte()),
            _ => None,
        }
    }

    /// Catppuccin Mocha — darkest flavor
    pub const fn mocha() -> Self {
        Self {
            base: 0xFF1E1E2E,
            mantle: 0xE6181825,
            teal: 0xFF94E2D5,
            sky: 0xFF89DCEB,
            text: 0xFFCDD6F4,
            subtext: 0xFFA6ADC8,
            red: 0xFFF38BA8,
            peach: 0xFFFAB387,
        }
    }

    /// Catppuccin Macchiato
    pub const fn macchiato() -> Self {
        Self {
            base: 0xFF24273A,
            mantle: 0xE61E2030,
            teal: 0xFF8BD5CA,
            sky: 0xFF91D7E3,
            text: 0xFFCAD3F5,
            subtext: 0xFFA5ADCB,
            red: 0xFFED8796,
            peach: 0xFFF5A97F,
        }
    }

    /// Catppuccin Frappé
    pub const fn frappe() -> Self {
        Self {
            base: 0xFF303446,
            mantle: 0xE6292C3C,
            teal: 0xFF81C8BE,
            sky: 0xFF99D1DB,
            text: 0xFFC6D0F5,
            subtext: 0xFFA5ADCE,
            red: 0xFFE78284,
            peach: 0xFFEF9F76,
        }
    }

    /// Catppuccin Latte — light flavor
    pub const fn latte() -> Self {
        Self {
            base: 0xFFEFF1F5,
            mantle: 0xE6E6E9EF,
            teal: 0xFF179299,
            sky: 0xFF04A5E5,
            text: 0xFF4C4F69,
            subtext: 0xFF6C6F85,
            red: 0xFFD20F39,
            peach: 0xFFFE640B,
        }
    }
}

pub const FUNNY_PHRASES: &[&str] = &[
    // Classics
    "It works on my machine...",
    "Have you tried turning it off and on again?",
    "sudo make me a sandwich?",
    "Nice try, script kiddie.",
    "Your password is in another castle.",
    // HTTP jokes
    "404: Password not found.",
    "401: Unauthorized. Obviously.",
    "403: Forbidden. Go away.",
    "418: I'm a teapot. You're locked out.",
    // Unix / sysadmin
    "SEGFAULT: User not found.",
    "Layer 8 Issue Detected.",
    "Kernel panic — not syncing: wrong password.",
    "Bus error: you got on the wrong bus.",
    "rm -rf /home/intruder",
    "Permission denied (publickey, password, your face).",
    "ssh: connect to host localhost: Connection refused.",
    // Git
    "Git blame: You.",
    "git push --force? Not this time.",
    "fatal: authentication failed.",
    "Merge conflict: your password vs. the correct one.",
    // Programming
    "Unexpected token: You.",
    "Compiling... just kidding, wrong password.",
    "Null pointer exception: you.",
    "Stack overflow detected in your brain.",
    "Undefined behavior: your password.",
    "error[E0308]: mismatched types — expected password.",
    "Cannot move out of borrowed content: the door.",
    "while true { reject(); }",
    "TypeError: expected str, got garbage.",
    "IndexError: password out of range.",
    // Rust-specific
    "borrow checker: access denied.",
    "lifetime 'password does not live long enough.",
    "error[E0502]: cannot borrow 'door' as mutable.",
    // Pop culture / misc
    "You shall not pass!",
    "This is fine. (It is not fine.)",
    "Have you considered just... not?",
    "Authentication failed. Obviously.",
    "Brute force detected. Deploying honeypot.",
    "malloc failed: insufficient cleverness.",
    "Your face is not a valid certificate.",
    "Type 'help' to get no help whatsoever.",
    "The cake is a lie. So is your password.",
    "BIOS password? No. Just no.",
];
