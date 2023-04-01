extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use std::collections::{HashMap, HashSet};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct ModelxParser;

#[derive(Debug, Default)]
pub enum FieldType {
    #[default]
    String,
    UUID,
    Integer,
}

#[derive(Debug, Default)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub is_key: bool,
    pub annotations: HashSet<String>,
}

#[derive(Debug, Default)]
pub struct Definition {
    pub name: String,
    pub fields: HashMap<String, Field>,
    pub annotations: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Modelx {
    pub definitions: HashMap<String, Definition>,
}

impl Modelx {
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Self {
        let input = std::fs::read_to_string(path).unwrap();
        Modelx::parse(&input)
    }

    pub fn parse(input: &str) -> Modelx {
        let file = ModelxParser::parse(Rule::file, input)
            .unwrap()
            .next()
            .unwrap();

        let mut modelx = Modelx {
            ..Default::default()
        };
        for table in file.into_inner() {
            match table.as_rule() {
                Rule::table => {
                    let mut curr_def = Definition {
                        ..Default::default()
                    };
                    for in_table in table.into_inner() {
                        match in_table.as_rule() {
                            Rule::ident => {
                                curr_def.name = in_table.as_str().to_string();
                            }
                            Rule::annotation => {
                                curr_def.annotations.push(in_table.as_str().to_string());
                            }
                            Rule::field => {
                                let mut curr_field = Field {
                                    ..Default::default()
                                };
                                for in_field in in_table.into_inner() {
                                    match in_field.as_rule() {
                                        Rule::is_key => {
                                            if in_field.as_str() == "key" {
                                                curr_field.is_key = true
                                            }
                                        }
                                        Rule::field_type => match in_field.as_str() {
                                            "String" => curr_field.field_type = FieldType::String,
                                            "UUID" => curr_field.field_type = FieldType::UUID,
                                            "Integer" => curr_field.field_type = FieldType::Integer,
                                            _ => unreachable!(),
                                        },
                                        Rule::ident => {
                                            curr_field.name = in_field.as_str().to_string();
                                        }
                                        Rule::annotation => {
                                            curr_field
                                                .annotations
                                                .insert(in_field.as_str().to_string());
                                        }
                                        _ => {}
                                    }
                                }
                                curr_def.fields.insert(curr_field.name.clone(), curr_field);
                            }
                            _ => {}
                        }
                    }
                    modelx.definitions.insert(curr_def.name.clone(), curr_def);
                }
                _ => (),
            }
        }
        modelx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_model_parse() {
        let result = Modelx::parse(
            r#"
            @anno1
            @anno2
            table foobar {
              @fieldanno1
              @fieldanno2
              key bla: UUID
              super:Integer
            }
        "#,
        );
        dbg!(&result);
    }

    #[test]
    fn sample_model_load() {
        let result = Modelx::load("test/example.modelx");
        dbg!(&result);
    }
}
