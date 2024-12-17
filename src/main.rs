mod initialisation;
mod utilities;
mod configuration;
mod user_management;
mod user_object;

use std::io::Write;
use anyhow::Result;
use validator::ValidateEmail;
use rand::Rng;
use std::error::Error;
use configuration::{Config};
use crate::utilities::get_menu_choice;
use serde::{Deserialize};

use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use std::sync::Mutex;
use tera::{Context, Tera};
use std::fs::File;
use std::io::BufReader;
use log::{info, error};

#[derive(Serialize)]
struct User {
    firstname: String,
    lastname: String,
}
struct AppState{
    tmpl: Tera,
    asset_manifest: AssetManifest,
}

#[derive(Deserialize)]
struct AssetManifest {
    files: std::collections::HashMap<String, String>,
}

async fn index(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = data.lock().unwrap();
    let mut ctx = Context::new();
    ctx.insert("title", "Rust + Tera + React 18 Application");
    ctx.insert("user", &User{
        firstname: "Rust".to_string(),
        lastname: "Rover".to_string(),
    });

    if let Some(main_js) = state.asset_manifest.files.get("main.js") {
        ctx.insert("main_js", main_js);
    }
    if let Some(main_css) = state.asset_manifest.files.get("main.css") {
        ctx.insert("main_css", main_css);
    }

    let rendered = match state.tmpl.render("index.html", &ctx){
        Ok(r) => r,
        Err(e) => {
            error!("Template render error: {}", e);
            return HttpResponse::InternalServerError().body("Template rendering error");
        }
    };
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

async fn api_hello() -> impl Responder {
    let response = serde_json::json!({
        "message": "Hello, from Rust!"
    });
    HttpResponse::Ok().json(response)
}

async fn react_index() -> actix_web::Result<fs::NamedFile> {
    let path: std::path::PathBuf = "./static/js/index.html".parse().unwrap();
    Ok(fs::NamedFile::open(path)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Starting server...");

    let tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };

    let manifest_file_path = "static/js/asset-manifest.json";
    let manifest_file = match File::open(manifest_file_path) {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to open {}: {}", manifest_file_path, e);
            std::process::exit(1);
        }
    };

    let reader = BufReader::new(manifest_file);
    let asset_manifest: AssetManifest = match serde_json::from_reader(reader) {
        Ok(m) => m,
        Err(e) => {
            error!("Error parsing asset-manifest.json: {}", e);
            std::process::exit(1);
        }
    };

    let app_state = web::Data::new(Mutex::new(AppState{tmpl: tera, asset_manifest: asset_manifest}));

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .route("/", web::get().to(index))
            .route("/api/hello", web::get().to(api_hello))
            .default_service(web::route().to(react_index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

/*
fn main() -> Result<()> {
    utilities::clear_screen();
    std::io::stdout().flush()?;

    let config_path = utilities::get_config_path();
    let mut config: Config = Config::default();
    if let Some(loaded_config) = Config::load(config_path.to_str().unwrap()) {
        println!("Configuration file found");
        println!("Loaded configuration: {:?}", loaded_config);
        config = loaded_config;
    } else {
        println!("Configuration file not found. Running initialisation...");
        utilities::pause(2);
        initialisation::check_initial(&mut config, config_path.to_str().unwrap());
        config.save(config_path.to_str().unwrap())?;
    }

    utilities::clear_screen();
    loop {
        let choice = get_menu_choice("login");
        match choice {
            1 => {
                let mut count: u32 = 1;
                loop {
                    if count >= 3 {
                        println!("Too many login attempts.");
                        break;
                    }

                    let(user, is_valid) = user_management::login_user(config.database_file.as_deref().expect("Failed to read configuration file."));
                    if is_valid {
                        println!("Logged in Successfully.");
                        // DEBUGGING
                        user.pretty_print();
                        break;
                        // END DEBUGGING
                    } else {
                        count+=1;
                    }
                }
            },
            2 => {
                user_management::register_user(config.database_file.as_deref().expect("Failed to read configuration file."));
            },
            3 => {
                println!("Exiting program...");
                std::process::exit(0);
            },
            0_usize | 4_usize.. => {
                println!("Exiting program...");
                std::process::exit(0);
            }
        }
    }
}
*/