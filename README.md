# OB - Oliver's Blog Script
A Blog and RSS system written in Rust.

## Features

- 

## Installation

Install Rust:
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install `ob`:
```shell
cargo install ob
```

## Setup

You will need to create four files:

A blog index file where the blog entries will be listed.

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

A template to be filled out with the content of a blog post.

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

An RSS feed.

```xml
<?xml version="1.0" encoding="utf-8"?>
<?xml-stylesheet type="text/css" href="rss.css" ?>
<rss version="2.0"
    xmlns:atom="http://www.w3.org/2005/Atom">

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

A configuration file containing the paths to your blog index, template and rss files.
It should also include the address of where the blog entries will be hosted and whether to include images or not (true/false).

```json
{
    "blog": "index.html",
    "rss": "rss.xml",
    "template": "template.html",
    "blog_address": "https://your_blog.com/blog/",
    "images": true
}
```

## Usage

