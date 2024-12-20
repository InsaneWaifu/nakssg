use std::{
    io::{BufRead, Write},
    path::Path,
};

use blog::BlogPageBase;
use chrono::{DateTime, FixedOffset};
use clap::{Parser, Subcommand};
use comrak::Options;
use trowel::{trowel_html, util::html_to_string};
mod blog;


#[derive(Debug)]
struct Page {
    title: String,
    timestamp: DateTime<FixedOffset>,
    slug: String,
    body: String,
}

impl Page {
    fn load(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let file = std::fs::File::open(path).expect("Could not find file");
        let reader = std::io::BufReader::new(file);
        let mut lines = reader.lines();
        let mut title = None;
        let mut timestamp = None;
        {
            assert!(lines.next().unwrap().unwrap() == "---");
            for line in lines.by_ref() {
                let line = line.unwrap();
                if line == "---" {
                    break;
                }
                let (key, value) = line.split_once(":").unwrap();
                match key {
                    "title" => {
                        title = Some(value.trim().to_string());
                    }
                    "timestamp" => {
                        timestamp = Some(chrono::DateTime::parse_from_rfc2822(value.trim()))
                    }
                    _ => unreachable!(),
                }
            }
            assert!(title.is_some());
            assert!(timestamp.is_some());
        }
        let slug = path.file_stem().unwrap();
        let remaining = lines.map(|x| x.unwrap() + "\n").collect::<String>();
        let mut options = Options::default();
        options.extension.table = true;
        options.extension.underline = true;
        options.extension.greentext = true;
        options.render.unsafe_ = true; // Allow html
        let body = comrak::markdown_to_html(&remaining, &options);
        Self {
            title: title.unwrap(),
            timestamp: timestamp.unwrap().unwrap(),
            slug: slug.to_str().unwrap().to_string(),
            body,
        }
    }
}

#[derive(Parser)]
struct Command {
    #[arg(short, long, default_value = "pages")]
    input_dir: String,
    #[command(subcommand)]
    command: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    Build {
        #[arg(short, long, default_value = "dist")]
        output_dir: String
    },
    New {
        slug_name: String,
    },
}

impl Command {
    fn run(self) {
        match self.command {
            Subcommands::New { slug_name } => {
                let path = Path::new(&self.input_dir)
                    .join(&slug_name)
                    .with_extension("md");
                assert!(!path.exists(), "Already exists!");
                let file = std::fs::File::create(path).expect("Could not create file");
                let mut writer = std::io::BufWriter::new(file);
                writeln!(writer, "---").unwrap();
                writeln!(writer, "title: {}", slug_name).unwrap();
                writeln!(writer, "timestamp: {}", chrono::Utc::now().to_rfc2822()).unwrap();
                writeln!(writer, "---").unwrap();
            }
            Subcommands::Build { output_dir } => {
                let output_dir = Path::new(&output_dir);
                std::fs::create_dir_all(output_dir).unwrap();
                let input_dir = Path::new(&self.input_dir);
                let static_dir = input_dir.join("static");
                let output_static_dir = output_dir.join("static");
                std::fs::create_dir_all(&static_dir).unwrap();
                std::fs::create_dir_all(&output_static_dir).unwrap();
                let pages = std::fs::read_dir(input_dir)
                    .unwrap()
                    .map(|x| x.unwrap().path())
                    .filter(|x| x.is_file() && x.extension().unwrap() == "md")
                    .map(Page::load)
                    .collect::<Vec<_>>();

                let pages2 = &pages;
                let index = html_to_string(trowel_html! {
                    !{let pages = pages2;},
                    !BlogPageBase(title: {Some("Blog")}) {
                        ul(style: "list-style-type:none;") {
                            {
                                pages.iter().map(|page| {
                                    Box::new(trowel_html!(
                                        li {
                                            a(href: {Some(format!("/{}", page.slug))}) {
                                                {page.title.as_str()},
                                                sub {
                                                    !{let timestamp = page.timestamp.to_rfc2822();},
                                                    time(datetime: {Some(&timestamp)}) {
                                                        {timestamp}
                                                    }
                                                }
                                            }
                                        }
                                    )) as Box<dyn Fn(&mut dyn trowel::HtmlWriter)>
                                }).collect::<Vec<Box<dyn Fn(&mut dyn trowel::HtmlWriter)>>>()
                            }
                        }
                    }
                });
                std::fs::write(output_dir.join("index.html"), index).unwrap();

                for page in pages {
                    let html = html_to_string(trowel_html! {
                        !BlogPageBase(title: {Some(&page.title)}, timestamp: {Some(page.timestamp.to_rfc2822())}) {
                            {page.body.as_str()}
                        }
                    });
                    std::fs::write(output_dir.join(page.slug).with_extension("html"), html)
                        .unwrap();
                }

                copy_static_content(static_dir, output_static_dir);
            }
        }
    }
}

fn copy_static_content(src: impl AsRef<Path>, dst: impl AsRef<Path>) {
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let src_path = entry.path();
        let dst_path = dst.as_ref().join(src_path.file_name().unwrap());
        if src_path.is_dir() {
            std::fs::create_dir_all(&dst_path).unwrap();
            copy_static_content(&src_path, &dst_path);
        } else {
            std::fs::copy(&src_path, &dst_path).unwrap();
        }
    }
}

fn main() {
    let command = Command::parse();
    command.run();
}
