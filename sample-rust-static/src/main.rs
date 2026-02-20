use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use clap::Parser;
use clap::Subcommand;
use env_logger::Env;
use ffi_log2::log_param;

use hams::hams::config::TaskConfig;
use hams::probe::manual::Manual;
use hams::probe::kick::Kick;
use hams::probe::FFIProbe;
use hams::probe::AsyncHealthProbe;
use hams::hams::Hams;

mod client;
mod config;

use config::Config;
use log::info;
use std::sync::Arc;

use crate::client::run_client_test;

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

            let mut hams = Hams::new(config.hams.clone());
            println!("HaMS version: {}", hams.hams_version);

            let probe0 = Manual::new("probe0", true);
            println!("New Manual Probe CREATED");

            let probe1 = Kick::new("probe1", Duration::from_secs(10));
            println!("New Kick Probe CREATED");

            let state_string = String::from("Hello from Rust PROMETHEUS");

            hams.register_prometheus_closure(move || {
                format!("test {}", state_string)
            }).expect("register prometheus closure");

            // Insert probes
            // Note: FFIProbe adapts synchronous HealthProbe (like Manual/Kick) to AsyncHealthProbe
            let ffi_probe0 = Box::new(FFIProbe::from(probe0.clone()));
            hams.alive_insert(ffi_probe0);
            println!("Probe0 inserted into alive");

            let ffi_probe1 = Box::new(FFIProbe::from(probe1.clone()));
            hams.alive_insert(ffi_probe1);

            let ffi_probe0_startup = Box::new(FFIProbe::from(probe0.clone()));
            hams.startup_insert(ffi_probe0_startup);

            // Add a startup task that just sleeps for a bit
            let startup_config = TaskConfig {
                retries: 3,
                sleep_ms: 100,
                timeout_ms: 5000,
            };
            hams.startup_task_insert(
                Arc::new(FFIProbe::from(probe0.clone())),
                startup_config
            );

            // Add a shutdown task
            let shutdown_config = TaskConfig {
                retries: 1,
                sleep_ms: 100,
                timeout_ms: 1000,
            };
            hams.shutdown_task_insert(
                Arc::new(FFIProbe::from(probe0.clone())),
                shutdown_config
            );

            info!("HaMS Created, now starting it");

            hams.start().unwrap();
            info!("HaMS Started, now waiting for 3 secs");

            run_client_test().expect("run client test");

            thread::sleep(Duration::from_secs(1));

            let probe0_to_remove = Box::new(FFIProbe::from(probe0.clone())) as Box<dyn AsyncHealthProbe>;
            hams.alive_remove(&probe0_to_remove);
            println!("Probe0 removed from alive");

            hams.deregister_prometheus().expect("deregister prometheus");

            run_client_test().expect("run client test");

            info!("HaMS Started, running for a while");

            thread::sleep(Duration::from_secs(5));

            hams.stop().unwrap();

            drop(probe0);
            drop(probe1);
            // drop(hams); // hams is dropped at end of scope

            ExitCode::SUCCESS
        }
        None => {
            println!("No command specified");
            ExitCode::FAILURE
        }
    }
}
