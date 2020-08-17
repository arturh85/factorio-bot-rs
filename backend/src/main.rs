use clap::{App, Arg};
use factorio_bot_backend::build_rocket;
use factorio_bot_backend::factorio::process_control::start_factorio;
use factorio_bot_backend::factorio::rcon::FactorioRcon;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    color_eyre::install().unwrap();
    let matches = App::new("factorio-bot-rs")
        .version("1.0")
        .author("Artur Hallmann <arturh@arturh.de>")
        .about("Bot for Factorio")
        .subcommand(
            App::new("rcon")
                .arg(Arg::with_name("command").required(true).last(true))
                .arg(
                    Arg::with_name("server")
                        .short("server")
                        .long("server")
                        .value_name("server")
                        .required(false)
                        .help("connect to server instead of starting a server"),
                )
                .about("send given rcon command"),
        )
        .subcommand(
            App::new("start")
                .about("start the factorio server and clients + web server")
                .arg(
                    Arg::with_name("clients")
                        .short("c")
                        .long("clients")
                        .default_value("1")
                        .help("number of clients to start in addition to the server"),
                )
                .arg(
                    Arg::with_name("server")
                        .short("server")
                        .long("server")
                        .value_name("server")
                        .required(false)
                        .help("connect to server instead of starting a server"),
                )
                .arg(
                    Arg::with_name("seed")
                        .long("seed")
                        .value_name("seed")
                        .required(false)
                        .help("recreate server level with given seed"),
                )
                .arg(
                    Arg::with_name("logs")
                        .short("l")
                        .long("logs")
                        .help("enabled writing server & client logs to workspace"),
                )
                .about("start given number of clients after server start"),
        )
        .get_matches();

    let mut settings = config::Config::default();
    settings
        // Add in `./Rocket.toml`
        .merge(config::File::with_name("Settings"))
        .unwrap()
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .merge(config::Environment::with_prefix("APP"))
        .unwrap();

    if let Some(matches) = matches.subcommand_matches("start") {
        let clients: u8 = matches.value_of("clients").unwrap().parse().unwrap();
        let write_logs: bool = matches.is_present("logs");
        let seed = matches.value_of("seed");
        let server_host = matches.value_of("server");
        let (world, rcon) = start_factorio(&settings, server_host, clients, seed, write_logs)
            .await
            .expect("failed to start factorio");

        if let Some(world) = world {
            build_rocket(settings, rcon, world)
                .await
                .launch()
                .await
                .unwrap();
        }
    } else if let Some(matches) = matches.subcommand_matches("rcon") {
        let command = matches.value_of("command").unwrap();
        let server_host = matches.value_of("server");
        let rcon = FactorioRcon::new(&settings, server_host).await.unwrap();
        rcon.send(command).await.unwrap();
    } else {
        eprintln!("Missing required Sub Command!");
        std::process::exit(1);
    }

    Ok(())
}
