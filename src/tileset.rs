use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Error, Result};
use log::{debug, info};
use serde::Deserialize;

use crate::imagemanager::ImageLoader;
use crate::properties::{PropertiesXml, PropertyMap};
use crate::slope::Slope;
use crate::smallintmap::SmallIntMap;
use crate::sprite::{Animation, Sprite};
use crate::utils::Rect;

pub type TileIndex = usize;

#[derive(Debug, Deserialize)]
struct ImageXml {
    #[serde(rename = "@source")]
    source: String,
    #[serde(rename = "@width")]
    _width: i32,
    #[serde(rename = "@height")]
    _height: i32,
}

#[derive(Debug, Deserialize)]
struct TileXml {
    #[serde(rename = "@id")]
    id: TileIndex,

    properties: PropertiesXml,
}

#[derive(Debug, Deserialize)]
struct TransformationsXml {
    #[serde(rename = "@hflip")]
    _hflip: i32,
    #[serde(rename = "@vflip")]
    _vflip: i32,
    #[serde(rename = "@rotate")]
    _rotate: i32,
    #[serde(rename = "@preferuntransformed")]
    _preferuntransformed: i32,
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

pub struct TileProperties {
    pub solid: bool,
    pub alternate: Option<TileIndex>,
    pub condition: Option<String>,
    pub oneway: Option<String>,
    pub slope: bool,
    pub left_y: i32,
    pub right_y: i32,
    pub deadly: bool,
    pub switch: Option<String>,

    pub raw: PropertyMap,
}

impl TryFrom<PropertyMap> for TileProperties {
    type Error = Error;

    fn try_from(value: PropertyMap) -> Result<Self, Self::Error> {
        Ok(TileProperties {
            solid: value.get_bool("solid")?.unwrap_or(true),
            alternate: value.get_int("alternate")?.map(|x| x as TileIndex),
            condition: value.get_string("condition")?.map(str::to_string),
            oneway: value.get_string("oneway")?.map(str::to_string),
            slope: value.get_bool("slope")?.unwrap_or(false),
            left_y: value.get_int("left_y")?.unwrap_or(0),
            right_y: value.get_int("right_y")?.unwrap_or(0),
            deadly: value.get_bool("deadly")?.unwrap_or(false),
            switch: value.get_string("switch")?.map(str::to_string),
            raw: value,
        })
    }
}

pub struct TileSetProperties {
    pub animation_path: Option<String>,
}

impl TryFrom<PropertyMap> for TileSetProperties {
    type Error = Error;

    fn try_from(value: PropertyMap) -> Result<Self, Self::Error> {
        Ok(TileSetProperties {
            animation_path: value.get_string("animations")?.map(str::to_string),
        })
    }
}

pub struct TileSet {
    _name: String,
    pub tilewidth: i32,
    pub tileheight: i32,
    tilecount: i32,
    columns: i32,
    pub sprite: Sprite,
    slopes: SmallIntMap<TileIndex, Slope>,
    pub animations: SmallIntMap<TileIndex, Animation>,
    pub properties: TileSetProperties,
    tile_properties: SmallIntMap<TileIndex, TileProperties>,
}

impl TileSet {
    pub fn from_file(path: &Path, images: &mut dyn ImageLoader) -> Result<TileSet> {
        info!("loading tileset from {:?}", path);
        let text = fs::read_to_string(path)?;
        let xml = quick_xml::de::from_str::<TileSetXml>(&text)?;
        Self::from_xml(xml, path, images)
    }

    fn from_xml(xml: TileSetXml, path: &Path, images: &mut dyn ImageLoader) -> Result<TileSet> {
        let name = xml.name;
        let tilewidth = xml.tilewidth;
        let tileheight = xml.tileheight;
        let tilecount = xml.tilecount;
        let columns = xml.columns;

        let mut sprite: Option<Sprite> = None;
        let mut properties = PropertyMap::new();
        let mut slopes = SmallIntMap::new();
        let mut tile_properties = SmallIntMap::new();

        for field in xml.fields {
            match field {
                TileSetXmlField::Image(img_xml) => {
                    let img_path = path
                        .parent()
                        .context(anyhow!("tileset path is root"))?
                        .join(img_xml.source);
                    sprite = Some(images.load_sprite(&img_path)?);
                }
                TileSetXmlField::Properties(props_xml) => {
                    properties = props_xml.try_into()?;
                }
                TileSetXmlField::Tile(tile_xml) => {
                    let id = tile_xml.id;
                    let props: PropertyMap = tile_xml.properties.try_into()?;
                    let props: TileProperties = props.try_into()?;
                    if props.slope {
                        slopes.insert(id, Slope::new(&props)?);
                    }
                    tile_properties.insert(id, props);
                }
                _ => {}
            }
        }
        //println!("tileset properties: {:?}", properties);
        //println!("tile properties: {:?}", tile_properties);

        let sprite = sprite.context("missing image")?;
        let properties: TileSetProperties = properties.try_into()?;

        let mut animations = SmallIntMap::new();
        if let Some(animations_path) = properties.animation_path.as_ref() {
            let animations_path = path
                .parent()
                .context("tileset path is root")?
                .join(animations_path);
            load_tile_animations(&animations_path, images, &mut animations)?;
        }

        Ok(TileSet {
            _name: name,
            tilewidth,
            tileheight,
            tilecount,
            columns,
            sprite,
            slopes,
            animations,
            properties,
            tile_properties,
        })
    }

    pub fn get_slope(&self, tile_id: TileIndex) -> Option<&Slope> {
        self.slopes.get(tile_id)
    }

    fn _rows(&self) -> i32 {
        (self.tilecount as f32 / self.columns as f32).ceil() as i32
    }

    pub fn get_source_rect(&self, index: TileIndex) -> Rect {
        let index = index as i32;
        if index < 0 || index > self.tilecount {
            panic!("index out of range");
        }
        let row = index / self.columns;
        let col = index % self.columns;
        let x = col * self.tilewidth;
        let y = row * self.tileheight;
        Rect {
            x,
            y,
            w: self.tilewidth,
            h: self.tileheight,
        }
    }

    pub fn get_tile_properties(&self, tile_id: TileIndex) -> Option<&TileProperties> {
        self.tile_properties.get(tile_id)
    }
}

// Loads a directory of animations to replace tiles. """
fn load_tile_animations(
    path: &Path,
    images: &mut dyn ImageLoader,
    animations: &mut SmallIntMap<TileIndex, Animation>,
) -> Result<()> {
    info!("loading tile animations from {:?}", path);
    let files = fs::read_dir(path)?;
    for file in files {
        let file = file?;
        if !file.file_type()?.is_file() {
            debug!("skipping non-file {:?}", file);
            continue;
        }
        let filename = file.file_name();
        let filename = filename
            .to_str()
            .context(format!("invalid file name: {:?}", file))?;
        if !filename.ends_with(".png") {
            debug!("skipping non-file {:?}", file);
            continue;
        }
        let tile_id = filename[..filename.len() - 4].parse::<TileIndex>()?;
        info!(
            "loading animation for tile {:?} from {:?}",
            tile_id,
            file.path()
        );
        let animation = images.load_animation(&file.path(), 8, 8)?;
        animations.insert(tile_id, animation);
    }
    Ok(())
}
