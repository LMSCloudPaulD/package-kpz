use chrono::Utc;
use clap::{App, Arg};
use regex::Regex;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::process::Command;
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

struct BuildParams {
    release_filename: String,
    pm_file_path_full_dist: String,
    translations_dir: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("package-kpz")
        .version("0.1.0")
        .author("Paul Derscheid <paul.derscheid@lmscloud.de>")
        .about("Package a Koha plugin for distribution")
        .arg(
            Arg::with_name("RELEASE_FILENAME")
                .short('r')
                .long("release-filename")
                .value_name("FILE")
                .help("Sets the release filename")
                .required(true),
        )
        .arg(
            Arg::with_name("PM_FILE_PATH")
                .short('p')
                .long("pm-file-path")
                .value_name("PATH")
                .help("Sets the full path for the PM file")
                .required(true),
        )
        .arg(
            Arg::with_name("TRANSLATIONS_DIR")
                .short('t')
                .long("translations-dir")
                .value_name("DIR")
                .help("Sets the translations directory")
                .required(false),
        )
        .get_matches();

    let package_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string("./package.json")?)?;

    let build_params = BuildParams {
        release_filename: matches.value_of("RELEASE_FILENAME").unwrap().to_string(),
        pm_file_path_full_dist: matches.value_of("PM_FILE_PATH").unwrap().to_string(),
        translations_dir: matches.value_of("TRANSLATIONS_DIR").map(|s| s.to_string()),
    };

    println!("{}", build_params.release_filename);
    println!("{}", build_params.pm_file_path_full_dist);

    build_directory()?;
    copy_files()?;
    convert_translations(&build_params)?;
    substitute_strings(&build_params, &package_json)?;
    create_zip(&build_params, &package_json)?;
    cleanup()?;
    Ok(())
}

fn build_directory() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir("dist").map_err(|err| format!("Failed to create dist directory: {}", err))?;
    Ok(())
}

fn copy_files() -> Result<(), Box<dyn std::error::Error>> {
    let src = Path::new("Koha");
    let dest = Path::new("dist");
    copy_dir_recursive(&src, &dest.join(src))
        .map_err(|err| format!("Failed to copy files: {}", err))?;
    Ok(())
}

fn substitute_strings(
    build_params: &BuildParams,
    package_json: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = BufReader::new(File::open(&build_params.pm_file_path_full_dist)?);
    let version_regex = Regex::new(r"\{VERSION\}")?;
    let date_regex = Regex::new(r"1900-01-01")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let contents = version_regex
        .replace_all(
            &contents,
            package_json["version"].as_str().unwrap_or_default(),
        )
        .to_string();
    let contents = date_regex
        .replace_all(
            &contents,
            Utc::now()
                .date_naive()
                .format("%Y-%m-%d")
                .to_string()
                .as_str(),
        )
        .to_string();
    let mut file = BufWriter::new(File::create(&build_params.pm_file_path_full_dist)?);
    file.write_all(contents.as_bytes())?;
    Ok(())
}

fn create_zip(
    build_params: &BuildParams,
    package_json: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let version = package_json["version"].as_str().unwrap_or_default();
    let release_filename = format!("{}-v{}.kpz", build_params.release_filename, version);
    let file = File::create(release_filename)?;
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755);
    let mut zip = ZipWriter::new(file);

    add_directory_to_zip("dist", "", &mut zip, &options)?;

    zip.finish()?;
    Ok(())
}

fn add_directory_to_zip<P: AsRef<Path>>(
    dir: P,
    prefix: &str,
    zip: &mut ZipWriter<File>,
    options: &FileOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = dir.as_ref();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let zip_path = if prefix.is_empty() {
            file_name.clone()
        } else {
            format!("{}/{}", prefix, file_name)
        };

        if path.is_dir() {
            zip.add_directory(zip_path.clone() + "/", *options)?;
            add_directory_to_zip(path, &zip_path, zip, options)?;
        } else {
            zip.start_file(zip_path, *options)?;
            let mut file = fs::File::open(path)?;
            io::copy(&mut file, zip)?;
        }
    }

    Ok(())
}

fn cleanup() -> Result<(), Box<dyn std::error::Error>> {
    fs::remove_dir_all("dist")
        .map_err(|err| format!("Failed to remove dist directory: {}", err))?;
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> std::io::Result<()> {
    if !src.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Source path is not a directory",
        ));
    }

    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dest.join(file_name);

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(path, dest_path)?;
        }
    }

    Ok(())
}

fn call_po2json(file: &str, options: &str) -> Result<String, std::io::Error> {
    let output = Command::new("npx")
        .arg("po2json")
        .arg(file)
        .arg("-f")
        .arg("mf")
        .arg(options)
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "po2json execution failed",
        ))
    }
}

fn convert_translations(build_params: &BuildParams) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(translations_dir) = &build_params.translations_dir {
        let src = Path::new(translations_dir);
        let dest = Path::new("dist/locale");

        for entry in fs::read_dir(&src)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("po") {
                let locale = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
                let dest_path = dest.join(format!("{}.json", locale));

                let options = "--pretty --fuzzy"; // Add more options as needed

                let data = call_po2json(&path.to_str().unwrap(), options)?;

                let mut json_data: serde_json::Value = serde_json::from_str(&data)?;
                json_data.as_object_mut().unwrap().insert(
                    "".to_string(),
                    serde_json::json!({
                        "language": locale,
                        "plural-forms": "nplurals=2; plural=n>1"
                    }),
                );

                let json = serde_json::to_string(&json_data)?;

                let mut json_file = BufWriter::new(File::create(dest_path)?);
                json_file.write_all(json.as_bytes())?;
            }
        }
    }
    Ok(())
}
