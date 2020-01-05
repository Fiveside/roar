use futures::prelude::*;

pub trait AsyncFile = AsyncRead + AsyncSeek;
