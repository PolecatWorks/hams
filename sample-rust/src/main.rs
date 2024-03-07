use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;
use env_logger::Env;
use ffi_log2::log_param;
use log::info;
use sample_rust::config::Config;
use sample_rust::hams_logger_init;
use sample_rust::hello_world;
use sample_rust::smoke::smokey;

use sample_rust::Hams;
use sample_rust::ProbeManual;
use sample_rust::NAME;
use sample_rust::VERSION;

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },
    /// Validate the configuration
    Validate {},
    /// Start the service
    Start {},
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

pub fn main() -> ExitCode {
    let log_level = Env::default().default_filter_or("info");
    env_logger::Builder::from_env(log_level).init();

    hams_logger_init(log_param()).unwrap();

    let cli = Cli::parse();

    info!("Value for config: {:?}", cli.config);

    let config: Config = Config::figment(cli.config)
        .extract()
        .expect("Config file loaded");

    match cli.command {
        Some(Commands::Test { list }) => {
            if list {
                println!("Listing test values");
            } else {
                println!("Testing things");
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Validate {}) => {
            println!("Validating the configuration");
            println!("Config: {:?}", config);
            ExitCode::SUCCESS
        }
        Some(Commands::Start {}) => {
            println!("Starting the service");
            println!("Sample version: {}:{}", NAME, VERSION);
            let hams_version = sample_rust::hams_version();
            println!("HaMS version: {}", hams_version);
            smokey();
            hello_world();

            let probe = ProbeManual::manual_new("test", true).unwrap();
            println!("New Manual Probe CREATED");
            drop(probe);

            let hams = Hams::new("sample").unwrap();
            drop(hams);

            ExitCode::SUCCESS
        }
        None => {
            println!("No command specified");
            ExitCode::FAILURE
        }
    }
}
