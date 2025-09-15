use std::str::FromStr;
use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use backend::http::users::{Organisation, User, UserCreation};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::ConnectOptions;
use comfy_table::Table;
use comfy_table::{Cell, Attribute, Color};

#[derive(Parser)]
#[command(name = "Mockomatic CLI")]
#[command(version, about = "For debugging + testing Mockomatic", long_about = None)]
struct Args {
    #[arg(long)]
    database_url: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    CreateUser {
        #[arg(short, long, value_name = "USERNAME")]
        username: Option<String>,

        #[arg(short, long, value_name = "PASSWORD")]
        password: Option<String>,

        #[arg(short, long, value_name = "ORG")]
        organisation: String,

        #[arg(short, long)]
        admin: bool,
    },
    DeleteUser {
        #[arg(short, long, value_name = "USERNAME")]
        username: String,
    },
    ListUsers,
    ToggleAdmin {
        #[arg(short, long, value_name = "USERNAME")]
        username: String,
    },
    CreateOrg {
        #[arg(short, long, value_name = "NAME")]
        name: String,
    },
    ListOrgs,
}


#[tokio::main]
async fn main() -> Result<()> {
    let mut args = Args::parse();

    if args.database_url.as_deref().is_none(){
        args.database_url = Some(dotenvy::var("DATABASE_URL").context("DATABASE_URL not set")?);
    }

    if let Some(url) = args.database_url.as_deref() {
        let connection_options = PgConnectOptions::from_str(url).unwrap()
            .disable_statement_logging().clone();
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_with(connection_options)
            .await?;
        match &args.command {
            Some(Commands::CreateUser { username, password, organisation, admin }) => {
                let username = username.clone().unwrap_or_else(|| String::from("testuser"));
                let password = password.clone().unwrap_or_else(|| String::from("cookedpwongfrfr"));
                let organisation_id = Organisation::get_id(&pool, &organisation).await?;
                let test_user = UserCreation {
                    username,
                    password,
                    organisation_id,
                };
                User::create(&pool, &test_user, admin).await?;
                println!("User {} created successfully", test_user.username);
                println!("Is Admin: {}", admin);
                Ok(())
            }
            Some(Commands::DeleteUser { username }) => {
                User::delete(&pool, username).await?;
                println!("User {} deleted successfully", username);
                Ok(())
            }
            Some(Commands::ListUsers) => {
                let users = User::get_all(&pool).await?;
                let mut table = Table::new();
                table
                    .load_preset(comfy_table::presets::UTF8_FULL)
                    .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
                    .set_header(vec![
                        Cell::new("ID").add_attribute(Attribute::Bold),
                        Cell::new("Username").add_attribute(Attribute::Bold),
                        Cell::new("Admin").add_attribute(Attribute::Bold),
                        Cell::new("Created At").add_attribute(Attribute::Bold),
                        Cell::new("Last Login").add_attribute(Attribute::Bold),
                    ]);
                for user in users {
                    if user.admin {
                        table.add_row(vec![
                            Cell::new(user.id.to_string()),
                            Cell::new(user.username),
                            Cell::new(user.admin.to_string()).add_attribute(Attribute::Bold).fg(Color::Green),
                            Cell::new(user.created_at.to_string()).fg(Color::Grey),
                            Cell::new(user.last_login.map_or("Null".to_string(), |dt| dt.to_string())).fg(Color::Grey),
                        ]);
                    } else {
                        table.add_row(vec![
                            Cell::new(user.id.to_string()),
                            Cell::new(user.username),
                            Cell::new(user.admin.to_string()).add_attribute(Attribute::Bold).fg(Color::Red),
                            Cell::new(user.created_at.to_string()).fg(Color::Grey),
                            Cell::new(user.last_login.map_or("Null".to_string(), |dt| dt.to_string())).fg(Color::Grey),
                        ]);
                    }
                }
                println!("{table}");
                Ok(())
            }
            Some (Commands::ToggleAdmin { username }) => {
                let new_status = User::toggle_admin(&pool, username).await?;
                println!("User {}: Admin = {}", username, new_status);
                Ok(())
            }
            Some(Commands::CreateOrg { name }) => {
                Organisation::create(&pool, name).await?;
                println!("Organisation {} created successfully", name);
                Ok(())
            }
            Some(Commands::ListOrgs) => {
                let orgs = Organisation::get_all(&pool).await?;
                let mut table = Table::new();
                table
                    .load_preset(comfy_table::presets::UTF8_FULL)
                    .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
                    .set_header(vec![
                        Cell::new("ID").add_attribute(Attribute::Bold),
                        Cell::new("Org Name").add_attribute(Attribute::Bold),
                    ]);
                for org in orgs {
                    table.add_row(vec![
                        Cell::new(org.id.to_string()),
                        Cell::new(org.name),
                    ]);
                }
                println!("{table}");
                Ok(())
            }
            None => {
                println!("No command provided. Use --help for more information.");
                Ok(())
            }
        }
    } else {
        return Err(anyhow::anyhow!("DATABASE_URL not set"));
    }
}
