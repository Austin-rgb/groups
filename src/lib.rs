//! Library root for SQLite repository layer

pub mod community;
mod handlers;
pub mod membership;
use std::sync::Arc;

use actix_web::web::ServiceConfig;
pub use handlers::configure_routes;
use sqlx::{Pool, Sqlite};

use crate::{
    community::{CommunityRepository, SqliteCommunityRepository},
    membership::{MembershipRepository, SqliteMembershipRepository},
};

pub struct Module {
    communities: Arc<dyn CommunityRepository>,
    membership: Arc<dyn MembershipRepository>,
}

impl Module {
    pub fn config(&self, cfg: &mut ServiceConfig) {
        cfg.app_data(self.communities.clone())
            .app_data(self.membership.clone())
            .configure(configure_routes);
    }

    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self {
            communities: Arc::new(SqliteCommunityRepository::new(pool.clone())),
            membership: Arc::new(SqliteMembershipRepository::new(pool.clone())),
        }
    }
}
