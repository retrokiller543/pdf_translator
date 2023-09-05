use clap::Parser;

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
                content: content,
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
            return self.content.clone();
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

}


mod translator {
    use reqwest;
    use std::env::var;
    use serde::Serialize;
    use std::collections::HashMap;

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
        let api_key: String = var("API_KEY").unwrap_or_default();
        let project_id: String = var("PROJECT_ID").unwrap_or_default();
        let a_t: String = var("ACCESS_TOKEN").unwrap_or_default();
        let client = reqwest::Client::new();
        let mut translated_texts = Vec::new();
    
        for (line_number, line) in formatted_content {
            let mut payload = HashMap::new();
            payload.insert("q", line.clone());
            payload.insert("source", "en".to_string());
            payload.insert("target", "sv".to_string());
            payload.insert("format", "text".to_string());
            payload.insert("key", api_key.clone());
    
            let access_token = "Bearer ".to_string() + a_t.as_str();
    
            let response: serde_json::Value = client
                .post(GOOGLE_TRANSLATE_API_ENDPOINT)
                .header("Authorization", access_token)
                .header("x-goog-user-project", project_id.clone())
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
        let translated_text = v["data"]["translations"][0]["translatedText"].as_str().unwrap_or_default().to_string();
        Ok(translated_text)
    }
}

mod config {
    use std::env;

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
    }

    pub fn setup(args: Config) {
        let api_key = args.api_key;
        let project_id = args.project_id;
        let access_token = args.access_token;

        if api_key == "" && project_id == "" && access_token == "" {
            println!("You must at least provide one of the following arguments '--api_key <API_KEY>', '--access_token <ACCESS_TOKEN>', '--project_id <PROJECT_ID>' ")
        }

        // add secrets to env variables
        if api_key != "" {
            env::set_var("API_KEY", &api_key);
        }
        if project_id != "" {
            env::set_var("PROJECT_ID", &project_id);
        }
        if access_token != "" {
            env::set_var("ACCESS_TOKEN", &access_token);
        }
    }
}


mod install {
    use std::process::Command;
    use std::io::Error;
    use rpassword::read_password;

    pub fn run() -> Result<(), String> {
        let _ = check_poppler();
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn install() -> Result<(), String> {
        let installed_manager = get_package_manager();

        if installed_manager == "" {
            return Err("No package manager is installed".to_string());
        }

        // Prompt user for password
        print!("Please enter your sudo password: ");
        let password = read_password().expect("Failed to read password");
        
        let error_msg = "Error installing using package manager '".to_string() + &installed_manager.as_str() + "'";

        // Pipe the password to sudo
        Command::new("sh")
            .arg("-c")
            .arg(format!("echo {} | sudo -S {} install -y poppler-utils", password.trim(), installed_manager))
            .spawn()
            .expect(error_msg.as_str());
        Ok(())
    }

    #[cfg(target_os = "mac")]
    fn install() -> Result<(), String> {
        // Prompt user for password
        print!("Please enter your sudo password: ");
        let password = read_password().expect("Failed to read password");
        let error_msg = "Error installing using package manager 'brew'";
        if !check_brew(){
            let brew_install = "/bin/bash -c " + "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)";
            Command::new("sh")
                .arg("-c")
                .arg(format!("echo {} | sudo -S {}", password.trim(), brew_install))
                .spawn()
                .expect(error_msg);
            
            let poppler_install = "brew install poppler";
        }

        Command::new("sh")
            .arg("-c")
            .arg(format!("echo {} | sudo -S {}", password.trim(), poppler_install))
            .spawn()
            .expect(error_msg);
        Ok(())
    }

    #[cfg(target_os = "mac")]
    fn check_brew() -> bool {
        let output = Command::new("which")
            .arg("brew")
            .output()
            .expect("Error: 'which' command not found!");

        // If the command succeeded, then brew exists on the system
        if output.status.success() {
            return true;
        }

        false
    }

    fn check_poppler() -> Result<(), Error> {
        let output = Command::new("pdftotext")
            .arg("-v")
            .output()?;
    
        let text = String::from_utf8(output.stderr).expect("Not UTF-8");
    
        if text.contains("Poppler") {
            Ok(())
        } else {
            println!("Poppler is not installed.");
            let _ = install();
            Ok(())
        }
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
}



mod program {
    use std::fs::File;
    use std::io::Write;
    use crate::pdf_reader;
    use crate::translator;

    
    pub async fn run(file_path: String) {
        let pdf_reader = pdf_reader::PdfReader::new(&file_path.as_str()).expect("Error reading pdf");
    
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


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: Option<String>,
    #[arg(short, long, default_value = "false")]
    install: bool,
    #[arg(short, long, default_value = "false")]
    config: bool,
    #[arg(long, default_value = "")]
    api_key: String,
    #[arg(long, default_value = "")]
    access_token: String,
    #[arg(long, default_value = "")]
    project_id: String,
    #[arg(short, long, default_value = "false")]
    debug: bool,
}


#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.debug {
        program::run("./test-files/example.pdf".to_string()).await;
    } else if args.install {
        let _result = install::run();
    } else if args.config {
        let config = config::Config::new(args.api_key, args.project_id, args.access_token);
        config::setup(config);
    } else {
        let path = args.path.unwrap();
        program::run(path).await;
    }
}