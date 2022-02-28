use std::env::args;
use std::error::Error;
use std::fs::{remove_dir_all, DirBuilder, File};
use std::io::prelude::*;
use std::process::Command;

use rayon::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    gen(
        &args().nth(1).ok_or("YOU GOTTA GIVE ME AN IN PATH MAN")?,
        &args().nth(2).ok_or("YOU GOTTA GIVE ME AN OUT PATH MAN")?,
    )
}

const TEMPLATE: &str = include_str!("article.dumb_template");

fn gen_html(raw: &str, title: &str) -> String {
    let content = markdown::to_html(raw)
        .replace("&lt;&lt;", "<img src=\"../")
        .replace("&gt;&gt;", "\">");

    TEMPLATE
        .replace("{{ TITLE }}", title)
        .replace("{{ CONTENT }}", &content)
}

fn deal_with_dir(in_path: &str, article_path: &str, out_path: &str) -> Result<(), Box<dyn Error>> {
    let mut content = File::open(format!("{in_path}/{article_path}.md"))?;
    let mut raw = String::new();
    content.read_to_string(&mut raw)?;
    let html = gen_html(&raw, article_path);

    let dir = DirBuilder::new();
    dir.create(format!("{out_path}/{article_path}"))?;

    File::create(format!("{out_path}/{article_path}/index.html"))?.write_all(html.as_bytes())?;

    Ok(())
}

const CSS: &[u8] = include_str!("style.css").as_bytes();
const LIST_TEMPLATE: &str = include_str!("list.dumb_template");

fn gen(in_path: &str, out_path: &str) -> Result<(), Box<dyn Error>> {
    let articles = String::from_utf8(Command::new("ls").arg("-t").arg(in_path).output()?.stdout)?;

    remove_dir_all(out_path)?;
    let dir = DirBuilder::new();
    dir.create(out_path)?;

    File::create(format!("{out_path}/style.css"))?.write_all(CSS)?;

    let list = LIST_TEMPLATE.replace(
        "{{ CONTENT }}",
        &format!(
            "<ul>{}</ul>",
            articles
                .lines()
                .filter_map(|file| {
                    if file.ends_with(".md") {
                        let article = file.replace(".md", "");
                        Some(format!("<li><a href=\"{article}\">{article}</a></li>"))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        ),
    );
    File::create(format!("{out_path}/index.html"))?.write_all(list.as_bytes())?;

    articles
        .lines()
        .par_bridge()
        .into_par_iter()
        .for_each(|dir| {
            if dir.ends_with(".md") {
                deal_with_dir(in_path, &dir.replace(".md", ""), out_path).unwrap();
            } else {
                Command::new("cp")
                    .arg(format!("{in_path}/{dir}"))
                    .arg(out_path)
                    .output()
                    .unwrap();
            }
        });

    Ok(())
}
