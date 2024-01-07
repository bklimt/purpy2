use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Error, Result};
use serde::Deserialize;

use crate::imagemanager::ImageManager;
use crate::properties::{PropertiesXml, PropertyMap};
use crate::slope::Slope;
use crate::sprite::{Animation, Sprite};
use crate::utils::{Direction, Rect};

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

pub struct TileProperties {
    pub alternate: Option<i32>,
    pub condition: Option<String>,
    pub oneway: Option<String>,
    pub slope: bool,
    pub left_y: i32,
    pub right_y: i32,

    pub raw: PropertyMap,
}

impl TryFrom<PropertyMap> for TileProperties {
    type Error = Error;

    fn try_from(value: PropertyMap) -> Result<Self, Self::Error> {
        Ok(TileProperties {
            alternate: value.get_int("alternate")?,
            condition: value.get_string("condition")?.map(str::to_string),
            oneway: value.get_string("oneway")?.map(str::to_string),
            slope: value.get_bool("slope")?.unwrap_or(false),
            left_y: value.get_int("left_y")?.unwrap_or(0),
            right_y: value.get_int("right_y")?.unwrap_or(0),
            raw: value,
        })
    }
}

pub struct TileSet<'a> {
    name: String,
    tilewidth: i32,
    tileheight: i32,
    tilecount: i32,
    columns: i32,
    pub sprite: Sprite<'a>,
    pub animations: HashMap<i32, Animation<'a>>,
    slopes: HashMap<i32, Slope>,
    properties: PropertyMap,
    tile_properties: HashMap<i32, TileProperties>,
}

impl<'a> TileSet<'a> {
    pub fn from_file<'b>(path: &Path, images: &ImageManager<'b>) -> Result<TileSet<'b>> {
        println!("loading tileset from {:?}", path);
        let text = fs::read_to_string(path)?;
        let xml = quick_xml::de::from_str::<TileSetXml>(&text)?;
        Self::from_xml(xml, path, images)
    }

    fn from_xml<'b>(
        xml: TileSetXml,
        path: &Path,
        images: &ImageManager<'b>,
    ) -> Result<TileSet<'b>> {
        let name = xml.name;
        let tilewidth = xml.tilewidth;
        let tileheight = xml.tileheight;
        let tilecount = xml.tilecount;
        let columns = xml.columns;

        let mut sprite: Option<Sprite> = None;
        let mut properties = PropertyMap::new();
        let mut slopes = HashMap::new();
        let mut tile_properties = HashMap::new();

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

        let mut animations = HashMap::new();
        if let Some(animations_path) = properties.get_string("animations")? {
            let animations_path = path
                .parent()
                .context("tileset path is root")?
                .join(animations_path);
            load_tile_animations(&animations_path, images, &mut animations)?;
        }

        Ok(TileSet {
            name,
            tilewidth,
            tileheight,
            tilecount,
            columns,
            sprite,
            animations,
            slopes,
            properties,
            tile_properties,
        })
    }

    fn is_slope(&self, tile_id: i32) -> bool {
        self.slopes.contains_key(&tile_id)
    }

    fn get_slope(&self, tile_id: i32) -> Option<&Slope> {
        self.slopes.get(&tile_id)
    }

    fn update_animations(&mut self) {
        for (_, animation) in self.animations.iter_mut() {
            animation.update();
        }
    }

    fn rows(&self) -> i32 {
        (self.tilecount as f32 / self.columns as f32).ceil() as i32
    }

    pub fn get_source_rect(&self, index: i32) -> Result<Rect> {
        if index < 0 || index > self.tilecount {
            bail!("index out of range");
        }
        let row = index / self.columns;
        let col = index % self.columns;
        let x = col * self.tilewidth;
        let y = row * self.tileheight;
        Ok(Rect {
            x,
            y,
            w: self.tilewidth,
            h: self.tileheight,
        })
    }

    pub fn get_tile_properties(&self, tile_id: i32) -> Option<&TileProperties> {
        self.tile_properties.get(&tile_id)
    }
}

// Loads a directory of animations to replace tiles. """
fn load_tile_animations<'b>(
    path: &Path,
    images: &ImageManager<'b>,
    animations: &mut HashMap<i32, Animation<'b>>,
) -> Result<()> {
    println!("loading tile animations from {:?}", path);
    let files = fs::read_dir(path)?;
    for file in files {
        let file = file?;
        if !file.file_type()?.is_file() {
            println!("skipping non-file {:?}", file);
            continue;
        }
        let filename = file.file_name();
        let filename = filename
            .to_str()
            .context(format!("invalid file name: {:?}", file))?;
        if !filename.ends_with(".png") {
            println!("skipping non-file {:?}", file);
            continue;
        }
        let tile_id = filename[..filename.len() - 4].parse::<i32>()?;
        println!(
            "loading animation for tile {:?} from {:?}",
            tile_id,
            file.path()
        );
        let animation = images.load_animation(&file.path(), 8, 8)?;
        animations.insert(tile_id, animation);
    }
    Ok(())
}
