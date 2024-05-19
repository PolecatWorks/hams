use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use clap::Parser;
use clap::Subcommand;
use env_logger::Env;
use ffi_log2::log_param;

use libc::c_void;

mod client;
mod config;
mod sample;

use config::Config;
use hamsrs::hams_logger_init;
use hamsrs::probes::ProbeKick;
use log::info;

use hamsrs::Hams;

use hamsrs::ProbeManual;
use hamsrs::NAME;
use hamsrs::VERSION;

use sample::{prometheus_response, prometheus_response_free};

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

#[no_mangle]
pub extern "C" fn prometheus_response0() {
    println!("Callback from C");
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
            let hams_version = hamsrs::hams_version();
            println!("HaMS version: {}", hams_version);

            let probe0 = ProbeManual::new("probe0", true).unwrap();
            println!("New Manual Probe CREATED");

            let probe1 = ProbeKick::new("probe1", Duration::from_secs(10)).unwrap();
            println!("New Kick Probe CREATED");

            let hams = Hams::new("sample").unwrap();
            println!("New HaMS CREATED");

            let state_string = String::from("Hello from Rust PROMETHEUS");

            hams.register_prometheus(
                prometheus_response,
                prometheus_response_free,
                &state_string as *const String as *const c_void,
            )
            .expect("register prometheus CBs");

            hams.alive_insert(&probe0)
                .expect("insert probe0 into alive");
            println!("Probe0 inserted into alive");

            hams.alive_insert(&probe1)
                .expect("insert probe1 into alive");

            hams.start().unwrap();
            info!("HaMS Started, now waiting for 3 secs");

            run_client_test().expect("run client test");

            thread::sleep(Duration::from_secs(1));

            hams.alive_remove(&probe0)
                .expect("remove probe0 from alive");

            hams.deregister_prometheus().expect("deregister prometheus");

            run_client_test().expect("run client test");

            thread::sleep(Duration::from_secs(10));

            hams.stop().unwrap();

            drop(probe0);
            drop(probe1);
            drop(hams);

            ExitCode::SUCCESS
        }
        None => {
            println!("No command specified");
            ExitCode::FAILURE
        }
    }
}
