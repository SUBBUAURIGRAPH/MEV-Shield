//! SQL Injection Protection Module
//!
//! Comprehensive SQL injection prevention with parameterized queries,
//! input sanitization, and query validation.

use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, warn, error};
use crate::validation::ValidationError;

/// SQL injection protection service
pub struct SqlProtection {
    injection_patterns: Vec<Regex>,
    dangerous_keywords: Vec<String>,
    allowed_operators: Vec<String>,
}

impl SqlProtection {
    pub fn new() -> Self {
        Self {
            injection_patterns: Self::compile_injection_patterns(),
            dangerous_keywords: Self::get_dangerous_keywords(),
            allowed_operators: Self::get_allowed_operators(),
        }
    }
    
    /// Compile comprehensive SQL injection detection patterns
    fn compile_injection_patterns() -> Vec<Regex> {
        let patterns = [
            // Classic SQL injection patterns
            r"(?i)(\'\s*(;|--|\|)\s*)",
            r"(?i)(\'\s*or\s*\'\s*=\s*\')",
            r"(?i)(\'\s*and\s*\'\s*=\s*\')",
            r"(?i)(\'\s*or\s*1\s*=\s*1)",
            r"(?i)(\'\s*or\s*\d+\s*=\s*\d+)",
            
            // Union-based injections
            r"(?i)\bunion\s+(all\s+)?select\b",
            r"(?i)\bunion\s+select\b",
            
            // Database-specific injections
            r"(?i)\b(exec|execute|sp_|xp_)\s*\(",
            r"(?i)\bxp_cmdshell\b",
            r"(?i)\bsp_executesql\b",
            r"(?i)\bsp_makewebtask\b",
            
            // Comment-based injections
            r"(?i)(\/\*.*?\*\/)",
            r"(?i)(--\s)",
            r"(?i)(#.*$)",
            
            // Blind SQL injection patterns
            r"(?i)\bwaitfor\s+delay\b",
            r"(?i)\bsleep\s*\(",
            r"(?i)\bbenchmark\s*\(",
            
            // Schema discovery patterns
            r"(?i)\binformation_schema\b",
            r"(?i)\bsys\.tables\b",
            r"(?i)\bsys\.columns\b",
            r"(?i)\buser_tables\b",
            
            // Data manipulation patterns
            r"(?i)\b(insert|update|delete|drop|create|alter|truncate)\s+",
            r"(?i)\bdrop\s+table\b",
            r"(?i)\bdrop\s+database\b",
            r"(?i)\bdelete\s+from\b",
            
            // Function-based injections
            r"(?i)\b(load_file|into\s+outfile|into\s+dumpfile)\b",
            r"(?i)\bchar\s*\(\s*\d+\s*\)",
            r"(?i)\bascii\s*\(",
            r"(?i)\bsubstr\s*\(",
            r"(?i)\bmid\s*\(",
            
            // Hexadecimal and encoding patterns
            r"(?i)0x[0-9a-f]+",
            r"(?i)\\x[0-9a-f]{2}",
            
            // Time-based patterns
            r"(?i)\bif\s*\(\s*\d+\s*=\s*\d+.*sleep",
            r"(?i)\bcase\s+when.*then.*sleep",
            
            // Error-based patterns
            r"(?i)\bcast\s*\(.*\bas\s+(int|decimal|numeric)\s*\)",
            r"(?i)\bconvert\s*\(.*,\s*(int|decimal|numeric)\s*\)",
            
            // Advanced evasion patterns
            r"(?i)(%27|%22|%2b|%2d|%3b|%7c)",
            r"(?i)(concat\s*\(|group_concat\s*\()",
            
            // Database fingerprinting
            r"(?i)\b@@version\b",
            r"(?i)\buser\s*\(\s*\)",
            r"(?i)\bdatabase\s*\(\s*\)",
            r"(?i)\bversion\s*\(\s*\)",
        ];
        
        patterns.iter()
            .filter_map(|pattern| {
                match Regex::new(pattern) {
                    Ok(regex) => Some(regex),
                    Err(e) => {
                        error!("Failed to compile SQL injection pattern {}: {}", pattern, e);
                        None
                    }
                }
            })
            .collect()
    }
    
    /// Get list of dangerous SQL keywords
    fn get_dangerous_keywords() -> Vec<String> {
        vec![
            "select".to_string(), "union".to_string(), "insert".to_string(),
            "update".to_string(), "delete".to_string(), "drop".to_string(),
            "create".to_string(), "alter".to_string(), "exec".to_string(),
            "execute".to_string(), "sp_".to_string(), "xp_".to_string(),
            "information_schema".to_string(), "sys".to_string(),
            "master".to_string(), "msdb".to_string(), "tempdb".to_string(),
            "database".to_string(), "table".to_string(), "column".to_string(),
            "schema".to_string(), "procedure".to_string(), "function".to_string(),
            "trigger".to_string(), "view".to_string(), "index".to_string(),
            "constraint".to_string(), "grant".to_string(), "revoke".to_string(),
            "truncate".to_string(), "load_file".to_string(), "outfile".to_string(),
            "dumpfile".to_string(), "into".to_string(), "from".to_string(),
            "where".to_string(), "having".to_string(), "group".to_string(),
            "order".to_string(), "limit".to_string(), "offset".to_string(),
            "waitfor".to_string(), "delay".to_string(), "sleep".to_string(),
            "benchmark".to_string(), "substring".to_string(), "mid".to_string(),
            "char".to_string(), "ascii".to_string(), "hex".to_string(),
            "unhex".to_string(), "concat".to_string(), "length".to_string(),
            "count".to_string(), "sum".to_string(), "avg".to_string(),
            "min".to_string(), "max".to_string(), "cast".to_string(),
            "convert".to_string(), "if".to_string(), "case".to_string(),
            "when".to_string(), "then".to_string(), "else".to_string(),
            "end".to_string(), "declare".to_string(), "set".to_string(),
            "open".to_string(), "close".to_string(), "fetch".to_string(),
            "cursor".to_string(), "begin".to_string(), "commit".to_string(),
            "rollback".to_string(), "transaction".to_string(),
        ]
    }
    
    /// Get list of allowed operators for safe queries
    fn get_allowed_operators() -> Vec<String> {
        vec![
            "=".to_string(), "!=".to_string(), "<>".to_string(),
            "<".to_string(), ">".to_string(), "<=".to_string(),
            ">=".to_string(), "like".to_string(), "in".to_string(),
            "between".to_string(), "is".to_string(), "null".to_string(),
            "not".to_string(), "and".to_string(), "or".to_string(),
        ]
    }
    
    /// Validate input for SQL injection attempts
    pub fn validate_sql_input(&self, input: &str, context: &str) -> Result<(), ValidationError> {
        debug!("Validating SQL input for context: {}", context);
        
        // Check against injection patterns
        for pattern in &self.injection_patterns {
            if pattern.is_match(input) {
                warn!("SQL injection pattern detected in {}: pattern matched", context);
                return Err(ValidationError::SqlInjection(
                    format!("Potential SQL injection in {}", context)
                ));
            }
        }
        
        // Check for dangerous keywords
        let input_lower = input.to_lowercase();
        for keyword in &self.dangerous_keywords {
            if input_lower.contains(keyword) {
                warn!("Dangerous SQL keyword '{}' detected in {}", keyword, context);
                return Err(ValidationError::SqlInjection(
                    format!("Dangerous SQL keyword '{}' detected in {}", keyword, context)
                ));
            }
        }
        
        // Check for suspicious character combinations
        self.check_suspicious_characters(input, context)?;
        
        Ok(())
    }
    
    /// Check for suspicious character combinations
    fn check_suspicious_characters(&self, input: &str, context: &str) -> Result<(), ValidationError> {
        // Multiple quote patterns
        if input.matches('\'').count() > 2 || input.matches('"').count() > 2 {
            warn!("Multiple quotes detected in {}", context);
            return Err(ValidationError::SqlInjection(
                format!("Suspicious quote patterns in {}", context)
            ));
        }
        
        // Semicolon followed by SQL keywords
        if input.contains(';') {
            let parts: Vec<&str> = input.split(';').collect();
            for part in parts.iter().skip(1) {
                let trimmed = part.trim().to_lowercase();
                if self.dangerous_keywords.iter().any(|keyword| trimmed.starts_with(keyword)) {
                    warn!("Semicolon followed by SQL keyword in {}", context);
                    return Err(ValidationError::SqlInjection(
                        format!("Suspicious semicolon usage in {}", context)
                    ));
                }
            }
        }
        
        // Comment patterns
        if input.contains("--") || input.contains("/*") || input.contains("*/") {
            warn!("SQL comment patterns detected in {}", context);
            return Err(ValidationError::SqlInjection(
                format!("SQL comment patterns in {}", context)
            ));
        }
        
        Ok(())
    }
    
    /// Sanitize input for safe SQL usage (last resort - prefer parameterized queries)
    pub fn sanitize_sql_input(&self, input: &str) -> String {
        let mut sanitized = input.to_string();
        
        // Escape single quotes
        sanitized = sanitized.replace('\'', "''");
        
        // Remove or escape dangerous characters
        sanitized = sanitized.replace(';', "");
        sanitized = sanitized.replace("--", "");
        sanitized = sanitized.replace("/*", "");
        sanitized = sanitized.replace("*/", "");
        
        // Remove null bytes
        sanitized = sanitized.replace('\0', "");
        
        // Limit length
        if sanitized.len() > 255 {
            sanitized.truncate(255);
        }
        
        debug!("Sanitized SQL input: {} chars", sanitized.len());
        sanitized
    }
    
    /// Generate parameterized query with safe placeholders
    pub fn create_parameterized_query(&self, base_query: &str, params: &[&str]) -> Result<String, ValidationError> {
        // Validate base query doesn't contain dangerous patterns
        self.validate_sql_input(base_query, "base_query")?;
        
        // Count placeholders
        let placeholder_count = base_query.matches('?').count();
        if placeholder_count != params.len() {
            return Err(ValidationError::InvalidFormat(
                format!("Parameter count mismatch: {} placeholders, {} parameters", 
                       placeholder_count, params.len())
            ));
        }
        
        // Validate all parameters
        for (i, param) in params.iter().enumerate() {
            self.validate_sql_input(param, &format!("parameter_{}", i))?;
        }
        
        Ok(base_query.to_string())
    }
    
    /// Validate database connection string
    pub fn validate_connection_string(&self, conn_str: &str) -> Result<(), ValidationError> {
        // Basic validation for database connection strings
        let dangerous_patterns = [
            r"(?i);.*exec",
            r"(?i);.*drop",
            r"(?i);.*create",
            r"(?i);.*alter",
            r"(?i);.*insert",
            r"(?i);.*update",
            r"(?i);.*delete",
        ];
        
        for pattern_str in &dangerous_patterns {
            let pattern = Regex::new(pattern_str).unwrap();
            if pattern.is_match(conn_str) {
                return Err(ValidationError::SqlInjection(
                    "Dangerous pattern in connection string".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Advanced pattern detection using machine learning-like heuristics
    pub fn advanced_detection(&self, input: &str) -> f64 {
        let mut risk_score = 0.0;
        
        // Character frequency analysis
        let single_quote_ratio = input.matches('\'').count() as f64 / input.len() as f64;
        if single_quote_ratio > 0.1 {
            risk_score += 0.3;
        }
        
        // Keyword density
        let input_lower = input.to_lowercase();
        let keyword_count = self.dangerous_keywords.iter()
            .filter(|&keyword| input_lower.contains(keyword))
            .count();
        
        risk_score += (keyword_count as f64) * 0.1;
        
        // Suspicious character patterns
        if input.contains("'='") || input.contains("1=1") || input.contains("0=0") {
            risk_score += 0.4;
        }
        
        // Union patterns
        if input_lower.contains("union") && input_lower.contains("select") {
            risk_score += 0.5;
        }
        
        // Comment patterns
        if input.contains("--") || input.contains("/*") {
            risk_score += 0.3;
        }
        
        // Encoding patterns
        if input.contains('%') && (input.contains("27") || input.contains("22")) {
            risk_score += 0.4;
        }
        
        risk_score.min(1.0)
    }
    
    /// Create whitelist-based validator for specific use cases
    pub fn create_whitelist_validator(&self, allowed_patterns: Vec<String>) -> WhitelistValidator {
        WhitelistValidator::new(allowed_patterns)
    }
}

/// Whitelist-based validator for restricted input scenarios
pub struct WhitelistValidator {
    allowed_patterns: Vec<Regex>,
}

impl WhitelistValidator {
    pub fn new(patterns: Vec<String>) -> Self {
        let allowed_patterns = patterns.iter()
            .filter_map(|pattern| Regex::new(pattern).ok())
            .collect();
        
        Self { allowed_patterns }
    }
    
    /// Validate input against whitelist
    pub fn validate(&self, input: &str) -> Result<(), ValidationError> {
        if self.allowed_patterns.is_empty() {
            return Ok(()); // No restrictions
        }
        
        for pattern in &self.allowed_patterns {
            if pattern.is_match(input) {
                return Ok(());
            }
        }
        
        Err(ValidationError::InvalidFormat(
            "Input does not match allowed patterns".to_string()
        ))
    }
}

/// SQL query builder with automatic injection prevention
pub struct SafeQueryBuilder {
    protection: SqlProtection,
    query_parts: Vec<String>,
    parameters: Vec<String>,
}

impl SafeQueryBuilder {
    pub fn new() -> Self {
        Self {
            protection: SqlProtection::new(),
            query_parts: Vec::new(),
            parameters: Vec::new(),
        }
    }
    
    /// Add SELECT clause
    pub fn select(&mut self, fields: &[&str]) -> Result<&mut Self, ValidationError> {
        // Validate field names
        for field in fields {
            self.protection.validate_sql_input(field, "select_field")?;
        }
        
        self.query_parts.push(format!("SELECT {}", fields.join(", ")));
        Ok(self)
    }
    
    /// Add FROM clause
    pub fn from(&mut self, table: &str) -> Result<&mut Self, ValidationError> {
        self.protection.validate_sql_input(table, "table_name")?;
        self.query_parts.push(format!("FROM {}", table));
        Ok(self)
    }
    
    /// Add WHERE clause with parameterized conditions
    pub fn where_eq(&mut self, field: &str, value: &str) -> Result<&mut Self, ValidationError> {
        self.protection.validate_sql_input(field, "where_field")?;
        self.protection.validate_sql_input(value, "where_value")?;
        
        self.query_parts.push(format!("WHERE {} = ?", field));
        self.parameters.push(value.to_string());
        Ok(self)
    }
    
    /// Build the final query
    pub fn build(&self) -> Result<(String, Vec<String>), ValidationError> {
        if self.query_parts.is_empty() {
            return Err(ValidationError::InvalidFormat("Empty query".to_string()));
        }
        
        let query = self.query_parts.join(" ");
        Ok((query, self.parameters.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sql_injection_detection() {
        let protection = SqlProtection::new();
        
        // Valid inputs
        assert!(protection.validate_sql_input("user123", "username").is_ok());
        assert!(protection.validate_sql_input("user@example.com", "email").is_ok());
        
        // SQL injection attempts
        assert!(protection.validate_sql_input("'; DROP TABLE users; --", "test").is_err());
        assert!(protection.validate_sql_input("' OR '1'='1", "test").is_err());
        assert!(protection.validate_sql_input("admin' UNION SELECT * FROM users --", "test").is_err());
        assert!(protection.validate_sql_input("1; exec xp_cmdshell('dir')", "test").is_err());
    }
    
    #[test]
    fn test_parameterized_query_creation() {
        let protection = SqlProtection::new();
        
        let query = "SELECT * FROM users WHERE username = ? AND password = ?";
        let params = vec!["admin", "password123"];
        
        assert!(protection.create_parameterized_query(query, &params).is_ok());
        
        // Mismatched parameter count
        let params_wrong = vec!["admin"];
        assert!(protection.create_parameterized_query(query, &params_wrong).is_err());
    }
    
    #[test]
    fn test_advanced_detection() {
        let protection = SqlProtection::new();
        
        // High risk inputs
        assert!(protection.advanced_detection("' OR 1=1 --") > 0.5);
        assert!(protection.advanced_detection("UNION SELECT password FROM users") > 0.5);
        
        // Low risk inputs
        assert!(protection.advanced_detection("normal_username") < 0.3);
        assert!(protection.advanced_detection("user@example.com") < 0.3);
    }
    
    #[test]
    fn test_query_builder() {
        let mut builder = SafeQueryBuilder::new();
        
        let result = builder
            .select(&["id", "username", "email"])
            .unwrap()
            .from("users")
            .unwrap()
            .where_eq("username", "test_user")
            .unwrap()
            .build();
        
        assert!(result.is_ok());
        let (query, params) = result.unwrap();
        assert!(query.contains("SELECT id, username, email"));
        assert!(query.contains("FROM users"));
        assert!(query.contains("WHERE username = ?"));
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], "test_user");
    }
    
    #[test]
    fn test_whitelist_validator() {
        let patterns = vec![r"^[a-zA-Z0-9_]+$".to_string()]; // Only alphanumeric and underscore
        let validator = WhitelistValidator::new(patterns);
        
        // Valid inputs
        assert!(validator.validate("user123").is_ok());
        assert!(validator.validate("test_user").is_ok());
        
        // Invalid inputs
        assert!(validator.validate("user'; DROP TABLE").is_err());
        assert!(validator.validate("user@example.com").is_err()); // Contains @
    }
    
    #[test]
    fn test_sanitization() {
        let protection = SqlProtection::new();
        
        let input = "user'; DROP TABLE users; --";
        let sanitized = protection.sanitize_sql_input(input);
        
        assert!(!sanitized.contains(';'));
        assert!(!sanitized.contains("--"));
        assert_eq!(sanitized, "user'' DROP TABLE users ");
    }
}