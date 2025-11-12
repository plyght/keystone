use keystone::lock::Lock;
use std::thread;
use std::time::Duration;

#[test]
fn test_lock_acquire_release() {
    let mut lock = Lock::new("test", "secret1").unwrap();
    
    lock.acquire("test_operation").unwrap();
    
    let mut lock2 = Lock::new("test", "secret1").unwrap();
    let result = lock2.acquire("test_operation");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Lock already held"));
    
    lock.release().unwrap();
    
    let mut lock3 = Lock::new("test", "secret1").unwrap();
    lock3.acquire("test_operation").unwrap();
    lock3.release().unwrap();
}

#[test]
fn test_lock_auto_release_on_drop() {
    {
        let mut lock = Lock::new("test", "secret2").unwrap();
        lock.acquire("test_operation").unwrap();
    }
    
    let mut lock2 = Lock::new("test", "secret2").unwrap();
    lock2.acquire("test_operation").unwrap();
    lock2.release().unwrap();
}

#[test]
fn test_lock_different_secrets() {
    let mut lock1 = Lock::new("test", "secret3").unwrap();
    lock1.acquire("test_operation").unwrap();
    
    let mut lock2 = Lock::new("test", "secret4").unwrap();
    lock2.acquire("test_operation").unwrap();
    
    lock1.release().unwrap();
    lock2.release().unwrap();
}

#[test]
fn test_lock_different_envs() {
    let mut lock1 = Lock::new("dev", "secret5").unwrap();
    lock1.acquire("test_operation").unwrap();
    
    let mut lock2 = Lock::new("prod", "secret5").unwrap();
    lock2.acquire("test_operation").unwrap();
    
    lock1.release().unwrap();
    lock2.release().unwrap();
}

