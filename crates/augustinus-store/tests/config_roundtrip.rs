use augustinus_store::config::{AppConfig, Language};

#[test]
fn config_roundtrips_toml() {
    let c = AppConfig {
        language: Language::Ja,
        shell: "/bin/bash".into(),
        git_repo: None,
        agents_cmd: None,
    };
    let toml = c.to_toml_string();
    let parsed = AppConfig::from_toml_str(&toml).unwrap();
    assert_eq!(parsed.language, Language::Ja);
}
