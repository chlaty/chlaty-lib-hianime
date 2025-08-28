use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};


use serde::{Deserialize, Serialize};
use serde_json::{Value};
use reqwest::header::{HeaderMap, HeaderValue, REFERER, HOST};
use visdom::Vis;
use visdom::types::{BoxDynError};

use std::collections::hash_map;
use urlencoding::encode;

use crate::{SERVER_HOST, SERVER_REFERER, SOURCE_HOST, SOURCE_REFERER};



#[derive(Debug, Serialize, Deserialize)]
struct ReturnConfig {
    host: String,
    referer: String,
    playlist_base_url: String
}

#[derive(Debug, Serialize, Deserialize)]
struct ReturnData{
    id: String,
    title: String,
    cover: String
}

#[derive(Debug, Serialize, Deserialize)]
struct ReturnResult {
    status: bool,
    message: String,
    data: Vec<ReturnData>,
    config: ReturnConfig,
}

#[unsafe(no_mangle)]
pub extern "C" fn search(
    search_ptr : *const c_char,
    page_ptr: *const c_int,
) -> *const c_char {

    let mut return_result = ReturnResult {
        status: false,
        message: String::from(""),
        data: Vec::new(),
        config: ReturnConfig {
            host: SERVER_HOST.to_string(),
            referer: SERVER_REFERER.to_string(),
            playlist_base_url: String::from("")
        }
    };

    // Check argument before processing
    let mut valid_argument: bool = true;
    if page_ptr.is_null() {
        return_result.message = String::from("'page' is required.");
        valid_argument = false;
    }

    if search_ptr.is_null() {
        return_result.message = String::from("'search' is required.");
        valid_argument = false;
    }
    // ================================================

    if valid_argument {
        let search_string = unsafe { CStr::from_ptr(search_ptr as *mut c_char).to_string_lossy().into_owned() };
        let page_number = unsafe { *page_ptr.clone() as isize };

        let client = reqwest::blocking::Client::new();
        let mut headers = HeaderMap::new();
        headers.insert(REFERER, HeaderValue::from_str(SOURCE_REFERER).unwrap());
        headers.insert(HOST, HeaderValue::from_str(SOURCE_HOST).unwrap());

        let url = format!("https://{}/search?keyword={}&page={}", 
            SOURCE_HOST, 
            if search_string.trim().is_empty() { "+" } else { &encode(&search_string) }, 
            encode(&page_number.to_string())
        );
        println!("url: {}", &url);
        let res = client.get(&url).headers(headers).send().unwrap();
        
        if res.status().is_success(){
            let html = res.text().unwrap();
            // println!("{}", &html);
            let root = Vis::load(html).unwrap();


            for ele in root.find(".flw-item") {
                let node = Vis::dom(&ele);
                let mut id = String::new();
                let mut cover = String::new();
                let mut title = String::new();
                match node.find(".film-poster").find("img").attr("data-src"){
                    Some(result) => {
                        cover = result.to_string();
                    }
                    _ => {
                        continue;
                    }
                }
                let detail_node = node.find(".film-detail").find(".film-name").find("a");
                match detail_node.attr("href") {
                    Some(result) => {
                        id = result.to_string().split("/").last().unwrap().to_string().split("?").nth(0).unwrap().to_string();
                    }
                    _ => {
                        continue;
                    }
                }

                title = detail_node.text();
                
                let return_data = ReturnData {
                    id: id,
                    title: title,
                    cover: cover
                };
                return_result.data.push(return_data);
            }
        }


    }
    
    let result = CString::new(serde_json::to_string(&return_result).unwrap()).unwrap();
    let result_ptr = result.as_ptr();
    std::mem::forget(result); // prevent Rust from freeing it
    return result_ptr;
}