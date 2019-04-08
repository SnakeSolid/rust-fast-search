use crate::config::ConfigRef;
use crate::error::ApplicationError;
use crate::error::ApplicationResult;
use crate::handler::FieldsHandler;
use crate::handler::SearchHandler;
use crate::index::TextIndexRef;
use crate::options::Options;
use iron::Iron;
use mount::Mount;
use staticfile::Static;

#[allow(clippy::needless_pass_by_value)]
pub fn start(
    options: &Options,
    config: &ConfigRef,
    text_index: &TextIndexRef,
) -> ApplicationResult {
    let mut mount = Mount::new();
    mount.mount("/api/v1/fields", FieldsHandler::new(config));
    mount.mount("/api/v1/search", SearchHandler::new(config, text_index));
    mount.mount("/static", Static::new("public/static"));
    mount.mount("/", Static::new("public"));

    let address = options.address();
    let port = options.port();

    println!("Listening on {}:{}...", address, port);

    Iron::new(mount)
        .http((address, port))
        .map_err(ApplicationError::server_error)?;

    Ok(())
}
