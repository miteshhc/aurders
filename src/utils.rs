//! utils module includes all the utlity and helper functions
use std::fs::{self, remove_file, File};
use std::io::{self, Cursor, ErrorKind, Write};
use std::env;
use std::path::{Path, PathBuf};
use std::process::exit;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use reqwest;
use sha256::try_digest;
use tar::{Archive, Builder};

/// input_string gets user input in the form of string, trims and then returns it
pub fn input_string(prompt: &str, default: &str) -> String {
    let mut input = String::new();

    println!("\n{}", prompt);
    print!("> ");
    io::stdout().flush().unwrap();

    match io::stdin().read_line(&mut input) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Unable to take input: {}.", e);
            dead();
        }
    };

    // remove any extra whitespaces
    input = input.trim().to_string();

    if input.is_empty() {
        return default.to_string();
    }

    input
}

/// input_string_strict is a more strict version of input_string, which gets string input from
/// user and returns the trimmed string
pub fn input_string_strict(prompt: &str) -> String {
    loop {
        let mut input = String::new();

        println!("\n{}", prompt);
        print!("> ");
        io::stdout().flush().unwrap();

        match io::stdin().read_line(&mut input) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Unable to take input: {}.", e);
                dead();
            }
        };

        // remove any extra whitespaces
        input = input.trim().to_string();

        if !input.is_empty() {
            return input;
        } else {
            eprintln!("This field is not optional. Try again.");
        }
    }
}

/// input_bool gets user input in the form of string, then returns true if the input is y or Y,
/// false otherwise
pub fn input_bool(prompt: &str) -> bool {
    let mut input = String::new();

    println!("\n{}", prompt);
    print!("> ");
    io::stdout().flush().unwrap();

    match io::stdin().read_line(&mut input) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Unable to take input: {}.", e);
            dead();
        }
    };

    match input.trim() {
        "y" | "Y" | "yes" | "definitely" => return true,
        _ => return false,
    };
}

/// get_sha256 performs sha256 digest generation and returns it
pub fn get_sha256(tarball: &String) -> Option<String> {
    let input = Path::new(&tarball);
    let value_result = try_digest(input);

    match value_result {
        Ok(value) => return Some(value),
        Err(e) => {
            eprintln!(
                "Failed to get sha256: {}.\nUsing 'SKIP' as default value.",
                e
            );
            return None;
        }
    };
}

/// create_tarball creates tarball of given source and returns the name of tarball
pub fn create_tarball(source: &PathBuf) -> Result<String, std::io::Error> {
    let source_file = match source.file_name() {
        Some(name) => match name.to_str() {
            Some(name_str) => name_str,
            None => {
                eprintln!("Failed to convert: &OsStr -> &str");
                dead();
                &"ERRROOORRR".to_string()
            }
        },
        None => {
            eprintln!("Failed to extract filename from source.");
            dead();
            &"ERRROOORRR".to_string()
        }
    };

    let tarball_name = format!("aurders/{}.tar.gz", source_file);

    let tar_gz = File::create(&tarball_name)?;

    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    match tar.append_dir_all(&source_file, &source) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to append source to tarball. Make sure source is a directory.");
            eprintln!("Got: {}.", e);
            dead();
            return Err(e);
        }
    };

    Ok(tarball_name)
}

/// select_arch functions allows user to choose from architectures easily
pub fn select_arch() -> Option<String> {
    println!("\nSelect the target architecture for your package:");
    io::stdout().flush().unwrap(); // Flush the output correctly

    loop {
        print!("  [1] x86_64(Default)    [2] i686    [3] any    [4] Enter manually\n> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(_) => (),
            Err(e) => eprintln!("Invalid input: {}", e),
        };

        let arch: u8 = match input.trim().parse() {
            Ok(ip) => ip,
            Err(_) => 1, // x86_64 as default arch
        };

        match arch {
            1 => return Some("x86_64".to_string()),
            2 => return Some("i686".to_string()),
            3 => return Some("any".to_string()),
            4 => {
                let mut arch = String::new();
                print!("Enter target architecture: ");
                io::stdout().flush().unwrap();

                io::stdin()
                    .read_line(&mut arch)
                    .expect("Failed to get input.");

                // nasty, but works
                return Some(arch.trim().replace(" ", "' '").to_string());
            }
            _ => {
                eprintln!("Invalid input. Try again");
            }
        };
    }
}

/// create_directory creates directory according to given path
pub fn create_directory(path: String) {
    match fs::create_dir(&path) {
        Ok(_) => println!("Created directory {}.", &path),
        Err(e) => match e.kind() {
            ErrorKind::AlreadyExists => println!("Directory already exists."),
            ErrorKind::PermissionDenied => {
                eprintln!("Cannot create directory, permission denied");
                dead();
            }
            _ => {
                eprintln!(
                    "Failed to create directory. Unknown error occurred.\nPath: {}.",
                    &path
                );
                dead();
            }
        },
    };
}

/// decompress_tarball decompresses the tarball specified at tarball_path
fn decompress_tarball(tarball_path: String) -> Result<(), std::io::Error> {
    let tar_gz = File::open(tarball_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(".")?;

    Ok(())
}

/// fetch_data fetches the data from given url and writes to given filename
fn fetch_data(url: String, filename: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("Attempting to fetch {}...", filename);
    let response = reqwest::blocking::get(url)?.bytes()?;
    let mut file = File::create(filename)?;
    let mut content = Cursor::new(response);
    io::copy(&mut content, &mut file)?;
    println!("Fetched templates successfully.");

    Ok(())
}

/// get_templates retrieve the template by calling fetch_data() correctly
// not to be confused with get_template functions in {pkgbuild, srcinfo}, they retrieve local
// templates from templates/ directory.
pub fn get_templates() {
    let url = "https://github.com/miteshhc/aurders/releases/download/template/templates.tar.gz";
    let filename = "templates.tar.gz";

    match fetch_data(url.to_string(), filename.to_string()) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Unable to fetch data: {}.", e);
            dead();
        }
    };

    match decompress_tarball(filename.to_string()) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to decompress archive: {}.", e);
            dead();
        }
    };

    match remove_file(filename) {
        Ok(_) => println!("Removed file: {}.", filename),
        Err(e) => eprintln!(
            "Failed to remove {}: {}.\nYou might want to remove it manually.",
            filename, e
        ),
    };
}

/// dead performs any required cleanup and exists the program abnormally
pub fn dead() {
    eprintln!("Exiting...");
    exit(1);
}

/// dead_probably asks the user, if they want to continue or ...
pub fn dead_probably() {
    let mut dead_huh = String::new();

    println!("Do you still want to continue(y/N)");
    print!("> ");

    match io::stdin().read_line(&mut dead_huh) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("argh, even asking for this wouldn't work??? {}", e);
            dead();
        }
    };

    match dead_huh.trim() {
        "y" | "Y" => dead(),
        _ => (),
    };
}

/// get_source gets the source from user
pub fn get_source() -> Option<String> {
    let mut input = String::new();

    println!("\nDo you want to specify source(s) manually?(y/N)");
    print!("> ");
    io::stdout().flush().unwrap();

    match io::stdin().read_line(&mut input) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Unable to take input: {}.", e);
            dead();
        }
    };

    let input = input.trim();

    match input {
        "Y" | "y" => {
            let mut source = String::new();
            print!("\nSource > ");
            io::stdout().flush().unwrap();

            match io::stdin().read_line(&mut source) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Unable to take input: {}.", e);
                    dead();
                }
            }

            return Some(source.trim().to_string());
        }
        _ => None,
    }
}

/// get_arch returns the current architecture
pub fn get_arch() -> String {
    let arch = match env::consts::ARCH {
        "x86_64" => "x86_64",
        // *Untested*
        "x86" => "i686", // arch dropped support in 2017, unofficial port is available
        "arm" => "arm",  // unofficial port is available
        "aarch64" => "aarch64", // again, unofficial port is available (ARM)
        _ => {
            eprintln!("Architecture is not supported by Arch Linux.");
            eprintln!("You might want to modify the file name of package (.pkg.tar.zst).");
            dead_probably();
            "UNSUPPORTED"
        }
    };

    return arch.to_string();
}
