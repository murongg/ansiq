use std::collections::{BTreeMap, BTreeSet};

use openapiv3::OpenAPI;
use serde_json::Value;

use crate::openapi_explorer::models::{OpenApiDocumentView, OperationView, SchemaView, TagView};

pub fn parse_document(source_label: &str, text: &str) -> Result<OpenApiDocumentView, String> {
    let value = parse_value(source_label, text)?;
    let _: OpenAPI = serde_json::from_value(value.clone()).map_err(|error| error.to_string())?;

    let title = value
        .get("info")
        .and_then(|info| info.get("title"))
        .and_then(Value::as_str)
        .unwrap_or("OpenAPI document")
        .to_string();

    let mut tags = BTreeMap::<String, Vec<OperationView>>::new();
    let paths = value
        .get("paths")
        .and_then(Value::as_object)
        .ok_or_else(|| "OpenAPI document does not contain paths".to_string())?;

    for (path, item) in paths {
        let Some(item) = item.as_object() else {
            continue;
        };

        for method in [
            "get", "put", "post", "delete", "options", "head", "patch", "trace",
        ] {
            let Some(operation) = item.get(method).and_then(Value::as_object) else {
                continue;
            };

            let summary = operation
                .get("summary")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let description = operation
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();

            let parameters = operation
                .get("parameters")
                .and_then(Value::as_array)
                .map(|items| items.iter().filter_map(parameter_line).collect())
                .unwrap_or_else(Vec::new);

            let request_body = operation
                .get("requestBody")
                .map(request_body_lines)
                .unwrap_or_else(Vec::new);

            let responses = operation
                .get("responses")
                .and_then(Value::as_object)
                .map(|items| {
                    items
                        .iter()
                        .map(|(status, response)| response_line(status, response))
                        .collect()
                })
                .unwrap_or_else(Vec::new);

            let schema_refs = collect_refs(&Value::Object(operation.clone()));
            let tag_name = operation
                .get("tags")
                .and_then(Value::as_array)
                .and_then(|items| items.first())
                .and_then(Value::as_str)
                .unwrap_or("default")
                .to_string();

            let operation_view = OperationView {
                id: format!("{} {}", method.to_uppercase(), path),
                method: method.to_uppercase(),
                path: path.to_string(),
                summary,
                description,
                parameters,
                request_body,
                responses,
                schema_refs,
            };

            tags.entry(tag_name).or_default().push(operation_view);
        }
    }

    let tags = tags
        .into_iter()
        .map(|(name, operations)| TagView { name, operations })
        .collect();

    let schema_index = value
        .get("components")
        .and_then(|components| components.get("schemas"))
        .and_then(Value::as_object)
        .map(build_schema_index)
        .unwrap_or_default();

    Ok(OpenApiDocumentView {
        title,
        source_label: source_label.to_string(),
        tags,
        schema_index,
    })
}

fn parse_value(source_label: &str, text: &str) -> Result<Value, String> {
    match source_label.rsplit('.').next() {
        Some("yaml" | "yml") => serde_yaml::from_str(text).map_err(|error| error.to_string()),
        Some("json") => serde_json::from_str(text).map_err(|error| error.to_string()),
        _ => serde_json::from_str(text)
            .or_else(|_| serde_yaml::from_str(text))
            .map_err(|error| error.to_string()),
    }
}

fn parameter_line(value: &Value) -> Option<String> {
    if let Some(reference) = value.get("$ref").and_then(Value::as_str) {
        return Some(format!("ref: {reference}"));
    }

    let location = value.get("in").and_then(Value::as_str).unwrap_or("unknown");
    let name = value
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("unnamed");
    let schema_type = value
        .get("schema")
        .and_then(|schema| schema.get("type"))
        .and_then(Value::as_str)
        .unwrap_or("object");

    Some(format!("{location} {name}: {schema_type}"))
}

fn request_body_lines(value: &Value) -> Vec<String> {
    if let Some(reference) = value.get("$ref").and_then(Value::as_str) {
        return vec![format!("ref: {reference}")];
    }

    value
        .get("content")
        .and_then(Value::as_object)
        .map(|content| {
            content
                .iter()
                .map(|(media_type, body)| {
                    let refs = collect_refs(body);
                    if refs.is_empty() {
                        media_type.to_string()
                    } else {
                        format!("{media_type}: {}", refs.join(", "))
                    }
                })
                .collect()
        })
        .unwrap_or_else(Vec::new)
}

fn response_line(status: &str, value: &Value) -> String {
    let description = value
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let refs = collect_refs(value);
    if refs.is_empty() {
        format!("{status}: {description}")
    } else {
        format!("{status}: {description} [{}]", refs.join(", "))
    }
}

fn collect_refs(value: &Value) -> Vec<String> {
    let mut refs = BTreeSet::new();
    collect_refs_inner(value, &mut refs);
    refs.into_iter().collect()
}

fn collect_refs_inner(value: &Value, refs: &mut BTreeSet<String>) {
    match value {
        Value::Object(object) => {
            if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
                refs.insert(reference.trim_start_matches("#/").to_string());
            }
            for child in object.values() {
                collect_refs_inner(child, refs);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_refs_inner(item, refs);
            }
        }
        _ => {}
    }
}

fn build_schema_index(schemas: &serde_json::Map<String, Value>) -> BTreeMap<String, SchemaView> {
    schemas
        .iter()
        .map(|(name, schema)| {
            let key = format!("components/schemas/{name}");
            let lines = serde_json::to_string_pretty(schema)
                .unwrap_or_else(|_| "{}".to_string())
                .lines()
                .map(|line| line.to_string())
                .collect();
            (key.clone(), SchemaView { title: key, lines })
        })
        .collect()
}
