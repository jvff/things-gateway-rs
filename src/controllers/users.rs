use actix_web::{HttpRequest, Json};

pub fn count<S>(request: &HttpRequest<S>) -> Json<usize> {
    Json(0)
}
