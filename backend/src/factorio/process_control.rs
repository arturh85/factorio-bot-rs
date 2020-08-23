use crate::factorio::instance_setup::setup_factorio_instance;
use crate::factorio::output_parser::FactorioWorld;
use crate::factorio::output_reader::read_output;
use crate::factorio::rcon::FactorioRcon;
use async_std::sync::channel;
use config::Config;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Instant;

pub async fn start_factorio(
    settings: &Config,
    server_host: Option<&str>,
    client_count: u8,
    recreate: bool,
    map_exchange_string: Option<&str>,
    seed: Option<&str>,
    write_logs: bool,
) -> anyhow::Result<(Option<Arc<FactorioWorld>>, FactorioRcon)> {
    let mut world: Option<Arc<FactorioWorld>> = None;
    if server_host.is_none() {
        setup_factorio_instance(&settings, "server", recreate, map_exchange_string, seed).await?;
        let started = Instant::now();
        world = Some(start_factorio_server(&settings, write_logs).await?);
        let rcon = FactorioRcon::new(&settings, server_host, false).await?;
        success!(
            "Started <bright-blue>server</> in <yellow>{:?}</>",
            started.elapsed()
        );
        rcon.silent_print("").await?;
        rcon.whoami("server").await?;
    }
    let settings = settings.clone();
    // tokio::spawn(async move {
    let rcon = FactorioRcon::new(&settings, server_host, false)
        .await
        .unwrap();
    for instance_number in 0..client_count {
        let instance_name = format!("client{}", instance_number + 1);
        if let Err(err) =
            setup_factorio_instance(&settings, &instance_name, false, None, None).await
        {
            error!("Failed to setup Factorio <red>{}</>: ", err);
            break;
        }
        let started = Instant::now();
        if let Err(err) =
            start_factorio_client(&settings, instance_name.clone(), server_host, write_logs).await
        {
            error!("Failed to start Factorio <red>{}</>", err);
            break;
        }
        success!(
            "Started <bright-blue>{}</> in <yellow>{:?}</>",
            &instance_name,
            started.elapsed()
        );
        rcon.whoami(&instance_name).await.unwrap();
        // Execute a dummy command to silence the warning about "using commands will
        // disable achievements". If we don't do this, the first command will be lost
        rcon.silent_print("").await.unwrap();
    }
    // });
    Ok((world, rcon))
}

pub async fn start_factorio_server(
    settings: &Config,
    write_logs: bool,
) -> anyhow::Result<Arc<FactorioWorld>> {
    let instance_name = "server";
    let workspace_path: String = settings.get("workspace_path")?;
    let workspace_path = Path::new(&workspace_path);
    if !workspace_path.exists() {
        error!(
            "Failed to find workspace at <bright-blue>{:?}</>",
            workspace_path
        );
        std::process::exit(1);
    }
    let instance_path = workspace_path.join(PathBuf::from(instance_name));
    let instance_path = Path::new(&instance_path);
    if !instance_path.exists() {
        error!(
            "Failed to find instance at <bright-blue>{:?}</>",
            instance_path
        );
        std::process::exit(1);
    }
    let binary = if cfg!(windows) {
        "bin/x64/factorio.exe"
    } else {
        "bin/x64/factorio"
    };
    let factorio_binary_path = instance_path.join(PathBuf::from(binary));
    if !factorio_binary_path.exists() {
        error!(
            "factorio binary missing at <bright-blue>{:?}</>",
            factorio_binary_path
        );
        std::process::exit(1);
    }
    let saves_path = instance_path.join(PathBuf::from("saves"));
    if !saves_path.exists() {
        error!("saves missing at <bright-blue>{:?}</>", saves_path);
        std::process::exit(1);
    }
    let saves_level_path = saves_path.join(PathBuf::from("level.zip"));
    if !saves_level_path.exists() {
        error!(
            "save file missing at <bright-blue>{:?}</>",
            saves_level_path
        );
        std::process::exit(1);
    }
    let server_settings_path = instance_path.join(PathBuf::from("server-settings.json"));
    if !server_settings_path.exists() {
        error!(
            "server settings missing at <bright-blue>{:?}</>",
            server_settings_path
        );
        std::process::exit(1);
    }
    let rcon_port: String = settings.get("rcon_port")?;
    let rcon_pass: String = settings.get("rcon_pass")?;
    let args = &[
        "--start-server",
        saves_level_path.to_str().unwrap(),
        "--rcon-port",
        &rcon_port,
        "--rcon-password",
        &rcon_pass,
        "--server-settings",
        &server_settings_path.to_str().unwrap(),
    ];
    info!(
        "Starting <bright-blue>server</> at {:?} with {:?}",
        &instance_path, &args
    );
    let mut child = Command::new(&factorio_binary_path)
        .args(args)
        // .stdout(Stdio::from(outputs))
        // .stderr(Stdio::from(errors))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start server");

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    let log_path = workspace_path.join(PathBuf::from_str(&"server-log.txt").unwrap());
    thread::spawn(move || {
        let exit_code = child.wait().expect("failed to wait for server");
        if let Some(code) = exit_code.code() {
            error!("<red>server stopped</> with exit code <yellow>{}</>", code);
        } else {
            error!("<red>server stopped</> without exit code");
        }
    });
    let (rx, world) = read_output(reader, log_path, write_logs, false).await?;
    // await for factorio to start before returning
    rx.recv().unwrap();
    Ok(world)
}

pub async fn start_factorio_client(
    settings: &Config,
    instance_name: String,
    server_host: Option<&str>,
    write_logs: bool,
) -> anyhow::Result<JoinHandle<ExitStatus>> {
    let workspace_path: String = settings.get("workspace_path")?;
    let workspace_path = Path::new(&workspace_path);
    if !workspace_path.exists() {
        error!(
            "Failed to find workspace at <bright-blue>{:?}</>",
            workspace_path
        );
        std::process::exit(1);
    }
    let instance_path = workspace_path.join(PathBuf::from(&instance_name));
    let instance_path = Path::new(&instance_path);
    if !instance_path.exists() {
        error!(
            "Failed to find instance at <bright-blue>{:?}</>",
            instance_path
        );
        std::process::exit(1);
    }
    let binary = if cfg!(windows) {
        "bin/x64/factorio.exe"
    } else {
        "bin/x64/factorio"
    };
    let factorio_binary_path = instance_path.join(PathBuf::from(binary));
    if !factorio_binary_path.exists() {
        error!(
            "factorio binary missing at <bright-blue>{:?}</>",
            factorio_binary_path
        );
        std::process::exit(1);
    }
    let args = &[
        "--mp-connect",
        server_host.unwrap_or("localhost"),
        // "--graphics-quality", "very-low",
        // "--force-graphics-preset", "very-low",
        // "--video-memory-usage", "low",

        // "--gfx-safe-mode",
        // "--low-vram",
        "--disable-audio",
        "--window-size",
        "maximized",
    ];
    info!(
        "Starting <bright-blue>{}</> at {:?} with {:?}",
        &instance_name, &instance_path, &args
    );
    let mut child = Command::new(&factorio_binary_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start client");
    let instance_name = instance_name.clone();
    let log_instance_name = instance_name.clone();
    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    let log_filename = format!(
        "{}/{}-log.txt",
        workspace_path.to_str().unwrap(),
        instance_name
    );
    let mut log_file = match write_logs {
        true => Some(File::create(log_filename)?),
        false => None,
    };
    let handle = thread::spawn(move || {
        let exit_code = child.wait().expect("failed to wait for client");
        if let Some(code) = exit_code.code() {
            error!(
                "<red>{} stopped</> with exit code <yellow>{}</>",
                &instance_name, code
            );
        } else {
            error!("<red>{} stopped</> without exit code", &instance_name);
        }
        exit_code
    });
    let is_client = server_host.is_some();
    let (tx, rx) = channel(100);
    tokio::spawn(async move {
        let mut initialized = false;
        for line in reader.lines() {
            if let Ok(line) = line {
                // wait for factorio init before sending confirmation
                if !initialized && line.find("my_client_id").is_some() {
                    initialized = true;
                    // info!("XXX player_path XXX CLIENT START SENDING");
                    tx.send(()).await;
                    // info!("XXX player_path XXX CLIENT START SEND");
                }
                log_file.iter_mut().for_each(|log_file| {
                    // filter out 6.6 million lines like 6664601 / 6665150...
                    if initialized || !line.contains(" / ") {
                        log_file
                            .write_all(line.as_bytes())
                            .expect("failed to write log file");
                        log_file.write_all(b"\n").expect("failed to write log file");
                    }
                });
                if is_client && !line.contains(" / ") && !line.starts_with("§") {
                    info!("<cyan>{}</>⮞ <magenta>{}</>", &log_instance_name, line);
                }
            } else {
                error!("failed to read client log");
                break;
            }
        }
    });
    rx.recv().await?;
    Ok(handle)
}
