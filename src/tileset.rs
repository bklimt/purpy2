use crate::properties::PropertiesXml;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ImageXml {
    #[serde(rename = "@source")]
    source: String,
    #[serde(rename = "@width")]
    width: i32,
    #[serde(rename = "@height")]
    height: i32,
}

#[derive(Debug, Deserialize)]
struct TileXml {
    #[serde(rename = "@id")]
    id: i32,

    properties: PropertiesXml,
}

#[derive(Debug, Deserialize)]
struct TransformationsXml {
    #[serde(rename = "@hflip")]
    hflip: i32,
    #[serde(rename = "@vflip")]
    vflip: i32,
    #[serde(rename = "@rotate")]
    rotate: i32,
    #[serde(rename = "@preferuntransformed")]
    preferuntransformed: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TileSetXmlField {
    Image(ImageXml),
    Properties(PropertiesXml),
    Tile(TileXml),
    Transformations(TransformationsXml),
    WangSets,
}

#[derive(Debug, Deserialize)]
pub struct TileSetXml {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@tilewidth")]
    tilewidth: i32,
    #[serde(rename = "@tileheight")]
    tileheight: i32,
    #[serde(rename = "@tilecount")]
    tilecount: i32,
    #[serde(rename = "@columns")]
    columns: i32,

    #[serde(rename = "$value")]
    fields: Vec<TileSetXmlField>,
}

pub struct TileSet {
    name: String,
    tilewidth: i32,
    tileheight: i32,
    tilecount: i32,
    columns: i32,
    image: ImageXml,
}
