use std::io::{self, Read, Write};
use std::fs::File;

struct Information {
    maintainer_name: String,
    maintainer_email: String,
    pkgname: String,
    pkgver: String,
    pkgrel: String,
    pkgdesc: String,
    url: String,
    license: String,
    arch: String,
    depends: String,
    makedepends: String,
    sha256sums: String,
}

fn main() {
    println!("Hello, world!");
    let pkginfo = Information {
        maintainer_name: input_string("Enter the name of maintainer: "),
        maintainer_email: input_string("Enter the email of maintainer: "),
        pkgname: input_string("Enter the name of package: "),
        pkgver: input_string("Enter the version of package: "),
        pkgrel: input_string("Enter the release number of package: "),
        pkgdesc: input_string("Enter the description about package: "),
        url: input_string("Enter the url of package: "),
        license: input_string("Enter the license of package: "),
        arch: input_string("Enter the architecture of package: "),
        depends: input_string("Enter the dependencies of package: "),
        makedepends: input_string("Enter the make dependencies of package: "),
        sha256sums: input_string("Enter the sha256sums of package: "),
    };

    let pkgbuild_result = generate_pkgbuild(&pkginfo);

    match pkgbuild_result {
        Ok(pkgbuild) => {
            println!("Successfully Generated PKGBUILD");
            save_pkgbuild(&pkgbuild);
        }
        Err(e) => {
            println!("Failed to generate PKGBUILD: {}", e);
        }
    }

    let srcinfo_result = generate_srcinfo(&pkginfo);

    match srcinfo_result {
        Ok(srcinfo) => {
            println!("Successfully Generated SRCINFO");
            save_srcinfo(&srcinfo);
        }
        Err(e) => {
            println!("Failed to generate SRCINFO: {}", e);
        }
    }
}

// generate_pkgbuild generates and returns the PKGBUILD
fn generate_pkgbuild(pkginfo: &Information) -> Result<String, std::io::Error> {
    let template = get_pkgbuild();
    let pkgbuild: String;

    match template {
        Ok(output) => {
            println!("Got PKGBUILD template");
            pkgbuild = output
                    .replace("{maintainer_name}", &pkginfo.maintainer_name)
                    .replace("{maintainer_email}", &pkginfo.maintainer_email)
                    .replace("{pkgname}", &pkginfo.pkgname)
                    .replace("{pkgver}", &pkginfo.pkgver)
                    .replace("{pkgrel}", &pkginfo.pkgrel)
                    .replace("{pkgdesc}", &pkginfo.pkgdesc)
                    .replace("{arch}", &pkginfo.arch)
                    .replace("{url}", &pkginfo.url)
                    .replace("{license}", &pkginfo.license)
                    .replace("{depends}", &pkginfo.depends)
                    .replace("{makedepends}", &pkginfo.makedepends)
                    .replace("{sha256sums}", &pkginfo.sha256sums);
        }
        Err(e) => {
            return Err(e)
        }
    }

    Ok(pkgbuild)
}

// generate_srcinfo generates and returns the SRCINFO
fn generate_srcinfo(pkginfo: &Information) -> Result<String, std::io::Error> {
    let template = get_srcinfo();
    let srcinfo: String;

    match template {
        Ok(output) => {
            println!("Got SRCINFO template");
            srcinfo = output
                .replace("{pkgbase}", &pkginfo.pkgname)
                .replace("{pkgdesc}", &pkginfo.pkgdesc)
                .replace("{pkgrel}", &pkginfo.pkgrel)
                .replace("{url}", &pkginfo.url)
                .replace("{arch}", &pkginfo.arch)
                .replace("{license}", &pkginfo.license)
                .replace("{makedepends}", &pkginfo.makedepends)
                .replace("{source}", "SOURCE")
                .replace("{sha256sums}", "sha256sums")
                .replace("{pkgname}", "pkgname");
        }
        Err(e) => {
            return Err(e)
        }
    }

    Ok(srcinfo)
}


// get_pkgbuild retrieves and returns the contents of templates/PKGBUILD
fn get_pkgbuild() -> std::io::Result<String> {
    let mut file = File::open("templates/PKGBUILD")?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// get_srcinfo retrieves and returns the contents of templates/SRCINFO
fn get_srcinfo() -> std::io::Result<String> {
    let mut file = File::open("templates/SRCINFO")?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// input_string is a helper function to get string input from user efficiently
fn input_string(prompt: &str) -> String {
    let mut input = String::new();

    print!("{}", prompt);
    io::stdout().flush().unwrap();  // Flush the output correctly

    match io::stdin().read_line(&mut input) {
        Ok(_) => (),
        Err(e) => println!("Unable to take input: {}", e),
    }

    input.trim().to_string()
}

// save_pkgbuild is a helper function to save PKGBUILD to disk
fn save_pkgbuild(pkgbuild: &String) {
    // create_new because it creates new file in read-write mode; errror if the file exists
    // and making sure that possibly existing PKGBUILD does not get overwritten
    let file_result = File::create_new("PKGBUILD");
    
    match file_result {
        Ok(mut file) => {
            match file.write_all(pkgbuild.as_bytes()) {
                Ok(_) => println!("Generated PKGBUILD successfully."),
                Err(e) => println!("Failed to write to PKGBUILD: {}", e),
            }
        },
        Err(e) => println!("Failed to create new PKGBUILD: {}", e),
    }
}

// save_srcinfo is a helper function to save .SRCINFO to disk
fn save_srcinfo(srcinfo: &String) {
    // create_new because it creates new file in read-write mode; error if the file exists
    // and making sure that possibly existing SRCINFO does not get overwritten
    let file_result = File::create_new(".SRCINFO");

    match file_result {
        Ok(mut file) => {
            match file.write_all(srcinfo.as_bytes()) {
                Ok(_) => println!("Generated .SRCINFO successfully."),
                Err(e) => println!("Failed to write to .SRCINFO: {}", e),
            }
        },
        Err(e) => println!("Failed to create new .SRCINFO: {}", e),
    }
}
