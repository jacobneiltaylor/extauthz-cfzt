use tonic::transport::Server;
use tonic::transport::server::Router;
use envoy_types::ext_authz::v3::pb::{Authorization, AuthorizationServer};
use std::process::ExitCode;

pub fn handle_error(error: Box<dyn std::error::Error>, message: &str, code: u8) -> ExitCode {
    log::error!("{}: {}", message, error);
    ExitCode::from(code)
}

pub fn new_router(server: impl Authorization) -> Router {
    Server::builder().add_service(AuthorizationServer::new(server))
}