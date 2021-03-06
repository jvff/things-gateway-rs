#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;

mod controllers;
mod models;
mod plugin;

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};
use std::thread;

use actix_web::{
    dev, fs, http::Method, middleware, server, ws, App, FromRequest, HttpRequest, HttpResponse,
    Json, Path, Result,
};

use actix::*;

use self::models::Users;
use self::plugin::{Device, Property};

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

fn get_properties(request: &HttpRequest<AppState>) -> Result<Json<Vec<Property>>> {
    let thing_name = Path::<String>::extract(request)?;
    let state = request.state();
    let devices = state.devices.read().unwrap();
    let device = devices.get(&thing_name.into_inner()).unwrap();
    let properties = device.properties.values().cloned().collect();

    Ok(Json(properties))
}

fn get_property(request: &HttpRequest<AppState>) -> Result<Json<Property>> {
    let (thing_name, property_id) = Path::<(String, String)>::extract(request)?.into_inner();
    let state = request.state();
    let devices = state.devices.read().unwrap();
    let device = devices.get(&thing_name).unwrap();
    let property = device.properties.get(&property_id).unwrap().clone();

    Ok(Json(property))
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

pub struct AppState {
    devices: RwLock<BTreeMap<String, Device>>,
    users: RwLock<Users>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            devices: RwLock::new(BTreeMap::new()),
            users: RwLock::new(Users::default()),
        }
    }
}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    ::std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    thread::spawn(|| plugin::manage_plugins());

    let sys = actix::System::new("static_index");

    server::new(move || {
        App::with_state(AppState::default())
            .middleware(middleware::Logger::default())
            .resource("/login", |r| {
                r.method(Method::POST).f(controllers::login::login)
            })
            .resource("/logs", |r| r.f(|req| ws::start(&req.drop_state(), Ws)))
            .resource("/ping", |r| r.f(ping))
            .scope("/users", |users_scope| {
                users_scope
                    .resource("/", |r| {
                        r.method(Method::POST).f(controllers::users::create)
                    })
                    .resource("/count", |r| {
                        r.method(Method::GET).f(controllers::users::count)
                    })
            })
            .scope("/things", |things_api_scope| {
                things_api_scope
                    .resource("/", |r| r.method(Method::GET).f(get_things))
                    .resource("/{thing_name}", |r| r.method(Method::GET).f(get_thing))
                    .nested("/{thing_name}/properties", |properties_scope| {
                        properties_scope
                            .resource("", |r| r.method(Method::GET).f(get_properties))
                            .resource("/{property_id}", |r| r.method(Method::GET).f(get_property))
                    })
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
