#![warn(missing_docs)]

//! Work out what needs to be configured inside the DLL to enable the log forwarding.
//! Create a ffi function that enables the logging in the DLL to be configured (safely).
//! Createa function in the main that allows creating of the object that is used to configure the DLL funciton.

use log::{Level, LevelFilter, Log, Metadata, Record, RecordBuilder};
use std::mem::ManuallyDrop;

/// FFI-safe borrowed Rust &str. Can represents `Option<&str>` by setting ptr to null.
#[repr(C)]
pub struct RustStr {
    /// pointer to c-FFI safe string chars
    pub ptr: *const u8,
    /// length of rust string for C
    pub len: usize,
}
/** Convert to RustStr from str */
impl<'a> From<&'a str> for RustStr {
    fn from(s: &'a str) -> Self {
        let bytes = s.as_bytes();
        Self {
            ptr: bytes.as_ptr(),
            len: bytes.len(),
        }
    }
}
impl<'a> From<Option<&'a str>> for RustStr {
    fn from(o: Option<&'a str>) -> Self {
        match o {
            None => Self {
                ptr: std::ptr::null(),
                len: 0,
            },
            Some(s) => Self::from(s),
        }
    }
}

impl RustStr {
    /// # Safety
    ///
    /// Convert RustStr to str. Care must be taken to check and validate across FFI boundaries
    pub unsafe fn to_str<'a>(&self) -> &'a str {
        let bytes = std::slice::from_raw_parts(self.ptr, self.len);
        std::str::from_utf8_unchecked(bytes)
    }
    /// # Safety
    ///
    /// Convert to Optional RustStr. Use null to reference as None
    pub unsafe fn to_opt_str<'a>(&self) -> Option<&'a str> {
        if self.ptr.is_null() {
            None
        } else {
            Some(self.to_str())
        }
    }
}

/// FFI-safe Metadata
#[repr(C)]
pub struct ExternCMetadata {
    /// Log verbosity
    pub level: Level,
    /// Log target
    pub target: RustStr,
}

impl ExternCMetadata {
    /// # Safety
    ///
    /// convert to metadata for use in log functions. Convert from FFI to Metadata
    pub unsafe fn as_metadata(&self) -> Metadata {
        let level = self.level;
        let target = self.target.to_str();
        Metadata::builder().level(level).target(target).build()
    }
}

impl<'a> From<&Metadata<'a>> for ExternCMetadata {
    fn from(m: &Metadata<'a>) -> Self {
        Self {
            level: m.level(),
            target: m.target().into(),
        }
    }
}
/// FFI-safe owned Rust String.
#[repr(C)]
pub struct RustString {
    /// pointer to characters
    pub ptr: *mut u8,
    /// capacity
    pub cap: usize,
    /// length
    pub len: usize,
}
impl RustString {
    /// # Safety
    ///
    /// covert to String from FFI version
    pub unsafe fn to_str<'a>(&self) -> &'a str {
        RustStr {
            ptr: self.ptr as _,
            len: self.len,
        }
        .to_str()
    }
    /// # Safety
    ///
    /// Convert to Optional String from FFI version
    pub unsafe fn into_string(self) -> String {
        String::from_raw_parts(self.ptr, self.len, self.cap)
    }
}

impl From<String> for RustString {
    fn from(s: String) -> Self {
        let mut me = ManuallyDrop::new(s);
        let (ptr, len, cap) = (me.as_mut_ptr(), me.len(), me.capacity());
        Self { ptr, len, cap }
    }
}
impl Drop for RustString {
    fn drop(&mut self) {
        unsafe {
            String::from_raw_parts(self.ptr, self.len, self.cap);
        }
    }
}

/// FFI-safe Record
#[repr(C)]
pub struct ExternCRecord {
    /// Extern C Metadata
    pub metadata: ExternCMetadata,
    /// fmt::Arguments<'a> are not FFI-safe, so we have no option but to format them beforehand.
    pub message: RustString,
    /// module path RustStr
    pub module_path: RustStr, // None points to null
    /// file name RustStr
    pub file: RustStr, // None points to null
    /// Line number of log entry
    pub line: i64, // None maps to -1, everything else should fit in u32.
}

impl<'a> From<&Record<'a>> for ExternCRecord {
    fn from(r: &Record<'a>) -> Self {
        let msg = r.args().to_string();
        Self {
            metadata: ExternCMetadata::from(r.metadata()),
            message: RustString::from(msg),
            module_path: RustStr::from(r.module_path()),
            file: RustStr::from(r.file()),
            line: r.line().map(|u| u as i64).unwrap_or(-1_i64),
        }
    }
}

impl ExternCRecord {
    /// # Safety
    ///
    /// Return the record build for the externCRecord
    pub unsafe fn as_record_builder(&self) -> RecordBuilder {
        let mut builder = Record::builder();
        builder
            // .args(self.message.to_str())
            // .args(format_args!("{}", "self.message.to_str()"))
            .metadata(self.metadata.as_metadata())
            .module_path(self.module_path.to_opt_str())
            .file(self.file.to_opt_str())
            .line(if self.line == -1 {
                None
            } else {
                Some(self.line as _)
            });
        builder
        // Return a Record here instead of a RecordBuilder
    }
}

/** LogParam is LogParam is a struct that transports the necessary objects to enable the configuration of the DLL logger.
 * This structure must be FFI-safe. It must be constructured into FFI safe structures from the original structures on teh sending side and reconstruced into the log structures on teh consume size of log functions.
 */
#[repr(C)]
pub struct LogParam {
    /// function to check if logging is enabled
    pub enabled: extern "C" fn(ExternCMetadata) -> bool,
    /// Write a log record
    pub log: extern "C" fn(&ExternCRecord),
    /// flush the logs
    pub flush: extern "C" fn(),
    /// value for the log level
    pub level: LevelFilter,
}

struct DLog;

static mut PARAM: Option<LogParam> = None;

/** init the DLL logging by passing in the references to the implemntation of the logging
 */
pub fn logger_init(param: LogParam) {
    let level = param.level;
    unsafe {
        if PARAM.is_some() {
            eprint!("log should only init once");
            return;
        }
        PARAM.replace(param);
    }
    if let Err(err) = log::set_logger(&LOGGER).map(|_| log::set_max_level(level)) {
        eprint!("set logger failed:{}", err);
    }
}

fn param() -> &'static LogParam {
    unsafe { PARAM.as_ref() }.unwrap()
}

/** Log implementation is the definition of the Interfaces used by the log library
 * This struct maps the Logging library API to the FFI provided objects for actual logging.
 */
impl Log for DLog {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let metadata = ExternCMetadata::from(metadata);
        (param().enabled)(metadata)
    }

    fn log(&self, record: &Record) {
        let record = ExternCRecord::from(record);
        (param().log)(&record)
    }

    fn flush(&self) {
        (param().flush)()
    }
}

static LOGGER: DLog = DLog;

extern "C" fn enabled(meta: ExternCMetadata) -> bool {
    let metadata = unsafe { meta.as_metadata() };
    log::logger().enabled(&metadata)
}

extern "C" fn log(ext_record: &ExternCRecord) {
    let mut record_builder = unsafe { ext_record.as_record_builder() };

    match format_args!("{}", unsafe { ext_record.message.to_str() }) {
        args => {
            let record = record_builder.args(args).build();
            log::logger().log(&record);
        }
    }
}

extern "C" fn flush() {
    log::logger().flush()
}

/** extract the log parameters from the existing log implementation so that they can be shared to the DLL
 */
pub fn log_param() -> LogParam {
    LogParam {
        enabled,
        log,
        flush,
        level: log::max_level(),
    }
}
