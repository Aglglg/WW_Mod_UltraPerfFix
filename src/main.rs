use reqwest::blocking::get;
use rfd::FileDialog;
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions, copy};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

const BACKUP_EXTENSION: &str = "ini_bakcup_from_ultraperf_fixer_tool";
const HASH_URL: &str = "https://raw.githubusercontent.com/Aglglg/WW_Mod_UltraPerfFix/refs/heads/main/WW_QualityToUltraPerf_Hashpair.txt";
fn main() {
    // Open file dialog for folder selection
    println!(
        "Select your \"Mods\" folder(D:\\WWMI\\Mods), or single mod folder (D:\\WWMI\\Mods\\RoverMod) "
    );
    let root_folder: Option<PathBuf> = FileDialog::new().pick_folder();

    match root_folder {
        Some(folder) => {
            let ini_files = find_ini_files(&folder);

            match fetch_hash_map() {
                Ok(hash_map) => {
                    for file_path in ini_files {
                        let file_path_str = file_path.to_string_lossy();

                        match process_ini_file(&file_path_str, &hash_map) {
                            Ok(sections) => {
                                if sections.is_empty() {
                                    println!(
                                        "SKIPPING {} -- Nothing to be added.",
                                        file_path.display()
                                    );
                                } else if let Err(e) =
                                    append_to_ini_file(&file_path_str, &file_path, &sections)
                                {
                                    eprintln!("Error writing to file: {}", e);
                                } else {
                                    println!(
                                        "MODIFIED {} -- Should be FIXED\n",
                                        file_path.display()
                                    );
                                }
                            }
                            Err(e) => eprintln!("Error processing file: {}", e),
                        }
                    }
                }
                Err(e) => eprintln!("Error fetching hashes data from cloud: {}", e),
            }
        }
        None => {
            println!("No folder selected. Enter to exit...");
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
        }
    }

    println!("Press Enter to exit...");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

fn fetch_hash_map() -> io::Result<HashMap<String, String>> {
    println!("Fetching hash map from: {}", HASH_URL);

    let response =
        get(HASH_URL).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let content = response
        .text()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let mut hash_map = HashMap::new();

    println!("\nFetched Raw Data:\n{}", content); // Print raw fetched data

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue; // Ignore comments and empty lines
        }

        if let Some((old_hash, new_hash)) = trimmed.split_once(':') {
            hash_map.insert(old_hash.to_string(), new_hash.to_string());
        }
    }
    Ok(hash_map)
}

fn find_ini_files(root_folder: &Path) -> Vec<PathBuf> {
    let mut ini_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root_folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                ini_files.extend(find_ini_files(&path)); // Recursively search subdirectories
            } else if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.ends_with(".ini")
                    && !file_name.to_lowercase().starts_with("disabled")
                    && file_name.to_lowercase() != "desktop.ini"
                {
                    ini_files.push(path);
                }
            }
        }
    }
    ini_files
}

fn process_ini_file(
    file_path: &str,
    hash_map: &HashMap<String, String>,
) -> io::Result<Vec<String>> {
    let path = Path::new(file_path);

    let mut sections: Vec<String> = Vec::new();
    let mut current_section: Vec<String> = Vec::new();
    let mut in_texture_section = false;
    let mut section_header = String::new();
    let mut has_modified_hash = false;
    let mut hash_set: HashSet<String> = HashSet::new(); // Store all existing hash values

    // First, clean the file by removing any [Texture..._LOWQ] sections
    let cleaned_lines: Vec<String> = {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        let mut lines = Vec::new();
        let mut skip_section = false;

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                // Check if this is a [Texture..._LOWQ] section
                if trimmed.to_lowercase().starts_with("[texture")
                    && trimmed.to_lowercase().ends_with("_lowq]")
                {
                    skip_section = true;
                    continue; // Don't include this section header
                } else {
                    skip_section = false;
                }
            }

            if !skip_section {
                lines.push(line);
            }
        }

        lines
    };

    std::fs::write(file_path, cleaned_lines.join("\n") + "\n")?;

    // First pass: Collect existing hash values in the file
    for line in &cleaned_lines {
        let trimmed_line = line.trim();

        if trimmed_line.starts_with(';') {
            continue; // Ignore commented lines
        }

        if let Some(pos) = trimmed_line.find("hash") {
            let parts: Vec<&str> = trimmed_line[pos..].split('=').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                hash_set.insert(parts[1].to_string()); // Store all existing hashes
            }
        }
    }

    for line in &cleaned_lines {
        let trimmed_line = line.trim();

        if trimmed_line.starts_with(';') {
            continue; // Ignore commented lines
        }

        if trimmed_line.starts_with('[') && trimmed_line.ends_with(']') {
            // Store the previous section **ONLY IF IT HAD A MODIFIED HASH**
            if in_texture_section && has_modified_hash && !current_section.is_empty() {
                sections.push(current_section.join("\n"));
            }

            // Reset for new section
            in_texture_section = trimmed_line.to_lowercase().starts_with("[texture")
                && !trimmed_line.to_lowercase().ends_with("_lowq]");
            has_modified_hash = false;
            current_section.clear();
            section_header = trimmed_line.to_string();

            if in_texture_section {
                // Rename section by appending "_LOWQ"
                section_header = format!("{}_LOWQ", section_header.trim_end_matches(']'));
            }
            current_section.push(section_header.clone() + "]");
        } else if in_texture_section {
            // Detect "hash" lines in any format (normalize spaces)
            if let Some(pos) = trimmed_line.find("hash") {
                let mut parts: Vec<&str> =
                    trimmed_line[pos..].split('=').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    let hash_value = parts[1]; // Get the actual hash value
                    if let Some(replacement) = hash_map.get(hash_value) {
                        // **Only modify if the replacement hash is NOT already in the file**
                        if !hash_set.contains(replacement) {
                            let modified_line = format!("hash = {}", replacement);
                            current_section.push(modified_line);
                            has_modified_hash = true; // Mark that this section should be added
                            continue;
                        }
                    }
                }
            }
            current_section.push(line.clone());
        }
    }

    // Store the last section **ONLY IF IT HAD A MODIFIED HASH**
    if in_texture_section && has_modified_hash && !current_section.is_empty() {
        sections.push(current_section.join("\n"));
    }

    Ok(sections)
}

fn append_to_ini_file(
    file_path: &str,
    file_path_buf: &PathBuf,
    sections: &[String],
) -> io::Result<()> {
    backup_file(file_path_buf);

    let mut file = OpenOptions::new().append(true).open(file_path)?;

    // Add a newline before appending to ensure separation from existing content
    file.write_all(b"\n")?;
    for section in sections {
        file.write_all(section.as_bytes())?;
        file.write_all(b"\n\n")?; // Add space between sections
    }
    Ok(())
}

fn backup_file(path: &PathBuf) {
    if let Some(stem) = path.file_stem() {
        let mut backup_path = path.clone();
        backup_path.set_extension(BACKUP_EXTENSION);

        if !backup_path.exists() {
            copy(path, &backup_path).ok();
            println!("\nBackup created: {:?}", backup_path.display());
        } else {
            println!("\nBackup already exists: {:?}", backup_path.display());
        }
    }
}

//I admit that most of it from ChatGPT ;-;
