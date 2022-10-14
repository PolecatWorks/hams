use log::{info};
use ffi_log2::{logger_init, LogParam};
use std::{process};


const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");


pub fn add(left: usize, right: usize) -> usize {
    left + right
}



/// Initialise the FFI based logging for this crate
#[no_mangle]
pub extern "C" fn hams_logger_init(param: LogParam) {
    todo!("Check the validity of param");
    logger_init(param);
    info!(
        "Logging registered for {}:{} (PID: {}) using FFI",
        NAME,
        VERSION,
        process::id()
    );
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
