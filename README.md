
# PDF Translator

The PDF Translator is a command-line tool written in Rust. It allows you to translate the content of a PDF file using Google's Translation API.

## Installation

Before using the tool, ensure you have the required dependencies installed:

1. `poppler-utils`: The tool relies on `pdftotext` from the `poppler-utils` package to read PDF files.

### Installing Poppler:

_(You should get a question about installing poppler-utils when you run the tool for the first time but currently it might fail.)_

On Linux:

```bash
sudo apt-get install poppler-utils
```

On MacOS:

```bash
brew install poppler
```

On Windows, the installation is currently not supported. Please refer to Poppler's official documentation for manual installation.

## Usage

To translate a PDF:

```bash
pdf-translator --path /path/to/your/pdf/file.pdf
```

This will create a translated text file in the same directory.

## Configuration

Before translating, you must configure the tool with your Google Cloud Platform API key, access token, and project ID:

```bash
pdf-translator --config --api_key YOUR_API_KEY --access_token YOUR_ACCESS_TOKEN --project_id YOUR_PROJECT_ID
```

## Dependencies

- `clap`: For argument parsing.
- `reqwest`: For making HTTP requests to the Google Translate API.
- `serde`: For serializing and deserializing JSON responses.
- `rpassword`: To securely prompt for the user's password during installation processes.
- `directories`: To determine the configuration file's path.
- `poppler-utils`: To convert PDF files to text.

## Development and Testing

To build the tool in development mode:

```bash
make dev
```

To build the tool in release mode:

```bash
make release
```

To run development mode with arg:

```bash
make dev-<r/c/i/h/hf>
```

The tool comes with unit tests for various modules. To run the tests:

```bash
make test
```

## Credits

Developed by Retrokiller543. Contributions are welcome. Please refer to the contribution guidelines before making a pull request.
