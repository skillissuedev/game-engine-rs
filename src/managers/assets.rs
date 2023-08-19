use super::debugger::crash;
use std::env;

pub fn get_full_asset_path(path: &str) -> String {
    let mut exec_path: String = "".to_string();

    match env::current_exe() {
        Ok(exe_path) => {
            let executable_path = exe_path.to_str();
            match executable_path {
                Some(executable_path_string) => exec_path = executable_path_string.to_owned(), //println!("Path of this executable is: {}", executable_path_string.to_owned() + "/" + path),
                None => crash("Getting current exe path error!"),
            }
        }
        Err(_e) => crash("Getting current exe path error!"),
    };

    let full_exec_path_splitted: Vec<&str> = exec_path.split("/").collect();
    //let full_path = exec_dir_path.to_string() + path;

    let mut full_path: String = "".to_string();

    for i in 0..full_exec_path_splitted.len() - 1 {
        full_path += full_exec_path_splitted[i];
        full_path += "/";
    }

    full_path += "assets/";
    full_path += path;

    if cfg!(windows) {
        return full_path.replace("/", r"\");
    }

    full_path
}
