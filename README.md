<br/>
<img src="https://image.flaticon.com/icons/png/512/4229/4229823.png" height="200" width="200"/>

# OB - Oliver's Blog Script
**A Blog and RSS system written in Rust.**

## Features

- Converts blog entries written in Markdown into HTML.   ✍🏻
- Keeps a rolling blog page.   🔎
- Keeps an RSS feed which includes blog posts in full.   📰
- Creates entries in the rolling blog page that are easily modifiable with CSS.   ⚡️
- One command to delete entries from the RSS feed, rolling blog and standalone entries simultaneously.   🚀
- Works on MacOS, Linux and Windows.   🖥
- Less than 350 lines of code.   🏝
- Blazingly fast.   🔥

## Installation

Install Rust:

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install OB:

```shell
cargo install ob
```

## Setup

You will need to create four files:

- A Rolling Blog File where the blog entries will be listed.

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <title>Your Blog</title>
    <meta charset="utf-8"/>
  </head>
  <body>
    <h1>Blog Updates</h1>
    <ul>
      <!-- OB -->
    </ul>
  </body>
</html>
```

- A Template to be filled out with the content of a blog post.

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <title></title>
    <meta charset="utf-8"/>
  </head>
  <body>
    <!-- OB -->
  </body>
</html>
```

- An RSS feed.

```xml
<?xml version="1.0" encoding="utf-8"?>
<?xml-stylesheet type="text/css" href="rss.css" ?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>Blog Title</title>
    <description>Description</description>
    <language>en-us</language>
    <link>https://your_blog.com/rss.xml</link>
    <atom:link href="https://your_blog.com/rss.xml" rel="self" type="application/rss+xml" />

    <!-- OB -->
  </channel>
</rss>
```

- A configuration file containing the paths to your blog index, template and rss files.   
It should also include the address of where the blog entries will be hosted, the maximum number 
of posts on the rss feed and whether to include images or not.

```json
{
    "blog": "index.html",
    "template": "template.html",
    "rss": "rss.xml",
    "items": 4,
    "blog_address": "https://your_blog.com/blog/",
    "images": true
}
```

## Markers

For the system to work, add the following comment line to the Rolling Blog File, the Template and RSS feed (as above).

```html
<!-- OB -->
```

When you publish a blog post, it will be added directly below that line in the proper format.

## Usage

```
USAGE:
    ob [FLAGS]

FLAGS:
    -d, --delete     Delete a draft
    -h, --help       Prints help information
    -n, --new        Create a new draft
    -p, --publish    Publish a daft
    -V, --version    Prints version information
```

The first time `ob` is used it will create a folder at: `/blog`.

When you create a new draft it will be located at: `/blog/drafts`.  
When you publish a new draft it will be located at: `/blog/example.html`.

**Example usage:**

```shell
ob --new
```

<br>

### You can see an example on [my blog](https://oliverbrotchie.github.io/) or [OB's website](https://oliverbrotchie.github.io/ob/) located in the `/docs` folder.
