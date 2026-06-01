use clap::{Arg, ArgMatches, Command};
use core::cluster_api::*;
use core::defaults::*;

fn main() {
    let matches = Command::new("garyctl")
        .version("0.1.0")
        .author("Marek C. <countsmarek@gmail.com>")
        .about("Control tool for Gary cluster")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file"),
        )
        .arg(
            Arg::new("v")
                .short('v')
                .action(clap::ArgAction::Count)
                .help("Sets the level of verbosity"),
        )
        .subcommand(
            Command::new("get").about("Get resources").subcommand(
                Command::new("nodes").about("List cluster nodes").arg(
                    Arg::new("target")
                        .short('t')
                        .long("target")
                        .help("Target node to query"),
                ),
            ),
        )
        .get_matches();

    if let Some(get_matches) = matches.subcommand_matches("get") {
        if let Some(node_matches) = get_matches.subcommand_matches("nodes") {
            get("nodes".to_string(), node_matches);
        }
    }
}

fn get(_resource_type: String, matches: &ArgMatches) {
    println!("getting nodes");

    let target = matches
        .get_one::<String>("target")
        .map(|s| s.as_str())
        .unwrap_or(TARGET);

    let connection = format!("tcp://{}:{}", target, API_PORT);

    println!("Getting list of nodes from {}", target);
    let ctx = zmq::Context::new();
    let sock = ctx.socket(zmq::SocketType::REQ).unwrap();
    let _ = sock.connect(&connection);
    let payload = ClusterRequest {
        action: ResourceAction::Get,
        r_type: ResourceType::Nodes,
        args: String::new(),
    };
    let serialized_msg = serde_cbor::to_vec(&payload).unwrap();
    let _ = sock.send(serialized_msg, 0);
    if let Ok(msg) = sock.recv_msg(0) {
        println!("Nodes:\n{}", msg.as_str().unwrap_or("(invalid utf8)"));
    }
}
