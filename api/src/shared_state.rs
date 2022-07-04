use std::sync::Arc;

use pic_store_auth::RootAuthEvaulator;
use pic_store_db as db;
use uuid::Uuid;

pub struct InnerState {
    pub production: bool,
    pub db: db::Pool,

    pub auth: RootAuthEvaulator,

    // Hardcoded values until we have real user auth and such.
    pub user_id: Uuid,
    pub team_id: Uuid,
    pub project_id: Uuid,
}

impl std::fmt::Debug for InnerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerState")
            .field("production", &self.production)
            .field("auth", &self.auth)
            .field("user_id", &self.user_id)
            .field("team_id", &self.team_id)
            .field("project_id", &self.project_id)
            .finish_non_exhaustive()
    }
}

pub type State = Arc<InnerState>;
