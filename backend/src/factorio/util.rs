use crate::types::{Direction, FactorioEntityPrototype, Position, Rect};
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
