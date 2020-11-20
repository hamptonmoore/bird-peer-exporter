use std::process::Command;
use std::env;

extern crate serde_json;
extern crate serde;

#[derive(serde::Serialize)]
struct Routes {
    imported: u16,
    exported: u16,
    preferred: u16
}

#[derive(serde::Serialize)]
struct Peer {
    name: String,
    protocol: String,
    bgp_state: String,
    neighbor_address: String,
    description: String,
    neighbor_as: u32,
    neighbor_id: String,
    v4_routes: Routes,
    v6_routes: Routes
}

fn main() {

    let output = Command::new(env::args().nth(1).expect("First argument should be path to birdc"))
        .args(&["show", "proto", "all"])
        .output()
        .expect("failed to execute process");


    let birdc_output: Vec<String> = String::from_utf8_lossy(&output.stdout).split("\n").map(|s| s.to_string()).collect();

    let mut peers: Vec<Peer> = Vec::new();
    let mut scanning_peer = false;
    let mut peer_index:usize = 0;
    let mut channel_check: String = String::new();
    let mut channel_checking: bool = false;
    let mut channel_up: bool = false;

    for line in &birdc_output{

        let line_split: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();

        if channel_checking {
            if line_split[1].as_str() == "UP" {
                channel_up = true;
                peers[peer_index].protocol.push_str(channel_check.as_str());

                if peers[peer_index].protocol.as_str() == "ipv4ipv6" {
                    peers[peer_index].protocol = "MP".to_string();
                }
            }
            channel_checking = false;
        }

        if !line.starts_with(" ") {
           if line.contains("BGP") {
               peer_index = peers.len();
               scanning_peer = true;

               peers.push(Peer {
                   name: line_split[0].clone(),
                   protocol: "".to_string(),
                   bgp_state: line_split[5].clone(),
                   neighbor_address: "".to_string(),
                   description: "".to_string(),
                   neighbor_as: 0,
                   neighbor_id: "".to_string(),
                   v4_routes: Routes {
                       imported: 0,
                       exported: 0,
                       preferred: 0
                   },
                   v6_routes: Routes {
                       imported: 0,
                       exported: 0,
                       preferred: 0
                   },
               });
           } else {
               scanning_peer = false;
           }
        } else if scanning_peer {
            match line_split[0].as_str() {
                "Description:" => {
                    peers[peer_index].description = line_split[1].clone();
                }
                "Neighbor" => {
                    match line_split[1].as_str() {
                        "AS:" => {
                            peers[peer_index].neighbor_as = line_split[2].parse::<u32>().unwrap();
                        },
                        "ID:" => {
                            peers[peer_index].neighbor_id = line_split[2].clone();
                        },
                        "address:" => {
                            peers[peer_index].neighbor_address = line_split[2].clone();
                        },
                        _ => {}
                    }
                }
                "Channel" => {
                    channel_checking = true;
                    channel_up = false;
                    channel_check = line_split[1].clone();
                },
                "Routes:" => {
                  if channel_up {
                      if channel_check == "ipv4" {
                          peers[peer_index].v4_routes.imported = line_split[1].parse::<u16>().unwrap();
                          peers[peer_index].v4_routes.exported = line_split[3].parse::<u16>().unwrap();
                          peers[peer_index].v4_routes.preferred = line_split[5].parse::<u16>().unwrap();
                      } else if channel_check == "ipv6" {
                          peers[peer_index].v6_routes.imported = line_split[1].parse::<u16>().unwrap();
                          peers[peer_index].v6_routes.exported = line_split[3].parse::<u16>().unwrap();
                          peers[peer_index].v6_routes.preferred = line_split[5].parse::<u16>().unwrap();
                      }
                  }
                },
                _ => {}
            }
        }
    }

    println!("{}", serde_json::to_string(&peers).expect("Oop"));
}
