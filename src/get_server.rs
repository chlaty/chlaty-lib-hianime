use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::panic;
use std::ptr;


use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str};
use reqwest::header::{HeaderMap, HeaderValue, REFERER, HOST};
use visdom::Vis;
use regex::Regex;
use visdom::types::{BoxDynError, BoxDynElement};
use url::Url;

use crate::{SERVER_HOST, SERVER_ORIGIN, SERVER_REFERER, SOURCE_HOST, SOURCE_REFERER};


#[derive(Serialize, Deserialize)]
struct ReturnConfig {
    host: String,
    referer: String,
    origin: String,
    playlist_base_url: String,
    segment_base_url: String
}

#[derive(Serialize, Deserialize)]
struct ReturnResult {
    status: bool,
    message: String,
    data: Value,
    config: ReturnConfig,
}

#[derive(Serialize, Deserialize)]
struct Arguments {
    id: String
}

#[unsafe(no_mangle)]
pub extern "C" fn get_server(
    arguments_ptr: *const c_char,
) -> *const c_char {
    let result = panic::catch_unwind(|| {
        let mut return_result = ReturnResult {
            status: false,
            message: String::from(""),
            data: Value::Null,
            config: ReturnConfig {
                host: String::from(""),
                referer: SERVER_REFERER.to_string(),
                origin: SERVER_ORIGIN.to_string(),
                playlist_base_url: String::from(""),
                segment_base_url: String::from("")
            }
        };

        // Check argument before processing
        let mut valid_arguments: bool = true;
        if arguments_ptr.is_null() {
            return_result.message = String::from("Expected 1 argument.");
            valid_arguments = false;
        }
        
        let mut args: Arguments = Arguments { id: String::from("") };
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
            
            let server_id = args.id;

            let client = reqwest::blocking::Client::new();


            // Get forward episode id
            let mut forward_server_id: Option<String> = None;
            let mut headers = HeaderMap::new();

            headers.insert(REFERER, HeaderValue::from_str(&SOURCE_REFERER).unwrap());
            headers.insert(HOST, HeaderValue::from_str(&SOURCE_HOST).unwrap());
            
            let url = format!("https://{}/ajax/v2/episode/sources?id={}", &SOURCE_HOST, &server_id);
            let res = client.get(url)
                .headers(headers.clone())
                .send().unwrap();

            if res.status().is_success() {
                let data = res.json::<Value>();
                match data {
                    Ok(result) => {
                        forward_server_id = Some(result.get("link").unwrap().to_string()
                            .split("/").last().unwrap().split("?").nth(0).unwrap().to_string());
                    }
                    _ => {
                        return_result.status = false;
                        return_result.message = String::from("Failed to get forward episode id.");
                        println!("[Failed] Failed to get forward episode id.");
                    }
                }
            }

            // ================================================



            // Get html dom then extract token.
            // println!("Forward episode id: {:?}", forward_server_id.as_ref().clone());
            let mut token: Option<String> = None;
            if forward_server_id.is_some() {
                let mut headers = HeaderMap::new();

                headers.insert(REFERER, HeaderValue::from_str(&SERVER_REFERER).unwrap());
                headers.insert(HOST, HeaderValue::from_str(&SERVER_HOST).unwrap());
                
                let url = format!("https://{}/embed-2/v3/e-1/{}?k=1", &SERVER_HOST, forward_server_id.as_ref().unwrap());

                let res = client.get(url)
                    .headers(headers.clone())
                    .send().unwrap();

                if res.status().is_success() {
                    let body = res.text();
                    // println!("Body: {}", body.as_ref().unwrap());
                    match extract_key_token(body.as_ref().unwrap()) {
                        Ok(Some(result)) => {
                            token = Some(result);
                        }
                        _ => {
                            return_result.status = false;
                            
                            println!("[Not Found] Failed to extract key token.");
                        }
                    }
                }
            }
            // ================================================

            // Get source info from generated token.
            // println!("Token: {:?}", token.clone());
            if token.is_some() {
                let mut headers = HeaderMap::new();
                headers.insert(REFERER, HeaderValue::from_str(&SERVER_REFERER).unwrap());
                headers.insert(HOST, HeaderValue::from_str(&SERVER_HOST).unwrap());
                let url = format!("https://{}/embed-2/v3/e-1/getSources?id={}&_k={}", &SERVER_HOST, forward_server_id.as_ref().unwrap(), &token.unwrap());
                let res = client.get(url)
                .headers(headers.clone())
                .send().unwrap();

                if res.status().is_success() {
                    let data = res.json::<Value>();
                    match data {
                        Ok(result) => {
                            let server = result.get("server").unwrap().as_u64().unwrap() as usize;
                            let file_url = result.get("sources").unwrap().get(0).unwrap().get("file").unwrap().as_str().unwrap();
                            let base_url  = file_url.split('/').collect::<Vec<_>>()[..file_url.matches('/').count()].join("/");
                            if server == 1 {
                                return_result.config.segment_base_url = base_url.clone();
                            }

                            match Url::parse(file_url) {
                                Ok(parsed_url) => {
                                    if let Some(host) = parsed_url.host_str() {
                                        return_result.status = true;
                                        if server != 6 {
                                            return_result.config.playlist_base_url = base_url.clone();
                                        }
                                        
                                        return_result.config.host = host.to_string();
                                        return_result.data = result;
                                        return_result.message = String::from("success");
                                    } else {
                                        return_result.message = String::from("Failed to get host.");
                                    }
                                }
                                Err(e) => {
                                    return_result.message = String::from(e.to_string());
                                },
                            }
                            
                        }
                        _ => {
                            return_result.status = false;
                            println!("[Failed] Failed to get sources.");
                        }
                    }
                }
                
            }

            // ========================================
        }
        
        return serde_json::to_string(&return_result).unwrap();
    });

    match result {
        Ok(data) => {
            let result = CString::new(data).unwrap();
            return result.into_raw();
        },
        _ => ptr::null(),
    }
}


fn extract_key_token(html: &str) -> Result<Option<String>, BoxDynError> {
    let dom = Vis::load(html)?;

    // 1. meta[name="_gg_fb"]
    if let Some(content) = dom.find(r#"meta[name="_gg_fb"]"#).attr("content") {
        return Ok(Some(content.to_string()));
    }

    // 2. script[nonce]
    if let Some(nonce) = dom.find("script[nonce]").attr("nonce") {
        return Ok(Some(nonce.to_string()));
    }

    // 3. div[data-dpi]
    if let Some(dpi) = dom.find("div[data-dpi]").attr("data-dpi") {
        return Ok(Some(dpi.to_string()));
    }

    // 4. script containing 'window._xy_ws'
    let token = dom
        .find("script")
        .map(|_idx: usize, el: &BoxDynElement| {
            let html = el.text();
            if html.contains("window._xy_ws") {
                html.split('"').nth(1).map(|s| s.to_string())
            } else {
                None
            }
        })
        .into_iter()
        .flatten()
        .next();
    if let Some(t) = token {
        return Ok(Some(t));
    }

    // 5. comment starting with "_is_th:"
    let comment_token = Regex::new(r#"<!--\s*_is_th:\s*(.*?)\s*-->"#)
        .ok()
        .and_then(|re| {
            re.captures_iter(html)
                .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
                .next()
        });
    if let Some(t) = comment_token {
        return Ok(Some(t));
    }

    // 6. script containing window._lk_db
    let js_token = Regex::new(
        r#"window\._lk_db\s*=\s*\{\s*x:\s*"([^"]+)",\s*y:\s*"([^"]+)",\s*z:\s*"([^"]+)"\s*\}"#
    ).ok()
        .and_then(|re| {
            re.captures(html).map(|caps| {
                let x = caps.get(1)?.as_str();
                let y = caps.get(2)?.as_str();
                let z = caps.get(3)?.as_str();
                Some(format!("{}{}{}", x, y, z))
            }).flatten()
        });


    if let Some(t) = js_token {
        return Ok(Some(t));
    }

    Ok(None)
}