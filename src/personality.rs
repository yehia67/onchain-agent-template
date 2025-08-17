use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct Personality {
    pub name: String,
    pub role: String,
    pub style: Style,
    pub rules: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Style {
    pub tone: String,
    pub formality: String,
    pub domain_focus: Vec<String>,
}

pub fn load_personality(path: &str) -> anyhow::Result<Personality> {
    let data = fs::read_to_string(path)?;
    let persona: Personality = serde_json::from_str(&data)?;
    Ok(persona)
}
