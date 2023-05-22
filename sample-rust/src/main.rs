#![warn(missing_docs)]

//! A minimal microservice built as an exec (caller) and a sharedobject. This allows the library to have exposed APIs that can be called from other languages

use std::{path::PathBuf, sync::atomic::Ordering, thread::sleep, time::Duration};

use clap::{Parser, Subcommand};
use env_logger::Env;

use ffi_log2::log_param;

use sample_rust::{self, hams_logger_init, AliveCheckKicked, Hams};

use log::{error, info};
mod sample;
mod sampleerror;

use sample::{Sample, SampleConfig};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,

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

    let config: SampleConfig = SampleConfig::figment(cli.config)
        .extract()
        .expect("Config file loaded");

    info!("Loaded config as {:?}", config);

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
            error!("Doing nothing here yet!!!")
            // todo!("Implement validate functions")
        }
        Some(Commands::Start {}) => {
            let mut sample_service = Sample::new("sample1", config);

            info!("Start");

            let my_running = sample_service.running();

            hams_logger_init(log_param()).unwrap();
            let hams = Hams::new("hello").unwrap();
            info!("I have a HaMS");
            let mut shutdown_closure = || {
                info!("Shutdown request closure triggered");
                error!("GOT 2 HERE");

                my_running.store(false, Ordering::Relaxed);
                info!("Shutdown closure completed");
            };
            let (state, callback) = unsafe { ffi_helpers::split_closure(&mut shutdown_closure) };
            hams.register_shutdown(state, callback).ok();
            hams.start().expect("HaMS started successfully");

            sample_service
                .start()
                .expect("Service started successfully");

            let my_alive = AliveCheckKicked::new("apple", Duration::from_secs(100)).unwrap();
            hams.add_alive(&my_alive).ok();
            while my_running.load(Ordering::Relaxed) {
                sleep(Duration::from_millis(1000));
                my_alive.kick();
            }
            hams.remove_alive(&my_alive).ok();

            sample_service.stop().ok();

            hams.stop().expect("HaMS stopped successfully");

            info!("HaMS will be released... by Drop");
        }
        None => {}
    }
}

#[cfg(test)]
mod tests {
    use libc::c_void;

    #[test]
    fn closure_lib_usage() {
        let mut my_object = false;

        let mut callback_closure = || {
            println!("Setting my_object to true");
            my_object = true;
        };

        type Callback = unsafe extern "C" fn(*mut c_void);

        unsafe fn some_c_function(user_data: *mut c_void, cb: Callback) {
            println!("About to trigger callback");
            cb(user_data);
            println!("Completed trigger callback");
        }

        unsafe {
            let (state, callback) = ffi_helpers::split_closure(&mut callback_closure);

            some_c_function(state, callback);
        }

        println!("done with cbs");
    }

    #[test]
    fn closure_reference_test() {
        let mut total = 0;

        // let's define a closure which will update a total and return its new value
        let mut some_closure = |n: usize| {
            total += n;
            total
        };

        type Callback = unsafe extern "C" fn(*mut c_void, usize) -> usize;

        unsafe fn some_c_function(max_value: usize, cb: Callback, user_data: *mut c_void) {
            for i in 0..max_value {
                let got = cb(user_data, i);
                println!("iteration: {}, total: {}", i, got);
            }
        }

        unsafe {
            // split the closure into its state section and the code section
            let (state, callback) = ffi_helpers::split_closure(&mut some_closure);

            // then pass it to the C function
            some_c_function(42, callback, state);
        }

        assert_eq!(total, 861);
    }

    #[test]
    fn closure_test() {
        /// Create a C ABI function
        unsafe extern "C" fn trampoline<F>(user_function: *mut c_void)
        where
            F: FnMut(),
        {
            let user_function = &mut *(user_function as *mut F);
            user_function();
        }

        pub type AddCallback = unsafe extern "C" fn(*mut c_void);
        pub fn get_trampoline<F>(_closure: &F) -> AddCallback
        where
            F: FnMut(),
        {
            trampoline::<F>
        }

        extern "C" {
            pub fn better_add_two_numbers(cb: AddCallback, user_data: *mut c_void);
        }

        let myname = String::from("Hello i am ben");
        let closure = || {
            println!("Callback called with data: {}", myname);
        };
        let trampoline = get_trampoline(&closure);

        type CFunction = extern "C" fn();

        closure();
        closure();
    }
}
