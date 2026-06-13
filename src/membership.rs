use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, FromRow)]
pub struct Membership {
    pub id: Uuid,
    pub community: Uuid,
    pub member: Uuid,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateMembership {
    pub community: Uuid,
    pub member: Uuid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct UpdateMembership {
    pub community: Option<Uuid>,
    pub member: Option<Uuid>,
}

#[derive(Debug, thiserror::Error)]
pub enum MembershipRepositoryError {
    #[error("membership not found")]
    NotFound,

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[async_trait::async_trait]
pub trait MembershipRepository: Send + Sync {
    async fn create(
        &self,
        request: CreateMembership,
    ) -> Result<Membership, MembershipRepositoryError>;

    async fn get_by_id(&self, id: Uuid) -> Result<Membership, MembershipRepositoryError>;

    async fn list(&self) -> Result<Vec<Membership>, MembershipRepositoryError>;

    async fn update(
        &self,
        id: Uuid,
        request: UpdateMembership,
    ) -> Result<Membership, MembershipRepositoryError>;

    async fn delete(&self, id: Uuid) -> Result<(), MembershipRepositoryError>;

    async fn list_by_member(
        &self,
        member: Uuid,
    ) -> Result<Vec<Membership>, MembershipRepositoryError>;
}

use sqlx::SqlitePool;

#[derive(Clone)]
pub struct SqliteMembershipRepository {
    pool: SqlitePool,
}

impl SqliteMembershipRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl MembershipRepository for SqliteMembershipRepository {
    async fn create(
        &self,
        request: CreateMembership,
    ) -> Result<Membership, MembershipRepositoryError> {
        let membership = Membership {
            id: Uuid::new_v4(),
            community: request.community,
            member: request.member,
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
        .execute(&self.pool)
        .await?;

        Ok(membership)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Membership, MembershipRepositoryError> {
        sqlx::query_as!(
            Membership,
            r#"
            SELECT
                id as "id: Uuid",
                community as "community: Uuid",
                member as "member: Uuid",
                created as "created: DateTime<Utc>"
            FROM memberships
            WHERE id = ?1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(MembershipRepositoryError::NotFound)
    }

    async fn list(&self) -> Result<Vec<Membership>, MembershipRepositoryError> {
        Ok(sqlx::query_as!(
            Membership,
            r#"
                SELECT
                    id as "id: Uuid",
                    community as "community: Uuid",
                    member as "member: Uuid",
                    created as "created: DateTime<Utc>"
                FROM memberships
                ORDER BY created DESC
                "#
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn update(
        &self,
        id: Uuid,
        request: UpdateMembership,
    ) -> Result<Membership, MembershipRepositoryError> {
        let membership = self.get_by_id(id).await?;

        let community = request.community.unwrap_or(membership.community);
        let member = request.member.unwrap_or(membership.member);

        let result = sqlx::query!(
            r#"
            UPDATE memberships
            SET
                community = ?1,
                member = ?2
            WHERE id = ?3
            "#,
            community,
            member,
            id,
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(MembershipRepositoryError::NotFound);
        }

        self.get_by_id(id).await
    }

    async fn delete(&self, id: Uuid) -> Result<(), MembershipRepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM memberships
            WHERE id = ?1
            "#,
            id,
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(MembershipRepositoryError::NotFound);
        }

        Ok(())
    }

    async fn list_by_member(
        &self,
        member: Uuid,
    ) -> Result<Vec<Membership>, MembershipRepositoryError> {
        Ok(sqlx::query_as!(
            Membership,
            r#"
                SELECT
                    id as "id: Uuid",
                    community as "community: Uuid",
                    member as "member: Uuid",
                    created as "created: DateTime<Utc>"
                FROM memberships
                WHERE member = ?1
                ORDER BY created DESC
                "#,
            member,
        )
        .fetch_all(&self.pool)
        .await?)
    }
}
