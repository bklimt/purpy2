use crate::properties::PropertiesXml;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TileSetSourceXml {
    #[serde(rename = "@source")]
    source: String,
}

#[derive(Debug, Deserialize)]
struct DataXml {
    #[serde(rename = "@encoding")]
    encoding: String,

    #[serde(rename = "$value")]
    data: String,
}

#[derive(Debug, Deserialize)]
struct LayerXml {
    #[serde(rename = "@id")]
    id: i32,
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@width")]
    width: i32,
    #[serde(rename = "@height")]
    height: i32,

    data: DataXml,
}

#[derive(Debug, Deserialize)]
struct ImageXml {
    #[serde(rename = "@source")]
    source: String,
}

#[derive(Debug, Deserialize)]
struct ImageLayerXml {
    image: Vec<ImageXml>,
}

#[derive(Debug, Deserialize)]
struct ObjectXml {
    #[serde(rename = "@id")]
    id: i32,
    #[serde(rename = "@x")]
    x: i32,
    #[serde(rename = "@y")]
    y: i32,
    #[serde(rename = "@width")]
    width: i32,
    #[serde(rename = "@height")]
    height: i32,
    #[serde(rename = "@gid")]
    gid: Option<i32>,

    properties: Option<PropertiesXml>,
}

#[derive(Debug, Deserialize)]
struct ObjectGroupXml {
    #[serde(default)]
    object: Vec<ObjectXml>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TileMapXmlField {
    TileSet(TileSetSourceXml),
    Properties(PropertiesXml),
    ObjectGroup(ObjectGroupXml),
    Layer(LayerXml),
    ImageLayer(ImageLayerXml),
}

fn default_backgroundcolor() -> String {
    "#000000".to_string()
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
    #[serde(rename = "@backgroundcolor", default = "default_backgroundcolor")]
    backgroundcolor: String,

    #[serde(rename = "$value")]
    fields: Vec<TileMapXmlField>,
}
