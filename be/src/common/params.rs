//! Parameter binding for SQL queries.
//!
//! Uses `$param_name` syntax for named parameters.
//! Parameters are bound safely via prepared statements.

use crate::error::{Error, Result};
use crate::models::ParamType;
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// Regex to match `$param_name` in SQL.
/// Parameter names must start with a letter and contain only alphanumeric + underscore.
static PARAM_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$([a-zA-Z][a-zA-Z0-9_]*)").unwrap());

/// Extract parameter names from SQL.
pub fn extract_params(sql: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut params = Vec::new();

    for cap in PARAM_REGEX.captures_iter(sql) {
        if let Some(name) = cap.get(1) {
            let name = name.as_str().to_string();
            if seen.insert(name.clone()) {
                params.push(name);
            }
        }
    }

    params
}

/// A typed parameter value ready for binding.
#[derive(Debug, Clone)]
pub enum TypedValue {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Date(chrono::NaiveDate),
    DateTime(chrono::DateTime<chrono::Utc>),
    Null,
}

impl TypedValue {
    /// Convert a JSON value to a typed value based on the declared parameter type.
    pub fn from_json(value: &Value, param_type: &ParamType) -> Result<Self> {
        match (value, param_type) {
            (Value::Null, _) => Ok(TypedValue::Null),

            (Value::String(s), ParamType::String) => Ok(TypedValue::String(s.clone())),

            (Value::Number(n), ParamType::Number) => {
                if let Some(i) = n.as_i64() {
                    Ok(TypedValue::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(TypedValue::Number(f))
                } else {
                    Err(Error::BadRequest("Invalid number value".into()))
                }
            }

            (Value::String(s), ParamType::Number) => s
                .parse::<f64>()
                .map(TypedValue::Number)
                .map_err(|_| Error::BadRequest(format!("Cannot parse '{}' as number", s))),

            (Value::Bool(b), ParamType::Boolean) => Ok(TypedValue::Boolean(*b)),

            (Value::String(s), ParamType::Boolean) => match s.to_lowercase().as_str() {
                "true" | "1" | "yes" => Ok(TypedValue::Boolean(true)),
                "false" | "0" | "no" => Ok(TypedValue::Boolean(false)),
                _ => Err(Error::BadRequest(format!(
                    "Cannot parse '{}' as boolean",
                    s
                ))),
            },

            (Value::String(s), ParamType::Date) => chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map(TypedValue::Date)
                .map_err(|_| {
                    Error::BadRequest(format!("Cannot parse '{}' as date (YYYY-MM-DD)", s))
                }),

            (Value::String(s), ParamType::DateTime) => {
                // Try ISO 8601 formats
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                    Ok(TypedValue::DateTime(dt.with_timezone(&chrono::Utc)))
                } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
                {
                    Ok(TypedValue::DateTime(dt.and_utc()))
                } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                {
                    Ok(TypedValue::DateTime(dt.and_utc()))
                } else {
                    Err(Error::BadRequest(format!(
                        "Cannot parse '{}' as datetime",
                        s
                    )))
                }
            }

            _ => Err(Error::BadRequest(format!(
                "Type mismatch: expected {:?}, got {:?}",
                param_type, value
            ))),
        }
    }

    /// Format for use in SQL literal (fallback for non-prepared queries).
    /// This is properly escaped but prepared statements are preferred.
    pub fn to_sql_literal(&self) -> String {
        match self {
            TypedValue::String(s) => format!("'{}'", s.replace('\'', "''")),
            TypedValue::Number(n) => n.to_string(),
            TypedValue::Integer(i) => i.to_string(),
            TypedValue::Boolean(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            TypedValue::Date(d) => format!("'{}'", d),
            TypedValue::DateTime(dt) => format!("'{}'", dt.to_rfc3339()),
            TypedValue::Null => "NULL".to_string(),
        }
    }
}

/// Parameter definitions from query metadata.
#[derive(Debug, Clone)]
pub struct ParamSchema {
    pub name: String,
    pub param_type: ParamType,
    pub required: bool,
    pub default: Option<Value>,
}

/// Resolved parameter values after validation.
pub struct BoundParams {
    /// The SQL with $name replaced by $1, $2, etc.
    pub sql: String,
    /// Values in positional order matching $1, $2, etc.
    pub values: Vec<TypedValue>,
    /// Original name to position mapping (for debugging)
    pub positions: HashMap<String, usize>,
}

/// Bind parameter values to a SQL query.
///
/// Converts named parameters ($name) to positional ($1, $2) and validates/coerces values.
pub fn bind_params(
    sql: &str,
    schema: &[ParamSchema],
    values: &HashMap<String, Value>,
) -> Result<BoundParams> {
    let schema_map: HashMap<_, _> = schema.iter().map(|p| (p.name.as_str(), p)).collect();

    let sql_params = extract_params(sql);

    // Validate all SQL params have definitions
    for param in &sql_params {
        if !schema_map.contains_key(param.as_str()) {
            return Err(Error::BadRequest(format!(
                "Parameter '{}' used in SQL but not defined",
                param
            )));
        }
    }

    // Build positional mapping and values
    let mut positions = HashMap::new();
    let mut typed_values = Vec::new();
    let mut bound_sql = sql.to_string();

    for (idx, param_name) in sql_params.iter().enumerate() {
        let schema = schema_map
            .get(param_name.as_str())
            .ok_or_else(|| Error::BadRequest(format!("Unknown parameter: {}", param_name)))?;

        // Get value from provided values or use default
        let value = values
            .get(param_name)
            .or(schema.default.as_ref())
            .ok_or_else(|| {
                if schema.required {
                    Error::BadRequest(format!("Required parameter '{}' not provided", param_name))
                } else {
                    Error::BadRequest(format!(
                        "Parameter '{}' has no value or default",
                        param_name
                    ))
                }
            })?;

        let typed = TypedValue::from_json(value, &schema.param_type)?;
        typed_values.push(typed);
        positions.insert(param_name.clone(), idx + 1);

        // Replace $name with $N (positional)
        let pattern = format!("${}", param_name);
        let replacement = format!("${}", idx + 1);
        bound_sql = bound_sql.replace(&pattern, &replacement);
    }

    Ok(BoundParams {
        sql: bound_sql,
        values: typed_values,
        positions,
    })
}

/// Simple substitution for SQL execution that doesn't support prepared statements.
/// Falls back to escaped literals. Use `bind_params` + prepared statements when possible.
pub fn substitute_params(
    sql: &str,
    schema: &[ParamSchema],
    values: &HashMap<String, Value>,
) -> Result<String> {
    let schema_map: HashMap<_, _> = schema.iter().map(|p| (p.name.as_str(), p)).collect();
    let mut result = sql.to_string();

    for cap in PARAM_REGEX.captures_iter(sql) {
        if let Some(name_match) = cap.get(1) {
            let param_name = name_match.as_str();

            let schema = schema_map
                .get(param_name)
                .ok_or_else(|| Error::BadRequest(format!("Unknown parameter: {}", param_name)))?;

            let value = values
                .get(param_name)
                .or(schema.default.as_ref())
                .ok_or_else(|| {
                    Error::BadRequest(format!("Parameter '{}' not provided", param_name))
                })?;

            let typed = TypedValue::from_json(value, &schema.param_type)?;
            let literal = typed.to_sql_literal();

            let pattern = format!("${}", param_name);
            result = result.replace(&pattern, &literal);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_params() {
        let sql = "SELECT * FROM orders WHERE status = $status AND date > $start_date";
        let params = extract_params(sql);
        assert_eq!(params, vec!["status", "start_date"]);
    }

    #[test]
    fn test_extract_params_dedupes() {
        let sql = "SELECT * FROM t WHERE a = $x OR b = $x";
        let params = extract_params(sql);
        assert_eq!(params, vec!["x"]);
    }

    #[test]
    fn test_no_params() {
        let sql = "SELECT * FROM orders";
        let params = extract_params(sql);
        assert!(params.is_empty());
    }

    #[test]
    fn test_bind_params_positional() {
        let sql = "SELECT * FROM t WHERE a = $foo AND b = $bar";
        let schema = vec![
            ParamSchema {
                name: "foo".into(),
                param_type: ParamType::String,
                required: true,
                default: None,
            },
            ParamSchema {
                name: "bar".into(),
                param_type: ParamType::Number,
                required: true,
                default: None,
            },
        ];
        let mut values = HashMap::new();
        values.insert("foo".into(), Value::String("hello".into()));
        values.insert("bar".into(), Value::Number(42.into()));

        let bound = bind_params(sql, &schema, &values).unwrap();

        assert_eq!(bound.sql, "SELECT * FROM t WHERE a = $1 AND b = $2");
        assert_eq!(bound.values.len(), 2);
    }

    #[test]
    fn test_substitute_params() {
        let sql = "SELECT * FROM t WHERE name = $name AND active = $active";
        let schema = vec![
            ParamSchema {
                name: "name".into(),
                param_type: ParamType::String,
                required: true,
                default: None,
            },
            ParamSchema {
                name: "active".into(),
                param_type: ParamType::Boolean,
                required: true,
                default: None,
            },
        ];
        let mut values = HashMap::new();
        values.insert("name".into(), Value::String("O'Brien".into()));
        values.insert("active".into(), Value::Bool(true));

        let result = substitute_params(sql, &schema, &values).unwrap();

        assert_eq!(
            result,
            "SELECT * FROM t WHERE name = 'O''Brien' AND active = TRUE"
        );
    }

    #[test]
    fn test_missing_required_param() {
        let sql = "SELECT * FROM t WHERE a = $foo";
        let schema = vec![ParamSchema {
            name: "foo".into(),
            param_type: ParamType::String,
            required: true,
            default: None,
        }];
        let values = HashMap::new();

        let result = bind_params(sql, &schema, &values);
        assert!(result.is_err());
    }

    #[test]
    fn test_uses_default_value() {
        let sql = "SELECT * FROM t WHERE a = $foo";
        let schema = vec![ParamSchema {
            name: "foo".into(),
            param_type: ParamType::String,
            required: false,
            default: Some(Value::String("default_val".into())),
        }];
        let values = HashMap::new();

        let bound = bind_params(sql, &schema, &values).unwrap();
        assert!(matches!(bound.values[0], TypedValue::String(ref s) if s == "default_val"));
    }
}
