use std::fs::File;
use std::io::Write;
use clap::Parser;

// create a module for reading the text of the pdf file and also checking if poppler is installed
mod pdf_reader {
    use std::process::Command;
    use std::io::{Error, Read};

    pub struct PdfReader {
        path: String,
    }

    impl PdfReader {
        pub fn new(path: &str) -> Result<PdfReader, Error> {
            PdfReader::check_poppler()?;

            PdfReader::read_pdf(path)?;

            Ok(PdfReader {
                path: path.to_string(),
            })
        }

        fn read_file(file_path: &str) -> Result<String, std::io::Error> {
            let mut file = std::fs::File::open(file_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            Ok(contents)
        }

        pub fn get_content(&self) -> String {
            let file_path = &self.path.replace(".pdf", ".txt");
            // read file at file_path and return content
            let content = PdfReader::read_file(file_path).unwrap_or_default();
            return content.to_owned();
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

    pub async fn translate_text(text: String) -> Result<String, reqwest::Error> {
        let client = reqwest::Client::new();
        let chunks = split_text(text.as_str());

        let mut translated_texts = Vec::new();

        for chunk in chunks {
            let mut payload = HashMap::new();
            payload.insert("q", chunk);
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

            let translated_chunk = parse_response(&response.to_string()).expect("Error parsing response");
            translated_texts.push(translated_chunk);
        }

        Ok(translated_texts.join(" "))
    }

    fn split_text(text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut chunk = String::new();
        for word in text.split_whitespace() {
            if chunk.len() + word.len() > 102400 {
                chunks.push(chunk);
                chunk = String::new();
            }
            chunk.push_str(word);
            chunk.push(' ');
        }
        if !chunk.is_empty() {
            chunks.push(chunk);
        }
        chunks
    }

    fn parse_response(response: &str) -> Result<String, serde_json::Error> {
        let v: serde_json::Value = serde_json::from_str(response)?;
        let translated_text = v["data"]["translations"][0]["translatedText"].as_str().unwrap_or_default().to_string();
        Ok(translated_text)
    }
}

fn format_by_sentences(text: &str, sentences_per_paragraph: usize) -> String {
    let sentences: Vec<&str> = text.split(". ").collect();
    
    let mut formatted_paragraphs = Vec::new();
    let mut paragraph = Vec::new();
    
    for sentence in &sentences {
        paragraph.push(*sentence);
        if paragraph.len() == sentences_per_paragraph {
            formatted_paragraphs.push(paragraph.join(". ") + ".");
            paragraph.clear();
        }
    }
    
    // Append any remaining sentences as the last paragraph
    if !paragraph.is_empty() {
        formatted_paragraphs.push(paragraph.join(". ") + ".");
    }
    
    formatted_paragraphs.join("\n\n")
}


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: String,
}


#[tokio::main]
async fn main() {
    let args = Args::parse();

    let pdf_reader = pdf_reader::PdfReader::new(args.path.as_str()).expect("Error reading pdf");
    
    match translator::translate_text(pdf_reader.get_content()).await {
        Ok(mut translated_text) => {
            // Format the translated text
            translated_text = format_by_sentences(&translated_text, 5);

            let mut file = File::create("translated_text.txt").expect("Error creating file");
            file.write_all(translated_text.as_bytes()).expect("Error writing to file");
            println!("Translation complete");
        },
        Err(e) => println!("Error translating: {}", e),
    }
}