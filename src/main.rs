use std::path::Path;

use clap::Parser;
use duct::cmd;

#[derive(Debug, Clone, clap::Parser)]
#[command(version, about, long_about = None)]
struct Args {
    input: String,
    output: Option<String>,
}

fn main() {
    println!("Starting Markdown to PDF Converter.");
    let args = Args::parse();
    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| args.input.clone().replace(".md", ".pdf"));
    convert_to_pdf(Path::new(&args.input), &output_path);
    println!("Done.");
}

fn convert_to_pdf(md_file: &Path, output: &str) {
    let template_string = r#"
#import "@preview/cmarker:0.1.6"
#import "@preview/mitex:0.2.5": mitex

#set page(
  paper: "a4",
  margin: (left: 2.5cm, right: 1.5cm, top: 2cm, bottom: 2cm),
  numbering: "1",
  number-align: center,
)

#set text(
  size: 11pt,
  lang: "ru",
  hyphenate: true,
)

#set par(
  justify: true,
  leading: 0.75em,
  spacing: 0.65em,
  first-line-indent: 0pt,
)

#show heading: it => {
  set text(weight: "bold")
  block(
    above: 1.4em,
    below: 0.8em,
    it
  )
}

#show heading.where(level: 1): set text(size: 20pt)
#show heading.where(level: 2): set text(size: 17pt)
#show heading.where(level: 3): set text(size: 14.5pt)

#show list: set block(spacing: 0.65em, above: 0.6em, below: 0.6em)
#show enum: set block(spacing: 0.65em, above: 0.6em, below: 0.6em)
#set list(indent: 0.8em, body-indent: 0.5em)
#set enum(indent: 0.8em, body-indent: 0.5em)

#show raw.where(block: false): it => text(
  fill: rgb("1e1e1e"),
  font: "DejaVu Sans Mono",
  size: 0.95em,
  it
)

#show raw.where(block: true): it => block(
  fill: rgb("f5f5f5"),
  inset: 10pt,
  radius: 4pt,
  width: 100%,
  above: 0.8em,
  below: 0.8em,
  it
)

#show quote.where(block: true): it => block(
  inset: (left: 1em, top: 0.5em, bottom: 0.5em, right: 0.5em),
//   stroke: (left: 3pt + rgb("4a90e2")),
  fill: rgb("f8f9fa"),
  radius: (right: 4pt),
  above: 0.8em,
  below: 0.8em,
  it.body
)

#show table: set block(above: 0.8em, below: 0.8em)

#let constrained_image(path, alt: none) = {
  let img = image(path, alt: alt)
  align(center, block(above: 0.8em, below: 0.8em, box(width: 100%, img)))
}

#show link: set text(fill: rgb("0066cc"))

#cmarker.render(
  read("REPLACE_ME"),
  smart-punctuation: true,
  math: mitex,
  h1-level: 1,
  raw-typst: true,
  scope: (
    image: constrained_image,
  )
)
"#
    .trim()
    .replace("REPLACE_ME", md_file.to_str().unwrap());

    let output_path = Path::new(output);

    println!("Compiling PDF...");
    cmd!("typst", "compile", "--root", ".", "-", output_path)
        .stdin_bytes(template_string.as_bytes())
        .run()
        .unwrap();

    println!("Processing PDF with OCR...");
    if let Err(e) = cmd!(
        "uvx",
        "ocrmypdf",
        "--tesseract-timeout",
        "1000",
        "--tesseract-downsample-large-images",
        "-l",
        "rus+eng",
        "-O",
        "3",
        "--jbig2-lossy",
        "--remove-vectors",
        "--skip-text",
        "--output-type",
        "pdf",
        output_path,
        output_path
    )
    .run()
    {
        eprintln!("Warning: OCR processing failed: {}", e);
        eprintln!("PDF was created successfully, but OCR processing was skipped.");
    }
}
