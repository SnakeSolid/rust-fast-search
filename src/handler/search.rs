use crate::config::ConfigRef;
use crate::handler::util::handle_request;
use crate::handler::HandlerError;
use crate::handler::HandlerResult;
use crate::index::TextIndexRef;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IronRequest;
use iron::Response as IronResponse;
use std::collections::HashMap;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::Field;
use tantivy::schema::Value;
use tantivy::IndexReader;

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
        query: &str,
    ) -> HandlerResult<Vec<HashMap<String, Option<String>>>> {
        let mut result = Vec::new();
        let searcher = reader.searcher();
        let index_schema = searcher.schema();
        let query_parser = QueryParser::for_index(
            searcher.index(),
            index_schema
                .fields()
                .iter()
                .filter(|entry| entry.is_indexed())
                .filter_map(|entry| schema.get(entry.name()))
                .cloned()
                .collect(),
        );
        let query = query_parser
            .parse_query(query)
            .map_err(|err| HandlerError::new(&format!("Invalid query - {:?}", err)))?;
        let top_docs: Vec<_> = searcher
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
}

impl Handler for SearchHandler {
    fn handle(&self, request: &mut IronRequest) -> IronResult<IronResponse> {
        handle_request(request, move |request: Request| {
            let query = request.query;

            if !query.is_empty() {
                Ok(self
                    .text_index
                    .read(|reader, schema| self.process_query(reader, schema, &query))
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
