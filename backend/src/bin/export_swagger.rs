use openduckrust_api::ApiDoc;
use utoipa::OpenApi;
use std::fs;

fn main() {
    let spec = ApiDoc::openapi().to_pretty_json().expect("Failed to serialize OpenAPI spec");
    fs::write("openapi.json", &spec).expect("Failed to write openapi.json");
    println!("OpenAPI spec exported to openapi.json");
}
