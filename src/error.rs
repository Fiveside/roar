use thiserror::Error;

#[derive(Error, Debug)]
pub enum RoarError {
    #[error("Unknown")]
    Unknown,

    #[error("IO Error")]
    IO(#[from] std::io::Error),

    #[error("Unknown block type: {0}")]
    UnknownBlockType(u8),
}

pub type Result<T> = ::std::result::Result<T, RoarError>;

// use failure::{Backtrace, Context, Fail};
// use std::fmt;

// // Type alias for handling errors through this crate.
// pub type Result<T> = ::std::result::Result<T, Error>;

// #[derive(Debug)]
// pub struct Error {
//     ctx: Context<ErrorKind>,
// }

// impl Error {
//     pub fn kind(&self) -> &ErrorKind {
//         self.ctx.get_context()
//     }

//     pub fn buffer_too_small(required: usize) -> Error {
//         Error::from(ErrorKind::BufferTooSmall(required))
//     }

//     pub fn io(wrapped: ::std::io::Error) -> Error {
//         Error::from(ErrorKind::Io(wrapped.to_string()))
//     }

//     pub fn aio(wrapped: ::async_std::io::Error) -> Error {
//         Error::from(ErrorKind::Aio(wrapped.to_string()))
//     }

//     pub fn bad_block(reason: String) -> Error {
//         Error::from(ErrorKind::BadBlock(reason))
//     }
// }

// impl Fail for Error {
//     fn cause(&self) -> Option<&dyn Fail> {
//         self.ctx.cause()
//     }

//     fn backtrace(&self) -> Option<&Backtrace> {
//         self.ctx.backtrace()
//     }
// }

// impl fmt::Display for Error {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         self.ctx.fmt(f)
//     }
// }

// #[derive(Debug, Clone, Eq, PartialEq)]
// pub enum ErrorKind {
//     // Buffer size too small, contains required buffer size.
//     BufferTooSmall(usize),

//     // Wrapped io error
//     Io(String),

//     // Wrapped async io error
//     Aio(String),

//     // Invalid block (corrupt archive?)
//     BadBlock(String),
// }

// impl fmt::Display for ErrorKind {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             ErrorKind::BufferTooSmall(ref size) => write!(
//                 f,
//                 "Buffer too small, require {} bytes before parsing will succeed",
//                 size
//             ),
//             ErrorKind::Io(ref msg) => write!(f, "I/O error: {}", msg),
//             ErrorKind::Aio(ref msg) => write!(f, "AIO error: {}", msg),
//             ErrorKind::BadBlock(ref msg) => write!(
//                 f,
//                 "Block Decoding error: {} (perhaps the archive is corrupt)",
//                 msg
//             ),
//         }
//     }
// }

// impl From<ErrorKind> for Error {
//     fn from(kind: ErrorKind) -> Error {
//         Error::from(Context::new(kind))
//     }
// }

// impl From<Context<ErrorKind>> for Error {
//     fn from(ctx: Context<ErrorKind>) -> Error {
//         Error { ctx }
//     }
// }

// impl From<::std::io::Error> for Error {
//     fn from(e: ::std::io::Error) -> Error {
//         Error::io(e)
//     }
// }
