use std::{
    any::TypeId,
    io::{Error, Write},
};

// https://adventures.michaelfbryan.com/posts/ffi-safe-polymorphism-in-rust/

pub struct FileHandle {
    pub(crate) type_id: TypeId,
    pub(crate) destroy: unsafe fn(*mut FileHandle),
    pub(crate) write: unsafe fn(*mut FileHandle, &[u8]) -> Result<usize, Error>,
    pub(crate) flush: unsafe fn(*mut FileHandle) -> Result<(), Error>,
}

pub(crate) struct Repr<W> {
    // SAFETY: The FileHandle must be the first field so we can cast between
    // *mut Repr<W> and *mut FileHandle
    pub(crate) base: FileHandle,
    pub(crate) writer: W,
}

impl FileHandle {
    pub fn for_writer<W>(writer: W) -> *mut FileHandle
    where
        W: Write + Send + Sync + 'static,
    {
        let repr = Repr {
            base: FileHandle::vtable::<W>(),
            writer,
        };

        let boxed = Box::into_raw(Box::new(repr));

        boxed as *mut _
    }

    fn vtable<W: Write + 'static>() -> FileHandle {
        let type_id = TypeId::of::<W>();

        FileHandle {
            type_id,
            destroy: destroy::<W>,
            write: write::<W>,
            flush: flush::<W>,
        }
    }
}

unsafe fn destroy<W>(handle: *mut FileHandle) {
    let repr = handle as *mut Repr<W>;
    let _ = Box::from_raw(repr);
}

unsafe fn write<W: Write>(handle: *mut FileHandle, data: &[u8]) -> Result<usize, Error> {
    let repr = &mut *(handle as *mut Repr<W>);
    repr.writer.write(data)
}

unsafe fn flush<W: Write>(handle: *mut FileHandle) -> Result<(), Error> {
    let repr = &mut *(handle as *mut Repr<W>);
    repr.writer.flush()
}
