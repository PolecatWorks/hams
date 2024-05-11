use ffi_log2::LogParam;

/// Opaque object representing HaMS objects.
/// Low level API access to the CAPI based on Rustonomican book (https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs)
/// This is a zero-sized type, which is a type that has no values. This type is used to represent the opaque object.
///
#[repr(C)]
pub struct Hams {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Opaque object representing HaMS Manual Probe objects.
/// Low level API access to the CAPI
#[repr(C)]
pub struct ManualProbe {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
pub struct KickProbe {
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
    pub fn hams_alive_insert(hams: *mut Hams, probe: *mut Probe) -> i32;
    pub fn hams_alive_remove(hams: *mut Hams, probe: *mut Probe) -> i32;
    pub fn hams_ready_insert(hams: *mut Hams, probe: *mut Probe) -> i32;
    pub fn hams_ready_remove(hams: *mut Hams, probe: *mut Probe) -> i32;
    pub fn hams_register_prometheus(
        hams: *mut Hams,
        my_cb: extern "C" fn() -> *const libc::c_char,
        my_cb_free: extern "C" fn(*mut libc::c_char),
    ) -> i32;

    pub fn hello_world();
    pub fn hello_callback(my_cb: extern "C" fn());
    pub fn hello_callback2(
        my_cb: extern "C" fn() -> *const libc::c_char,
        my_cb_free: extern "C" fn(*mut libc::c_char),
    );
    pub fn hams_version() -> *const libc::c_char;

    pub fn probe_manual_new(name: *const libc::c_char, valid: bool) -> *mut ManualProbe;
    pub fn probe_manual_free(probe: *mut ManualProbe) -> i32;
    pub fn probe_manual_boxed(probe: *mut ManualProbe) -> *mut Probe;
    pub fn probe_manual_enable(probe: *mut ManualProbe, valid: bool) -> i32;
    pub fn probe_manual_disable(probe: *mut ManualProbe) -> i32;
    pub fn probe_manual_toggle(probe: *mut ManualProbe) -> i32;
    pub fn probe_manual_check(probe: *mut ManualProbe) -> i32;

    pub fn probe_kick_new(name: *const libc::c_char, margin: u64) -> *mut KickProbe;
    pub fn probe_kick_free(probe: *mut KickProbe) -> i32;
    pub fn probe_kick_boxed(probe: *mut KickProbe) -> *mut Probe;
    pub fn probe_kick_kick(probe: *mut KickProbe) -> i32;

    pub fn probe_free(probe: *mut Probe) -> i32;
}
