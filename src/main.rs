use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex};
use std::thread;
use std::collections::HashSet;
use colored::*;

static CONSOLE_METHODS: &[&str] = &[
    "assert", "clear", "count", "countReset", "debug", "dir", "dirxml",
    "error", "group", "groupCollapsed", "groupEnd", "info", "log",
    "table", "time", "timeEnd", "timeLog", "timeStamp", "trace", "warn",
];

lazy_static::lazy_static! {
    static ref ALLOWED_METHODS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

fn remove_console_logs_from_file(file_path: &str) -> io::Result<()> {
    let mut code = String::new();
    let mut file = File::open(file_path)?;
    file.read_to_string(&mut code)?;

    let mut lines: Vec<String> = code.lines().map(|line| line.to_string()).collect();

    for line in &mut lines {
        for method in ALLOWED_METHODS.lock().unwrap().iter() {
            if line.contains(&format!("console.{}(", method)) {
                *line = String::new();
                break;
            }
        }
    }

    let updated_code = lines.join("\n");
    let mut file = File::create(file_path)?;
    file.write_all(updated_code.as_bytes())?;

    Ok(())
}

fn process_file(file_path: PathBuf) {
    let ext = file_path.extension().unwrap_or_default().to_str().unwrap_or_default();
    if ext == "js" || ext == "ts" || ext == "jsx" || ext == "tsx" {
        if let Err(err) = remove_console_logs_from_file(file_path.to_str().unwrap()) {
            eprintln!("{}", format!("Error processing file {}: {}", file_path.display(), err).red());
        } else {
            println!("{}", format!("Processed: {}", file_path.display()).green());
        }
    }
}

fn process_directory(dir_path: PathBuf) {
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let full_path = entry.path();
            if full_path.is_dir() {
                if !is_excluded_dir(&entry.file_name()) {
                    process_directory(full_path);
                }
            } else {
                let file_path = full_path.clone();
                thread::spawn(move || process_file(file_path));
            }
        }
    }
}

fn is_excluded_dir(dir_name: &std::ffi::OsStr) -> bool {
    let excluded_dirs: HashSet<String> = [
        "node_modules", ".git", "dist", "build",
    ].iter().map(|&s| s.to_string()).collect();
    excluded_dirs.contains(&dir_name.to_string_lossy().to_string())
}

fn main() {
    if std::env::args().len() < 2 {
        eprintln!("Usage: js-logs-remover [path] [log-methods]");
        return;
    }

    let args: Vec<String> = std::env::args().skip(2).collect();

    if args.len() == 1 && args[0] == "all" {
        for &method in CONSOLE_METHODS {
            ALLOWED_METHODS.lock().unwrap().insert(method.to_string());
        }
    } else {
        for arg in args {
            let methods: Vec<&str> = arg.split(',').collect();
            for method in methods {
                ALLOWED_METHODS.lock().unwrap().insert(method.to_string());
            }
        }
    }

    let target_dir = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());

    process_directory(Path::new(&target_dir).to_path_buf());

    println!("{}", "✅ All selected console methods removed!".green());
}
