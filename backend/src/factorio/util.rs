use crate::types::{Direction, Position, Rect};
use noisy_float::types::r64;
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
        Direction::North => Position {
            x: Box::new(r64(pos.x())),
            y: Box::new(r64(pos.y() - offset)),
        },
        Direction::NorthWest => Position {
            x: Box::new(r64(pos.x() - offset)),
            y: Box::new(r64(pos.y() - offset)),
        },
        Direction::NorthEast => Position {
            x: Box::new(r64(pos.x() + offset)),
            y: Box::new(r64(pos.y() - offset)),
        },
        Direction::South => Position {
            x: Box::new(r64(pos.x())),
            y: Box::new(r64(pos.y() + offset)),
        },
        Direction::SouthWest => Position {
            x: Box::new(r64(pos.x() - offset)),
            y: Box::new(r64(pos.y() + offset)),
        },
        Direction::SouthEast => Position {
            x: Box::new(r64(pos.x() + offset)),
            y: Box::new(r64(pos.y() + offset)),
        },
        Direction::West => Position {
            x: Box::new(r64(pos.x() - offset)),
            y: Box::new(r64(pos.y())),
        },
        Direction::East => Position {
            x: Box::new(r64(pos.x() + offset)),
            y: Box::new(r64(pos.y())),
        },
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
