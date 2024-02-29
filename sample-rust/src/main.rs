use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;
use env_logger::Env;
use sample_rust::ffi::hello_world;
use sample_rust::smoke::smokey;

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

pub fn main() {
    let log_level = Env::default().default_filter_or("info");
    env_logger::Builder::from_env(log_level).init();

    println!("Hello, world! {}:{}", NAME, VERSION);
    smokey();
    unsafe { hello_world() };
}
