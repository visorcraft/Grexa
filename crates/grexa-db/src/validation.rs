// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: Apache-2.0

//! Schema validation — checks records against their collection's field
//! definitions.
//!
//! Phase 1 adds the `validate_all()` / `validate_record()` methods that
//! the design doc listed as opt-in. Validation is **diagnostic only** —
//! it never modifies records.

use crate::record::Record;
use crate::schema::{FieldDef, FieldType};
use serde_yaml::Value;

/// A single validation problem found in one record.
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub record_path: String,
    pub field: String,
    pub message: String,
}

/// Validate a single record against a list of field definitions.
pub fn validate_record(record: &Record, fields: &[FieldDef]) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    for fd in fields {
        match record.field(&fd.name) {
            None | Some(Value::Null) => {
                if fd.required {
                    errors.push(ValidationError {
                        record_path: record.path().to_string(),
                        field: fd.name.clone(),
                        message: "required field is missing or null".into(),
                    });
                }
            }
            Some(value) => {
                if let Err(msg) = validate_value(value, &fd.field_type) {
                    errors.push(ValidationError {
                        record_path: record.path().to_string(),
                        field: fd.name.clone(),
                        message: msg,
                    });
                    continue;
                }
                if let Some((min, max)) = fd.range
                    && let Err(msg) = validate_range(value, min, max)
                {
                    errors.push(ValidationError {
                        record_path: record.path().to_string(),
                        field: fd.name.clone(),
                        message: msg,
                    });
                }
            }
        }
    }
    errors
}

fn validate_value(value: &Value, field_type: &FieldType) -> Result<(), String> {
    match field_type {
        FieldType::String => {
            value
                .as_str()
                .ok_or_else(|| format!("expected string, got {}", type_name(value)))?;
        }
        FieldType::Integer => {
            value
                .as_i64()
                .ok_or_else(|| format!("expected integer, got {}", type_name(value)))?;
        }
        FieldType::Float => {
            as_f64(value).ok_or_else(|| format!("expected float, got {}", type_name(value)))?;
        }
        FieldType::Boolean => {
            value
                .as_bool()
                .ok_or_else(|| format!("expected boolean, got {}", type_name(value)))?;
        }
        FieldType::Date => {
            let s = value
                .as_str()
                .ok_or_else(|| format!("expected date string, got {}", type_name(value)))?;
            if !is_iso_date(s) {
                return Err(format!("invalid date `{s}` (expected YYYY-MM-DD)"));
            }
        }
        FieldType::DateTime => {
            let s = value
                .as_str()
                .ok_or_else(|| format!("expected datetime string, got {}", type_name(value)))?;
            if !is_iso_datetime(s) {
                return Err(format!("invalid datetime `{s}` (expected RFC 3339)"));
            }
        }
        FieldType::Array(inner) => {
            let seq = value
                .as_sequence()
                .ok_or_else(|| format!("expected array, got {}", type_name(value)))?;
            for (i, item) in seq.iter().enumerate() {
                validate_value(item, inner).map_err(|msg| format!("[{i}]: {msg}"))?;
            }
        }
        FieldType::Enum(variants) => {
            let s = value
                .as_str()
                .ok_or_else(|| format!("expected enum value, got {}", type_name(value)))?;
            if !variants.iter().any(|v| v == s) {
                return Err(format!("`{s}` not in: {}", variants.join(" | ")));
            }
        }
        FieldType::Ref(_) => {
            let s = value
                .as_str()
                .ok_or_else(|| format!("expected ref path string, got {}", type_name(value)))?;
            if s.is_empty()
                || s.starts_with('/')
                || s.contains('\\')
                || s.split('/').any(|c| c == "..")
            {
                return Err(format!("ref `{s}` is empty or has forbidden components"));
            }
        }
    }
    Ok(())
}

fn validate_range(value: &Value, min: f64, max: f64) -> Result<(), String> {
    let n = as_f64(value).ok_or("cannot range-check non-numeric value")?;
    if !n.is_finite() {
        return Err(format!("value {n} is not finite"));
    }
    if n < min || n > max {
        return Err(format!("value {n} outside range [{min}, {max}]"));
    }
    Ok(())
}

use crate::query::as_f64;

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Sequence(_) => "array",
        Value::Mapping(_) => "object",
        _ => "other",
    }
}

fn is_iso_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    if !b[..4].iter().all(u8::is_ascii_digit) {
        return false;
    }
    let month = (b[5] - b'0') * 10 + (b[6] - b'0');
    let day = (b[8] - b'0') * 10 + (b[9] - b'0');
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

fn is_iso_datetime(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() >= 19
        && b[4] == b'-'
        && b[7] == b'-'
        && (b[10] == b'T' || b[10] == b' ')
        && b[13] == b':'
        && b[16] == b':'
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[8..10].iter().all(u8::is_ascii_digit)
        && b[11..13].iter().all(u8::is_ascii_digit)
        && b[14..16].iter().all(u8::is_ascii_digit)
        && b[17..19].iter().all(u8::is_ascii_digit)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(content: &str) -> Record {
        Record::from_content("test.md", content).unwrap()
    }

    fn fd(name: &str, ft: FieldType) -> FieldDef {
        FieldDef {
            name: name.into(),
            field_type: ft,
            required: false,
            range: None,
        }
    }

    #[test]
    fn valid_record_no_errors() {
        let r = rec("---\ntitle: Hello\nrating: 5\n---\nbody\n");
        let fields = vec![
            fd("title", FieldType::String),
            fd("rating", FieldType::Integer),
        ];
        assert!(validate_record(&r, &fields).is_empty());
    }

    #[test]
    fn missing_required_field() {
        let r = rec("---\nrating: 5\n---\nbody\n");
        let mut f = fd("title", FieldType::String);
        f.required = true;
        let errors = validate_record(&r, &[f]);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("required"));
    }

    #[test]
    fn missing_optional_field_ok() {
        let r = rec("---\ntitle: Hello\n---\nbody\n");
        let fields = vec![fd("rating", FieldType::Integer)];
        assert!(validate_record(&r, &fields).is_empty());
    }

    #[test]
    fn wrong_type_string_for_integer() {
        let r = rec("---\nrating: high\n---\nbody\n");
        let errors = validate_record(&r, &[fd("rating", FieldType::Integer)]);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("expected integer"));
    }

    #[test]
    fn range_violation() {
        let r = rec("---\nrating: 10\n---\nbody\n");
        let mut f = fd("rating", FieldType::Integer);
        f.range = Some((1.0, 5.0));
        let errors = validate_record(&r, &[f]);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("outside range"));
    }

    #[test]
    fn range_satisfied() {
        let r = rec("---\nrating: 3\n---\nbody\n");
        let mut f = fd("rating", FieldType::Integer);
        f.range = Some((1.0, 5.0));
        assert!(validate_record(&r, &[f]).is_empty());
    }

    #[test]
    fn valid_date() {
        let r = rec("---\nd: 2024-03-15\n---\nbody\n");
        assert!(validate_record(&r, &[fd("d", FieldType::Date)]).is_empty());
    }

    #[test]
    fn invalid_date() {
        let r = rec("---\nd: March 15\n---\nbody\n");
        let errors = validate_record(&r, &[fd("d", FieldType::Date)]);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn enum_valid() {
        let r = rec("---\nstatus: published\n---\nbody\n");
        assert!(
            validate_record(
                &r,
                &[fd(
                    "status",
                    FieldType::Enum(vec!["draft".into(), "published".into()])
                )]
            )
            .is_empty()
        );
    }

    #[test]
    fn enum_invalid() {
        let r = rec("---\nstatus: deleted\n---\nbody\n");
        let errors = validate_record(
            &r,
            &[fd(
                "status",
                FieldType::Enum(vec!["draft".into(), "published".into()]),
            )],
        );
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("not in"));
    }

    #[test]
    fn ref_valid_path() {
        let r = rec("---\nsource: bookmarks/rust.md\n---\nbody\n");
        assert!(
            validate_record(&r, &[fd("source", FieldType::Ref("bookmarks".into()))]).is_empty()
        );
    }

    #[test]
    fn ref_traversal_rejected() {
        let r = rec("---\nsource: ../../../etc/passwd\n---\nbody\n");
        let errors = validate_record(&r, &[fd("source", FieldType::Ref("bookmarks".into()))]);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("forbidden"));
    }

    #[test]
    fn array_type_check() {
        let r = rec("---\ntags: [rust, 5]\n---\nbody\n");
        let errors =
            validate_record(&r, &[fd("tags", FieldType::Array(Box::new(FieldType::String)))]);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("[1]"));
    }
}
