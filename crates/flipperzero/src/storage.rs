use core::ffi::{c_char, CStr};
use core::fmt;

use flipperzero_sys as sys;
use flipperzero_sys::furi::UnsafeRecord;

const RECORD_STORAGE: *const c_char = sys::c_string!("storage");

/// Stream and file system related error type
#[derive(Debug, Clone, Copy)]
pub enum Error {
    Ok,
    NotReady,
    Exists,
    NotExists,
    InvalidParameter,
    Denied,
    InvalidName,
    Internal,
    NotImplemented,
    AlreadyOpen,
}

impl Error {
    pub fn to_sys(&self) -> sys::FS_Error {
        match self {
            Self::Ok => sys::FS_Error_FSE_OK,
            Self::NotReady => sys::FS_Error_FSE_NOT_READY,
            Self::Exists => sys::FS_Error_FSE_EXIST,
            Self::NotExists => sys::FS_Error_FSE_NOT_EXIST,
            Self::InvalidParameter => sys::FS_Error_FSE_INVALID_PARAMETER,
            Self::Denied => sys::FS_Error_FSE_DENIED,
            Self::InvalidName => sys::FS_Error_FSE_INVALID_NAME,
            Self::Internal => sys::FS_Error_FSE_INTERNAL,
            Self::NotImplemented => sys::FS_Error_FSE_NOT_IMPLEMENTED,
            Self::AlreadyOpen => sys::FS_Error_FSE_ALREADY_OPEN,
        }
    }

    pub fn from_sys(err: sys::FS_Error) -> Self {
        match err {
            sys::FS_Error_FSE_OK => Self::Ok,
            sys::FS_Error_FSE_NOT_READY => Self::NotReady,
            sys::FS_Error_FSE_EXIST => Self::Exists,
            sys::FS_Error_FSE_NOT_EXIST => Self::NotExists,
            sys::FS_Error_FSE_INVALID_PARAMETER => Self::InvalidParameter,
            sys::FS_Error_FSE_DENIED => Self::Denied,
            sys::FS_Error_FSE_INVALID_NAME => Self::InvalidName,
            sys::FS_Error_FSE_INTERNAL => Self::Internal,
            sys::FS_Error_FSE_NOT_IMPLEMENTED => Self::NotImplemented,
            sys::FS_Error_FSE_ALREADY_OPEN => Self::AlreadyOpen,
            _ => unimplemented!(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = unsafe { CStr::from_ptr(sys::filesystem_api_error_get_desc(self.to_sys())) };
        write!(f, "{}", msg.to_bytes().escape_ascii())
    }
}

/// Trait comparable to `std::Read` for the Flipper stream API
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error>;
}

/// Enumeration of possible methods to seek within an I/O object.
///
/// It is used by the Seek trait.
pub enum SeekFrom {
    Start(i32),
    End(i32),
    Current(i32),
}

/// Trait comparable to `std::Seek` for the Flipper stream API
pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<usize, Error>;

    fn rewind(&mut self) -> Result<(), Error> {
        self.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    fn stream_len(&mut self) -> Result<usize, Error> {
        let old_pos = self.stream_position()?;
        let len = self.seek(SeekFrom::End(0))?;

        // Avoid seeking a third time when we were already at the end of the
        // stream. The branch is usually way cheaper than a seek operation.
        if old_pos != len {
            self.seek(SeekFrom::Start(old_pos as i32))?;
        }

        Ok(len)
    }

    fn stream_position(&mut self) -> Result<usize, Error> {
        self.seek(SeekFrom::Current(0))
    }
}

/// Trait comparable to `std::Write` for the Flipper stream API
pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error>;
    fn flush(&mut self) -> Result<(), Error>;

    fn write_all(&mut self, mut buf: &[u8]) -> Result<(), Error> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    // TODO
                }
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct OpenOptions {
    access_mode: u8,
    open_mode: u8,
}

impl OpenOptions {
    pub fn new() -> Self {
        Self::default()
    }

    fn from_parts(access_mode: u8, open_mode: u8) -> Self {
        OpenOptions {
            access_mode,
            open_mode,
        }
    }

    /// Read access
    pub fn read(self, set: bool) -> Self {
        OpenOptions::from_parts(
            if set {
                self.access_mode | sys::FS_AccessMode_FSAM_READ
            } else {
                self.access_mode & !sys::FS_AccessMode_FSAM_READ
            },
            self.open_mode,
        )
    }

    /// Write access
    pub fn write(self, set: bool) -> Self {
        OpenOptions::from_parts(
            if set {
                self.access_mode | sys::FS_AccessMode_FSAM_WRITE
            } else {
                self.access_mode & !sys::FS_AccessMode_FSAM_WRITE
            },
            self.open_mode,
        )
    }

    /// Open file, fail if file doesn't exist
    pub fn open_existing(self, set: bool) -> Self {
        OpenOptions::from_parts(
            self.access_mode,
            if set {
                self.open_mode | sys::FS_OpenMode_FSOM_OPEN_EXISTING
            } else {
                self.open_mode & !sys::FS_OpenMode_FSOM_OPEN_EXISTING
            },
        )
    }

    /// Open file. Create new file if not exist
    pub fn open_always(self, set: bool) -> Self {
        OpenOptions::from_parts(
            self.access_mode,
            if set {
                self.open_mode | sys::FS_OpenMode_FSOM_OPEN_ALWAYS
            } else {
                self.open_mode & !sys::FS_OpenMode_FSOM_OPEN_ALWAYS
            },
        )
    }

    /// Open file. Create new file if not exist. Set R/W pointer to EOF
    pub fn open_append(self, set: bool) -> Self {
        OpenOptions::from_parts(
            self.access_mode,
            if set {
                self.open_mode | sys::FS_OpenMode_FSOM_OPEN_APPEND
            } else {
                self.open_mode & !sys::FS_OpenMode_FSOM_OPEN_APPEND
            },
        )
    }

    /// Creates a new file. Fails if the file is exist
    pub fn create_new(self, set: bool) -> Self {
        OpenOptions::from_parts(
            self.access_mode,
            if set {
                self.open_mode | sys::FS_OpenMode_FSOM_CREATE_NEW
            } else {
                self.open_mode & !sys::FS_OpenMode_FSOM_CREATE_NEW
            },
        )
    }

    /// Creates a new file. If file exist, truncate to zero size
    pub fn create_always(self, set: bool) -> Self {
        OpenOptions::from_parts(
            self.access_mode,
            if set {
                self.open_mode | sys::FS_OpenMode_FSOM_CREATE_ALWAYS
            } else {
                self.open_mode & !sys::FS_OpenMode_FSOM_CREATE_ALWAYS
            },
        )
    }

    pub fn open(self, path: &CStr) -> Result<BufferedFile, Error> {
        let f = BufferedFile::new();
        if unsafe {
            sys::buffered_file_stream_open(
                f.0,
                path.as_ptr() as *const i8,
                self.access_mode,
                self.open_mode,
            )
        } {
            Ok(f)
        } else {
            // Per docs, "you need to close the file even if the open operation
            // failed," but this is handled by `Drop`.
            Err(Error::from_sys(unsafe {
                sys::buffered_file_stream_get_error(f.0)
            }))
        }
    }
}

/// File stream with buffered read operations.
pub struct BufferedFile(*mut sys::Stream);

impl BufferedFile {
    pub fn new() -> Self {
        unsafe {
            BufferedFile(sys::buffered_file_stream_alloc(
                UnsafeRecord::open(RECORD_STORAGE).as_ptr(),
            ))
        }
    }
}

impl Drop for BufferedFile {
    fn drop(&mut self) {
        unsafe {
            // `buffered_file_stream_close` calls `buffered_file_stream_sync`
            // internally, so it's not necesssary to call it here.
            sys::buffered_file_stream_close(self.0);
        }
    }
}

impl Read for BufferedFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        Ok(unsafe { sys::stream_read(self.0, buf.as_mut_ptr(), buf.len()) })
    }
}

impl Seek for BufferedFile {
    fn seek(&mut self, pos: SeekFrom) -> Result<usize, Error> {
        let (offset_type, offset) = match pos {
            SeekFrom::Start(n) => (sys::StreamOffset_StreamOffsetFromStart, n),
            SeekFrom::End(n) => (sys::StreamOffset_StreamOffsetFromEnd, n),
            SeekFrom::Current(n) => (sys::StreamOffset_StreamOffsetFromCurrent, n),
        };
        unsafe {
            if sys::stream_seek(self.0, offset, offset_type) {
                Ok(sys::stream_tell(self.0))
            } else {
                Err(Error::from_sys(sys::buffered_file_stream_get_error(self.0)))
            }
        }
    }

    fn rewind(&mut self) -> Result<(), Error> {
        unsafe { sys::stream_rewind(self.0) };
        Ok(())
    }

    fn stream_len(&mut self) -> Result<usize, Error> {
        Ok(unsafe { sys::stream_size(self.0) })
    }

    fn stream_position(&mut self) -> Result<usize, Error> {
        Ok(unsafe { sys::stream_tell(self.0) })
    }
}

impl Write for BufferedFile {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        if unsafe { sys::stream_insert(self.0, buf.as_ptr(), buf.len()) } {
            Ok(buf.len())
        } else {
            Err(Error::from_sys(unsafe {
                sys::buffered_file_stream_get_error(self.0)
            }))
        }
    }

    fn flush(&mut self) -> Result<(), Error> {
        if unsafe { sys::buffered_file_stream_sync(self.0) } {
            Ok(())
        } else {
            Err(Error::from_sys(unsafe {
                sys::buffered_file_stream_get_error(self.0)
            }))
        }
    }
}
