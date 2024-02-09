use std::cmp::Ordering;
use std::num::ParseIntError;
use std::ops::{Index, IndexMut};
use std::path::Path;
use std::str::FromStr;

use crate::constants::MAX_GRAVITY;
use crate::filemanager::FileManager;
use crate::geometry::{Pixels, Point, Rect, Subpixels};
use crate::imagemanager::ImageLoader;
use crate::properties::{PropertiesXml, PropertyMap};
use crate::rendercontext::{RenderContext, RenderLayer};
use crate::slope::Slope;
use crate::smallintset::SmallIntSet;
use crate::sprite::{Animation, Sprite};
use crate::switchstate::SwitchState;
use crate::tileset::{LocalTileIndex, TileProperties, TileSet};
use crate::utils::{cmp_in_direction, try_move_to_bounds, Color, Direction};

use anyhow::{anyhow, bail, Context, Result};
use log::info;
use num_traits::Zero;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TileSetSourceXml {
    #[serde(rename = "@source")]
    source: String,

    #[serde(rename = "@firstgid")]
    firstgid: usize,
}

#[derive(Debug, Deserialize)]
struct DataXml {
    #[serde(rename = "@encoding")]
    _encoding: String,

    #[serde(rename = "$value")]
    data: String,
}

#[derive(Debug, Deserialize)]
struct LayerXml {
    #[serde(rename = "@id")]
    id: u32,
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@width")]
    width: u32,
    #[serde(rename = "@height")]
    height: u32,

    data: DataXml,

    properties: Option<PropertiesXml>,
}

#[derive(Debug, Deserialize)]
struct ImageXml {
    #[serde(rename = "@source")]
    source: String,
}

#[derive(Debug, Deserialize)]
struct ImageLayerXml {
    #[serde(rename = "@id")]
    _id: i32,
    #[serde(rename = "@offsetx")]
    _offsetx: Option<String>,
    #[serde(rename = "@offsety")]
    _offsety: Option<String>,

    image: ImageXml,
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
    width: Option<i32>,
    #[serde(rename = "@height")]
    height: Option<i32>,
    #[serde(rename = "@gid")]
    gid: Option<u32>,

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
struct TileMapXml {
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

    properties: Option<PropertiesXml>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileIndex(usize);

impl From<TileIndex> for usize {
    fn from(value: TileIndex) -> Self {
        value.0
    }
}

impl From<usize> for TileIndex {
    fn from(value: usize) -> Self {
        TileIndex(value)
    }
}

impl FromStr for TileIndex {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TileIndex(s.parse::<usize>()?))
    }
}

struct ImageLayer {
    surface: Sprite,
}

impl ImageLayer {
    fn from_xml(
        xml: ImageLayerXml,
        path: &Path,
        images: &mut dyn ImageLoader,
    ) -> Result<ImageLayer> {
        let path = path
            .parent()
            .context("xml file is root")?
            .join(xml.image.source);
        let surface = images.load_sprite(&path)?;
        Ok(ImageLayer { surface })
    }
}

struct TileLayer {
    _id: u32,
    _name: String,
    _width: u32,
    _height: u32,
    data: Vec<Vec<TileIndex>>,
    player: bool,
}

impl TileLayer {
    fn from_xml(xml: LayerXml) -> Result<TileLayer> {
        let id = xml.id;
        let name = xml.name;
        let width = xml.width;
        let height = xml.height;

        let props: Option<PropertyMap> = xml.properties.map(|x| x.try_into()).transpose()?;
        let props = props.unwrap_or_default();
        let player = props.get_bool("player")?.unwrap_or(false);

        let mut data = Vec::new();
        for line in xml.data.data.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let mut row = Vec::new();
            for part in line.split(',') {
                if part.is_empty() {
                    continue;
                }
                row.push(part.parse().context(format!("parsing {:?}", part))?);
            }
            if row.len() as u32 != width {
                bail!("row len = {}, but width = {}", row.len(), width);
            }
            data.push(row);
        }
        if data.len() as u32 != height {
            bail!("row data height = {}, but height = {}", data.len(), height);
        }

        Ok(TileLayer {
            _id: id,
            _name: name,
            _width: width,
            _height: height,
            data,
            player,
        })
    }

    fn get(&self, row: usize, col: usize) -> Option<&TileIndex> {
        self.data.get(row).and_then(|r| r.get(col))
    }

    fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut TileIndex> {
        self.data.get_mut(row).and_then(|r| r.get_mut(col))
    }
}

impl Index<(usize, usize)> for TileLayer {
    type Output = TileIndex;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        self.get(index.0, index.1)
            .with_context(|| anyhow!("indices must be valid: ({}, {})", index.0, index.1))
            .expect("indices must be valid")
    }
}

impl IndexMut<(usize, usize)> for TileLayer {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        self.get_mut(index.0, index.1)
            .expect("indices must be valid")
    }
}

enum Layer {
    Tile(TileLayer),
    Image(ImageLayer),
}

#[derive(Debug, Clone, Copy)]
pub enum Overflow {
    Oscillate,
    Wrap,
    Clamp,
}

impl FromStr for Overflow {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "oscillate" => Ok(Overflow::Oscillate),
            "wrap" => Ok(Overflow::Wrap),
            "clamp" => Ok(Overflow::Clamp),
            _ => Err(anyhow!("invalid overflow type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConveyorDirection {
    Left,
    Right,
}

impl FromStr for ConveyorDirection {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "W" => Ok(ConveyorDirection::Left),
            "E" => Ok(ConveyorDirection::Right),
            _ => Err(anyhow!("invalid conveyor direction: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonType {
    OneShot,
    Toggle,
    Momentary,
    Smart,
}

impl FromStr for ButtonType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "oneshot" => ButtonType::OneShot,
            "toggle" => ButtonType::Toggle,
            "momentary" => ButtonType::Momentary,
            "smart" => ButtonType::Smart,
            _ => bail!("invalid button type: {}", s),
        })
    }
}

#[derive(Debug)]
pub struct MapObjectProperties {
    // Types
    pub platform: bool,
    pub bagel: bool,
    pub spring: bool,
    pub button: bool,
    pub door: bool,
    pub star: bool,
    pub spawn: bool,
    // Tiles
    pub solid: bool,
    // Map Areas
    pub preferred_x: Option<Pixels>,
    pub preferred_y: Option<Pixels>,
    // Platforms
    pub distance: i32,
    pub speed: Option<Pixels>,
    pub condition: Option<String>,
    pub overflow: Overflow,
    pub direction: Direction,
    pub convey: Option<ConveyorDirection>,
    // Buttons
    pub button_type: ButtonType,
    pub color: Option<String>,
    // Doors
    pub sprite: Option<String>,
    pub destination: Option<String>,
    pub stars_needed: i32,
    // Spawn points
    pub facing_left: bool,
    pub dx: Pixels,
    pub dy: Pixels,
    // Warp zones
    pub warp: Option<String>,
    _raw: PropertyMap,
}

impl TryFrom<PropertyMap> for MapObjectProperties {
    type Error = anyhow::Error;
    fn try_from(properties: PropertyMap) -> Result<Self> {
        Ok(MapObjectProperties {
            platform: properties.get_bool("platform")?.unwrap_or(false),
            bagel: properties.get_bool("bagel")?.unwrap_or(false),
            spring: properties.get_bool("spring")?.unwrap_or(false),
            button: properties.get_bool("button")?.unwrap_or(false),
            door: properties.get_bool("door")?.unwrap_or(false),
            star: properties.get_bool("star")?.unwrap_or(false),
            spawn: properties.get_bool("spawn")?.unwrap_or(false),
            solid: properties.get_bool("solid")?.unwrap_or(false),
            preferred_x: properties.get_int("preferred_x")?.map(Pixels::new),
            preferred_y: properties.get_int("preferred_y")?.map(Pixels::new),
            distance: properties.get_int("distance")?.unwrap_or(0),
            speed: properties.get_int("speed")?.map(Pixels::new),
            condition: properties.get_string("condition")?.map(str::to_string),
            overflow: properties
                .get_string("overflow")?
                .unwrap_or("oscillate")
                .parse()?,
            direction: properties.get_string("direction")?.unwrap_or("N").parse()?,
            convey: properties
                .get_string("convey")?
                .map(|s| s.parse())
                .transpose()?,
            button_type: properties
                .get_string("button_type")?
                .unwrap_or("toggle")
                .parse()?,
            color: properties.get_string("color")?.map(str::to_string),
            sprite: properties.get_string("sprite")?.map(str::to_string),
            destination: properties.get_string("destination")?.map(str::to_string),
            stars_needed: properties.get_int("stars_needed")?.unwrap_or(0),
            dx: Pixels::new(properties.get_int("dx")?.unwrap_or(0)),
            dy: Pixels::new(properties.get_int("dy")?.unwrap_or(0)),
            facing_left: properties.get_bool("facing_left")?.unwrap_or(false),
            warp: properties.get_string("warp")?.map(str::to_string),
            _raw: properties,
        })
    }
}

pub struct MapObject {
    pub id: i32,
    pub gid: Option<TileIndex>,
    pub position: Rect<Pixels>,
    pub properties: MapObjectProperties,
}

impl MapObject {
    fn new(xml: ObjectXml, tilesets: &TileSetList) -> Result<MapObject> {
        let id = xml.id;
        let x = xml.x;
        let mut y = xml.y;
        let width = xml.width.unwrap_or(0);
        let height = xml.height.unwrap_or(0);
        let mut properties: PropertyMap = xml
            .properties
            .map(|x| x.try_into())
            .transpose()?
            .unwrap_or_default();
        let gid = xml.gid.map(|index| (index as usize).into());

        if let Some(gid) = gid {
            let (tileset, tile_id) = tilesets.lookup(gid);
            let defaults = tileset.get_tile_properties(tile_id);
            if let Some(props) = defaults {
                properties.set_defaults(&props.raw);
            }
            // For some reason, the position is the bottom left sometimes?
            y -= height;
        }

        let x = Pixels::new(x);
        let y = Pixels::new(y);
        let w = Pixels::new(width);
        let h = Pixels::new(height);
        let position = Rect { x, y, w, h };

        let properties = properties.try_into()?;

        Ok(MapObject {
            id,
            gid,
            position,
            properties,
        })
    }
}

struct TileSetList {
    tilesets: Vec<TileSet>,
}

impl TileSetList {
    fn new() -> Self {
        Self {
            tilesets: Vec::new(),
        }
    }

    fn add(&mut self, tileset: TileSet) {
        self.tilesets.push(tileset);
        self.tilesets.sort_by_key(|tileset| tileset.gid_sort_key());
    }

    fn lookup(&self, tile_gid: TileIndex) -> (&TileSet, LocalTileIndex) {
        for tileset in self.tilesets.iter() {
            if let Some(tile_id) = tileset.get_local_tile_index(tile_gid) {
                return (tileset, tile_id);
            }
        }
        panic!("invalid tile_gid {:?}", tile_gid);
    }
}

pub struct TileMapProperties {
    pub dark: bool,
    pub gravity: Option<Subpixels>,
}

impl TryFrom<PropertyMap> for TileMapProperties {
    type Error = anyhow::Error;
    fn try_from(properties: PropertyMap) -> Result<Self> {
        Ok(TileMapProperties {
            dark: properties.get_bool("is_dark")?.unwrap_or(false),
            gravity: properties.get_int("gravity")?.map(Subpixels::new),
        })
    }
}

pub struct TileMap {
    pub width: i32,
    pub height: i32,
    pub tilewidth: Pixels,
    pub tileheight: Pixels,
    backgroundcolor: Color,
    tilesets: TileSetList,
    layers: Vec<Layer>,
    player_layer: Option<i32>, // TODO: Should just be i32.
    pub objects: Vec<MapObject>,
    pub properties: TileMapProperties,
}

impl TileMap {
    pub fn from_file(
        path: &Path,
        files: &FileManager,
        images: &mut dyn ImageLoader,
    ) -> Result<TileMap> {
        info!("loading tilemap from {:?}", path);
        let text = files
            .read_to_string(path)
            .map_err(|e| anyhow!("unable to open {:?}: {}", path, e))?;
        let xml = quick_xml::de::from_str::<TileMapXml>(&text)?;
        Self::from_xml(xml, path, files, images)
    }

    fn from_xml(
        xml: TileMapXml,
        path: &Path,
        files: &FileManager,
        images: &mut dyn ImageLoader,
    ) -> Result<TileMap> {
        let width = xml.width;
        let height = xml.height;
        let tilewidth = Pixels::new(xml.tilewidth);
        let tileheight = Pixels::new(xml.tileheight);
        let backgroundcolor = xml.backgroundcolor.parse().context(format!(
            "parsing background color {:?}",
            &xml.backgroundcolor
        ))?;

        let mut tilesets = TileSetList::new();
        for field in xml.fields.iter() {
            if let TileMapXmlField::TileSet(tileset) = field {
                let firstgid = tileset.firstgid.into();
                let tileset_path = path
                    .parent()
                    .context("cannot load root as map")?
                    .join(tileset.source.clone());
                let tileset = TileSet::from_file(&tileset_path, firstgid, files, images)?;
                tilesets.add(tileset);
            }
        }
        if tilesets.tilesets.is_empty() {
            bail!("at least one tileset must be present");
        }

        let mut player_layer: Option<i32> = None;
        let mut layers = Vec::new();
        let mut objects: Vec<MapObject> = Vec::new();
        for field in xml.fields {
            match field {
                TileMapXmlField::Layer(layer) => {
                    let layer = TileLayer::from_xml(layer)?;
                    if layer.player {
                        if player_layer.is_some() {
                            bail!("too many player layers");
                        }
                        player_layer = Some(layers.len() as i32);
                    }
                    layers.push(Layer::Tile(layer));
                }
                TileMapXmlField::ImageLayer(layer) => {
                    layers.push(Layer::Image(ImageLayer::from_xml(layer, path, images)?));
                }
                TileMapXmlField::ObjectGroup(group) => {
                    for object in group.object {
                        objects.push(MapObject::new(object, &tilesets)?);
                    }
                }
                _ => {}
            }
        }

        let properties = if let Some(props) = xml.properties {
            props.try_into()?
        } else {
            PropertyMap::new()
        };

        let properties = properties.try_into()?;

        Ok(TileMap {
            width,
            height,
            tilewidth,
            tileheight,
            backgroundcolor,
            tilesets,
            layers,
            player_layer,
            objects,
            properties,
        })
    }

    fn is_condition_met(&self, tile_gid: TileIndex, switches: &SwitchState) -> bool {
        let Some(props) = self.get_tile_properties(tile_gid) else {
            return true;
        };
        let Some(condition) = &props.condition else {
            return true;
        };
        switches.is_condition_true(condition)
    }

    fn draw_image_layer(
        &self,
        layer: &ImageLayer,
        context: &mut RenderContext,
        render_layer: RenderLayer,
        _dest: Rect<Subpixels>,
        offset: Point<Subpixels>,
    ) {
        let dest = Rect {
            x: offset.x,
            y: offset.y,
            w: layer.surface.area.w.as_subpixels(),
            h: layer.surface.area.h.as_subpixels(),
        };
        let source = Rect {
            x: Pixels::zero(),
            y: Pixels::zero(),
            w: layer.surface.area.w,
            h: layer.surface.area.h,
        };
        context.draw(layer.surface, render_layer, dest, source);
    }

    fn draw_tile_layer(
        &self,
        layer: &TileLayer,
        context: &mut RenderContext,
        render_layer: RenderLayer,
        dest: Rect<Subpixels>,
        offset: Point<Subpixels>,
        switches: &SwitchState,
    ) {
        let one_subpixel = Subpixels::new(1);

        let offset_x = offset.x;
        let offset_y = offset.y;
        let tileheight: Subpixels = self.tileheight.as_subpixels();
        let tilewidth: Subpixels = self.tilewidth.as_subpixels();

        let dest_h = (dest.h / one_subpixel) as f32;
        let dest_w = (dest.w / one_subpixel) as f32;
        let tileheight_f = (tileheight / one_subpixel) as f32;
        let tilewidth_f = (tilewidth / one_subpixel) as f32;

        let row_count = (dest_h / tileheight_f).ceil() as i32 + 1;
        let col_count = (dest_w / tilewidth_f).ceil() as i32 + 1;

        let start_row = (-(offset_y / tileheight)).max(0);
        let end_row = (start_row + row_count).min(self.height);

        let start_col = (-(offset_x / tilewidth)).max(0);
        let end_col = (start_col + col_count).min(self.width);

        for row in start_row..end_row {
            for col in start_col..end_col {
                // Compute what to draw where.
                let index = layer
                    .data
                    .get(row as usize)
                    .expect("size was checked at init")
                    .get(col as usize)
                    .expect("size was checked at init");
                let index = *index;
                if index.0 == 0 {
                    continue;
                }

                let (tileset, tile_id) = self.tilesets.lookup(index);

                let tile_id = if self.is_condition_met(index, switches) {
                    tile_id
                } else {
                    let Some(props) = self.get_tile_properties(index) else {
                        continue;
                    };
                    let Some(alt) = props.alternate else {
                        continue;
                    };
                    alt
                };

                let mut source = tileset.get_source_rect(tile_id);
                let mut pos_x = tilewidth * col + dest.x + offset_x;
                let mut pos_y = tileheight * row + dest.y + offset_y;

                // If it's off the top/left side, trim it.
                if pos_x < dest.x {
                    let extra = (dest.left() - pos_x).as_pixels();
                    source.x += extra;
                    source.w -= extra;
                    pos_x = dest.x;
                }
                if pos_y < dest.y {
                    let extra = (dest.top() - pos_y).as_pixels();
                    source.y += extra;
                    source.h -= extra;
                    pos_y = dest.y;
                }
                if source.w <= Pixels::zero() || source.h <= Pixels::zero() {
                    continue;
                }

                // If it's off the right/bottom side, trim it.
                let pos_right = pos_x + tilewidth;
                if pos_right >= dest.right() {
                    source.w -= (pos_right - dest.right()).as_pixels();
                }
                if source.w <= Pixels::zero() {
                    continue;
                }
                let pos_bottom = pos_y + tileheight;
                if pos_bottom >= dest.bottom() {
                    source.h -= (pos_bottom - dest.bottom()).as_pixels();
                }
                if source.h <= Pixels::zero() {
                    continue;
                }

                // TODO: Trim the dest separately so that we don't have subpixel rounding errors.

                // Draw the rest of the turtle.
                let destination = Rect {
                    x: pos_x,
                    y: pos_y,
                    w: source.w.as_subpixels(),
                    h: source.h.as_subpixels(),
                };
                if let Some(animation) = self.get_animation(index) {
                    animation.blit(context, render_layer, destination, false);
                } else {
                    context.draw(tileset.sprite, render_layer, destination, source);
                }
            }
        }
    }

    fn draw_layer(
        &self,
        layer: &Layer,
        context: &mut RenderContext,
        render_layer: RenderLayer,
        dest: Rect<Subpixels>,
        offset: Point<Subpixels>,
        switches: &SwitchState,
    ) {
        match layer {
            Layer::Image(layer) => {
                self.draw_image_layer(layer, context, render_layer, dest, offset)
            }
            Layer::Tile(layer) => {
                self.draw_tile_layer(layer, context, render_layer, dest, offset, switches)
            }
        }
    }

    pub fn draw_background(
        &self,
        context: &mut RenderContext,
        render_layer: RenderLayer,
        dest: Rect<Subpixels>,
        offset: Point<Subpixels>,
        switches: &SwitchState,
    ) {
        context.fill_rect(dest, render_layer, self.backgroundcolor);
        for layer in self.layers.iter() {
            self.draw_layer(layer, context, render_layer, dest, offset, switches);
            if let Layer::Tile(TileLayer { player: true, .. }) = layer {
                return;
            }
        }
    }

    pub fn draw_foreground(
        &self,
        context: &mut RenderContext,
        render_layer: RenderLayer,
        dest: Rect<Subpixels>,
        offset: Point<Subpixels>,
        switches: &SwitchState,
    ) {
        if self.player_layer.is_none() {
            return;
        }
        let mut drawing = false;
        for layer in self.layers.iter() {
            if drawing {
                self.draw_layer(layer, context, render_layer, dest, offset, switches);
            }
            if let Layer::Tile(TileLayer { player: true, .. }) = layer {
                drawing = true;
            }
        }
    }

    fn get_rect(&self, row: i32, col: i32) -> Rect<Pixels> {
        Rect {
            x: self.tilewidth * col,
            y: self.tileheight * row,
            w: self.tilewidth,
            h: self.tileheight,
        }
    }
    fn is_solid_in_direction(
        &self,
        tile_gid: TileIndex,
        direction: Direction,
        is_backwards: bool,
    ) -> bool {
        let Some(TileProperties {
            oneway: Some(oneway),
            ..
        }) = self.get_tile_properties(tile_gid)
        else {
            return true;
        };
        if is_backwards {
            return false;
        }
        match direction {
            Direction::Up => oneway == "S",
            Direction::Down => oneway == "N",
            Direction::Right => oneway == "W",
            Direction::Left => oneway == "E",
        }
    }

    // Returns the offset needed to account for the closest one.
    pub fn try_move_to(
        &self,
        player_rect: Rect<Subpixels>,
        direction: Direction,
        switches: &SwitchState,
        is_backwards: bool,
    ) -> MoveResult {
        let mut result = MoveResult::new();

        let right_edge = (self.tilewidth * self.width).as_subpixels();
        let bottom_edge = (self.tileheight * self.height).as_subpixels();

        match direction {
            Direction::Left => {
                if player_rect.x < Subpixels::zero() {
                    result.hard_offset = player_rect.x * -1;
                    result.soft_offset = result.hard_offset;
                    return result;
                }
            }
            Direction::Up => {
                if player_rect.y < Subpixels::zero() {
                    result.hard_offset = player_rect.y * -1;
                    result.soft_offset = result.hard_offset;
                    return result;
                }
            }
            Direction::Right => {
                if player_rect.right() >= right_edge {
                    result.hard_offset = (right_edge - player_rect.right()) - Subpixels::new(1);
                    result.soft_offset = result.hard_offset;
                    return result;
                }
            }
            Direction::Down => {
                if player_rect.bottom() >= bottom_edge {
                    result.hard_offset = (bottom_edge - player_rect.bottom()) - Subpixels::new(1);
                    result.soft_offset = result.hard_offset;
                    return result;
                }
            }
        }

        let row1 = player_rect.top() / self.tileheight.as_subpixels();
        let col1 = player_rect.left() / self.tilewidth.as_subpixels();
        let row2 = player_rect.bottom() / self.tileheight.as_subpixels();
        let col2 = player_rect.right() / self.tilewidth.as_subpixels();

        let row1 = row1.max(0);
        let col1 = col1.max(0);
        let row2 = row2.max(0);
        let col2 = col2.max(0);

        for row in row1..=row2 {
            for col in col1..=col2 {
                let tile_rect = self.get_rect(row, col);
                for layer in self.layers.iter() {
                    let Layer::Tile(layer) = layer else {
                        continue;
                    };
                    if !layer.player && self.player_layer.is_some() {
                        continue;
                    }
                    let mut tile_gid = layer[(row as usize, col as usize)];
                    if tile_gid.0 == 0 {
                        continue;
                    }
                    let (tileset, mut tile_id) = self.tilesets.lookup(tile_gid);
                    if !self.is_condition_met(tile_gid, switches) {
                        let Some(TileProperties {
                            alternate: Some(alt),
                            ..
                        }) = self.get_tile_properties(tile_gid)
                        else {
                            continue;
                        };
                        // Use an alt tile instead of the original.
                        tile_id = *alt;
                        tile_gid = tileset.get_global_tile_index(tile_id);
                    }
                    let solid = self
                        .get_tile_properties(tile_gid)
                        .map(|p| p.solid)
                        .unwrap_or(true);
                    if !solid {
                        continue;
                    }
                    if !self.is_solid_in_direction(tile_gid, direction, is_backwards) {
                        continue;
                    }

                    let mut tile_bounds: Rect<Subpixels> = tile_rect.into();
                    if let Some(props) = self.get_tile_properties(tile_gid) {
                        tile_bounds = Rect {
                            x: tile_bounds.x + props.hitbox_left.as_subpixels(),
                            y: tile_bounds.y + props.hitbox_top.as_subpixels(),
                            w: tile_bounds.w
                                - (props.hitbox_left + props.hitbox_right).as_subpixels(),
                            h: tile_bounds.h
                                - (props.hitbox_top + props.hitbox_bottom).as_subpixels(),
                        };
                    }
                    let soft_offset = try_move_to_bounds(player_rect, tile_bounds, direction);
                    let mut hard_offset = soft_offset;

                    if let Some(slope) = tileset.get_slope(tile_id) {
                        hard_offset = slope.try_move_to_bounds(player_rect, tile_bounds, direction);
                    };

                    result.consider_tile(tile_gid, hard_offset, soft_offset, direction);
                }
            }
        }
        result
    }

    pub fn get_gravity(&self) -> Subpixels {
        self.properties.gravity.unwrap_or(MAX_GRAVITY)
    }

    pub fn get_preferred_view(
        &self,
        player_rect: Rect<Subpixels>,
    ) -> (Option<Subpixels>, Option<Subpixels>) {
        let mut preferred_x = None;
        let mut preferred_y = None;
        for obj in self.objects.iter() {
            if obj.gid.is_some() {
                continue;
            }
            if !player_rect.intersects(obj.position.into()) {
                continue;
            }
            if let Some(p_x) = obj.properties.preferred_x {
                preferred_x = Some(p_x.as_subpixels());
            }
            if let Some(p_y) = obj.properties.preferred_y {
                preferred_y = Some(p_y.as_subpixels());
            }
        }
        (preferred_x, preferred_y)
    }

    pub fn draw_tile(
        &self,
        context: &mut RenderContext,
        tile_gid: TileIndex,
        layer: RenderLayer,
        dest: Rect<Subpixels>,
    ) {
        let (tileset, tile_id) = self.tilesets.lookup(tile_gid);
        let src = tileset.get_source_rect(tile_id);
        context.draw(tileset.sprite, layer, dest, src);
    }

    pub fn get_animation(&self, tile_gid: TileIndex) -> Option<&Animation> {
        let (tileset, tile_id) = self.tilesets.lookup(tile_gid);
        tileset.animations.get(tile_id)
    }

    pub fn get_tile_properties(&self, tile_gid: TileIndex) -> Option<&TileProperties> {
        let (tileset, tile_id) = self.tilesets.lookup(tile_gid);
        tileset.get_tile_properties(tile_id)
    }

    pub fn get_slope(&self, tile_gid: TileIndex) -> Option<&Slope> {
        let (tileset, tile_id) = self.tilesets.lookup(tile_gid);
        tileset.get_slope(tile_id)
    }
}

/*
 * We keep track of two different offsets so that you can be "on" a
 * slope even if there's a higher block next to it. That way, if you're
 * at the top of a slope, you can be down the slope a little, and not
 * wait until you're completely clear of the flat area before falling.
 */
pub struct MoveResult {
    pub hard_offset: Subpixels,
    pub soft_offset: Subpixels,
    pub tile_ids: SmallIntSet<TileIndex>,
}

impl MoveResult {
    fn new() -> MoveResult {
        MoveResult {
            // This is the offset that stops the player.
            hard_offset: Subpixels::zero(),
            // This is the offset for being on a slope.
            soft_offset: Subpixels::zero(),
            tile_ids: SmallIntSet::new(),
        }
    }

    fn consider_tile(
        &mut self,
        tile_gid: TileIndex,
        hard_offset: Subpixels,
        soft_offset: Subpixels,
        direction: Direction,
    ) {
        if matches!(
            cmp_in_direction(hard_offset, self.hard_offset, direction),
            Ordering::Less
        ) {
            self.hard_offset = hard_offset;
        }

        match cmp_in_direction(soft_offset, self.soft_offset, direction) {
            Ordering::Less => {
                let mut ids = SmallIntSet::new();
                ids.insert(tile_gid);
                self.soft_offset = soft_offset;
                self.tile_ids = ids;
            }
            Ordering::Equal => self.tile_ids.insert(tile_gid),
            Ordering::Greater => {}
        }
    }
}
