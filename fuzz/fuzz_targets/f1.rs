#![no_main]
use libfuzzer_sys::fuzz_target;

use futures::io::{self, Cursor};
use futures::io::{AsyncRead, AsyncWrite};
use futures::task::{Context, Poll};
use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Body, Request, Response};
use std::pin::Pin;

#[derive(Default)]
struct FakeClient {
    data: Cursor<Vec<u8>>,
}

impl FakeClient {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: Cursor::new(data),
        }
    }
}

impl tokio::io::AsyncRead for FakeClient {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.data).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for FakeClient {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Pin::new(&mut self.data).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
}

fuzz_target!(|data: &[u8]| {
    let fake_client = FakeClient::new(data.to_vec());
    let mut runtime_builder = tokio::runtime::Builder::new();
    runtime_builder.enable_all();
    runtime_builder.basic_scheduler();
    let mut runtime = runtime_builder.build().unwrap();
    runtime.block_on(async move {
        let server = Http::new();
        let service = service_fn(|_req: Request<Body>| async move {
            Ok(Response::new(Body::from("OK"))) as Result<_, io::Error>
        });
        let cnx = server.serve_connection(fake_client, service);
        cnx.await.ok();
    });
});
