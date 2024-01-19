use std::ops::{Index, IndexMut};
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::{fs, path::Path};

use crate::constants::SUBPIXELS;
use crate::imagemanager::ImageManager;
use crate::properties::{PropertiesXml, PropertyMap};
use crate::rendercontext::{RenderContext, RenderLayer};
use crate::smallintset::SmallIntSet;
use crate::sprite::Sprite;
use crate::switchstate::SwitchState;
use crate::tileset::{TileIndex, TileProperties, TileSet};
use crate::utils::{
    cmp_in_direction, intersect, try_move_to_bounds, Color, Direction, Point, Rect,
};

use anyhow::{anyhow, bail, Context, Result};
use log::info;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TileSetSourceXml {
    #[serde(rename = "@source")]
    source: String,
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
    width: i32,
    #[serde(rename = "@height")]
    height: i32,
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

struct ImageLayer<'a> {
    surface: Rc<Sprite<'a>>,
}

impl<'a> ImageLayer<'a> {
    fn from_xml<'b>(
        xml: ImageLayerXml,
        path: &Path,
        images: &ImageManager<'b>,
    ) -> Result<ImageLayer<'b>> {
        let path = path
            .parent()
            .context("xml file is root")?
            .join(&xml.image.source);
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
        let props = props.unwrap_or_else(|| PropertyMap::new());
        let player = props.get_bool("player")?.unwrap_or(false);

        let mut data = Vec::new();
        for line in xml.data.data.lines() {
            let line = line.trim();
            if line.len() == 0 {
                continue;
            }
            let mut row = Vec::new();
            for part in line.split(",") {
                if part.len() == 0 {
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
        self.get(index.0, index.1).expect("indices must be valid")
    }
}

impl IndexMut<(usize, usize)> for TileLayer {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        self.get_mut(index.0, index.1)
            .expect("indices must be valid")
    }
}

enum Layer<'a> {
    Tile(TileLayer),
    Image(ImageLayer<'a>),
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
    // Tiles
    pub solid: bool,
    // Map Areas
    pub preferred_x: Option<i32>,
    pub preferred_y: Option<i32>,
    // Platforms
    pub distance: i32,
    pub speed: Option<i32>,
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
            solid: properties.get_bool("solid")?.unwrap_or(false),
            preferred_x: properties.get_int("preferred_x")?,
            preferred_y: properties.get_int("preferred_y")?,
            distance: properties.get_int("distance")?.unwrap_or(0),
            speed: properties.get_int("speed")?,
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
            _raw: properties,
        })
    }
}

pub struct MapObject {
    pub id: i32,
    pub gid: Option<u32>,
    pub position: Rect,
    pub properties: MapObjectProperties,
}

impl MapObject {
    fn new(xml: ObjectXml, tileset: &TileSet) -> Result<MapObject> {
        let id = xml.id;
        let x = xml.x;
        let mut y = xml.y;
        let width = xml.width;
        let height = xml.height;
        let mut properties: PropertyMap = xml
            .properties
            .map(|x| x.try_into())
            .transpose()?
            .unwrap_or_else(|| PropertyMap::new());
        let gid = xml.gid;

        if let Some(gid) = gid {
            // TODO: Figure this part out.
            if let Some(props) = tileset.get_tile_properties(gid as TileIndex - 1) {
                properties.set_defaults(&props.raw);
            }
            // For some reason, the position is the bottom left sometimes?
            y -= height;
        }

        let position = Rect {
            x,
            y,
            w: width,
            h: height,
        };

        let properties = properties.try_into()?;

        Ok(MapObject {
            id,
            gid,
            position,
            properties,
        })
    }
}

pub struct TileMap<'a> {
    pub width: i32,
    pub height: i32,
    pub tilewidth: i32,
    pub tileheight: i32,
    backgroundcolor: Color,
    pub tileset: Rc<TileSet<'a>>,
    layers: Vec<Layer<'a>>,
    player_layer: Option<i32>, // TODO: Should just be i32.
    pub objects: Vec<MapObject>,
    is_dark: bool,
}

impl<'a> TileMap<'a> {
    pub fn from_file<'b, 'c>(path: &Path, images: &'c ImageManager<'b>) -> Result<TileMap<'b>>
    where
        'b: 'c,
    {
        info!("loading tilemap from {:?}", path);
        let text = fs::read_to_string(path)?;
        let xml = quick_xml::de::from_str::<TileMapXml>(&text)?;
        Self::from_xml(xml, path, images)
    }

    fn from_xml<'b>(
        xml: TileMapXml,
        path: &Path,
        images: &ImageManager<'b>,
    ) -> Result<TileMap<'b>> {
        let width = xml.width;
        let height = xml.height;
        let tilewidth: i32 = xml.tilewidth;
        let tileheight: i32 = xml.tileheight;
        let backgroundcolor = xml.backgroundcolor.parse().context(format!(
            "parsing background color {:?}",
            &xml.backgroundcolor
        ))?;

        let mut tileset_path: Option<PathBuf> = None;
        for field in xml.fields.iter() {
            if let TileMapXmlField::TileSet(tileset) = field {
                tileset_path = Some(
                    path.parent()
                        .context("cannot load root as map")?
                        .join(tileset.source.clone()),
                );
            }
        }
        let tileset_path = tileset_path.context("at least one tileset must be present")?;
        let tileset = TileSet::from_file(&tileset_path, images)?;

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
                        objects.push(MapObject::new(object, &tileset)?);
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

        let is_dark = properties.get_bool("dark")?.unwrap_or(false);

        let tileset = Rc::new(tileset);

        Ok(TileMap {
            width,
            height,
            tilewidth,
            tileheight,
            backgroundcolor,
            tileset,
            layers,
            player_layer,
            objects,
            is_dark,
        })
    }

    fn is_dark(&self) -> bool {
        self.is_dark
    }

    fn is_condition_met(&self, tile: TileIndex, switches: &SwitchState) -> bool {
        let Some(props) = self.tileset.get_tile_properties(tile) else {
            return true;
        };
        let Some(condition) = &props.condition else {
            return true;
        };
        switches.is_condition_true(condition)
    }

    fn draw_image_layer<'b>(
        &self,
        layer: &ImageLayer<'a>,
        context: &'b mut RenderContext<'a>,
        render_layer: RenderLayer,
        _dest: Rect,
        offset: Point,
    ) where
        'a: 'b,
    {
        let dest = Rect {
            x: offset.x(),
            y: offset.y(),
            w: layer.surface.width() as i32 * SUBPIXELS,
            h: layer.surface.height() as i32 * SUBPIXELS,
        };
        let source = Rect {
            x: 0,
            y: 0,
            w: dest.w,
            h: dest.h,
        };
        context.draw(&layer.surface, render_layer, dest, source);
    }

    fn draw_tile_layer<'b>(
        &self,
        layer: &TileLayer,
        context: &'b mut RenderContext<'a>,
        render_layer: RenderLayer,
        dest: Rect,
        offset: Point,
        switches: &SwitchState,
    ) where
        'a: 'b,
    {
        let offset_x = offset.x();
        let offset_y = offset.y();
        let tileheight = self.tileheight * SUBPIXELS;
        let tilewidth = self.tilewidth * SUBPIXELS;
        let row_count = (dest.h as f32 / tileheight as f32).ceil() as i32 + 1;
        let col_count = (dest.w as f32 / tilewidth as f32).ceil() as i32 + 1;

        let start_row = (offset_y / -tileheight).max(0);
        let end_row = (start_row + row_count).min(self.height);

        let start_col = (offset_x / -tilewidth).max(0);
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
                if index == 0 {
                    continue;
                }
                let index = index - 1;

                let index = if self.is_condition_met(index, switches) {
                    index
                } else {
                    let Some(props) = self.tileset.get_tile_properties(index) else {
                        continue;
                    };
                    let Some(alt) = props.alternate else {
                        continue;
                    };
                    alt
                };

                let mut source = self.tileset.get_source_rect(index);
                let mut pos_x = col * tilewidth + dest.x + offset_x;
                let mut pos_y = row * tileheight + dest.y + offset_y;

                // If it's off the top/left side, trim it.
                if pos_x < dest.x {
                    let extra = (dest.left() - pos_x) / SUBPIXELS;
                    source.x += extra;
                    source.w -= extra;
                    pos_x = dest.x;
                }
                if pos_y < dest.y {
                    let extra = (dest.top() - pos_y) / SUBPIXELS;
                    source.y += extra;
                    source.h -= extra;
                    pos_y = dest.y;
                }
                if source.w <= 0 || source.h <= 0 {
                    continue;
                }

                // If it's off the right/bottom side, trim it.
                let pos_right = pos_x + self.tilewidth;
                if pos_right >= dest.right() {
                    source.w -= pos_right - dest.right();
                }
                if source.w <= 0 {
                    continue;
                }
                let pos_bottom = pos_y + self.tileheight;
                if pos_bottom >= dest.bottom() {
                    source.h -= pos_bottom - dest.bottom();
                }
                if source.h <= 0 {
                    continue;
                }

                // TODO: Trim the dest separately so that we don't have subpixel rounding errors.

                // Draw the rest of the turtle.
                let destination = Rect {
                    x: pos_x,
                    y: pos_y,
                    w: source.w * SUBPIXELS,
                    h: source.h * SUBPIXELS,
                };
                if let Some(animation) = self.tileset.animations.get(index) {
                    animation.blit(context, render_layer, destination, false);
                } else {
                    context.draw(&self.tileset.sprite, render_layer, destination, source);
                }
            }
        }
    }

    fn draw_layer<'b>(
        &self,
        layer: &Layer<'a>,
        context: &'b mut RenderContext<'a>,
        render_layer: RenderLayer,
        dest: Rect,
        offset: Point,
        switches: &SwitchState,
    ) where
        'a: 'b,
    {
        match layer {
            Layer::Image(layer) => {
                self.draw_image_layer(layer, context, render_layer, dest, offset)
            }
            Layer::Tile(layer) => {
                self.draw_tile_layer(layer, context, render_layer, dest, offset, switches)
            }
        }
    }

    pub fn draw_background<'b>(
        &self,
        context: &'b mut RenderContext<'a>,
        render_layer: RenderLayer,
        dest: Rect,
        offset: Point,
        switches: &SwitchState,
    ) where
        'a: 'b,
    {
        context.fill_rect(dest.clone(), render_layer, self.backgroundcolor);
        for layer in self.layers.iter() {
            self.draw_layer(layer, context, render_layer, dest, offset, switches);
            if let Layer::Tile(TileLayer { player: true, .. }) = layer {
                return;
            }
        }
    }

    pub fn draw_foreground<'b>(
        &self,
        context: &'b mut RenderContext<'a>,
        render_layer: RenderLayer,
        dest: Rect,
        offset: Point,
        switches: &SwitchState,
    ) where
        'a: 'b,
    {
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

    fn get_rect(&self, row: i32, col: i32) -> Rect {
        Rect {
            x: col * self.tilewidth,
            y: row * self.tileheight,
            w: self.tilewidth,
            h: self.tileheight,
        }
    }
    fn is_solid_in_direction(
        &self,
        tile_id: TileIndex,
        direction: Direction,
        is_backwards: bool,
    ) -> bool {
        let Some(TileProperties {
            oneway: Some(oneway),
            ..
        }) = self.tileset.get_tile_properties(tile_id)
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
            _ => panic!("unexpected direction"),
        }
    }

    // Returns the offset needed to account for the closest one.
    pub fn try_move_to(
        &self,
        player_rect: Rect,
        direction: Direction,
        switches: &SwitchState,
        is_backwards: bool,
    ) -> MoveResult {
        let mut result = MoveResult::new();

        let right_edge = self.width * self.tilewidth * SUBPIXELS;
        let bottom_edge = self.height * self.tileheight * SUBPIXELS;

        match direction {
            Direction::Left => {
                if player_rect.x < 0 {
                    result.hard_offset = -player_rect.x;
                    result.soft_offset = result.hard_offset;
                    return result;
                }
            }
            Direction::Up => {
                if player_rect.y < 0 {
                    result.hard_offset = -player_rect.y;
                    result.soft_offset = result.hard_offset;
                    return result;
                }
            }
            Direction::Right => {
                if player_rect.right() >= right_edge {
                    result.hard_offset = (right_edge - player_rect.right()) - 1;
                    result.soft_offset = result.hard_offset;
                    return result;
                }
            }
            Direction::Down => {
                if player_rect.bottom() >= bottom_edge {
                    result.hard_offset = (bottom_edge - player_rect.bottom()) - 1;
                    result.soft_offset = result.hard_offset;
                    return result;
                }
            }
            Direction::None => panic!("unexpected direction"),
        }

        let row1 = player_rect.top() / (self.tileheight * SUBPIXELS);
        let col1 = player_rect.left() / (self.tilewidth * SUBPIXELS);
        let row2 = player_rect.bottom() / (self.tileheight * SUBPIXELS);
        let col2 = player_rect.right() / (self.tilewidth * SUBPIXELS);

        for row in row1..=row2 {
            for col in col1..=col2 {
                let tile_rect = self.get_rect(row, col);
                let tile_bounds = Rect {
                    x: tile_rect.x * SUBPIXELS,
                    y: tile_rect.y * SUBPIXELS,
                    w: tile_rect.w * SUBPIXELS,
                    h: tile_rect.h * SUBPIXELS,
                };
                for layer in self.layers.iter() {
                    let Layer::Tile(layer) = layer else {
                        continue;
                    };
                    if !layer.player && self.player_layer.is_some() {
                        continue;
                    }
                    let mut index = layer[(row as usize, col as usize)];
                    if index == 0 {
                        continue;
                    }
                    // TODO: This should use the start_gid and tileset.
                    index -= 1;
                    if !self.is_condition_met(index, switches) {
                        let Some(TileProperties {
                            alternate: Some(alt),
                            ..
                        }) = self.tileset.get_tile_properties(index)
                        else {
                            continue;
                        };
                        // Use an alt tile instead of the original.
                        index = *alt;
                    }
                    let solid = self
                        .tileset
                        .get_tile_properties(index)
                        .map(|p| p.solid)
                        .unwrap_or(true);
                    if !solid {
                        continue;
                    }
                    if !self.is_solid_in_direction(index, direction, is_backwards) {
                        continue;
                    }

                    let soft_offset = try_move_to_bounds(player_rect, tile_bounds, direction);
                    let mut hard_offset = soft_offset;

                    if let Some(slope) = self.tileset.get_slope(index) {
                        hard_offset = slope.try_move_to_bounds(player_rect, tile_bounds, direction);
                    };

                    result.consider_tile(index, hard_offset, soft_offset, direction);
                }
            }
        }
        result
    }

    pub fn get_preferred_view(&self, player_rect: Rect) -> (Option<i32>, Option<i32>) {
        let mut preferred_x = None;
        let mut preferred_y = None;
        for obj in self.objects.iter() {
            if obj.gid.is_some() {
                continue;
            }
            if !intersect(player_rect, obj.position.clone()) {
                continue;
            }
            if let Some(p_x) = obj.properties.preferred_x {
                preferred_x = Some(p_x);
            }
            if let Some(p_y) = obj.properties.preferred_y {
                preferred_y = Some(p_y);
            }
        }
        (preferred_x, preferred_y)
    }
}

/*
 * We keep track of two different offsets so that you can be "on" a
 * slope even if there's a higher block next to it. That way, if you're
 * at the top of a slope, you can be down the slope a little, and not
 * wait until you're completely clear of the flat area before falling.
 */
pub struct MoveResult {
    pub hard_offset: i32,
    pub soft_offset: i32,
    pub tile_ids: SmallIntSet<TileIndex>,
}

impl MoveResult {
    fn new() -> MoveResult {
        MoveResult {
            // This is the offset that stops the player.
            hard_offset: 0,
            // This is the offset for being on a slope.
            soft_offset: 0,
            tile_ids: SmallIntSet::new(),
        }
    }

    fn consider_tile(
        &mut self,
        index: TileIndex,
        hard_offset: i32,
        soft_offset: i32,
        direction: Direction,
    ) {
        let cmp = cmp_in_direction(hard_offset, self.hard_offset, direction);
        if cmp < 0 {
            self.hard_offset = hard_offset;
        }

        let cmp = cmp_in_direction(soft_offset, self.soft_offset, direction);
        if cmp < 0 {
            let mut ids = SmallIntSet::new();
            ids.insert(index);
            self.soft_offset = soft_offset;
            self.tile_ids = ids;
        } else if cmp == 0 {
            self.tile_ids.insert(index);
        }
    }
}
