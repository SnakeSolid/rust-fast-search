use crate::config::ConfigRef;
use crate::config::DataType;
use crate::config::FieldConfig;
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
                .map(ResponseField::from_field)
                .collect();

            Ok(result)
        })
    }
}

#[derive(Debug, Serialize)]
struct ResponseField {
    name: String,
    display: String,
    description: String,
    data_type: String,
}

impl ResponseField {
    fn from_field(field: &FieldConfig) -> Self {
        let data_type = match field.data_type() {
            DataType::Int { .. } | DataType::UInt { .. } => "number",
            DataType::Text => "string",
        };

        ResponseField {
            name: field.name().into(),
            display: field.display().into(),
            description: field.description().into(),
            data_type: data_type.into(),
        }
    }
}
