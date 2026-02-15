use actix_web::HttpRequest;
use loupe::{Error, models::OrgRole};
use uuid::Uuid;
use crate::AppState;

/// Permission level required for an action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    /// Only organization admins can perform this action
    Admin,
    /// Editors and admins can perform this action
    Editor,
    /// All organization members can perform this action (including viewers)
    Viewer,
}

/// Get user context including role for permission checking
///
/// Returns: (user_id, org_id, role)
pub async fn get_user_context(
    state: &AppState,
    req: &HttpRequest,
) -> Result<(Uuid, Uuid, OrgRole), Error> {
    let (user_id, org_id) = crate::routes::auth::get_auth_context(state, req).await?;

    // Fetch user to get role
    let user = state.db.get_user(user_id).await?;

    // Verify user belongs to the expected org (defense in depth)
    if user.org_id != org_id {
        return Err(Error::Unauthorized("User does not belong to this organization".to_string()));
    }

    Ok((user_id, org_id, user.role))
}

/// Check if a role meets the minimum permission requirement
///
/// # Permission Hierarchy
/// - Viewer: Can read all resources in their organization
/// - Editor: Can read and write resources (create, update, delete)
/// - Admin: Full access including user management and org settings
pub fn has_permission(user_role: OrgRole, required: Permission) -> bool {
    match required {
        Permission::Viewer => true, // All roles can view
        Permission::Editor => matches!(user_role, OrgRole::Editor | OrgRole::Admin),
        Permission::Admin => matches!(user_role, OrgRole::Admin),
    }
}

/// Require a specific permission level, returning Forbidden error if not met
///
/// # Example
/// ```
/// let (user_id, org_id, role) = get_user_context(&state, &req).await?;
/// require_permission(role, Permission::Editor)?;
/// // Now we know the user is an Editor or Admin
/// ```
pub fn require_permission(user_role: OrgRole, required: Permission) -> Result<(), Error> {
    if has_permission(user_role, required) {
        Ok(())
    } else {
        let required_str = match required {
            Permission::Admin => "Admin",
            Permission::Editor => "Editor or Admin",
            Permission::Viewer => "any role",
        };
        Err(Error::Forbidden(format!(
            "This action requires {} permission. Your role does not have sufficient privileges.",
            required_str
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewer_permissions() {
        // Viewers can only view
        assert!(has_permission(OrgRole::Viewer, Permission::Viewer));
        assert!(!has_permission(OrgRole::Viewer, Permission::Editor));
        assert!(!has_permission(OrgRole::Viewer, Permission::Admin));
    }

    #[test]
    fn test_editor_permissions() {
        // Editors can view and edit
        assert!(has_permission(OrgRole::Editor, Permission::Viewer));
        assert!(has_permission(OrgRole::Editor, Permission::Editor));
        assert!(!has_permission(OrgRole::Editor, Permission::Admin));
    }

    #[test]
    fn test_admin_permissions() {
        // Admins can do everything
        assert!(has_permission(OrgRole::Admin, Permission::Viewer));
        assert!(has_permission(OrgRole::Admin, Permission::Editor));
        assert!(has_permission(OrgRole::Admin, Permission::Admin));
    }

    #[test]
    fn test_require_permission() {
        // Viewer trying to edit should fail
        let result = require_permission(OrgRole::Viewer, Permission::Editor);
        assert!(result.is_err());

        // Editor trying to edit should succeed
        let result = require_permission(OrgRole::Editor, Permission::Editor);
        assert!(result.is_ok());

        // Admin trying anything should succeed
        assert!(require_permission(OrgRole::Admin, Permission::Admin).is_ok());
        assert!(require_permission(OrgRole::Admin, Permission::Editor).is_ok());
        assert!(require_permission(OrgRole::Admin, Permission::Viewer).is_ok());
    }
}
