//! Library root for SQLite repository layer

pub mod community;
mod handlers;
pub mod membership;
use std::sync::Arc;
mod invite;
use actix_web::web::ServiceConfig;
use handlers::configure_routes;
use sqlx::{Pool, Sqlite};

use crate::{
    community::{CommunityRepository, SqliteCommunityRepository},
    invite::{InviteRepository, SqliteInviteRepository},
    membership::{MembershipRepository, SqliteMembershipRepository},
};

pub struct Module {
    communities: Arc<dyn CommunityRepository>,
    membership: Arc<dyn MembershipRepository>,
    invites: Arc<dyn InviteRepository>,
}

impl Module {
    pub fn config(&self, cfg: &mut ServiceConfig) {
        cfg.app_data(self.communities.clone())
            .app_data(self.membership.clone())
            .app_data(self.invites.clone())
            .configure(configure_routes);
    }

    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self {
            invites: Arc::new(SqliteInviteRepository::new(pool.clone())),
            communities: Arc::new(SqliteCommunityRepository::new(pool.clone())),
            membership: Arc::new(SqliteMembershipRepository::new(pool.clone())),
        }
    }
}
