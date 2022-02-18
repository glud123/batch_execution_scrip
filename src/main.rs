use serde_json::Value;
use std::ffi::OsStr;
use std::path::{self, Path};
use std::process::Command;
use std::{env, fs, io, thread};

#[warn(unused_must_use)]
fn main() {
    let args: Vec<String> = env::args().collect();
    let dir_path = &args[1];
    let scripts = &args[2..];
    let mut path_list: Vec<path::PathBuf> = Vec::new();

    visit_dirs(Path::new(dir_path), &mut path_list).unwrap();
    for path in path_list {
        find_script(scripts.to_vec(), path);
    }
}

fn visit_dirs(dir: &Path, list: &mut Vec<path::PathBuf>) -> io::Result<()> {
    if dir.is_dir() && Some(OsStr::new("node_modules")) != dir.file_name() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, list)?;
            } else {
                let package_path = Path::new("package.json");
                let is_package = path.ends_with(package_path);
                if is_package {
                    list.push(path)
                }
            }
        }
    }
    Ok(())
}

fn execute_script(file_path: &String, command: &String) {
    let path = fs::canonicalize(file_path).unwrap();
    println!(
        "📂 Current file path {:?} \n🚀 Start script execution ===> {:?} ",
        file_path, command
    );
    if cfg!(target_os = "windows") {
        let script = Command::new("cmd")
            .current_dir(&path)
            .arg("/c")
            .arg(command)
            .spawn()
            .expect("cmd exec error!")
            .wait_with_output()
            .expect("failed to wait on child");
        if script.status.success() {
            println!("🎉 The script was executed successfully!");
        }
    } else {
        let script = Command::new("sh")
            .current_dir(&path)
            .arg("-c")
            .arg(command)
            .spawn()
            .expect("sh exec error!")
            .wait_with_output()
            .expect("failed to wait on child");
        if script.status.success() {
            println!("🎉 The script was executed successfully!");
        }
    }
}

fn find_script(scripts: Vec<String>, path: path::PathBuf) {
    let expensive_closure = |scripts: Vec<String>, path: path::PathBuf| {
        let handler = thread::spawn(move || {
            let json_file = fs::File::open(&path);
            let json_file = match json_file {
                Ok(file) => file,
                Err(e) => return Err(e),
            };

            let json: Value = serde_json::from_reader(json_file).unwrap();

            if json["scripts"].is_object() {
                match &json["scripts"] {
                    Value::Object(x) => {
                        for script in scripts {
                            if x.contains_key(&script) {
                                match &x[&script] {
                                    Value::String(command) => {
                                        match path.parent() {
                                            Some(parent_path) => {
                                                let file_path =
                                                    String::from(parent_path.to_str().unwrap());
                                                execute_script(&file_path, &command);
                                            }
                                            None => {
                                                println!("🚫 The script was not executed!");
                                            }
                                        };
                                    }
                                    _ => (),
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
            Ok(())
        });
        handler
            .join()
            .expect("Couldn't join on the associated thread");
    };
    expensive_closure(scripts, path)
}
