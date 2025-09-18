/**
 * Frontend Input Sanitization and XSS Prevention Utilities
 * 
 * Comprehensive client-side input validation and sanitization using DOMPurify
 * and custom validation rules for MEV Shield dashboard security.
 */

import DOMPurify from 'dompurify';

// Configuration for different sanitization contexts
interface SanitizationConfig {
  allowedTags: string[];
  allowedAttributes: { [key: string]: string[] };
  disallowedTagsMode: 'discard' | 'escape' | 'keep';
  stripIgnoreTag: boolean;
  stripIgnoreTagBody: string[];
}

// Predefined configurations for different contexts
const SANITIZATION_CONFIGS: { [key: string]: SanitizationConfig } = {
  // Strict mode - no HTML allowed
  strict: {
    allowedTags: [],
    allowedAttributes: {},
    disallowedTagsMode: 'discard',
    stripIgnoreTag: true,
    stripIgnoreTagBody: ['script', 'style'],
  },
  
  // Basic formatting only
  basic: {
    allowedTags: ['p', 'br', 'strong', 'em', 'u', 'i', 'b'],
    allowedAttributes: {},
    disallowedTagsMode: 'discard',
    stripIgnoreTag: true,
    stripIgnoreTagBody: ['script', 'style'],
  },
  
  // Rich text with safe tags
  rich: {
    allowedTags: [
      'p', 'br', 'strong', 'em', 'u', 'i', 'b', 'span', 'div',
      'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
      'ul', 'ol', 'li', 'blockquote', 'pre', 'code'
    ],
    allowedAttributes: {
      '*': ['class', 'id'],
      'span': ['style'],
      'div': ['style']
    },
    disallowedTagsMode: 'discard',
    stripIgnoreTag: true,
    stripIgnoreTagBody: ['script', 'style'],
  }
};

/**
 * Input validation error class
 */
export class ValidationError extends Error {
  constructor(
    message: string,
    public field?: string,
    public code?: string
  ) {
    super(message);
    this.name = 'ValidationError';
  }
}

/**
 * Main sanitization service
 */
export class InputSanitizer {
  private static instance: InputSanitizer;
  
  private constructor() {
    this.configureDOMPurify();
  }
  
  public static getInstance(): InputSanitizer {
    if (!InputSanitizer.instance) {
      InputSanitizer.instance = new InputSanitizer();
    }
    return InputSanitizer.instance;
  }
  
  /**
   * Configure DOMPurify with security-focused settings
   */
  private configureDOMPurify(): void {
    // Add hooks for additional security
    DOMPurify.addHook('beforeSanitizeElements', (node) => {
      // Log suspicious attempts
      if (node.tagName && ['SCRIPT', 'IFRAME', 'OBJECT', 'EMBED'].includes(node.tagName)) {
        console.warn('Blocked dangerous tag:', node.tagName);
      }
    });
    
    DOMPurify.addHook('beforeSanitizeAttributes', (node) => {
      // Remove dangerous event handlers
      for (const attr of node.attributes || []) {
        if (attr.name.toLowerCase().startsWith('on')) {
          console.warn('Blocked event handler:', attr.name);
          node.removeAttribute(attr.name);
        }
      }
    });
  }
  
  /**
   * Sanitize HTML content based on context
   */
  public sanitizeHtml(input: string, context: 'strict' | 'basic' | 'rich' = 'strict'): string {
    if (!input || typeof input !== 'string') {
      return '';
    }
    
    const config = SANITIZATION_CONFIGS[context];
    
    return DOMPurify.sanitize(input, {
      ALLOWED_TAGS: config.allowedTags,
      ALLOWED_ATTR: Object.keys(config.allowedAttributes),
      KEEP_CONTENT: config.disallowedTagsMode === 'keep',
      STRIP_IGNORE_TAG: config.stripIgnoreTag,
      STRIP_IGNORE_TAG_BODY: config.stripIgnoreTagBody,
      USE_PROFILES: { html: true },
      SANITIZE_DOM: true,
      SANITIZE_NAMED_PROPS: true,
      FORBID_TAGS: ['script', 'iframe', 'object', 'embed', 'form', 'input', 'button'],
      FORBID_ATTR: ['onerror', 'onload', 'onclick', 'onfocus', 'onmouseover']
    });
  }
  
  /**
   * Validate and sanitize Ethereum address
   */
  public validateEthereumAddress(address: string): string {
    if (!address || typeof address !== 'string') {
      throw new ValidationError('Address is required', 'address', 'REQUIRED');
    }
    
    // Remove any HTML/XSS attempts
    const cleaned = this.sanitizeHtml(address.trim(), 'strict');
    
    // Ethereum address regex
    const ethAddressRegex = /^0x[a-fA-F0-9]{40}$/;
    
    if (!ethAddressRegex.test(cleaned)) {
      throw new ValidationError(
        'Invalid Ethereum address format',
        'address',
        'INVALID_FORMAT'
      );
    }
    
    // Check for null address
    if (cleaned.toLowerCase() === '0x0000000000000000000000000000000000000000') {
      throw new ValidationError(
        'Null address not allowed',
        'address',
        'NULL_ADDRESS'
      );
    }
    
    return cleaned.toLowerCase();
  }
  
  /**
   * Validate and sanitize numeric amounts
   */
  public validateAmount(amount: string | number, field: string = 'amount'): string {
    let cleaned: string;
    
    if (typeof amount === 'number') {
      cleaned = amount.toString();
    } else if (typeof amount === 'string') {
      cleaned = this.sanitizeHtml(amount.trim(), 'strict');
    } else {
      throw new ValidationError('Amount must be a string or number', field, 'INVALID_TYPE');
    }
    
    // Remove any non-numeric characters except decimal point
    cleaned = cleaned.replace(/[^0-9.]/g, '');
    
    // Validate numeric format
    const numericRegex = /^\d+(\.\d+)?$/;
    if (!numericRegex.test(cleaned)) {
      throw new ValidationError(
        'Invalid numeric format',
        field,
        'INVALID_FORMAT'
      );
    }
    
    // Check for reasonable bounds (prevent overflow)
    const num = parseFloat(cleaned);
    if (num < 0) {
      throw new ValidationError('Amount cannot be negative', field, 'NEGATIVE_VALUE');
    }
    
    if (num > Number.MAX_SAFE_INTEGER) {
      throw new ValidationError('Amount too large', field, 'TOO_LARGE');
    }
    
    return cleaned;
  }
  
  /**
   * Validate and sanitize transaction hash
   */
  public validateTransactionHash(hash: string): string {
    if (!hash || typeof hash !== 'string') {
      throw new ValidationError('Transaction hash is required', 'hash', 'REQUIRED');
    }
    
    const cleaned = this.sanitizeHtml(hash.trim(), 'strict');
    
    // Transaction hash regex (64 hex characters with 0x prefix)
    const hashRegex = /^0x[a-fA-F0-9]{64}$/;
    
    if (!hashRegex.test(cleaned)) {
      throw new ValidationError(
        'Invalid transaction hash format',
        'hash',
        'INVALID_FORMAT'
      );
    }
    
    return cleaned.toLowerCase();
  }
  
  /**
   * Validate and sanitize email address
   */
  public validateEmail(email: string): string {
    if (!email || typeof email !== 'string') {
      throw new ValidationError('Email is required', 'email', 'REQUIRED');
    }
    
    const cleaned = this.sanitizeHtml(email.trim().toLowerCase(), 'strict');
    
    // Basic email regex
    const emailRegex = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;
    
    if (!emailRegex.test(cleaned)) {
      throw new ValidationError(
        'Invalid email format',
        'email',
        'INVALID_FORMAT'
      );
    }
    
    // Length validation
    if (cleaned.length > 320) {
      throw new ValidationError(
        'Email address too long',
        'email',
        'TOO_LONG'
      );
    }
    
    return cleaned;
  }
  
  /**
   * Validate and sanitize username
   */
  public validateUsername(username: string): string {
    if (!username || typeof username !== 'string') {
      throw new ValidationError('Username is required', 'username', 'REQUIRED');
    }
    
    const cleaned = this.sanitizeHtml(username.trim(), 'strict');
    
    // Username validation (alphanumeric, underscore, hyphen)
    const usernameRegex = /^[a-zA-Z0-9_-]{3,50}$/;
    
    if (!usernameRegex.test(cleaned)) {
      throw new ValidationError(
        'Username must be 3-50 characters (letters, numbers, underscore, hyphen only)',
        'username',
        'INVALID_FORMAT'
      );
    }
    
    // Cannot start or end with special characters
    if (cleaned.startsWith('-') || cleaned.startsWith('_') ||
        cleaned.endsWith('-') || cleaned.endsWith('_')) {
      throw new ValidationError(
        'Username cannot start or end with underscore or hyphen',
        'username',
        'INVALID_FORMAT'
      );
    }
    
    return cleaned;
  }
  
  /**
   * Validate and sanitize URLs
   */
  public validateUrl(url: string): string {
    if (!url || typeof url !== 'string') {
      return '';
    }
    
    const cleaned = this.sanitizeHtml(url.trim(), 'strict');
    
    // Check for dangerous protocols
    const dangerousProtocols = ['javascript:', 'data:', 'vbscript:', 'livescript:'];
    const lowerUrl = cleaned.toLowerCase();
    
    for (const protocol of dangerousProtocols) {
      if (lowerUrl.startsWith(protocol)) {
        throw new ValidationError(
          'Dangerous URL protocol detected',
          'url',
          'DANGEROUS_PROTOCOL'
        );
      }
    }
    
    // Basic URL validation
    try {
      new URL(cleaned);
    } catch {
      throw new ValidationError(
        'Invalid URL format',
        'url',
        'INVALID_FORMAT'
      );
    }
    
    return cleaned;
  }
  
  /**
   * Sanitize free-form text input
   */
  public sanitizeText(text: string, maxLength: number = 1000): string {
    if (!text || typeof text !== 'string') {
      return '';
    }
    
    // Remove HTML and normalize whitespace
    let cleaned = this.sanitizeHtml(text, 'strict');
    
    // Normalize whitespace
    cleaned = cleaned.replace(/\s+/g, ' ').trim();
    
    // Length validation
    if (cleaned.length > maxLength) {
      cleaned = cleaned.substring(0, maxLength).trim();
    }
    
    return cleaned;
  }
  
  /**
   * Validate file upload
   */
  public validateFile(file: File, allowedTypes: string[] = ['image/jpeg', 'image/png', 'application/pdf']): void {
    if (!file) {
      throw new ValidationError('File is required', 'file', 'REQUIRED');
    }
    
    // File type validation
    if (!allowedTypes.includes(file.type)) {
      throw new ValidationError(
        `File type ${file.type} not allowed`,
        'file',
        'INVALID_TYPE'
      );
    }
    
    // File size validation (10MB max)
    const maxSize = 10 * 1024 * 1024;
    if (file.size > maxSize) {
      throw new ValidationError(
        'File size exceeds 10MB limit',
        'file',
        'TOO_LARGE'
      );
    }
    
    // Filename validation
    const filename = this.sanitizeText(file.name, 255);
    if (!filename || filename !== file.name) {
      throw new ValidationError(
        'Invalid filename',
        'file',
        'INVALID_FILENAME'
      );
    }
  }
  
  /**
   * Bulk validation for form data
   */
  public validateFormData(data: Record<string, any>, rules: Record<string, any>): Record<string, any> {
    const sanitized: Record<string, any> = {};
    const errors: Record<string, string> = {};
    
    for (const [field, rule] of Object.entries(rules)) {
      const value = data[field];
      
      try {
        switch (rule.type) {
          case 'email':
            sanitized[field] = this.validateEmail(value);
            break;
          case 'username':
            sanitized[field] = this.validateUsername(value);
            break;
          case 'address':
            sanitized[field] = this.validateEthereumAddress(value);
            break;
          case 'amount':
            sanitized[field] = this.validateAmount(value, field);
            break;
          case 'text':
            sanitized[field] = this.sanitizeText(value, rule.maxLength || 1000);
            break;
          case 'url':
            sanitized[field] = this.validateUrl(value);
            break;
          case 'hash':
            sanitized[field] = this.validateTransactionHash(value);
            break;
          default:
            sanitized[field] = this.sanitizeText(value);
        }
      } catch (error) {
        if (error instanceof ValidationError) {
          errors[field] = error.message;
        } else {
          errors[field] = 'Validation failed';
        }
      }
    }
    
    if (Object.keys(errors).length > 0) {
      throw new ValidationError('Form validation failed', undefined, 'FORM_ERRORS');
    }
    
    return sanitized;
  }
  
  /**
   * Create a safe innerHTML setter
   */
  public createSafeInnerHTML(html: string, context: 'strict' | 'basic' | 'rich' = 'basic') {
    return {
      __html: this.sanitizeHtml(html, context)
    };
  }
}

// Export singleton instance
export const inputSanitizer = InputSanitizer.getInstance();

// Utility functions for common use cases
export const sanitizeHtml = (input: string, context: 'strict' | 'basic' | 'rich' = 'strict') => 
  inputSanitizer.sanitizeHtml(input, context);

export const validateAddress = (address: string) => 
  inputSanitizer.validateEthereumAddress(address);

export const validateAmount = (amount: string | number) => 
  inputSanitizer.validateAmount(amount);

export const sanitizeText = (text: string, maxLength?: number) => 
  inputSanitizer.sanitizeText(text, maxLength);

export const createSafeHTML = (html: string, context: 'strict' | 'basic' | 'rich' = 'basic') =>
  inputSanitizer.createSafeInnerHTML(html, context);

// React hook for form validation
export const useFormValidation = () => {
  return {
    validateField: (value: any, type: string, options?: any) => {
      try {
        switch (type) {
          case 'email':
            return inputSanitizer.validateEmail(value);
          case 'username':
            return inputSanitizer.validateUsername(value);
          case 'address':
            return inputSanitizer.validateEthereumAddress(value);
          case 'amount':
            return inputSanitizer.validateAmount(value);
          case 'text':
            return inputSanitizer.sanitizeText(value, options?.maxLength);
          default:
            return inputSanitizer.sanitizeText(value);
        }
      } catch (error) {
        if (error instanceof ValidationError) {
          throw error;
        }
        throw new ValidationError('Validation failed');
      }
    },
    
    sanitize: (input: string, context: 'strict' | 'basic' | 'rich' = 'strict') => 
      inputSanitizer.sanitizeHtml(input, context),
  };
};

// CSP helper functions
export const generateCSPNonce = (): string => {
  const array = new Uint8Array(16);
  crypto.getRandomValues(array);
  return Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');
};

export const createCSPMetaTag = (nonce: string): string => {
  const csp = [
    "default-src 'self'",
    `script-src 'self' 'nonce-${nonce}'`,
    "style-src 'self' 'unsafe-inline'",
    "img-src 'self' data: https:",
    "font-src 'self'",
    "connect-src 'self' wss: https:",
    "frame-src 'none'",
    "object-src 'none'",
    "base-uri 'self'",
    "form-action 'self'"
  ].join('; ');
  
  return `<meta http-equiv="Content-Security-Policy" content="${csp}">`;
};