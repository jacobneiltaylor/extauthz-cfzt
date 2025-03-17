mod config;
mod helpers;
mod server;
mod socket;

use config::audience;
use config::audience::discovery::discover_audience_provider;
use config::audience::schema::AudienceProvider;
use helpers::new_router;
use jnt::types::EmptyResult;
use server::extauthz::CloudflareZeroTrustAuthorizationServer;
use socket::run_server;
use std::{process::ExitCode, sync::Arc};
use tokio::runtime::Builder;
use tokio_cron_scheduler::{Job, JobScheduler};

use config::bootstrap::discovery::discover_bootstrap_configuration;
use config::bootstrap::schema::Configuration as BootstrapConfiguration;

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc; // Use mimalloc allocator for Muslc targets

fn main() -> ExitCode {
    env_logger::init();
    log::info!("Starting runtime");

    match start() {
        Ok(_) => ExitCode::from(0),
        Err(e) => e,
    }
}

fn start() -> Result<(), ExitCode> {
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()
        .or_else(|e| {
            Err(helpers::handle_error(
                Box::new(e),
                "error during runtime start",
                1,
            ))
        })?;

    log::info!("Performing bootstrap configuration discovery");
    let configuration = discover_bootstrap_configuration()
        .or_else(|e| Err(helpers::handle_error(e, "error during config discovery", 2)))?;

    log::info!("Performing audience provider discovery");
    let aud_provider = Arc::new(discover_audience_provider().or_else(|e| {
        Err(helpers::handle_error(
            e,
            "error during audience provider discovery",
            3,
        ))
    })?);

    runtime
        .block_on(async_main(configuration, aud_provider.clone()))
        .or_else(|e| Err(helpers::handle_error(e, "error during execution", 100)))
}

async fn async_main(
    bootstrap: BootstrapConfiguration,
    aud_provider: Arc<Box<dyn AudienceProvider>>,
) -> jnt::types::EmptyResult {
    let listener = bootstrap.open_listener()?;
    let validator = Arc::new(bootstrap.new_validator()?);
    let mut scheduler = JobScheduler::new().await?;

    let router = new_router(CloudflareZeroTrustAuthorizationServer::new(
        validator.clone(),
        aud_provider,
        &bootstrap.validator.get_default_team_name(),
        bootstrap.nbf_validation,
        bootstrap.exp_validation,
    ));

    if bootstrap.validator.requires_refresh() {
        log::info!("Registering validator syncronisation job");
        scheduler
            .add(Job::new(bootstrap.sync_schedule, move |_, _| {
                log::info!("Triggering validator syncronisation");
                let _ = validator.sync();
            })?)
            .await?;
    }

    log::info!("Starting validation syncronisation job");
    scheduler.start().await?;

    log::info!("Running ExtAuthz server");
    run_server(router, listener).await?;

    log::info!("Server stopped, shutting down validation syncronisation job");
    scheduler.shutdown().await?;

    Ok(())
}
