macro_rules! try_stream {
    ($e:expr) => {
        match $e {
            Err(err) => return Err(From::from(err)),
            Ok(::futures::Async::NotReady) => return Ok(::futures::Async::NotReady),
            Ok(::futures::Async::Ready(None)) => return Ok(::futures::Async::Ready(None)),
            Ok(::futures::Async::Ready(Some(value))) => value,
        }
    };
}

/// like `try_stream` but returns `None` if `NotReady`
macro_rules! try_maybe_stream {
    ($e:expr) => {
        match $e {
            Err(err) => return Err(From::from(err)),
            Ok(::futures::Async::NotReady) => None,
            Ok(::futures::Async::Ready(None)) => return Ok(::futures::Async::Ready(None)),
            Ok(::futures::Async::Ready(Some(value))) => Some(value),
        }
    };
}
