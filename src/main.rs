use std::fs::File;
use std::io::Write;
use clap::Parser;

// create a module for reading the text of the pdf file and also checking if poppler is installed
mod pdf_reader {
    use std::process::Command;
    use std::io::{Error, Read};

    pub struct PdfReader {
        content: Vec<(usize, String)>,
    }

    impl PdfReader {
        pub fn new(path: &str) -> Result<PdfReader, Error> {
            PdfReader::check_poppler()?;

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
        

        fn read_file(file_path: &str) -> Result<String, std::io::Error> {
            let mut file = std::fs::File::open(file_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            Ok(contents)
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
    
        pub fn check_poppler() -> Result<(), Error> {
            let output = Command::new("pdftotext")
                .arg("-v")
                .output()?;
        
            let text = String::from_utf8(output.stderr).expect("Not UTF-8");
        
            if text.contains("Poppler") {
                Ok(())
            } else {
                println!("Poppler is not installed. Would you like to install it now? (yes/no)");
                let mut user_input = String::new();
                std::io::stdin().read_line(&mut user_input).expect("Failed to read line");
                if user_input.trim() == "yes" {
                    Command::new("sudo")
                        .arg("apt")
                        .arg("install")
                        .arg("-y")
                        .arg("poppler-utils")
                        .spawn()
                        .expect("Failed to start poppler installation process");
                    println!("Poppler installed successfully!");
                    Ok(())
                } else {
                    Err(Error::new(std::io::ErrorKind::Other, "Poppler not installed"))
                }
            }
        }
    }

}


mod translator {
    use reqwest;
    use serde::Serialize;
    use std::collections::HashMap;
    use dotenv_codegen::{self, dotenv};

    const GOOGLE_TRANSLATE_API_ENDPOINT: &str = "https://translation.googleapis.com/language/translate/v2";
    const API_KEY: &str = dotenv!("API_KEY");
    const PROJECT_ID: &str = dotenv!("PROJECT_ID");
    const ACCESS_TOKEN: &str = dotenv!("ACCESS_TOKEN");
    
    #[derive(Serialize)]
    struct TranslateRequest {
        q: String,
        source: String,
        target: String,
        format: String,
        key: String,
    }

    pub async fn translate_text(formatted_content: Vec<(usize, String)>) -> Result<Vec<(usize, String)>, reqwest::Error> {
        let client = reqwest::Client::new();
        let mut translated_texts = Vec::new();
    
        for (line_number, line) in formatted_content {
            let mut payload = HashMap::new();
            payload.insert("q", line.clone());
            payload.insert("source", "en".to_string());
            payload.insert("target", "sv".to_string());
            payload.insert("format", "text".to_string());
            payload.insert("key", API_KEY.to_string());
    
            let access_token = "Bearer ".to_string() + ACCESS_TOKEN;
    
            let response: serde_json::Value = client
                .post(GOOGLE_TRANSLATE_API_ENDPOINT)
                .header("Authorization", access_token)
                .header("x-goog-user-project", PROJECT_ID)
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


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: String,
    #[arg(short, long)]
    debug: bool,
}


#[tokio::main]
async fn main() {
    let args = Args::parse();

    let pdf_reader = pdf_reader::PdfReader::new(args.path.as_str()).expect("Error reading pdf");
    
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