#![allow(non_snake_case)]
///
/// Digital Ocean Kubernetes
///
use reqwest;
use reqwest::header::CONTENT_TYPE;
use reqwest::StatusCode;

use anyhow::Result;
use console::Style;
use dirs;

use std::fs::{create_dir, remove_dir_all, File};
use std::io::prelude::*;
use std::vec::Vec;
use std::{env, io, thread, time};

use serde_derive::{Deserialize, Serialize};

const ENV_DO_PROVIDER: &str = "HAKE_PROVIDER_DIGITALOCEAN_API_KEY";

#[derive(Serialize)]
struct NodePool {
    size: String,
    count: u16,
    name: String,
}

#[derive(Serialize)]
struct Cluster {
    name: String,
    region: String,
    version: String,
    node_pools: Vec<NodePool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct KubernetesCluster {
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    kubernetes_cluster: KubernetesCluster,
}

pub fn create(name: &str) {
    // TODO: parameterize
    let new_cluster = Cluster {
        name: String::from(name),
        region: String::from("lon1"),
        version: String::from("1.17.5-do.0"),
        node_pools: vec![NodePool {
            size: String::from("s-6vcpu-16gb"),
            count: 2,
            name: String::from("this-nodepool"),
        }],
    };

    let api_key = env::var(ENV_DO_PROVIDER).unwrap();
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post("https://api.digitalocean.com/v2/kubernetes/clusters")
        .bearer_auth(&api_key)
        .header(CONTENT_TYPE, "application/json")
        .json(&new_cluster)
        .send()
        .unwrap();

    if resp.status() != StatusCode::CREATED {
        println!("Could not create cluster, status is {}", resp.status());
        println!("Text: {:?}", resp.text());

        return;
    }

    let cyan = Style::new().cyan();
    let json_response: Response = resp.json().unwrap();
    println!(
        "Cluster created with id: {}",
        cyan.apply_to(&json_response.kubernetes_cluster.id)
    );

    let cluster_dir = format!("{}/{}", get_config_dir(), name);
    create_dir(&cluster_dir).unwrap();

    let url = format!(
        "https://api.digitalocean.com/v2/kubernetes/clusters/{}/kubeconfig",
        json_response.kubernetes_cluster.id
    );

    // need to wait for the server to be "prepared"
    let ten_secs = time::Duration::from_secs(10);
    thread::sleep(ten_secs);

    let mut resp = client
        .get(&url)
        .bearer_auth(&api_key)
        .header(CONTENT_TYPE, "application/json")
        .send()
        .unwrap();

    let mut out =
        File::create(format!("{}/kubeconfig", &cluster_dir)).expect("failed to create file");
    io::copy(&mut resp, &mut out).expect("failed to copy content");

    let mut cluster_uuid = File::create(format!("{}/cluster_uuid", &cluster_dir)).unwrap();

    cluster_uuid
        .write_all(&json_response.kubernetes_cluster.id.as_bytes())
        .unwrap();
}

fn get_config_dir() -> String {
    let home = String::from(
        dirs::home_dir()
            .expect("User does not have a home")
            .to_str()
            .unwrap(),
    );

    format!("{}/.hake", home)
}

pub fn delete(name: &str) -> Result<()> {
    let api_key = env::var(ENV_DO_PROVIDER).unwrap();

    let doid = format!("{}/{}/cluster_uuid", get_config_dir(), name);
    let mut file = File::open(doid)?;
    let mut cluster_id = String::new();
    file.read_to_string(&mut cluster_id)?;

    let client = reqwest::blocking::Client::new();
    client
        .delete(&format!(
            "https://api.digitalocean.com/v2/kubernetes/clusters/{}",
            cluster_id
        ))
        .bearer_auth(&api_key)
        .send()?;

    remove_dir_all(format!("{}/{}", get_config_dir(), name))?;

    Ok(())
}
