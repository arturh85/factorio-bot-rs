use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::ChildStdout;
use std::sync::Arc;

use actix::Addr;

use crate::factorio::output_parser::OutputParser;
use crate::factorio::process_control::FactorioStartCondition;
use crate::factorio::rcon::{FactorioRcon, RconSettings};
use crate::factorio::world::FactorioWorld;
use crate::factorio::ws::FactorioWebSocketServer;

pub async fn read_output(
    reader: BufReader<ChildStdout>,
    rcon_settings: &RconSettings,
    log_path: PathBuf,
    websocket_server: Option<Addr<FactorioWebSocketServer>>,
    write_logs: bool,
    silent: bool,
    wait_until: FactorioStartCondition,
) -> anyhow::Result<(Arc<FactorioWorld>, Arc<FactorioRcon>)> {
    let mut log_file = match write_logs {
        true => Some(File::create(log_path)?),
        false => None,
    };
    let mut parser = OutputParser::new(websocket_server);

    let wait_until_thread = wait_until.clone();
    let (tx1, rx1) = async_std::sync::channel(1);
    tx1.send(()).await;
    let (tx2, rx2) = async_std::sync::channel(1);
    tx2.send(()).await;
    let world = parser.world();
    std::thread::spawn(move || {
        actix::run(async move {
            let lines = reader.lines();
            let mut initialized = false;
            for line in lines {
                match line {
                    Ok(line) => {
                        // after we receive this line we can connect via rcon
                        if !initialized && line.find("my_client_id").is_some() {
                            rx1.recv().await.unwrap();
                            rx1.recv().await.unwrap();
                            if wait_until_thread == FactorioStartCondition::Initialized {
                                initialized = true;
                            }
                        }
                        // wait for factorio init before sending confirmation
                        if !initialized && (line.find("initial discovery done").is_some() || line.find("(100% done)").is_some()) {
                            initialized = true;
                            parser.on_init().unwrap();
                            rx2.recv().await.unwrap();
                            rx2.recv().await.unwrap();
                        }
                        // filter out 6 million lines like 6664601 / 6665150
                        if initialized || !line.contains(" / ") {
                            log_file.iter_mut().for_each(|log_file| {
                                log_file
                                    .write_all(line.as_bytes())
                                    .expect("failed to write log file");
                                log_file.write_all(b"\n").expect("failed to write log file");
                            });

                            if !line.is_empty() && &line[0..2] == "§" {
                                if let Some(pos) = line[2..].find('§') {
                                    let tick: u64 = (&line[2..pos + 2]).parse().unwrap();
                                    let rest = &line[pos + 4..];
                                    if let Some(pos) = rest.find('§') {
                                        let action = &rest[0..pos];
                                        let rest = &rest[pos + 2..];
                                        if !silent {
                                            match action {
                                                "on_player_changed_position"
                                                | "on_player_main_inventory_changed"
                                                | "on_player_changed_distance"
                                                | "entity_prototypes"
                                                | "recipes"
                                                | "force"
                                                | "item_prototypes"
                                                | "graphics"
                                                | "tiles"
                                                | "STATIC_DATA_END"
                                                | "entities"
                                                 => {}
                                                _ => {
                                                    info!(
                                                        "<cyan>server</>⮞ §{}§<bright-blue>{}</>§<green>{}</>",
                                                        tick, action, rest
                                                    );
                                                }
                                            }
                                        }

                                        let result = parser.parse(tick, action, rest).await;
                                        if let Err(err) = result {
                                            error!(
                                                "<red>failed to parse</> <bright-blue>'{}'</>",
                                                line
                                            );
                                            error!("<red>error: {}</>", err);
                                        }
                                    }
                                }
                            } else if line.contains("Error") && !silent {
                                warn!("<cyan>server</>⮞ <red>{}</>", line);
                            } else if !silent {
                                info!("<cyan>server</>⮞ <magenta>{}</>", line);
                            }
                        }
                    }
                    Err(err) => {
                        error!("<red>failed to read server log: {}</>", err);
                        break;
                    }
                };
            }
        }).unwrap();
    });
    tx1.send(()).await;
    let rcon = Arc::new(
        FactorioRcon::new(rcon_settings, silent)
            .await
            .expect("failed to rcon"),
    );
    rcon.silent_print("").await.expect("failed to silent print");
    rcon.whoami("server").await.expect("failed to whoami");
    rcon.send("/silent-command game.surfaces[1].always_day=true")
        .await
        .expect("always day");

    if wait_until == FactorioStartCondition::DiscoveryComplete {
        tx2.send(()).await;
    }

    Ok((world, rcon))
}
