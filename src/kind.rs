#![allow(non_snake_case)]

use anyhow::Result;
use serde_derive::{Serialize, Deserialize};
use serde_json::json;
use dirs;

use base64::encode;
use std::io::{Read, Write};
use std::fs::{File, create_dir, remove_dir_all};

use std::process::{Command, Stdio};
use std::str;


#[derive(Serialize, Deserialize, Debug)]
struct ExtraMount {
    containerPath: String,
    hostPath: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Node {
    role: String,
    extraMounts: Vec<ExtraMount>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ClusterConfig {
    kind: String,
    apiVersion: String,
    nodes: Vec<Node>,
}

#[derive(Deserialize, Debug)]
struct DockerLogin {
    Username: String,
    Secret: String,
}

pub struct Kind {
    pub name: String,
    pub ecr_repo: String,
    config_dir: String,
}

impl Kind {
    fn get_kind_config(dockerconfig: &str) -> Result<String> {
        let cc = ClusterConfig {
            kind: String::from("Cluster"),
            apiVersion: String::from("kind.sigs.k8s.io/v1alpha3"),
            nodes: vec![
                Node {
                    role: String::from("control-plane"),
                    extraMounts: vec![
                        ExtraMount {
                            containerPath: String::from("/var/lib/kubelet/config.json"),
                            hostPath: String::from(dockerconfig),
                        }
                    ]
                }
            ]
        };

        Ok(serde_yaml::to_string(&cc)?)
    }

    fn get_docker_login(registry: &str) -> Result<String> {
        let creds = Kind::get_docker_credentials_from_helper(registry)?;

        let login: DockerLogin = serde_json::from_str(&creds)?;
        let encoded = encode(&format!("{}:{}", login.Secret, login.Username));

        Ok(
            json!({
                "auths": {
                    registry: {
                        "auth": encoded
                    }
                }
            }
        ).to_string())
    }

    fn get_docker_credentials_from_helper(registry: &str) -> Result<String> {
        let mut cmd = Command::new("docker-credential-ecr-login")
            .arg("get")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        cmd.stdin.as_mut().unwrap().write_all(registry.as_bytes())?;
        cmd.wait()?;

        let mut output = String::new();
        cmd.stdout.unwrap().read_to_string(&mut output)?;

        Ok(output)
    }

    fn create_kind_config(&self) -> Result<()> {
        // save these files where they belong (nomake dir)
        let docker_login = Kind::get_docker_login(&self.ecr_repo)
            .expect("could not get docker login");

        // save docker_login()
        let docker_config_path = format!("{}/docker_config", self.config_dir);
        let mut docker_config = File::create(&docker_config_path)?;
        docker_config.write_all(&docker_login.into_bytes())?;

        let kind_cluster_config = Kind::get_kind_config(&docker_config_path)
            .expect("no kind");

        let kind_config_path = format!("{}/kind_config", self.config_dir);
        let mut kind_config = File::create(&kind_config_path)?;
        kind_config.write_all(&kind_cluster_config.into_bytes())?;

        Ok(())
    }

    pub fn create(&mut self) -> Result<()> {
        let mut args = vec!["create", "cluster"];
        let kubeconfig;

        if self.name != "" {
            // remove home_dir
            let home = String::from(
                dirs::home_dir().expect("user does not have a home").to_str().unwrap());
            self.config_dir = format!("{}/.nomake/{}", home, self.name);
            println!("Config dir is {}", self.config_dir);
            create_dir(&self.config_dir)?;

            args.push("--name");
            args.push(&self.name);

            kubeconfig = format!("{}/kubeconfig", self.config_dir);
            args.push("--kubeconfig");
            args.push(&kubeconfig);
        }

        let config = &format!("{}/kind_config", self.config_dir);
        if self.ecr_repo != "" {
            self.create_kind_config()?;
            args.push("--config");
            args.push(config);
        }

        Command::new("kind")
            .args(args)
            .output()
            .expect("could not find kind");

        Ok(())
    }

    pub fn delete(&self) -> Result<()> {
        let mut args = vec!["delete", "cluster"];
        if self.name != "" {
            args.push("--name");
            args.push(&self.name);
        }

        let _cmd = Command::new("kind")
            .args(args)
            .output()
            .expect("could not find kind");

        remove_dir_all(&self.config_dir)?;
        Ok(())
    }

    pub fn new(name: &str, ecr_repo: &str) -> Kind {
        Kind{
            name: String::from(name),
            ecr_repo: String::from(ecr_repo),
            config_dir: String::new(),
        }
    }
}
