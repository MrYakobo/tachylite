// creates a static site directory html_output given input folder site

use clap::Parser;
use comrak::plugins::syntect::SyntectAdapterBuilder;
use comrak::{markdown_to_html_with_plugins, Anchorizer, ComrakOptions};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::{self, read_to_string};
use std::io;
use std::path::Path;

fn generate_docnav(markdown: &str) -> String {
    let arena = comrak::Arena::new();
    let root = comrak::parse_document(&arena, markdown, &ComrakOptions::default());

    let mut heading_ids: HashMap<String, String> = HashMap::new();
    let mut heading_html = String::new();

    for node in root.descendants() {
        if let comrak::nodes::NodeValue::Heading(ref heading) = node.data.borrow().value {
            if let Some(text_node) = node.first_child() {
                if let comrak::nodes::NodeValue::Text(ref text) = text_node.data.borrow().value {
                    let slug = Anchorizer::new().anchorize(text.to_string());
                    heading_ids.insert(slug.clone(), text.clone());
                    heading_html
                        .push_str(&format!(r##"<li><a href="#{}">{}</a></li>"##, slug, text));
                }
            }
        }
    }

    let mut docnav_html = String::new();
    docnav_html.push_str(&format!(
        r#"<nav id="docnav"><div class="prose prose-invert uppercase mb-4"><p>On this page</p></div><ul>"#
    ));
    docnav_html.push_str(&heading_html);
    docnav_html.push_str("</ul></nav>");
    docnav_html
}

fn generate_html(markdown: &str, blog_name: &str, title: &str, sidebar: &str, toc: &str) -> String {
    let mut options = comrak::ComrakOptions::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.superscript = true;
    options.extension.header_ids = Some("".to_owned());

    let builder = SyntectAdapterBuilder::new().theme("base16-ocean.dark");
    let adapter = builder.build();

    let mut plugins = comrak::Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    let content = markdown_to_html_with_plugins(markdown, &options, &plugins);

    let title = format!("{} | {}", title, blog_name);

    include_str!("template.html")
        .replace("{{title}}", &title)
        .replace("{{blog_name}}", blog_name)
        .replace("{{sidebar}}", sidebar)
        .replace("{{content}}", &content)
        .replace("{{toc}}", toc)
}

fn generate_sidebar(
    dir_path: &Path,
    current_file_path: &Path,
    site_root: &str,
    is_subdir: bool,
) -> String {
    let mut sidebar_html = String::new();
    if is_subdir {
        sidebar_html.push_str(r#"<ul class="">"#);
    } else {
        sidebar_html.push_str(r#"<ul class="pl-4">"#);
    }

    let mut entries = fs::read_dir(dir_path)
        .expect("Failed to read directory")
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .expect("uh");

    entries.sort_by_key(|path| path.is_file());

    for path in entries {
        if path.is_dir() {
            // Generate a collapsible folder
            let is_current_file_in_dir = current_file_path.starts_with(&path);
            let summary_attribute = if is_current_file_in_dir { "open" } else { "" };
            sidebar_html.push_str(&format!(
                r#"<li><details {}><summary class="pl-4 py-1 hover:bg-neutral-700">{}</summary><ul class="border-l border-white/30 ml-[1.15rem] my-2">{}</ul></details></li>"#,
                summary_attribute,
                path.file_name().expect("uh").to_string_lossy(),
                generate_sidebar(&path, current_file_path, site_root, true)
            ));
        } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            // Generate a relative path from current_file_path to the file's output HTML path
            let file_name = path.file_stem().unwrap().to_string_lossy();

            if file_name == "index" {
                continue;
            }

            // the link should strip the site root prefix
            let abs_path = path.strip_prefix(site_root).unwrap();
            let html_path = abs_path.with_extension("html");

            // let class = if is_subdir { "border-l" } else { "" };
            let class = "";

            // current link
            let active_class = if current_file_path
                .to_string_lossy()
                .ends_with(&*abs_path.to_string_lossy())
            {
                "text-violet-400 border-violet-400"
            } else {
                "hover:border-white/80 hover:text-white/80 border-white/30"
            };

            sidebar_html.push_str(&format!(
                r#"<li><a class="{} {} hover:bg-neutral-700 block active:border-violet-400 active:text-violet-400 pl-4 py-1 whitespace-nowrap" href="/{}">{}</a></li>"#,
                class,
                active_class,
                html_path.display(),
                file_name
            ));
        }
    }

    sidebar_html.push_str("</ul>");
    sidebar_html
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(help = "Sets the input directory", required = true)]
    input_dir: String,

    #[clap(
        short,
        long,
        help = "Sets the output directory",
        default_value = "output"
    )]
    output_dir: String,
}

fn process_markdown_files(input_dir: &str, output_dir: &str) {
    let input_path = Path::new(input_dir);

    // Recursively iterate through files and directories
    fn process_dir(dir_path: &Path, output_dir: &Path, input_path: &Path, input_dir: &str) {
        let mut entries = fs::read_dir(dir_path)
            .expect("Failed to read directory")
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()
            .expect("uh");

        entries.sort_by_key(|path| path.is_file());

        for path in entries {
            if path.is_dir() {
                // Create the output directory if it doesn't exist
                let output_subdir = output_dir.join(path.file_name().unwrap());
                fs::create_dir_all(&output_subdir).expect("Failed to create output directory");

                // Recursive call for subdirectories
                process_dir(&path, &output_dir, input_path, input_dir);
            } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                // Read the Markdown content
                let markdown_content = read_to_string(&path).expect("Could not read markdown file");

                // Get the file title
                let file_title = path.file_stem().unwrap().to_str().unwrap().to_string();

                // Generate sidebar for the file
                let sidebar_html = generate_sidebar(input_path, &path, input_dir, false);

                // Generate documentation navigation
                let docnav_html = generate_docnav(&markdown_content);

                let blog_name = Anchorizer::new().anchorize(input_dir.to_string());

                // Convert the Markdown content to HTML
                let html_content = generate_html(
                    &markdown_content,
                    &blog_name,
                    &file_title,
                    &sidebar_html,
                    &docnav_html,
                );

                // Create the HTML file in the output directory
                let output_file_path = output_dir
                    .join(path.strip_prefix(input_path).unwrap())
                    .with_extension("html");

                fs::create_dir_all(&output_file_path.parent().unwrap())
                    .expect("couldn't create subdir in output directory");

                let err = format!("Could not write HTML file to {:?}", output_file_path);
                fs::write(&output_file_path, html_content).expect(&err);
                println!("Generated HTML file: {:?}", output_file_path);
            }
        }
    }

    // Start processing from the input directory
    process_dir(input_path, Path::new(output_dir), &input_path, &input_dir);
}

const CSS: &[u8] = include_bytes!("assets/style.css");
const JS: &[u8] = include_bytes!("assets/scroll.js");

fn copy_assets(output_dir: &str) {
    let output_assets_dir = Path::new(output_dir).join("assets");
    fs::create_dir_all(&output_assets_dir).expect("Failed to create assets directory");

    let css_path = output_assets_dir.join("style.css");
    fs::write(css_path, CSS).expect("Failed to write CSS file");

    let js_path = output_assets_dir.join("scroll.js");
    fs::write(js_path, JS).expect("Failed to write JS file");

    println!("Assets copied successfully to {:?}", output_assets_dir);
}

fn main() {
    let args = Args::parse();

    let input_dir = &args.input_dir;
    let output_dir = &args.output_dir;

    // Create the output directory if it doesn't exist
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    process_markdown_files(input_dir, output_dir);
    copy_assets(output_dir)
}
