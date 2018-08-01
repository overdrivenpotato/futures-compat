//! futures 0.1.x compatibility.
use std::io;

use futures_v01x::{
    Async as Async01,
    Future as Future01,
    Poll as Poll01,
    Stream as Stream01,
    Sink as Sink01,
    StartSend as StartSend01,
    AsyncSink as AsyncSink01,
};
use futures_v01x::executor::{Notify, NotifyHandle, UnsafeNotify, with_notify};
use futures_v01x::future::{Executor as Executor01};

use futures_v02x::{Async as Async02, Future as Future02, Never, Poll as Poll02, Stream as Stream02};
use futures_v02x::executor::{Executor as Executor02, SpawnError};
use futures_v02x::task::{Context, Waker};
use futures_v02x::io::{AsyncRead as AsyncRead02, AsyncWrite as AsyncWrite02};
use futures_v02x::{Sink as Sink02};

use tokio_io::{AsyncRead as AsyncReadTk, AsyncWrite as AsyncWriteTk};

use super::futures_02::{BoxedExecutor02, Future02NeverAs01Unit};

/// Wrap a `Future` from v0.1 as a `Future` from v0.2.
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct Future01As02<F> {
    v01: F,
}

/// Wrap a `Stream` from v0.1 as a `Stream` from v0.2.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Stream01As02<S> {
    v01: S,
}

/// Wrap a `Sink` from v0.1 as a `Sink` from v0.2.
///
/// Internally, this buffers all `SinkItem` values until flushed or closed.
#[derive(Debug)]
pub struct Sink01As02<S> where S: Sink01 {
    v01: S,
    buf: Vec<S::SinkItem>,
}

/// Wrap an `Executor` from v0.1 as a `Executor` from v0.2.
#[derive(Clone, Debug)]
pub struct Executor01As02<E> {
    v01: E,
}

/// Wrap a IO from tokio-io as an `AsyncRead`/`AsyncWrite` from v0.2.
#[derive(Debug)]
pub struct TokioAsAsyncIo02<I> {
    v01: I,
}

/// A trait to convert any `Future` from v0.1 into a [`Future01As02`](Future01As02).
///
/// Implemented for all types that implement v0.1's `Future` automatically.
pub trait FutureInto02: Future01 {
    /// Converts this future into a `Future01As02`.
    fn into_02_compat(self) -> Future01As02<Self> where Self: Sized;
}
/// A trait to convert any `Stream` from v0.1 into a [`Stream01As02`](Stream01As02).
///
/// Implemented for all types that implement v0.1's `Stream` automatically.
pub trait StreamInto02: Stream01 {
    /// Converts this stream into a `Stream01As02`.
    fn into_02_compat(self) -> Stream01As02<Self> where Self: Sized;
}

/// A trait convert any `Sink` from v0.1 into a [`Sink01As02`](Sink01As02).
///
/// Implemented for all types that implement v0.1's `Sink` automatically.
pub trait SinkInto02: Sink01 {
    /// Converts this sink into a `Sink01As02`.
    fn sink_into_02_compat(self) -> Sink01As02<Self> where Self: Sized;
}

/// A trait to convert an `Executor` from v0.1 into an [`Executor01As02`](Executor01As02).
///
/// Implemented for generic v0.1 `Executor`s automatically.
pub trait ExecutorInto02: Executor01<
        Future02NeverAs01Unit<
            BoxedExecutor02,
            Box<Future02<Item=(), Error=Never> + Send>
        >
    > + Clone + Send + 'static {
    /// Converts this stream into a `Executor01As02`.
    fn into_02_compat(self) -> Executor01As02<Self> where Self: Sized;
}

/// A trait to convert any `AsyncRead`/`AsyncWrite` from tokio-io into a [`TokioAsAsyncIo02`](TokioAsAsyncIo02).
///
/// Implemented for all types that implement tokio-io's `AsyncRead`/`AsyncWrite` automatically.
pub trait TokioIntoAsyncIo02 {
    /// Converts this IO into an `TokioAsAsyncIo02`.
    fn into_v02_compat(self) -> TokioAsAsyncIo02<Self>
    where
        Self: AsyncReadTk + AsyncWriteTk + Sized;
}

impl<F> FutureInto02 for F
where
    F: Future01,
{
    fn into_02_compat(self) -> Future01As02<Self>
    where
        Self: Sized,
    {
        Future01As02 {
            v01: self,
        }
    }
}

impl<F> Future02 for Future01As02<F>
where
    F: Future01,
{
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self, cx: &mut Context) -> Poll02<Self::Item, Self::Error> {
        with_context_poll(cx, || self.v01.poll())
    }
}

impl<S> StreamInto02 for S
where
    S: Stream01,
{
    fn into_02_compat(self) -> Stream01As02<Self>
    where
        Self: Sized,
    {
        Stream01As02 {
            v01: self,
        }
    }
}

impl<S> Stream02 for Stream01As02<S>
where
    S: Stream01,
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll_next(&mut self, cx: &mut Context) -> Poll02<Option<Self::Item>, Self::Error> {
        with_context_poll(cx, || self.v01.poll())
    }
}

impl<S> Sink01 for Stream01As02<S>
where
    S: Sink01,
{
    type SinkItem = S::SinkItem;
    type SinkError = S::SinkError;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend01<Self::SinkItem, Self::SinkError> {
        self.v01.start_send(item)
    }

    fn poll_complete(&mut self) -> Poll01<(), Self::SinkError> {
        self.v01.poll_complete()
    }
}

impl<S> Sink02 for Stream01As02<S>
where
    S: Sink02,
{
    type SinkItem = S::SinkItem;
    type SinkError = S::SinkError;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll02<(), Self::SinkError> {
        self.v01.poll_ready(cx)
    }

    fn start_send(&mut self, item: Self::SinkItem) -> Result<(), Self::SinkError> {
        self.v01.start_send(item)
    }

    fn poll_flush(&mut self, cx: &mut Context) -> Poll02<(), Self::SinkError> {
        self.v01.poll_flush(cx)
    }

    fn poll_close(&mut self, cx: &mut Context) -> Poll02<(), Self::SinkError> {
        self.v01.poll_close(cx)
    }
}

impl<S> SinkInto02 for S
where
    S: Sink01,
{
    fn sink_into_02_compat(self) -> Sink01As02<Self>
    where
        Self: Sized,
    {
        Sink01As02 {
            v01: self,
            buf: Vec::new(),
        }
    }
}

impl<S> Sink02 for Sink01As02<S>
where
    S: Sink01,
{
    type SinkItem = S::SinkItem;
    type SinkError = S::SinkError;

    fn poll_ready(&mut self, _cx: &mut Context) -> Poll02<(), Self::SinkError> {
        // Due to the internal buffer, this sink will always be ready.
        Ok(Async02::Ready(()))
    }

    fn start_send(&mut self, item: Self::SinkItem) -> Result<(), Self::SinkError> {
        // Again, the buffer is always ready.
        self.buf.push(item);
        Ok(())
    }

    fn poll_flush(&mut self, cx: &mut Context) -> Poll02<(), Self::SinkError> {
        // Try sending all buffered items one by one.
        loop {
            if self.buf.len() == 0 {
                break;
            }

            let item = self.buf.remove(0);

            let start_send = with_context(cx, || self.v01.start_send(item));

            match start_send {
                Ok(AsyncSink01::NotReady(t)) => {
                    // Queue the item back and stop trying.
                    self.buf.insert(0, t);
                    break;
                },

                Err(e) => return Err(e),

                // Keep going.
                Ok(AsyncSink01::Ready) => continue,
            }
        }

        with_context_poll(cx, || self.v01.poll_complete())
    }

    fn poll_close(&mut self, cx: &mut Context) -> Poll02<(), Self::SinkError> {
        self.poll_flush(cx)
    }
}

impl<S> Stream01 for Sink01As02<S>
where
    S: Sink01 + Stream01,
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll01<Option<Self::Item>, Self::Error> {
        self.v01.poll()
    }
}

impl<S> Stream02 for Sink01As02<S>
where
    S: Sink01 + Stream02,
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll_next(&mut self, cx: &mut Context) -> Poll02<Option<Self::Item>, Self::Error> {
        self.v01.poll_next(cx)
    }
}

impl<E> ExecutorInto02 for E
where
    E: Executor01<
        Future02NeverAs01Unit<
            BoxedExecutor02,
            Box<Future02<Item=(), Error=Never> + Send>
        >
    >,
    E: Clone + Send + 'static,
{
    fn into_02_compat(self) -> Executor01As02<Self> {
        Executor01As02 {
            v01: self,
        }
    }
}

impl<E> Executor02 for Executor01As02<E>
where
    E: Executor01<
        Future02NeverAs01Unit<
            BoxedExecutor02,
            Box<Future02<Item=(), Error=Never> + Send>
        >
    >,
    E: Clone + Send + 'static,
{
    fn spawn(&mut self, f: Box<Future02<Item=(), Error=Never> + Send>) -> Result<(), SpawnError> {
        use super::futures_02::FutureInto01;

        self.v01.execute(f.into_01_compat_never_unit(BoxedExecutor02(Box::new(self.clone()))))
            .map_err(|_| SpawnError::shutdown())
    }
}


impl<I> TokioIntoAsyncIo02 for I {
    fn into_v02_compat(self) -> TokioAsAsyncIo02<Self>
    where
        Self: AsyncReadTk + AsyncWriteTk + Sized,
    {
        TokioAsAsyncIo02 {
            v01: self,
        }
    }
}

impl<I: AsyncReadTk> AsyncRead02 for TokioAsAsyncIo02<I> {
    fn poll_read(&mut self, cx: &mut Context, buf: &mut [u8]) -> Poll02<usize, io::Error> {
        with_context_poll(cx, || self.v01.poll_read(buf))
    }
}

impl<I: AsyncWriteTk> AsyncWrite02 for TokioAsAsyncIo02<I> {
    fn poll_write(&mut self, cx: &mut Context, buf: &[u8]) -> Poll02<usize, io::Error> {
        with_context_poll(cx, || self.v01.poll_write(buf))
    }

    fn poll_flush(&mut self, cx: &mut Context) -> Poll02<(), io::Error> {
        with_context_poll(cx, || self.v01.poll_flush())
    }

    fn poll_close(&mut self, cx: &mut Context) -> Poll02<(), io::Error> {
        with_context_poll(cx, || self.v01.shutdown())
    }
}

/// Execute a function with the context used as a v0.1 `Notifier`.
pub fn with_context<F, R>(cx: &mut Context, f: F) -> R
where
    F: FnOnce() -> R,
{
    with_notify(&WakerToHandle(cx.waker()), 0, f)
}

/// Execute a function with the context used as a v0.1 `Notifier`, converting
/// v0.1 `Poll` into v0.2 version.
pub fn with_context_poll<F, R, E>(cx: &mut Context, f: F) -> Poll02<R, E>
where
    F: FnOnce() -> Poll01<R, E>,
{
    with_context(cx, move || {
        match f() {
            Ok(Async01::Ready(val)) => Ok(Async02::Ready(val)),
            Ok(Async01::NotReady) => Ok(Async02::Pending),
            Err(err) => Err(err),
        }
    })
}

struct NotifyWaker(Waker);

#[allow(missing_debug_implementations)]
#[derive(Clone)]
struct WakerToHandle<'a>(&'a Waker);

#[doc(hidden)]
impl<'a> From<WakerToHandle<'a>> for NotifyHandle {
    fn from(handle: WakerToHandle<'a>) -> NotifyHandle {
        let ptr = Box::new(NotifyWaker(handle.0.clone()));

        unsafe {
            NotifyHandle::new(Box::into_raw(ptr))
        }
    }
}

impl Notify for NotifyWaker {
    fn notify(&self, _: usize) {
        self.0.wake();
    }
}

unsafe impl UnsafeNotify for NotifyWaker {
    unsafe fn clone_raw(&self) -> NotifyHandle {
        WakerToHandle(&self.0).into()
    }

    unsafe fn drop_raw(&self) {
        let ptr: *const UnsafeNotify = self;
        drop(Box::from_raw(ptr as *mut UnsafeNotify));
    }
}
