use ffi_log2::LogParam;

/// Opaque object representing HaMS objects.
/// Low level API access to the CAPI
#[repr(C)]
pub struct Hams {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Opaque object representing HaMS Probe objects.
/// Low level API access to the CAPI
#[repr(C)]
pub struct Probe {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[link(name = "hams", kind = "dylib")]
extern "C" {
    /// Configure logging for HaMS
    pub fn hams_logger_init(param: LogParam) -> i32;

    pub fn hams_new(name: *const libc::c_char) -> *mut Hams;
    pub fn hams_free(hams: *mut Hams) -> i32;
    pub fn hams_start(hams: *mut Hams) -> i32;
    pub fn hams_stop(hams: *mut Hams) -> i32;

    pub fn hello_world();
    pub fn hams_version() -> *const libc::c_char;
    pub fn probe_manual_new(name: *const libc::c_char, valid: bool) -> *mut Probe;
    pub fn probe_manual_free(probe: *mut Probe) -> i32;
    pub fn probe_free(probe: *mut Probe) -> i32;
}
