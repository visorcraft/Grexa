// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: Apache-2.0

//! Schema parsing for collection `schema.md` files.
//!
//! A schema lives in the YAML frontmatter of a `schema.md` file and defines
//! the collection name and its typed fields. The body of the file is
//! free-form human documentation.
//!
//! ```yaml
//! ---
//! collection: notes
//! fields:
//!   - { name: title,   type: string,           required: true }
//!   - { name: tags,    type: "array<string>" }
//!   - { name: rating,  type: integer,          range: [1, 5] }
//!   - { name: source,  type: "ref<bookmarks>", optional: true }
//! ---
//! ```

use crate::frontmatter;
use serde_yaml::Value;
use std::collections::HashSet;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("schema frontmatter is not a YAML mapping")]
    NotAMapping,
    #[error("schema is missing the `collection` field")]
    MissingCollection,
    #[error("schema is missing the `fields` sequence")]
    MissingFields,
    #[error("field definition is not a YAML mapping")]
    FieldNotAMapping,
    #[error("field is missing `name`")]
    MissingFieldName,
    #[error("field is missing `type`")]
    MissingFieldType,
    #[error("unknown field type: `{0}`")]
    UnknownType(String),
    #[error("invalid field type syntax: `{0}`")]
    InvalidType(String),
    #[error("duplicate field name: `{0}`")]
    DuplicateField(String),
    #[error("`required` and `optional` are both true for field `{0}`")]
    ContradictoryOptionality(String),
    #[error("frontmatter error: {0}")]
    Frontmatter(String),
    #[error("schema file has no frontmatter")]
    NoFrontmatter,
}

/// The set of types a field can hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    Date,
    DateTime,
    Array(Box<FieldType>),
    Enum(Vec<String>),
    Ref(String),
}

impl FieldType {
    /// Parse a type string like `"array<string>"`, `"enum<a|b|c>"`,
    /// `"ref<bookmarks>"`, or `"integer"` into a [`FieldType`].
    pub fn parse(s: &str) -> Result<Self, SchemaError> {
        let s = s.trim();

        if let Some(inner) = s.strip_prefix("array<").and_then(|i| i.strip_suffix('>')) {
            return Ok(FieldType::Array(Box::new(FieldType::parse(inner)?)));
        }

        if let Some(inner) = s.strip_prefix("enum<").and_then(|i| i.strip_suffix('>')) {
            let variants: Vec<String> = inner.split('|').map(|v| v.trim().to_string()).collect();
            if variants.is_empty() || variants.iter().any(String::is_empty) {
                return Err(SchemaError::InvalidType(s.into()));
            }
            return Ok(FieldType::Enum(variants));
        }

        if let Some(inner) = s.strip_prefix("ref<").and_then(|i| i.strip_suffix('>')) {
            if inner.trim().is_empty() {
                return Err(SchemaError::InvalidType(s.into()));
            }
            return Ok(FieldType::Ref(inner.trim().to_string()));
        }

        match s {
            "string" => Ok(FieldType::String),
            "integer" => Ok(FieldType::Integer),
            "float" => Ok(FieldType::Float),
            "boolean" => Ok(FieldType::Boolean),
            "date" => Ok(FieldType::Date),
            "datetime" => Ok(FieldType::DateTime),
            _ => Err(SchemaError::UnknownType(s.into())),
        }
    }
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldType::String => write!(f, "string"),
            FieldType::Integer => write!(f, "integer"),
            FieldType::Float => write!(f, "float"),
            FieldType::Boolean => write!(f, "boolean"),
            FieldType::Date => write!(f, "date"),
            FieldType::DateTime => write!(f, "datetime"),
            FieldType::Array(t) => write!(f, "array<{}>", t),
            FieldType::Enum(variants) => write!(f, "enum<{}>", variants.join("|")),
            FieldType::Ref(coll) => write!(f, "ref<{}>", coll),
        }
    }
}

/// A single field definition within a [`Schema`].
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub range: Option<(f64, f64)>,
}

/// A parsed collection schema — the frontmatter of a `schema.md` file.
#[derive(Debug, Clone)]
pub struct Schema {
    pub collection: String,
    pub fields: Vec<FieldDef>,
}

impl Schema {
    /// Parse a schema from a `schema.md` file's full content (frontmatter +
    /// body). The body is ignored; only the frontmatter is parsed.
    pub fn from_markdown(content: &str) -> Result<Self, SchemaError> {
        let split =
            frontmatter::split(content).map_err(|e| SchemaError::Frontmatter(e.to_string()))?;
        let fm = split.frontmatter.ok_or(SchemaError::NoFrontmatter)?;
        Self::from_value(&fm)
    }

    /// Parse a schema from a pre-parsed YAML frontmatter value.
    pub fn from_value(value: &Value) -> Result<Self, SchemaError> {
        let mapping = value.as_mapping().ok_or(SchemaError::NotAMapping)?;

        let collection = mapping
            .get(Value::String("collection".into()))
            .and_then(Value::as_str)
            .ok_or(SchemaError::MissingCollection)?;

        let fields_seq = mapping
            .get(Value::String("fields".into()))
            .and_then(Value::as_sequence)
            .ok_or(SchemaError::MissingFields)?;

        let mut fields = Vec::with_capacity(fields_seq.len());
        let mut seen: HashSet<String> = HashSet::new();

        for field_val in fields_seq {
            let field_map = field_val
                .as_mapping()
                .ok_or(SchemaError::FieldNotAMapping)?;

            let name = field_map
                .get(Value::String("name".into()))
                .and_then(Value::as_str)
                .ok_or(SchemaError::MissingFieldName)?;

            if !seen.insert(name.to_string()) {
                return Err(SchemaError::DuplicateField(name.into()));
            }

            let type_str = field_map
                .get(Value::String("type".into()))
                .and_then(Value::as_str)
                .ok_or(SchemaError::MissingFieldType)?;

            let field_type = FieldType::parse(type_str)?;

            let required = field_map
                .get(Value::String("required".into()))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let optional = field_map
                .get(Value::String("optional".into()))
                .and_then(Value::as_bool)
                .unwrap_or(false);

            if required && optional {
                return Err(SchemaError::ContradictoryOptionality(name.into()));
            }

            let range = if matches!(field_type, FieldType::Integer | FieldType::Float) {
                field_map
                    .get(Value::String("range".into()))
                    .and_then(Value::as_sequence)
                    .and_then(|seq| {
                        if seq.len() == 2 {
                            let min = seq[0].as_f64()?;
                            let max = seq[1].as_f64()?;
                            Some((min, max))
                        } else {
                            None
                        }
                    })
            } else {
                None
            };

            fields.push(FieldDef {
                name: name.to_string(),
                field_type,
                required,
                range,
            });
        }

        Ok(Self {
            collection: collection.to_string(),
            fields,
        })
    }

    /// Look up a field definition by name.
    pub fn field(&self, name: &str) -> Option<&FieldDef> {
        self.fields.iter().find(|f| f.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_primitive_types() {
        assert_eq!(FieldType::parse("string").unwrap(), FieldType::String);
        assert_eq!(FieldType::parse("integer").unwrap(), FieldType::Integer);
        assert_eq!(FieldType::parse("float").unwrap(), FieldType::Float);
        assert_eq!(FieldType::parse("boolean").unwrap(), FieldType::Boolean);
        assert_eq!(FieldType::parse("date").unwrap(), FieldType::Date);
        assert_eq!(FieldType::parse("datetime").unwrap(), FieldType::DateTime);
    }

    #[test]
    fn parse_array_type() {
        assert_eq!(
            FieldType::parse("array<string>").unwrap(),
            FieldType::Array(Box::new(FieldType::String))
        );
        assert_eq!(
            FieldType::parse("array<integer>").unwrap(),
            FieldType::Array(Box::new(FieldType::Integer))
        );
    }

    #[test]
    fn parse_nested_array() {
        assert_eq!(
            FieldType::parse("array<array<string>>").unwrap(),
            FieldType::Array(Box::new(FieldType::Array(Box::new(FieldType::String))))
        );
    }

    #[test]
    fn parse_enum_type() {
        assert_eq!(
            FieldType::parse("enum<draft|published|archived>").unwrap(),
            FieldType::Enum(vec!["draft".into(), "published".into(), "archived".into()])
        );
    }

    #[test]
    fn parse_ref_type() {
        assert_eq!(FieldType::parse("ref<bookmarks>").unwrap(), FieldType::Ref("bookmarks".into()));
    }

    #[test]
    fn parse_ref_with_spaces() {
        assert_eq!(
            FieldType::parse("ref< my_collection >").unwrap(),
            FieldType::Ref("my_collection".into())
        );
    }

    #[test]
    fn empty_ref_is_error() {
        assert!(matches!(FieldType::parse("ref<>"), Err(SchemaError::InvalidType(_))));
    }

    #[test]
    fn empty_enum_is_error() {
        assert!(matches!(FieldType::parse("enum<>"), Err(SchemaError::InvalidType(_))));
    }

    #[test]
    fn unknown_type_is_error() {
        assert!(matches!(FieldType::parse("blob"), Err(SchemaError::UnknownType(_))));
    }

    #[test]
    fn type_display_roundtrips() {
        let cases = [
            "string",
            "integer",
            "float",
            "boolean",
            "date",
            "datetime",
            "array<string>",
            "array<array<integer>>",
            "enum<a|b|c>",
            "ref<bookmarks>",
        ];
        for s in cases {
            let parsed = FieldType::parse(s).unwrap();
            assert_eq!(parsed.to_string(), s, "roundtrip failed for {s}");
        }
    }

    #[test]
    fn full_schema_from_markdown() {
        let content = "---\ncollection: notes\nfields:\n  - name: title\n    type: string\n    required: true\n  - name: tags\n    type: \"array<string>\"\n  - name: rating\n    type: integer\n    range: [1, 5]\n  - name: source\n    type: \"ref<bookmarks>\"\n    optional: true\n---\n\n# Notes\nHuman docs.\n";
        let schema = Schema::from_markdown(content).unwrap();
        assert_eq!(schema.collection, "notes");
        assert_eq!(schema.fields.len(), 4);

        let title = schema.field("title").unwrap();
        assert!(title.required);
        assert_eq!(title.field_type, FieldType::String);

        let tags = schema.field("tags").unwrap();
        assert!(!tags.required);
        assert_eq!(tags.field_type, FieldType::Array(Box::new(FieldType::String)));

        let rating = schema.field("rating").unwrap();
        assert_eq!(rating.range, Some((1.0, 5.0)));

        let source = schema.field("source").unwrap();
        assert!(!source.required);
    }

    #[test]
    fn flow_style_yaml_in_schema() {
        let content = "---\ncollection: tiny\nfields:\n  - { name: x, type: string }\n---\n";
        let schema = Schema::from_markdown(content).unwrap();
        assert_eq!(schema.collection, "tiny");
        assert_eq!(schema.fields[0].name, "x");
    }

    #[test]
    fn missing_collection_errors() {
        let content = "---\nfields: []\n---\n";
        assert!(matches!(Schema::from_markdown(content), Err(SchemaError::MissingCollection)));
    }

    #[test]
    fn missing_fields_errors() {
        let content = "---\ncollection: notes\n---\n";
        assert!(matches!(Schema::from_markdown(content), Err(SchemaError::MissingFields)));
    }

    #[test]
    fn duplicate_field_name_errors() {
        let content = "---\ncollection: notes\nfields:\n  - { name: x, type: string }\n  - { name: x, type: integer }\n---\n";
        assert!(matches!(Schema::from_markdown(content), Err(SchemaError::DuplicateField(_))));
    }

    #[test]
    fn contradictory_required_and_optional_errors() {
        let content = "---\ncollection: notes\nfields:\n  - { name: x, type: string, required: true, optional: true }\n---\n";
        assert!(matches!(
            Schema::from_markdown(content),
            Err(SchemaError::ContradictoryOptionality(_))
        ));
    }

    #[test]
    fn no_frontmatter_errors() {
        assert!(matches!(
            Schema::from_markdown("no frontmatter here"),
            Err(SchemaError::NoFrontmatter)
        ));
    }

    #[test]
    fn field_lookup_returns_none_for_missing() {
        let content = "---\ncollection: notes\nfields:\n  - { name: x, type: string }\n---\n";
        let schema = Schema::from_markdown(content).unwrap();
        assert!(schema.field("nonexistent").is_none());
    }

    #[test]
    fn range_ignored_for_non_numeric_types() {
        let content =
            "---\ncollection: notes\nfields:\n  - { name: x, type: string, range: [1, 5] }\n---\n";
        let schema = Schema::from_markdown(content).unwrap();
        assert!(schema.field("x").unwrap().range.is_none());
    }

    #[test]
    fn empty_fields_list() {
        let content = "---\ncollection: empty\nfields: []\n---\n";
        let schema = Schema::from_markdown(content).unwrap();
        assert_eq!(schema.collection, "empty");
        assert!(schema.fields.is_empty());
    }
}
