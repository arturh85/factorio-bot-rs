use crate::factorio::output_parser::{FactorioWorld, OutputParser};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::ChildStdout;
use std::sync::{mpsc, Arc};

pub async fn read_output(
    reader: BufReader<ChildStdout>,
    log_path: PathBuf,
    write_logs: bool,
) -> anyhow::Result<(mpsc::Receiver<()>, Arc<FactorioWorld>)> {
    let mut log_file = match write_logs {
        true => Some(File::create(log_path)?),
        false => None,
    };
    let mut parser = OutputParser::new();

    let (tx, rx) = mpsc::channel();
    let world = parser.world();
    tokio::spawn(async move {
        let lines = reader.lines();
        let mut initialized = false;
        for line in lines {
            match line {
                Ok(line) => {
                    // wait for factorio init before sending confirmation
                    if !initialized && line.find("my_client_id").is_some() {
                        initialized = true;
                        // info!("XXX player_path XXX SERVER START SENDING");
                        tx.send(()).unwrap();
                        // info!("XXX player_path XXX SERVER START SEND");
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
                                    match action {
                                        "on_player_changed_position"
                                        | "on_player_main_inventory_changed"
                                        | "entity_prototypes"
                                        | "recipes"
                                        | "item_prototypes"
                                        | "graphics"
                                        | "tiles"
                                        | "resources"
                                        | "objects" => {}
                                        _ => {
                                            info!(
                                                "<cyan>server</>⮞ §{}§<bright-blue>{}</>§<green>{}</>",
                                                tick, action, rest
                                            );
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
                        } else {
                            if line.contains("Error") {
                                warn!("<cyan>server</>⮞ <red>{}</>", line);
                            } else {
                                info!("<cyan>server</>⮞ <magenta>{}</>", line);
                            }
                        }
                    }
                }
                Err(err) => {
                    error!("<red>failed to read server log: {}</>", err);
                    break;
                }
            };
        }
    });
    Ok((rx, world))
}
