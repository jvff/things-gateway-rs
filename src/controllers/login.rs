use actix_web::{
    error::ResponseError, http::StatusCode, Error, FromRequest, HttpRequest, HttpResponse, Json,
    Responder,
};
use futures::Future;

use super::super::models::WebToken;

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Fail)]
enum LoginError {
    #[fail(display = "No user with that name")]
    InvalidUser,
    #[fail(display = "Wrong password")]
    InvalidPassword,
}

impl ResponseError for LoginError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::new(StatusCode::UNAUTHORIZED)
    }
}

pub fn login<S: 'static>(
    request: &HttpRequest<S>,
) -> Box<Future<Item = Json<WebToken>, Error = Error>> {
    Box::new(Json::<LoginRequest>::extract(request).and_then(|body| {
        if body.email != "test@account.test" {
            Err(LoginError::InvalidUser.into())
        } else if body.password != "password" {
            Err(LoginError::InvalidPassword.into())
        } else {
            Ok(Json(WebToken::issue()))
        }
    }))
}
