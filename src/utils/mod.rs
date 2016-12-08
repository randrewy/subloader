use std::io::{Result, Read, Write, Cursor};
use std::fs::File;

use hyper::Client;
use select::document::Document;
use zip::ZipArchive;

pub fn write_file(fname: &str, data: &Vec<u8>) -> Result<()> {
    let mut f = File::create(fname).unwrap();
    f.write_all(data).expect("Unable to write data");
    Ok(())
}

pub fn get_document(url: &str) -> Document {
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

pub fn read_bytes(url: &str) -> Result<Vec<u8>> {
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

pub fn unzip_to_subs(buf: &Vec<u8>) -> Vec<Vec<u8>> {
    let mut subtitles: Vec<Vec<u8>> = Vec::new();

    let reader = Cursor::new(buf);
    if let Ok(mut zip) = ZipArchive::new(reader) {
        for i in 0..zip.len() {
            let file = zip.by_index(i).unwrap();

            if file.name().ends_with(".srt") {
                let sub = file.bytes().map(|b| b.unwrap()).collect();
                subtitles.push(sub);
            }
        }
    }
    subtitles
}

pub fn unzip_first_sub(buf: &Vec<u8>) -> Vec<u8> {
    let reader = Cursor::new(buf);
    if let Ok(mut zip) = ZipArchive::new(reader) {
        for i in 0..zip.len() {
            let file = zip.by_index(i).unwrap();

            if file.name().ends_with(".srt") {
                return file.bytes().map(|b| b.unwrap()).collect();
            }
        }
    }
    Vec::new()
}
