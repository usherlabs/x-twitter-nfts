use rocket::serde::json::json;

use crate::models::response::{ErrorResponse, NetworkResponse, ResponseBody};

#[catch(422)]
pub fn unprocessable_entity_catcher() -> NetworkResponse{
    let response = ErrorResponse {
        message: "Incorrect Payload Structure".into(),
        data: ResponseBody::Message("Kindly see Readme for the correct payload structure[Swagger implementation coming soon]".to_string())
    };

    NetworkResponse::Status422(json!(response))
}

#[catch(404)]
pub async fn not_found (
) -> NetworkResponse {
    let response = ErrorResponse {
        message: "This Path or method does not exist on this server".into(),
        data: ResponseBody::Message("404 Not Found".to_string())
    };

    NetworkResponse::NotFound(json!(response))
}