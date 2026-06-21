// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 services/account_service.py
// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 services/account_service.py
use crate::Result;
use chrono::Utc;
use lilium_api_client::http::{DzmmApi, DzmmApiAuth};
use lilium_common::LiliumError;
use lilium_models::dzmm::{account as dzmm_account, websocket_connection as websocket_connections};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use std::borrow::Cow;
use tracing::instrument;
use uuid::Uuid;

type DzmmAccount = dzmm_account::Model;

pub struct CreateAccountParams<'a> {
    pub user_id: Uuid,
    pub user_profile: serde_json::Value,
    pub email: Option<&'a str>,
    pub password: Option<&'a str>,
    pub signin_code: Option<&'a str>,
    pub signin_code_image: Option<&'a [u8]>,
    pub signin_code_image_mime: Option<&'a str>,
    pub cookies: Option<&'a str>,
}

pub fn create_auth_client(account: DzmmAccount) -> Result<DzmmApi> {
    DzmmApi::new(DzmmApiAuth {
        email: account.email.map(Cow::Owned),
        password: account.password.map(Cow::Owned),
        signin_code: account.signin_code.map(Cow::Owned),
        signin_code_image: account.signin_code_image,
        signin_code_image_mime: account.signin_code_image_mime.map(Cow::Owned),
        cookies: account.cookies.map(Cow::Owned),
        user_id: Some(Cow::Owned(account.user_id.to_string())),
        auto_refresh: true,
        on_cookies_refreshed: None,
    })
    .map_err(|e| LiliumError::service("ACCOUNT_AUTH_CLIENT_BUILD_FAILED", e.to_string()))
}

#[instrument(level = "debug"
    skip(db, params),
    fields(
        user_id = %params.user_id,
        has_email = params.email.is_some(),
        has_password = params.password.is_some(),
        has_signin_code = params.signin_code.is_some(),
        has_signin_code_image = params.signin_code_image.is_some(),
        has_signin_code_image_mime = params.signin_code_image_mime.is_some(),
        has_cookies = params.cookies.is_some()
    )
)]
pub async fn create_account<C>(db: &C, params: CreateAccountParams<'_>) -> Result<DzmmAccount>
where
    C: ConnectionTrait,
{
    let CreateAccountParams {
        user_id,
        user_profile,
        email,
        password,
        signin_code,
        signin_code_image,
        signin_code_image_mime,
        cookies,
    } = params;

    let existing = get_account(db, user_id).await?;
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
    let account = dzmm_account::ActiveModel {
        user_id: Set(user_id),
        user_profile: Set(user_profile),
        email: Set(email.map(str::to_owned)),
        password: Set(password.map(str::to_owned)),
        signin_code: Set(signin_code.map(str::to_owned)),
        signin_code_image: Set(signin_code_image.map(|value| value.to_vec())),
        signin_code_image_mime: Set(signin_code_image_mime.map(str::to_owned)),
        cookies: Set(cookies.map(str::to_owned)),
        is_enabled: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await?;

    Ok(account)
}

#[instrument(level = "debug" skip(db), fields(user_id = %user_id))]
pub async fn get_account<C>(db: &C, user_id: Uuid) -> Result<Option<DzmmAccount>>
where
    C: ConnectionTrait,
{
    let account = dzmm_account::Entity::find_by_id(user_id).one(db).await?;

    Ok(account)
}

#[instrument(level = "debug" skip(db), fields(enabled_only))]
pub async fn list_accounts<C>(db: &C, enabled_only: bool) -> Result<Vec<DzmmAccount>>
where
    C: ConnectionTrait,
{
    let accounts = if enabled_only {
        dzmm_account::Entity::find()
            .filter(dzmm_account::Column::IsEnabled.eq(true))
            .order_by_desc(dzmm_account::Column::CreatedAt)
            .all(db)
            .await?
    } else {
        dzmm_account::Entity::find()
            .order_by_desc(dzmm_account::Column::CreatedAt)
            .all(db)
            .await?
    };

    Ok(accounts.into_iter().collect())
}

#[instrument(level = "debug" skip(db, new_password), fields(user_id = %user_id))]
pub async fn update_password<C>(db: &C, user_id: Uuid, new_password: &str) -> Result<DzmmAccount>
where
    C: ConnectionTrait,
{
    let account = get_account(db, user_id).await?;
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
    let updated = dzmm_account::ActiveModel {
        user_id: Set(user_id),
        password: Set(Some(new_password.to_owned())),
        updated_at: Set(now),
        ..Default::default()
    }
    .update(db)
    .await?;

    Ok(updated)
}

#[instrument(level = "debug" skip(db, cookies), fields(user_id = %user_id))]
pub async fn update_cookies<C>(db: &C, user_id: Uuid, cookies: &str) -> Result<()>
where
    C: ConnectionTrait,
{
    let account = get_account(db, user_id).await?;
    if account.is_none() {
        return Err(LiliumError::domain_service_with_code(
            "ACCOUNT_INVALID_REQUEST",
            format!("Account '{}' not found", user_id),
        ));
    }

    let now = Utc::now();
    dzmm_account::ActiveModel {
        user_id: Set(user_id),
        cookies: Set(Some(cookies.to_owned())),
        updated_at: Set(now),
        ..Default::default()
    }
    .update(db)
    .await?;

    Ok(())
}

#[instrument(level = "debug" skip(db, user_profile), fields(user_id = %user_id))]
pub async fn update_user_profile<C>(
    db: &C,
    user_id: Uuid,
    user_profile: serde_json::Value,
) -> Result<DzmmAccount>
where
    C: ConnectionTrait,
{
    let account = get_account(db, user_id).await?;
    if account.is_none() {
        return Err(LiliumError::domain_service_with_code(
            "ACCOUNT_INVALID_REQUEST",
            format!("Account '{}' not found", user_id),
        ));
    }

    let now = Utc::now();
    let updated = dzmm_account::ActiveModel {
        user_id: Set(user_id),
        user_profile: Set(user_profile),
        updated_at: Set(now),
        ..Default::default()
    }
    .update(db)
    .await?;

    Ok(updated)
}

#[instrument(level = "debug" skip(db), fields(user_id = %user_id))]
pub async fn activate_account<C>(db: &C, user_id: Uuid) -> Result<DzmmAccount>
where
    C: ConnectionTrait,
{
    let account = get_account(db, user_id).await?;
    if account.is_none() {
        return Err(LiliumError::domain_service_with_code(
            "ACCOUNT_INVALID_REQUEST",
            format!("Account '{}' not found", user_id),
        ));
    }

    let now = Utc::now();
    let updated = dzmm_account::ActiveModel {
        user_id: Set(user_id),
        is_enabled: Set(true),
        updated_at: Set(now),
        ..Default::default()
    }
    .update(db)
    .await?;

    Ok(updated)
}

#[instrument(level = "debug" skip(db), fields(user_id = %user_id))]
pub async fn deactivate_account<C>(db: &C, user_id: Uuid) -> Result<DzmmAccount>
where
    C: ConnectionTrait,
{
    let account = get_account(db, user_id).await?;
    if account.is_none() {
        return Err(LiliumError::domain_service_with_code(
            "ACCOUNT_INVALID_REQUEST",
            format!("Account '{}' not found", user_id),
        ));
    }

    let now = Utc::now();
    let updated = dzmm_account::ActiveModel {
        user_id: Set(user_id),
        is_enabled: Set(false),
        updated_at: Set(now),
        ..Default::default()
    }
    .update(db)
    .await?;

    Ok(updated)
}

#[instrument(level = "debug" skip(db), fields(user_id = %user_id))]
pub async fn delete_account<C>(db: &C, user_id: Uuid) -> Result<()>
where
    C: ConnectionTrait,
{
    let account = get_account(db, user_id).await?;
    if account.is_none() {
        return Err(LiliumError::domain_service_with_code(
            "ACCOUNT_INVALID_REQUEST",
            format!("Account '{}' not found", user_id),
        ));
    }

    let has_active = websocket_connections::Entity::find()
        .filter(websocket_connections::Column::AccountUserId.eq(user_id))
        .count(db)
        .await?
        > 0;

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

    dzmm_account::Entity::delete_by_id(user_id).exec(db).await?;

    Ok(())
}

#[instrument(level = "debug" skip(db))]
pub async fn get_next_available_account<C>(db: &C) -> Result<Option<DzmmAccount>>
where
    C: ConnectionTrait,
{
    let account = dzmm_account::Entity::find()
        .filter(dzmm_account::Column::IsEnabled.eq(true))
        .order_by_asc(dzmm_account::Column::CreatedAt)
        .one(db)
        .await?;

    Ok(account)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use lilium_models::dzmm::user as users;
    use serde_json::json;

    #[tokio::test]
    async fn create_and_get_account_roundtrip() {
        let test_db =
            lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::Account)
                .await
                .expect("init account db");

        lilium_database::transaction!(test_db.database(), |tx| {
            let user_id = Uuid::new_v4();
            let now = Utc::now();
            users::Entity::insert(users::ActiveModel {
                user_id: Set(user_id),
                message_count: Set(0),
                deleted_count: Set(0),
                recalled_count: Set(0),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            })
            .exec(tx)
            .await
            .expect("seed account user");
            let profile = json!({"nickname": "test_account"});
            let created = create_account(
                tx,
                CreateAccountParams {
                    user_id,
                    user_profile: profile.clone(),
                    email: Some("test@example.com"),
                    password: Some("password"),
                    signin_code: None,
                    signin_code_image: None,
                    signin_code_image_mime: None,
                    cookies: Some("a=b"),
                },
            )
            .await
            .expect("create account");
            assert_eq!(created.user_id, user_id);
            let fetched = get_account(tx, user_id)
                .await
                .expect("fetch account")
                .expect("account exists");
            assert_eq!(fetched.user_id, user_id);
            assert_eq!(fetched.user_profile, profile);
            delete_account(tx, user_id).await.expect("delete account");
            Ok(())
        })
        .await
        .expect("account roundtrip");
    }
}
