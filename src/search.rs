use std::ffi::{CString, CStr};
use std::os::raw::{c_char};


use serde::{Deserialize, Serialize};
use serde_json::{ from_str};
use reqwest::header::{HeaderMap, HeaderValue, REFERER, HOST};
use visdom::Vis;

use urlencoding::encode;

use crate::{SOURCE_HOST, SOURCE_REFERER};





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
}

#[derive(Serialize, Deserialize)]
struct Arguments {
    search: String,
    page: isize
}


#[unsafe(no_mangle)]
pub extern "C" fn search(
    arguments_ptr : *const c_char,
) -> *const c_char {

    let mut return_result = ReturnResult {
        status: false,
        message: String::from(""),
        data: Vec::new(),
    };

    // Check argument before processing
    let mut valid_arguments: bool = true;
    if arguments_ptr.is_null() {
        return_result.message = String::from("Expected 1 argument.");
        valid_arguments = false;
    }

    let mut args: Arguments = Arguments { 
        search: String::from(""),
        page: 1
    };

    if valid_arguments {
        unsafe { 
            match from_str::<Arguments>(&CStr::from_ptr(arguments_ptr as *mut c_char).to_string_lossy().into_owned()) {
                Ok(result) => {
                    args.search = result.search;
                    args.page = result.page
                },
                Err(e) => {
                    return_result.message = String::from(format!("Invalid arguments: {}", e.to_string()));
                    valid_arguments = false;
                }
            }
        };
    }
    // ================================================

    if valid_arguments {

        let search_string = args.search;
        let page_number = args.page;

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
                match node.find(".film-poster").find("img").attr("data-src"){
                    Some(result) => {
                        cover = result.to_string();
                    }
                    _ => {}
                }
                
                match node.find(".film-poster").find("a").attr("data-id") {
                    Some(result) => {
                        id = result.to_string();
                    }
                    _ => {}
                }
                let detail_node = node.find(".film-detail").find(".film-name").find("a");
                let title = detail_node.text();

                if cover.is_empty() || id.is_empty() || title.is_empty() {
                    continue;
                }

                let return_data = ReturnData {
                    id,
                    title,
                    cover
                };
                return_result.data.push(return_data);
            }
            return_result.status = true;
        }


    }
    
    let result = CString::new(serde_json::to_string(&return_result).unwrap()).unwrap();
    let result_ptr = result.as_ptr();
    std::mem::forget(result); // prevent Rust from freeing it
    return result_ptr;
}