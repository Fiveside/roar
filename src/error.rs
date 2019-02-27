use failure::{Backtrace, Context, Fail};
use std::fmt;

// Type alias for handling errors through this crate.
pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    ctx: Context<ErrorKind>,
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.ctx.get_context()
    }

    pub fn buffer_too_small(required: usize) -> Error {
        Error::from(ErrorKind::BufferTooSmall(required))
    }

    pub fn io(wrapped: ::std::io::Error) -> Error {
        Error::from(ErrorKind::Io(wrapped.to_string()))
    }

    pub fn bad_block(reason: String) -> Error {
        Error::from(ErrorKind::BadBlock(reason))
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.ctx.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.ctx.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.ctx.fmt(f)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ErrorKind {
    // Buffer size too small, contains required buffer size.
    BufferTooSmall(usize),

    // Wrapped io error
    Io(String),

    // Invalid block (corrupt archive?)
    BadBlock(String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::BufferTooSmall(ref size) => write!(
                f,
                "Buffer too small, require {} bytes before parsing will succeed",
                size
            ),
            ErrorKind::Io(ref msg) => write!(f, "I/O error: {}", msg),
            ErrorKind::BadBlock(ref msg) => write!(
                f,
                "Block Decoding error: {} (perhaps the archive is corrupt)",
                msg
            ),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error::from(Context::new(kind))
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(ctx: Context<ErrorKind>) -> Error {
        Error { ctx }
    }
}
