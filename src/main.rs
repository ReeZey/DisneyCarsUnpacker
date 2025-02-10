mod handler;
use std::path::PathBuf;

//const PATH: &str = "C:\\SteamLibrary\\steamapps\\common\\Cars\\Data";
const PATH: &str = "input";
const VERBOSE: bool = false;

fn main() {
    //handler::extract_all(&PathBuf::from(PATH));

    handler::repack_all(&PathBuf::from("output"), &PathBuf::from("repacked"));
}
