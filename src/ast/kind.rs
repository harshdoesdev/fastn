#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct VariableKind {
    pub modifier: Option<VariableModifier>,
    pub kind: String,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum VariableModifier {
    List,
    Optional,
}

pub const OPTIONAL: &str = "optional";
pub const LIST: &str = "list";

impl VariableModifier {
    pub(crate) fn is_optional_from_expr(expr: &str) -> bool {
        expr.eq(OPTIONAL)
    }

    pub(crate) fn is_list_from_expr(expr: &str) -> bool {
        expr.eq(LIST)
    }

    fn is_list(&self) -> bool {
        matches!(self, VariableModifier::List)
    }

    fn is_optional(&self) -> bool {
        matches!(self, VariableModifier::Optional)
    }

    pub(crate) fn get_modifier(expr: &str) -> Option<VariableModifier> {
        let expr = expr.split_whitespace().collect::<Vec<&str>>();
        if expr.len() == 2 {
            if VariableModifier::is_optional_from_expr(expr.get(0).unwrap()) {
                return Some(VariableModifier::Optional);
            } else if VariableModifier::is_list_from_expr(expr.get(1).unwrap()) {
                return Some(VariableModifier::List);
            }
        }
        None
    }
}

impl VariableKind {
    fn new(kind: &str, modifier: Option<VariableModifier>) -> VariableKind {
        VariableKind {
            modifier,
            kind: kind.to_string(),
        }
    }

    pub(crate) fn get_kind(
        kind: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::ast::Result<VariableKind> {
        let expr = kind.split_whitespace().collect::<Vec<&str>>();
        if expr.len() > 2 || expr.is_empty() {
            return ftd::ast::parse_error(
                format!("Invalid variable kind, found: `{}`", kind),
                doc_id,
                line_number,
            );
        }

        let modifier = VariableModifier::get_modifier(kind);
        let kind = match modifier {
            Some(VariableModifier::Optional) if expr.len() == 2 => expr.get(1).unwrap(),
            Some(VariableModifier::List) if expr.len() == 2 => expr.get(0).unwrap(),
            None => expr.get(0).unwrap(),
            _ => {
                return ftd::ast::parse_error(
                    format!("Invalid variable kind, found: `{}`", kind),
                    doc_id,
                    line_number,
                )
            }
        };

        Ok(VariableKind::new(kind, modifier))
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum VariableValue {
    Optional(Box<Option<VariableValue>>),
    List(Vec<VariableValue>),
    Record {
        name: String,
        caption: Box<Option<VariableValue>>,
        headers: Vec<HeaderValue>,
        body: Option<BodyValue>,
        line_number: usize,
    },
    String {
        value: String,
        line_number: usize,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct BodyValue {
    value: String,
    line_number: usize,
}

impl BodyValue {
    fn new(value: &str, line_number: usize) -> BodyValue {
        BodyValue {
            value: value.to_string(),
            line_number,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct HeaderValue {
    key: String,
    value: VariableValue,
    line_number: usize,
}

impl HeaderValue {
    fn new(key: &str, value: VariableValue, line_number: usize) -> HeaderValue {
        HeaderValue {
            key: key.to_string(),
            value,
            line_number,
        }
    }
}

impl VariableValue {
    pub(crate) fn inner(&self) -> Option<VariableValue> {
        match self {
            VariableValue::Optional(value) => value.as_ref().as_ref().map(|v| v.to_owned()),
            t => Some(t.to_owned()),
        }
    }

    fn is_null(&self) -> bool {
        VariableValue::Optional(Box::new(None)).eq(self)
    }

    fn is_list(&self) -> bool {
        matches!(self, VariableValue::List(_))
    }

    fn into_optional(self) -> VariableValue {
        match self {
            t @ VariableValue::Optional(_) => t,
            t => VariableValue::Optional(Box::new(Some(t))),
        }
    }

    pub(crate) fn from_p1_with_modifier(
        section: &ftd::p11::Section,
        doc_id: &str,
        modifier: &Option<VariableModifier>,
    ) -> ftd::ast::Result<VariableValue> {
        let value = VariableValue::from_p1(section);
        value.into_modifier(doc_id, section.line_number, modifier)
    }

    pub(crate) fn from_header_with_modifier(
        header: &ftd::p11::Header,
        doc_id: &str,
        modifier: &Option<VariableModifier>,
    ) -> ftd::ast::Result<VariableValue> {
        let value = VariableValue::from_p1_header(header);
        value.into_modifier(doc_id, header.get_line_number(), modifier)
    }

    pub(crate) fn into_modifier(
        self,
        doc_id: &str,
        line_number: usize,
        modifier: &Option<VariableModifier>,
    ) -> ftd::ast::Result<VariableValue> {
        match modifier {
            Some(modifier) if modifier.is_list() => {
                if self.is_null() {
                    Ok(VariableValue::List(vec![]))
                } else if self.is_list() {
                    Ok(self)
                } else {
                    ftd::ast::parse_error(
                        format!("Expected List found: `{:?}`", self),
                        doc_id,
                        line_number,
                    )
                }
            }
            Some(modifier) if modifier.is_optional() => Ok(self.into_optional()),
            _ => Ok(self),
        }
    }

    pub(crate) fn from_p1(section: &ftd::p11::Section) -> VariableValue {
        use itertools::Itertools;

        if !section.sub_sections.is_empty() {
            return VariableValue::List(
                section
                    .sub_sections
                    .iter()
                    .map(VariableValue::from_p1)
                    .collect_vec(),
            );
        }

        let caption = section
            .caption
            .as_ref()
            .and_then(|v| VariableValue::from_p1_header(v).inner());

        let headers = section
            .headers
            .0
            .iter()
            .filter(|v| !ftd::ast::utils::is_condition(v.get_key().as_str(), &v.get_kind()))
            .map(|header| {
                HeaderValue::new(
                    header.get_key().as_str(),
                    VariableValue::from_p1_header(header),
                    header.get_line_number(),
                )
            })
            .collect_vec();

        let body = section
            .body
            .as_ref()
            .map(|v| BodyValue::new(v.get_value().as_str(), v.line_number));

        if headers.is_empty() && !(caption.is_some() && body.is_some()) {
            return if let Some(caption) = caption {
                caption
            } else if let Some(body) = body {
                VariableValue::String {
                    value: body.value,
                    line_number: body.line_number,
                }
            } else {
                VariableValue::Optional(Box::new(None))
            };
        }

        VariableValue::Record {
            name: section.name.to_string(),
            caption: Box::new(caption),
            headers,
            body,
            line_number: section.line_number,
        }
    }

    pub(crate) fn from_p1_header(header: &ftd::p11::Header) -> VariableValue {
        use itertools::Itertools;

        match header {
            ftd::p11::Header::KV(ftd::p11::header::KV {
                value, line_number, ..
            }) => VariableValue::from_value(value, *line_number),
            ftd::p11::Header::Section(ftd::p11::header::Section { section, .. }) => {
                VariableValue::List(section.iter().map(VariableValue::from_p1).collect_vec())
            }
        }
    }

    pub(crate) fn from_value(value: &Option<String>, line_number: usize) -> VariableValue {
        match value {
            Some(value) if value.ne(NULL) => VariableValue::String {
                value: value.to_string(),
                line_number,
            },
            _ => VariableValue::Optional(Box::new(None)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Condition {
    expression: String,
    line_number: usize,
}

impl Condition {
    fn new(expression: &str, line_number: usize) -> Condition {
        Condition {
            expression: expression.to_string(),
            line_number,
        }
    }

    pub(crate) fn from_headers(
        headers: &ftd::p11::Headers,
        doc_id: &str,
    ) -> ftd::ast::Result<Option<Condition>> {
        let condition = headers
            .0
            .iter()
            .find(|v| ftd::ast::utils::is_condition(v.get_key().as_str(), &v.get_kind()));
        let condition = if let Some(condition) = condition {
            condition
        } else {
            return Ok(None);
        };

        let expression = condition
            .get_value(doc_id)?
            .ok_or(ftd::ast::Error::ParseError {
                message: "`if` condition must contain expression".to_string(),
                doc_id: doc_id.to_string(),
                line_number: condition.get_line_number(),
            })?;

        Ok(Some(Condition::new(
            expression.as_str(),
            condition.get_line_number(),
        )))
    }
}

pub const NULL: &str = "NULL";
