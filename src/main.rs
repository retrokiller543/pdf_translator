// create a module for reading the text of the pdf file and also checking if poppler is installed
mod pdf_reader {
    use crate::install;
    use std::io::{Error, Read};
    use std::process::Command;

    pub struct PdfReader {
        content: Vec<(usize, String)>,
    }

    impl PdfReader {
        pub fn new(path: &str) -> Result<PdfReader, Error> {
            let _ = install::run();
            PdfReader::read_pdf(path)?;

            let file_path = path.replace(".pdf", ".txt");
            let content = PdfReader::read_file_with_formatting(&file_path)?;

            Ok(PdfReader { content })
        }

        fn read_file_with_formatting(
            file_path: &str,
        ) -> Result<Vec<(usize, String)>, std::io::Error> {
            let mut file = std::fs::File::open(file_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            let lines_with_numbers: Vec<(usize, String)> = contents
                .lines()
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

    mod tests {
        #[allow(unused_imports)]
        use super::*;

        #[test]
        fn test_read_basic_pdf() {
            let path = format!("{}/test-files/example.pdf", env!("CARGO_MANIFEST_DIR"));
            let pdf_reader = PdfReader::new(&path).expect("Error reading pdf");
            let content = pdf_reader.get_content();
            let correct_content: Vec<(usize, String)> =
                vec![(0, "Hello World!".to_string()), (1, "\u{c}".to_string())];

            // compare correct content with the content from the pdf
            assert_eq!(content, correct_content);
        }
    }
}

mod translator {

    use serde::Serialize;
    use std::collections::HashMap;

    use crate::config;

    const GOOGLE_TRANSLATE_API_ENDPOINT: &str =
        "https://translation.googleapis.com/language/translate/v2";

    #[derive(Serialize)]
    struct TranslateRequest {
        q: String,
        source: String,
        target: String,
        format: String,
        key: String,
    }

    #[derive(Debug, Clone)]
    pub struct TranslateInput {
        pub formatted_content: Vec<(usize, String)>,
        pub source: String,
        pub target: String,
    }

    pub async fn translate_text(
        input: TranslateInput,
    ) -> Result<Vec<(usize, String)>, reqwest::Error> {
        let config: config::Config = config::Config::load().expect("Failed to load configuration");
        let client = reqwest::Client::new();
        let mut translated_texts = Vec::new();

        for (line_number, line) in input.formatted_content {
            let mut payload = HashMap::new();
            payload.insert("q", line.clone());
            payload.insert("source", input.source.clone());
            payload.insert("target", input.target.clone());
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

            let translated_line =
                parse_response(&response.to_string()).expect("Error parsing response");
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
        let translated_text = v["data"]["translations"][0]["translatedText"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        Ok(translated_text)
    }
}

mod config {
    use directories::ProjectDirs;
    use serde::{Deserialize, Serialize};
    use std::fs;

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
            if let Ok(conf) = prev_conf {
                prev_key = conf.api_key;
                prev_project_id = conf.project_id;
                prev_access_token = conf.access_token;
            }
            #[cfg(debug_assertions)]
            {
                dbg!(config_path.clone());
            }

            if self.api_key.is_empty() && !prev_key.is_empty() {
                self.api_key = prev_key;
                #[cfg(debug_assertions)]
                {
                    println!("Updating api_key to match old config");
                }
            }

            if self.project_id.is_empty() && !prev_project_id.is_empty() {
                self.project_id = prev_project_id;
                #[cfg(debug_assertions)]
                {
                    println!("Updating project_id to match old config");
                }
            }

            if self.access_token.is_empty() && !prev_access_token.is_empty() {
                self.access_token = prev_access_token;
                #[cfg(debug_assertions)]
                {
                    println!("Updating access_token to match old config");
                }
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

    mod tests {
        #[allow(unused_imports)]
        use super::*;
        #[allow(unused_imports)]
        use std::path::Path;

        #[test]
        fn test_save_config() {
            // Backup current config (if exists)
            let backup_path = format!("{}/config_backup.toml", env!("CARGO_MANIFEST_DIR"));
            if let Ok(current_config) = Config::load() {
                fs::write(&backup_path, toml::to_string(&current_config).unwrap()).unwrap();
            }

            // Test saving a dummy config
            let mut dummy_config = Config::new(
                "dummy_key".to_string(),
                "dummy_project".to_string(),
                "dummy_token".to_string(),
            );
            let save_result = dummy_config.save();
            assert!(save_result.is_ok());

            // Restore the backed up config
            if Path::new(&backup_path).exists() {
                fs::copy(
                    backup_path.clone(),
                    format!("{}/config.toml", env!("CARGO_MANIFEST_DIR")),
                )
                .unwrap();
                fs::remove_file(backup_path).unwrap();
            }
        }
    }
}

/// The `install` module which provides functions to check if `poppler-utils` is installed and install it if it is not.
mod install {
    #[cfg(target_os = "linux")]
    use rpassword::read_password;
    #[cfg(target_os = "macos")]
    use rpassword::read_password;
    use std::process::Command;

    /// This function checks if `poppler-utils` is installed and installs it if it is not.
    pub fn run() -> Result<(), String> {
        println!("Checking if poppler-utils is installed...");
        let result = check_poppler();
        if result.is_ok() {
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

        let error_msg = "Error installing using package manager '".to_owned()
            + installed_manager.as_str()
            + "'";

        // Pipe the password to sudo
        Command::new("sh")
            .arg("-c")
            .arg(format!(
                "echo {} | sudo -S {} install -y poppler-utils",
                password.trim(),
                installed_manager
            ))
            .spawn()
            .unwrap_or_else(|_| panic!("{}", error_msg));
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
        if !check_brew() {
            let brew_install = format!(
                "/bin/bash -c {}",
                "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
            );
            Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "echo {} | sudo -S {}",
                    password.trim(),
                    brew_install
                ))
                .spawn()
                .expect(error_msg);
        }

        Command::new("sh")
            .arg("-c")
            .arg(format!(
                "echo {} | sudo -S {}",
                password.trim(),
                poppler_install_cmd
            ))
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
        let output_result = Command::new("pdftotext").arg("-v").output();

        match output_result {
            Ok(output) => {
                let text = String::from_utf8(output.stderr).unwrap_or_else(|_| String::from(""));

                if text.contains("Poppler") {
                    Ok(())
                } else {
                    Err(String::from(
                        "Error occured while checking if poppler is installed.",
                    ))
                }
            }
            Err(_) => {
                println!("Poppler is not installed.");
                let result = install();
                if result.is_ok() {
                    println!("Poppler installed successfully!");
                    Ok(())
                } else {
                    Err(result
                        .err()
                        .unwrap_or_else(|| String::from("Error installing Poppler")))
                }
            }
        }
    }

    mod tests {
        #[allow(unused_imports)]
        use super::*;

        #[cfg(target_os = "linux")]
        #[test]
        fn test_linux_package_manager_check() {
            let result = get_package_manager();
            assert!(!result.is_empty());
        }

        #[cfg(target_os = "macos")]
        #[test]
        fn test_macos_brew_check() {
            let result = check_brew();
            assert!(result);
        }

        #[cfg(target_os = "windows")]
        #[test]
        fn test_windows_chocolaty_check() {
            let result = check_chocolaty();
            assert!(result);
        }
    }
}

mod program {
    use crate::pdf_reader;
    use crate::translator;
    use std::fs::File;
    use std::io::Write;

    pub struct Args {
        pub file_path: String,
        pub source: String,
        pub target: String,
    }

    pub async fn run(mut args: Args) {
        let pdf_reader =
            pdf_reader::PdfReader::new(args.file_path.as_str()).expect("Error reading pdf");

        if args.source.is_empty() {
            println!("No source language provided, defaulting to 'en'");
            args.source = "en".to_string();
        }

        if args.target.is_empty() {
            println!("No target language provided, defaulting to 'sv'");
            args.target = "sv".to_string();
        }

        let request = translator::TranslateInput {
            formatted_content: pdf_reader.get_content(),
            source: args.source,
            target: args.target,
        };

        match translator::translate_text(request).await {
            Ok(translated_content) => {
                let mut file = File::create("translated_text.txt").expect("Error creating file");
                for (line_number, line) in translated_content {
                    writeln!(file, "{}: {}", line_number, line).expect("Error writing to file");
                }
                println!("Translation complete");
            }
            Err(e) => println!("Error translating: {}", e),
        }
    }
}

use clap::Parser;

static SUPPORTED_LANGUAGES: &[(&str, &str)] = &[
    ("Afrikaans", "af"),
    ("Albanian", "sq"),
    ("Amharic", "am"),
    ("Arabic", "ar"),
    ("Armenian", "hy"),
    ("Assamese", "as"),
    ("Aymara", "ay"),
    ("Azerbaijani", "az"),
    ("Bambara", "bm"),
    ("Basque", "eu"),
    ("Belarusian", "be"),
    ("Bengali", "bn"),
    ("Bhojpuri", "bho"),
    ("Bosnian", "bs"),
    ("Bulgarian", "bg"),
    ("Catalan", "ca"),
    ("Cebuano", "ceb"),
    ("Chinese (Simplified)", "zh-CN or zh"),
    ("Chinese (Traditional)", "zh-TW"),
    ("Corsican", "co"),
    ("Czech", "cs"),
    ("Danish", "da"),
    ("Dhivehi", "dv"),
    ("Dogri", "doi"),
    ("Dutch", "nl"),
    ("English", "en"),
    ("Esperanto", "eo"),
    ("Estonian", "et"),
    ("Ewe", "ee"),
    ("Filipino (Tagalog)", "fil"),
    ("Finnish", "fi"),
    ("French", "fr"),
    ("Frisian", "fy"),
    ("Galician", "gl"),
    ("Georgian", "ka"),
    ("German", "de"),
    ("Greek", "el"),
    ("Guarani", "gn"),
    ("Gujarati", "gu"),
    ("Haitian Creole", "ht"),
    ("Hausa", "ha"),
    ("Hawaiian", "haw"),
    ("Hebrew", "he or iw"),
    ("Hindi", "hi"),
    ("Hmong", "hmn"),
    ("Hungarian", "hu"),
    ("Hebrew", "he or iw"),
    ("Hindi", "hi"),
    ("Hmong", "hmn"),
    ("Hungarian", "hu"),
    ("Icelandic", "is"),
    ("Igbo", "ig"),
    ("Ilocano", "ilo"),
    ("Indonesian", "id"),
    ("Irish", "ga"),
    ("Italian", "it"),
    ("Japanese", "ja"),
    ("Javanese", "jv or jw"),
    ("Kannada", "kn"),
    ("Kazakh", "kk"),
    ("Khmer", "km"),
    ("Kinyarwanda", "rw"),
    ("Konkani", "gom"),
    ("Korean", "ko"),
    ("Krio", "kri"),
    ("Kurdish", "ku"),
    ("Kurdish (Sorani)", "ckb"),
    ("Kyrgyz", "ky"),
    ("Lao", "lo"),
    ("Latin", "la"),
    ("Latvian", "lv"),
    ("Lingala", "ln"),
    ("Lithuanian", "lt"),
    ("Luganda", "lg"),
    ("Luxembourgish", "lb"),
    ("Macedonian", "mk"),
    ("Maithili", "mai"),
    ("Malagasy", "mg"),
    ("Malay", "ms"),
    ("Malayalam", "ml"),
    ("Maltese", "mt"),
    ("Maori", "mi"),
    ("Marathi", "mr"),
    ("Meiteilon (Manipuri)", "mni-Mtei"),
    ("Mizo", "lus"),
    ("Mongolian", "mn"),
    ("Myanmar (Burmese)", "my"),
    ("Nepali", "ne"),
    ("Norwegian", "no"),
    ("Nyanja (Chichewa)", "ny"),
    ("Odia (Oriya)", "or"),
    ("Oromo", "om"),
    ("Pashto", "ps"),
    ("Persian", "fa"),
    ("Polish", "pl"),
    ("Portuguese (Portugal, Brazil)", "pt"),
    ("Punjabi", "pa"),
    ("Quechua", "qu"),
    ("Romanian", "ro"),
    ("Russian", "ru"),
    ("Samoan", "sm"),
    ("Sanskrit", "sa"),
    ("Scots Gaelic", "gd"),
    ("Sepedi", "nso"),
    ("Serbian", "sr"),
    ("Sesotho", "st"),
    ("Shona", "sn"),
    ("Sindhi", "sd"),
    ("Sinhala (Sinhalese)", "si"),
    ("Slovak", "sk"),
    ("Slovenian", "sl"),
    ("Somali", "so"),
    ("Spanish", "es"),
    ("Sundanese", "su"),
    ("Swahili", "sw"),
    ("Swedish", "sv"),
    ("Tagalog (Filipino)", "tl"),
    ("Tajik", "tg"),
    ("Tamil", "ta"),
    ("Tatar", "tt"),
    ("Telugu", "te"),
    ("Thai", "th"),
    ("Tigrinya", "ti"),
    ("Tsonga", "ts"),
    ("Turkish", "tr"),
    ("Turkmen", "tk"),
    ("Twi (Akan)", "ak"),
    ("Ukrainian", "uk"),
    ("Urdu", "ur"),
    ("Uyghur", "ug"),
    ("Uzbek", "uz"),
    ("Vietnamese", "vi"),
    ("Welsh", "cy"),
    ("Xhosa", "xh"),
    ("Yiddish", "yi"),
    ("Yoruba", "yo"),
    ("Zulu", "zu"),
];

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, name = "pdf-translator")]
struct Args {
    #[arg(
        short,
        long,
        long_help = "The path to the pdf file you want to translate"
    )]
    path: Option<String>,
    #[arg(
        short,
        long,
        default_value = "en",
        long_help = "The source language of the pdf file"
    )]
    source: String,
    #[arg(
        short,
        long,
        default_value = "sv",
        long_help = "The target language of the output text"
    )]
    target: String,
    #[arg(
        long,
        default_value = "false",
        long_help = "Prints the list of supported languages"
    )]
    list: bool,
    #[arg(
        short,
        long,
        default_value = "false",
        long_help = "Install poppler on your system, requires sudo or root access\nCurrently only works on Linux and MacOS"
    )]
    install: bool,
    #[arg(
        short,
        long,
        default_value = "false",
        long_help = "Setup the configuration file,\nneeds atleast one of these:\n\t'--api-key'\n\t'--access-token'\n\t'--project-id'"
    )]
    config: bool,
    #[arg(
        long,
        default_value = "",
        long_help = "The API key for the Google Cloud Platform"
    )]
    api_key: String,
    #[arg(
        long,
        default_value = "",
        long_help = "The project ID for the Google Cloud Platform"
    )]
    access_token: String,
    #[arg(
        long,
        default_value = "",
        long_help = "The access token for the Google Cloud Platform"
    )]
    project_id: String,
    #[cfg(debug_assertions)]
    #[arg(
        short,
        long,
        default_value = "false",
        long_help = "Run the program in debug mode,\nneeds a path to a pdf file called 'example.pdf' in the 'test-files' folder"
    )]
    debug: bool,
}

fn list_langs() {
    const NAME_WIDTH: usize = 30;
    const CODE_WIDTH: usize = 12;

    println!(
        "{:<width$} | {:<CODE_WIDTH$}",
        "Language",
        "ISO-639 Code",
        width = NAME_WIDTH
    );
    println!("{:-<width$}---{:-<CODE_WIDTH$}", "", "", width = NAME_WIDTH);

    for &(lang, code) in SUPPORTED_LANGUAGES.iter() {
        println!(
            "{:<width$} -> {:<CODE_WIDTH$}",
            lang,
            code,
            width = NAME_WIDTH
        );
    }
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
            let run_args = program::Args {
                file_path: "./test-files/example.pdf".to_string(),
                source: "en".to_string(),
                target: "sv".to_string(),
            };

            program::run(run_args).await;
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
        } else if args.list {
            list_langs();
        } else if args.config {
            let config = config::Config::new(args.api_key, args.project_id, args.access_token);
            config::setup(config);
        } else {
            let run_args = program::Args {
                file_path: args.path.unwrap(),
                source: "en".to_string(),
                target: "sv".to_string(),
            };
            program::run(run_args).await;
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
        } else if args.list {
            list_langs();
        } else if args.config {
            let config = config::Config::new(args.api_key, args.project_id, args.access_token);
            config::setup(config);
        } else {
            let run_args = program::Args {
                file_path: args.path.unwrap(),
                source: "en".to_string(),
                target: "sv".to_string(),
            };
            program::run(run_args).await;
        }
    }
}
