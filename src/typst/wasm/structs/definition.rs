use typst::{
    foundations::{CastInfo, ParamInfo, Value},
    syntax::SyntaxKind,
};
use typst_ide::{Definition, Tooltip};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    ast::parse_let_binding,
    console_log,
    typst::{source_file::SourceFile, TypstCore},
};

use super::{
    error::TypstCoreError,
    range::{MonacoRange, TypstCoreRange},
};

fn parse_cast_info(cast: &CastInfo) -> String {
    match cast {
        CastInfo::Any => "any".to_string(),
        CastInfo::Type(kind) => kind.short_name().to_string(),
        CastInfo::Value(val, _desc) => val.name().map_or_else(|| "undef".to_string(), |n| n),
        CastInfo::Union(types) => {
            let types_str: Vec<String> = types.iter().map(|t| parse_cast_info(t)).collect();
            format!("({})", types_str.join(" | "))
        }
    }
}

fn parse_params(params: &'static [ParamInfo]) -> String {
    params
        .iter()
        .map(|p| {
            let variadic = if p.variadic { "..." } else { "" };
            let name = p.name;
            let ty = parse_cast_info(&p.input);
            let default = if p.default.is_some() {
                let val = p.default.unwrap()();
                format!(" = {}", val.display().plain_text())
            } else {
                String::new()
            };
            let optional = if !p.required { "?" } else { "" };
            let named = if p.named { ": " } else { "" };
            format!("{}{}{}{}{}{}", variadic, name, optional, named, ty, default)
        })
        .collect::<Vec<String>>()
        .join(", ")
}

pub trait DefinitionExt {
    fn name(&self) -> Option<String>;
    fn kind(&self) -> Option<String>;
    fn docs(&self) -> Option<String>;
}

impl DefinitionExt for Value {
    fn name(&self) -> Option<String> {
        match self {
            Value::Func(f) => {
                let name = f.name().map(|n| n.to_string());
                let params = f.params().map(|p| parse_params(p));
                let return_type = f.returns().map(|r| parse_cast_info(r));
                let mut name_str = String::new();
                if let Some(name) = name {
                    name_str.push_str(&name);
                }
                if let Some(params) = params {
                    name_str.push_str(&format!("({})", params));
                }
                if let Some(return_type) = return_type {
                    name_str.push_str(&format!(" -> {}", return_type));
                }
                Some(name_str)
            }
            _ => Some("undef".to_string()),
        }
    }

    fn kind(&self) -> Option<String> {
        match self {
            Value::Func(_) => None,
            _ => Some(self.ty().short_name().to_string()),
        }
    }

    fn docs(&self) -> Option<String> {
        self.docs().map(|d| d.to_string())
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct Args {
    pub name: String,
    pub docs: Option<String>,
}

impl From<(String, Vec<String>)> for Args {
    fn from(value: (String, Vec<String>)) -> Self {
        let docs = if value.1.is_empty() {
            None
        } else {
            Some(value.1.join("\n"))
        };
        Self {
            name: value.0,
            docs,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug)]
pub struct TypstCoreDefinition {
    pub name: Option<String>,
    pub kind: Option<String>,
    pub docs: Option<String>,
    pub is_std: bool,
    pub is_fn: bool,
    pub args: Vec<Args>,
    pub def_span: Option<TypstCoreRange>,
    pub tooltip: Option<String>,
}

impl TypstCoreDefinition {
    pub fn new(
        core: &TypstCore,
        typst_def: Option<Definition>,
        typst_tt: Option<Tooltip>,
        source_file: &SourceFile,
    ) -> Result<Self, TypstCoreError> {
        let (name, kind, docs, is_std, def_span, is_fn, args) = if let Some(def) = typst_def {
            match def {
                Definition::Span(span) => {
                    let output_raw = core.resolve_span(span, |node, id, source_file| {
                        let def_span = TypstCoreRange::with_source(span, &source_file.source())?;

                        let parsed =
                            parse_let_binding(&node).expect("Definition should be a let binding!");

                        let is_fn = node
                            .parent_kind()
                            .map(|parent_kind| parent_kind == SyntaxKind::Closure)
                            .expect("Let binding identifier should have a parent.");

                        Ok::<
                            (
                                TypstCoreRange,
                                (String, Vec<String>),
                                Vec<(String, Vec<String>)>,
                                bool,
                            ),
                            TypstCoreError,
                        >((def_span, parsed.0, parsed.1, is_fn))
                    });
                    let output = output_raw
                        .map_err(|e| TypstCoreError::from(e))?
                        .expect("Node should be valid and found!")?;

                    let (name, raw_docs) = output.1;
                    let docs = if raw_docs.is_empty() {
                        None
                    } else {
                        Some(raw_docs.join("\n"))
                    };

                    (Some(name), None, docs, false, Some(output.0), output.3, output.2)
                }
                Definition::Std(value) => {
                    let name = value.name();
                    let kind = value.kind();
                    let docs = value.docs().map(|d| d.to_string());
                    let is_std = true;

                    console_log!(
                        "Std definition: name: {:?}, kind: {:?}, docs: {:?}",
                        name,
                        kind,
                        docs
                    );
                    (name, kind, docs, is_std, None, false, Vec::new())
                }
            }
        } else {
            (None, None, None, false, None, false, Vec::new())
        };

        Ok(Self {
            name,
            kind,
            docs,
            is_std,
            is_fn,
            args: args
                .into_iter()
                .map(Args::from)
                .collect(),
            def_span,
            tooltip: None,
        })
        // };

        /* Self {
            name,
            kind,
            docs,
            is_std,
            def_path,
            def_span,
            tooltip: Some("None".to_string())
        } */
    }
}
