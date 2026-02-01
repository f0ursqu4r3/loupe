use crate::AppState;
use crate::permissions::{get_user_context, require_permission, Permission};
use actix_web::{HttpRequest, HttpResponse, web};
use loupe::Error;
use loupe::models::{OrgRole, UserResponse};
use std::sync::Arc;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/organizations")
            .route("/users", web::get().to(list_organization_users))
            .route("/users/{user_id}/role", web::put().to(update_user_role))
            .route("/users/{user_id}", web::delete().to(remove_user_from_organization)),
    );
}

#[derive(serde::Deserialize)]
pub struct UpdateUserRoleRequest {
    pub role: OrgRole,
}

async fn list_organization_users(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (_, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Viewer)?;

    let users = state.db.list_organization_users(org_id).await?;
    let response: Vec<UserResponse> = users.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(response))
}

async fn update_user_role(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateUserRoleRequest>,
) -> Result<HttpResponse, Error> {
    let (requesting_user_id, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Admin)?;

    let target_user_id = path.into_inner();

    // Prevent users from changing their own role
    if requesting_user_id == target_user_id {
        return Err(Error::BadRequest(
            "You cannot change your own role".to_string(),
        ));
    }

    // Verify the target user belongs to this organization
    state.db.get_user_in_organization(target_user_id, org_id).await?;

    // Update the user's role
    let updated_user = state
        .db
        .update_user_role(target_user_id, org_id, body.role)
        .await?;

    Ok(HttpResponse::Ok().json(UserResponse::from(updated_user)))
}

async fn remove_user_from_organization(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (requesting_user_id, org_id, role) = get_user_context(&state, &req).await?;
    require_permission(role, Permission::Admin)?;

    let target_user_id = path.into_inner();

    // Prevent users from removing themselves
    if requesting_user_id == target_user_id {
        return Err(Error::BadRequest(
            "You cannot remove yourself from the organization".to_string(),
        ));
    }

    // Verify the target user belongs to this organization
    state.db.get_user_in_organization(target_user_id, org_id).await?;

    // Remove the user from the organization
    state
        .db
        .remove_user_from_organization(target_user_id, org_id)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}
