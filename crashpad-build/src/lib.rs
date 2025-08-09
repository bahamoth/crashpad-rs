//! Shared build logic for crashpad-rs
//!
//! This crate provides the configuration and build phases for building
//! Google Crashpad. It's used by both crashpad-sys build.rs and xtask.

pub mod config;
pub mod phases;

pub use config::BuildConfig;
pub use phases::BuildPhases;
