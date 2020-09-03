use crate::factorio::instance_setup::setup_factorio_instance;
use crate::factorio::output_parser::FactorioWorld;
use crate::factorio::process_control::start_factorio_server;
use crate::factorio::rcon::{FactorioRcon, RconSettings};
use crate::factorio::util::calculate_distance;
use crate::types::{AreaFilter, Position};
use async_std::sync::{Arc, Mutex};
use config::Config;
use std::cmp::Ordering;
use std::thread::JoinHandle;
use std::time::Instant;

#[derive(Debug, Copy, Clone)]
pub enum RollSeedLimit {
    Rolls(u64),
    Seconds(u64),
}

pub async fn roll_seed(
    settings: Config,
    map_exchange_string: String,
    limit: RollSeedLimit,
    parallel: u8,
) -> anyhow::Result<Option<(u32, f64)>> {
    let roll: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
    let best_seed_with_score: Arc<Mutex<Option<(u32, f64)>>> = Arc::new(Mutex::new(None));
    let workspace_path: Arc<String> = Arc::new(settings.get("workspace_path")?);
    let map_exchange_string = Arc::new(map_exchange_string);

    let mut join_handles: Vec<JoinHandle<()>> = vec![];
    info!("preparing instances ...");
    for p in 0..parallel {
        let instance_name = format!("roll{}", p + 1);
        let rcon_settings = RconSettings {
            host: None,
            pass: "roll".into(),
            port: 1234 + p as u16,
        };
        let factorio_port: u16 = 2345 + p as u16;
        setup_factorio_instance(
            &workspace_path,
            &rcon_settings,
            Some(factorio_port),
            &instance_name,
            true,
            true,
            true,
            Some(&map_exchange_string),
            None,
            true,
        )
        .await
        .expect("failed to initially setup instance");
    }
    info!("finished preparing. spawning {} instances", parallel);
    let started = Instant::now();
    for p in 0..parallel {
        let instance_name = format!("roll{}", p + 1);
        let rcon_settings = RconSettings {
            host: None,
            pass: "roll".into(),
            port: 1234 + p as u16,
        };
        let factorio_port: u16 = 2345 + p as u16;
        let best_seed_with_score = best_seed_with_score.clone();
        let workspace_path = workspace_path.clone();
        let map_exchange_string = map_exchange_string.clone();
        let roll = roll.clone();

        join_handles.push(std::thread::spawn(move || {
            actix::run(async move {
                while match limit {
                    RollSeedLimit::Rolls(max_rolls) => *roll.lock().await < max_rolls,
                    RollSeedLimit::Seconds(max_seconds) => {
                        started.elapsed() < std::time::Duration::from_secs(max_seconds)
                    }
                } {
                    let roll_started = Instant::now();
                    let mut roll_mutex = roll.lock().await;
                    *roll_mutex += 1;
                    let roll: u64 = *roll_mutex;
                    drop(roll_mutex);

                    let seed: u32 = rand::random();
                    setup_factorio_instance(
                        &workspace_path,
                        &rcon_settings,
                        Some(factorio_port),
                        &instance_name,
                        true,
                        true,
                        false,
                        Some(&map_exchange_string),
                        Some(&seed.to_string()),
                        true,
                    )
                    .await
                    .expect("failed to setup instance");
                    let (world, mut child) = start_factorio_server(
                        &workspace_path,
                        &rcon_settings,
                        Some(factorio_port),
                        &instance_name,
                        None,
                        false,
                        true,
                    )
                    .await
                    .expect("failed to start");
                    let rcon = FactorioRcon::new(&rcon_settings, true)
                        .await
                        .expect("failed to rcon");
                    rcon.silent_print("").await.expect("failed to silent print");
                    // info!(
                    //     "generated {} in <yellow>{:?}</>",
                    //     seed,
                    //     roll_started.elapsed()
                    // );
                    let score = score_seed(&rcon, &world, seed)
                        .await
                        .expect("failed to score seed");
                    child.kill().expect("failed to kill child");

                    let mut best_seed_with_score = best_seed_with_score.lock().await;
                    if let Some((_, previous_score)) = *best_seed_with_score {
                        if score > previous_score {
                            (*best_seed_with_score) = Some((seed, score));
                        }
                    } else {
                        (*best_seed_with_score) = Some((seed, score));
                    }
                    info!(
                        "instance #{} rolled #{}: seed {}{}</> scored {}{}</> in <yellow>{:?}</>",
                        p + 1,
                        roll,
                        if score > -10000. { "<bold><blue>" } else { "" },
                        seed,
                        if score > -10000. { "<bold><green>" } else { "" },
                        score,
                        roll_started.elapsed()
                    );
                }
            })
            .unwrap();
        }));
    }
    for join_handle in join_handles {
        join_handle.join().unwrap();
    }

    info!(
        "scored <green>{}</> seeds in <yellow>{:?}</>",
        *roll.lock().await,
        started.elapsed()
    );
    let best_seed_with_score = best_seed_with_score.lock().await;
    match *best_seed_with_score {
        Some((best_seed, best_score)) => {
            info!("best <blue>{}</> with score {}", best_seed, best_score)
        }
        None => error!("no best? {:?}", limit),
    }
    Ok(*best_seed_with_score)
}

pub async fn score_seed(
    rcon: &FactorioRcon,
    _world: &FactorioWorld,
    _seed: u32,
) -> anyhow::Result<f64> {
    let _started = Instant::now();
    let center = Position::new(0., 0.);
    let resources = vec![
        "rock-huge",
        "iron-ore",
        "coal",
        "copper-ore",
        "stone",
        "crude-oil",
    ];
    let mut score = 0.;
    for resource in resources {
        let nearest = find_nearest_resource(rcon, &center, resource).await?;
        match nearest {
            Some(nearest) => {
                // info!("nearest {} @ {}/{}", resource, nearest.x(), nearest.y());
                score -= calculate_distance(&center, &nearest);
            }
            None => {
                // warn!("not found: {}", resource);
                score -= 10000.;
            }
        }
    }
    // info!("scored {} in <yellow>{:?}</>", seed, started.elapsed());
    Ok(score.floor())
}

pub async fn find_nearest_resource(
    rcon: &FactorioRcon,
    search_center: &Position,
    name: &str,
) -> anyhow::Result<Option<Position>> {
    let mut entities = rcon
        .find_entities_filtered(
            &AreaFilter::PositionRadius((search_center.clone(), Some(3000.0))),
            Some(name.into()),
            None,
        )
        .await?;
    entities.sort_by(|a, b| {
        let da = calculate_distance(&a.position, &search_center);
        let db = calculate_distance(&b.position, &search_center);
        if da < db {
            Ordering::Less
        } else if da > db {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });
    Ok(entities.pop().map(|entity| entity.position))
}