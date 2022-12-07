#![warn(missing_docs)]

//! A minimal microservice built as an exec (caller) and a sharedobject. This allows the library to have exposed APIs that can be called from other languages

use std::{path::PathBuf, thread::sleep, time::Duration};

use clap::{Parser, Subcommand};
use env_logger::Env;

use ffi_log2::log_param;
use sample_rust::{self, hams_logger_init, Hams};

use log::info;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

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

pub fn main() {
    //! Initialise the shared library
    let log_level = Env::default().default_filter_or("info");
    env_logger::Builder::from_env(log_level).init();

    let cli = Cli::parse();

    if let Some(name) = cli.name.as_deref() {
        info!("Value for name: {}", name);
    }

    if let Some(config_path) = cli.config.as_deref() {
        info!("Value for config: {}", config_path.display());
    }

    match cli.debug {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    match &cli.command {
        Some(Commands::Test { list }) => {
            if *list {
                println!("Printing testing lists...");
            } else {
                println!("Not printing testing lists...");
            }
        }
        Some(Commands::Validate {}) => {
            println!("Validating");
            todo!("Implement validate functions")
        }
        Some(Commands::Start {}) => {
            info!("Start");
            hams_logger_init(log_param()).unwrap();

            let hams = Hams::new("hello").unwrap();

            info!("I have a HaMS");
            hams.start().expect("HaMS started successfully");

            let sleep_time = 10;
            info!("Sleeping for {} secs", sleep_time);
            sleep(Duration::from_secs(sleep_time));

            hams.stop().expect("HaMS stopped successfully");

            info!("HaMS will be released... by Drop");
        }
        None => {}
    }
}
