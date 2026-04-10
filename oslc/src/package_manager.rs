use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::io::{self, Write};

pub const ANSI_GREEN: &str = "\x1b[92m";
pub const ANSI_RED: &str = "\x1b[91m";
pub const ANSI_YELLOW: &str = "\x1b[93m";
pub const ANSI_CYAN: &str = "\x1b[96m";
pub const ANSI_BOLD: &str = "\x1b[1m";
pub const ANSI_RESET: &str = "\x1b[0m";

fn colorize(color: &str, text: &str) -> String {
    format!("{}{}{}", color, text, ANSI_RESET)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageSpec {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallResult {
    pub spec: String,
    pub download_url: String,
    pub install_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct InstalledPackage {
    pub name: String,
    pub path: PathBuf,
}

pub fn is_installed(spec: &str) -> bool {
    if let Ok(parsed) = parse_package_spec(spec) {
        if let Ok(install_dir) = package_install_dir(&parsed) {
            return install_dir.exists();
        }
    }
    false
}

pub fn confirm_install(spec: &str) -> bool {
    print!("{}Install{} {}? [y/N] ", ANSI_YELLOW, ANSI_RESET, spec);
    io::stdout().flush().ok();
    
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        input.trim().to_lowercase() == "y"
    } else {
        false
    }
}

pub fn install_from_registry(spec: &str) -> Result<InstallResult, String> {
    let parsed = parse_package_spec(spec)?;
    let download_url = package_download_url(&parsed);
    let install_dir = package_install_dir(&parsed)?;

    if install_dir.exists() {
        return Err(format!(
            "{} is already installed at {}\n  Use {}oslc uninstall {}{} to remove it first",
            spec,
            install_dir.display(),
            ANSI_CYAN, spec, ANSI_RESET
        ));
    }

    let temp_zip = temp_zip_path(&parsed);
    fs::create_dir_all(&install_dir)
        .map_err(|e| format!("Failed to create install dir {}: {}", install_dir.display(), e))?;

    download_zip(&download_url, &temp_zip)?;
    unzip_archive(&temp_zip, &install_dir)?;

    let _ = fs::remove_file(&temp_zip);

    Ok(InstallResult {
        spec: parsed.name.clone(),
        download_url,
        install_dir,
    })
}

pub fn parse_package_spec(spec: &str) -> Result<PackageSpec, String> {
    let spec = spec.trim();
    if spec.is_empty() {
        return Err("Package spec cannot be empty".to_string());
    }

    validate_package_name(spec)?;
    Ok(PackageSpec {
        name: spec.to_string(),
    })
}

pub fn package_download_url(spec: &PackageSpec) -> String {
    format!(
        "https://oslc.dev/api/packages/download?name={}",
        spec.name.replace('/', "%2F")
    )
}

pub fn package_install_dir(spec: &PackageSpec) -> Result<PathBuf, String> {
    let home = env::var_os("HOME").ok_or_else(|| "HOME is not set".to_string())?;
    let mut dir = PathBuf::from(home);
    dir.push(".oslc");
    dir.push("packages");
    for part in spec.name.split('/') {
        dir.push(part);
    }
    Ok(dir)
}

fn validate_package_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Package names cannot be empty".to_string());
    }

    let parts: Vec<&str> = name.split('/').collect();
    if parts.is_empty() || parts.len() > 2 {
        return Err("Package names must be `name` or `namespace/name`".to_string());
    }

    for part in parts {
        let valid = !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && !part.starts_with('-')
            && !part.ends_with('-');

        if !valid {
            return Err(format!(
                "Invalid package name `{}`: use lowercase letters, numbers, hyphens, and at most one slash",
                name
            ));
        }
    }

    Ok(name.to_string())
}

fn temp_zip_path(spec: &PackageSpec) -> PathBuf {
    let mut path = env::temp_dir();
    let safe_name = spec.name.replace('/', "_");
    path.push(format!("oslc-{}.zip", safe_name));
    path
}

fn download_zip(url: &str, output: &Path) -> Result<(), String> {
    let status = Command::new("curl")
        .args(["-fsSL", url, "-o"])
        .arg(output)
        .status()
        .map_err(|e| format!("Failed to start curl: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Failed to download package from {}", url))
    }
}

pub fn uninstall_package(spec: &str) -> Result<(), String> {
    let parsed = parse_package_spec(spec)?;
    let install_dir = package_install_dir(&parsed)?;

    if !install_dir.exists() {
        return Err(format!("{} is not installed", spec));
    }

    fs::remove_dir_all(&install_dir)
        .map_err(|e| format!("Failed to remove {}: {}", install_dir.display(), e))?;

    println!("{}Uninstalled{} {}", ANSI_RED, ANSI_RESET, spec);
    Ok(())
}

pub fn list_installed_packages() -> Result<Vec<InstalledPackage>, String> {
    let packages_base = packages_base_dir()?;
    let mut packages = Vec::new();

    if !packages_base.exists() {
        return Ok(packages);
    }

    for entry in fs::read_dir(&packages_base)
        .map_err(|e| format!("Failed to read packages dir: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            let namespace = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            
            for pkg_entry in fs::read_dir(&path)
                .map_err(|e| format!("Failed to read namespace dir: {}", e))?
            {
                let pkg_entry = pkg_entry.map_err(|e| format!("Failed to read pkg entry: {}", e))?;
                if pkg_entry.path().is_dir() {
                    let pkg_name = pkg_entry.file_name()
                        .to_str()
                        .unwrap_or("")
                        .to_string();
                    packages.push(InstalledPackage {
                        name: format!("{}/{}", namespace, pkg_name),
                        path: pkg_entry.path(),
                    });
                }
            }
        }
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(packages)
}

pub fn search_packages(query: &str) -> Result<Vec<String>, String> {
    let url = format!("https://oslc.dev/api/packages/search?q={}", 
        query.replace(' ', "%20"));
    
    let output = Command::new("curl")
        .args(["-fsSL", &url])
        .output()
        .map_err(|e| format!("Failed to search: {}", e))?;

    if !output.status.success() {
        return Err("Search failed".to_string());
    }

    let body = String::from_utf8_lossy(&output.stdout);
    let packages: Vec<String> = body
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();

    Ok(packages)
}

fn packages_base_dir() -> Result<PathBuf, String> {
    let home = env::var_os("HOME").ok_or_else(|| "HOME is not set".to_string())?;
    let mut dir = PathBuf::from(home);
    dir.push(".oslc");
    dir.push("packages");
    Ok(dir)
}

pub fn get_package_include_path(spec: &str) -> Option<PathBuf> {
    let parsed = parse_package_spec(spec).ok()?;
    package_install_dir(&parsed).ok()
}

fn unzip_archive(zip_path: &Path, install_dir: &Path) -> Result<(), String> {
    let status = Command::new("unzip")
        .args(["-oq"])
        .arg(zip_path)
        .args(["-d"])
        .arg(install_dir)
        .status()
        .map_err(|e| format!("Failed to start unzip: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Failed to unpack {} into {}",
            zip_path.display(),
            install_dir.display()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_package_name() {
        let spec = parse_package_spec("help-test").unwrap();
        assert_eq!(spec.name, "help-test");
    }

    #[test]
    fn parses_namespaced_package_name() {
        let spec = parse_package_spec("help/example").unwrap();
        assert_eq!(spec.name, "help/example");
    }

    #[test]
    fn rejects_uppercase_package_names() {
        assert!(parse_package_spec("Help/example").is_err());
    }

    #[test]
    fn builds_registry_download_url() {
        let spec = parse_package_spec("help/example").unwrap();
        assert_eq!(
            package_download_url(&spec),
            "https://oslc.dev/api/packages/download?name=help%2Fexample"
        );
    }

    #[test]
    fn builds_local_install_dir() {
        let spec = parse_package_spec("help/example").unwrap();
        let dir = package_install_dir(&spec).unwrap();
        let dir_string = dir.to_string_lossy();
        assert!(dir_string.contains(".oslc/packages/help/example"));
    }
}
