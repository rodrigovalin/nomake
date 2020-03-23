use anyhow::Result;
mod kind;

use std::fs;
use std::path::Path;
use std::vec::Vec;

use console::Style;

use crate::kind::Kind;
use structopt::StructOpt;

const DEFAULT_NAME: &str = "nomake-default";

#[derive(StructOpt, Debug)]
#[structopt(name = "Kind")]
/// The kind bla
enum Opt {
    /// Creates a kind cluster
    Create {
        /// Name of the cluster
        #[structopt(long, default_value = DEFAULT_NAME)]
        name: String,

        /// Configures access to an ECR private registry
        #[structopt(long)]
        ecr: Option<String>,

        /// Configure access to local Docker registry
        #[structopt(long)]
        use_local_registry: Option<String>,

        /// Verbose
        #[structopt(short)]
        verbose: bool,
    },
    /// Deletes a kind cluster
    Delete {
        /// Name of the cluster
        #[structopt(long, default_value = DEFAULT_NAME)]
        name: String,
    },
    /// Get cluster configuration
    Config {
        /// name of the cluster
        #[structopt(long, default_value = DEFAULT_NAME)]
        name: String,

        /// Make the output "evalable"
        #[structopt(long)]
        env: bool,
    },
    /// Display list of known clusters
    List,
    /// Removes clusters that are not reachable anymore
    Clean {
        /// Force removal of directories
        #[structopt(long)]
        force: bool,
    },
}

fn create(
    name: String,
    ecr: Option<String>,
    use_local_registry: Option<String>,
    verbose: bool,
) -> Result<()> {
    let mut cluster = Kind::new(&name);
    cluster.configure_private_registry(ecr);

    if let Some(container_name) = use_local_registry {
        cluster.use_local_registry(&container_name)
    }

    cluster.set_verbose(verbose);

    let cyan = Style::new().cyan();
    println!("Creating cluster: {}", cyan.apply_to(name));
    cluster.create()
}

fn delete(name: String) -> Result<()> {
    let cluster = Kind::new(&name);

    let cyan = Style::new().cyan();
    println!("Deleting cluster: {}", cyan.apply_to(name));
    cluster.delete()
}

fn config(name: String, env: bool) -> Result<()> {
    let cluster = Kind::new(&name);

    if env {
        println!("export KUBECONFIG={}", cluster.get_kube_config());
    } else {
        println!("{}", cluster.get_kube_config());
    }

    Ok(())
}

fn all_clusters() -> Vec<String> {
    let mut clusters = Vec::new();

    if let Ok(config) = Kind::get_config_dir() {
        let config = Path::new(&config);
        for entry in fs::read_dir(config).expect("could not read dir") {
            let entry = entry.unwrap();
            let entry = entry.file_name().to_str().unwrap().to_string();
            clusters.push(entry);
        }
    }

    clusters
}

fn list() {
    for cluster in all_clusters() {
        println!("{}", cluster);
    }
}

fn clean(force: bool) -> Result<()> {
    let kc = Kind::get_kind_containers()?;
    let clusters = all_clusters();

    for cluster in clusters {
        if !kc.iter().any(|c| *c == cluster) {
            let dir = format!("{}/{}", Kind::get_config_dir()?, cluster);
            if force {
                println!("Removing {}", dir);
                fs::remove_dir_all(dir)?
            } else {
                println!("Not removing {}. Use --force", dir);
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let matches = Opt::from_args();

    match matches {
        Opt::Create {
            name,
            ecr,
            use_local_registry,
            verbose,
        } => create(name, ecr, use_local_registry, verbose),
        Opt::Delete { name } => delete(name),
        Opt::Config { name, env } => config(name, env),
        Opt::List => Ok(list()),
        Opt::Clean { force } => clean(force),
    }
}
