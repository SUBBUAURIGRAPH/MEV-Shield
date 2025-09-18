use clap::{Parser, Subcommand};
use anyhow::Result;
use mev_shield::cli::CliHandler;

#[derive(Parser)]
#[clap(name = "mev-shield-cli")]
#[clap(about = "MEV Shield Command Line Interface", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Submit a protected transaction
    Submit {
        /// Transaction data in JSON format
        #[clap(short, long)]
        transaction: String,
        
        /// Protection level (basic, standard, maximum)
        #[clap(short, long, default_value = "standard")]
        protection: String,
    },
    
    /// Check transaction status
    Status {
        /// Transaction ID
        #[clap(short, long)]
        id: String,
    },
    
    /// View MEV analytics
    Analytics {
        /// Timeframe (1h, 24h, 7d, 30d)
        #[clap(short, long, default_value = "24h")]
        timeframe: String,
    },
    
    /// Check pending rewards
    Rewards {
        /// User address
        #[clap(short, long)]
        address: String,
    },
    
    /// Register as a block builder
    Register {
        /// Builder address
        #[clap(short, long)]
        address: String,
        
        /// Stake amount in ETH
        #[clap(short, long)]
        stake: f64,
    },
    
    /// Monitor system health
    Monitor {
        /// Enable continuous monitoring
        #[clap(short, long)]
        continuous: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let handler = CliHandler::new()?;
    
    match &cli.command {
        Commands::Submit { transaction, protection } => {
            handler.submit_transaction(transaction, protection).await?;
        }
        Commands::Status { id } => {
            handler.check_status(id).await?;
        }
        Commands::Analytics { timeframe } => {
            handler.show_analytics(timeframe).await?;
        }
        Commands::Rewards { address } => {
            handler.check_rewards(address).await?;
        }
        Commands::Register { address, stake } => {
            handler.register_builder(address, *stake).await?;
        }
        Commands::Monitor { continuous } => {
            handler.monitor_system(*continuous).await?;
        }
    }
    
    Ok(())
}
