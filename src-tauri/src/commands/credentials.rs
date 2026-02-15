//! Credential commands: master password and debrid API key management

use crate::state::AppState;
use crate::debrid::types::DebridProviderType;
use crate::crypto::{self, CryptoManager};
use std::collections::HashMap;
use tauri::State;

/// Check if master password is set
#[tauri::command]
pub async fn check_master_password_set(state: State<'_, AppState>) -> Result<bool, String> {
    state.database
        .has_master_password()
        .map_err(|e| format!("Failed to check master password: {}", e))
}

/// Set master password (first time setup)
#[tauri::command]
pub async fn set_master_password(
    password: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    tracing::info!("Setting master password");

    // Check if already set
    if state.database.has_master_password()
        .map_err(|e| format!("Failed to check existing password: {}", e))?
    {
        return Err("Master password already set. Use change_master_password instead.".to_string());
    }

    // Create password data
    let salt = crypto::generate_salt();
    let password_hash = crypto::hash_master_password(&password, &salt)
        .map_err(|e| format!("Failed to hash password: {}", e))?;

    let password_data = crate::database::MasterPasswordData {
        password_hash,
        salt,
    };

    // Save to database
    state.database
        .save_master_password(&password_data)
        .map_err(|e| format!("Failed to save password: {}", e))?;

    // Cache password in memory
    let mut cached_password = state.master_password.write().await;
    *cached_password = Some(password);

    tracing::info!("Master password set successfully");
    Ok(())
}

/// Unlock debrid services with master password
#[tauri::command]
pub async fn unlock_with_master_password(
    password: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    tracing::info!("Attempting to unlock with master password");

    // Load password data
    let password_data = state.database
        .load_master_password()
        .map_err(|e| format!("Failed to load password: {}", e))?
        .ok_or_else(|| "Master password not set".to_string())?;

    // Verify password
    let is_valid = crypto::verify_master_password(&password, &password_data.password_hash)
        .map_err(|e| format!("Failed to verify password: {}", e))?;

    if is_valid {
        // Cache password in memory
        let mut cached_password = state.master_password.write().await;
        *cached_password = Some(password);

        tracing::info!("Master password verified and cached");
        Ok(true)
    } else {
        tracing::warn!("Invalid master password attempt");
        Ok(false)
    }
}

/// Change master password
#[tauri::command]
pub async fn change_master_password(
    old_password: String,
    new_password: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    tracing::info!("Attempting to change master password");

    // Load current password data
    let password_data = state.database
        .load_master_password()
        .map_err(|e| format!("Failed to load password: {}", e))?
        .ok_or_else(|| "Master password not set".to_string())?;

    // Verify old password
    let is_valid = crypto::verify_master_password(&old_password, &password_data.password_hash)
        .map_err(|e| format!("Failed to verify password: {}", e))?;

    if !is_valid {
        return Err("Invalid old password".to_string());
    }

    // Load all credentials with old password
    let old_credentials = state.database
        .load_all_debrid_credentials()
        .map_err(|e| format!("Failed to load credentials: {}", e))?;

    // Decrypt all API keys with old password
    let mut decrypted_keys: HashMap<DebridProviderType, String> = HashMap::new();
    for cred in old_credentials {
        let old_crypto = CryptoManager::from_password(&old_password, &password_data.salt)
            .map_err(|e| format!("Failed to create crypto manager: {}", e))?;
        let api_key = old_crypto.decrypt(&cred.api_key_encrypted, &cred.nonce)
            .map_err(|e| format!("Failed to decrypt credentials for {}: {}", cred.provider.as_str(), e))?;
        decrypted_keys.insert(cred.provider, api_key);
    }

    // Create new password hash
    let new_salt = crypto::generate_salt();
    let new_password_hash = crypto::hash_master_password(&new_password, &new_salt)
        .map_err(|e| format!("Failed to hash new password: {}", e))?;

    // Re-encrypt all API keys with new password
    let new_crypto = CryptoManager::from_password(&new_password, &new_salt)
        .map_err(|e| format!("Failed to create crypto manager: {}", e))?;

    for (provider, api_key) in decrypted_keys {
        let (encrypted_api_key, nonce) = new_crypto.encrypt(&api_key)
            .map_err(|e| format!("Failed to encrypt credentials for {}: {}", provider.as_str(), e))?;

        let new_cred = crate::database::DebridCredentials {
            provider,
            api_key_encrypted: encrypted_api_key,
            nonce,
            created_at: chrono::Utc::now().timestamp(),
            last_validated: 0,
            is_valid: false,
        };

        state.database
            .save_debrid_credentials(&new_cred)
            .map_err(|e| format!("Failed to save re-encrypted credentials: {}", e))?;
    }

    // Save new password
    let new_password_data = crate::database::MasterPasswordData {
        password_hash: new_password_hash,
        salt: new_salt,
    };

    state.database
        .save_master_password(&new_password_data)
        .map_err(|e| format!("Failed to save new password: {}", e))?;

    // Update cached password
    let mut cached_password = state.master_password.write().await;
    *cached_password = Some(new_password);

    tracing::info!("Master password changed successfully");
    Ok(())
}

/// Lock debrid services (clear cached password)
#[tauri::command]
pub async fn lock_debrid_services(state: State<'_, AppState>) -> Result<(), String> {
    tracing::info!("Locking debrid services");

    let mut cached_password = state.master_password.write().await;
    *cached_password = None;

    Ok(())
}

/// Save debrid provider credentials
#[tauri::command]
pub async fn save_debrid_credentials(
    provider: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    tracing::info!("Saving credentials for provider: {}", provider);

    let provider_type = super::parse_provider(&provider)?;

    // Get cached master password
    let cached_password = state.master_password.read().await;
    let master_password = cached_password.as_ref()
        .ok_or_else(|| "Master password not unlocked. Please unlock first.".to_string())?;

    // Load master password data for salt
    let password_data = state.database
        .load_master_password()
        .map_err(|e| format!("Failed to load password data: {}", e))?
        .ok_or_else(|| "Master password not set".to_string())?;

    // Encrypt API key
    let crypto_manager = CryptoManager::from_password(master_password, &password_data.salt)
        .map_err(|e| format!("Failed to create crypto manager: {}", e))?;
    let (encrypted_api_key, nonce) = crypto_manager.encrypt(&api_key)
        .map_err(|e| format!("Failed to encrypt API key: {}", e))?;

    tracing::debug!("Successfully encrypted API key for {}", provider);

    // Create credentials struct
    let credentials = crate::database::DebridCredentials {
        provider: provider_type,
        api_key_encrypted: encrypted_api_key,
        nonce,
        created_at: chrono::Utc::now().timestamp(),
        last_validated: 0,
        is_valid: false,
    };

    // Save to database
    state.database
        .save_debrid_credentials(&credentials)
        .map_err(|e| {
            tracing::error!("Failed to save to database: {}", e);
            format!("Failed to save credentials: {}", e)
        })?;

    tracing::info!("Saved to database, now initializing provider");

    // Initialize provider in DebridManager
    let mut debrid_manager = state.debrid_manager.write().await;
    debrid_manager.initialize_provider(provider_type, api_key)
        .await
        .map_err(|e| {
            tracing::error!("Failed to initialize provider: {}", e);
            format!("Failed to initialize provider: {}", e)
        })?;

    tracing::info!("Credentials saved successfully for {}", provider);
    Ok(())
}

/// Get status of all configured debrid credentials
#[tauri::command]
pub async fn get_debrid_credentials_status(
    state: State<'_, AppState>,
) -> Result<Vec<super::CredentialStatus>, String> {
    let all_credentials = state.database
        .load_all_debrid_credentials()
        .map_err(|e| format!("Failed to load credentials: {}", e))?;

    let mut statuses = Vec::new();

    for cred in all_credentials {
        statuses.push(super::CredentialStatus {
            provider: cred.provider.as_str().to_string(),
            is_configured: true,
            is_valid: None,
            last_validated: Some(cred.last_validated),
        });
    }

    Ok(statuses)
}

/// Delete debrid provider credentials
#[tauri::command]
pub async fn delete_debrid_credentials(
    provider: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    tracing::info!("Deleting credentials for provider: {}", provider);

    let provider_type = super::parse_provider(&provider)?;

    state.database
        .delete_debrid_credentials(provider_type)
        .map_err(|e| format!("Failed to delete credentials: {}", e))?;

    tracing::info!("Credentials deleted for {}", provider);
    Ok(())
}

/// Validate debrid provider credentials
#[tauri::command]
pub async fn validate_debrid_provider(
    provider: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    tracing::info!("Validating credentials for provider: {}", provider);

    let provider_type = super::parse_provider(&provider)?;

    // Get cached master password
    let cached_password = state.master_password.read().await;
    let master_password = cached_password.as_ref()
        .ok_or_else(|| "Master password not unlocked. Please unlock first.".to_string())?;

    // Load credentials
    let credentials = state.database
        .load_debrid_credentials(provider_type)
        .map_err(|e| {
            tracing::error!("Failed to load credentials for {}: {}", provider, e);
            format!("Failed to load credentials: {}", e)
        })?
        .ok_or_else(|| {
            tracing::warn!("No credentials found for {}", provider);
            format!("No credentials found for {}", provider)
        })?;

    // Load master password data for salt
    let password_data = state.database
        .load_master_password()
        .map_err(|e| format!("Failed to load password data: {}", e))?
        .ok_or_else(|| "Master password not set".to_string())?;

    // Decrypt API key
    let crypto_manager = CryptoManager::from_password(master_password, &password_data.salt)
        .map_err(|e| format!("Failed to create crypto manager: {}", e))?;
    let api_key = crypto_manager.decrypt(&credentials.api_key_encrypted, &credentials.nonce)
        .map_err(|e| format!("Failed to decrypt API key: {}", e))?;

    tracing::debug!("Validating {} with API (first 10 chars: {}...)", provider, &api_key.chars().take(10).collect::<String>());

    // Validate with provider
    let debrid_manager = state.debrid_manager.read().await;
    let is_valid = debrid_manager.validate_provider(provider_type, &api_key)
        .await
        .map_err(|e| {
            tracing::error!("Validation failed for {}: {}", provider, e);
            format!("Validation failed: {}", e)
        })?;

    tracing::info!("Validation result for {}: {}", provider, is_valid);

    if is_valid {
        // Update last_validated timestamp
        let mut updated_cred = credentials;
        updated_cred.last_validated = chrono::Utc::now().timestamp();
        updated_cred.is_valid = true;
        state.database
            .save_debrid_credentials(&updated_cred)
            .map_err(|e| format!("Failed to update validation timestamp: {}", e))?;

        tracing::info!("Updated validation timestamp for {}", provider);
    }

    Ok(is_valid)
}
