// handlers.rs
use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
use actixutils::{Auth, Identity};
use std::sync::Arc;
use uuid::Uuid;

use crate::community::{
    CommunityRepository, CommunityRepositoryError, CreateCommunity, SqliteCommunityRepository,
    UpdateCommunity,
};

use crate::membership::*;

use crate::invite::*;

// Helper for error responses
fn error_response(
    error: impl std::fmt::Display,
    status: actix_web::http::StatusCode,
) -> HttpResponse {
    HttpResponse::build(status).json(serde_json::json!({
        "error": error.to_string()
    }))
}

/// Checks membership in a single query. Returns Ok(()) or a ready HttpResponse.
async fn assert_member_of(
    membership_repo: &SqliteMembershipRepository,
    user_id: Uuid,
    community_id: Uuid,
) -> Result<(), HttpResponse> {
    match membership_repo.list_by_member(user_id).await {
        Err(e) => Err(error_response(
            e,
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        )),
        Ok(memberships) => {
            if memberships.iter().any(|m| m.community == community_id) {
                Ok(())
            } else {
                Err(error_response(
                    "Forbidden",
                    actix_web::http::StatusCode::FORBIDDEN,
                ))
            }
        }
    }
}

// ============ Community Handlers ============

#[post("/communities")]
async fn create_community(
    Auth(user): Auth<Identity>,
    repo: web::Data<Arc<SqliteCommunityRepository>>,
    request: web::Json<CreateCommunity>,
) -> impl Responder {
    let cmd = request.into_inner();
    match repo.create(cmd, user.sub).await {
        Ok(community) => HttpResponse::Created().json(community),

        Err(e) => error_response(e, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[get("/communities/{id}")]
async fn get_community(
    Auth(user): Auth<Identity>,
    repo: web::Data<Arc<SqliteCommunityRepository>>,
    membership_repo: web::Data<Arc<SqliteMembershipRepository>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    if let Err(resp) = assert_member_of(&membership_repo, user.sub, id).await {
        return resp;
    }
    match repo.get_by_id(id).await {
        Ok(community) => HttpResponse::Ok().json(community),
        Err(CommunityRepositoryError::NotFound) => error_response(
            "Community not found",
            actix_web::http::StatusCode::NOT_FOUND,
        ),
        Err(e) => error_response(e, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Single JOIN query — no fan-out.
#[get("/communities")]
async fn list_communities(
    Auth(user): Auth<Identity>,
    repo: web::Data<Arc<SqliteCommunityRepository>>,
) -> impl Responder {
    match repo.list_by_member(user.sub).await {
        Ok(communities) => HttpResponse::Ok().json(communities),
        Err(e) => error_response(e, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[put("/communities/{id}")]
async fn update_community(
    Auth(user): Auth<Identity>,
    repo: web::Data<Arc<SqliteCommunityRepository>>,
    membership_repo: web::Data<Arc<SqliteMembershipRepository>>,
    path: web::Path<Uuid>,
    request: web::Json<UpdateCommunity>,
) -> impl Responder {
    let id = path.into_inner();
    if let Err(resp) = assert_member_of(&membership_repo, user.sub, id).await {
        return resp;
    }
    match repo.update(id, request.into_inner()).await {
        Ok(community) => HttpResponse::Ok().json(community),
        Err(CommunityRepositoryError::NotFound) => error_response(
            "Community not found",
            actix_web::http::StatusCode::NOT_FOUND,
        ),
        Err(e) => error_response(e, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[delete("/communities/{id}")]
async fn delete_community(
    Auth(user): Auth<Identity>,
    repo: web::Data<Arc<SqliteCommunityRepository>>,
    membership_repo: web::Data<Arc<SqliteMembershipRepository>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    if let Err(resp) = assert_member_of(&membership_repo, user.sub, id).await {
        return resp;
    }
    match repo.delete(id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(CommunityRepositoryError::NotFound) => error_response(
            "Community not found",
            actix_web::http::StatusCode::NOT_FOUND,
        ),
        Err(e) => error_response(e, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// ============ Membership Handlers ============

#[get("/memberships/{id}")]
async fn get_membership(
    Auth(user): Auth<Identity>,
    repo: web::Data<Arc<SqliteMembershipRepository>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    match repo.get_by_id(id).await {
        Ok(membership) if membership.member == user.sub => HttpResponse::Ok().json(membership),
        Ok(_) => error_response("Forbidden", actix_web::http::StatusCode::FORBIDDEN),
        Err(MembershipRepositoryError::NotFound) => error_response(
            "Membership not found",
            actix_web::http::StatusCode::NOT_FOUND,
        ),
        Err(e) => error_response(e, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[get("/memberships")]
async fn list_memberships(
    Auth(user): Auth<Identity>,
    repo: web::Data<Arc<SqliteMembershipRepository>>,
) -> impl Responder {
    match repo.list_by_member(user.sub).await {
        Ok(memberships) => HttpResponse::Ok().json(memberships),
        Err(e) => error_response(e, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}


#[delete("/memberships/{id}")]
async fn delete_membership(
    Auth(user): Auth<Identity>,
    repo: web::Data<Arc<SqliteMembershipRepository>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    match repo.get_by_id(id).await {
        Ok(membership) if membership.member != user.sub => {
            return error_response("Forbidden", actix_web::http::StatusCode::FORBIDDEN);
        }
        Ok(_) => {}
        Err(MembershipRepositoryError::NotFound) => {
            return error_response(
                "Membership not found",
                actix_web::http::StatusCode::NOT_FOUND,
            );
        }
        Err(e) => return error_response(e, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
    match repo.delete(id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(MembershipRepositoryError::NotFound) => error_response(
            "Membership not found",
            actix_web::http::StatusCode::NOT_FOUND,
        ),
        Err(e) => error_response(e, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}
// ============ Invites =============

#[post("/invites")]
async fn create_invite(
    Auth(user): Auth<Identity>,
    repo: web::Data<Arc<SqliteInviteRepository>>,
    membership_repo: web::Data<Arc<SqliteMembershipRepository>>,
    request: web::Json<CreateInvite>,
) -> impl Responder {
    let cmd = request.into_inner();

    if let Err(resp) = assert_member_of(&membership_repo, user.sub, cmd.community).await {
        return resp;
    }

    match repo.create(cmd).await {
        Ok(invite) => HttpResponse::Created().json(invite),

        Err(InviteRepositoryError::AlreadyExists) => error_response(
            "Invite already exists",
            actix_web::http::StatusCode::CONFLICT,
        ),

        Err(e) => error_response(e, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[get("/invites/{id}")]
async fn get_invite(
    Auth(user): Auth<Identity>,
    repo: web::Data<Arc<SqliteInviteRepository>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();

    match repo.get_by_id(id).await {
        Ok(invite) if invite.user == user.sub => HttpResponse::Ok().json(invite),

        Ok(_) => error_response("Forbidden", actix_web::http::StatusCode::FORBIDDEN),

        Err(InviteRepositoryError::NotFound) => {
            error_response("Invite not found", actix_web::http::StatusCode::NOT_FOUND)
        }

        Err(e) => error_response(e, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[get("/invites")]
async fn list_invites(
    Auth(user): Auth<Identity>,
    repo: web::Data<Arc<SqliteInviteRepository>>,
) -> impl Responder {
    match repo.list_by_user(user.sub).await {
        Ok(invites) => HttpResponse::Ok().json(invites),

        Err(e) => error_response(e, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}
#[post("/invites/{id}/accept")]
async fn accept_invite(
    Auth(user): Auth<Identity>,
    invite_repo: web::Data<Arc<SqliteInviteRepository>>,
    membership_repo: web::Data<Arc<SqliteMembershipRepository>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let invite_id = path.into_inner();

    let invite = match invite_repo.get_by_id(invite_id).await {
        Ok(invite) => invite,

        Err(InviteRepositoryError::NotFound) => {
            return error_response("Invite not found", actix_web::http::StatusCode::NOT_FOUND);
        }

        Err(e) => {
            return error_response(e, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Only the invited user may accept.
    if invite.user != user.sub {
        return error_response("Forbidden", actix_web::http::StatusCode::FORBIDDEN);
    }

    // Optional: enforce expiration.
    if invite.exp < chrono::Utc::now() {
        return error_response("Invite expired", actix_web::http::StatusCode::BAD_REQUEST);
    }

    match membership_repo
        .create(CreateMembership {
            community: invite.community,
            member: invite.user,
        })
        .await
    {
        Ok(_) => {}

        Err(e) => {
            return error_response(e, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Consume the invite.
    if let Err(e) = invite_repo.delete(invite_id).await {
        return error_response(e, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    HttpResponse::NoContent().finish()
}

#[delete("/invites/{id}")]
async fn delete_invite(
    Auth(user): Auth<Identity>,
    repo: web::Data<Arc<SqliteInviteRepository>>,
    membership_repo: web::Data<Arc<SqliteMembershipRepository>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();

    let invite = match repo.get_by_id(id).await {
        Ok(invite) => invite,

        Err(InviteRepositoryError::NotFound) => {
            return error_response("Invite not found", actix_web::http::StatusCode::NOT_FOUND);
        }

        Err(e) => {
            return error_response(e, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let is_invitee = invite.user == user.sub;

    let is_community_member = assert_member_of(&membership_repo, user.sub, invite.community)
        .await
        .is_ok();

    if !is_invitee && !is_community_member {
        return error_response("Forbidden", actix_web::http::StatusCode::FORBIDDEN);
    }

    match repo.delete(id).await {
        Ok(()) => HttpResponse::NoContent().finish(),

        Err(InviteRepositoryError::NotFound) => {
            error_response("Invite not found", actix_web::http::StatusCode::NOT_FOUND)
        }

        Err(e) => error_response(e, actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// ============ Route Configuration ============

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(create_invite)
        .service(get_invite)
        .service(list_invites)
        .service(delete_invite)
        .service(create_community)
        .service(get_community)
        .service(list_communities)
        .service(update_community)
        .service(delete_community)
        .service(get_membership)
        .service(list_memberships)
        .service(delete_membership);
}
