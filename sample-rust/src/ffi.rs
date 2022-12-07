use ffi_log2::LogParam;
/// Low level API access to the CAPI

/// Opaque object representing HaMS objects
#[repr(C)]
pub struct Hams {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[link(name = "hams", kind = "dylib")]
extern "C" {
    /// Configure logging for UService
    pub fn hams_logger_init(param: LogParam) -> i32;
    /// Init a HaMS and return the reference to the UService object
    pub fn hams_init(name: *const libc::c_char) -> *mut Hams;
    /// Free an UService
    pub fn hams_free(hams: *mut Hams) -> i32;
    /// Start HaMS
    pub fn hams_start(ptr: *mut Hams) -> i32;
    /// Start HaMS
    pub fn hams_stop(ptr: *mut Hams) -> i32;
}
