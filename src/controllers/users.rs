use actix_web::{HttpRequest, Json};

use super::super::AppState;

pub fn count(request: &HttpRequest<AppState>) -> Json<usize> {
    let users = request.state().users.read().unwrap();

    Json(users.count())
}
