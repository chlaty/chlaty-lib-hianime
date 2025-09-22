use std::collections::HashMap;
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
    id: String,
    title: String
}

#[derive(Debug, Serialize, Deserialize)]
struct ReturnResult {
    status: bool,
    message: String,
    data: HashMap<String, Vec<ReturnData>>,
}

#[derive(Serialize, Deserialize)]
struct Arguments {
    id: String
}

#[unsafe(no_mangle)]
pub extern "C" fn get_episode_server(
    arguments_ptr : *const c_char,
) -> *const c_char {

    let mut return_result = ReturnResult {
        status: false,
        message: String::from(""),
        data: HashMap::new(),
        
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

        // https://hianime.to/ajax/v2/episode/servers?episodeId=141568
        let url = format!("https://{}/ajax/v2/episode/servers?episodeId={}", 
            SOURCE_HOST, encode(&id)
        );
        
        let res = client.get(&url).headers(headers).send().unwrap();
        
        if res.status().is_success(){
            let data = res.json::<Value>().unwrap_or(Value::Null);

            let root = Vis::load(data.get("html").unwrap_or(&Value::Null).as_str().unwrap_or("")).unwrap();

            let mut server_type: HashMap<String, Vec<ReturnData>> = HashMap::new();

            for server_type_ele in root.find(".ps_-block") {
                let server_type_node = Vis::dom(&server_type_ele);
                let server_type_title = server_type_node.find(".ps__-title").text().replace(":", "");

                let mut server_list_per_type: Vec<ReturnData> = Vec::new();
                

                for server_ele in server_type_node.find(".ps__-list").find(".server-item") {
                    let server_ele_node = Vis::dom(&server_ele);

                    let server_id = server_ele_node.attr("data-id");
                    if server_id.is_none() {
                        continue;
                    }

                    let server_title = server_ele_node.find("a").text();

                    server_list_per_type.push(ReturnData {
                        id: server_id.unwrap().to_string(),
                        title: server_title
                    });
                }
                server_type.insert(server_type_title, server_list_per_type);

            }
            return_result.data = server_type;
            return_result.status = true;
        }
    }
    
    let json_string = serde_json::to_string(&return_result).unwrap();
    let result = CString::new(json_string).unwrap();
    return result.into_raw();
}