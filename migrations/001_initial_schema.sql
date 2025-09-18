-- MEV Shield Database Schema
-- Version: 1.0.0

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Encrypted transactions table
CREATE TABLE IF NOT EXISTS encrypted_transactions (
    id BYTEA PRIMARY KEY,
    encrypted_data BYTEA NOT NULL,
    submission_time TIMESTAMP WITH TIME ZONE NOT NULL,
    unlock_time TIMESTAMP WITH TIME ZONE,
    priority INTEGER NOT NULL,
    gas_price NUMERIC(78, 0) NOT NULL,
    chain_id INTEGER NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_encrypted_tx_unlock_time ON encrypted_transactions(unlock_time);
CREATE INDEX idx_encrypted_tx_priority_submission ON encrypted_transactions(priority DESC, submission_time ASC);
CREATE INDEX idx_encrypted_tx_chain_status ON encrypted_transactions(chain_id, status);

-- Decryption shares table
CREATE TABLE IF NOT EXISTS decryption_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id BYTEA NOT NULL REFERENCES encrypted_transactions(id) ON DELETE CASCADE,
    validator_index INTEGER NOT NULL,
    share_data BYTEA NOT NULL,
    signature BYTEA NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(transaction_id, validator_index)
);

CREATE INDEX idx_decryption_shares_tx ON decryption_shares(transaction_id);

-- Block proposals table
CREATE TABLE IF NOT EXISTS block_proposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    block_hash BYTEA NOT NULL,
    builder_address BYTEA NOT NULL,
    transaction_count INTEGER NOT NULL,
    mev_protection_proof JSONB NOT NULL,
    signature BYTEA NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    finalized_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_block_proposals_status ON block_proposals(status);
CREATE INDEX idx_block_proposals_builder ON block_proposals(builder_address);

-- Block builders table
CREATE TABLE IF NOT EXISTS block_builders (
    address BYTEA PRIMARY KEY,
    reputation_score NUMERIC(5, 2) NOT NULL DEFAULT 50.00,
    blocks_built BIGINT NOT NULL DEFAULT 0,
    blocks_accepted BIGINT NOT NULL DEFAULT 0,
    stake_amount NUMERIC(78, 0) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT false,
    last_active TIMESTAMP WITH TIME ZONE,
    registered_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_builders_active ON block_builders(is_active);
CREATE INDEX idx_builders_reputation ON block_builders(reputation_score DESC);

-- MEV incidents table
CREATE TABLE IF NOT EXISTS mev_incidents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_type VARCHAR(50) NOT NULL,
    confidence_score NUMERIC(5, 4) NOT NULL,
    affected_transactions JSONB NOT NULL,
    evidence JSONB NOT NULL,
    severity VARCHAR(20) NOT NULL,
    detected_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    resolved BOOLEAN DEFAULT false,
    resolved_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_mev_incidents_type ON mev_incidents(incident_type);
CREATE INDEX idx_mev_incidents_severity ON mev_incidents(severity);
CREATE INDEX idx_mev_incidents_detected ON mev_incidents(detected_at DESC);

-- MEV pool table
CREATE TABLE IF NOT EXISTS mev_pool (
    id SERIAL PRIMARY KEY,
    epoch BIGINT NOT NULL UNIQUE,
    total_captured NUMERIC(78, 0) NOT NULL DEFAULT 0,
    available_for_distribution NUMERIC(78, 0) NOT NULL DEFAULT 0,
    distributed_this_epoch NUMERIC(78, 0) NOT NULL DEFAULT 0,
    reserved_for_gas NUMERIC(78, 0) NOT NULL DEFAULT 0,
    last_distribution TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_mev_pool_epoch ON mev_pool(epoch DESC);

-- User contributions table
CREATE TABLE IF NOT EXISTS user_contributions (
    address BYTEA PRIMARY KEY,
    total_gas_used NUMERIC(78, 0) NOT NULL DEFAULT 0,
    transaction_count BIGINT NOT NULL DEFAULT 0,
    value_contributed NUMERIC(78, 0) NOT NULL DEFAULT 0,
    accumulated_rewards NUMERIC(78, 0) NOT NULL DEFAULT 0,
    last_activity TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_user_contributions_activity ON user_contributions(last_activity DESC);

-- Distributions table
CREATE TABLE IF NOT EXISTS distributions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    epoch BIGINT NOT NULL,
    recipient BYTEA NOT NULL,
    amount NUMERIC(78, 0) NOT NULL,
    reason VARCHAR(50) NOT NULL,
    transaction_hash BYTEA,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    processed_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_distributions_epoch ON distributions(epoch);
CREATE INDEX idx_distributions_recipient ON distributions(recipient);
CREATE INDEX idx_distributions_status ON distributions(status);

-- Security alerts table
CREATE TABLE IF NOT EXISTS security_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    severity VARCHAR(20) NOT NULL,
    alert_type VARCHAR(50) NOT NULL,
    message TEXT NOT NULL,
    details JSONB,
    handled BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    handled_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_security_alerts_severity ON security_alerts(severity);
CREATE INDEX idx_security_alerts_type ON security_alerts(alert_type);
CREATE INDEX idx_security_alerts_created ON security_alerts(created_at DESC);

-- System metrics table (for historical data)
CREATE TABLE IF NOT EXISTS system_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_name VARCHAR(100) NOT NULL,
    metric_value NUMERIC NOT NULL,
    labels JSONB,
    recorded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_system_metrics_name ON system_metrics(metric_name);
CREATE INDEX idx_system_metrics_recorded ON system_metrics(recorded_at DESC);

-- API access logs table
CREATE TABLE IF NOT EXISTS api_access_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    ip_address INET,
    user_agent TEXT,
    request_body JSONB,
    response_status INTEGER,
    response_time_ms INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_api_logs_endpoint ON api_access_logs(endpoint);
CREATE INDEX idx_api_logs_created ON api_access_logs(created_at DESC);

-- Configuration table
CREATE TABLE IF NOT EXISTS configuration (
    key VARCHAR(255) PRIMARY KEY,
    value JSONB NOT NULL,
    description TEXT,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_by VARCHAR(255)
);

-- Create update timestamp trigger
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply update timestamp trigger to relevant tables
CREATE TRIGGER update_encrypted_transactions_updated_at
    BEFORE UPDATE ON encrypted_transactions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_block_builders_updated_at
    BEFORE UPDATE ON block_builders
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_mev_pool_updated_at
    BEFORE UPDATE ON mev_pool
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_user_contributions_updated_at
    BEFORE UPDATE ON user_contributions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

-- Insert initial configuration
INSERT INTO configuration (key, value, description) VALUES
    ('system.version', '"1.0.0"', 'System version'),
    ('encryption.threshold', '67', 'Percentage of validators required for decryption'),
    ('redistribution.percentage', '80', 'Percentage of MEV to redistribute'),
    ('detection.confidence_threshold', '0.8', 'MEV detection confidence threshold')
ON CONFLICT (key) DO NOTHING;

-- Create views for analytics
CREATE OR REPLACE VIEW mev_analytics AS
SELECT 
    DATE_TRUNC('hour', detected_at) as hour,
    incident_type,
    severity,
    COUNT(*) as incident_count,
    AVG(confidence_score) as avg_confidence
FROM mev_incidents
GROUP BY DATE_TRUNC('hour', detected_at), incident_type, severity;

CREATE OR REPLACE VIEW builder_performance AS
SELECT 
    address,
    reputation_score,
    blocks_built,
    blocks_accepted,
    CASE 
        WHEN blocks_built > 0 THEN blocks_accepted::NUMERIC / blocks_built::NUMERIC * 100
        ELSE 0 
    END as acceptance_rate,
    stake_amount,
    is_active,
    last_active
FROM block_builders
ORDER BY reputation_score DESC;

CREATE OR REPLACE VIEW distribution_summary AS
SELECT 
    epoch,
    COUNT(DISTINCT recipient) as recipient_count,
    SUM(amount) as total_distributed,
    AVG(amount) as avg_distribution,
    MIN(created_at) as distribution_start,
    MAX(processed_at) as distribution_end
FROM distributions
WHERE status = 'completed'
GROUP BY epoch
ORDER BY epoch DESC;

-- Grant permissions (adjust as needed)
-- GRANT SELECT ON ALL TABLES IN SCHEMA public TO mevshield_reader;
-- GRANT ALL ON ALL TABLES IN SCHEMA public TO mevshield_admin;

-- Add comments for documentation
COMMENT ON TABLE encrypted_transactions IS 'Stores encrypted transaction data before decryption and ordering';
COMMENT ON TABLE block_builders IS 'Registered block builders with reputation and performance metrics';
COMMENT ON TABLE mev_incidents IS 'Detected MEV attacks and suspicious activities';
COMMENT ON TABLE mev_pool IS 'MEV value captured and available for redistribution';
COMMENT ON TABLE user_contributions IS 'User activity and contribution tracking for MEV redistribution';
COMMENT ON TABLE distributions IS 'MEV redistribution records to users';

-- Create initial indexes for performance
ANALYZE;
