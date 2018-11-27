use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::thread;

use nanomsg::{Protocol, Socket};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "messageType", content = "data")]
pub enum PluginRegistrationRequest {
    #[serde(rename_all = "camelCase")]
    RegisterPlugin { plugin_id: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "messageType", content = "data")]
pub enum PluginRegistrationReply {
    #[serde(rename_all = "camelCase")]
    RegisterPluginReply {
        plugin_id: String,
        ipc_base_addr: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    name: String,
    value: serde_json::Value,
    visible: bool,
    label: Option<String>,
    #[serde(rename = "type")]
    type_name: Option<String>,
    #[serde(rename = "@type")]
    schema_type: Option<String>,
    unit: Option<String>,
    description: Option<String>,
    maximum: Option<f64>,
    minimum: Option<f64>,
    #[serde(rename = "enum")]
    enum_values: Option<Vec<serde_json::Value>>,
    read_only: Option<bool>,
    multiple_of: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    label: String,
    description: String,
    input: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    description: String,
    #[serde(rename = "type")]
    type_name: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    pub description: String,
    pub properties: HashMap<String, Property>,
    pub actions: HashMap<String, Action>,
    pub events: HashMap<String, Event>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "messageType", content = "data")]
pub enum IncomingMessage {
    #[serde(rename_all = "camelCase")]
    AddAdapter {
        plugin_id: String,
        adapter_id: String,
        name: String,
        package_name: String,
    },
    #[serde(rename_all = "camelCase")]
    HandleDeviceAdded {
        plugin_id: String,
        adapter_id: String,
        #[serde(flatten)]
        device: Device,
    },
}

pub fn manage_plugins() {
    let mut plugins = HashSet::new();
    let mut socket = Socket::new(Protocol::Rep).expect("Failed to open socket");
    let mut endpoint = socket
        .bind("ipc:///tmp/gateway.addonManager")
        .expect("Failed to bind socket");
    let mut message = String::new();

    loop {
        socket
            .read_to_string(&mut message)
            .expect("Failed to read from socket");

        println!("Got message: {:?}", message);

        let request: PluginRegistrationRequest =
            serde_json::from_str(&message).expect("Failed to deserialize request");
        let PluginRegistrationRequest::RegisterPlugin { plugin_id } = request;
        let ipc_base_addr = format!("gateway.plugin.{}", plugin_id);
        let plugin_socket_address = format!("ipc:///tmp/{}", ipc_base_addr);

        plugins.insert(plugin_id.clone());

        thread::spawn(|| handle_plugin(plugin_socket_address));

        let reply = PluginRegistrationReply::RegisterPluginReply {
            plugin_id,
            ipc_base_addr,
        };

        let reply_message = serde_json::to_string(&reply).expect("Failed to send reply");
        socket.write_all(reply_message.as_bytes());

        message.clear();
    }

    endpoint.shutdown();
}

fn handle_plugin(socket_address: String) {
    let mut socket = Socket::new(Protocol::Pair).expect("Failed to open socket for plugin");
    let mut endpoint = socket
        .bind(&socket_address)
        .expect("Failed to bind plugin socket");
    let mut message = String::new();

    loop {
        socket
            .read_to_string(&mut message)
            .expect("Failed to read from plugin socket");
        println!("Plugin sent: {}", message);

        let deserialized: IncomingMessage =
            serde_json::from_str(&message).expect("Failed to deserialize incoming message");

        println!("Incoming message: {:#?}", deserialized);
        message.clear();
    }

    endpoint.shutdown();
}
