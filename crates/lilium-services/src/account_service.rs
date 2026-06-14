use anyhow::{bail, Result};
use chrono::Utc;
use lilium_database::DbSessionContext;

use lilium_models::dzmm::account::DzmmAccount;

pub struct AccountService<'a> {
    session: DbSessionContext<'a>,
}

impl<'a> AccountService<'a> {
    pub fn new(session: DbSessionContext<'a>) -> Self {
        Self { session }
    }

    pub async fn create_account(
        &mut self,
        user_id: &str,
        user_profile: serde_json::Value,
        email: Option<&str>,
        password: Option<&str>,
        signin_code: Option<&str>,
        signin_code_image: Option<&[u8]>,
        signin_code_image_mime: Option<&str>,
        cookies: Option<&str>,
    ) -> Result<DzmmAccount> {
        let existing = self.get_account(user_id).await?;
        if existing.is_some() {
            bail!(lilium_common::error::LiliumError::domain_service(format!(
                "Account with user_id '{}' already exists",
                user_id
            )));
        }

        if email.is_none() && signin_code.is_none() && signin_code_image.is_none() {
            bail!(lilium_common::error::LiliumError::domain_service(
                "Must provide either email/password, signin_code, or signin_code_image".to_string()
            ));
        }

        if email.is_some() && password.is_none() {
            bail!(lilium_common::error::LiliumError::domain_service(
                "Password required when email is provided".to_string()
            ));
        }

        if signin_code_image.is_some() && signin_code_image_mime.is_none() {
            bail!(lilium_common::error::LiliumError::domain_service(
                "signin_code_image_mime required when signin_code_image is provided".to_string()
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
        .fetch_one(self.session.as_mut())
        .await?;

        Ok(account)
    }

    pub async fn get_account(&mut self, user_id: &str) -> Result<Option<DzmmAccount>> {
        let account =
            sqlx::query_as::<_, DzmmAccount>("SELECT * FROM dzmm_account WHERE user_id = $1")
                .bind(user_id)
                .fetch_optional(self.session.as_mut())
                .await?;

        Ok(account)
    }

    pub async fn list_accounts(&mut self, enabled_only: bool) -> Result<Vec<DzmmAccount>> {
        let accounts = if enabled_only {
            sqlx::query_as::<_, DzmmAccount>(
                "SELECT * FROM dzmm_account WHERE is_enabled = true ORDER BY created_at DESC",
            )
            .fetch_all(self.session.as_mut())
            .await?
        } else {
            sqlx::query_as::<_, DzmmAccount>("SELECT * FROM dzmm_account ORDER BY created_at DESC")
                .fetch_all(self.session.as_mut())
                .await?
        };

        Ok(accounts)
    }

    pub async fn update_password(
        &mut self,
        user_id: &str,
        new_password: &str,
    ) -> Result<DzmmAccount> {
        let account = self.get_account(user_id).await?;
        let account = match account {
            Some(a) => a,
            None => bail!(lilium_common::error::LiliumError::domain_service(format!(
                "Account '{}' not found",
                user_id
            ))),
        };

        if account.email.is_none() {
            bail!(lilium_common::error::LiliumError::domain_service(format!(
                "Cannot update password for QR code signin account '{}'",
                user_id
            )));
        }

        let now = Utc::now();
        let updated = sqlx::query_as::<_, DzmmAccount>(
            "UPDATE dzmm_account SET password = $1, updated_at = $2 WHERE user_id = $3 RETURNING *",
        )
        .bind(new_password)
        .bind(now)
        .bind(user_id)
        .fetch_one(self.session.as_mut())
        .await?;

        Ok(updated)
    }

    pub async fn update_cookies(&mut self, user_id: &str, cookies: &str) -> Result<()> {
        let account = self.get_account(user_id).await?;
        if account.is_none() {
            bail!(lilium_common::error::LiliumError::domain_service(format!(
                "Account '{}' not found",
                user_id
            )));
        }

        let now = Utc::now();
        sqlx::query("UPDATE dzmm_account SET cookies = $1, updated_at = $2 WHERE user_id = $3")
            .bind(cookies)
            .bind(now)
            .bind(user_id)
            .execute(self.session.as_mut())
            .await?;

        Ok(())
    }

    pub async fn update_user_profile(
        &mut self,
        user_id: &str,
        user_profile: serde_json::Value,
    ) -> Result<DzmmAccount> {
        let account = self.get_account(user_id).await?;
        if account.is_none() {
            bail!(lilium_common::error::LiliumError::domain_service(format!(
                "Account '{}' not found",
                user_id
            )));
        }

        let now = Utc::now();
        let updated = sqlx::query_as::<_, DzmmAccount>(
            "UPDATE dzmm_account SET user_profile = $1, updated_at = $2 WHERE user_id = $3 RETURNING *",
        )
        .bind(user_profile)
        .bind(now)
        .bind(user_id)
        .fetch_one(self.session.as_mut())
        .await?;

        Ok(updated)
    }

    pub async fn activate_account(&mut self, user_id: &str) -> Result<DzmmAccount> {
        let account = self.get_account(user_id).await?;
        if account.is_none() {
            bail!(lilium_common::error::LiliumError::domain_service(format!(
                "Account '{}' not found",
                user_id
            )));
        }

        let now = Utc::now();
        let updated = sqlx::query_as::<_, DzmmAccount>(
            "UPDATE dzmm_account SET is_enabled = true, updated_at = $1 WHERE user_id = $2 RETURNING *",
        )
        .bind(now)
        .bind(user_id)
        .fetch_one(self.session.as_mut())
        .await?;

        Ok(updated)
    }

    pub async fn deactivate_account(&mut self, user_id: &str) -> Result<DzmmAccount> {
        let account = self.get_account(user_id).await?;
        if account.is_none() {
            bail!(lilium_common::error::LiliumError::domain_service(format!(
                "Account '{}' not found",
                user_id
            )));
        }

        let now = Utc::now();
        let updated = sqlx::query_as::<_, DzmmAccount>(
            "UPDATE dzmm_account SET is_enabled = false, updated_at = $1 WHERE user_id = $2 RETURNING *",
        )
        .bind(now)
        .bind(user_id)
        .fetch_one(self.session.as_mut())
        .await?;

        Ok(updated)
    }

    pub async fn delete_account(&mut self, user_id: &str) -> Result<()> {
        let account = self.get_account(user_id).await?;
        if account.is_none() {
            bail!(lilium_common::error::LiliumError::domain_service(format!(
                "Account '{}' not found",
                user_id
            )));
        }

        let has_active = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM websocket_connections WHERE account_user_id = $1)",
        )
        .bind(user_id)
        .fetch_one(self.session.as_mut())
        .await?;

        if has_active {
            bail!(lilium_common::error::LiliumError::service(
                "ACCOUNT_HAS_ACTIVE_CONNECTIONS",
                format!(
                    "Cannot delete account '{}': Account has active WebSocket connections. \
                         Deactivate the account and wait for connections to close before deletion.",
                    user_id
                )
            ));
        }

        sqlx::query("DELETE FROM dzmm_account WHERE user_id = $1")
            .bind(user_id)
            .execute(self.session.as_mut())
            .await?;

        Ok(())
    }

    pub async fn get_next_available_account(&mut self) -> Result<Option<DzmmAccount>> {
        let account = sqlx::query_as::<_, DzmmAccount>(
            "SELECT * FROM dzmm_account WHERE is_enabled = true ORDER BY created_at ASC LIMIT 1",
        )
        .fetch_optional(self.session.as_mut())
        .await?;

        Ok(account)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    #[tokio::test]
    async fn service_struct_can_be_created() {
        lilium_database::test_fixtures::with_db_session(
            lilium_database::test_fixtures::TestServiceFixture::Account,
            |session| {
                Box::pin(async move {
                    let _svc = AccountService::new(session);
                    Ok(())
                })
            },
        )
        .await
        .expect("service struct can be created");
    }

    #[tokio::test]
    async fn create_and_get_account_roundtrip() {
        lilium_database::test_fixtures::with_db_session(
            lilium_database::test_fixtures::TestServiceFixture::Account,
            |session| {
                Box::pin(async move {
                    let mut svc = AccountService::new(session);
                    let user_id = format!(
                        "account_{}_{}",
                        Utc::now().timestamp_micros(),
                        std::process::id()
                    );
                    let profile = json!({"nickname": "test_account"});
                    let created = svc
                        .create_account(
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
                    let fetched = svc
                        .get_account(&user_id)
                        .await
                        .expect("fetch account")
                        .expect("account exists");
                    assert_eq!(fetched.user_id, user_id);
                    assert_eq!(fetched.user_profile, profile);
                    svc.delete_account(&user_id).await.expect("delete account");
                    Ok(())
                })
            },
        )
        .await
        .expect("account roundtrip");
    }
}
