mod repack;
mod utils;
mod unpack;
mod riff;

//const PATH: &str = "C:\\SteamLibrary\\steamapps\\common\\Cars\\Data";
const INPUT_PATH: &str = "input";
const UNPACKED_PATH: &str = "unpacked";
const REPACKED_PATH: &str = "repacked";

const VERBOSE: bool = false;

fn main() {
    println!("Unpacking files...\n");

    unpack::all();

    println!("\ndone with unpacking, continuing with packing\n");

    repack::all();

    println!("\ndone with repacking");
}
