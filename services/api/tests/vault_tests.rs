use uuid::Uuid;

#[test]
fn test_vault_encryption_decryption() {
    std::env::set_var(
        "VAULT_MASTER_KEY",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    );

    let encryption = birch_api::vault::encryption::VaultEncryption::new()
        .expect("Failed to create encryption");

    let workspace_id = Uuid::new_v4();
    let plaintext = "test-secret-value";

    let encrypted = encryption
        .encrypt(&workspace_id, plaintext)
        .expect("Failed to encrypt");

    assert!(encrypted.len() > plaintext.len());

    let decrypted = encryption
        .decrypt(&workspace_id, &encrypted)
        .expect("Failed to decrypt");

    assert_eq!(decrypted, plaintext);

    let different_workspace_id = Uuid::new_v4();
    let result = encryption.decrypt(&different_workspace_id, &encrypted);

    assert!(
        result.is_err(),
        "Decryption should fail with different workspace ID"
    );
}

#[test]
fn test_credential_mode_parsing() {
    use birch_api::credentials::modes::CredentialMode;
    use std::str::FromStr;

    assert_eq!(
        CredentialMode::from_str("hosted").unwrap(),
        CredentialMode::Hosted
    );
    assert_eq!(
        CredentialMode::from_str("oauth").unwrap(),
        CredentialMode::OAuth
    );
    assert_eq!(
        CredentialMode::from_str("kms").unwrap(),
        CredentialMode::Kms
    );
    assert_eq!(
        CredentialMode::from_str("api_key").unwrap(),
        CredentialMode::ApiKey
    );
    assert!(CredentialMode::from_str("invalid").is_err());
}

#[test]
fn test_api_key_generation() {
    use birch_api::auth::api_keys::ApiKeyService;

    let key1 = ApiKeyService::generate_api_key();
    let key2 = ApiKeyService::generate_api_key();

    assert!(key1.starts_with("sk_"));
    assert!(key2.starts_with("sk_"));
    assert_ne!(key1, key2);
    assert!(key1.len() > 32);
}

#[test]
fn test_api_key_hashing_and_verification() {
    use birch_api::auth::api_keys::ApiKeyService;

    let api_key = "sk_test_key_12345";

    let hash = ApiKeyService::hash_api_key(api_key).expect("Failed to hash API key");

    assert!(ApiKeyService::verify_api_key(api_key, &hash).expect("Failed to verify API key"));

    assert!(!ApiKeyService::verify_api_key("wrong_key", &hash).expect("Failed to verify wrong key"));
}

#[test]
fn test_rbac_permissions() {
    use birch_api::workspace::models::Role;
    use birch_api::workspace::rbac::Permission;

    let owner = Role::Owner;
    assert!(owner.has_permission(Permission::Rotate));
    assert!(owner.has_permission(Permission::Workspace));
    assert!(owner.can_manage_workspace());

    let operator = Role::Operator;
    assert!(operator.has_permission(Permission::Rotate));
    assert!(!operator.has_permission(Permission::Workspace));
    assert!(!operator.can_manage_workspace());

    let viewer = Role::Viewer;
    assert!(viewer.has_permission(Permission::View));
    assert!(!viewer.has_permission(Permission::Rotate));

    let auditor = Role::Auditor;
    assert!(auditor.has_permission(Permission::Audit));
    assert!(!auditor.has_permission(Permission::Rotate));
}

#[test]
fn test_plan_tier_rotation_limits() {
    use birch_api::workspace::models::PlanTier;

    assert_eq!(PlanTier::Free.rotation_limit(), Some(100));
    assert_eq!(PlanTier::Starter.rotation_limit(), Some(1000));
    assert_eq!(PlanTier::Pro.rotation_limit(), Some(10000));
    assert_eq!(PlanTier::Enterprise.rotation_limit(), None);
}
