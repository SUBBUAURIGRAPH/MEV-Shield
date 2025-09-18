//! Input Sanitization Module
//!
//! Comprehensive input sanitization utilities for preventing XSS,
//! path traversal, and other injection attacks.

use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, warn};
use crate::validation::ValidationError;

/// HTML/XSS sanitization service
pub struct HtmlSanitizer {
    dangerous_tags: Vec<String>,
    dangerous_attributes: Vec<String>,
    allowed_tags: Vec<String>,
    allowed_attributes: Vec<String>,
    url_protocols: Vec<String>,
}

impl HtmlSanitizer {
    pub fn new() -> Self {
        Self {
            dangerous_tags: Self::get_dangerous_tags(),
            dangerous_attributes: Self::get_dangerous_attributes(),
            allowed_tags: Self::get_allowed_tags(),
            allowed_attributes: Self::get_allowed_attributes(),
            url_protocols: Self::get_safe_url_protocols(),
        }
    }
    
    /// Get list of dangerous HTML tags
    fn get_dangerous_tags() -> Vec<String> {
        vec![
            "script".to_string(), "iframe".to_string(), "object".to_string(),
            "embed".to_string(), "applet".to_string(), "form".to_string(),
            "input".to_string(), "button".to_string(), "select".to_string(),
            "textarea".to_string(), "option".to_string(), "link".to_string(),
            "meta".to_string(), "style".to_string(), "base".to_string(),
            "frame".to_string(), "frameset".to_string(), "noframes".to_string(),
            "noscript".to_string(), "xml".to_string(), "import".to_string(),
            "video".to_string(), "audio".to_string(), "source".to_string(),
            "track".to_string(), "canvas".to_string(), "svg".to_string(),
        ]
    }
    
    /// Get list of dangerous HTML attributes
    fn get_dangerous_attributes() -> Vec<String> {
        vec![
            "onload".to_string(), "onerror".to_string(), "onclick".to_string(),
            "onmouseover".to_string(), "onmouseout".to_string(), "onfocus".to_string(),
            "onblur".to_string(), "onchange".to_string(), "onsubmit".to_string(),
            "onreset".to_string(), "onselect".to_string(), "onkeydown".to_string(),
            "onkeypress".to_string(), "onkeyup".to_string(), "onabort".to_string(),
            "oncanplay".to_string(), "oncanplaythrough".to_string(),
            "ondurationchange".to_string(), "onemptied".to_string(),
            "onended".to_string(), "onloadeddata".to_string(),
            "onloadedmetadata".to_string(), "onloadstart".to_string(),
            "onpause".to_string(), "onplay".to_string(), "onplaying".to_string(),
            "onprogress".to_string(), "onratechange".to_string(),
            "onseeked".to_string(), "onseeking".to_string(), "onstalled".to_string(),
            "onsuspend".to_string(), "ontimeupdate".to_string(),
            "onvolumechange".to_string(), "onwaiting".to_string(),
            "href".to_string(), "src".to_string(), "action".to_string(),
            "formaction".to_string(), "data".to_string(), "codebase".to_string(),
            "cite".to_string(), "background".to_string(), "poster".to_string(),
            "manifest".to_string(), "xmlns".to_string(),
        ]
    }
    
    /// Get list of allowed HTML tags (for basic formatting)
    fn get_allowed_tags() -> Vec<String> {
        vec![
            "p".to_string(), "br".to_string(), "strong".to_string(),
            "em".to_string(), "u".to_string(), "i".to_string(),
            "b".to_string(), "span".to_string(), "div".to_string(),
            "h1".to_string(), "h2".to_string(), "h3".to_string(),
            "h4".to_string(), "h5".to_string(), "h6".to_string(),
            "ul".to_string(), "ol".to_string(), "li".to_string(),
            "blockquote".to_string(), "pre".to_string(), "code".to_string(),
        ]
    }
    
    /// Get list of allowed HTML attributes
    fn get_allowed_attributes() -> Vec<String> {
        vec![
            "class".to_string(), "id".to_string(), "title".to_string(),
            "alt".to_string(), "lang".to_string(), "dir".to_string(),
            "style".to_string(), // Only if CSS is sanitized separately
        ]
    }
    
    /// Get list of safe URL protocols
    fn get_safe_url_protocols() -> Vec<String> {
        vec![
            "http".to_string(), "https".to_string(), "ftp".to_string(),
            "ftps".to_string(), "mailto".to_string(),
        ]
    }
    
    /// Sanitize HTML content
    pub fn sanitize_html(&self, input: &str) -> Result<String, ValidationError> {
        debug!("Sanitizing HTML content: {} chars", input.len());
        
        let mut sanitized = input.to_string();
        
        // Remove dangerous tags
        for tag in &self.dangerous_tags {
            let patterns = [
                format!(r"(?i)<{}\b[^>]*>.*?</{}>", tag, tag),
                format!(r"(?i)<{}\b[^>]*/>", tag),
                format!(r"(?i)<{}\b[^>]*>", tag),
            ];
            
            for pattern in &patterns {
                if let Ok(regex) = Regex::new(pattern) {
                    sanitized = regex.replace_all(&sanitized, "").to_string();
                }
            }
        }
        
        // Remove dangerous attributes
        for attr in &self.dangerous_attributes {
            let pattern = format!(r"(?i)\b{}=([\"'])[^\"']*\1", attr);
            if let Ok(regex) = Regex::new(&pattern) {
                sanitized = regex.replace_all(&sanitized, "").to_string();
            }
        }
        
        // Remove javascript: and data: URLs
        let js_pattern = Regex::new(r"(?i)(javascript|vbscript|data|livescript|mocha|jscript|ecmascript)\s*:").unwrap();
        sanitized = js_pattern.replace_all(&sanitized, "blocked:").to_string();
        
        // Remove CSS expressions
        let css_pattern = Regex::new(r"(?i)expression\s*\(").unwrap();
        sanitized = css_pattern.replace_all(&sanitized, "blocked(").to_string();
        
        // Encode HTML entities
        sanitized = self.encode_html_entities(&sanitized);
        
        Ok(sanitized)
    }
    
    /// Encode HTML entities
    fn encode_html_entities(&self, input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
            .replace('/', "&#x2F;")
    }
    
    /// Strict sanitization (remove all HTML)
    pub fn strip_html(&self, input: &str) -> String {
        debug!("Stripping all HTML from input: {} chars", input.len());
        
        // Remove all HTML tags
        let tag_pattern = Regex::new(r"<[^>]*>").unwrap();
        let no_tags = tag_pattern.replace_all(input, "");
        
        // Decode HTML entities and then re-encode dangerous ones
        let decoded = self.decode_html_entities(&no_tags);
        self.encode_html_entities(&decoded)
    }
    
    /// Decode common HTML entities
    fn decode_html_entities(&self, input: &str) -> String {
        input
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#x27;", "'")
            .replace("&#x2F;", "/")
            .replace("&nbsp;", " ")
    }
    
    /// Validate and sanitize URL
    pub fn sanitize_url(&self, url: &str) -> Result<String, ValidationError> {
        debug!("Sanitizing URL: {}", url);
        
        // Basic URL validation
        if url.is_empty() {
            return Ok(String::new());
        }
        
        // Check for dangerous protocols
        let lower_url = url.to_lowercase();
        if lower_url.starts_with("javascript:") || 
           lower_url.starts_with("vbscript:") ||
           lower_url.starts_with("data:") ||
           lower_url.starts_with("livescript:") ||
           lower_url.starts_with("mocha:") ||
           lower_url.starts_with("jscript:") {
            return Err(ValidationError::InvalidFormat(
                "Dangerous URL protocol detected".to_string()
            ));
        }
        
        // Validate protocol if present
        if url.contains(':') {
            let protocol = url.split(':').next().unwrap_or("").to_lowercase();
            if !self.url_protocols.contains(&protocol) && !protocol.is_empty() {
                warn!("Potentially unsafe URL protocol: {}", protocol);
                return Err(ValidationError::InvalidFormat(
                    format!("Unsafe URL protocol: {}", protocol)
                ));
            }
        }
        
        // Remove null bytes and control characters
        let sanitized = url.chars()
            .filter(|c| !c.is_control() || *c == '\t' || *c == '\n' || *c == '\r')
            .collect::<String>();
        
        // Length validation
        if sanitized.len() > 2048 {
            return Err(ValidationError::TooLong(sanitized.len(), 2048));
        }
        
        Ok(sanitized)
    }
}

/// Path traversal protection
pub struct PathSanitizer {
    dangerous_patterns: Vec<Regex>,
}

impl PathSanitizer {
    pub fn new() -> Self {
        Self {
            dangerous_patterns: Self::compile_path_patterns(),
        }
    }
    
    /// Compile path traversal detection patterns
    fn compile_path_patterns() -> Vec<Regex> {
        let patterns = [
            r"\.\.[\\/]",                    // Directory traversal
            r"[\\/]\.\.[\\/]",               // Directory traversal with slashes
            r"\.\.\\",                       // Windows path traversal
            r"\.\./",                        // Unix path traversal
            r"[<>:\"|?*]",                   // Windows forbidden characters
            r"\0",                           // Null byte
            r"^[\\/]",                       // Absolute path
            r"^[a-zA-Z]:",                   // Windows drive letter
            r"~[\\/]",                       // Home directory reference
            r"\$\{.*\}",                     // Variable expansion
            r"%[0-9a-fA-F]{2}",             // URL encoding
            r"\\x[0-9a-fA-F]{2}",           // Hex encoding
        ];
        
        patterns.iter()
            .filter_map(|pattern| Regex::new(pattern).ok())
            .collect()
    }
    
    /// Sanitize file path
    pub fn sanitize_path(&self, path: &str) -> Result<String, ValidationError> {
        debug!("Sanitizing file path: {}", path);
        
        // Check against dangerous patterns
        for pattern in &self.dangerous_patterns {
            if pattern.is_match(path) {
                warn!("Dangerous path pattern detected: {}", path);
                return Err(ValidationError::InvalidFormat(
                    "Path contains dangerous patterns".to_string()
                ));
            }
        }
        
        // Remove null bytes and control characters
        let sanitized = path.chars()
            .filter(|c| !c.is_control())
            .collect::<String>();
        
        // Length validation
        if sanitized.len() > 255 {
            return Err(ValidationError::TooLong(sanitized.len(), 255));
        }
        
        // Ensure it's a relative path only
        let normalized = sanitized.trim_start_matches('/').trim_start_matches('\\');
        
        Ok(normalized.to_string())
    }
    
    /// Validate filename only (no path separators)
    pub fn validate_filename(&self, filename: &str) -> Result<String, ValidationError> {
        debug!("Validating filename: {}", filename);
        
        // Check for path separators
        if filename.contains('/') || filename.contains('\\') {
            return Err(ValidationError::InvalidFormat(
                "Filename cannot contain path separators".to_string()
            ));
        }
        
        // Check for reserved names (Windows)
        let reserved_names = [
            "CON", "PRN", "AUX", "NUL",
            "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
            "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ];
        
        let upper_filename = filename.to_uppercase();
        if reserved_names.contains(&upper_filename.as_str()) {
            return Err(ValidationError::InvalidFormat(
                "Reserved filename not allowed".to_string()
            ));
        }
        
        // Check for dangerous characters
        let forbidden_chars = ['<', '>', ':', '"', '|', '?', '*', '\0'];
        for ch in forbidden_chars {
            if filename.contains(ch) {
                return Err(ValidationError::InvalidCharacters(
                    format!("Forbidden character '{}' in filename", ch)
                ));
            }
        }
        
        // Length validation
        if filename.len() > 255 {
            return Err(ValidationError::TooLong(filename.len(), 255));
        }
        
        if filename.is_empty() {
            return Err(ValidationError::TooShort(0, 1));
        }
        
        Ok(filename.to_string())
    }
}

/// Text content sanitizer
pub struct TextSanitizer;

impl TextSanitizer {
    /// Sanitize plain text input
    pub fn sanitize_text(&self, input: &str, max_length: usize) -> Result<String, ValidationError> {
        debug!("Sanitizing text input: {} chars", input.len());
        
        // Length validation
        if input.len() > max_length {
            return Err(ValidationError::TooLong(input.len(), max_length));
        }
        
        // Remove null bytes and most control characters
        let sanitized = input.chars()
            .filter(|c| !c.is_control() || *c == '\t' || *c == '\n' || *c == '\r')
            .collect::<String>();
        
        // Normalize whitespace
        let normalized = self.normalize_whitespace(&sanitized);
        
        Ok(normalized)
    }
    
    /// Normalize whitespace (remove excessive spaces, normalize line endings)
    fn normalize_whitespace(&self, input: &str) -> String {
        // Normalize line endings to \n
        let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
        
        // Remove excessive whitespace
        let space_pattern = Regex::new(r" {2,}").unwrap();
        let no_excess_spaces = space_pattern.replace_all(&normalized, " ");
        
        // Remove excessive newlines
        let newline_pattern = Regex::new(r"\n{3,}").unwrap();
        let no_excess_newlines = newline_pattern.replace_all(&no_excess_spaces, "\n\n");
        
        no_excess_newlines.trim().to_string()
    }
    
    /// Sanitize and validate email address
    pub fn sanitize_email(&self, email: &str) -> Result<String, ValidationError> {
        debug!("Sanitizing email address");
        
        // Basic email regex
        let email_pattern = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        
        let trimmed = email.trim().to_lowercase();
        
        if !email_pattern.is_match(&trimmed) {
            return Err(ValidationError::InvalidFormat(
                "Invalid email format".to_string()
            ));
        }
        
        // Length validation
        if trimmed.len() > 320 {
            return Err(ValidationError::TooLong(trimmed.len(), 320));
        }
        
        // Check for dangerous characters
        if trimmed.contains('<') || trimmed.contains('>') || trimmed.contains('"') {
            return Err(ValidationError::InvalidCharacters(
                "Email contains forbidden characters".to_string()
            ));
        }
        
        Ok(trimmed)
    }
    
    /// Sanitize username
    pub fn sanitize_username(&self, username: &str) -> Result<String, ValidationError> {
        debug!("Sanitizing username");
        
        let trimmed = username.trim();
        
        // Length validation
        if trimmed.len() < 3 {
            return Err(ValidationError::TooShort(trimmed.len(), 3));
        }
        
        if trimmed.len() > 50 {
            return Err(ValidationError::TooLong(trimmed.len(), 50));
        }
        
        // Username pattern (alphanumeric, underscore, hyphen)
        let username_pattern = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
        
        if !username_pattern.is_match(trimmed) {
            return Err(ValidationError::InvalidFormat(
                "Username can only contain letters, numbers, underscore, and hyphen".to_string()
            ));
        }
        
        // Cannot start or end with hyphen/underscore
        if trimmed.starts_with('-') || trimmed.starts_with('_') ||
           trimmed.ends_with('-') || trimmed.ends_with('_') {
            return Err(ValidationError::InvalidFormat(
                "Username cannot start or end with hyphen or underscore".to_string()
            ));
        }
        
        Ok(trimmed.to_string())
    }
}

/// All-in-one sanitizer combining all sanitization methods
pub struct ComprehensiveSanitizer {
    html_sanitizer: HtmlSanitizer,
    path_sanitizer: PathSanitizer,
    text_sanitizer: TextSanitizer,
}

impl ComprehensiveSanitizer {
    pub fn new() -> Self {
        Self {
            html_sanitizer: HtmlSanitizer::new(),
            path_sanitizer: PathSanitizer::new(),
            text_sanitizer: TextSanitizer,
        }
    }
    
    /// Auto-detect input type and apply appropriate sanitization
    pub fn auto_sanitize(&self, input: &str, context: &str) -> Result<String, ValidationError> {
        match context {
            "html" | "content" | "description" => self.html_sanitizer.sanitize_html(input),
            "text" | "plaintext" => self.text_sanitizer.sanitize_text(input, 1000),
            "path" | "filepath" => self.path_sanitizer.sanitize_path(input),
            "filename" => self.path_sanitizer.validate_filename(input),
            "email" => self.text_sanitizer.sanitize_email(input),
            "username" => self.text_sanitizer.sanitize_username(input),
            "url" => self.html_sanitizer.sanitize_url(input),
            _ => {
                // Default to text sanitization
                self.text_sanitizer.sanitize_text(input, 1000)
            }
        }
    }
    
    /// Bulk sanitization for multiple inputs
    pub fn sanitize_map(&self, inputs: HashMap<String, String>) -> Result<HashMap<String, String>, ValidationError> {
        let mut sanitized = HashMap::new();
        
        for (key, value) in inputs {
            let sanitized_value = self.auto_sanitize(&value, &key)?;
            sanitized.insert(key, sanitized_value);
        }
        
        Ok(sanitized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_html_sanitization() {
        let sanitizer = HtmlSanitizer::new();
        
        // XSS attempts
        let xss_input = "<script>alert('xss')</script>";
        let sanitized = sanitizer.sanitize_html(xss_input).unwrap();
        assert!(!sanitized.contains("<script>"));
        
        // Dangerous attributes
        let onclick_input = "<div onclick='alert(1)'>Click me</div>";
        let sanitized = sanitizer.sanitize_html(onclick_input).unwrap();
        assert!(!sanitized.contains("onclick"));
        
        // JavaScript URLs
        let js_url = "<a href='javascript:alert(1)'>Link</a>";
        let sanitized = sanitizer.sanitize_html(js_url).unwrap();
        assert!(!sanitized.contains("javascript:"));
    }
    
    #[test]
    fn test_path_sanitization() {
        let sanitizer = PathSanitizer::new();
        
        // Path traversal attempts
        assert!(sanitizer.sanitize_path("../../../etc/passwd").is_err());
        assert!(sanitizer.sanitize_path("..\\..\\windows\\system32").is_err());
        assert!(sanitizer.sanitize_path("/etc/passwd").is_err());
        
        // Valid relative paths
        assert!(sanitizer.sanitize_path("documents/file.txt").is_ok());
        assert!(sanitizer.sanitize_path("uploads/image.jpg").is_ok());
    }
    
    #[test]
    fn test_filename_validation() {
        let sanitizer = PathSanitizer::new();
        
        // Valid filenames
        assert!(sanitizer.validate_filename("document.pdf").is_ok());
        assert!(sanitizer.validate_filename("image_001.jpg").is_ok());
        
        // Invalid filenames
        assert!(sanitizer.validate_filename("../file.txt").is_err());
        assert!(sanitizer.validate_filename("file/name.txt").is_err());
        assert!(sanitizer.validate_filename("CON").is_err());
        assert!(sanitizer.validate_filename("file<name>.txt").is_err());
    }
    
    #[test]
    fn test_text_sanitization() {
        let sanitizer = TextSanitizer;
        
        // Normal text
        let normal = "Hello, world!";
        assert_eq!(sanitizer.sanitize_text(normal, 100).unwrap(), normal);
        
        // Text with excessive whitespace
        let whitespace = "Hello,    world!\n\n\n\nNew paragraph.";
        let sanitized = sanitizer.sanitize_text(whitespace, 100).unwrap();
        assert!(!sanitized.contains("    "));
        assert!(!sanitized.contains("\n\n\n"));
    }
    
    #[test]
    fn test_email_sanitization() {
        let sanitizer = TextSanitizer;
        
        // Valid emails
        assert!(sanitizer.sanitize_email("user@example.com").is_ok());
        assert!(sanitizer.sanitize_email("test.email+tag@domain.co.uk").is_ok());
        
        // Invalid emails
        assert!(sanitizer.sanitize_email("invalid-email").is_err());
        assert!(sanitizer.sanitize_email("user@").is_err());
        assert!(sanitizer.sanitize_email("@domain.com").is_err());
        assert!(sanitizer.sanitize_email("user<script>@domain.com").is_err());
    }
    
    #[test]
    fn test_username_sanitization() {
        let sanitizer = TextSanitizer;
        
        // Valid usernames
        assert!(sanitizer.sanitize_username("user123").is_ok());
        assert!(sanitizer.sanitize_username("test_user").is_ok());
        assert!(sanitizer.sanitize_username("user-name").is_ok());
        
        // Invalid usernames
        assert!(sanitizer.sanitize_username("us").is_err()); // Too short
        assert!(sanitizer.sanitize_username("_username").is_err()); // Starts with underscore
        assert!(sanitizer.sanitize_username("username_").is_err()); // Ends with underscore
        assert!(sanitizer.sanitize_username("user@name").is_err()); // Invalid character
    }
    
    #[test]
    fn test_url_sanitization() {
        let sanitizer = HtmlSanitizer::new();
        
        // Valid URLs
        assert!(sanitizer.sanitize_url("https://example.com").is_ok());
        assert!(sanitizer.sanitize_url("http://test.org/path").is_ok());
        assert!(sanitizer.sanitize_url("mailto:user@example.com").is_ok());
        
        // Invalid URLs
        assert!(sanitizer.sanitize_url("javascript:alert(1)").is_err());
        assert!(sanitizer.sanitize_url("vbscript:msgbox(1)").is_err());
        assert!(sanitizer.sanitize_url("data:text/html,<script>alert(1)</script>").is_err());
    }
    
    #[test]
    fn test_comprehensive_sanitizer() {
        let sanitizer = ComprehensiveSanitizer::new();
        
        // Test auto-detection
        assert!(sanitizer.auto_sanitize("<script>alert(1)</script>", "html").is_ok());
        assert!(sanitizer.auto_sanitize("../../../etc/passwd", "path").is_err());
        assert!(sanitizer.auto_sanitize("user@example.com", "email").is_ok());
        
        // Test map sanitization
        let mut inputs = HashMap::new();
        inputs.insert("email".to_string(), "user@example.com".to_string());
        inputs.insert("html".to_string(), "<p>Hello</p>".to_string());
        
        let result = sanitizer.sanitize_map(inputs);
        assert!(result.is_ok());
    }
}