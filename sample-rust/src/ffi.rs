use ffi_log2::LogParam;
use libc::c_void;

/// Opaque object representing HaMS objects.
/// Low level API access to the CAPI
#[repr(C)]
pub struct Hams {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Opaque object representing AliveChecked objects
#[repr(C)]
pub struct AliveCheckKicked {
    _data: libc::c_void,
    // _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[link(name = "hams", kind = "dylib")]
extern "C" {
    /// Configure logging for HaMS
    pub fn hams_logger_init(param: LogParam) -> i32;
    /// Init a HaMS and return the reference to the HaMS object
    pub fn hams_init(name: *const libc::c_char) -> *mut Hams;
    /// Free HaMS
    pub fn hams_free(hams: *mut Hams) -> i32;
    /// Start HaMS
    pub fn hams_start(ptr: *mut Hams) -> i32;
    /// Stop HaMS
    pub fn hams_stop(ptr: *mut Hams) -> i32;
    /// Create kicked health check
    pub fn kicked_create(
        name: *const libc::c_char,
        duration_millis: libc::c_ulong,
    ) -> *mut AliveCheckKicked;
    /// Free AliveChecked
    pub fn kicked_free(kicked: *mut AliveCheckKicked) -> i32;
    /// Kick AliveChecked
    pub fn kicked_kick(kicked: *mut AliveCheckKicked) -> i32;
    /// Add KickAliveCheck to hams
    pub fn hams_add_alive(hams: *mut Hams, kicked: *mut AliveCheckKicked) -> i32;
    pub fn hams_remove_alive(hams: *mut Hams, kicked: *mut AliveCheckKicked) -> i32;
    pub fn hams_register_shutdown(
        hams: *mut Hams,
        user_data: *mut c_void,
        cb: unsafe extern "C" fn(*mut c_void),
    ) -> i32;
}
