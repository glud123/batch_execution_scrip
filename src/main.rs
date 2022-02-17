use serde_json::Value;
use std::path::{self, Path};
use std::process::Command;
use std::{env, fs, io, thread};

fn main() {
    let args: Vec<String> = env::args().collect();

    let dir_path = &args[1];
    let scripts = &args[2..];
    let mut path_list: Vec<path::PathBuf> = Vec::new();

    visit_dirs(Path::new(dir_path), &mut path_list).unwrap();
    for path in path_list {
        println!("{:?}", path.parent());
        find_script(scripts.to_vec(), path);
    }
}

fn visit_dirs(dir: &Path, list: &mut Vec<path::PathBuf>) -> io::Result<()> {
    if dir.is_dir() {
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

fn execute_script(pre_command: &String, command: &String) {
    if cfg!(target_os = "windows") {
        let script = Command::new("cmd")
            .arg("/c")
            .arg(String::from(pre_command) + command)
            .spawn()
            .expect("cmd exec error!")
            .wait_with_output()
            .expect("failed to wait on child");
        if script.status.success() {
            println!("🎉 The script was executed successfully!");
        }
    } else {
        println!("{:?}", command);
        let script = Command::new("sh")
            .arg("-c")
            .arg(String::from(pre_command) + command)
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
                                                let pre_command = String::from("cd ")
                                                    + parent_path.to_str().unwrap()
                                                    + "; ";
                                                execute_script(&pre_command, &command);
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
        handler.join().unwrap();
    };
    expensive_closure(scripts, path)
}
