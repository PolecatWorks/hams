use ffi_log2::LogParam;

#[link(name = "hams", kind = "dylib")]
extern "C" {
    // CAPI methods from shared library

    /// Configure logging for UService
    fn hams_logger_init(param: LogParam);
}

// Initialise logging
pub fn hams_logger_init_ffi(param: LogParam) {
    unsafe { hams_logger_init(param) };
}
