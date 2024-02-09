use std::collections::HashMap;

use anyhow::{anyhow, bail, Result};
use serde::Deserialize;

fn default_type() -> String {
    "string".to_owned()
}

#[derive(Debug, Deserialize)]
struct PropertyXml {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@type", default = "default_type")]
    typ: String,
    #[serde(rename = "@value")]
    value: String,
}

#[derive(Debug, Deserialize)]
pub struct PropertiesXml {
    property: Vec<PropertyXml>,
}

#[derive(Debug, Clone)]
enum PropertyValue {
    Int(i32),
    String(String),
    Bool(bool),
}

#[derive(Debug)]
pub struct PropertyMap(HashMap<String, PropertyValue>);

impl PropertyMap {
    pub fn new() -> Self {
        PropertyMap(HashMap::new())
    }

    pub fn set_defaults(&mut self, other: &PropertyMap) {
        for (k, v) in other.0.iter() {
            if !self.0.contains_key(k) {
                self.0.insert(k.clone(), v.clone());
            }
        }
    }

    pub fn get_int(&self, k: &str) -> Result<Option<i32>> {
        self.0
            .get(k)
            .map(|v| match v {
                PropertyValue::Int(n) => Ok(*n),
                _ => Err(anyhow!("property {k} is not an int")),
            })
            .transpose()
    }

    pub fn get_string(&self, k: &str) -> Result<Option<&str>> {
        self.0
            .get(k)
            .map(|v| match v {
                PropertyValue::String(s) => Ok(s.as_str()),
                _ => Err(anyhow!("property {k} is not a string")),
            })
            .transpose()
    }

    pub fn get_bool(&self, k: &str) -> Result<Option<bool>> {
        self.0
            .get(k)
            .map(|v| match v {
                PropertyValue::Bool(b) => Ok(*b),
                _ => Err(anyhow!("property {k} is not a bool")),
            })
            .transpose()
    }
}

impl Default for PropertyMap {
    fn default() -> Self {
        Self::new()
    }
}

impl TryFrom<PropertyXml> for PropertyValue {
    type Error = anyhow::Error;

    fn try_from(value: PropertyXml) -> Result<Self, Self::Error> {
        Ok(match value.typ.as_ref() {
            "int" => PropertyValue::Int(value.value.parse()?),
            "string" => PropertyValue::String(value.value.to_owned()),
            "bool" => PropertyValue::Bool(value.value == "true"),
            _ => bail!("invalid property type: {:?}", &value),
        })
    }
}

impl TryFrom<PropertiesXml> for PropertyMap {
    type Error = anyhow::Error;

    fn try_from(value: PropertiesXml) -> Result<Self, Self::Error> {
        let mut map = HashMap::new();
        for prop in value.property {
            let key = prop.name.to_owned();
            let value = prop.try_into()?;
            map.insert(key, value);
        }
        Ok(PropertyMap(map))
    }
}
