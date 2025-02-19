use clap::{arg, command, Parser, ValueEnum};

mod repack;
mod utils;
mod unpack;
mod riff;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct StartArguments {
    #[arg(short, long = "input", default_value = "input")]
    input_path: String,

    #[arg(short, long = "unpacked", default_value = "unpacked")]
    unpacked_path: String,

    #[arg(short, long = "repacked", default_value = "repacked")]
    repacked_path: String,

    #[arg(short, long, default_value = "false")]
    verbose: bool,

    #[arg(long = "img", default_value = "false")]
    extract_images: bool,
    
    #[arg(short, long)]
    mode: Mode,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
enum Mode {
    X,
    Xr,
    R,
}

fn main() {
    let args = StartArguments::parse();

    println!("Disney Cars 2006 PAK unpacker/repacker");
    println!(" Made with ❤️  by ReeZey");
    println!();
    println!("--- Arguments ---");
    println!("Mode: {:?}", args.mode);
    println!("Input path: {}", args.input_path);
    println!("Unpacked path: {}", args.unpacked_path);
    println!("Repacked path: {}", args.repacked_path);
    if args.verbose {
        println!("Verbose mode enabled");
    }
    if args.extract_images && args.mode != Mode::R {
        println!("Extracting images enabled");
    }
    println!();

    match args.mode {
        Mode::X => {
            println!("--- Unpacking files ---\n");
            unpack::all(args)
        },
        Mode::Xr => {
            println!("--- Unpacking files ---\n");
            unpack::all(args.clone());

            println!("--- Repacking files ---\n");
            repack::all(args);
        },
        Mode::R => {
            println!("--- Repacking files ---\n");
            repack::all(args)
        },
    }

    println!("done bye");
}
