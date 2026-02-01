use crate::error::{Error, Result};
use sqlparser::ast::{Expr, Function, FunctionArg, FunctionArgExpr, ObjectName, Query, Select, SelectItem, SetExpr, Statement, TableFactor, TableWithJoins, Visit, Visitor};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

/// SQL Validator for preventing SQL injection and restricting dangerous operations
pub struct SqlValidator {
    max_query_length: usize,
    allow_subqueries: bool,
}

impl SqlValidator {
    /// Create a new SQL validator with default settings
    pub fn new() -> Self {
        Self {
            max_query_length: 100_000, // 100KB max
            allow_subqueries: true,
        }
    }

    /// Create a new SQL validator with custom settings
    pub fn with_config(max_query_length: usize, allow_subqueries: bool) -> Self {
        Self {
            max_query_length,
            allow_subqueries,
        }
    }

    /// Validate SQL query and return parsed AST if safe
    pub fn validate(&self, sql: &str) -> Result<Vec<Statement>> {
        // Check length first
        if sql.len() > self.max_query_length {
            return Err(Error::BadRequest(format!(
                "SQL query exceeds maximum length of {} characters",
                self.max_query_length
            )));
        }

        // Parse SQL
        let dialect = PostgreSqlDialect {};
        let statements = Parser::parse_sql(&dialect, sql).map_err(|e| {
            tracing::warn!(error = %e, "SQL parsing failed");
            Error::BadRequest(format!("Invalid SQL syntax: {}", e))
        })?;

        // Validate each statement
        for statement in &statements {
            self.validate_statement(statement)?;
        }

        Ok(statements)
    }

    /// Validate a single SQL statement
    fn validate_statement(&self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Query(_query) => {
                // SELECT statements are allowed
                self.validate_query_safety(stmt)?;
                Ok(())
            }
            _ => {
                // Block all non-SELECT statements
                Err(Error::BadRequest(
                    "Only SELECT queries are allowed. DROP, INSERT, UPDATE, DELETE, ALTER, CREATE, and other modification statements are forbidden.".to_string(),
                ))
            }
        }
    }

    /// Validate that query doesn't use dangerous functions
    fn validate_query_safety(&self, stmt: &Statement) -> Result<()> {
        let mut visitor = DangerousVisitor::new();
        let _ = stmt.visit(&mut visitor);

        if !visitor.dangerous_functions.is_empty() {
            return Err(Error::BadRequest(format!(
                "Dangerous function(s) detected: {}. These functions are not allowed for security reasons.",
                visitor.dangerous_functions.join(", ")
            )));
        }

        Ok(())
    }
}

impl Default for SqlValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Visitor to detect dangerous SQL functions and patterns
struct DangerousVisitor {
    dangerous_functions: Vec<String>,
}

impl DangerousVisitor {
    fn new() -> Self {
        Self {
            dangerous_functions: Vec::new(),
        }
    }

    fn check_function_name(&mut self, name: &ObjectName) {
        let function_name = name.to_string().to_lowercase();

        // List of dangerous PostgreSQL functions
        let dangerous = [
            // File system access
            "pg_read_file",
            "pg_read_binary_file",
            "pg_ls_dir",
            "pg_stat_file",
            // Command execution
            "pg_execute_server_program",
            "copy",
            // Network access
            "dblink",
            "dblink_connect",
            "dblink_exec",
            // Administrative functions
            "pg_terminate_backend",
            "pg_cancel_backend",
            "pg_reload_conf",
            "pg_rotate_logfile",
            // Large object functions
            "lo_import",
            "lo_export",
            "lo_unlink",
            // Extension loading
            "pg_create_extension",
            "pg_drop_extension",
            // Crypto/encoding that could be used for attacks
            "pg_crypto",
            // XML/External entity attacks
            "xmlparse",
            "xpath",
            // Generic execution
            "execute",
            // User/role management
            "pg_create_user",
            "pg_drop_user",
            "pg_create_role",
            "pg_drop_role",
        ];

        for &dangerous_name in &dangerous {
            if function_name.contains(dangerous_name) {
                self.dangerous_functions.push(function_name.clone());
                break;
            }
        }
    }
}

impl Visitor for DangerousVisitor {
    type Break = ();

    fn pre_visit_expr(&mut self, expr: &Expr) -> std::ops::ControlFlow<Self::Break> {
        if let Expr::Function(func) = expr {
            self.check_function_name(&func.name);
        }
        std::ops::ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_select() {
        let validator = SqlValidator::new();
        let sql = "SELECT id, name, email FROM users WHERE active = true";
        assert!(validator.validate(sql).is_ok());
    }

    #[test]
    fn test_valid_join() {
        let validator = SqlValidator::new();
        let sql = "SELECT u.name, o.title FROM users u JOIN orders o ON u.id = o.user_id";
        assert!(validator.validate(sql).is_ok());
    }

    #[test]
    fn test_valid_aggregate() {
        let validator = SqlValidator::new();
        let sql = "SELECT COUNT(*), AVG(price) FROM products GROUP BY category";
        assert!(validator.validate(sql).is_ok());
    }

    #[test]
    fn test_invalid_insert() {
        let validator = SqlValidator::new();
        let sql = "INSERT INTO users (name) VALUES ('test')";
        let result = validator.validate(sql);
        assert!(result.is_err());
        if let Err(Error::BadRequest(msg)) = result {
            assert!(msg.contains("Only SELECT queries are allowed"));
        }
    }

    #[test]
    fn test_invalid_drop() {
        let validator = SqlValidator::new();
        let sql = "DROP TABLE users";
        let result = validator.validate(sql);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_update() {
        let validator = SqlValidator::new();
        let sql = "UPDATE users SET name = 'hacked' WHERE id = 1";
        let result = validator.validate(sql);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_delete() {
        let validator = SqlValidator::new();
        let sql = "DELETE FROM users WHERE id = 1";
        let result = validator.validate(sql);
        assert!(result.is_err());
    }

    #[test]
    fn test_dangerous_function_pg_read_file() {
        let validator = SqlValidator::new();
        let sql = "SELECT pg_read_file('/etc/passwd')";
        let result = validator.validate(sql);
        assert!(result.is_err());
        if let Err(Error::BadRequest(msg)) = result {
            assert!(msg.contains("Dangerous function"));
        }
    }

    #[test]
    fn test_dangerous_function_pg_ls_dir() {
        let validator = SqlValidator::new();
        let sql = "SELECT pg_ls_dir('/')";
        let result = validator.validate(sql);
        assert!(result.is_err());
    }

    #[test]
    fn test_query_too_long() {
        let validator = SqlValidator::with_config(100, true);
        let sql = format!("SELECT {}", "a,".repeat(1000));
        let result = validator.validate(&sql);
        assert!(result.is_err());
        if let Err(Error::BadRequest(msg)) = result {
            assert!(msg.contains("exceeds maximum length"));
        }
    }

    #[test]
    fn test_valid_subquery() {
        let validator = SqlValidator::new();
        let sql = "SELECT * FROM (SELECT id, name FROM users) AS subq WHERE id > 10";
        assert!(validator.validate(sql).is_ok());
    }

    #[test]
    fn test_case_expression() {
        let validator = SqlValidator::new();
        let sql = "SELECT CASE WHEN age > 18 THEN 'adult' ELSE 'minor' END FROM users";
        assert!(validator.validate(sql).is_ok());
    }

    #[test]
    fn test_valid_functions() {
        let validator = SqlValidator::new();

        // Common safe functions should work
        assert!(validator.validate("SELECT COUNT(*) FROM users").is_ok());
        assert!(validator.validate("SELECT SUM(price) FROM orders").is_ok());
        assert!(validator.validate("SELECT AVG(rating) FROM reviews").is_ok());
        assert!(validator.validate("SELECT UPPER(name) FROM users").is_ok());
        assert!(validator.validate("SELECT NOW()").is_ok());
        assert!(validator.validate("SELECT COALESCE(name, 'Unknown') FROM users").is_ok());
    }
}
