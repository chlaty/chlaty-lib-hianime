use std::ffi::{CString, CStr};
use std::os::raw::{c_char};

use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str};
use reqwest::header::{HeaderMap, HeaderValue, REFERER, HOST};
use visdom::Vis;
use urlencoding::encode;

use crate::{ SOURCE_HOST, SOURCE_REFERER};




#[derive(Debug, Serialize, Deserialize)]
struct ReturnData{
    index: usize,
    id: String,
    title: String
}

#[derive(Debug, Serialize, Deserialize)]
struct ReturnResult {
    status: bool,
    message: String,
    data: Vec<Vec<ReturnData>>,
}

#[derive(Serialize, Deserialize)]
struct Arguments {
    id: String
}

#[unsafe(no_mangle)]
pub extern "C" fn get_episode_list(
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
        id: String::from("") 
    };
    if valid_arguments {
        unsafe { 
            match from_str::<Arguments>(&CStr::from_ptr(arguments_ptr as *mut c_char).to_string_lossy().into_owned()) {
                Ok(result) => {
                    args.id = result.id
                },
                Err(e) => {
                    return_result.message = String::from(e.to_string());
                    valid_arguments = false;
                }
            }
        };
    }
    // ================================================

    if valid_arguments {

        let id = args.id;

        let client = reqwest::blocking::Client::new();
        let mut headers = HeaderMap::new();
        headers.insert(REFERER, HeaderValue::from_str(SOURCE_REFERER).unwrap());
        headers.insert(HOST, HeaderValue::from_str(SOURCE_HOST).unwrap());

        // https://hianime.to/ajax/v2/episode/list/100
        let url = format!("https://{}/ajax/v2/episode/list/{}", 
            SOURCE_HOST, encode(&id)
        );
        
        let res = client.get(&url).headers(headers).send().unwrap();
        
        if res.status().is_success(){
            let data = res.json::<Value>().unwrap_or(Value::Null);

            let root = Vis::load(data.get("html").unwrap_or(&Value::Null).as_str().unwrap_or("")).unwrap();

            let mut episode_page_list: Vec<Vec<ReturnData>> = Vec::new();

            for ep_page_ele in root.find(".ss-list") {
                let mut episode_per_page: Vec<ReturnData> = Vec::new();
                let ep_page_node = Vis::dom(&ep_page_ele);

                for ep_ele in ep_page_node.find(".ssl-item ")  {
                    let ep_node = Vis::dom(&ep_ele);

                    let mut index:usize = 0;
                    match ep_node.attr("data-number") {
                        Some(result) => {
                            index = result.to_string().parse().unwrap();
                        },
                        None => {}
                    }

                    let mut id = String::from("");
                    match ep_node.attr("data-id") {
                        Some(result) => {
                            id = result.to_string();
                        },
                        None => {}
                    }

                    let mut title = String::from("");
                    match ep_node.attr("title") {
                        Some(result) => {
                            title = result.to_string();
                        },
                        None => {}
                    }
                
                    episode_per_page.push(ReturnData {
                        index,
                        id,
                        title,
                    });
                }

                episode_page_list.push(episode_per_page);
            }

            return_result.data = episode_page_list;
            return_result.status = true;
        }
    }
    
    let result = CString::new(serde_json::to_string(&return_result).unwrap()).unwrap();
    let result_ptr = result.as_ptr();
    std::mem::forget(result); // prevent Rust from freeing it
    return result_ptr;
}