use crate::config::ConfigRef;
use crate::handler::util::handle_request;
use crate::handler::HandlerError;
use crate::index::TextIndexRef;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IronRequest;
use iron::Response as IronResponse;
use std::collections::HashMap;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::Value;

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
}

impl Handler for SearchHandler {
    fn handle(&self, request: &mut IronRequest) -> IronResult<IronResponse> {
        handle_request(request, move |request: Request| {
            let query = request.query;
            let mut result: Vec<HashMap<String, Option<String>>> = Vec::new();

            if !query.is_empty() {
                self.text_index
                    .read(|reader, _schema| {
                        let searcher = reader.searcher();
                        let index_schema = searcher.schema();
                        let query_parser = QueryParser::for_index(
                            searcher.index(),
                            index_schema
                                .fields()
                                .iter()
                                .filter(|entry| entry.is_indexed())
                                .filter_map(|entry| _schema.get(entry.name()))
                                .cloned()
                                .collect(),
                        );
                        let query = query_parser.parse_query(&query).unwrap();

                        let top_docs: Vec<_> =
                            searcher.search(&query, &TopDocs::with_limit(50)).unwrap();

                        for (_score, doc_address) in top_docs {
                            let retrieved_doc = searcher.doc(doc_address).unwrap();
                            let mut doc = HashMap::new();

                            for field_value in retrieved_doc.field_values() {
                                let field = field_value.field();
                                let name = index_schema.get_field_name(field);
                                let value = field_value.value();

                                doc.insert(name.into(), value_to_string(value));
                            }

                            result.push(doc);
                        }

                        Ok(())
                    })
                    .map_err(|err| HandlerError::new(&format!("{}", err)))?;
            }

            Ok(result)
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
