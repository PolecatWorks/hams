#![warn(missing_docs)]

//! A minimal microservice built as an exec (caller) and a sharedobject. This allows the library to have exposed APIs that can be called from other languages

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use env_logger::Env;

use ffi_log2::log_param;
mod ffi;

use crate::ffi::{hams_free_ffi, hams_init_ffi, hams_logger_init_ffi};
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
            hams_logger_init_ffi(log_param()).unwrap();

            let hams = hams_init_ffi("hello").unwrap();

            info!("I have a HaMS");

            info!("Releasing my HaMS");
            hams_free_ffi(hams).unwrap();
            info!("HaMS has been released");
        }
        None => {}
    }

    // let matches = App::new("k8s uService")
    //     .version("0.1.0")
    //     .author("B.Greene <BenJGreene+github@gmail.com>")
    //     .about("Rust uService")
    //     .arg(
    //         Arg::new("library")
    //             .short('l')
    //             .long("library")
    //             .default_value("sample01")
    //             .value_name("LIBRARY")
    //             .help("Library to dynamically load for process functions. This is automatically expanded to the OS specific library name")
    //             .takes_value(true),
    //     )
    //     .arg(
    //         Arg::new("config")
    //             .short('c')
    //             .long("config")
    //             .value_name("FILE")
    //             .help("Sets a custom config file")
    //             .takes_value(true),
    //     )
    //     .arg(
    //         Arg::new("v")
    //             .short('v')
    //             .multiple_occurrences(true)
    //             .takes_value(true)
    //             .help("Sets the level of verbosity"),
    //     )
    //     .subcommand(App::new("validate").about("Validate input yaml"))
    //     .subcommand(App::new("start").about("Start service"))
    //     .subcommand(App::new("version").about("Version info"))
    //     .get_matches();

    // let library = matches
    //     .value_of("library")
    //     .expect("Library value configured");

    // if let Some(c) = matches.value_of("config") {
    //     println!("Value for config: {}", c);
    //     panic!("Config loading not implemented yet");
    // }

    // let my_config="";

    // // You can see how many times a particular flag or argument occurred
    // // Note, only flags can have multiple occurrences
    // let verbose = matches.occurrences_of("v");

    // if verbose > 0 {
    //     println!("Verbosity set to: {}", verbose);
    // }

    // match matches.subcommand() {
    //     Some(("version", _version_matches)) => {
    //         const NAME: &str = env!("CARGO_PKG_NAME");
    //         println!("Name: {}", NAME);
    //         const VERSION: &str = env!("CARGO_PKG_VERSION");
    //         println!("Version: {}", VERSION);
    //     }
    //     Some(("validate", validate_matches)) => {
    //         println!("parse and validate {:?}", validate_matches);
    //         panic!("validate not implemented yet");
    //     }
    //     Some(("start", _start_matches)) => {
    //         info!("Calling start");

    //         info!("Loading library {}", library);

    //         uservice_logger_init_ffi(log_param());

    //         let uservice = uservice_init_ffi("pear").expect("UService did not initialise");
    //         info!("Initialised UService");

    //         // let pservice_lib = so_library_register_ffi(library).expect("PService loaded");

    //         pservice_register_ffi(uservice, "apple", library).expect("Load pservice library");
    //         info!("Service loaded");

    //         pservices_init_ffi(uservice, my_config).expect("init completes");
    //         info!("PServices init completed");

    //         // Start the UService here

    //         uservice_start_ffi(uservice).expect("uservice init completes");
    //         info!("uservice completed and exited");

    //         // uservice_stop_ffi(uservice).expect("")  // NOT needed as already stopped
    //         pservice_free_ffi(uservice, "apple").expect("pservice freed");

    //         uservice_free_ffi(uservice).expect("uservice free");

    //         // so_library_free_ffi(lib);

    //         info!("service deregistered");
    //     }
    //     None => println!("No command provided"),
    //     _ => unreachable!(),
    // }
}
