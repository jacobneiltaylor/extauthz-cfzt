use jnt::sockets::Listener;
use jnt::types::{StdResult, UnixListener, EmptyResult};
use tokio::net::{TcpListener as TokioTcpListener, UnixListener as TokioUnixListener};
use std::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::server::Router;

#[cfg(unix)]
use tokio_stream::wrappers::UnixListenerStream;

#[cfg(not(unix))]
type UnixListenerStream = ();

#[cfg(unix)]
fn bind_unix_socket(listener: UnixListener) -> StdResult<UnixListenerStream> {
    Ok(UnixListenerStream::new(TokioUnixListener::from_std(listener)?))
}

#[cfg(not(unix))]
fn bind_unix_socket() -> StdResult<UnixListenerStream> {
    jnt::opaque_err!("unsupported platform")
}

fn bind_tcp_socket(listener: TcpListener) -> StdResult<TcpListenerStream> {
    Ok(TcpListenerStream::new(TokioTcpListener::from_std(listener)?))
}

fn handle_result(result: Result<(), tonic::transport::Error>) -> EmptyResult {
    match result {
        Ok(_) =>  Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

pub async fn run_server(router: Router, listener: Listener) -> EmptyResult {
    match listener {
        Listener::Unix(socket) => handle_result(
            router.serve_with_incoming(bind_unix_socket(socket)?).await
        ),
        Listener::Tcp(socket) => handle_result(
            router.serve_with_incoming(bind_tcp_socket(socket)?).await
        ),
    }
}
