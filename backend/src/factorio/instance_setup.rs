use archiver_rs::{Archive, Compressed};
use async_std::fs::create_dir;
use config::Config;
use indicatif::HumanDuration;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use paris::Logger;
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

pub async fn setup_factorio_instance(
    settings: &Config,
    instance_name: &str,
    seed: Option<&str>,
) -> anyhow::Result<()> {
    let is_server = instance_name == "server";
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
        info!("Creating <bright-blue>{:?}</>", &instance_path);
        create_dir(instance_path).await?;
    }
    let readdir = instance_path.read_dir()?;
    if readdir.count() == 0 {
        let archive_path: String = settings.get("archive_path")?;
        let archive_path = Path::new(&archive_path);
        if !archive_path.exists() {
            error!(
                "Failed to find factorio archive (.zip or .tar.xz) file at <bright-blue>{:?}</>. You may change this in <bright-blue>Settings.toml</>",
                archive_path
            );
            std::process::exit(1);
        }
        info!(
            "Extracting <bright-blue>{}</> to <magenta>{}</>",
            &archive_path.to_str().unwrap(),
            instance_path.to_str().unwrap()
        );
        let started = Instant::now();
        let extension = archive_path
            .extension()
            .expect("archive needs extension!")
            .to_str()
            .unwrap();
        match &extension[..] {
            "zip" => {
                let mut archive = archiver_rs::open(archive_path)?;
                let files = archive.files().unwrap();
                let bar = ProgressBar::new(files.len() as u64);
                bar.set_draw_target(ProgressDrawTarget::stdout());
                bar.set_style(
                    ProgressStyle::default_spinner().template("{msg}\n{wide_bar} {pos}/{len}"),
                );
                for file in files {
                    let message = format!("extracting {}", &file);
                    bar.set_message(&message);
                    bar.tick();
                    // output_path is like Factorio_0.18.36\bin\x64\factorio.exe
                    let output_path = PathBuf::from(&file);
                    // output_path is like bin\x64\factorio.exe
                    let output_path =
                        output_path.strip_prefix(output_path.components().next().unwrap())?;
                    // output_path is like $instance_path\bin\x64\factorio.exe
                    let output_path = PathBuf::from(instance_path).join(PathBuf::from(output_path));

                    if (&*file).ends_with('/') {
                        fs::create_dir_all(&output_path)?;
                    } else {
                        if let Some(p) = output_path.parent() {
                            if !p.exists() {
                                fs::create_dir_all(&p)?;
                            }
                        }
                        archive.extract_single(&output_path, file).unwrap();
                    }
                    bar.inc(1);
                }
                bar.finish();
            }
            "xz" => {
                let tar_path = archive_path.to_path_buf().with_extension("");
                if !tar_path.exists() {
                    let mut logger = Logger::new();
                    logger.loading(format!(
                        "Uncompressing <bright-blue>{}</> to <magenta>{}</> ...",
                        &archive_path.to_str().unwrap(),
                        tar_path.to_str().unwrap()
                    ));
                    let mut archive = archiver_rs::Xz::open(archive_path)?;
                    archive
                        .decompress(&tar_path)
                        .expect("failed to decompress xz");
                    logger.success(format!(
                        "Uncompressed <bright-blue>{}</> to <magenta>{}</>",
                        &archive_path.to_str().unwrap(),
                        tar_path.to_str().unwrap()
                    ));
                }
                let mut logger = Logger::new();
                logger.loading(format!(
                    "Extracting <bright-blue>{}</> to <magenta>{}</> ...",
                    &tar_path.to_str().unwrap(),
                    workspace_path.to_str().unwrap()
                ));
                let mut archive = archiver_rs::Tar::open(&tar_path).unwrap();
                archive.extract(workspace_path).expect("failed to extract");
                logger.success("Extraction finished");

                let extracted_path = workspace_path.join(PathBuf::from("factorio"));
                if extracted_path.exists() {
                    std::fs::remove_dir(&instance_path).expect("failed to delete empty folder");
                    std::fs::rename(&extracted_path, instance_path).expect("failed to rename");
                    success!("Renamed {:?} to {:?}", &extracted_path, instance_path);
                } else {
                    error!("Failed to find {:?}", &extracted_path);
                }
            }
            _ => panic!("unsupported archive format"),
        }

        info!(
            "Extracting took <yellow>{}</>",
            HumanDuration(started.elapsed())
        );
    }

    let mods_path = instance_path.join(PathBuf::from("mods"));
    if !mods_path.exists() {
        info!("Creating <bright-blue>{:?}</>", &mods_path);
        create_dir(&mods_path).await?;
    }
    let mod_info_path = mods_path.join(PathBuf::from("mod-list.json"));
    if !mod_info_path.exists() {
        let template_file = include_bytes!("../data/mod-list.json");
        let mut outfile = fs::File::create(&mod_info_path)?;
        info!("Creating <bright-blue>{:?}</>", &mod_info_path);
        outfile.write_all(template_file)?;
    }
    let data_botbridge_path = std::fs::canonicalize(PathBuf::from("mod"))?;
    let mods_botbridge_path = mods_path.join(PathBuf::from("BotBridge"));
    if !mods_botbridge_path.exists() {
        info!(
            "Creating Symlink for <bright-blue>{:?}</>",
            &mods_botbridge_path
        );
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&data_botbridge_path, &mods_botbridge_path)?;
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(&data_botbridge_path, &mods_botbridge_path)?;
        }
    }
    let script_output_put = instance_path.join(PathBuf::from("script-output"));
    if script_output_put.exists() {
        for entry in fs::read_dir(script_output_put)? {
            let entry = entry.unwrap();
            std::fs::remove_file(entry.path())
                .unwrap_or_else(|_| panic!("failed to delete {}", entry.path().to_str().unwrap()));
        }
    }
    if is_server {
        let server_settings_path = instance_path.join(PathBuf::from("server-settings.json"));
        if !server_settings_path.exists() {
            let server_settings_data = include_bytes!("../data/server-settings.json");
            let mut outfile = fs::File::create(&server_settings_path)?;
            info!("Creating <bright-blue>{:?}</>", &server_settings_path);
            // io::copy(&mut template_file, &mut outfile)?;
            outfile.write_all(server_settings_data)?;
        }

        let saves_path = instance_path.join(PathBuf::from("saves"));
        if !saves_path.exists() {
            info!("Creating <bright-blue>{:?}</>", &saves_path);
            create_dir(&saves_path).await?;
        }

        let saves_level_path = saves_path.join(PathBuf::from("level.zip"));
        if saves_level_path.exists() && seed.is_some() {
            std::fs::remove_file(&saves_level_path).unwrap_or_else(|_| {
                panic!("failed to delete {}", &saves_level_path.to_str().unwrap())
            });
        }
        if !saves_level_path.exists() {
            let mut logger = Logger::new();
            logger.loading(format!(
                "Creating Level at <bright-blue>{:?}</>...",
                &saves_level_path
            ));

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
            let mut args = vec!["--create", saves_level_path.to_str().unwrap()];
            if let Some(seed) = seed {
                args.push("--map-gen-seed");
                args.push(seed);
            }
            let output = Command::new(&factorio_binary_path)
                .args(args)
                .output()
                .expect("failed to run factorio --create");

            if !saves_level_path.exists() {
                error!(
                    "failed to create factorio level. Output: \n\n{}\n\n{}",
                    std::str::from_utf8(&output.stdout).unwrap(),
                    std::str::from_utf8(&output.stderr).unwrap()
                );
                std::process::exit(1);
            }
            logger.success(format!(
                "Created Level at <bright-blue>{:?}</>",
                &saves_level_path
            ));
        }
    } else {
        let player_data_path = instance_path.join(PathBuf::from("player-data.json"));
        if !player_data_path.exists() {
            let player_data = include_bytes!("../data/player-data.json");
            let mut outfile = fs::File::create(&player_data_path)?;
            outfile.write_all(player_data)?;
            info!("Created <bright-blue>{:?}</>", &player_data_path);
        }
        let player_data_content = std::fs::read_to_string(&player_data_path)?;
        let mut value: Value = serde_json::from_str(player_data_content.as_str())?;
        value["service-username"] = Value::from(instance_name);
        let player_data_file = File::create(&player_data_path)?;
        serde_json::to_writer_pretty(player_data_file, &value)?;
    }

    Ok(())
}
