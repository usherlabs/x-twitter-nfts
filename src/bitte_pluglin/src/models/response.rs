use rocket::serde::json::Value;
use rocket::serde::Serialize;
use rocket::Responder;
use serde::Deserialize;

#[derive(Responder, Debug)]
pub enum NetworkResponse {
    #[response(status = 200)]
    StatusOk(Value),
    #[response(status = 201)]
    Created(Value),
    #[response(status = 400)]
    BadRequest(Value),
    #[response(status = 401)]
    Unauthorized(Value),
    #[response(status = 404)]
    NotFound(Value),
    #[response(status = 422)]
    Status422(Value),
    #[response(status = 409)]
    Conflict(Value),
    #[response(status = 500)]
    InternalServerError(Value),
}

#[derive(Serialize, Deserialize)]
pub enum ResponseBody {
    Message(String),
    AuthToken(String),
    Body(Value),
    Error(Value)
}


#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Response {
    pub body: ResponseBody,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ResponseData {
    pub data: Value,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ErrorResponse {
    pub data: ResponseBody,
    pub message: String,

}
