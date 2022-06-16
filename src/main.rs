use chrono::prelude::*;
use quick_xml::{events::Event, Reader, Writer};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{self, Cursor},
    path::{Path, PathBuf},
    str,
};
use structopt::StructOpt;

/// Json representation of an entry
#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Entry {
    id: String,
    name: String,
    date: String,
    author: String,
    img: Option<String>,
    published: bool,
}

/// The OB data file
#[derive(Serialize, Deserialize)]
struct BlogFile {
    /// The directory containing the config file
    config_dir: PathBuf,
    /// The name of the config file
    config: PathBuf,
    /// The list of entries
    entries: Vec<Entry>,
}

/// The template of the configuration file
#[derive(Serialize, Deserialize)]
struct Config {
    blog: PathBuf,
    template: PathBuf,
    rss: PathBuf,
    items: usize,
    blog_address: String,
    images: bool,
}

/// Arguments
#[derive(StructOpt, Debug, PartialEq)]
#[structopt(
    name = "OB",
    about = "A Blog and RSS system written in Rust.",
    author = "Oliver Brotchie, o.brotchie@gmail.com"
)]
enum Args {
    /// Create a new draft.
    New,
    /// Edit a published entry.
    Edit,
    /// Publish a daft
    Publish,
    /// Deletes an entry.
    Delete,
    /// Regenerates all blog entries
    Regen,
}

#[derive(PartialEq, Debug)]
enum Flag {
    Rss,
    Blog,
    Template,
    Regen,
}

struct State<'a> {
    r: Reader<&'a [u8]>,
    buf: Vec<u8>,
}

impl<'a> Iterator for State<'a> {
    type Item = Event<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.buf.clear();
        match self.r.read_event(&mut self.buf) {
            Ok(Event::Eof) => None,
            Ok(e) => Some(e.into_owned()),
            Err(e) => panic!(
                "Error when reading file, {}, at position: {:#?}.",
                self.r.buffer_position(),
                e
            ),
        }
    }
}

macro_rules! state {
    ( $x:expr ) => {
        (
            State {
                r: Reader::from_str($x),
                buf: Vec::<u8>::new(),
            },
            Writer::new(Cursor::new(Vec::new())),
        )
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup()?;
    let blog_file: BlogFile = serde_json::from_str(&fs::read_to_string("blog/.config.json")?)?;
    let config: Config = serde_json::from_str(&fs::read_to_string(
        &blog_file.config_dir.join(&blog_file.config),
    )?)?;
    let args = Args::from_args();

    if args == Args::New {
        new_draft(blog_file)?
    } else if blog_file.entries.is_empty() {
        println!("No blog entries exist.")
    } else {
        match args {
            Args::Edit => edit(blog_file, config)?,
            Args::Publish => publish_draft(blog_file, config)?,
            Args::Delete => delete(blog_file, config)?,
            Args::Regen => regen(blog_file, config)?,
            _ => {}
        }
    }
    Ok(())
}

/// Setup the cwd with the required file structure
fn setup() -> Result<(), io::Error> {
    let cur_path_buf = env::current_dir()?;
    let cur_dir = cur_path_buf.as_path();

    // Run setup if it hasnt alread been done
    if !fs::read_dir(cur_dir)?.any(|x| {
        let x = x.expect("Error");
        x.file_type().unwrap().is_dir() && x.file_name().to_str().unwrap() == "blog"
    }) {
        let mut config: String;
        println!("Blog Setup");
        loop {
            println!("Please input the path to your config file:");
            match read_input() {
                Ok(input) => {
                    config = input;
                    clear();
                    match fs::read_to_string(&config) {
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
        let i: usize = config.rfind('/').unwrap_or(0) + 1;
        let f = BlogFile {
            config_dir: PathBuf::from(if i == 1 { "" } else { &config[..i] }),
            config: PathBuf::from(if i == 1 { &config } else { &config[i..] }),
            entries: Vec::new(),
        };
        fs::create_dir_all("blog/drafts/")?;
        fs::write("blog/.config.json", serde_json::to_string(&f)?)?;
        clear();
    }
    Ok(())
}

/// Create a new draft (will be saved in '/blog/drafts')
fn new_draft(mut blog_file: BlogFile) -> Result<(), io::Error> {
    println!("Please enter the title of the blog post:");
    let name = read_input()?;

    println!("Please enter the name of the author:");
    let author = read_input()?;

    // Create draft
    fs::File::create(format!("blog/drafts/{}.md", name))?;
    blog_file.entries.push(Entry {
        id: random_string::generate(14, "0123456789"),
        name,
        author,
        published: false,
        date: String::new(),
        img: None,
    });
    fs::write("blog/.config.json", serde_json::to_string(&blog_file)?)?;
    Ok(())
}

/// Delete a blog entry or draft
fn delete(mut blog_file: BlogFile, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Please enter the number of the blog post you wish to delete:");
    let choice = blog_file
        .entries
        .remove(display_choices(&blog_file.entries)?);

    if choice.published {
        fs::remove_file(format!("blog/{}.html", choice.id))?;

        // Remove XML and HTML entries
        remove_xml(blog_file.config_dir.join(config.rss), &choice)?;
        remove_xml(blog_file.config_dir.join(config.blog), &choice)?;
    } else {
        fs::remove_file(format!("blog/drafts/{}.md", &choice.name))?;
    }

    fs::write("blog/.config.json", serde_json::to_string(&blog_file)?)?;

    Ok(())
}

/// Edit given XML or HTML files and remove certain tags
fn remove_xml(path: PathBuf, entry: &Entry) -> Result<(), Box<dyn std::error::Error>> {
    let f = fs::read_to_string(&path)?;
    let (mut s, mut w) = state!(&f);
    let mut found = false;

    // Loop over the xml tags
    while let Some(e) = s.next() {
        match e {
            Event::CData(e) if !found => {
                w.write(
                    format!(
                        "<![CDATA[{}]]>\n",
                        str::from_utf8(&e.unescaped()?.into_owned())?
                    )
                    .as_bytes(),
                )?;
            }
            Event::Start(ref e)
                if (e.name() == b"item" || e.name() == b"li")
                    && e.attributes().any(|a| {
                        a.unwrap().value.into_owned() == entry.id.clone().into_bytes()
                    }) =>
            {
                found = true
            }
            Event::End(ref e) if found && (e.name() == b"item" || e.name() == b"li") => {
                found = false
            }
            e if !found => w.write_event(e)?,
            _ => {}
        }
    }
    fs::write(path, w.into_inner().into_inner())?;
    Ok(())
}

/// Publish a draft and convert it from Markdown to HTML
fn publish_draft(
    mut blog_file: BlogFile,
    config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read in choice
    println!("Please enter the number of the draft you wish to publish:");
    let (list, mut choices): (Vec<Entry>, Vec<Entry>) =
        blog_file.entries.into_iter().partition(|e| e.published);
    let i = display_choices(&choices)?;

    let mut already_exists = false;

    let html = match fs::read_to_string(format!("blog/drafts/{}.md", &choices[i].name)) {
        // Convert markdown file to html
        Ok(s) => markdown::to_html(&s),
        // Draft is already html
        Err(_) => {
            already_exists = true;
            fs::read_to_string(format!("blog/drafts/{}.html", &choices[i].name))?
        }
    };

    // Ask for images
    if config.images {
        println!("Please enter a url to a cover image:");
        choices[i].img = Some(read_input()?);
    }
    choices[i].date = Utc::now().to_rfc2822();

    // Create blog entry
    insert_xml(
        &blog_file.config_dir.join(&config.template),
        &config,
        &choices[i],
        &html,
        Flag::Template,
    )?;
    // Edit rolling blog and rss feed
    insert_xml(
        &blog_file.config_dir.join(&config.rss),
        &config,
        &choices[i],
        &html,
        Flag::Rss,
    )?;

    // Test if editing
    if !already_exists {
        insert_xml(
            &blog_file.config_dir.join(&config.blog),
            &config,
            &choices[i],
            &html,
            Flag::Blog,
        )?
    };

    choices[i].published = true;
    choices.extend(list);
    blog_file.entries = choices.clone();
    fs::write("blog/.config.json", serde_json::to_string(&blog_file)?)?;
    if already_exists {
        fs::remove_file(format!("blog/drafts/{}.html", choices[i].name))?;
    } else {
        fs::remove_file(format!("blog/drafts/{}.md", choices[i].name))?;
    }
    Ok(())
}

/// Insert XML or HTML into a given file
fn insert_xml(
    path: &Path,
    config: &Config,
    entry: &Entry,
    html: &str,
    flag: Flag,
) -> Result<(), Box<dyn std::error::Error>> {
    let f = fs::read_to_string(&path)?;
    let (mut s, mut w) = state!(&f);

    let insert = if flag == Flag::Regen {
        html.to_owned()
    } else {
        // Create template string
        let mut insert = format!(
            "<{size}>{name}</{size}><span>by {author} </span><time datetime='{rfc}'>{date}</time>",
            name = &entry.name,
            author = &entry.author,
            rfc = &entry.date,
            size = if flag == Flag::Blog { "h3" } else { "h1" },
            date = entry
                .date
                .split(' ')
                .take(4)
                .fold(String::new(), |acc, e| acc + " " + e)[1..]
                .to_string()
        );
        if let Some(img) = &entry.img {
            insert = format!("<img src='{}'/><label>{}</label>", xml_escape(img), insert);
        }
        match flag {
            Flag::Rss => {
                insert = format!(
                    "<item id='{id}'><title>{name}</title><guid>{address}{id}</guid><pubDate>{rfc}</pubDate><description><![CDATA[{template}{html}]]></description></item>",
                    id = entry.id,
                    name = xml_escape(&entry.name),
                    address = config.blog_address,
                    rfc = &entry.date,
                    template = &insert,
                    html = html.replace('\n', ""),
                )
            }
            Flag::Blog => {
                insert = format!(
                    "<li id='{id}'><a href='{address}{id}'>{insert}</a></li>",
                    id = entry.id,
                    address = config.blog_address,
                    insert = insert,
                )
            }
            Flag::Template => insert = insert + "\n" + html,
            _ => {},
        }
        insert
    };

    // Rss count variables
    let mut found = false;
    let mut count = 1;

    // Loop over every tag
    while let Some(e) = s.next() {
        match e {
            Event::CData(e) if found && count <= config.items => {
                w.write(
                    format!(
                        "<![CDATA[{}]]>\n",
                        str::from_utf8(&e.unescaped()?.into_owned())?
                    )
                    .as_bytes(),
                )?;
            }
            // Remove excess items on the rss feed
            Event::Start(e) if flag == Flag::Rss && e.name() == b"item" => {
                count += 1;
                found = true;
                if count <= config.items {
                    w.write_event(Event::Start(e))?;
                }
            }
            Event::End(e) if flag == Flag::Rss && e.name() == b"item" => {
                found = false;
                if count <= config.items {
                    w.write_event(Event::End(e))?;
                }
            }

            // Add titles to the template
            Event::Start(e) if flag == Flag::Template && e.name() == b"title" => {
                w.write(format!("<title>{}", entry.name).as_bytes())?
            }

            // Generic insert
            Event::Comment(e) => {
                if &*e.unescaped()? == b" OB " {
                    w.write(b"<!-- OB -->\n")?;
                    w.write(insert.as_bytes())?;
                } else {
                    w.write_event(Event::Comment(e))?;
                }
            }

            // Remove excess items on the rss feed
            _ if found && count > config.items => (),
            e => w.write_event(e)?,
        }
    }

    fs::write(
        if flag == Flag::Template || flag == Flag::Regen {
            PathBuf::from(format!("blog/{}.html", entry.id))
        } else {
            path.to_path_buf()
        },
        w.into_inner().into_inner(),
    )?;
    Ok(())
}

fn get_inner(path: PathBuf) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let f = fs::read_to_string(&path)?;
    let (mut s, mut w) = state!(&f);
    let mut skip = true;
    let mut cur: usize = 0;
    let mut inner_level: usize = usize::MAX;

    // Loop over every tag
    while let Some(e) = s.next() {
        match &e {
            Event::Comment(e) if &*e.unescaped()? == b" OB " => inner_level = cur,
            Event::Start(_) => cur += 1,
            Event::End(_) => cur -= 1,
            _ => {}
        }
        if cur >= inner_level {
            if skip {
                // Skip generated title etc
                for _ in 1..14 {
                    s.next();
                }
                skip = false;
            } else {
                w.write_event(e)?;
            }
        }
    }

    Ok(w.into_inner().into_inner())
}

fn edit(mut blog_file: BlogFile, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Please enter the number of the article you wish to edit:");
    let (list, mut choices): (Vec<Entry>, Vec<Entry>) =
        blog_file.entries.into_iter().partition(|e| !e.published);
    let i = display_choices(&choices)?;

    let path = format!("blog/{}.html", choices[i].id);
    fs::write(
        &format!("blog/drafts/{}.html", choices[i].name),
        get_inner(PathBuf::from(format!("blog/{}.html", choices[i].id)))?,
    )?;
    fs::remove_file(path)?;
    remove_xml(blog_file.config_dir.join(config.rss), &choices[i])?;

    choices[i].published = false;
    choices.extend(list);
    blog_file.entries = choices.clone();
    fs::write("blog/.config.json", serde_json::to_string(&blog_file)?)?;

    Ok(())
}

fn regen(blog_file: BlogFile, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let choices = blog_file
        .entries
        .into_iter()
        .filter(|e| e.published)
        .collect::<Vec<Entry>>();

    for entry in choices {
        insert_xml(
            &blog_file.config_dir.join(&config.template),
            &config,
            &entry,
            str::from_utf8(&get_inner(PathBuf::from(format!(
                "blog/{}.html",
                entry.id
            )))?)?,
            Flag::Template,
        )?;
    }
    Ok(())
}

/// Get user input
fn read_input() -> Result<String, io::Error> {
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.replace("\n", ""))
}

/// Display a set of entries and get user to choose one
fn display_choices(blog_file: &[Entry]) -> Result<usize, Box<dyn std::error::Error>> {
    let input: usize;
    for (i, e) in blog_file.iter().enumerate() {
        println!("{}. {}", i + 1, e.name);
    }
    loop {
        match read_input()?.parse::<usize>() {
            Ok(n) => {
                if n - 1 < blog_file.len() {
                    input = n - 1;
                    break;
                } else {
                    println!("Input was not an option.")
                }
            }
            _ => println!("Input was not a number.",),
        }
    }
    Ok(input)
}

/// Fully escape any XML string
fn xml_escape(s: &str) -> String {
    s.replace('\"', "&quot;")
        .replace('\'', "&apos;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('&', "&amp;")
}

/// Clear the terminal
fn clear() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}
