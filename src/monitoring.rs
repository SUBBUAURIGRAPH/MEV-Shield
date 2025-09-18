use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use tracing::{info, error, warn};
use prometheus::{Registry, Counter, Gauge, Histogram, HistogramOpts, Encoder, TextEncoder};
use ethers::types::{Address, U256};

use crate::types::{Transaction, Block};
use crate::error::MEVShieldError;

/// Metrics Collector for monitoring system performance
pub struct MetricsCollector {
    registry: Registry,
    
    // Transaction metrics
    transactions_processed: Counter,
    transactions_protected: Counter,
    transactions_failed: Counter,
    
    // MEV metrics
    mev_detected: Counter,
    mev_prevented: Counter,
    mev_value_captured: Counter,
    mev_value_distributed: Counter,
    
    // Performance metrics
    encryption_latency: Histogram,
    ordering_latency: Histogram,
    detection_latency: Histogram,
    
    // System metrics
    active_builders: Gauge,
    mempool_size: Gauge,
    pending_distributions: Gauge,
    
    // Alert system
    alert_system: AlertSystem,
}

/// Alert System for security monitoring
pub struct AlertSystem {
    alerts: Arc<RwLock<Vec<SecurityAlert>>>,
    alert_handlers: Vec<Box<dyn AlertHandler + Send + Sync>>,
}

#[derive(Clone, Debug)]
pub struct SecurityAlert {
    pub severity: AlertSeverity,
    pub alert_type: AlertType,
    pub message: String,
    pub details: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug)]
pub enum AlertType {
    MEVDetected,
    SecurityBreach,
    PerformanceDegradation,
    SystemFailure,
    AnomalousActivity,
}

#[async_trait]
pub trait AlertHandler {
    async fn handle_alert(&self, alert: &SecurityAlert) -> Result<()>;
}

impl MetricsCollector {
    pub fn new() -> Result<Self> {
        let registry = Registry::new();
        
        // Transaction metrics
        let transactions_processed = Counter::new(
            "mev_shield_transactions_processed_total",
            "Total number of transactions processed",
        )?;
        let transactions_protected = Counter::new(
            "mev_shield_transactions_protected_total",
            "Total number of transactions protected from MEV",
        )?;
        let transactions_failed = Counter::new(
            "mev_shield_transactions_failed_total",
            "Total number of failed transactions",
        )?;
        
        // MEV metrics
        let mev_detected = Counter::new(
            "mev_shield_mev_detected_total",
            "Total number of MEV attempts detected",
        )?;
        let mev_prevented = Counter::new(
            "mev_shield_mev_prevented_total",
            "Total number of MEV attempts prevented",
        )?;
        let mev_value_captured = Counter::new(
            "mev_shield_mev_value_captured_wei",
            "Total MEV value captured in wei",
        )?;
        let mev_value_distributed = Counter::new(
            "mev_shield_mev_value_distributed_wei",
            "Total MEV value distributed in wei",
        )?;
        
        // Performance metrics
        let encryption_latency = Histogram::with_opts(
            HistogramOpts::new(
                "mev_shield_encryption_latency_seconds",
                "Latency of transaction encryption",
            )
        )?;
        let ordering_latency = Histogram::with_opts(
            HistogramOpts::new(
                "mev_shield_ordering_latency_seconds",
                "Latency of fair ordering computation",
            )
        )?;
        let detection_latency = Histogram::with_opts(
            HistogramOpts::new(
                "mev_shield_detection_latency_seconds",
                "Latency of MEV detection",
            )
        )?;
        
        // System metrics
        let active_builders = Gauge::new(
            "mev_shield_active_builders",
            "Number of active block builders",
        )?;
        let mempool_size = Gauge::new(
            "mev_shield_mempool_size",
            "Current size of encrypted mempool",
        )?;
        let pending_distributions = Gauge::new(
            "mev_shield_pending_distributions",
            "Number of pending MEV distributions",
        )?;
        
        // Register all metrics
        registry.register(Box::new(transactions_processed.clone()))?;
        registry.register(Box::new(transactions_protected.clone()))?;
        registry.register(Box::new(transactions_failed.clone()))?;
        registry.register(Box::new(mev_detected.clone()))?;
        registry.register(Box::new(mev_prevented.clone()))?;
        registry.register(Box::new(mev_value_captured.clone()))?;
        registry.register(Box::new(mev_value_distributed.clone()))?;
        registry.register(Box::new(encryption_latency.clone()))?;
        registry.register(Box::new(ordering_latency.clone()))?;
        registry.register(Box::new(detection_latency.clone()))?;
        registry.register(Box::new(active_builders.clone()))?;
        registry.register(Box::new(mempool_size.clone()))?;
        registry.register(Box::new(pending_distributions.clone()))?;
        
        Ok(Self {
            registry,
            transactions_processed,
            transactions_protected,
            transactions_failed,
            mev_detected,
            mev_prevented,
            mev_value_captured,
            mev_value_distributed,
            encryption_latency,
            ordering_latency,
            detection_latency,
            active_builders,
            mempool_size,
            pending_distributions,
            alert_system: AlertSystem::new(),
        })
    }
    
    /// Record transaction processed
    pub fn record_transaction_processed(&self) {
        self.transactions_processed.inc();
    }
    
    /// Record transaction protected
    pub fn record_transaction_protected(&self) {
        self.transactions_protected.inc();
    }
    
    /// Record transaction failed
    pub fn record_transaction_failed(&self) {
        self.transactions_failed.inc();
    }
    
    /// Record MEV detected
    pub fn record_mev_detected(&self) {
        self.mev_detected.inc();
    }
    
    /// Record MEV prevented
    pub fn record_mev_prevented(&self) {
        self.mev_prevented.inc();
    }
    
    /// Record MEV value captured
    pub fn record_mev_value_captured(&self, value: U256) {
        self.mev_value_captured.inc_by(value.as_u64() as f64);
    }
    
    /// Record MEV value distributed
    pub fn record_mev_value_distributed(&self, value: U256) {
        self.mev_value_distributed.inc_by(value.as_u64() as f64);
    }
    
    /// Record encryption latency
    pub fn record_encryption_latency(&self, duration: Duration) {
        self.encryption_latency.observe(duration.as_secs_f64());
    }
    
    /// Record ordering latency
    pub fn record_ordering_latency(&self, duration: Duration) {
        self.ordering_latency.observe(duration.as_secs_f64());
    }
    
    /// Record detection latency
    pub fn record_detection_latency(&self, duration: Duration) {
        self.detection_latency.observe(duration.as_secs_f64());
    }
    
    /// Update active builders count
    pub fn update_active_builders(&self, count: usize) {
        self.active_builders.set(count as f64);
    }
    
    /// Update mempool size
    pub fn update_mempool_size(&self, size: usize) {
        self.mempool_size.set(size as f64);
    }
    
    /// Update pending distributions
    pub fn update_pending_distributions(&self, count: usize) {
        self.pending_distributions.set(count as f64);
    }
    
    /// Export metrics in Prometheus format
    pub fn export_metrics(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        String::from_utf8(buffer).map_err(|e| anyhow!("Failed to encode metrics: {}", e))
    }
    
    /// Start metrics exporter
    pub async fn start_exporter(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(10));
        
        loop {
            interval.tick().await;
            
            match self.export_metrics() {
                Ok(metrics) => {
                    // In production, this would expose metrics via HTTP endpoint
                    // For now, just log that metrics are available
                    info!("📊 Metrics updated");
                }
                Err(e) => {
                    error!("Failed to export metrics: {}", e);
                }
            }
        }
    }
}

impl AlertSystem {
    pub fn new() -> Self {
        Self {
            alerts: Arc::new(RwLock::new(Vec::new())),
            alert_handlers: Vec::new(),
        }
    }
    
    /// Add an alert handler
    pub fn add_handler(&mut self, handler: Box<dyn AlertHandler + Send + Sync>) {
        self.alert_handlers.push(handler);
    }
    
    /// Send an alert
    pub async fn send_alert(&self, alert: SecurityAlert) -> Result<()> {
        // Store alert
        {
            let mut alerts = self.alerts.write().await;
            alerts.push(alert.clone());
            
            // Keep only last 1000 alerts
            if alerts.len() > 1000 {
                alerts.drain(0..alerts.len() - 1000);
            }
        }
        
        // Process alert with all handlers
        for handler in &self.alert_handlers {
            if let Err(e) = handler.handle_alert(&alert).await {
                error!("Alert handler error: {}", e);
            }
        }
        
        // Log based on severity
        match alert.severity {
            AlertSeverity::Low => info!("ℹ️ Alert: {}", alert.message),
            AlertSeverity::Medium => warn!("⚠️ Alert: {}", alert.message),
            AlertSeverity::High => error!("🚨 Alert: {}", alert.message),
            AlertSeverity::Critical => error!("🔴 CRITICAL Alert: {}", alert.message),
        }
        
        Ok(())
    }
    
    /// Send multiple alerts
    pub async fn send_alerts(&self, alerts: &[SecurityAlert]) -> Result<()> {
        for alert in alerts {
            self.send_alert(alert.clone()).await?;
        }
        Ok(())
    }
    
    /// Get recent alerts
    pub async fn get_recent_alerts(&self, count: usize) -> Result<Vec<SecurityAlert>> {
        let alerts = self.alerts.read().await;
        let start = alerts.len().saturating_sub(count);
        Ok(alerts[start..].to_vec())
    }
    
    /// Clear alerts
    pub async fn clear_alerts(&self) -> Result<()> {
        let mut alerts = self.alerts.write().await;
        alerts.clear();
        Ok(())
    }
}

/// Console Alert Handler (for development)
pub struct ConsoleAlertHandler;

#[async_trait]
impl AlertHandler for ConsoleAlertHandler {
    async fn handle_alert(&self, alert: &SecurityAlert) -> Result<()> {
        println!("🚨 Alert: {:?}", alert);
        Ok(())
    }
}

/// Webhook Alert Handler
pub struct WebhookAlertHandler {
    webhook_url: String,
    client: reqwest::Client,
}

impl WebhookAlertHandler {
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AlertHandler for WebhookAlertHandler {
    async fn handle_alert(&self, alert: &SecurityAlert) -> Result<()> {
        let payload = serde_json::json!({
            "severity": format!("{:?}", alert.severity),
            "type": format!("{:?}", alert.alert_type),
            "message": alert.message,
            "details": alert.details,
            "timestamp": alert.timestamp.to_rfc3339(),
        });
        
        self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?;
        
        Ok(())
    }
}

/// Security Monitor
pub struct SecurityMonitor {
    metrics_collector: Arc<MetricsCollector>,
    threat_detector: ThreatDetector,
}

/// Threat Detector
pub struct ThreatDetector;

#[derive(Clone, Debug)]
pub struct SecurityMetrics {
    pub failed_decryptions: u64,
    pub suspicious_transactions: u64,
    pub rate_limit_violations: u64,
    pub authentication_failures: u64,
    pub unusual_gas_patterns: u64,
}

#[derive(Clone, Debug)]
pub struct Threat {
    pub severity: ThreatSeverity,
    pub threat_type: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum ThreatSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl SecurityMonitor {
    pub fn new(metrics_collector: Arc<MetricsCollector>) -> Self {
        Self {
            metrics_collector,
            threat_detector: ThreatDetector,
        }
    }
    
    pub async fn monitor_transaction_flow(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(10));
        
        loop {
            interval.tick().await;
            
            // Collect security metrics
            let metrics = self.collect_security_metrics().await?;
            
            // Detect threats
            let threats = self.threat_detector.analyze_metrics(&metrics).await?;
            
            // Send alerts for high-severity threats
            for threat in threats {
                if threat.severity >= ThreatSeverity::High {
                    let alert = SecurityAlert {
                        severity: match threat.severity {
                            ThreatSeverity::Low => AlertSeverity::Low,
                            ThreatSeverity::Medium => AlertSeverity::Medium,
                            ThreatSeverity::High => AlertSeverity::High,
                            ThreatSeverity::Critical => AlertSeverity::Critical,
                        },
                        alert_type: AlertType::AnomalousActivity,
                        message: threat.description,
                        details: HashMap::new(),
                        timestamp: chrono::Utc::now(),
                    };
                    
                    self.metrics_collector.alert_system.send_alert(alert).await?;
                }
            }
        }
    }
    
    async fn collect_security_metrics(&self) -> Result<SecurityMetrics> {
        // TODO: Implement actual metric collection
        Ok(SecurityMetrics {
            failed_decryptions: 0,
            suspicious_transactions: 0,
            rate_limit_violations: 0,
            authentication_failures: 0,
            unusual_gas_patterns: 0,
        })
    }
}

impl ThreatDetector {
    pub async fn analyze_metrics(&self, metrics: &SecurityMetrics) -> Result<Vec<Threat>> {
        let mut threats = Vec::new();
        
        if metrics.failed_decryptions > 10 {
            threats.push(Threat {
                severity: ThreatSeverity::High,
                threat_type: "DecryptionFailure".to_string(),
                description: format!("High number of failed decryptions: {}", metrics.failed_decryptions),
            });
        }
        
        if metrics.suspicious_transactions > 5 {
            threats.push(Threat {
                severity: ThreatSeverity::Medium,
                threat_type: "SuspiciousActivity".to_string(),
                description: format!("Suspicious transactions detected: {}", metrics.suspicious_transactions),
            });
        }
        
        if metrics.rate_limit_violations > 100 {
            threats.push(Threat {
                severity: ThreatSeverity::Low,
                threat_type: "RateLimitViolation".to_string(),
                description: format!("Rate limit violations: {}", metrics.rate_limit_violations),
            });
        }
        
        Ok(threats)
    }
}
