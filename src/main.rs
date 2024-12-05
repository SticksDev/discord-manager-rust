use reqwest::StatusCode;
use tracing::{error, info, info_span};

async fn check_discord_token(token: &str) -> bool {
    info!("Checking token...");

    let client = reqwest::Client::new();
    let response = client
        .get("https://discord.com/api/v10/users/@me")
        .header("Authorization", format!("{}", token))
        .send()
        .await;

    match response {
        Ok(response) => {
            if response.status().is_success() {
                let json: serde_json::Value = response.json().await.unwrap_or_default();
                info!(
                    "Token is valid! Welcome back {} ({})",
                    json["username"], json["id"]
                );
                true
            } else {
                error!(
                    "Token is invalid: {:?}",
                    response.text().await.unwrap_or_default()
                );
                false
            }
        }
        Err(e) => {
            error!("Failed to check token: {:?}", e);
            false
        }
    }
}

#[derive(Debug, Clone)]
struct Guild {
    id: String,
    name: String,
}

impl Guild {
    fn new(id: String, name: String) -> Self {
        Self { id, name }
    }
}

async fn get_guilds(token: &str) -> Vec<Guild> {
    info!("Getting guilds...");

    let client = reqwest::Client::new();
    let response = client
        .get("https://discord.com/api/v10/users/@me/guilds")
        .header("Authorization", format!("{}", token))
        .send()
        .await;

    match response {
        Ok(response) => {
            if response.status().is_success() {
                let json: Vec<serde_json::Value> = response.json().await.unwrap_or_default();
                let guilds = json
                    .into_iter()
                    .map(|guild| {
                        Guild::new(
                            guild["id"].as_str().unwrap_or_default().to_string(),
                            guild["name"].as_str().unwrap_or_default().to_string(),
                        )
                    })
                    .collect();

                info!("Successfully got guilds!");
                guilds
            } else {
                error!(
                    "Failed to get guilds: {:?}",
                    response.text().await.unwrap_or_default()
                );
                Vec::new()
            }
        }
        Err(e) => {
            error!("Failed to get guilds: {:?}", e);
            Vec::new()
        }
    }
}

async fn leave_guild(token: &str, guild_id: &str) -> bool {
    info!("Leaving guild {}...", guild_id);

    let client = reqwest::Client::new();
    let response = client
        .delete(&format!(
            "https://discord.com/api/v10/users/@me/guilds/{}",
            guild_id
        ))
        .header("Authorization", format!("{}", token))
        .send()
        .await;

    match response {
        Ok(response) => {
            if response.status() == StatusCode::NO_CONTENT {
                info!("Successfully left guild {}!", guild_id);
                true
            } else {
                error!(
                    "Failed to leave guild {}: {:?}",
                    guild_id,
                    response.text().await.unwrap_or_default()
                );
                false
            }
        }
        Err(e) => {
            error!("Failed to leave guild {}: {:?}", guild_id, e);
            false
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let main_span = info_span!("DiscordManager");
    let _main_span_guard = main_span.enter();

    info!("Initializing...");

    let token = std::env::args()
        .nth(1)
        .unwrap_or_else(|| std::fs::read_to_string("token.txt").unwrap_or_default());

    if token.trim().is_empty() {
        error!("No token provided! Please provide a token in token.txt or as an argument.");
        std::process::exit(1);
    }

    let token = token.trim();
    if !check_discord_token(token).await {
        error!("Invalid token provided! Please provide a valid token.");
        std::process::exit(1);
    }

    info!("Successfully initialized! Dropping to main prompt.");

    loop {
        println!("What would you like to do?");
        println!("1. Mass leave guilds");
        println!("2. Exit");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => {
                let guilds = get_guilds(token).await;
                if guilds.is_empty() {
                    println!("No guilds found.");
                    continue;
                }

                // Ask for each guild if they want to leave
                for guild in guilds {
                    println!("Would you like to leave guild {} (y/n)?", guild.name);

                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();

                    match input.trim() {
                        "y" => {
                            if leave_guild(token, &guild.id).await {
                                println!("Successfully left guild {}!", guild.name);
                            } else {
                                println!("Failed to leave guild {}!", guild.name);
                            }
                        }
                        "n" => {
                            println!("Skipped leaving guild {}.", guild.name);
                        }
                        _ => {
                            println!("Invalid input! Please try again.");
                        }
                    }
                }
            }
            "2" => break,
            _ => println!("Invalid input! Please try again."),
        }
    }

    Ok(())
}
