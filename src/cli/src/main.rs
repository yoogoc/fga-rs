use clap::{Parser, Subcommand};
use migration::{
    run_migrate,
    sea_orm::{ConnectOptions, Database},
};
use sea_orm_cli::MigrateSubcommands;
use server::{
    config::{Config, Datasource, GrpcConfig, HttpConfig},
    Servers,
};

#[derive(Debug, Parser)]
#[command(name = "fgars")]
#[command(about = "fga cli", long_about = None, subcommand_required=true)]
struct Cli {
    #[arg(
        global = true,
        short = 's',
        long,
        env = "DATABASE_SCHEMA",
        long_help = "Database schema\n - For MySQL and SQLite, this argument is ignored.\n - For PostgreSQL, this argument is optional with default value 'public'.\n"
    )]
    database_schema: Option<String>,

    #[arg(global = true, short = 'u', long, env = "DATABASE_URL", help = "Database URL")]
    database_url: Option<String>,

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
    },
    Migration {
        #[command(subcommand)]
        command: Option<MigrateSubcommands>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let url = args.database_url.expect("Environment variable 'DATABASE_URL' not set");
    let schema = args.database_schema.unwrap_or_else(|| "public".to_owned());

    match args.command {
        Commands::Server {
            http_addr,
            // http_timeout,
            grpc_addr,
        } => {
            // env_logger::init();
            tracing_subscriber::fmt::init();

            let mut config = Config {
                datasource: Datasource { uri: url, schema },
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
        Commands::Migration { command } => {
            let connect_options = ConnectOptions::new(url).set_schema_search_path(schema).to_owned();
            let db = &Database::connect(connect_options)
                .await
                .expect("Fail to acquire database connection");

            run_migrate(db, command, false)
                .await
                .map_err(|err| anyhow::anyhow!(err.to_string()))?;
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
