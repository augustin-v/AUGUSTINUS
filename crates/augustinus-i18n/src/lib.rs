mod strings;

pub use strings::{Language, Strings};

pub fn strings(language: Language) -> Strings {
    strings::strings(language)
}

