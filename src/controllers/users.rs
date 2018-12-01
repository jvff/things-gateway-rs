use actix_web::{HttpRequest, Json, Result};

use super::super::{models::WebToken, AppState};

pub fn count(request: &HttpRequest<AppState>) -> Json<usize> {
    let users = request.state().users.read().unwrap();

    Json(users.count())
}

pub fn create(request: &HttpRequest<AppState>) -> Result<Json<WebToken>> {
    Ok(Json(WebToken::issue()))
}
