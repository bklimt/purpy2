use std::{fs, path::Path};

use crate::{
    image_manager::ImageManager, properties::PropertiesXml, properties::PropertyMap,
    sprite::Sprite, tileset::TileSet, utils::Rect,
};
use anyhow::{bail, Context, Result};
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
    id: i32,
    #[serde(rename = "@offsetx")]
    offsetx: String,
    #[serde(rename = "@offsety")]
    offsety: String,

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

    properties: Option<PropertiesXml>,
}

struct ImageLayer<'a> {
    surface: Sprite<'a>,
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
    id: i32,
    name: String,
    width: i32,
    height: i32,
    data: Vec<Vec<i32>>,
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
                row.push(part.parse()?);
            }
            if row.len() as i32 != width {
                bail!("row len = {}, but width = {}", row.len(), width);
            }
            data.push(row);
        }
        if data.len() as i32 != height {
            bail!("row data height = {}, but height = {}", data.len(), height);
        }

        Ok(TileLayer {
            id,
            name,
            width,
            height,
            data,
            player,
        })
    }
}

enum Layer<'a> {
    Tile(TileLayer),
    Image(ImageLayer<'a>),
}

struct MapObject {
    id: i32,
    gid: Option<i32>,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    properties: PropertyMap,
}

impl MapObject {
    fn new(xml: ObjectXml, tileset: &TileSet) -> Result<MapObject> {
        let id = xml.id;
        let x = xml.x;
        let mut y = xml.x;
        let width = xml.width;
        let height = xml.height;
        let mut properties: PropertyMap = xml
            .properties
            .map(|x| x.try_into())
            .transpose()?
            .unwrap_or_else(|| PropertyMap::new());
        let gid = xml.gid;

        if let Some(gid) = gid {
            if let Some(props) = tileset.get_tile_properties(gid - 1) {
                properties.copy_from(props);
            }
            // For some reason, the position is the bottom left sometimes?
            y -= height;
        }

        Ok(MapObject {
            id,
            gid,
            x,
            y,
            width,
            height,
            properties,
        })
    }

    fn rect(&self) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            w: self.width,
            h: self.height,
        }
    }
}

struct TileMap<'a> {
    width: i32,
    height: i32,
    tilewidth: i32,
    tileheight: i32,
    backgroundcolor: String,
    tileset: TileSet<'a>,
    layers: Vec<Layer<'a>>,
    player_layer: Option<i32>, // TODO: Should just be i32.
    objects: Vec<MapObject>,
    properties: PropertyMap,
}

impl<'a> TileMap<'a> {
    pub fn from_file<'b>(path: &Path, images: &ImageManager<'b>) -> Result<TileMap<'b>> {
        println!("loading tilemap from {:?}", path);
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
        let backgroundcolor = xml.backgroundcolor;
        let tileset = TileSet::from_file(path, images)?;

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
            properties,
        })
    }
}

/*
    @property
    def is_dark(self) -> bool:
        return assert_bool(self.properties.get('dark', False))

    def is_condition_met(self, tile: int, switches: SwitchState):
        condition = self.tileset.get_str_property(tile, 'condition')
        if condition is None:
            return True
        return switches.is_condition_true(condition)

    def draw_background(self,
                        context: RenderContext,
                        batch: SpriteBatch,
                        dest: pygame.Rect,
                        offset: tuple[int, int],
                        switches: SwitchState):
        batch.draw_rect(dest, self.backgroundcolor)
        for layer in self.layers:
            self.draw_layer(context, batch, layer, dest, offset, switches)
            if isinstance(layer, TileLayer) and layer.player:
                return

    def draw_foreground(self,
                        context: RenderContext,
                        batch: SpriteBatch,
                        dest: pygame.Rect,
                        offset: tuple[int, int],
                        switches: SwitchState):
        if self.player_layer is None:
            return
        drawing = False
        for layer in self.layers:
            if drawing:
                self.draw_layer(context, batch, layer, dest, offset, switches)
            if isinstance(layer, TileLayer) and layer.player:
                drawing = True

    def draw_layer(self,
                   context: RenderContext,
                   batch: SpriteBatch,
                   layer: TileLayer | ImageLayer,
                   dest: pygame.Rect,
                   offset: tuple[int, int],
                   switches: SwitchState):
        if isinstance(layer, ImageLayer):
            dest = pygame.Rect(
                offset[0],
                offset[1],
                layer.surface.get_width() * SUBPIXELS,
                layer.surface.get_height() * SUBPIXELS)
            batch.draw(layer.surface, dest)
            return

        offset_x = offset[0]
        offset_y = offset[1]
        tileheight = self.tileheight * SUBPIXELS
        tilewidth = self.tilewidth * SUBPIXELS
        row_count = math.ceil(dest.height / tileheight) + 1
        col_count = math.ceil(dest.width / tilewidth) + 1

        start_row = offset_y // -tileheight
        end_row = start_row + row_count
        if start_row < 0:
            start_row = 0
        if end_row > self.height:
            end_row = self.height

        start_col = offset_x // -tilewidth
        end_col = start_col + col_count
        if start_col < 0:
            start_col = 0
        if end_col > self.width:
            end_col = self.width

        for row in range(start_row, end_row):
            for col in range(start_col, end_col):
                # Compute what to draw where.
                index = layer.data[row][col]
                if index == 0:
                    continue
                index -= 1

                if not self.is_condition_met(index, switches):
                    alt = self.tileset.get_int_property(index, 'alternate')
                    if alt is None:
                        continue
                    index = alt

                source = self.tileset.get_source_rect(index)
                pos_x = col * tilewidth + dest.left + offset_x
                pos_y = row * tileheight + dest.top + offset_y

                # If it's off the top/left side, trim it.
                if pos_x < dest.left:
                    extra = (dest.left - pos_x) // SUBPIXELS
                    source.left += extra
                    source.width -= extra
                    pos_x = dest.left
                if pos_y < dest.top:
                    extra = (dest.top - pos_y) // SUBPIXELS
                    source.top += extra
                    source.height -= extra
                    pos_y = dest.top
                if source.width <= 0 or source.height <= 0:
                    continue

                # If it's off the right/bottom side, trim it.
                pos_right = pos_x + self.tilewidth
                if pos_right >= dest.right:
                    source.width = source.width - (pos_right - dest.right)
                if source.width <= 0:
                    continue
                pos_bottom = pos_y + self.tileheight
                if pos_bottom >= dest.bottom:
                    source.height = source.height - \
                        (pos_bottom - dest.bottom)
                if source.height <= 0:
                    continue

                # Draw the rest of the turtle.
                destination = pygame.Rect(
                    pos_x,
                    pos_y,
                    tilewidth,
                    tileheight)
                if index in self.tileset.animations:
                    self.tileset.animations[index].blit(
                        batch, destination, reverse=False)
                else:
                    batch.draw(self.tileset.surface, destination, source)

    def get_rect(self, row: int, col: int) -> pygame.Rect:
        return pygame.Rect(
            col * self.tilewidth,
            row * self.tileheight,
            self.tilewidth,
            self.tileheight)

    def is_solid_in_direction(self, tile_id: int, direction: Direction, is_backwards: bool) -> bool:
        oneway = self.tileset.get_str_property(tile_id, 'oneway')
        if oneway is None:
            return True
        if is_backwards:
            return False
        match direction:
            case Direction.UP:
                return oneway == 'S'
            case Direction.DOWN:
                return oneway == 'N'
            case Direction.RIGHT:
                return oneway == 'W'
            case Direction.LEFT:
                return oneway == 'E'
        raise Exception('unexpection direction')

    class MoveResult:
        # We keep track of two different offsets so that you can be "on" a
        # slope even if there's a higher block next to it. That way, if you're
        # at the top of a slope, you can be down the slope a little, and not
        # wait until you're completely clear of the flat area before falling.

        # This is the offset that stops the player.
        hard_offset: int = 0
        # This is the offset for being on a slope.
        soft_offset: int = 0
        tile_ids: set[int]

        def __init__(self):
            self.tile_ids = set()

        def consider_tile(self,
                          index: int,
                          hard_offset: int,
                          soft_offset: int,
                          direction: Direction):
            cmp = cmp_in_direction(
                hard_offset, self.hard_offset, direction)
            if cmp < 0:
                self.hard_offset = hard_offset

            cmp = cmp_in_direction(
                soft_offset, self.soft_offset, direction)
            if cmp < 0:
                self.soft_offset = soft_offset
                self.tile_ids = set([index])
            elif cmp == 0:
                self.tile_ids.add(index)

    def try_move_to(self,
                    player_rect: pygame.Rect,
                    direction: Direction,
                    switches: SwitchState,
                    is_backwards: bool) -> MoveResult:
        """ Returns the offset needed to account for the closest one. """
        result = TileMap.MoveResult()

        right_edge = self.width * self.tilewidth * SUBPIXELS
        bottom_edge = self.height * self.tileheight * SUBPIXELS

        if direction == Direction.LEFT and player_rect.x < 0:
            result.hard_offset = -player_rect.x
            result.soft_offset = result.hard_offset
            return result
        if direction == Direction.UP and player_rect.y < 0:
            result.hard_offset = -player_rect.y
            result.soft_offset = result.hard_offset
            return result
        if direction == Direction.RIGHT and player_rect.right >= right_edge:
            result.hard_offset = (right_edge - player_rect.right) - 1
            result.soft_offset = result.hard_offset
            return result
        if direction == Direction.DOWN and player_rect.bottom >= bottom_edge:
            result.hard_offset = (bottom_edge - player_rect.bottom) - 1
            result.soft_offset = result.hard_offset
            return result

        row1 = player_rect.top // (self.tileheight * SUBPIXELS)
        col1 = player_rect.left // (self.tilewidth * SUBPIXELS)
        row2 = player_rect.bottom // (self.tileheight * SUBPIXELS)
        col2 = player_rect.right // (self.tilewidth * SUBPIXELS)

        for row in range(row1, row2+1):
            for col in range(col1, col2+1):
                tile_rect = self.get_rect(row, col)
                tile_bounds = pygame.Rect(
                    tile_rect.x * SUBPIXELS,
                    tile_rect.y * SUBPIXELS,
                    tile_rect.w * SUBPIXELS,
                    tile_rect.h * SUBPIXELS)
                for layer in self.layers:
                    if not isinstance(layer, TileLayer):
                        continue
                    if layer.player or self.player_layer is None:
                        index = layer.data[row][col]
                        if index == 0:
                            continue
                        index -= 1
                        if not self.is_condition_met(index, switches):
                            alt = self.tileset.get_int_property(
                                index, 'alternate')
                            if alt is None:
                                continue
                            # Use an alt tile instead of the original.
                            index = alt
                        if not self.tileset.get_bool_property(index, 'solid', True):
                            continue
                        if not self.is_solid_in_direction(index, direction, is_backwards):
                            continue

                        soft_offset = try_move_to_bounds(
                            player_rect,
                            tile_bounds,
                            direction)
                        hard_offset = soft_offset

                        if self.tileset.is_slope(index):
                            slope = self.tileset.get_slope(index)
                            hard_offset = slope.try_move_to_bounds(
                                player_rect,
                                tile_bounds,
                                direction)

                        result.consider_tile(
                            index, hard_offset, soft_offset, direction)
        return result

    def get_preferred_view(self, player_rect: pygame.Rect) -> tuple[int | None, int | None]:
        preferred_x: int | None = None
        preferred_y: int | None = None
        for obj in self.objects:
            if obj.gid is not None:
                continue
            if not intersect(player_rect, obj.rect()):
                continue
            p_x = obj.properties.get('preferred_x', None)
            p_y = obj.properties.get('preferred_y', None)
            if isinstance(p_x, int):
                preferred_x = p_x
            if isinstance(p_y, int):
                preferred_y = p_y
        return (preferred_x, preferred_y)

    def update_animations(self):
        self.tileset.update_animations()


def load_map(path: str, images: ImageManager):
    print('loading map from ' + path)
    root = xml.etree.ElementTree.parse(path).getroot()
    if not isinstance(root, xml.etree.ElementTree.Element):
        raise Exception('root was not an element')
    return TileMap(root, path, images)

*/
