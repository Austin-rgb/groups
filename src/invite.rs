use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Invite {
    pub id: Uuid,
    pub community: Uuid,
    pub user: Uuid,
    pub created: DateTime<Utc>,
    pub exp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvite {
    pub community: Uuid,
    pub user: Uuid,
    pub exp: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum InviteRepositoryError {
    #[error("invite not found")]
    NotFound,

    #[error("invite already exists")]
    AlreadyExists,

    #[error("database error")]
    DbError,
}

#[async_trait]
pub trait InviteRepository: Send + Sync {
    async fn create(&self, invite: CreateInvite) -> Result<Invite, InviteRepositoryError>;

    async fn get_by_id(&self, id: Uuid) -> Result<Invite, InviteRepositoryError>;

    async fn list(&self) -> Result<Vec<Invite>, InviteRepositoryError>;

    async fn list_by_user(&self, user: Uuid) -> Result<Vec<Invite>, InviteRepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<(), InviteRepositoryError>;
}

#[derive(Clone)]
pub struct SqliteInviteRepository {
    pool: SqlitePool,
}

impl SqliteInviteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InviteRepository for SqliteInviteRepository {
    async fn create(&self, invite: CreateInvite) -> Result<Invite, InviteRepositoryError> {
        let entity = Invite {
            id: Uuid::new_v4(),
            community: invite.community,
            user: invite.user,
            created: Utc::now(),
            exp: invite.exp,
        };

        let result = sqlx::query!(
            r#"
            INSERT INTO invite (
                id,
                community,
                user,
                created,
                exp
            )
            VALUES (?, ?, ?, ?, ?)
            "#,
            entity.id,
            entity.community,
            entity.user,
            entity.created,
            entity.exp,
        )
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(entity),
            Err(sqlx::Error::Database(_)) => Err(InviteRepositoryError::AlreadyExists),
            Err(_) => Err(InviteRepositoryError::DbError),
        }
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Invite, InviteRepositoryError> {
        sqlx::query_as!(
            Invite,
            r#"
            SELECT
                id as "id: Uuid",
                community as "community: Uuid",
                user as "user: Uuid",
                created as "created: DateTime<Utc>",
                exp as "exp: DateTime<Utc>"
            FROM invite
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| InviteRepositoryError::DbError)?
        .ok_or(InviteRepositoryError::NotFound)
    }

    async fn list(&self) -> Result<Vec<Invite>, InviteRepositoryError> {
        sqlx::query_as!(
            Invite,
            r#"
            SELECT
                id as "id: Uuid",
                community as "community: Uuid",
                user as "user: Uuid",
                created as "created: DateTime<Utc>",
                exp as "exp: DateTime<Utc>"
            FROM invite
            ORDER BY created DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| InviteRepositoryError::DbError)
    }

    async fn list_by_user(&self, user: Uuid) -> Result<Vec<Invite>, InviteRepositoryError> {
        sqlx::query_as!(
            Invite,
            r#"
        SELECT
            id as "id: Uuid",
            community as "community: Uuid",
            user as "user: Uuid",
            created as "created: DateTime<Utc>",
            exp as "exp: DateTime<Utc>"
        FROM invite
        WHERE user = ?
        ORDER BY created DESC
        "#,
            user,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| InviteRepositoryError::DbError)
    }

    async fn delete(&self, id: Uuid) -> Result<(), InviteRepositoryError> {
        let rows = sqlx::query!(
            r#"
            DELETE FROM invite
            WHERE id = ?
            "#,
            id,
        )
        .execute(&self.pool)
        .await
        .map_err(|_| InviteRepositoryError::DbError)?;

        if rows.rows_affected() == 0 {
            return Err(InviteRepositoryError::NotFound);
        }

        Ok(())
    }
}
