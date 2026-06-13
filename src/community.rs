use crate::membership::Membership;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, FromRow)]
pub struct Community {
    pub id: Uuid,
    pub name: String,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateCommunity {
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct UpdateCommunity {
    pub name: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CommunityRepositoryError {
    #[error("community not found")]
    NotFound,

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[async_trait::async_trait]
pub trait CommunityRepository: Send + Sync {
    async fn create(
        &self,
        request: CreateCommunity,
        owner_id: Uuid,
    ) -> Result<Community, CommunityRepositoryError>;

    async fn get_by_id(&self, id: Uuid) -> Result<Community, CommunityRepositoryError>;

    async fn list(&self) -> Result<Vec<Community>, CommunityRepositoryError>;

    /// Returns only the communities that `member` belongs to.
    async fn list_by_member(
        &self,
        member: Uuid,
    ) -> Result<Vec<Community>, CommunityRepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        request: UpdateCommunity,
    ) -> Result<Community, CommunityRepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<(), CommunityRepositoryError>;
}

use sqlx::SqlitePool;

#[derive(Clone)]
pub struct SqliteCommunityRepository {
    pool: SqlitePool,
}

impl SqliteCommunityRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CommunityRepository for SqliteCommunityRepository {
    // In SqliteCommunityRepository
    async fn create(
        &self,
        request: CreateCommunity,
        owner_id: Uuid,
    ) -> Result<Community, CommunityRepositoryError> {
        let mut tx = self.pool.begin().await?;

        let community = Community {
            id: Uuid::new_v4(),
            name: request.name,
            created: Utc::now(),
        };

        sqlx::query!(
            r#"
            INSERT INTO communities (
                id,
                name,
                created
            )
            VALUES (?1, ?2, ?3)
            "#,
            community.id,
            community.name,
            community.created,
        )
        .execute(&mut *tx)
        .await?;

        let membership = Membership {
            id: Uuid::new_v4(),
            community: community.id,
            member: owner_id,
            created: Utc::now(),
        };

        sqlx::query!(
            r#"
            INSERT INTO memberships (
                id,
                community,
                member,
                created
            )
            VALUES (?1, ?2, ?3, ?4)
            "#,
            membership.id,
            membership.community,
            membership.member,
            membership.created,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(community)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Community, CommunityRepositoryError> {
        sqlx::query_as!(
            Community,
            r#"
            SELECT
                id as "id: Uuid",
                name,
                created as "created: DateTime<Utc>"
            FROM communities
            WHERE id = ?1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(CommunityRepositoryError::NotFound)
    }

    async fn list(&self) -> Result<Vec<Community>, CommunityRepositoryError> {
        Ok(sqlx::query_as!(
            Community,
            r#"
                SELECT
                    id as "id: Uuid",
                    name,
                    created as "created: DateTime<Utc>"
                FROM communities
                ORDER BY created DESC
                "#
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn list_by_member(
        &self,
        member: Uuid,
    ) -> Result<Vec<Community>, CommunityRepositoryError> {
        Ok(sqlx::query_as!(
            Community,
            r#"
                SELECT
                    c.id as "id: Uuid",
                    c.name,
                    c.created as "created: DateTime<Utc>"
                FROM communities c
                INNER JOIN memberships m ON m.community = c.id
                WHERE m.member = ?1
                ORDER BY c.created DESC
                "#,
            member,
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn update(
        &self,
        id: Uuid,
        request: UpdateCommunity,
    ) -> Result<Community, CommunityRepositoryError> {
        let community = self.get_by_id(id).await?;

        let name = request.name.unwrap_or(community.name);

        let result = sqlx::query!(
            r#"
            UPDATE communities
            SET name = ?1
            WHERE id = ?2
            "#,
            name,
            id,
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(CommunityRepositoryError::NotFound);
        }

        self.get_by_id(id).await
    }

    async fn delete(&self, id: Uuid) -> Result<(), CommunityRepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM communities
            WHERE id = ?1
            "#,
            id,
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(CommunityRepositoryError::NotFound);
        }

        Ok(())
    }
}
