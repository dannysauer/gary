#![deny(warnings, rust_2018_idioms)]

use futures::Future;
use hyper::client::connect::{Destination, HttpConnector};
use rayon::prelude::*;
use std::time::Instant;
use tower_grpc::Request;
use tower_hyper::{client, util};
use tower_util::MakeService;

pub mod hello_world {
    include!(concat!(env!("OUT_DIR"), "/helloworld.rs"));
}

pub fn main() {
    let _ = ::env_logger::init();

    let mut list = Vec::with_capacity(10000);
    for i in 0..10000 {
        list.push(i);
    }

    let now = Instant::now();
    let _:Vec<i32> = list.par_iter().map( |_| {
        make_request_to_server();
        return 1;
    }).collect();
    /*
    for _ in 1..10000 {
        make_request_to_server();
    }
    */
    println!("Time Taken: {}", now.elapsed().as_millis());
}

pub fn make_request_to_server() {
    let uri: http::Uri = format!("http://[::1]:50051").parse().unwrap();

    let dst = Destination::try_from_uri(uri.clone()).unwrap();
    let connector = util::Connector::new(HttpConnector::new(4));
    let settings = client::Builder::new().http2_only(true).clone();
    let mut make_client = client::Connect::with_builder(connector, settings);

    let say_hello = make_client
        .make_service(dst)
        .map_err(|e| panic!("connect error: {:?}", e))
        .and_then(move |conn| {
            use hello_world::client::Greeter;
            let conn = tower_request_modifier::Builder::new()
                .set_origin(uri)
                .build(conn)
                .unwrap();

            // Wait until the client is ready...
            Greeter::new(conn).ready()
        })
        .and_then(|mut client| {
            use hello_world::HelloRequest;

            client.say_hello(Request::new(HelloRequest {
                name: "What is in a name?".to_string(),
            }))
        })
        .and_then(|_| {
            //println!("RESPONSE = {:?}", response);
            Ok(())
        })
        .map_err(|e| {
            println!("ERR = {:?}", e);
        });
    tokio::run(say_hello);
}
