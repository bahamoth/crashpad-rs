//! Safe Rust bindings for Google Crashpad
//! 
//! This crate provides a safe, idiomatic Rust interface to the Crashpad crash reporting library.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrashpadError {
    #[error("Failed to initialize Crashpad")]
    InitializationFailed,
    
    #[error("Failed to start handler")]
    HandlerStartFailed,
    
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, CrashpadError>;

// TODO: Implement Client, Handler, and other core types