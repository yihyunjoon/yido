use std::collections::HashMap;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LayoutError {
    #[error("배열 TOML을 파싱하지 못했습니다: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("{key}의 jamo는 정확히 한 글자여야 합니다: {value}")]
    InvalidJamo { key: String, value: String },
}

#[derive(Debug, Clone)]
pub struct Layout {
    id: String,
    name: String,
    engine: String,
    keys: HashMap<String, KeyEntry>,
}

#[derive(Debug, Clone)]
pub struct KeyEntry {
    normal: Option<KeyMapping>,
    shift: Option<KeyMapping>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyMapping {
    jamo: char,
    role: JamoRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JamoRole {
    Auto,
    Initial,
    Medial,
    Final,
}

impl Layout {
    pub fn from_toml(source: &str) -> Result<Self, LayoutError> {
        let raw: RawLayoutFile = toml::from_str(source)?;
        let mut keys = HashMap::new();

        for (key, entry) in raw.keys {
            keys.insert(
                key.clone(),
                KeyEntry {
                    normal: entry
                        .normal
                        .map(|mapping| mapping.into_mapping(format!("{key}.normal")))
                        .transpose()?,
                    shift: entry
                        .shift
                        .map(|mapping| mapping.into_mapping(format!("{key}.shift")))
                        .transpose()?,
                },
            );
        }

        Ok(Self {
            id: raw.layout.id,
            name: raw.layout.name,
            engine: raw.layout.engine,
            keys,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn engine(&self) -> &str {
        &self.engine
    }

    pub fn lookup(&self, key: &str, shift: bool) -> Option<&KeyMapping> {
        let entry = self.keys.get(key)?;
        if shift {
            entry.shift.as_ref().or(entry.normal.as_ref())
        } else {
            entry.normal.as_ref()
        }
    }
}

impl KeyMapping {
    pub fn jamo(&self) -> char {
        self.jamo
    }

    pub fn role(&self) -> JamoRole {
        self.role
    }
}

#[derive(Debug, Deserialize)]
struct RawLayoutFile {
    layout: RawLayoutMetadata,
    keys: HashMap<String, RawKeyEntry>,
}

#[derive(Debug, Deserialize)]
struct RawLayoutMetadata {
    id: String,
    name: String,
    engine: String,
}

#[derive(Debug, Deserialize)]
struct RawKeyEntry {
    normal: Option<RawKeyMapping>,
    shift: Option<RawKeyMapping>,
}

#[derive(Debug, Deserialize)]
struct RawKeyMapping {
    jamo: String,
    role: JamoRole,
}

impl RawKeyMapping {
    fn into_mapping(self, key: String) -> Result<KeyMapping, LayoutError> {
        let mut chars = self.jamo.chars();
        let Some(jamo) = chars.next() else {
            return Err(LayoutError::InvalidJamo {
                key,
                value: self.jamo,
            });
        };

        if chars.next().is_some() {
            return Err(LayoutError::InvalidJamo {
                key,
                value: self.jamo,
            });
        }

        Ok(KeyMapping {
            jamo,
            role: self.role,
        })
    }
}
