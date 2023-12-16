use anyhow::Context;
use anyhow::Result;
use delta_sharing::config;
use delta_sharing::logging;
use delta_sharing::server::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let app = clap::Command::new("delta-sharing")
        .author("Shingo OKAWA <shingo.okawa.g.h.c@gmail.com>")
        .version(delta_sharing::VERSION)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            clap::Command::new("server")
                .about("Launch the server process")
                .after_help("The server implements Delta Sharing REST protocol."),
        );
    let args = app.get_matches();
    match args.subcommand().expect("subcommand is required") {
        ("server", _args) => {
            logging::setup();
            tracing::info!("delta sharing server is starting");
            tracing::debug!(
                db_url = config::fetch::<String>("db_url"),
                server_addr = config::fetch::<String>("server_addr"),
                server_bind = config::fetch::<String>("server_bind"),
                jwt_secret = config::fetch::<String>("jwt_secret"),
                signed_url_ttl = config::fetch::<i64>("signed_url_ttl"),
                use_json_log = config::fetch::<bool>("use_json_log"),
                log_filter = config::fetch::<String>("log_filter"),
            );
            let server = Server::new().await.context("failed to create server")?;
            server.start().await.context("failed to start server")
        }
        _ => unreachable!("clap should have already checked the subcommands"),
    }
}
