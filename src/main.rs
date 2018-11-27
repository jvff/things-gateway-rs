#[macro_use]
extern crate serde_derive;

mod plugin;
mod users;

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};
use std::thread;

use actix_web::{
    dev, fs, http::Method, middleware, server, ws, App, FromRequest, HttpRequest, HttpResponse,
    Json, Path, Result,
};

use crate::users::UserService;
use actix::*;

use self::plugin::Device;

struct UserCountHandler(UserService);

impl<S> dev::Handler<S> for UserCountHandler {
    type Result = HttpResponse;

    /// Handle request
    fn handle(&self, req: &HttpRequest<S>) -> Self::Result {
        let count = self.0.user_count();

        HttpResponse::Ok()
            .content_type("application/json")
            .json(count)
    }
}

fn ping<S>(req: &HttpRequest<S>) -> Result<HttpResponse> {
    Ok(HttpResponse::NoContent().into())
}

fn get_things(request: &HttpRequest<AppState>) -> Result<Json<BTreeMap<String, Device>>> {
    let state = request.state();

    Ok(Json(state.devices.read().unwrap().clone()))
}

fn get_thing(request: &HttpRequest<AppState>) -> Result<Json<Device>> {
    let thing_name = Path::<String>::extract(request)?;
    let state = request.state();

    Ok(Json(
        state
            .devices
            .read()
            .unwrap()
            .get(&thing_name.into_inner())
            .unwrap()
            .clone(),
    ))
}

struct Ws;

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<ws::Message, ws::ProtocolError> for Ws {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Binary(bin) => ctx.binary(bin),
            _ => (),
        }
    }
}

struct AppState {
    devices: RwLock<BTreeMap<String, Device>>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            devices: RwLock::new(BTreeMap::new()),
        }
    }
}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    ::std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    thread::spawn(|| plugin::manage_plugins());

    let user_db = Arc::new(RwLock::new(BTreeMap::default()));

    let sys = actix::System::new("static_index");

    server::new(move || {
        let user_db_clone = user_db.clone();
        App::with_state(AppState::default())
            .middleware(middleware::Logger::default())
            .resource("/logs", |r| r.f(|req| ws::start(&req.drop_state(), Ws)))
            .resource("/ping", |r| r.f(ping))
            .resource("/users/count", |r| {
                r.h(UserCountHandler(UserService::with_db(user_db_clone)))
            })
            .scope("/things", |things_api_scope| {
                things_api_scope
                    .resource("/", |r| r.method(Method::GET).f(get_things))
                    .resource("/{thing_name}", |r| r.method(Method::GET).f(get_thing))
            })
            .handler(
                "/",
                fs::StaticFiles::new("./build/static/")
                    .unwrap()
                    .index_file("index.html"),
            )
    })
    .bind("127.0.0.1:8000")
    .expect("Can not start server on given IP/Port")
    .start();

    println!("Started http server: 127.0.0.1:8000");
    let _ = sys.run();
}
