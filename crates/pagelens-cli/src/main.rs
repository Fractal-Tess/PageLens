use clap::Parser;
use pagelens_cli::cli::{open_browser_for_server, run_audit, run_server, Cli};
use pagelens_logging::{error, info};
use std::net::IpAddr;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    pagelens_logging::init("pagelens-cli");
    let cli = Cli::parse();

    if cli.serve {
        let host = match cli.host.parse::<IpAddr>() {
            Ok(host) => host,
            Err(err) => {
                error!(host = %cli.host, error = %err, "Invalid --host value");
                eprintln!("Error: invalid --host '{}': {}", cli.host, err);
                std::process::exit(1);
            }
        };

        let db_path = cli
            .db_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(pagelens_cli::cli::default_server_db_path);

        info!(host = %host, port = cli.port, db_path = %db_path.display(), "Starting PageLens server mode");
        open_browser_for_server(host, cli.port);

        if let Err(err) = run_server(host, cli.port, db_path).await {
            error!(error = %err, "Server mode failed");
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
        return;
    }

    let (url, request_headers) = match cli.validate_and_extract() {
        Ok(result) => result,
        Err(err) => {
            error!(error = %err, "Invalid CLI arguments");
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    };

    info!(url = %url, mode = ?cli.mode, json = cli.json, seo_only = cli.seo, "Starting audit run");

    if let Err(e) = run_audit(
        &url,
        cli.json,
        cli.seo,
        cli.mode,
        cli.assets,
        cli.asset_sort,
        cli.asset_group,
        &request_headers,
        cli.perf,
        cli.contrast,
        cli.site_files,
    )
    .await
    {
        error!(url = %url, error = %e, "Audit run failed");
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    info!(url = %url, "Audit run completed");
}
