// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// Actually, we do want this maybe
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use pixi::cli::{add, init, run, task, LockFileUsageArgs};
use rattler_conda_types::Platform;
use rfd::FileDialog;
use std::{path::PathBuf, vec};

#[tauri::command]
async fn set_project_path() -> Option<PathBuf> {
    FileDialog::new().pick_folder()
}

#[tauri::command]
async fn setup<'a>(
    path: PathBuf,
    python_version: &'a str,
    conda_channel: &'a str,
) -> Result<String, String> {
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

    let filtered_conda_channel: Option<Vec<String>> = if conda_channel.is_empty() {
        None
    } else {
        Some(vec![conda_channel.to_string()])
    };
    // Define the arguments for the Pixi init command
    let args = init::Args {
        path: path.clone(),
        channels: filtered_conda_channel,
        platforms: vec![Platform::current().to_string()],
    };

    // Execute the Pixi init command
    init::execute(args)
        .await
        .map_err(|e| format!("Failed to initialize Pixi project: {}", e))?;

    let add_args = add::Args {
        specs: vec![
            format!("python={}", python_version),
            "python.app".to_string(),
            "spyder=5.4.3".to_string(),
            "jupyterlab".to_string(),
        ],
        manifest_path: Some(path.join("pixi.toml")),
        host: false,
        build: false,
        pypi: false,
        no_lockfile_update: false,
        no_install: false,
        platform: vec![],
    };

    // Use block_in_place to ensure add::execute runs on the current thread
    match tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(add::execute(add_args))
    }) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to add Python dependency: {}", e);
            std::process::exit(1);
        }
    }

    // Check if we are on macOS
    if cfg!(target_os = "macos") {
        let pythonw_path = path.join(".pixi/envs/default/bin/pythonw");
        let pythonw_content = std::fs::read_to_string(&pythonw_path)
            .map_err(|e| format!("Failed to read pythonw file: {}", e))?;

        // Replace "python.app" with "pythonapp"
        let modified_content = pythonw_content.replace("python.app", "pythonapp");

        // Write the modified content back to the file
        std::fs::write(&pythonw_path, modified_content)
            .map_err(|e| format!("Failed to write pythonw file: {}", e))?;
    }

    // Finally add tasks to launch spyder and jupyterlab
    let spyder_add_args = task::AddArgs {
        name: "spyder".into(),
        commands: vec![
            "spyder".to_string(),
            "-w".to_string(),
            path.to_string_lossy().to_string(),
        ],
        depends_on: None,
        platform: None,
        feature: None,
        cwd: None,
    };

    let spyder_task_args = task::Args {
        operation: pixi::cli::task::Operation::Add(spyder_add_args),
        manifest_path: Some(path.join("pixi.toml")),
    };

    let _ = task::execute(spyder_task_args);

    let notebook_dir = format!("--notebook-dir={}", path.to_string_lossy());
    let preferred_dir = format!("--preferred-dir={}", path.to_string_lossy());

    let jupyerlab_add_args = task::AddArgs {
        name: "jupyterlab".into(),
        commands: vec![
            "jupyter".to_string(),
            "lab".to_string(),
            notebook_dir,
            preferred_dir,
        ],
        depends_on: None,
        platform: None,
        feature: None,
        cwd: None,
    };

    let jupyter_task_args = task::Args {
        operation: pixi::cli::task::Operation::Add(jupyerlab_add_args),
        manifest_path: Some(path.join("pixi.toml")),
    };

    let _ = task::execute(jupyter_task_args);

    Ok("Pixi project initialized and Python dependency added successfully.".to_string())
}

#[tauri::command]
async fn launch<'a>(path: PathBuf, program: String) -> Result<String, String> {
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
            task: vec!["jupyterlab".to_string()],
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
    })
    .map_err(|e| format!("Failed to launch Pixi project: {}", e))?;
    Ok("Launching Pixi project...".to_string())
}

#[tauri::command]
async fn is_set_up(path: PathBuf) -> bool {
    if path.exists() {
        path.join("pixi.toml").exists()
    } else {
        false
    }
}

// struct State {
//     folder: Option<PathBuf>,
// }

fn main() {
    // let folder = FileDialog::new()
    // .pick_folder();

    // let state = State {
    //     folder,
    // };

    tauri::Builder::default()
        // .manage(state)
        .invoke_handler(tauri::generate_handler![
            set_project_path,
            is_set_up,
            setup,
            launch
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
