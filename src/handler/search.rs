use crate::config::ConfigRef;
use crate::handler::util::handle_request;
use crate::handler::HandlerError;
use crate::handler::HandlerResult;
use crate::index::TextIndexRef;
use crate::parser::parse_query;
use crate::parser::Occurance;
use crate::parser::Token;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IronRequest;
use iron::Response as IronResponse;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Bound;
use std::str::FromStr;
use tantivy::collector::TopDocs;
use tantivy::query::BooleanQuery;
use tantivy::query::Occur;
use tantivy::query::Query;
use tantivy::query::RangeQuery;
use tantivy::query::TermQuery;
use tantivy::schema::Field;
use tantivy::schema::FieldType;
use tantivy::schema::IndexRecordOption;
use tantivy::schema::Schema;
use tantivy::schema::Value;
use tantivy::IndexReader;
use tantivy::Term;

#[derive(Debug)]
pub struct SearchHandler {
    config: ConfigRef,
    text_index: TextIndexRef,
}

impl SearchHandler {
    pub fn new(config: &ConfigRef, text_index: &TextIndexRef) -> SearchHandler {
        SearchHandler {
            config: config.clone(),
            text_index: text_index.clone(),
        }
    }

    fn process_query(
        &self,
        reader: &IndexReader,
        schema: &HashMap<String, Field>,
        tokens: &[Token],
    ) -> HandlerResult<Vec<HashMap<String, Option<String>>>> {
        let mut result = Vec::new();
        let searcher = reader.searcher();
        let index_schema = searcher.schema();
        let query = self.build_query(tokens, schema, index_schema)?;
        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(50))
            .map_err(|err| HandlerError::new(&format!("Search error - {:?}", err)))?;

        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher
                .doc(doc_address)
                .map_err(|err| HandlerError::new(&format!("Search document - {:?}", err)))?;
            let mut doc = HashMap::new();

            for field_value in retrieved_doc.field_values() {
                let field = field_value.field();
                let name = index_schema.get_field_name(field);
                let value = field_value.value();

                doc.insert(name.into(), value_to_string(value));
            }

            result.push(doc);
        }

        Ok(result)
    }

    fn build_query(
        &self,
        tokens: &[Token],
        field_schema: &HashMap<String, Field>,
        index_schema: &Schema,
    ) -> HandlerResult<Box<Query>> {
        let mut text_fields = HashMap::new();
        let mut u64_fields = HashMap::new();
        let mut i64_fields = HashMap::new();

        for entry in index_schema
            .fields()
            .iter()
            .filter(|entry| entry.is_indexed())
        {
            let name = entry.name();

            if let Some(field) = field_schema.get(name).cloned() {
                match entry.field_type() {
                    FieldType::Str(..) => {
                        text_fields.insert(name, field);
                    }
                    FieldType::U64(..) => {
                        u64_fields.insert(name, field);
                    }
                    FieldType::I64(..) => {
                        i64_fields.insert(name, field);
                    }
                    _ => {}
                }
            }
        }

        let mut terms: Vec<(_, Box<Query>)> = Vec::new();

        for token in tokens {
            match token {
                Token::Text { occurance, strings } => {
                    let occur = self.map_occurance(occurance);

                    for string in strings {
                        let string = string.to_lowercase();
                        let inner_terms: Vec<_> = text_fields
                            .values()
                            .cloned()
                            .map(|field| {
                                (Occur::Should, self.create_term_query_text(field, &string))
                            })
                            .collect();

                        terms.push((occur, Box::new(BooleanQuery::from(inner_terms))));
                    }
                }
                Token::FilterEquals {
                    occurance,
                    field,
                    value,
                } => {
                    let occur = self.map_occurance(occurance);

                    if let Some(field) = text_fields.get(field.as_str()).cloned() {
                        let value = value.to_lowercase();

                        terms.push((occur, self.create_term_query_text(field, &value)));
                    } else if let Some(field) = u64_fields.get(field.as_str()).cloned() {
                        let value = value.parse().map_err(|err| {
                            HandlerError::new(&format!(
                                "Failed to parse value `{}` - {}",
                                value, err
                            ))
                        })?;

                        terms.push((occur, self.create_term_query_u64(field, value)));
                    } else if let Some(field) = i64_fields.get(field.as_str()).cloned() {
                        let value = value.parse().map_err(|err| {
                            HandlerError::new(&format!(
                                "Failed to parse value `{}` - {}",
                                value, err
                            ))
                        })?;

                        terms.push((occur, self.create_term_query_i64(field, value)));
                    } else {
                        return Err(HandlerError::new(&format!("Field `{}` not defined", field)));
                    }
                }
                Token::FilterRange {
                    occurance,
                    field,
                    left_bound,
                    right_bound,
                } => {
                    let occur = self.map_occurance(occurance);

                    if let Some(field) = u64_fields.get(field.as_str()).cloned() {
                        let left_bound = self.parse_bound(&left_bound)?;
                        let right_bound = self.parse_bound(&right_bound)?;

                        terms.push((
                            occur,
                            self.create_bound_query_u64(field, left_bound, right_bound),
                        ));
                    } else if let Some(field) = i64_fields.get(field.as_str()).cloned() {
                        let left_bound = self.parse_bound(&left_bound)?;
                        let right_bound = self.parse_bound(&right_bound)?;

                        terms.push((
                            occur,
                            self.create_bound_query_i64(field, left_bound, right_bound),
                        ));
                    } else {
                        return Err(HandlerError::new(&format!("Field `{}` not numeric", field)));
                    }
                }
            }
        }

        Ok(Box::new(BooleanQuery::from(terms)))
    }

    fn map_occurance(&self, occurance: &Option<Occurance>) -> Occur {
        match occurance {
            Some(Occurance::Must) => Occur::Must,
            Some(Occurance::MustNot) => Occur::MustNot,
            None => Occur::Should,
        }
    }

    fn parse_bound<T, E>(&self, value: &Option<String>) -> HandlerResult<Bound<T>>
    where
        T: FromStr<Err = E>,
        E: Display,
    {
        match value {
            Some(value) if value.is_empty() => Ok(Bound::Unbounded),
            Some(value) => {
                let value = value.parse().map_err(|err| {
                    HandlerError::new(&format!("Failed to parse value `{}` - {}", value, err))
                })?;

                Ok(Bound::Included(value))
            }
            None => Ok(Bound::Unbounded),
        }
    }

    fn create_bound_query_i64(
        &self,
        field: Field,
        left_bound: Bound<i64>,
        right_bound: Bound<i64>,
    ) -> Box<Query> {
        Box::new(RangeQuery::new_i64_bounds(field, left_bound, right_bound))
    }

    fn create_bound_query_u64(
        &self,
        field: Field,
        left_bound: Bound<u64>,
        right_bound: Bound<u64>,
    ) -> Box<Query> {
        Box::new(RangeQuery::new_u64_bounds(field, left_bound, right_bound))
    }

    fn create_term_query_i64(&self, field: Field, value: i64) -> Box<Query> {
        Box::new(TermQuery::new(
            Term::from_field_i64(field, value),
            IndexRecordOption::WithFreqs,
        ))
    }

    fn create_term_query_u64(&self, field: Field, value: u64) -> Box<Query> {
        Box::new(TermQuery::new(
            Term::from_field_u64(field, value),
            IndexRecordOption::WithFreqs,
        ))
    }

    fn create_term_query_text(&self, field: Field, value: &str) -> Box<Query> {
        Box::new(TermQuery::new(
            Term::from_field_text(field, value),
            IndexRecordOption::WithFreqs,
        ))
    }
}

impl Handler for SearchHandler {
    fn handle(&self, request: &mut IronRequest) -> IronResult<IronResponse> {
        handle_request(request, move |request: Request| {
            let tokens = parse_query(&request.query)
                .map_err(|err| HandlerError::new(&format!("{}", err)))?;

            if !tokens.is_empty() {
                Ok(self
                    .text_index
                    .read(|reader, schema| self.process_query(reader, schema, &tokens))
                    .map_err(|err| HandlerError::new(&format!("{}", err)))??)
            } else {
                Ok(Vec::new())
            }
        })
    }
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::Str(value) => Some(value.clone()),
        Value::U64(value) => Some(value.to_string()),
        Value::I64(value) => Some(value.to_string()),
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
struct Request {
    query: String,
}
