// ANSI color codes for terminal output

pub struct ANSI;

impl ANSI {
    pub const RED: &'static str = "\u{001B}[31m";
    pub const GREEN: &'static str = "\u{001B}[32m";
    pub const YELLOW: &'static str = "\u{001B}[33m";
    pub const BLUE: &'static str = "\u{001B}[34m";
    pub const MAGENTA: &'static str = "\u{001B}[35m";
    pub const CYAN: &'static str = "\u{001B}[36m";
    pub const WHITE: &'static str = "\u{001B}[37m";
    pub const DEFAULT: &'static str = "\u{001B}[39m";
}