use core::comm::ClusterCommunicator;
use core::network::{Message, MessageType};
use std::sync::mpsc::Sender;

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use rand::Rng;
use serde_cbor;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// #[derive(Serialize, Deserialize, Debug)]  // Can't serialize 'sender'
pub struct ZmqNode {
    host_addr: String,           // Unique ID
    gossip_fanout: u8,           // Adjacent nodes updated each gossip cycle
    node_comm_ctx: zmq::Context, // zmq context - ToDo: make generic for other comm libs
    #[allow(dead_code)]
    main_thread_sender: Sender<&'static str>, // Sender to main thread channel
    pub adjacent: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
    delinquent: HashMap<String, DateTime<Utc>>, // Format is (host_addr, time_reported)
    removed: HashMap<String, DateTime<Utc>>,    // Format is (host_addr, time_reported)
}

impl ClusterCommunicator for ZmqNode {
    fn send_message(&self, target: &str, msg: &Message) -> bool {
        const TIMEOUTPERIOD: i32 = 1000; // timeout in milliseconds

        let serialized_msg = serde_cbor::to_vec(msg).unwrap();
        let requester = self.node_comm_ctx.socket(zmq::REQ).unwrap();
        requester.set_sndtimeo(TIMEOUTPERIOD).unwrap();
        requester.set_rcvtimeo(TIMEOUTPERIOD).unwrap();
        let target_addr = format!("tcp://{}:5555", target);
        assert!(requester.connect(&target_addr).is_ok());
        requester.send(&serialized_msg, 0).unwrap();
        let mut ack = String::new();
        match requester.recv_string(0) {
            Ok(v) => ack = v.unwrap(),
            Err(e) => println!("Error sending message to {}: {:?}", target, e),
        }
        assert!(requester.disconnect(&target_addr).is_ok());
        // TODO: Check that ack == "ACK" ?
        !ack.is_empty()
    }

    fn handle_message(&mut self, msg: &Message) {
        match &msg.msg_type {
            MessageType::Join => println!("Message Type Received: {:?}", &msg.msg_type),
            MessageType::Remove => println!("Message Type Received: {:?}", &msg.msg_type),
            MessageType::Gossip => {
                println!("Message Type Received: {:?}", &msg.msg_type);
                self.comm_recv_gossip(&msg.payload);
            }
            MessageType::Sync => println!("Message Type Received: {:?}", &msg.msg_type),
            MessageType::Ping => println!("Message Type Received: {:?}", &msg.msg_type),
            MessageType::Heartbeat => {
                println!("Message Type Received: {:?}", &msg.msg_type);
            }
        }
    }

    fn comm_recv_gossip(&mut self, payload: &[String]) {
        if !payload.is_empty() {
            for node_addr in payload {
                if let Ok(mut a) = self.adjacent.lock() {
                    if !a.contains_key(node_addr) {
                        a.insert(node_addr.to_string(), Utc::now());
                    }
                }
            }
        }
    }

    fn get_nghbr_sample(
        &self,
        a: &std::collections::HashMap<String, DateTime<Utc>>,
    ) -> Vec<String> {
        let mut adj_node_sample: Vec<String> = Vec::new();
        if (a.len() as u8) <= self.gossip_fanout {
            // Not sure about the 'as' conversion
            adj_node_sample = a.keys().cloned().collect();
        } else {
            let adjacent_keys: Vec<String> = a.keys().cloned().collect();
            let adjacent_keys_len = adjacent_keys.len();
            let mut rng = rand::thread_rng();
            while (adj_node_sample.len() as u8) < self.gossip_fanout {
                let rand_index: usize = rng.gen_range(0..adjacent_keys_len);
                if !adj_node_sample.contains(&adjacent_keys[rand_index]) {
                    adj_node_sample.push(adjacent_keys[rand_index].clone());
                }
            }
        }

        adj_node_sample
    }

    fn update_neighbors(&mut self) {
        if let Ok(mut a) = self.adjacent.lock() {
            if !a.is_empty() {
                let nghbr_sample = self.get_nghbr_sample(&a);
                let adjacent_vec: Vec<String> = a.keys().cloned().collect();
                for nghbr in nghbr_sample {
                    let msg = Message {
                        target: &nghbr,
                        sender: &self.host_addr,
                        msg_type: MessageType::Gossip,
                        payload: adjacent_vec.clone(), // ToDo: use ref instead
                    };
                    let result = self.send_message(&nghbr, &msg);
                    if !result {
                        if !self.delinquent.contains_key(&nghbr) {
                            self.delinquent.insert(nghbr.clone(), Utc::now());
                        }
                        a.remove(&nghbr);
                    }
                }
            }
        }
    }

    fn delinquent_node_check(&mut self) {
        const DELINQUENTPERIOD: u64 = 3600; // seconds
        let now = Utc::now();

        let allowed_duration_delinquent = Duration::from_secs(DELINQUENTPERIOD);

        println!("Checking delinquent nodes again...");
        let delinquent_vec: Vec<String> = self.delinquent.keys().cloned().collect();
        for node in delinquent_vec {
            let msg = Message {
                target: &node,
                sender: &self.host_addr,
                msg_type: MessageType::Heartbeat,
                payload: Vec::new(),
            };
            let result = self.send_message(&node, &msg);
            if result {
                if let Ok(mut a) = self.adjacent.lock() {
                    a.insert(node.clone(), Utc::now());
                    self.delinquent.remove(&node);
                }
            } else {
                let node_delinquent_time = *self.delinquent.get(&node).unwrap();
                let time_difference = now
                    .signed_duration_since(node_delinquent_time)
                    .to_std()
                    .unwrap();

                if time_difference > allowed_duration_delinquent {
                    self.removed.insert(node.clone(), Utc::now());
                    self.delinquent.remove(&node);
                }
            }
        }
    }
}

impl ZmqNode {
    pub fn new(
        mt_sender: Sender<&'static str>,
        host_addr: &str,
        init_nodes: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
    ) -> ZmqNode {
        ZmqNode {
            host_addr: host_addr.to_string(),
            gossip_fanout: 3,
            node_comm_ctx: zmq::Context::new(),
            main_thread_sender: mt_sender,
            adjacent: init_nodes,
            delinquent: HashMap::new(),
            removed: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    fn send_to_chan(&self, val: &'static str) {
        self.main_thread_sender.send(val).unwrap();
    }

    pub fn run(&mut self) {
        const GOSSIPPERIOD: u64 = 2; // seconds
        const DELCHECKPERIOD: u64 = 30; // seconds

        let responder = self.node_comm_ctx.socket(zmq::REP).unwrap();
        assert!(responder.bind("tcp://*:5555").is_ok());

        let allowed_duration_gossip = Duration::from_secs(GOSSIPPERIOD);
        let mut gossip_period_start_time = Instant::now();

        let allowed_duration_del_check = Duration::from_secs(DELCHECKPERIOD);
        let mut del_check_period_start_time = Instant::now();

        loop {
            if responder
                .poll(zmq::POLLIN, 10)
                .expect("client failed polling")
                > 0
            {
                let message = responder.recv_msg(0).unwrap();
                let deserialized: Message = serde_cbor::from_slice(&message).unwrap();
                responder.send("ACK", 0).unwrap();
                self.handle_message(&deserialized); // TODO: Handle on green threads
            }

            if gossip_period_start_time.elapsed() > allowed_duration_gossip {
                if let Ok(a) = self.adjacent.lock() {
                    println!("Adjacent nodes: {:#?}", a);
                    println!("Delinquent nodes: {:#?}", self.delinquent);
                }
                self.update_neighbors();
                gossip_period_start_time = Instant::now();
            }

            if del_check_period_start_time.elapsed() > allowed_duration_del_check {
                self.delinquent_node_check();
                del_check_period_start_time = Instant::now();
            }
        }
    }
}
