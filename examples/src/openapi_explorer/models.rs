use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenApiDocumentView {
    pub title: String,
    pub source_label: String,
    pub tags: Vec<TagView>,
    pub schema_index: BTreeMap<String, SchemaView>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagView {
    pub name: String,
    pub operations: Vec<OperationView>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationView {
    pub id: String,
    pub method: String,
    pub path: String,
    pub summary: String,
    pub description: String,
    pub parameters: Vec<String>,
    pub request_body: Vec<String>,
    pub responses: Vec<String>,
    pub schema_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaView {
    pub title: String,
    pub lines: Vec<String>,
}
