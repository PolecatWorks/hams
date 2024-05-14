use libc::c_void;
use std::ffi::CString;

#[no_mangle]
pub extern "C" fn prometheus_response(ptr: *const c_void) -> *const libc::c_char {
    println!("Callback from C2");

    let state = unsafe { &*(ptr as *const String) };

    let prometheus = format!("test {state}");
    let c_str_prometheus = std::ffi::CString::new(prometheus).unwrap();

    c_str_prometheus.into_raw()
}

#[no_mangle]
pub extern "C" fn prometheus_response_free(ptr: *mut libc::c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    };
}
