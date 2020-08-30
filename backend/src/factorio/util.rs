use crate::types::{
    Direction, FactorioEntity, FactorioEntityPrototype, FactorioTile, Position, Rect,
};
use evmap::ReadHandle;
use factorio_blueprint::BlueprintCodec;
use factorio_blueprint::Container::{Blueprint, BlueprintBook};
use num_traits::ToPrimitive;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub fn hashmap_to_lua(map: HashMap<String, String>) -> String {
    let mut parts: Vec<String> = Vec::new();
    for (k, v) in map {
        parts.push(String::from(&format!("{}={}", k, v)));
    }
    format!("{{{}}}", parts.join(","))
}

pub fn value_to_lua(value: &Value) -> String {
    match value {
        Value::Null => "nil".into(),
        Value::Bool(bool) => bool.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => str_to_lua(&string),
        Value::Array(vec) => format!(
            "{{ {} }}",
            vec.iter()
                .map(|value| value_to_lua(&value))
                .collect::<Vec<String>>()
                .join(", ")
        ),
        Value::Object(map) => {
            let mut parts: Vec<String> = Vec::new();
            for (k, v) in map {
                parts.push(String::from(&format!("{}={}", k, value_to_lua(v))));
            }
            format!("{{{}}}", parts.join(","))
        }
    }
}

pub fn position_to_lua(position: &Position) -> String {
    format!("{{{},{}}}", position.x, position.y)
}

pub fn rect_to_lua(rect: &Rect) -> String {
    format!(
        "{{{},{}}}",
        position_to_lua(&rect.left_top),
        position_to_lua(&rect.right_bottom)
    )
}

pub fn vec_to_lua(vec: Vec<String>) -> String {
    format!("{{ {} }}", vec.join(", "))
}

pub fn str_to_lua(str: &str) -> String {
    format!("'{}'", str)
}

pub fn calculate_distance(pos1: &Position, pos2: &Position) -> f64 {
    let x = pos1.x() - pos2.x();
    let y = pos1.y() - pos2.y();
    (x * x + y * y).sqrt()
}

pub fn move_position(pos: &Position, direction: Direction, offset: f64) -> Position {
    match direction {
        Direction::North => Position::new(pos.x(), pos.y() - offset),
        Direction::NorthWest => Position::new(pos.x() - offset, pos.y() - offset),
        Direction::NorthEast => Position::new(pos.x() + offset, pos.y() - offset),
        Direction::South => Position::new(pos.x(), pos.y() + offset),
        Direction::SouthWest => Position::new(pos.x() - offset, pos.y() + offset),
        Direction::SouthEast => Position::new(pos.x() + offset, pos.y() + offset),
        Direction::West => Position::new(pos.x() - offset, pos.y()),
        Direction::East => Position::new(pos.x() + offset, pos.y()),
    }
}

pub fn read_to_value(path: &PathBuf) -> anyhow::Result<Value> {
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(content.as_str())?)
}

pub fn write_value_to(value: &Value, path: &PathBuf) -> anyhow::Result<()> {
    let mut outfile = fs::File::create(&path)?;
    let bytes = serde_json::to_string(value).unwrap();
    outfile.write_all(bytes.as_ref())?;
    Ok(())
}

pub fn pad_rect(
    rect: &Rect,
    margin_left: f64,
    margin_top: f64,
    margin_right: f64,
    margin_bottom: f64,
) -> Rect {
    Rect {
        left_top: Position::new(
            rect.left_top.x() - margin_left,
            rect.left_top.y() - margin_top,
        ),
        right_bottom: Position::new(
            rect.right_bottom.x() + margin_right,
            rect.right_bottom.y() + margin_bottom,
        ),
    }
}

pub fn expand_rect_floor_ceil(rect: &Rect) -> Rect {
    Rect {
        left_top: Position::new(
            (rect.left_top.x() * 2.0).floor() / 2.0,
            (rect.left_top.y() * 2.0).floor() / 2.0,
        ),
        right_bottom: Position::new(
            (rect.right_bottom.x() * 2.0).ceil() / 2.0,
            (rect.right_bottom.y() * 2.0).ceil() / 2.0,
        ),
    }
}

pub fn add_to_rect(rect: &Rect, position: &Position) -> Rect {
    Rect {
        left_top: Position::new(
            position.x() + rect.left_top.x(),
            position.y() + rect.left_top.y(),
        ),
        right_bottom: Position::new(
            position.x() + rect.right_bottom.x(),
            position.y() + rect.right_bottom.y(),
        ),
    }
}

pub fn expand_rect(total_rect: &mut Rect, rect: &Rect) {
    if rect.left_top.x() < total_rect.left_top.x() {
        total_rect.left_top = Position::new(rect.left_top.x(), total_rect.left_top.y());
    }
    if rect.left_top.y() < total_rect.left_top.y() {
        total_rect.left_top = Position::new(total_rect.left_top.x(), rect.left_top.y());
    }
    if rect.right_bottom.x() > total_rect.right_bottom.x() {
        total_rect.right_bottom = Position::new(rect.right_bottom.x(), total_rect.right_bottom.y());
    }
    if rect.right_bottom.y() > total_rect.right_bottom.y() {
        total_rect.right_bottom = Position::new(total_rect.right_bottom.x(), rect.right_bottom.y());
    }
}

pub fn position_in_rect(rect: &Rect, position: &Position) -> bool {
    position.x() > rect.left_top.x()
        && position.x() < rect.right_bottom.x()
        && position.y() > rect.left_top.y()
        && position.y() < rect.right_bottom.y()
}

pub fn blueprint_build_area(
    entity_prototypes: &ReadHandle<String, FactorioEntityPrototype>,
    blueprint: &str,
) -> Rect {
    let decoded = BlueprintCodec::decode_string(&blueprint).expect("failed to decode blueprint");
    let mut build_area = Rect::new(&Position::new(999.0, 999.0), &Position::new(-999.0, -999.0));
    match decoded {
        BlueprintBook(_blueprint_book) => {
            panic!("blueprint books are not supported!");
        }
        Blueprint(blueprint) => {
            for entity in blueprint.entities {
                let prototype = entity_prototypes.get_one(&entity.name);
                if let Some(prototype) = prototype {
                    let entity_position = Position::new(
                        entity.position.x.to_f64().unwrap(),
                        entity.position.y.to_f64().unwrap(),
                    );
                    let collision_box = expand_rect_floor_ceil(&prototype.collision_box);
                    let collision_rect = add_to_rect(&collision_box, &entity_position);
                    expand_rect(&mut build_area, &collision_rect)
                }
            }
        }
    };
    build_area
}

pub fn vector_length(vector: &Position) -> f64 {
    (vector.x() * vector.x() + vector.y() * vector.y()).sqrt()
}

pub fn vector_normalize(vector: &Position) -> Position {
    let len = vector_length(vector);
    Position::new(vector.x() / len, vector.y() / len)
}

pub fn vector_substract(a: &Position, b: &Position) -> Position {
    Position::new(a.x() - b.x(), a.y() - b.y())
}

pub fn vector_add(a: &Position, b: &Position) -> Position {
    Position::new(a.x() + b.x(), a.y() + b.y())
}

pub fn vector_multiply(a: &Position, len: f64) -> Position {
    Position::new(a.x() * len, a.y() * len)
}

/*
https://limnu.com/sketch-easy-90-degree-rotate-vectors/#:~:text=Normally%20rotating%20vectors%20involves%20matrix,swap%20X%20and%20Y%20values.
Normally rotating vectors involves matrix math, but there’s a really simple trick for rotating a 2D vector by 90° clockwise:
just multiply the X part of the vector by -1, and then swap X and Y values.
 */
pub fn vector_rotate_clockwise(vector: &Position) -> Position {
    Position::new(vector.y(), vector.x() * -1.0)
}

pub fn span_rect(a: &Position, b: &Position, margin: f64) -> Rect {
    Rect::new(
        &Position::new(
            if a.x() < b.x() { a.x() } else { b.x() } - margin,
            if a.y() < b.y() { a.y() } else { b.y() } - margin,
        ),
        &Position::new(
            if a.x() > b.x() { a.x() } else { b.x() } + margin,
            if a.y() > b.y() { a.y() } else { b.y() } + margin,
        ),
    )
}

use pathfinding::prelude::{absdiff, astar};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Pos(i32, i32);
impl Pos {
    fn distance(&self, other: &Pos) -> u32 {
        (absdiff(self.0, other.0) + absdiff(self.1, other.1)) as u32
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_entity_path(
    entity_prototypes: &ReadHandle<String, FactorioEntityPrototype>,
    entity_name: &str,
    entity_type: &str,
    from_position: &Position,
    to_position: &Position,
    to_direction: Direction,
    block_entities: Vec<FactorioEntity>,
    block_tiles: Vec<FactorioTile>,
) -> anyhow::Result<Vec<FactorioEntity>> {
    let from_position = Pos(
        from_position.x().floor() as i32,
        from_position.y().floor() as i32,
    );
    let to_position = Pos(
        to_position.x().floor() as i32,
        to_position.y().floor() as i32,
    );

    let path = astar(
        &from_position,
        |p| {
            let mut options: Vec<(Pos, i32)> = vec![];
            for direction in Direction::orthogonal() {
                let target = move_position(&Position::new(p.0 as f64, p.1 as f64), direction, 1.0);
                let entity: Option<&FactorioEntity> = block_entities.iter().find(|entity| {
                    match entity_prototypes.get_one(&entity.name) {
                        Some(entity_prototype) => position_in_rect(
                            &pad_rect(
                                &add_to_rect(
                                    &expand_rect_floor_ceil(&entity_prototype.collision_box),
                                    &entity.position,
                                ),
                                1.,
                                1.,
                                0.,
                                0.,
                            ),
                            &target,
                        ),
                        None => position_equal(&entity.position, &target),
                    }
                });
                let tile: Option<&FactorioTile> = block_tiles
                    .iter()
                    .find(|tile| position_equal(&tile.position, &target));
                if tile.is_none() && entity.is_none() {
                    options.push((Pos(target.x().floor() as i32, target.y().floor() as i32), 1));
                }
                // TODO: add options for underground belts / pipes
            }
            options
        },
        |p| (p.distance(&to_position) / 3) as i32,
        |p| *p == to_position,
    );
    match path {
        Some((path, _cost)) => {
            let mut result: Vec<FactorioEntity> = vec![];

            for i in 0..path.len() {
                let pos = &path[i];
                let next: Option<&Pos> = if i < path.len() - 1 {
                    Some(&path[i + 1])
                } else {
                    None
                };
                let direction = if let Some(next) = next {
                    if next.0 < pos.0 {
                        Direction::West
                    } else if next.0 > pos.0 {
                        Direction::East
                    } else if next.1 < pos.1 {
                        Direction::North
                    } else {
                        Direction::South
                    }
                } else {
                    to_direction
                };
                result.push(FactorioEntity {
                    name: entity_name.into(),
                    entity_type: entity_type.into(),
                    position: Position::new(pos.0 as f64, pos.1 as f64),
                    direction: direction.to_u8().unwrap(),
                    recipe: None,
                    ghost_name: None,
                    ghost_type: None,
                    amount: None,
                });
            }
            Ok(result)
        }
        None => Err(anyhow!("no path found")),
    }
}

pub fn floor_position(position: &Position) -> Position {
    Position::new(position.x().floor(), position.y().floor())
}

pub fn position_equal(a: &Position, b: &Position) -> bool {
    (a.x().floor() - b.x().floor()).abs() < f64::EPSILON
        && (a.y().floor() - b.y().floor()).abs() < f64::EPSILON
}
