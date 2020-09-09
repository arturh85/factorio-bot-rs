#[cfg(test)]
use crate::types::{FactorioEntity, FactorioEntityPrototype};
use evmap::{ReadHandle, WriteHandle};
use factorio_blueprint::{BlueprintCodec, Container};
use std::collections::HashMap;

pub fn blueprint_entities(
    str: &str,
    prototypes: &ReadHandle<String, FactorioEntityPrototype>,
) -> anyhow::Result<Vec<FactorioEntity>> {
    let decoded = BlueprintCodec::decode_string(str).expect("failed to parse blueprint");
    let mut entities: Vec<FactorioEntity> = vec![];
    match decoded {
        Container::Blueprint(blueprint) => {
            for ent in blueprint.entities {
                entities.push(FactorioEntity::from_blueprint_entity(ent, prototypes)?);
            }
        }
        _ => panic!("blueprint books not supported"),
    }
    Ok(entities)
}

pub fn fixture_entity_prototypes() -> (
    ReadHandle<String, FactorioEntityPrototype>,
    WriteHandle<String, FactorioEntityPrototype>,
) {
    let prototypes: HashMap<String, FactorioEntityPrototype> =
        serde_json::from_str(include_str!("../data/entity-prototype-fixtures.json"))
            .expect("failed to parse fixture");
    let (reader, mut writer) = evmap::new::<String, FactorioEntityPrototype>();
    for (name, p) in prototypes {
        writer.insert(name, p);
    }
    writer.refresh();
    (reader, writer)
}
