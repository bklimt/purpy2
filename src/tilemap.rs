use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PropertyXml {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@type")]
    typ: String,
    #[serde(rename = "@value")]
    value: String,
}

#[derive(Debug, Deserialize)]
struct PropertiesXml {
    property: Vec<PropertyXml>,
}

#[derive(Debug, Deserialize)]
struct TileSetSourceXml {
    #[serde(rename = "source")]
    source: String,
}

#[derive(Debug, Deserialize)]
struct LayerXml {
    id: i32,
    name: String,
    width: i32,
    height: i32,
    data: String,
}

#[derive(Debug, Deserialize)]
struct ImageXml {
    #[serde(rename = "source")]
    source: String,
}

#[derive(Debug, Deserialize)]
struct ImageLayerXml {
    image: Vec<ImageXml>,
}

#[derive(Debug, Deserialize)]
struct ObjectXml {
    id: i32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    gid: Option<i32>,
    properties: PropertiesXml,
}

#[derive(Debug, Deserialize)]
struct ObjectGroupXml {
    object: Vec<ObjectXml>,
}

#[derive(Debug, Deserialize)]
pub struct TileMapXml {
    #[serde(rename = "@width")]
    width: i32,
    #[serde(rename = "@height")]
    height: i32,
    #[serde(rename = "@tilewidth")]
    tilewidth: i32,
    #[serde(rename = "@tileheight")]
    tileheight: i32,
    #[serde(rename = "@backgroundcolor")]
    backgroundcolor: String,

    tileset: Vec<TileSetSourceXml>,

    properties: PropertyXml,
}
