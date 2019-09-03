use futures::io::AsyncRead;
//use std::pin::Unpin;

pub trait AsyncFile = AsyncRead + Unpin;
