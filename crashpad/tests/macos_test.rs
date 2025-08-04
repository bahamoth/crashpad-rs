#[cfg(target_os = "macos")]
#[cfg(test)]
mod macos_tests {
    use crashpad::{CrashpadClient, Result};
    
    #[test]
    fn test_mach_service() -> Result<()> {
        let client = CrashpadClient::new()?;
        
        // This should fail because the service doesn't exist, but it tests the API
        let result = client.set_handler_mach_service("com.example.nonexistent");
        assert!(result.is_err());
        
        Ok(())
    }
    
    #[test]
    fn test_system_default_handler() -> Result<()> {
        let client = CrashpadClient::new()?;
        
        // This sets up the system default handler
        // It should succeed on macOS
        client.use_system_default_handler()?;
        
        Ok(())
    }
}