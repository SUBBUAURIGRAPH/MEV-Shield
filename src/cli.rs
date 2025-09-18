// cli/src/main.rs
//! MEV Shield Command Line Interface
//! 
//! Provides a command-line tool for interacting with MEV Shield services,
//! submitting transactions, checking status, and managing the system.

use clap::{Parser, Subcommand};
use mev_shield_core::{
    config::MEVShieldConfig,
    types::*,
};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

/// MEV Shield CLI tool
#[derive(Parser)]
#[command(name = "mev-shield")]
#[command(about = "A CLI tool for MEV Shield - Maximum Extractable Value Protection")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// MEV Shield API endpoint
    #[arg(long, default_value = "http://localhost:8080")]
    endpoint: String,
    
    /// API key for authentication
    #[arg(long, env = "MEV_SHIELD_API_KEY")]
    api_key: Option<String>,
    
    /// Output format (json, table, yaml)
    #[arg(long, default_value = "table")]
    output: String,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Submit a transaction for MEV protection
    Submit {
        /// Transaction sender address
        #[arg(long)]
        from: String,
        
        /// Transaction recipient address
        #[arg(long)]
        to: String,
        
        /// Transaction value in wei
        #[arg(long, default_value = "0")]
        value: String,
        
        /// Gas limit
        #[arg(long, default_value = "21000")]
        gas: u64,
        
        /// Gas price in wei
        #[arg(long, default_value = "20000000000")]
        gas_price: String,
        
        /// Transaction nonce
        #[arg(long)]
        nonce: u64,
        
        /// Transaction data (hex)
        #[arg(long, default_value = "0x")]
        data: String,
        
        /// Chain ID
        #[arg(long, default_value = "1")]
        chain_id: u64,
        
        /// Protection level (basic, standard, maximum, enterprise)
        #[arg(long, default_value = "standard")]
        protection: String,
        
        /// Use private pool
        #[arg(long)]
        private_pool: bool,
        
        /// Time lock duration in seconds
        #[arg(long)]
        time_lock: Option<u64>,
        
        /// Maximum slippage percentage
        #[arg(long)]
        max_slippage: Option<f64>,
    },
    
    /// Check transaction status
    Status {
        /// Transaction ID to check
        transaction_id: String,
        
        /// Watch for status changes
        #[arg(short, long)]
        watch: bool,
        
        /// Watch interval in seconds
        #[arg(long, default_value = "5")]
        interval: u64,
    },
    
    /// List transactions
    List {
        /// Maximum number of transactions to show
        #[arg(long, default_value = "10")]
        limit: u32,
        
        /// Offset for pagination
        #[arg(long, default_value = "0")]
        offset: u32,
        
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        
        /// Filter by chain ID
        #[arg(long)]
        chain_id: Option<u64>,
    },
    
    /// Analytics commands
    Analytics {
        #[command(subcommand)]
        analytics_command: AnalyticsCommands,
    },
    
    /// System administration commands
    Admin {
        #[command(subcommand)]
        admin_command: AdminCommands,
    },
    
    /// Configuration management
    Config {
        #[command(subcommand)]
        config_command: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum AnalyticsCommands {
    /// Get MEV analytics
    Mev {
        /// Time frame (1h, 24h, 7d, 30d)
        #[arg(long, default_value = "24h")]
        timeframe: String,
        
        /// Filter by chain ID
        #[arg(long)]
        chain_id: Option<u64>,
    },
    
    /// Get user analytics
    User {
        /// User address
        address: String,
    },
    
    /// Get network analytics
    Network {
        /// Chain ID
        chain_id: u64,
    },
}

#[derive(Subcommand)]
enum AdminCommands {
    /// Check system status
    Status,
    
    /// Get system metrics
    Metrics,
    
    /// List validators
    Validators,
    
    /// Get configuration
    Config,
    
    /// Health check
    Health,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    
    /// Validate configuration file
    Validate {
        /// Configuration file path
        #[arg(long, default_value = "mev-shield.toml")]
        file: String,
    },
    
    /// Generate default configuration
    Generate {
        /// Output file path
        #[arg(long, default_value = "mev-shield.toml")]
        output: String,
    },
}

/// HTTP client for API calls
struct ApiClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl ApiClient {
    fn new(base_url: String, api_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            base_url,
            api_key,
        }
    }
    
    async fn submit_transaction(
        &self,
        request: &SubmitTransactionRequest,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/transactions", self.base_url);
        
        let mut req = self.client.post(&url).json(request);
        
        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req.send().await?;
        let json: Value = response.json().await?;
        
        Ok(json)
    }
    
    async fn get_transaction_status(
        &self,
        transaction_id: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/transactions/{}", self.base_url, transaction_id);
        
        let mut req = self.client.get(&url);
        
        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req.send().await?;
        let json: Value = response.json().await?;
        
        Ok(json)
    }
    
    async fn list_transactions(
        &self,
        params: &HashMap<String, String>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/transactions", self.base_url);
        
        let mut req = self.client.get(&url);
        
        for (key, value) in params {
            req = req.query(&[(key, value)]);
        }
        
        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req.send().await?;
        let json: Value = response.json().await?;
        
        Ok(json)
    }
    
    async fn get_analytics(
        &self,
        endpoint: &str,
        params: &HashMap<String, String>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/analytics/{}", self.base_url, endpoint);
        
        let mut req = self.client.get(&url);
        
        for (key, value) in params {
            req = req.query(&[(key, value)]);
        }
        
        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req.send().await?;
        let json: Value = response.json().await?;
        
        Ok(json)
    }
    
    async fn get_admin_endpoint(
        &self,
        endpoint: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/{}", self.base_url, endpoint);
        
        let mut req = self.client.get(&url);
        
        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req.send().await?;
        
        if response.status().is_success() {
            let json: Value = response.json().await?;
            Ok(json)
        } else {
            Err(format!("HTTP {}: {}", response.status(), response.text().await?).into())
        }
    }
}

/// API request structures
#[derive(serde::Serialize)]
struct SubmitTransactionRequest {
    transaction: TransactionData,
    protection: ProtectionConfigData,
    #[serde(rename = "chainId")]
    chain_id: u64,
}

#[derive(serde::Serialize)]
struct TransactionData {
    from: String,
    to: String,
    value: String,
    gas: u64,
    #[serde(rename = "gasPrice")]
    gas_price: String,
    nonce: u64,
    data: String,
}

#[derive(serde::Serialize)]
struct ProtectionConfigData {
    level: String,
    #[serde(rename = "privatePool")]
    private_pool: bool,
    #[serde(rename = "timeLock")]
    time_lock: Option<String>,
    #[serde(rename = "maxSlippage")]
    max_slippage: Option<String>,
}

/// Output formatters
struct OutputFormatter {
    format: String,
}

impl OutputFormatter {
    fn new(format: String) -> Self {
        Self { format }
    }
    
    fn format_value(&self, value: &Value) -> String {
        match self.format.as_str() {
            "json" => serde_json::to_string_pretty(value).unwrap_or_default(),
            "yaml" => serde_yaml::to_string(value).unwrap_or_default(),
            "table" | _ => self.format_as_table(value),
        }
    }
    
    fn format_as_table(&self, value: &Value) -> String {
        match value {
            Value::Object(map) => {
                let mut output = String::new();
                
                for (key, val) in map {
                    output.push_str(&format!("{:<20} {}\n", key, self.format_simple_value(val)));
                }
                
                output
            }
            _ => self.format_simple_value(value),
        }
    }
    
    fn format_simple_value(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => serde_json::to_string(value).unwrap_or_default(),
        }
    }
}

/// Main application logic
async fn run_cli(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let api_client = ApiClient::new(cli.endpoint, cli.api_key);
    let formatter = OutputFormatter::new(cli.output);
    
    match cli.command {
        Commands::Submit {
            from,
            to,
            value,
            gas,
            gas_price,
            nonce,
            data,
            chain_id,
            protection,
            private_pool,
            time_lock,
            max_slippage,
        } => {
            let request = SubmitTransactionRequest {
                transaction: TransactionData {
                    from,
                    to,
                    value,
                    gas,
                    gas_price,
                    nonce,
                    data,
                },
                protection: ProtectionConfigData {
                    level: protection,
                    private_pool,
                    time_lock: time_lock.map(|t| format!("{}s", t)),
                    max_slippage: max_slippage.map(|s| format!("{}%", s)),
                },
                chain_id,
            };
            
            if cli.verbose {
                println!("Submitting transaction...");
            }
            
            let response = api_client.submit_transaction(&request).await?;
            println!("{}", formatter.format_value(&response));
        }
        
        Commands::Status {
            transaction_id,
            watch,
            interval,
        } => {
            if !watch {
                let response = api_client.get_transaction_status(&transaction_id).await?;
                println!("{}", formatter.format_value(&response));
            } else {
                println!("Watching transaction status (press Ctrl+C to stop)...\n");
                
                loop {
                    match api_client.get_transaction_status(&transaction_id).await {
                        Ok(response) => {
                            println!("\x1B[2J\x1B[1;1H"); // Clear screen
                            println!("Transaction Status (updated: {})", chrono::Utc::now().format("%H:%M:%S"));
                            println!("{}", formatter.format_value(&response));
                            
                            // Check if transaction is final
                            if let Some(status) = response.get("data").and_then(|d| d.get("status")) {
                                if status == "executed" || status == "failed" {
                                    println!("\nTransaction reached final status.");
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error checking status: {}", e);
                        }
                    }
                    
                    sleep(Duration::from_secs(interval)).await;
                }
            }
        }
        
        Commands::List {
            limit,
            offset,
            status,
            chain_id,
        } => {
            let mut params = HashMap::new();
            params.insert("limit".to_string(), limit.to_string());
            params.insert("offset".to_string(), offset.to_string());
            
            if let Some(status) = status {
                params.insert("status".to_string(), status);
            }
            
            if let Some(chain_id) = chain_id {
                params.insert("chain_id".to_string(), chain_id.to_string());
            }
            
            let response = api_client.list_transactions(&params).await?;
            println!("{}", formatter.format_value(&response));
        }
        
        Commands::Analytics { analytics_command } => {
            match analytics_command {
                AnalyticsCommands::Mev { timeframe, chain_id } => {
                    let mut params = HashMap::new();
                    params.insert("timeframe".to_string(), timeframe);
                    
                    if let Some(chain_id) = chain_id {
                        params.insert("chain_id".to_string(), chain_id.to_string());
                    }
                    
                    let response = api_client.get_analytics("mev", &params).await?;
                    println!("{}", formatter.format_value(&response));
                }
                
                AnalyticsCommands::User { address } => {
                    let response = api_client
                        .get_analytics(&format!("user/{}", address), &HashMap::new())
                        .await?;
                    println!("{}", formatter.format_value(&response));
                }
                
                AnalyticsCommands::Network { chain_id } => {
                    let response = api_client
                        .get_analytics(&format!("network/{}", chain_id), &HashMap::new())
                        .await?;
                    println!("{}", formatter.format_value(&response));
                }
            }
        }
        
        Commands::Admin { admin_command } => {
            match admin_command {
                AdminCommands::Status => {
                    let response = api_client.get_admin_endpoint("status").await?;
                    println!("{}", formatter.format_value(&response));
                }
                
                AdminCommands::Metrics => {
                    let response = api_client.get_admin_endpoint("metrics").await?;
                    println!("{}", response);
                }
                
                AdminCommands::Validators => {
                    let response = api_client.get_admin_endpoint("admin/validators").await?;
                    println!("{}", formatter.format_value(&response));
                }
                
                AdminCommands::Config => {
                    let response = api_client.get_admin_endpoint("admin/config").await?;
                    println!("{}", formatter.format_value(&response));
                }
                
                AdminCommands::Health => {
                    let response = api_client.get_admin_endpoint("health").await?;
                    println!("{}", formatter.format_value(&response));
                }
            }
        }
        
        Commands::Config { config_command } => {
            match config_command {
                ConfigCommands::Show => {
                    let config = MEVShieldConfig::default();
                    let json = serde_json::to_value(&config)?;
                    println!("{}", formatter.format_value(&json));
                }
                
                ConfigCommands::Validate { file } => {
                    match MEVShieldConfig::from_file(&file) {
                        Ok(config) => {
                            match config.validate() {
                                Ok(_) => println!("✅ Configuration is valid"),
                                Err(e) => {
                                    eprintln!("❌ Configuration validation failed: {}", e);
                                    std::process::exit(1);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to load configuration: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                
                ConfigCommands::Generate { output } => {
                    let config = MEVShieldConfig::default();
                    let toml_content = toml::to_string_pretty(&config)?;
                    
                    std::fs::write(&output, toml_content)?;
                    println!("✅ Generated default configuration: {}", output);
                }
            }
        }
    }
    
    Ok(())
}

/// CLI utilities
fn validate_address(address: &str) -> Result<(), String> {
    if !address.starts_with("0x") || address.len() != 42 {
        return Err("Invalid Ethereum address format".to_string());
    }
    
    if let Err(_) = hex::decode(&address[2..]) {
        return Err("Invalid hex characters in address".to_string());
    }
    
    Ok(())
}

fn validate_transaction_id(id: &str) -> Result<(), String> {
    if let Err(_) = Uuid::parse_str(id) {
        return Err("Invalid transaction ID format (must be UUID)".to_string());
    }
    
    Ok(())
}

/// Examples and help text
fn print_examples() {
    println!("Examples:");
    println!();
    println!("  # Submit a simple transfer");
    println!("  mev-shield submit \\");
    println!("    --from 0x742d35cc6465c3c962800060acea9d8ac2e7a0cf \\");
    println!("    --to 0x1234567890123456789012345678901234567890 \\");
    println!("    --value 1000000000000000000 \\");
    println!("    --nonce 42 \\");
    println!("    --protection maximum");
    println!();
    println!("  # Check transaction status");
    println!("  mev-shield status 550e8400-e29b-41d4-a716-446655440000");
    println!();
    println!("  # Watch transaction status");
    println!("  mev-shield status 550e8400-e29b-41d4-a716-446655440000 --watch");
    println!();
    println!("  # Get MEV analytics");
    println!("  mev-shield analytics mev --timeframe 7d");
    println!();
    println!("  # Get system status");
    println!("  mev-shield admin status");
    println!();
    println!("  # Generate configuration file");
    println!("  mev-shield config generate --output my-config.toml");
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    // Show examples if requested
    if std::env::args().any(|arg| arg == "--examples") {
        print_examples();
        return;
    }
    
    if let Err(e) = run_cli(cli).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_address() {
        assert!(validate_address("0x742d35cc6465c3c962800060acea9d8ac2e7a0cf").is_ok());
        assert!(validate_address("0x1234567890123456789012345678901234567890").is_ok());
        assert!(validate_address("742d35cc6465c3c962800060acea9d8ac2e7a0cf").is_err());
        assert!(validate_address("0x742d35cc6465c3c962800060acea9d8ac2e7a0c").is_err());
        assert!(validate_address("0x742d35cc6465c3c962800060acea9d8ac2e7a0cg").is_err());
    }
    
    #[test]
    fn test_validate_transaction_id() {
        assert!(validate_transaction_id("550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(validate_transaction_id("not-a-uuid").is_err());
        assert!(validate_transaction_id("").is_err());
    }
    
    #[test]
    fn test_output_formatter() {
        let formatter = OutputFormatter::new("json".to_string());
        let value = serde_json::json!({"test": "value"});
        let output = formatter.format_value(&value);
        assert!(output.contains("test"));
        assert!(output.contains("value"));
    }
}