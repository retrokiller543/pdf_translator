// create a module for reading the text of the pdf file and also checking if poppler is installed
mod pdf_reader {
    use std::process::Command;
    use std::io::{Error, Read};
    use crate::install;

    pub struct PdfReader {
        content: Vec<(usize, String)>,
    }

    impl PdfReader {
        pub fn new(path: &str) -> Result<PdfReader, Error> {
            let _ = install::run();
            PdfReader::read_pdf(path)?;

            let file_path = path.replace(".pdf", ".txt");
            let content = PdfReader::read_file_with_formatting(&file_path)?;

            Ok(PdfReader {
                content,
            })
        }

        fn read_file_with_formatting(file_path: &str) -> Result<Vec<(usize, String)>, std::io::Error> {
            let mut file = std::fs::File::open(file_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
        
            let lines_with_numbers: Vec<(usize, String)> = contents.lines()
                .enumerate()
                .map(|(idx, line)| (idx, line.to_string()))
                .collect();
        
            Ok(lines_with_numbers)
        }

        pub fn get_content(&self) -> Vec<(usize, String)> {
            self.content.clone()
        }

        fn read_pdf(path: &str) -> Result<String, Error> {
            let output = Command::new("pdftotext")
                .arg(path)
                .arg("-layout")
                .output()?;
    
            let text = String::from_utf8(output.stdout).expect("Not UTF-8");
    
            Ok(text)
        }
    }

    /*
    mod tests {
        #[allow(unused_imports)]
        use super::*;

        #[test]
        fn test_read_basic_pdf() {
            let pdf_reader = PdfReader::new("./test-files/example.pdf").expect("Error reading pdf");
            let content = pdf_reader.get_content();
            let correct_content: Vec<(usize, String)> = vec![(0, "Hello World!".to_string()), (1, "\u{c}".to_string())];

            // compare correct content with the content from the pdf
            assert_eq!(content, correct_content);
        }
    }
     */
}


mod translator {
    
    use serde::Serialize;
    use std::collections::HashMap;

    use crate::config;

    const GOOGLE_TRANSLATE_API_ENDPOINT: &str = "https://translation.googleapis.com/language/translate/v2";

    #[derive(Serialize)]
    struct TranslateRequest {
        q: String,
        source: String,
        target: String,
        format: String,
        key: String,
    }

    pub async fn translate_text(formatted_content: Vec<(usize, String)>) -> Result<Vec<(usize, String)>, reqwest::Error> {
        let config: config::Config = config::Config::load().expect("Failed to load configuration");
        let client = reqwest::Client::new();
        let mut translated_texts = Vec::new();
    
        for (line_number, line) in formatted_content {
            let mut payload = HashMap::new();
            payload.insert("q", line.clone());
            payload.insert("source", "en".to_string());
            payload.insert("target", "sv".to_string());
            payload.insert("format", "text".to_string());
            payload.insert("key", config.get_api_key());
    
            let access_token = "Bearer ".to_string() + config.get_access_token().as_str();
    
            let response: serde_json::Value = client
                .post(GOOGLE_TRANSLATE_API_ENDPOINT)
                .header("Authorization", access_token)
                .header("x-goog-user-project", config.get_project_id())
                .header("Content-Type", "application/json; charset=utf-8")
                .json(&payload)
                .send()
                .await?
                .json()
                .await?;
    
            let translated_line = parse_response(&response.to_string()).expect("Error parsing response");
            translated_texts.push((line_number, translated_line));
        }
    
        Ok(translated_texts)
    }
    

    fn parse_response(response: &str) -> Result<String, serde_json::Error> {
        let v: serde_json::Value = serde_json::from_str(response)?;
        #[cfg(debug_assertions)]
        {
            // if status is not 200, then print the response
            if !v["error"]["code"].is_null() {
                dbg!(v.clone());
            }
        }
        let translated_text = v["data"]["translations"][0]["translatedText"].as_str().unwrap_or_default().to_string();
        Ok(translated_text)
    }
}

mod config {
    use std::fs;
    use directories::ProjectDirs;
    use serde::{Deserialize, Serialize};
    

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Config {
        api_key: String,
        project_id: String,
        access_token: String,
    }

    impl Config {
        pub fn new(api_key: String, project_id: String, access_token: String) -> Config {
            Config {
                api_key,
                project_id,
                access_token,
            }
        }
    
        /// Loads the configuration from the default config file.
        pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
            let config_path = Self::get_config_path()?;
            #[cfg(debug_assertions)]
            {
                dbg!(config_path.clone());
            }
            let config_str = fs::read_to_string(config_path)?;
            let config: Config = toml::from_str(&config_str)?;
            Ok(config)
        }
    
        /// Saves the current configuration to the default config file.
        pub fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            let config_path = Self::get_config_path()?;
            let prev_conf = Self::load();
            let mut prev_key: String = "".to_string();
            let mut prev_project_id: String = "".to_string();
            let mut prev_access_token: String = "".to_string();
            if prev_conf.is_ok() {
                let prev_conf = prev_conf.unwrap();
                prev_key = prev_conf.api_key;
                prev_project_id = prev_conf.project_id;
                prev_access_token = prev_conf.access_token;
            }
            #[cfg(debug_assertions)]
            {
                dbg!(config_path.clone());
            }

            if self.api_key.is_empty() && !prev_key.is_empty() {
                self.api_key = prev_key;
            }

            if self.project_id.is_empty() && !prev_project_id.is_empty() {
                self.project_id = prev_project_id;
            }

            if self.access_token.is_empty() && !prev_access_token.is_empty() {
                self.access_token = prev_access_token;
            }

            let config_str = toml::to_string(self)?;
            fs::write(config_path, config_str)?;
            Ok(())
        }

        pub fn get_api_key(&self) -> String {
            self.api_key.clone()
        }

        pub fn get_project_id(&self) -> String {
            self.project_id.clone()
        }

        pub fn get_access_token(&self) -> String {
            self.access_token.clone()
        }
    
        /// Determines the path for the configuration file using the `directories` crate.
        fn get_config_path() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
            let proj_dirs = ProjectDirs::from("com", "pdf_translator_company", "PDF Translator")
                .ok_or("Failed to get project directories")?;
            let config_dir = proj_dirs.config_dir();
            if !config_dir.exists() {
                fs::create_dir_all(config_dir)?;
            }
            Ok(config_dir.join("config.toml"))
        }

    }
    
    pub fn setup(mut args: Config) {    
        if args.api_key.is_empty() && args.project_id.is_empty() && args.access_token.is_empty() {
            println!("You must at least provide one of the following arguments '--api_key <API_KEY>', '--access_token <ACCESS_TOKEN>', '--project_id <PROJECT_ID>' ");
            return;
        }
    
        args.save().expect("Failed to save configuration");
        println!("Configuration saved successfully!");
    }
    
}

/// The `install` module which provides functions to check if `poppler-utils` is installed and install it if it is not.
mod install {
    use std::process::Command;
    #[cfg(target_os = "linux")]
    use rpassword::read_password;
    #[cfg(target_os = "macos")]
    use rpassword::read_password;

    /// This function checks if `poppler-utils` is installed and installs it if it is not.
    pub fn run() -> Result<(), String> {
        println!("Checking if poppler-utils is installed...");
        let result = check_poppler();
        if result.is_ok() {
            println!("Poppler is already installed!");
            Ok(())
        } else {
            Err(result.err().unwrap())
        }
    }

    #[cfg(target_os = "linux")]
    fn install() -> Result<(), String> {
        let installed_manager = get_package_manager();
        #[cfg(debug_assertions)]
        {
            dbg!(installed_manager.clone());
        }

        if installed_manager.is_empty() {
            return Err("No package manager is installed".to_string());
        }

        // Prompt user for password
        print!("Please enter your sudo password: ");
        let password = read_password().expect("Failed to read password");
        
        let error_msg = "Error installing using package manager '".to_owned() + installed_manager.as_str() + "'";

        // Pipe the password to sudo
        Command::new("sh")
            .arg("-c")
            .arg(format!("echo {} | sudo -S {} install -y poppler-utils", password.trim(), installed_manager))
            .spawn()
            .unwrap_or_else(|_| { panic!("{}", error_msg) });
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn get_package_manager() -> String {
        let package_managers = vec!["apt", "yum", "pacman"];

        for manager in package_managers {
            let output = Command::new("which")
                .arg(manager)
                .output()
                .expect("Error: 'which' command not found!");

            // If the command succeeded, then the package manager exists on the system
            if output.status.success() {
                return manager.to_string();
            }
        }

        "".to_string() // Return an empty string if no package manager found
    }

    #[cfg(target_os = "macos")]
    fn install() -> Result<(), String> {
        // Prompt user for password
        print!("Please enter your sudo password: ");
        let password = read_password().expect("Failed to read password");
        let error_msg = "Error installing using package manager 'brew'";
        let poppler_install_cmd = "brew install poppler";
        if !check_brew(){
            let brew_install = format!("/bin/bash -c {}", "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)");
            Command::new("sh")
                .arg("-c")
                .arg(format!("echo {} | sudo -S {}", password.trim(), brew_install))
                .spawn()
                .expect(error_msg);
            }

        Command::new("sh")
            .arg("-c")
            .arg(format!("echo {} | sudo -S {}", password.trim(), poppler_install_cmd))
            .spawn()
            .expect(error_msg);
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn check_brew() -> bool {
        let output = Command::new("which")
            .arg("brew")
            .output()
            .expect("Error: 'which' command not found!");

        #[cfg(debug_assertions)]
        {
            dbg!(output.status.clone());
        }

        // If the command succeeded, then brew exists on the system
        if output.status.success() {
            return true;
        }

        false
    }

    #[cfg(target_os = "windows")]
    fn install() -> Result<(), String> {
        let error_msg_choco = "Error installing chocolaty";
        let error_msg_poppler = "Error installing using package manager 'choco'";
        
        // Check if Chocolaty is installed, if not then install it
        if !check_chocolaty() {
            Command::new("powershell")
                .arg("-Command")
                .arg("Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))")
                .spawn()
                .expect(error_msg_choco);
        }
        
        // Install poppler-utils using Chocolaty
        Command::new("choco")
            .arg("install")
            .arg("poppler")
            .spawn()
            .expect(error_msg_poppler);
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn check_chocolaty() -> bool {
        let output = Command::new("where")
            .arg("choco")
            .output()
            .expect("Error: 'where' command not found!");

        // If the command succeeded, then Chocolaty exists on the system
        if output.status.success() {
            return true;
        }

        false
    }

    fn check_poppler() -> Result<(), String> {
        let output_result = Command::new("pdftotext")
            .arg("-v")
            .output();
    
        match output_result {
            Ok(output) => {
                let text = String::from_utf8(output.stderr).unwrap_or_else(|_| String::from(""));
    
                if text.contains("Poppler") {
                    Ok(())
                } else {
                    Err(String::from("Error occured while checking if poppler is installed."))
                }
            },
            Err(_) => {
                println!("Poppler is not installed.");
                let result = install();
                if result.is_ok() {
                    println!("Poppler installed successfully!");
                    Ok(())
                } else {
                    Err(result.err().unwrap_or_else(|| String::from("Error installing Poppler")))
                }
            }
        }
    }

    mod tests {
        #[allow(unused_imports)]
        use super::*;

        /*
        #[test]
        fn test_check_poppler() {
            let result = check_poppler();
            assert!(result.is_err());
        }
        */
        #[test]
        fn test_check_package_manager() {
            #[cfg(target_os = "linux")]
            {
                let result = get_package_manager();
                assert!(!result.is_empty());
            }
            #[cfg(target_os = "macos")]
            {
                let result = check_brew();
                assert!(result == true);
            }
            #[cfg(target_os = "windows")]
            {
                let result = check_chocolaty();
                assert!(result);
            }
        }
    }
}


mod program {
    use std::fs::File;
    use std::io::Write;
    use crate::pdf_reader;
    use crate::translator;

    
    pub async fn run(file_path: String) {
        let pdf_reader = pdf_reader::PdfReader::new(file_path.as_str()).expect("Error reading pdf");
    
        match translator::translate_text(pdf_reader.get_content()).await {
            Ok(translated_content) => {
                let mut file = File::create("translated_text.txt").expect("Error creating file");
                for (line_number, line) in translated_content {
                    writeln!(file, "{}: {}", line_number, line).expect("Error writing to file");
                }
                println!("Translation complete");
            },
            Err(e) => println!("Error translating: {}", e),
        }
    }
}

use clap::Parser;
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, name = "pdf-translator")]
struct Args {
    #[arg(short, long, long_help = "The path to the pdf file you want to translate")]
    path: Option<String>,
    #[arg(short, long, default_value = "false", long_help = "Install poppler on your system, requires sudo or root access\nCurrently only works on Linux and MacOS")]
    install: bool,
    #[arg(short, long, default_value = "false", long_help = "Setup the configuration file,\nneeds atleast one of these:\n\t'--api-key'\n\t'--access-token'\n\t'--project-id'")]
    config: bool,
    #[arg(long, default_value = "", long_help = "The API key for the Google Cloud Platform")]
    api_key: String,
    #[arg(long, default_value = "", long_help = "The project ID for the Google Cloud Platform")]
    access_token: String,
    #[arg(long, default_value = "", long_help = "The access token for the Google Cloud Platform")]
    project_id: String,
    #[cfg(debug_assertions)]
    #[arg(short, long, default_value = "false", long_help = "Run the program in debug mode,\nneeds a path to a pdf file called 'example.pdf' in the 'test-files' folder")]
    debug: bool,
}


#[tokio::main]
async fn main() {
    let args = Args::parse();

    #[cfg(debug_assertions)]
    {
        #[cfg(target_os = "linux")]
        let target_os = "linux";
        #[cfg(target_os = "macos")]
        let target_os = "macos";
        #[cfg(target_os = "windows")]
        let target_os = "windows";
        dbg!("{:?}", args.clone());
        dbg!(target_os);
        if args.debug {
            program::run("./test-files/example.pdf".to_string()).await;
        } else if args.install {
            #[cfg(target_os = "linux")]
            {
                let result = install::run();
                if result.is_ok() {
                    println!("Poppler installed successfully!");
                } else {
                    println!("Error installing poppler: {}", result.err().unwrap());
                }
            }
            #[cfg(target_os = "macos")]
            {
                let result = install::run();
                if result.is_ok() {
                    println!("Poppler installed successfully!");
                } else {
                    println!("Error installing poppler: {}", result.err().unwrap());
                }
            }
            #[cfg(target_os = "windows")]
            {
                println!("The installer for poppler is currently broken on Windows.\nPlease install poppler manually, or use a Linux or MacOS machine.")
            }
        } else if args.config {
            let config = config::Config::new(args.api_key, args.project_id, args.access_token);
            config::setup(config);
        } else {
            let path = args.path.unwrap();
            program::run(path).await;
        }
    }
    #[cfg(not(debug_assertions))]
    {
        if args.install {
            #[cfg(target_os = "linux")]
            {
                let result = install::run();
                if result.is_ok() {
                    println!("Poppler installed successfully!");
                } else {
                    println!("Error installing poppler: {}", result.err().unwrap());
                }
            }
            #[cfg(target_os = "macos")]
            {
                let result = install::run();
                if result.is_ok() {
                    println!("Poppler installed successfully!");
                } else {
                    println!("Error installing poppler: {}", result.err().unwrap());
                }
            }
            #[cfg(target_os = "windows")]
            {
                println!("The installer for poppler is currently broken on Windows.\nPlease install poppler manually, or use a Linux or MacOS machine.")
            }
        } else if args.config {
            let mut config = config::Config::new(args.api_key, args.project_id, args.access_token);
            config::setup(config);
        } else {
            let path = args.path.unwrap();
            program::run(path).await;
        }
    }
}