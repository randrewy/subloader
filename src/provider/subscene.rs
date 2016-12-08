use select::document::Document;
use select::predicate::{Attr, Name, And};

use provider::{Provider, Downloadable};

#[derive(Debug)]
struct SubsceneDownloadable {
    pub name: String,
    pub url: String,
    pub lang: String
}

pub struct SubsceneProvider {}

fn subscene_abs(relative: &str) -> String {
    let mut url: String = "https://subscene.com".to_owned();
    url.push_str(&relative);
    url
}

fn get_download_url(document: &Document) -> Option<String> {
    if let Some(url) = document.find(And(Name("a"), Attr("id", "downloadButton"))).first() {
        return Some(url.attr("href").unwrap_or("").trim().to_owned())
    }
    None
}

fn search_on_page (document: &Document, language: &str) -> Vec<Box<Downloadable>> {
    let mut subtitles: Vec<Box<Downloadable>> = Vec::new();

    for node in document.find(And(Name("td"), Attr("class", "a1"))).iter() {
        if let (Some(lang), Some(name), Some(url)) = (
            node.find(And(Name("span"), Attr("class", ()))).first(),
            node.find(And(Name("span"), Attr("class", ()))).next().next().first(),
            node.find(And(Name("a"), Attr("href", ()))).first()) {
            if lang.text().trim().to_owned() == language {
                subtitles.push(Box::new(SubsceneDownloadable {
                    name: name.text().trim().to_owned(),
                    url: url.attr("href").unwrap_or("").trim().to_owned(),
                    lang: lang.text().trim().to_owned()
                })
                );
            }
        }
    }
    subtitles
}


impl Downloadable for SubsceneDownloadable {
    fn name(&self) -> &str {
        &self.name
    }

    fn lang(&self) -> &str {
        &self.lang
    }

    fn download(&self) -> Vec<u8> {
        let dl_document = ::utils::get_document(&subscene_abs(&self.url));
        if let Some(ref url) =  get_download_url(&dl_document) {
            let resp = ::utils::read_bytes(&subscene_abs(&url));
            let sub = ::utils::unzip_first_sub(&resp.unwrap());
            return sub;
        }
        Vec::new()
    }

    fn dbg(&self) {
        println!("{:?}", &self);
    }
}

impl Provider for SubsceneProvider{
    fn search(&self, name: &str, lang: &str) -> Vec<Box<Downloadable>> {
        let mut query = "https://subscene.com/subtitles/release?q=".to_owned();
        query += name;
        query += "&l=";

        let document = ::utils::get_document(&query);
        search_on_page(&document, lang)
    }
}