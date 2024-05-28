// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use pixi::cli::{add, init, run, LockFileUsageArgs};
use rattler_conda_types::Platform;
use std::{path::PathBuf, vec};

#[tauri::command]
async fn setup(
    project_path: &str,
    python_version: &str,
) -> Result<String, String> {
    let path = PathBuf::from(project_path);

    // Check if the directory exists
    if !path.exists() {
        // Attempt to create the directory
        std::fs::create_dir_all(&path).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // Check if "pixi.toml" already exists in the directory
    let config_file = path.join("pixi.toml");
    if config_file.exists() {
        return Err("A 'pixi.toml' file already exists in the specified project path.".to_string());
    }

    // Define the arguments for the Pixi init command
    let args = init::Args {
        path: path.clone(),
        channels: None,
        platforms: vec![Platform::current().to_string()],
    };

    // Execute the Pixi init command
    init::execute(args).await.map_err(|e| format!("Failed to initialize Pixi project: {}", e))?;

    let add_args = add::Args {
        specs: vec![format!("python={}", python_version), "spyder".to_string(), "jupyterlab".to_string()],
        manifest_path: Some(path.join("pixi.toml")),
        host: false,
        build: false,
        pypi: false,
        no_lockfile_update: false,
        no_install: false,
        platform: vec![],
    };

    // Use block_in_place to ensure add::execute runs on the current thread
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(add::execute(add_args))
    }).map_err(|e| format!("Failed to add Python dependency: {}", e)).unwrap();

    Ok("Pixi project initialized and Python dependency added successfully.".to_string())
}

#[tauri::command]
async fn launch(project_path: &str, program: String) -> Result<String, String> {
    let path = PathBuf::from(project_path);
    let manifest_path = path.join("pixi.toml");

    let run_args = if program == "spyder" {
        run::Args {
            task: vec!["spyder".to_string()],
            manifest_path: Some(manifest_path),
            lock_file_usage: LockFileUsageArgs::default(),
            environment: None,
        }
    } else if program == "jupyterlab" {
        run::Args {
            task: vec!["jupyter".to_string(), "lab".to_string()],
            manifest_path: Some(manifest_path),
            lock_file_usage: LockFileUsageArgs::default(),
            environment: None,
        }
    } else {
        unimplemented!()
    };


    println!("Launching {}", program);
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(run::execute(run_args))
    }).map_err(|e| format!("Failed to launch Pixi project: {}", e))?;
    Ok("Launching Pixi project...".to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![setup, launch])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
