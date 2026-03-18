use clap::{Parser, Subcommand};
use docker::Docker;
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
        /// Build context path
        context: String,
        /// Image name
        #[arg(short, long)]
        tag: String,
        /// Path to Dockerfile
        #[arg(short, long)]
        dockerfile: Option<String>,
        /// Do not use cache
        #[arg(long)]
        no_cache: bool,
        /// Target stage for multi-stage builds
        #[arg(long)]
        target: Option<String>,
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
    let mut docker = match Docker::new() {
        Ok(docker) => docker,
        Err(e) => {
            eprintln!("Error initializing Podman: {:?}", e);
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Run { image, name, port } => match docker
            .run(image, name, port, None, None, None, false, false)
            .await
        {
            Ok(container) => println!("Container created: {}", container.id),
            Err(e) => eprintln!("Error running container: {:?}", e),
        },
        Commands::Ps { all } => match docker.list_containers(all).await {
            Ok(containers) => {
                for container in containers {
                    println!("{}", to_string_pretty(&container).unwrap());
                }
            }
            Err(e) => eprintln!("Error listing containers: {:?}", e),
        },
        Commands::Stop { container } => match docker.stop_container(&container).await {
            Ok(_) => println!("Container stopped: {}", container),
            Err(e) => eprintln!("Error stopping container: {:?}", e),
        },
        Commands::Rm { container } => match docker.remove_container(&container).await {
            Ok(_) => println!("Container removed: {}", container),
            Err(e) => eprintln!("Error removing container: {:?}", e),
        },
        Commands::Build {
            context,
            tag,
            dockerfile: _,
            no_cache,
            target: _,
        } => match docker
            .build_image(&context, &tag, false, no_cache, false)
            .await
        {
            Ok(image) => println!("Image built: {}", image.id),
            Err(e) => eprintln!("Error building image: {:?}", e),
        },
        Commands::Images => match docker.list_images().await {
            Ok(images) => {
                for image in images {
                    println!("{}", to_string_pretty(&image).unwrap());
                }
            }
            Err(e) => eprintln!("Error listing images: {:?}", e),
        },
        Commands::Pull { image, tag } => match docker.pull_image(&image, &tag).await {
            Ok(image_info) => println!("Image pulled: {}", image_info.id),
            Err(e) => eprintln!("Error pulling image: {:?}", e),
        },
    }
}
