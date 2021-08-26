use chrono::prelude::*;
use chrono::{NaiveDate, NaiveDateTime};
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::env;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::Cursor;
use std::path::PathBuf;
use std::str;
use structopt::StructOpt;

#[derive(Serialize, Deserialize)]
struct Entry {
    name: String,
    date: String,
    published: bool,
}

#[derive(Serialize, Deserialize)]
struct BlogFile {
    config: PathBuf,
    entries: Vec<Entry>,
}

#[derive(Serialize, Deserialize)]
struct Config {
    blog: PathBuf,
    rss: PathBuf,
    template: PathBuf,
    website: String,
}

/// Arguments
#[derive(StructOpt, Debug)]
#[structopt(
    name = "OB - Oliver's Blog System",
    about = "A Blog and RSS system written in Rust.",
    author = "Oliver Brotchie, o.brotchie@gmail.com"
)]
struct Args {
    /// Create a new draft.
    #[structopt(short, long)]
    new: bool,
    /// Delete a draft.
    #[structopt(short, long)]
    delete: bool,
    /// Publish a daft
    #[structopt(short, long)]
    publish: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if check_setup().is_ok() {
        let blog_file: BlogFile = serde_json::from_str(&fs::read_to_string("/blog/blog.json")?)?;
        let config: Config = serde_json::from_str(&fs::read_to_string(&blog_file.config)?)?;
        let args = Args::from_args();

        if args.new {
            new_draft(blog_file)?
        } else if args.publish {
            publish_draft(blog_file, config)?
        } else {
            delete(blog_file, config)?
        }
    }
    Ok(())
}

fn check_setup() -> Result<(), io::Error> {
    let cur_path_buf = env::current_dir()?;
    let cur_dir = cur_path_buf.as_path();

    // Run setup if it hasnt alread been done
    if !fs::read_dir(cur_dir)?.any(|x| {
        let x = x.expect("Error");
        x.file_type().unwrap().is_dir() && x.file_name().to_str().unwrap() == "blog"
    }) {
        match setup() {
            Ok(()) => {}
            Err(e) => {
                println!("Something went wrong when with setup. Error: {}", e)
            }
        }
    }
    Ok(())
}

fn setup() -> Result<(), io::Error> {
    let mut config_path: PathBuf;
    loop {
        println!("Please input the path to your config file:\n\t");
        match read_input() {
            Ok(input) => {
                config_path = PathBuf::from(input);
                match fs::read_to_string(&config_path) {
                    Ok(s) => match serde_json::from_str::<Config>(&s) {
                        Ok(_) => break,
                        _ => println!("Config file was not valid."),
                    },
                    _ => println!("Path to config file was not valid."),
                }
            }
            _ => println!("Error whilst reading input."),
        }
    }
    let f = BlogFile {
        config: config_path,
        entries: Vec::new(),
    };
    fs::create_dir_all("/blog/drafts")?;
    fs::write("/blog/blog.json", serde_json::to_string(&f)?)?;
    Ok(())
}

fn read_input() -> Result<String, io::Error> {
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf)
}

fn display_choices(blog_file: &Vec<Entry>) -> Result<usize, Box<dyn std::error::Error>> {
    for i in 0..blog_file.len() {
        println!("{}. {}", i, blog_file[i].name);
    }
    println!("\n\t");
    Ok(read_input()?.parse::<usize>()?)
}

fn new_draft(mut blog_file: BlogFile) -> Result<(), io::Error> {
    println!("Please enter the title of the blog post:\n\t");
    let name = read_input()?;

    // Create draft
    fs::File::create(format!("/blog/drafts/{}.md", name))?;
    blog_file.entries.push(Entry {
        name,
        published: false,
        date: String::new(),
    });
    fs::write("/blog/blog.json", serde_json::to_string(&blog_file)?)?;
    Ok(())
}

fn delete(mut blog_file: BlogFile, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Please enter the number of the entry you wish to delete:");
    let choice = blog_file
        .entries
        .remove(display_choices(&blog_file.entries)?);

    if choice.published {
        fs::remove_file(format!("/blog/{}.html", &choice.name))?;
    } else {
        fs::remove_file(format!("/blog/drafts/{}.md", &choice.name))?;
    }

    Ok(())
}

fn publish_draft(blog_file: BlogFile, config: Config) -> Result<(), io::Error> {
    Ok(())
}

/// Interpolate json data into word file.
fn interpolate_json(
    buf: Vec<u8>,
    json: &Map<String, Value>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_str(str::from_utf8(&buf)?);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut xml_buf = Vec::new();
    let mut found = false;
    // Loop over every tag in the XML document
    loop {
        if !found {
            // Continue to iterate until the start of a variable
            match reader.read_event(&mut xml_buf) {
                Ok(Event::Empty(ref e))
                    if e.name() == b"w:fldChar"
                        && e.attributes()
                            .any(|a| a.unwrap().value.into_owned() == b"begin") =>
                {
                    found = true
                }
                Ok(Event::Eof) => break,
                Ok(e) => writer.write_event(e)?,
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            }
        } else {
            // When the start of a new variable is found,
            // skip through and replace it with the desired json value.
            match reader.read_event(&mut xml_buf) {
                Ok(Event::Start(ref e)) if e.name() == b"w:t" => {
                    let mut text_buf = Vec::new();
                    reader.read_text(e.name(), &mut text_buf)?;
                    let text = String::from_utf8(text_buf)?
                        .replace("«", "")
                        .replace("»", "")
                        .replace("/w:t", "");
                    // Test each json value
                    json.iter().for_each(|(key, value)| {
                        if text == key.trim() {
                            // Write in a text tag
                            writer
                                .write_event(Event::Start(BytesStart::borrowed(e, e.name().len())))
                                .expect("Error whilst writing value");
                            writer
                                .write_event(Event::Text(BytesText::from_plain_str(
                                    value.as_str().unwrap(),
                                )))
                                .expect("Error: Incorrect Json, key was not a String");
                            writer
                                .write_event(Event::End(BytesEnd::borrowed(b"w:t")))
                                .expect("Error: Could not close tag");
                        }
                    })
                }
                Ok(Event::Empty(ref e))
                    if e.name() == b"w:fldChar"
                        && e.attributes()
                            .any(|a| a.unwrap().value.into_owned() == b"end") =>
                {
                    found = false
                }
                Ok(_) => (),
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            }
        }
        xml_buf.clear();
    }
    Ok(writer.into_inner().into_inner())
}
