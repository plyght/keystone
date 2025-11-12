use keystone::audit::{AuditAction, AuditLogger};
use tempfile::TempDir;

#[test]
fn test_audit_logging() {
    let logger = AuditLogger::new().unwrap();
    
    logger
        .log(
            "TEST_SECRET".to_string(),
            "test".to_string(),
            Some("test_service".to_string()),
            AuditAction::Rotate,
            true,
            Some("***test".to_string()),
        )
        .unwrap();
    
    let entries = logger
        .read_logs(Some("TEST_SECRET".to_string()), Some("test".to_string()), None)
        .unwrap();
    
    assert!(entries.len() > 0);
    
    let entry = &entries[0];
    assert_eq!(entry.secret_name, "TEST_SECRET");
    assert_eq!(entry.env, "test");
    assert_eq!(entry.service, Some("test_service".to_string()));
    assert!(entry.success);
    
    assert!(logger.verify_entry(entry).unwrap());
}

#[test]
fn test_audit_signature_verification() {
    let logger = AuditLogger::new().unwrap();
    
    logger
        .log(
            "VERIFY_SECRET".to_string(),
            "test".to_string(),
            None,
            AuditAction::Rollback,
            true,
            None,
        )
        .unwrap();
    
    let entries = logger
        .read_logs(Some("VERIFY_SECRET".to_string()), None, None)
        .unwrap();
    
    assert!(entries.len() > 0);
    let entry = &entries[0];
    
    assert!(logger.verify_entry(entry).unwrap());
    
    let mut tampered = entry.clone();
    tampered.secret_name = "TAMPERED".to_string();
    
    assert!(!logger.verify_entry(&tampered).unwrap());
}

#[test]
fn test_audit_filter_by_env() {
    let logger = AuditLogger::new().unwrap();
    
    logger
        .log(
            "FILTER_SECRET".to_string(),
            "dev".to_string(),
            None,
            AuditAction::Rotate,
            true,
            None,
        )
        .unwrap();
    
    logger
        .log(
            "FILTER_SECRET".to_string(),
            "prod".to_string(),
            None,
            AuditAction::Rotate,
            true,
            None,
        )
        .unwrap();
    
    let dev_entries = logger
        .read_logs(Some("FILTER_SECRET".to_string()), Some("dev".to_string()), None)
        .unwrap();
    
    assert!(dev_entries.iter().all(|e| e.env == "dev"));
    
    let prod_entries = logger
        .read_logs(Some("FILTER_SECRET".to_string()), Some("prod".to_string()), None)
        .unwrap();
    
    assert!(prod_entries.iter().all(|e| e.env == "prod"));
}

#[test]
fn test_audit_limit() {
    let logger = AuditLogger::new().unwrap();
    
    for i in 0..10 {
        logger
            .log(
                format!("LIMIT_SECRET_{}", i),
                "test".to_string(),
                None,
                AuditAction::Rotate,
                true,
                None,
            )
            .unwrap();
    }
    
    let entries = logger.read_logs(None, Some("test".to_string()), Some(5)).unwrap();
    
    assert_eq!(entries.len(), 5);
}

