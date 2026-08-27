use crate::error::Error;

#[derive(Debug, Clone, Copy)]
pub struct Template {
    pub id: &'static str,
    pub summary: &'static str,
}

pub const TEMPLATES: &[Template] = &[
    Template {
        id: "api",
        summary: "FastAPI HTTP service with secure defaults",
    },
    Template {
        id: "api-rust",
        summary: "Axum HTTP service with secure defaults",
    },
    Template {
        id: "worker",
        summary: "Python background worker with secure defaults",
    },
];

pub fn require(id: &str) -> Result<&'static Template, Error> {
    TEMPLATES
        .iter()
        .find(|template| template.id == id)
        .ok_or_else(|| Error::UnknownType {
            got: id.to_string(),
            expected: expected_ids(),
        })
}

pub fn expected_ids() -> String {
    TEMPLATES
        .iter()
        .map(|template| template.id)
        .collect::<Vec<_>>()
        .join(", ")
}
