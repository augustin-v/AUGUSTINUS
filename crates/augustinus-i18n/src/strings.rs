use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    En,
    Fr,
    Ja,
}

#[derive(Debug, Clone, Copy)]
pub struct Strings {
    pub app_name: &'static str,
}

pub fn strings(language: Language) -> Strings {
    match language {
        Language::En => Strings { app_name: "AUGUSTINUS" },
        Language::Fr => Strings { app_name: "AUGUSTINUS" },
        Language::Ja => Strings { app_name: "AUGUSTINUS" },
    }
}

