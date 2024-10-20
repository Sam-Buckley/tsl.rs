pub trait Color {
    fn colorize(&self) -> String;
}

pub struct Red;
pub struct Green;
pub struct Blue;
pub struct Yellow;

impl Color for Red {
    fn colorize(&self) -> String {
        "\x1b[31m".to_string()
    }
}

impl Color for Green {
    fn colorize(&self) -> String {
        "\x1b[32m".to_string()
    }
}

impl Color for Blue {
    fn colorize(&self) -> String {
        "\x1b[34m".to_string()
    }
}

impl Color for Yellow {
    fn colorize(&self) -> String {
        "\x1b[33m".to_string()
    }
}

pub fn colorize<T: Color>(color: T, text: &str) -> String {
    format!("{}{}\x1b[0m", color.colorize(), text)
}

#[macro_export]
macro_rules! colorize {
    ($color:ident, $text:expr) => {
        $crate::color::colorize($color, $text)
    };
}
