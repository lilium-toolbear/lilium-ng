use crate::Result;
use chrono::Utc;
use lilium_api_client::http::DzmmApi;
use lilium_database::DbSession;

use lilium_common::LiliumError;
use lilium_models::dzmm::account::DzmmAccount;
use tracing::instrument;

#[instrument(
    skip(account),
    fields(
        user_id = %account.user_id,
        has_email = account.email.is_some(),
        has_password = account.password.is_some(),
        has_signin_code = account.signin_code.is_some(),
        has_signin_code_image = account.signin_code_image.is_some(),
        has_cookies = account.cookies.is_some()
    )
)]
pub fn create_auth_client(account: &DzmmAccount) -> Result<DzmmApi> {
    DzmmApi::new(
        account.email.clone(),
        account.password.clone(),
        account.signin_code.clone(),
        account.signin_code_image.clone(),
        account.signin_code_image_mime.clone(),
        account.cookies.clone(),
        Some(account.user_id.clone()),
        true,
        None,
    )
    .map_err(|e| LiliumError::service("ACCOUNT_AUTH_CLIENT_BUILD_FAILED", e.to_string()))
}

#[instrument(
    skip(
        session,
        user_profile,
        email,
        password,
        signin_code,
        signin_code_image,
        signin_code_image_mime,
        cookies
    ),
    fields(
        user_id = %user_id,
        has_email = email.is_some(),
        has_password = password.is_some(),
        has_signin_code = signin_code.is_some(),
        has_signin_code_image = signin_code_image.is_some(),
        has_signin_code_image_mime = signin_code_image_mime.is_some(),
        has_cookies = cookies.is_some()
    )
)]
pub async fn create_account(
    session: &mut DbSession,
    user_id: &str,
    user_profile: serde_json::Value,
    email: Option<&str>,
    password: Option<&str>,
    signin_code: Option<&str>,
    signin_code_image: Option<&[u8]>,
    signin_code_image_mime: Option<&str>,
    cookies: Option<&str>,
) -> Result<DzmmAccount> {
    let existing = get_account(session, user_id).await?;
    if existing.is_some() {
        return Err(LiliumError::domain_service_with_code(
            "ACCOUNT_INVALID_REQUEST",
            format!("Account with user_id '{}' already exists", user_id),
        ));
    }

    if email.is_none() && signin_code.is_none() && signin_code_image.is_none() {
        return Err(LiliumError::domain_service_with_code(
            "ACCOUNT_INVALID_REQUEST",
            "Must provide either email/password, signin_code, or signin_code_image".to_string(),
        ));
    }

    if email.is_some() && password.is_none() {
        return Err(LiliumError::domain_service_with_code(
            "ACCOUNT_INVALID_REQUEST",
            "Password required when email is provided".to_string(),
        ));
    }

    if signin_code_image.is_some() && signin_code_image_mime.is_none() {
        return Err(LiliumError::domain_service_with_code(
            "ACCOUNT_INVALID_REQUEST",
            "signin_code_image_mime required when signin_code_image is provided".to_string(),
        ));
    }

    let now = Utc::now();
    let account = sqlx::query_as::<_, DzmmAccount>(
        r#"INSERT INTO dzmm_account (user_id, user_profile, email, password, signin_code, signin_code_image, signin_code_image_mime, cookies, is_enabled, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9, $10)
             RETURNING *"#,
    )
    .bind(user_id)
    .bind(user_profile)
    .bind(email)
    .bind(password)
    .bind(signin_code)
    .bind(signin_code_image)
    .bind(signin_code_image_mime)
    .bind(cookies)
    .bind(now)
    .bind(now)
    .fetch_one(session.as_mut())
    .await?;

    Ok(account)
}

#[instrument(skip(session), fields(user_id = %user_id))]
pub async fn get_account(session: &mut DbSession, user_id: &str) -> Result<Option<DzmmAccount>> {
    let account = sqlx::query_as::<_, DzmmAccount>("SELECT * FROM dzmm_account WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(session.as_mut())
        .await?;

    Ok(account)
}

#[instrument(skip(session), fields(enabled_only))]
pub async fn list_accounts(
    session: &mut DbSession,
    enabled_only: bool,
) -> Result<Vec<DzmmAccount>> {
    let accounts = if enabled_only {
        sqlx::query_as::<_, DzmmAccount>(
            "SELECT * FROM dzmm_account WHERE is_enabled = true ORDER BY created_at DESC",
        )
        .fetch_all(session.as_mut())
        .await?
    } else {
        sqlx::query_as::<_, DzmmAccount>("SELECT * FROM dzmm_account ORDER BY created_at DESC")
            .fetch_all(session.as_mut())
            .await?
    };

    Ok(accounts)
}

#[instrument(skip(session, new_password), fields(user_id = %user_id))]
pub async fn update_password(
    session: &mut DbSession,
    user_id: &str,
    new_password: &str,
) -> Result<DzmmAccount> {
    let account = get_account(session, user_id).await?;
    let account = match account {
        Some(a) => a,
        None => {
            return Err(LiliumError::domain_service_with_code(
                "ACCOUNT_INVALID_REQUEST",
                format!("Account '{}' not found", user_id),
            ));
        }
    };

    if account.email.is_none() {
        return Err(LiliumError::domain_service_with_code(
            "ACCOUNT_INVALID_REQUEST",
            format!(
                "Cannot update password for QR code signin account '{}'",
                user_id
            ),
        ));
    }

    let now = Utc::now();
    let updated = sqlx::query_as::<_, DzmmAccount>(
        "UPDATE dzmm_account SET password = $1, updated_at = $2 WHERE user_id = $3 RETURNING *",
    )
    .bind(new_password)
    .bind(now)
    .bind(user_id)
    .fetch_one(session.as_mut())
    .await?;

    Ok(updated)
}

#[instrument(skip(session, cookies), fields(user_id = %user_id))]
pub async fn update_cookies(session: &mut DbSession, user_id: &str, cookies: &str) -> Result<()> {
    let account = get_account(session, user_id).await?;
    if account.is_none() {
        return Err(LiliumError::domain_service_with_code(
            "ACCOUNT_INVALID_REQUEST",
            format!("Account '{}' not found", user_id),
        ));
    }

    let now = Utc::now();
    sqlx::query("UPDATE dzmm_account SET cookies = $1, updated_at = $2 WHERE user_id = $3")
        .bind(cookies)
        .bind(now)
        .bind(user_id)
        .execute(session.as_mut())
        .await?;

    Ok(())
}

#[instrument(skip(session, user_profile), fields(user_id = %user_id))]
pub async fn update_user_profile(
    session: &mut DbSession,
    user_id: &str,
    user_profile: serde_json::Value,
) -> Result<DzmmAccount> {
    let account = get_account(session, user_id).await?;
    if account.is_none() {
        return Err(LiliumError::domain_service_with_code(
            "ACCOUNT_INVALID_REQUEST",
            format!("Account '{}' not found", user_id),
        ));
    }

    let now = Utc::now();
    let updated = sqlx::query_as::<_, DzmmAccount>(
        "UPDATE dzmm_account SET user_profile = $1, updated_at = $2 WHERE user_id = $3 RETURNING *",
    )
    .bind(user_profile)
    .bind(now)
    .bind(user_id)
    .fetch_one(session.as_mut())
    .await?;

    Ok(updated)
}

#[instrument(skip(session), fields(user_id = %user_id))]
pub async fn activate_account(session: &mut DbSession, user_id: &str) -> Result<DzmmAccount> {
    let account = get_account(session, user_id).await?;
    if account.is_none() {
        return Err(LiliumError::domain_service_with_code(
            "ACCOUNT_INVALID_REQUEST",
            format!("Account '{}' not found", user_id),
        ));
    }

    let now = Utc::now();
    let updated = sqlx::query_as::<_, DzmmAccount>(
        "UPDATE dzmm_account SET is_enabled = true, updated_at = $1 WHERE user_id = $2 RETURNING *",
    )
    .bind(now)
    .bind(user_id)
    .fetch_one(session.as_mut())
    .await?;

    Ok(updated)
}

#[instrument(skip(session), fields(user_id = %user_id))]
pub async fn deactivate_account(session: &mut DbSession, user_id: &str) -> Result<DzmmAccount> {
    let account = get_account(session, user_id).await?;
    if account.is_none() {
        return Err(LiliumError::domain_service_with_code(
            "ACCOUNT_INVALID_REQUEST",
            format!("Account '{}' not found", user_id),
        ));
    }

    let now = Utc::now();
    let updated = sqlx::query_as::<_, DzmmAccount>(
        "UPDATE dzmm_account SET is_enabled = false, updated_at = $1 WHERE user_id = $2 RETURNING *",
    )
    .bind(now)
    .bind(user_id)
    .fetch_one(session.as_mut())
    .await?;

    Ok(updated)
}

#[instrument(skip(session), fields(user_id = %user_id))]
pub async fn delete_account(session: &mut DbSession, user_id: &str) -> Result<()> {
    let account = get_account(session, user_id).await?;
    if account.is_none() {
        return Err(LiliumError::domain_service_with_code(
            "ACCOUNT_INVALID_REQUEST",
            format!("Account '{}' not found", user_id),
        ));
    }

    let has_active = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM websocket_connections WHERE account_user_id = $1)",
    )
    .bind(user_id)
    .fetch_one(session.as_mut())
    .await?;

    if has_active {
        return Err(LiliumError::service(
            "ACCOUNT_HAS_ACTIVE_CONNECTIONS",
            format!(
                "Cannot delete account '{}': Account has active WebSocket connections. \
                         Deactivate the account and wait for connections to close before deletion.",
                user_id
            ),
        ));
    }

    sqlx::query("DELETE FROM dzmm_account WHERE user_id = $1")
        .bind(user_id)
        .execute(session.as_mut())
        .await?;

    Ok(())
}

#[instrument(skip(session))]
pub async fn get_next_available_account(session: &mut DbSession) -> Result<Option<DzmmAccount>> {
    let account = sqlx::query_as::<_, DzmmAccount>(
        "SELECT * FROM dzmm_account WHERE is_enabled = true ORDER BY created_at ASC LIMIT 1",
    )
    .fetch_optional(session.as_mut())
    .await?;

    Ok(account)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    #[tokio::test]
    async fn create_and_get_account_roundtrip() {
        let test_db =
            lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::Account)
                .await
                .expect("init account db");

        lilium_database::transaction!(test_db.database(), |session| {
            let user_id = format!(
                "account_{}_{}",
                Utc::now().timestamp_micros(),
                std::process::id()
            );
            lilium_test_fixtures::seed_test_users(session, &[&user_id])
                .await
                .expect("seed account user");
            let profile = json!({"nickname": "test_account"});
            let created = create_account(
                session,
                &user_id,
                profile.clone(),
                Some("test@example.com"),
                Some("password"),
                None,
                None,
                None,
                Some("a=b"),
            )
            .await
            .expect("create account");
            assert_eq!(created.user_id, user_id);
            let fetched = get_account(session, &user_id)
                .await
                .expect("fetch account")
                .expect("account exists");
            assert_eq!(fetched.user_id, user_id);
            assert_eq!(fetched.user_profile, profile);
            delete_account(session, &user_id)
                .await
                .expect("delete account");
            Ok(())
        })
        .await
        .expect("account roundtrip");
    }
}
