use clap::{Parser, Subcommand};
use docker::Docker;
use docker_types::ContainerInfo;
use serde_json::to_string_pretty;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a container
    Run {
        /// Image name
        image: String,
        /// Container name
        #[arg(short, long)]
        name: Option<String>,
        /// Port mappings (host:container)
        #[arg(short, long)]
        port: Vec<String>,
    },
    /// List containers
    Ps {
        /// Show all containers (default shows just running)
        #[arg(short, long)]
        all: bool,
    },
    /// Stop a container
    Stop {
        /// Container ID or name
        container: String,
    },
    /// Remove a container
    Rm {
        /// Container ID or name
        container: String,
    },
    /// Build an image
    Build {
        /// Path to Dockerfile
        path: String,
        /// Image name
        #[arg(short, long)]
        tag: String,
    },
    /// List images
    Images,
    /// Pull an image
    Pull {
        /// Image name
        image: String,
        /// Tag name
        #[arg(short, long, default_value = "latest")]
        tag: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut docker = Docker::new().unwrap();

    match cli.command {
        Commands::Run { image, name, port } => {
            let container = docker
                .run(image, name, port, None, None, None, false, false)
                .await
                .unwrap();
            println!("Container created: {}", container.id);
        }
        Commands::Ps { all } => {
            let containers = docker.list_containers(all).await.unwrap();
            for container in containers {
                println!("{}", to_string_pretty(&container).unwrap());
            }
        }
        Commands::Stop { container } => {
            docker.stop_container(&container).await.unwrap();
            println!("Container stopped: {}", container);
        }
        Commands::Rm { container } => {
            docker.remove_container(&container).await.unwrap();
            println!("Container removed: {}", container);
        }
        Commands::Build { path, tag } => {
            let image = docker
                .build_image(&path, &tag, false, false, false)
                .await
                .unwrap();
            println!("Image built: {}", image.id);
        }
        Commands::Images => {
            let images = docker.list_images().await.unwrap();
            for image in images {
                println!("{}", to_string_pretty(&image).unwrap());
            }
        }
        Commands::Pull { image, tag } => {
            let image_info = docker.pull_image(&image, &tag).await.unwrap();
            println!("Image pulled: {}", image_info.id);
        }
    }
}
