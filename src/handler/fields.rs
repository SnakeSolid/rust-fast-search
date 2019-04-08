use crate::config::ConfigRef;
use crate::handler::util::handle_empty;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IronRequest;
use iron::Response as IronResponse;

#[derive(Debug)]
pub struct FieldsHandler {
    config: ConfigRef,
}

impl FieldsHandler {
    pub fn new(config: &ConfigRef) -> FieldsHandler {
        FieldsHandler {
            config: config.clone(),
        }
    }
}

impl Handler for FieldsHandler {
    fn handle(&self, _request: &mut IronRequest) -> IronResult<IronResponse> {
        handle_empty(move || {
            let result: Vec<ResponseField> = self
                .config
                .schema()
                .iter()
                .map(|field| ResponseField::new(field.name(), field.display()))
                .collect();

            Ok(result)
        })
    }
}

#[derive(Debug, Serialize)]
struct ResponseField {
    name: String,
    display: String,
}

impl ResponseField {
    fn new(name: &str, display: &str) -> Self {
        ResponseField {
            name: name.into(),
            display: display.into(),
        }
    }
}
