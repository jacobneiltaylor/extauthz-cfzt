mod config;
mod helpers;
mod server;
mod socket;

use helpers::new_router;
use server::extauthz::CloudflareZeroTrustAuthorizationServer;
use socket::run_server;
use tokio::runtime::Builder;
use tokio_cron_scheduler::{Job, JobScheduler};
use std::{process::ExitCode, sync::Arc};

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc; // Use mimalloc allocator for Muslc targets

fn main() -> ExitCode {
    env_logger::init();
    log::info!("Starting runtime");
    match Builder::new_multi_thread().enable_all().build() {
        Ok(rt) => {
            log::info!("Performing configuration discovery");
            match config::discovery::discover_configuration() {
                Ok(configuration) => {
                    match rt.block_on(async_main(configuration)) {
                        Ok(_) => ExitCode::from(0),
                        Err(e) => helpers::handle_error(e, "error during execution", 3)
                    }
                },
                Err(e) => helpers::handle_error(e, "error during config discovery", 2)
            }
        },
        Err(e) => helpers::handle_error(Box::new(e), "error during runtime start", 1)
    }
}

async fn async_main(configuration: config::schema::Configuration) -> jnt::types::EmptyResult {
    let listener = configuration.open_listener()?;
    let validator = Arc::new(configuration.new_validator()?);
    let mut scheduler = JobScheduler::new().await?;

    let router = new_router(CloudflareZeroTrustAuthorizationServer::new(
        validator.clone(),
        &configuration.validator.get_default_team_name()
    ));

    scheduler.add(Job::new(configuration.sync_schedule, move |_, _| {
        log::info!("Triggering validator syncronisation");
        let _ = validator.sync();
    })?).await?;

    log::info!("Starting validation syncronisation job");
    scheduler.start().await?;

    log::info!("Running ExtAuthz server");
    run_server(router, listener).await?;

    log::info!("Server stopped, shutting down validation syncronisation job");
    scheduler.shutdown().await?;

    Ok(())
}
