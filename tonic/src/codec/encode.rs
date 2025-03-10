use crate::{body::BytesBuf, Code, Status};
use bytes::{BufMut, BytesMut, IntoBuf};
use futures_core::{Stream, TryStream};
use futures_util::{ready, StreamExt, TryStreamExt};
use http::HeaderMap;
use http_body::Body;
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_codec::Encoder;

const BUFFER_SIZE: usize = 8 * 1024;

pub(crate) fn encode_server<T, U>(
    encoder: T,
    source: U,
) -> EncodeBody<impl Stream<Item = Result<BytesBuf, Status>>>
where
    T: Encoder<Error = Status>,
    U: Stream<Item = Result<T::Item, Status>>,
{
    let stream = encode(encoder, source).into_stream();
    EncodeBody::new_server(stream)
}

pub(crate) fn encode_client<T, U>(
    encoder: T,
    source: U,
) -> EncodeBody<impl Stream<Item = Result<BytesBuf, Status>>>
where
    T: Encoder<Error = Status>,
    U: Stream<Item = Result<T::Item, Status>>,
{
    let stream = encode(encoder, source).into_stream();
    EncodeBody::new_client(stream)
}

fn encode<T, U>(mut encoder: T, source: U) -> impl TryStream<Ok = BytesBuf, Error = Status>
where
    T: Encoder<Error = Status>,
    U: Stream<Item = Result<T::Item, Status>>,
{
    async_stream::stream! {
        let mut buf = BytesMut::with_capacity(BUFFER_SIZE);
        futures_util::pin_mut!(source);

        loop {
            match source.next().await {
                Some(Ok(item)) => {
                    buf.reserve(5);
                    unsafe {
                        buf.advance_mut(5);
                    }
                    encoder.encode(item, &mut buf).map_err(drop).unwrap();

                    // now that we know length, we can write the header
                    let len = buf.len() - 5;
                    assert!(len <= std::u32::MAX as usize);
                    {
                        let mut cursor = std::io::Cursor::new(&mut buf[..5]);
                        cursor.put_u8(0); // byte must be 0, reserve doesn't auto-zero
                        cursor.put_u32_be(len as u32);
                    }

                    yield Ok(buf.split_to(len + 5).freeze().into_buf());
                },
                Some(Err(status)) => yield Err(status),
                None => break,
            }
        }
    }
}

#[derive(Debug)]
enum Role {
    Client,
    Server,
}

#[pin_project]
#[derive(Debug)]
pub(crate) struct EncodeBody<S> {
    #[pin]
    inner: S,
    error: Option<Status>,
    role: Role,
}

impl<S> EncodeBody<S>
where
    S: Stream<Item = Result<crate::body::BytesBuf, Status>>,
{
    pub(crate) fn new_client(inner: S) -> Self {
        Self {
            inner,
            error: None,
            role: Role::Client,
        }
    }

    pub(crate) fn new_server(inner: S) -> Self {
        Self {
            inner,
            error: None,
            role: Role::Server,
        }
    }
}

impl<S> Body for EncodeBody<S>
where
    S: Stream<Item = Result<crate::body::BytesBuf, Status>>,
{
    type Data = BytesBuf;
    type Error = Status;

    fn is_end_stream(&self) -> bool {
        false
    }

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let mut self_proj = self.project();
        match ready!(self_proj.inner.try_poll_next_unpin(cx)) {
            Some(Ok(d)) => Some(Ok(d)).into(),
            Some(Err(status)) => match self_proj.role {
                Role::Client => Some(Err(status)).into(),
                Role::Server => {
                    *self_proj.error = Some(status);
                    None.into()
                }
            },
            None => None.into(),
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Status>> {
        match self.role {
            Role::Client => Poll::Ready(Ok(None)),
            Role::Server => {
                let self_proj = self.project();
                let status = if let Some(status) = self_proj.error.take() {
                    status
                } else {
                    Status::new(Code::Ok, "")
                };

                Poll::Ready(Ok(Some(status.to_header_map()?)))
            }
        }
    }
}
