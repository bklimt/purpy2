use std::num::ParseIntError;
use std::path::Path;
use std::str::FromStr;

use anyhow::{anyhow, Context, Error, Result};
use log::{debug, info};
use serde::Deserialize;

use crate::filemanager::{DirEntryType, FileManager};
use crate::geometry::{Pixels, Rect};
use crate::imagemanager::ImageLoader;
use crate::properties::{PropertiesXml, PropertyMap};
use crate::slope::Slope;
use crate::smallintmap::SmallIntMap;
use crate::sprite::{Animation, Sprite};
use crate::tilemap::TileIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalTileIndex(usize);

impl From<LocalTileIndex> for usize {
    fn from(value: LocalTileIndex) -> Self {
        value.0
    }
}

impl From<usize> for LocalTileIndex {
    fn from(value: usize) -> Self {
        LocalTileIndex(value)
    }
}

impl FromStr for LocalTileIndex {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(LocalTileIndex(s.parse::<usize>()?))
    }
}

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
    id: usize,

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
    // switches
    pub switch: Option<String>,
    pub condition: Option<String>,
    pub alternate: Option<LocalTileIndex>,
    // oneway
    pub oneway: Option<String>,
    // slopes
    pub slope: bool,
    pub left_y: Pixels,
    pub right_y: Pixels,
    // custom hitboxes
    pub hitbox_top: Pixels,
    pub hitbox_left: Pixels,
    pub hitbox_right: Pixels,
    pub hitbox_bottom: Pixels,
    // spikes
    pub deadly: bool,

    pub raw: PropertyMap,
}

impl TryFrom<PropertyMap> for TileProperties {
    type Error = Error;

    fn try_from(value: PropertyMap) -> Result<Self, Self::Error> {
        Ok(TileProperties {
            solid: value.get_bool("solid")?.unwrap_or(true),
            alternate: value
                .get_int("alternate")?
                .map(|x| LocalTileIndex(x as usize)),
            condition: value.get_string("condition")?.map(str::to_string),
            oneway: value.get_string("oneway")?.map(str::to_string),
            slope: value.get_bool("slope")?.unwrap_or(false),
            left_y: Pixels::new(value.get_int("left_y")?.unwrap_or(0)),
            right_y: Pixels::new(value.get_int("right_y")?.unwrap_or(0)),
            hitbox_top: Pixels::new(value.get_int("hitbox_top")?.unwrap_or(0)),
            hitbox_left: Pixels::new(value.get_int("hitbox_left")?.unwrap_or(0)),
            hitbox_right: Pixels::new(value.get_int("hitbox_right")?.unwrap_or(0)),
            hitbox_bottom: Pixels::new(value.get_int("hitbox_bottom")?.unwrap_or(0)),
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
    firstgid: TileIndex,
    pub tilewidth: Pixels,
    pub tileheight: Pixels,
    tilecount: i32,
    columns: i32,
    pub sprite: Sprite,
    slopes: SmallIntMap<LocalTileIndex, Slope>,
    pub animations: SmallIntMap<LocalTileIndex, Animation>,
    pub properties: TileSetProperties,
    tile_properties: SmallIntMap<LocalTileIndex, TileProperties>,
}

impl TileSet {
    pub fn from_file(
        path: &Path,
        firstgid: TileIndex,
        files: &FileManager,
        images: &mut dyn ImageLoader,
    ) -> Result<TileSet> {
        info!("loading tileset from {:?}", path);
        let text = files
            .read_to_string(path)
            .map_err(|e| anyhow!("unable to open {:?}: {}", path, e))?;
        let xml = quick_xml::de::from_str::<TileSetXml>(&text)?;
        Self::from_xml(xml, path, firstgid, files, images)
    }

    fn from_xml(
        xml: TileSetXml,
        path: &Path,
        firstgid: TileIndex,
        files: &FileManager,
        images: &mut dyn ImageLoader,
    ) -> Result<TileSet> {
        let name = xml.name;
        let tilewidth = Pixels::new(xml.tilewidth);
        let tileheight = Pixels::new(xml.tileheight);
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
                    let id = LocalTileIndex(tile_xml.id);
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
            load_tile_animations(&animations_path, files, images, &mut animations)?;
        }

        Ok(TileSet {
            _name: name,
            firstgid,
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

    pub fn get_local_tile_index(&self, tile_gid: TileIndex) -> Option<LocalTileIndex> {
        let tile_gid: usize = tile_gid.into();
        let firstgid: usize = self.firstgid.into();
        if tile_gid >= firstgid {
            Some((tile_gid - firstgid).into())
        } else {
            None
        }
    }

    pub fn get_global_tile_index(&self, tile_id: LocalTileIndex) -> TileIndex {
        let tile_id: usize = tile_id.into();
        let firstgid: usize = self.firstgid.into();
        (firstgid + tile_id).into()
    }

    pub fn gid_sort_key(&self) -> i32 {
        let key: usize = self.firstgid.into();
        let key = key as i32;
        -key
    }

    pub fn get_slope(&self, tile_id: LocalTileIndex) -> Option<&Slope> {
        self.slopes.get(tile_id)
    }

    fn _rows(&self) -> i32 {
        (self.tilecount as f32 / self.columns as f32).ceil() as i32
    }

    pub fn get_source_rect(&self, index: LocalTileIndex) -> Rect<Pixels> {
        let index = index.0 as i32;
        if index < 0 || index > self.tilecount {
            panic!("index out of range");
        }
        let row = index / self.columns;
        let col = index % self.columns;
        let x = self.tilewidth * col;
        let y = self.tileheight * row;
        Rect {
            x,
            y,
            w: self.tilewidth,
            h: self.tileheight,
        }
    }

    pub fn get_tile_properties(&self, tile_id: LocalTileIndex) -> Option<&TileProperties> {
        self.tile_properties.get(tile_id)
    }
}

// Loads a directory of animations to replace tiles.
fn load_tile_animations(
    path: &Path,
    file_manager: &FileManager,
    images: &mut dyn ImageLoader,
    animations: &mut SmallIntMap<LocalTileIndex, Animation>,
) -> Result<()> {
    info!("loading tile animations from {:?}", path);
    let files = file_manager.read_dir(path)?;
    for file in files {
        if !matches!(file.file_type, DirEntryType::File) {
            debug!("skipping non-file {:?}", file.full_path);
            continue;
        }
        let filename = file.name;
        if !filename.ends_with(".png") {
            debug!("skipping non-file {:?}", file.full_path);
            continue;
        }
        let tile_id = filename[..filename.len() - 4].parse::<LocalTileIndex>()?;
        info!(
            "loading animation for tile {:?} from {:?}",
            tile_id, file.full_path
        );
        let animation = images.load_animation(&file.full_path, Pixels::new(8), Pixels::new(8))?;
        animations.insert(tile_id, animation);
    }
    Ok(())
}
