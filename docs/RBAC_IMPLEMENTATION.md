# Role-Based Access Control (RBAC) Implementation

**Date:** January 31, 2026
**Status:** ✅ Implemented for core routes

---

## Overview

Loupe now enforces role-based permissions across all API endpoints. Every user in an organization has one of three roles that determine what actions they can perform.

## Roles & Permissions

### Permission Hierarchy

```text
Admin   ──► Full access (create, read, update, delete everything)
  │
  ▼
Editor  ──► Create and modify resources, execute queries
  │
  ▼
Viewer  ──► Read-only access to all resources
```

### Role Definitions

| Role       | Capabilities                                                                                                                                                  |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Admin**  | - All Editor permissions<br>- Create/edit/delete datasources<br>- Manage organization settings<br>- Manage users and roles<br>- Access sensitive operations   |
| **Editor** | - All Viewer permissions<br>- Create/edit/delete dashboards, queries, visualizations<br>- Execute ad-hoc SQL queries<br>- Create scheduled jobs               |
| **Viewer** | - Read all resources (dashboards, queries, visualizations, etc.)<br>- Execute existing saved queries<br>- View query results<br>- Test datasource connections |

---

## Implementation Details

### Core Module

**File:** [`be/src/api/permissions.rs`](../be/src/api/permissions.rs)

#### Key Functions

**`get_user_context`** - Extract user, org, and role from request

```rust
pub async fn get_user_context(
    state: &AppState,
    req: &HttpRequest,
) -> Result<(Uuid, Uuid, OrgRole), Error>
```

**`has_permission`** - Check if role meets requirement

```rust
pub fn has_permission(user_role: OrgRole, required: Permission) -> bool
```

**`require_permission`** - Enforce permission or return error

```rust
pub fn require_permission(user_role: OrgRole, required: Permission) -> Result<(), Error>
```

#### Permission Enum

```rust
pub enum Permission {
    Admin,   // Only admins
    Editor,  // Editors and admins
    Viewer,  // All roles (admins, editors, viewers)
}
```

### Route-Level Enforcement

All route handlers now follow this pattern:

```rust
async fn handler(
    state: web::Data<Arc<AppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    // 1. Get user context including role
    let (user_id, org_id, role) = get_user_context(&state, &req).await?;

    // 2. Enforce required permission
    require_permission(role, Permission::Editor)?;

    // 3. Proceed with business logic
    // ...
}
```

---

## Endpoint Permissions

### Dashboards (`/api/v1/dashboards`)

| Endpoint              | Method | Permission | Description           |
| --------------------- | ------ | ---------- | --------------------- |
| `/`                   | GET    | Viewer     | List all dashboards   |
| `/`                   | POST   | Editor     | Create dashboard      |
| `/:id`                | GET    | Viewer     | Get dashboard details |
| `/:id`                | PUT    | Editor     | Update dashboard      |
| `/:id`                | DELETE | Editor     | Delete dashboard      |
| `/:id/tiles`          | POST   | Editor     | Add tile to dashboard |
| `/:id/tiles/:tile_id` | PUT    | Editor     | Update tile           |
| `/:id/tiles/:tile_id` | DELETE | Editor     | Remove tile           |

### Queries (`/api/v1/queries`)

| Endpoint  | Method | Permission | Description       |
| --------- | ------ | ---------- | ----------------- |
| `/`       | GET    | Viewer     | List all queries  |
| `/`       | POST   | Editor     | Create query      |
| `/export` | GET    | Viewer     | Export queries    |
| `/import` | POST   | Editor     | Import queries    |
| `/:id`    | GET    | Viewer     | Get query details |
| `/:id`    | PUT    | Editor     | Update query      |
| `/:id`    | DELETE | Editor     | Delete query      |

### Runs (`/api/v1/runs`)

| Endpoint      | Method | Permission | Description          |
| ------------- | ------ | ---------- | -------------------- |
| `/`           | GET    | Viewer     | List query runs      |
| `/`           | POST   | Viewer     | Execute saved query  |
| `/execute`    | POST   | **Editor** | Execute ad-hoc SQL ⚠️ |
| `/:id`        | GET    | Viewer     | Get run details      |
| `/:id/result` | GET    | Viewer     | Get run results      |

**Note:** Ad-hoc SQL execution requires Editor role for security.

### Datasources (`/api/v1/datasources`)

| Endpoint      | Method | Permission | Description            |
| ------------- | ------ | ---------- | ---------------------- |
| `/`           | GET    | Viewer     | List datasources       |
| `/`           | POST   | **Admin**  | Create datasource ⚠️    |
| `/:id`        | GET    | Viewer     | Get datasource details |
| `/:id`        | PUT    | **Admin**  | Update datasource ⚠️    |
| `/:id`        | DELETE | **Admin**  | Delete datasource ⚠️    |
| `/:id/test`   | POST   | Viewer     | Test connection        |
| `/:id/schema` | GET    | Viewer     | Get database schema    |

**Note:** Datasource management requires Admin role due to sensitive connection strings.

### Visualizations (`/api/v1/visualizations`)

| Endpoint | Method | Permission | Description               |
| -------- | ------ | ---------- | ------------------------- |
| `/`      | GET    | Viewer     | List all visualizations   |
| `/`      | POST   | Editor     | Create visualization      |
| `/:id`   | GET    | Viewer     | Get visualization details |
| `/:id`   | PUT    | Editor     | Update visualization      |
| `/:id`   | DELETE | Editor     | Delete visualization      |

### Schedules (`/api/v1/schedules`)

| Endpoint       | Method | Permission | Description        |
| -------------- | ------ | ---------- | ------------------ |
| `/`            | GET    | Viewer     | List all schedules |
| `/`            | POST   | Editor     | Create schedule    |
| `/:id`         | GET    | Viewer     | Get schedule       |
| `/:id`         | PATCH  | Editor     | Update schedule    |
| `/:id`         | DELETE | Editor     | Delete schedule    |
| `/:id/enable`  | POST   | Editor     | Enable schedule    |
| `/:id/disable` | POST   | Editor     | Disable schedule   |
| `/:id/trigger` | POST   | Editor     | Trigger schedule   |

### Canvases (`/api/v1/canvases`)

| Endpoint                | Method | Permission | Description      |
| ----------------------- | ------ | ---------- | ---------------- |
| `/`                     | GET    | Viewer     | List canvases    |
| `/`                     | POST   | Editor     | Create canvas    |
| `/:id`                  | GET    | Viewer     | Get canvas       |
| `/:id`                  | PUT    | Editor     | Update canvas    |
| `/:id`                  | DELETE | Editor     | Delete canvas    |
| `/:id/nodes`            | POST   | Editor     | Add node         |
| `/:id/nodes/:node_id`   | PUT    | Editor     | Update node      |
| `/:id/nodes/:node_id`   | DELETE | Editor     | Delete node      |
| `/:id/edges`            | POST   | Editor     | Add edge         |
| `/:id/edges/:edge_id`   | PUT    | Editor     | Update edge      |
| `/:id/edges/:edge_id`   | DELETE | Editor     | Delete edge      |

### Organizations (`/api/v1/organizations`)

| Endpoint                    | Method | Permission  | Description                 |
| --------------------------- | ------ | ----------- | --------------------------- |
| `/users`                    | GET    | Viewer      | List organization users     |
| `/users/:user_id/role`      | PUT    | **Admin** ⚠️ | Update user role            |
| `/users/:user_id`           | DELETE | **Admin** ⚠️ | Remove user from org        |

**Note:** User role management requires Admin permission. Users cannot modify their own role or remove themselves from the organization.

**Business Rules:**
- Admins cannot change their own role (prevents accidental lockout)
- Admins cannot remove themselves from the organization (prevents accidental lockout)
- Only users within the same organization can be modified
- All role changes are logged in the `updated_at` timestamp

---

## Error Responses

### Forbidden (403)

When a user lacks the required permission:

```json
{
  "error": {
    "type": "forbidden",
    "message": "This action requires Editor or Admin permission. Your role does not have sufficient privileges."
  }
}
```

### Unauthorized (401)

When authentication fails:

```json
{
  "error": {
    "type": "unauthorized",
    "message": "Invalid or expired token"
  }
}
```

---

## Security Considerations

### Defense in Depth

1. **JWT Authentication** - All requests must include valid JWT token
2. **Organization Isolation** - Users can only access their org's data
3. **Role-Based Authorization** - Actions are restricted by role
4. **Database-Level Checks** - All queries include `org_id` filter

### Critical Security Points

- **Datasource Management**: Admin-only to prevent connection string exposure
- **Ad-hoc SQL**: Editor-only to prevent unauthorized data access
- **User Role Verification**: Fetched from database on every permission check

---

## Testing

### Unit Tests

Permissions module includes comprehensive tests:

```bash
cd be
cargo test permissions::tests
```

**Test Coverage:**

- ✅ Viewer can only view
- ✅ Editor can view and edit
- ✅ Admin has full access
- ✅ Permission requirements enforced

### Manual Testing

**Test as Viewer:**

```bash
# Should succeed
curl -H "Authorization: Bearer $VIEWER_TOKEN" http://localhost:8080/api/v1/dashboards

# Should fail with 403
curl -X POST -H "Authorization: Bearer $VIEWER_TOKEN" \
  -d '{"name":"Test"}' http://localhost:8080/api/v1/dashboards
```

**Test as Editor:**

```bash
# Should succeed
curl -X POST -H "Authorization: Bearer $EDITOR_TOKEN" \
  -d '{"name":"Test"}' http://localhost:8080/api/v1/dashboards

# Should fail with 403 (datasource creation is Admin-only)
curl -X POST -H "Authorization: Bearer $EDITOR_TOKEN" \
  -d '{"name":"DB", "type":"postgres"}' http://localhost:8080/api/v1/datasources
```

**Test as Admin:**

```bash
# Should succeed (all operations)
curl -X POST -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{"name":"DB", "type":"postgres"}' http://localhost:8080/api/v1/datasources
```

---

## Migration Notes

### Breaking Changes

⚠️ **IMPORTANT:** Users without explicit roles will default to `Viewer`.

### Upgrading Existing Users

If you have existing users in the database:

```sql
-- Set default role for users without one
UPDATE users SET role = 'viewer' WHERE role IS NULL;

-- Promote specific users to editors
UPDATE users SET role = 'editor' WHERE email IN (...);

-- Promote admins
UPDATE users SET role = 'admin' WHERE email IN (...);
```

### Role Assignment

Currently, role assignment happens at user creation. Future enhancements:

- Organization management API (see [BE_TODO #48](./BE_TODO.md#48-organization-management))
- User invitation system with role selection
- Admin UI for role management

---

## Future Enhancements

See [BE_TODO.md](./BE_TODO.md) sections:

1. **Resource-Level Permissions** (planned)
   - Per-dashboard permissions
   - Per-query ownership
   - Fine-grained access control

2. **Organization Management API** (#48)
   - List organization users
   - Update user roles
   - Remove users from organization

3. **Invitation System** (planned)
   - Invite users with specific roles
   - Token-based invitations
   - Email notifications

4. **Audit Logging** (planned)
   - Track role changes
   - Log permission-denied attempts
   - Security event monitoring

---

## Related Documents

- [BE_TODO.md](./BE_TODO.md) - Backend improvement tracking
- [SECURITY_AUDIT_2026-01.md](./SECURITY_AUDIT_2026-01.md) - Security assessment
- [SECURITY_FIXES_2026-01.md](./SECURITY_FIXES_2026-01.md) - Security implementation details

---

## Questions?

For role permission questions or access issues, check:

1. User's role: `SELECT role FROM users WHERE id = 'user-id';`
2. Required permission for endpoint (see tables above)
3. Error logs for permission denials

**Permission Hierarchy Summary:**

- Want to read? → Viewer (or higher)
- Want to create/edit? → Editor (or higher)
- Want to manage datasources/users? → Admin
