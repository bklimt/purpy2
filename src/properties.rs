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
