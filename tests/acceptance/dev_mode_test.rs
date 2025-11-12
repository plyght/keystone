use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_dev_mode_env_update() {
    let temp_dir = TempDir::new().unwrap();
    let env_path = temp_dir.path().join(".env");
    
    fs::write(
        &env_path,
        "# Test env file\nAPI_KEY=old_value\nOTHER_VAR=unchanged\n",
    )
    .unwrap();
    
    keystone::dev::update_env_file(
        "API_KEY",
        "new_value",
        Some(env_path.to_str().unwrap()),
    )
    .await
    .unwrap();
    
    let contents = fs::read_to_string(&env_path).unwrap();
    assert!(contents.contains("API_KEY=new_value"));
    assert!(contents.contains("OTHER_VAR=unchanged"));
    assert!(contents.contains("# Test env file"));
    
    let rollback_path = temp_dir.path().join(".keystone-rollback");
    assert!(rollback_path.exists());
    
    let rollback_contents = fs::read_to_string(&rollback_path).unwrap();
    assert!(rollback_contents.contains("API_KEY=old_value"));
}

#[tokio::test]
async fn test_dev_mode_new_secret() {
    let temp_dir = TempDir::new().unwrap();
    let env_path = temp_dir.path().join(".env");
    
    fs::write(&env_path, "EXISTING_VAR=value\n").unwrap();
    
    keystone::dev::update_env_file(
        "NEW_SECRET",
        "secret_value",
        Some(env_path.to_str().unwrap()),
    )
    .await
    .unwrap();
    
    let contents = fs::read_to_string(&env_path).unwrap();
    assert!(contents.contains("NEW_SECRET=secret_value"));
    assert!(contents.contains("EXISTING_VAR=value"));
}

#[tokio::test]
async fn test_dev_mode_preserves_formatting() {
    let temp_dir = TempDir::new().unwrap();
    let env_path = temp_dir.path().join(".env");
    
    fs::write(
        &env_path,
        "# Header comment\n\nAPI_KEY=old\n\n# Another comment\nOTHER=value\n",
    )
    .unwrap();
    
    keystone::dev::update_env_file(
        "API_KEY",
        "new",
        Some(env_path.to_str().unwrap()),
    )
    .await
    .unwrap();
    
    let contents = fs::read_to_string(&env_path).unwrap();
    assert!(contents.contains("# Header comment"));
    assert!(contents.contains("# Another comment"));
    assert!(contents.contains("API_KEY=new"));
}

