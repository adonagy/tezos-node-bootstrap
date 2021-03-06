use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use reqwest;

use crate::types::{NodeType};

pub(crate) fn start_bootstrap(bootstrap_level: i32) {
    let measure_tezedge = spawn_monitor_thread(NodeType::Tezedge, bootstrap_level.clone()).unwrap();
    let measure_ocaml = spawn_monitor_thread(NodeType::Ocaml, bootstrap_level.clone()).unwrap();
    // let measure_tezedge_master = spawn_monitor_thread(NodeType::TezedgeMaster, bootstrap_level).unwrap();

    measure_tezedge.join().unwrap();
    measure_ocaml.join().unwrap();
    // measure_tezedge_master.join().unwrap();
}

fn spawn_monitor_thread(node_type: NodeType, bootstrap_level: i32) -> Result<JoinHandle<()>, failure::Error> {
    Ok(thread::spawn(move || {
        let now = Instant::now();

        let bootstrapping_tezedge = create_monitor_node_thread(node_type.clone(), bootstrap_level);
        bootstrapping_tezedge.join().unwrap();

        let elapsed = now.elapsed();
        let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
        println!("[{}] Duration in seconds: {}", node_type, sec);
    }))
}

fn create_monitor_node_thread(node: NodeType, bootstrap_level: i32) -> JoinHandle<()> {
    let bootstrap_monitoring_thread = thread::spawn(move || loop {
        match is_bootstrapped(&node) {
            Ok(s) => {
                // empty string means, the rpc server is running, but the bootstraping has not started yet
                if s != "" {
                    // let block_timestamp = DateTime::parse_from_rfc3339(&s).unwrap();
                    let block_level: i32 = s.parse().unwrap();

                    if block_level >= bootstrap_level {
                        println!("[{}] Done Bootstrapping", node.to_string());
                        break;
                    } else {
                        println!(
                            "[{}] Bootstrapping . . . level: {}",
                            node.to_string(),
                            s
                        );
                        thread::sleep(Duration::from_secs(10));
                    }
                } else {
                    println!(
                        "[{}] Waiting for node to start bootstrapping...",
                        node.to_string()
                    );
                    thread::sleep(Duration::from_secs(10));
                }
            }
            Err(_e) => {
                // panic!("Error in bootstrap check: {}", e);
                // NOTE: This should be handled more carefully
                println!("[{}] Waiting for node to run", node.to_string());
                println!("[{}] Error: {}", node.to_string(), _e);

                thread::sleep(Duration::from_secs(10));
            }
        }
    });
    bootstrap_monitoring_thread
}

#[allow(dead_code)]
fn is_bootstrapped(node: &NodeType) -> Result<String, reqwest::Error> {
    let response;
    match node {
        NodeType::Tezedge => {
            response =
                reqwest::blocking::get("http://tezedge-node-run:18732/chains/main/blocks/head")?;
        }
        NodeType::Ocaml => {
            response =
                reqwest::blocking::get("http://ocaml-node-run:8732/chains/main/blocks/head")?;
        }
        NodeType::TezedgeMaster => {
            response = 
                reqwest::blocking::get("http:/tezedge-master-node-run:28732/chains/main/blocks/head")?;
        }
    }
    // if there is no response, the node has not started bootstrapping
    if response.status().is_success() {
        let response_node: serde_json::value::Value =
            serde_json::from_str(&response.text()?).expect("JSON was not well-formatted");

        Ok(response_node["header"]["level"]
            .to_string()
            .replace("\"", ""))
    } else {
        Ok(String::new())
    }
}