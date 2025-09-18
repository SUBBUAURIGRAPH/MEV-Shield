mod api;
mod auth;
mod cli;
mod config;
mod core;
mod detection;
mod encryption;
mod error;
mod ordering;
mod redistribution;
mod block_builder;
mod monitoring;
mod types;
mod traits;

use std::sync::Arc;
use anyhow::Result;
use clap::Parser;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::Config,
    core::MEVShieldCore,
    api::ApiServer,
    monitoring::MetricsCollector,
};

#[derive(Parser)]
#[clap(name = "MEV Shield")]
#[clap(author = "Aurigraph DLT")]
#[clap(version = "1.0.0")]
#[clap(about = "Comprehensive MEV protection for blockchain networks")]
struct Args {
    /// Path to configuration file
    #[clap(short, long, default_value = "config.toml")]
    config: String,

    /// Enable debug mode
    #[clap(short, long)]
    debug: bool,

    /// Port to listen on
    #[clap(short, long, default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging
    init_logging(args.debug)?;

    info!("🛡️ MEV Shield v1.0.0 - Starting up");
    info!("📍 Loading configuration from: {}", args.config);

    // Load configuration
    let mut config = Config::from_file(&args.config)?;
    config.api.port = args.port;

    // Initialize metrics collector
    let metrics = Arc::new(MetricsCollector::new()?);

    // Initialize core MEV Shield system
    info!("🔧 Initializing MEV Shield core...");
    let core = MEVShieldCore::new(config.clone(), metrics.clone()).await?;
    let core = Arc::new(core);

    // Start background services
    info!("🚀 Starting background services...");
    start_background_services(core.clone()).await?;

    // Start API server
    info!("🌐 Starting API server on port {}", config.api.port);
    let api_server = ApiServer::new(core.clone(), config.api.clone());
    
    // Run the server
    api_server.run().await?;

    Ok(())
}

fn init_logging(debug: bool) -> Result<()> {
    let env_filter = if debug {
        "debug"
    } else {
        "info"
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| env_filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

async fn start_background_services(core: Arc<MEVShieldCore>) -> Result<()> {
    // Start encryption service
    {
        let core = core.clone();
        tokio::spawn(async move {
            if let Err(e) = core.encryption_service.start_cleanup_service().await {
                error!("Encryption cleanup service error: {}", e);
            }
        });
    }

    // Start ordering service
    {
        let core = core.clone();
        tokio::spawn(async move {
            if let Err(e) = core.ordering_service.start_batch_processor().await {
                error!("Ordering batch processor error: {}", e);
            }
        });
    }

    // Start MEV detection service
    {
        let core = core.clone();
        tokio::spawn(async move {
            if let Err(e) = core.detection_service.start_monitoring().await {
                error!("MEV detection monitoring error: {}", e);
            }
        });
    }

    // Start redistribution service
    {
        let core = core.clone();
        tokio::spawn(async move {
            if let Err(e) = core.redistribution_service.start_distribution_service().await {
                error!("MEV redistribution service error: {}", e);
            }
        });
    }

    // Start block builder coordinator
    {
        let core = core.clone();
        tokio::spawn(async move {
            if let Err(e) = core.block_builder.start_coordinator().await {
                error!("Block builder coordinator error: {}", e);
            }
        });
    }

    // Start metrics exporter
    {
        let core = core.clone();
        tokio::spawn(async move {
            if let Err(e) = core.metrics_collector.start_exporter().await {
                error!("Metrics exporter error: {}", e);
            }
        });
    }

    info!("✅ All background services started successfully");

    Ok(())
}
