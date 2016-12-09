extern crate regex;
use regex::Regex;

use std::path::{Path, PathBuf};
use std::ffi::OsStr;

extern crate sub_searcher;
use sub_searcher::utils;
use sub_searcher::provider::*;

extern crate toml;
extern crate rustc_serialize;

#[derive(RustcDecodable)]
struct Config {
    movie_folder: String,
    movie_ext: Vec<String>,
    max_results: usize,
}

#[derive(Debug)]
pub struct Video {
    subs: bool,
    path: Box<PathBuf>,
}

impl Video {
    pub fn name(&self) -> String {
        self.path.file_name().unwrap_or(&OsStr::new("")).to_str().unwrap_or("").to_string()
    }

    pub fn path(&self) -> &str {
        self.path.as_path().to_str().unwrap()
    }
}

fn is_video(path: &Path, extensions: &[String]) -> bool {
    extensions.contains(&path.extension().unwrap_or(&OsStr::new("")).to_str().unwrap_or("").to_owned())
}

fn has_downloded_subs(path: &Path) -> bool {
    Path::new(&make_sub_path(&path)).exists()
}

fn make_sub_name(path: &Path) -> String {
    path.file_stem().unwrap_or(&OsStr::new("")).to_str().unwrap_or("").to_string() + ".srt"
}

fn make_sub_path(path: &Path) -> PathBuf {
    let dir = path.parent().unwrap_or(Path::new(""));
    dir.join(&make_sub_name(&path))
}

fn extract_name(file: &str) -> String {
    // TODO: lazy static
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
        .unwrap_or(&OsStr::new(""))
        .to_str()
        .unwrap_or("")
        .to_string()
}

fn get_search_query(name: &str) -> String {
    extract_name(name).replace(".", "+").replace(" ", "+")
}

fn collect_videos(dir: &str, video_extensions: &[String]) -> Vec<Video> {
    let mut result = Vec::new();
    if let Ok(paths) = std::path::Path::read_dir(std::path::Path::new(dir)) {
        for path in paths {
            if let Ok(p) = path {
                if p.path().is_dir() {
                    if let Some(str_dir) = p.path().to_str() {
                        result.append(&mut collect_videos(str_dir, video_extensions));
                    } else {
                        println!("Stringify failed on {:?}", p.path());
                    }
                } else if is_video(&p.path(), video_extensions) {
                    result.push(Video {
                        subs: has_downloded_subs(&p.path()),
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

fn print_names(vids: &[Video]) {
    for (i, v) in vids.iter()
        .enumerate() {
        println!("{}: {}", i, v.name());
    }
}

fn read_index(max_value: usize) -> usize {
    let mut input = String::new();
    println!("Please input index");
    std::io::stdin().read_line(&mut input).expect("failed to read index");
    let idx = input.trim().parse().expect("Please type a number!");
    if idx >= max_value {
        panic!("index out of range")
    }
    idx
}

fn get_providers() -> Vec<Box<Provider>> {
    vec![
        Box::new(subscene::SubsceneProvider{})
    ]
}

fn search(providers: &[Box<Provider>], name: &str, lang: &str) -> Vec<Box<Downloadable>> {
    let mut result:Vec<Box<Downloadable>> = Vec::new();
    for provider in providers.iter() {
        if provider.accepts_whole_name() {
            result.append(&mut provider.search(name, lang));
        } else {
            result.append(&mut provider.search(&get_search_query(name), lang));
        }

    }
    result
}


/// example of config.toml:
/// movie_folder = "~/Downloads"
/// movie_ext = [ "avi", "mkv" ]
/// max_results = 10
fn main() {
    let mut config_path = std::env::current_exe().unwrap();
    config_path.pop();
    config_path.push(Path::new("config.toml"));

    let config: Config;
    if let Ok(toml_str) = utils::open_file_to_str(config_path.to_str().unwrap()) {
        config = toml::decode_str(&toml_str).unwrap();
    } else {
        panic!("config file read failed");
    }

    let vids = collect_videos(&config.movie_folder, &config.movie_ext);
    print_names(&vids);
    let index = read_index(vids.len());
    let video = &vids[index];
    println!("selected {}", &video.name());

    let providers = get_providers();
    let search_result = search(&providers, &video.name(), "English");
    for (i, v) in search_result.iter()
        .enumerate()
        .take(config.max_results) {
        println!("{}: {}", i, v.name());
    }
    let index = read_index(search_result.len());
    let sub = search_result[index].download();
    if let Ok(_) = utils::write_file(make_sub_path(&video.path.as_path()).to_str().unwrap(), &sub) {
        println!("downloaded successfully");
    }
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