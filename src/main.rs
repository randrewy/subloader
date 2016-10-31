extern crate xmlrpc;
extern crate hyper;
extern crate zip;
extern crate regex;

use hyper::Client;
use std::fs::{File};
use std::path::{Path, PathBuf};
use std::io::{Read, Result, Write};
use regex::Regex;

extern crate select;
use select::document::Document;
use select::predicate::{Attr, Name, And};


fn write_file(fname: &str, data: &Vec<u8>) -> Result<()> {
    let mut f = File::create(fname).unwrap();
    f.write_all(data).expect("Unable to write data");
    Ok(())
}

#[allow(dead_code)]
fn stub_opensubtitles() {
    let client = Client::new();
    let pow_request = xmlrpc::Request::new("ServerInfo");
    let request_result = pow_request.call(&client, "http://api.opensubtitles.org:80/xml-rpc");

    println!("Result: {:?}", request_result);
}

#[derive(Debug)]
pub struct Subtitle {
    name: String,
    url: String,
    lang: String
}

fn get_document(url: &str) -> Document {
    let client = Client::new();
    let mut response = client.get(url)
        .send()
        .unwrap();

    let mut content = String::new();
    match response.read_to_string(&mut content) {
        Ok(_) => Document::from(content.as_str()),
        Err(e) => panic!("Request failed {}", e),
    }
}

fn read_bytes(url: &str) -> Result<Vec<u8>> {
    let client = Client::new();
    let mut response = client.get(url)
        .send()
        .unwrap();

    let mut byte_vec = Vec::new();
    match response.read_to_end(&mut byte_vec) {
        Ok(_) => Ok(byte_vec),
        Err(e) => panic!("Request failed. {}", e),
    }
}

fn unzip_to_subs(buf: &Vec<u8>) -> Vec<Vec<u8>> {
    let mut subtitles: Vec<Vec<u8>> = Vec::new();

    let mut reader = std::io::Cursor::new(buf);
    if let Ok(mut zip) = zip::ZipArchive::new(reader) {
        for i in 0..zip.len() {
            let mut file = zip.by_index(i).unwrap();

            if file.name().ends_with(".srt") {
                let sub = file.bytes().map(|b| b.unwrap()).collect();
                subtitles.push(sub);
            }
        }
    }

    subtitles
}

fn extract_subs(document: &Document, language: &str) -> Vec<Subtitle> {
    let mut subtitles: Vec<Subtitle> = Vec::new();

    for node in document.find(And(Name("td"), Attr("class", "a1"))).iter() {
        if let (Some(lang), Some(name), Some(url)) = (
            node.find(And(Name("span"), Attr("class", ()))).first(),
            node.find(And(Name("span"), Attr("class", ()))).next().next().first(),
            node.find(And(Name("a"), Attr("href", ()))).first()) {

            if lang.text().trim().to_string() == language {
                subtitles.push(Subtitle {
                    name:name.text().trim().to_string(),
                    url:url.attr("href").unwrap_or("").trim().to_string(),
                    lang:lang.text().trim().to_string()}
                );
            }
        }
    }
    subtitles
}

fn get_download_url(document: &Document) -> Option<String> {
    if let Some(url) = document.find(And(Name("a"), Attr("id", "downloadButton"))).first() {
        return Some(url.attr("href").unwrap_or("").trim().to_string())
    }
    None
}

// TODO: return slice
fn get_top_match(subtitles: &Vec<Subtitle>) -> Option<&Subtitle> {
    subtitles.iter().nth(0)
}

fn subscene_abs(relative: &str) -> String {
    let mut url: String = "https://subscene.com".to_owned();
    url.push_str(&relative);
    url
}

fn stub_subscene(name: &str, language: &str) {
    let mut query = "https://subscene.com/subtitles/release?q=".to_string();
    query += name;
    query += "&l=";

    let document = get_document(&query);
    let subtitles = extract_subs(&document, language);

    if let Some(ref best) = get_top_match(&subtitles) {
        let dl_document = get_document(&subscene_abs(&best.url));
        if let Some(ref url) =  get_download_url(&dl_document) {
            let resp = read_bytes(&subscene_abs(&url));
            let subs = unzip_to_subs(&resp.unwrap());

            for (i, ref s) in subs.iter().enumerate() {
                write_file(&i.to_string(), &s);
            }

        }
    }
}

fn is_video(path: &Path) -> bool {
    let extensions = vec!["avi",  "mkv"];
    extensions.contains(&path.extension().unwrap_or(&std::ffi::OsStr::new("")).to_str().unwrap())
}

fn have_downloded_subs(path: &Path) -> bool {
    Path::new(&make_sub_path(&path)).exists()
}

fn make_sub_name(path: &Path) -> String {
    path.file_stem().unwrap().to_str().unwrap().to_string() + ".srt"
}

fn make_sub_path(path: &Path) -> PathBuf {
    let dir = path.parent().unwrap_or(Path::new(""));
    dir.join(&make_sub_name(&path))
}

fn print_files(dir: &str) {
    let paths = std::path::Path::read_dir(std::path::Path::new(dir)).unwrap();

    for path in paths {
        let p = path.unwrap().path();
        if p.is_dir() {
            print_files(&p.to_str().unwrap());
        } else if is_video(&p) {
            println!("Name: {}", p.display());
        }
    }
}

fn extract_name(file: &str) -> String {
    {
        let re = Regex::new(r"(.+?\.)([Ss]\d+\.?[Ee]\d+)").unwrap();
        for caps in re.captures_iter(file) {
            return caps.at(1).unwrap().to_string() + caps.at(2).unwrap();
        }
    }
    {
        let re = Regex::new(r"(.+?\.)(\d+[Xx]\d+)").unwrap();
        for caps in re.captures_iter(file) {
            return caps.at(1).unwrap().to_string() + caps.at(2).unwrap();
        }
    }
    Path::new(file)
        .file_stem()
        .unwrap_or(&std::ffi::OsStr::new(""))
        .to_str()
        .unwrap_or("")
        .to_string()
}

fn get_search_query(file: &str) -> String {
    extract_name(file).replace(".", "+").replace(" ", "+")
}

#[derive(Debug)]
pub struct Video {
    subs: bool,
    path: Box<std::path::PathBuf>,
}

impl Video {
    pub fn name(&self) -> String {
        self.path.file_name().unwrap_or(&std::ffi::OsStr::new("")).to_str().unwrap_or("").to_string()
    }
}

fn collect_videos(dir: &str) -> Vec<Video> {
    let mut result = Vec::new();
    if let Ok(paths) = std::path::Path::read_dir(std::path::Path::new(dir)) {
        for path in paths {
            if let Ok(p) = path {
                if p.path().is_dir() {
                    if let Some(str_dir) = p.path().to_str() {
                        result.append(&mut collect_videos(str_dir));
                    } else {
                        println!("Stringify failed on {:?}", p.path());
                    }
                } else if is_video(&p.path()) {
                    result.push(Video {
                        subs: have_downloded_subs(&p.path()),
                        path: Box::from(p.path()),
                    });
                }
            } else {
                println!("Path problem");
            }
        }
    } else {
        println!("Paths problem {}", &dir);
    }
    result
}




fn main() {

    let vids = collect_videos("D:/Downloads");
    for (i, v) in vids.iter()
        .enumerate()
        .filter(|x| !x.1.subs) {
        println!("{}: {}", i, v.name());
    }

    let mut input = String::new();
    println!("Please input video index");
    std::io::stdin().read_line(&mut input).expect("failed to read index");
    let index: usize = input.trim().parse().expect("Please type a number!");

    let video = &vids[index];
    println!("selected {:?}", &video);
    println!("query is {}", get_search_query(&video.name()));

    println!("{:?}", &stub_subscene(&get_search_query(&video.name()), "English"));
}


#[cfg(test)]
mod tests {
    use super::get_search_query;

    #[test]
    fn test_get_search_query() {
        assert_eq!("Fargo.S01E10", get_search_query("Fargo.S01E10.Morton's.Fork.720p.[rofl].mkv"));
        assert_eq!("Fargo.1X09", get_search_query("Fargo.1X09.A.Fox.A.Ra..."));
        assert_eq!("Fargo The Heap", get_search_query("Fargo The Heap.avi"));
    }

}