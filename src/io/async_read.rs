use super::ReadBuf;
use std::io;
use std::ops::DerefMut;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Reads bytes from the source
pub trait AsyncRead {
    /// attempt to read into the `buf`
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>>;
}

macro_rules! deref_async_read {
    () => {
        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            Pin::new(&mut **self).poll_read(cx, buf)
        }
    };
}

impl<T: ?Sized + AsyncRead + Unpin> AsyncRead for Box<T> {
    deref_async_read!();
}

impl<T: ?Sized + AsyncRead + Unpin> AsyncRead for &mut T {
    deref_async_read!();
}

impl<P> AsyncRead for Pin<P>
where
    P: DerefMut + Unpin,
    P::Target: AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.get_mut().as_mut().poll_read(cx, buf)
    }
}

impl AsyncRead for &[u8] {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let amt = std::cmp::min(self.len(), buf.remaining());
        let (a, b) = self.split_at(amt);
        buf.put_slice(a);
        *self = b;
        Poll::Ready(Ok(()))
    }
}

impl<T: AsRef<[u8]> + Unpin> AsyncRead for io::Cursor<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let pos = self.position();
        let slice: &[u8] = (*self).get_ref().as_ref();

        // The position could technically be out of bounds, so don't panic...
        if pos > slice.len() as u64 {
            return Poll::Ready(Ok(()));
        }

        let start = pos as usize;
        let amt = std::cmp::min(slice.len() - start, buf.remaining());
        // Add won't overflow because of pos check above.
        let end = start + amt;
        buf.put_slice(&slice[start..end]);
        self.set_position(end as u64);

        Poll::Ready(Ok(()))
    }
}