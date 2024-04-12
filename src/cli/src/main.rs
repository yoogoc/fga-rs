use clap::{Parser, Subcommand};
use server::{
    config::{Config, Datasource, GrpcConfig, HttpConfig},
    Servers,
};

#[derive(Debug, Parser)]
#[command(name = "fgars")]
#[command(about = "fga cli", long_about = None, subcommand_required=true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Server {
        #[arg(default_value_t = http_default_addr(), short='a', long)]
        http_addr: String,
        // http_timeout: Option<Duration>,
        #[arg(default_value_t = grpc_default_addr(), short='g', long)]
        grpc_addr: String,
        #[arg(short = 'd')]
        db: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // env_logger::init();
    tracing_subscriber::fmt::init();

    let args = Cli::parse();
    match args.command {
        Commands::Server {
            http_addr,
            // http_timeout,
            grpc_addr,
            db,
        } => {
            let mut config = Config {
                datasource: Datasource { uri: db },
                ..Default::default()
            };
            config.http = Some(HttpConfig {
                addr: http_addr,
                ..Default::default() // timeout: http_timeout,
            });
            config.grpc = Some(GrpcConfig {
                addr: grpc_addr,
                ..Default::default() // timeout: grpc_timeout,
            });
            let servers = Servers::new(config).await;
            servers.start().await?;
        }
    }
    Ok(())
}

fn http_default_addr() -> String {
    String::from("0.0.0.0:5555")
}

fn grpc_default_addr() -> String {
    String::from("0.0.0.0:5556")
}
