use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use log::debug;
use std::time::Duration;
// use serde_json::*;


#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Debug);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(PekeAllRoundRoot {
            transponder_config: serde_json::Value::String(String::new()),
            first_byte: String::new(),
        })
    });
}

fn to_json(data: String) -> serde_json::Value {
    let v: serde_json::Value = serde_json::from_str(&data).unwrap(); 
    return v;
}

struct PekeAllRoundRoot {
    transponder_config: serde_json::Value,
    first_byte: String,
}


impl Context for PekeAllRoundRoot {}

impl RootContext for PekeAllRoundRoot {
    fn on_configure(&mut self, _: usize) -> bool {
        if let Some(config_bytes) = self.get_configuration() {
            self.transponder_config = to_json(String::from_utf8(config_bytes).unwrap())
        }
        true
    }


    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(PekeAllRound {
            transponder_config: self.transponder_config.clone(),
            first_byte : self.first_byte.clone(),
        }))

    }
}


struct PekeAllRound {
    transponder_config: serde_json::Value,
    first_byte: String,
}

impl Context for PekeAllRound {
    fn on_http_call_response(&mut self, _: u32, _: usize, body_size: usize, _: usize) {
// here we can do stuff before continuing the request
       if let Some(body) = self.get_http_call_response_body(0, body_size) {
            debug!("{}", format!("Back from check with body[0] = {}",body[0]));
            self.first_byte = format!("{}",body[0]).as_str().to_string();
            self.set_http_request_header("peke-first-byte-check", Some(&format!("{}",self.first_byte).to_string()));
        } 
       self.resume_http_request();
       return;
   }

}

impl HttpContext for PekeAllRound {
    fn on_http_request_headers(&mut self, _: usize) -> Action {
        self.set_http_request_header("peke-extra-url-mask", self.transponder_config["extraUrlsToMask"].as_str());

        self.dispatch_http_call(
            self.transponder_config["checksvc"].as_str().unwrap(),
            vec![
                (":method", "GET"),
                (":path", "/bytes/1"),
                (":authority", "httpbin.org"),
            ],
            None,
            vec![],
            Duration::from_secs(5),
        )
        .unwrap();
 
 //       self.set_http_request_header("peke2-first-byte-check", Some(&format!("{}",self.first_byte).to_string()));
 
        Action::Pause
    }


    fn on_http_response_headers(&mut self, _: usize) -> Action {
        // If there is a Content-Length header and we change the length of
        // the body later, then clients will break. So remove it.
        // We must do this here, because once we exit this function we
        // can no longer modify the response headers.
        self.set_http_response_header("content-length", None);
        self.set_http_response_header("peketest",Some("was-here"));

        Action::Continue
    }

    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            // Wait -- we'll be called again when the complete body is buffered
            // at the host side.
            return Action::Pause;
        }

        // Replace the message body if it contains the text "secret".
        // Since we returned "Pause" previuously, this will return the whole body.
        if let Some(body_bytes) = self.get_http_response_body(0, body_size) {
            let body_str = String::from_utf8(body_bytes).unwrap();
            if body_str.contains("secret") {
                let new_body = body_str.replace("secret", "open");
                let v: serde_json::Value = to_json(body_str);
                let new_body2 = format!("{{ orig:{}, pekeJsonParsing_url: {}, headersEnabled: {}, extraUrlsToMask: {}, body: {} }}\n", new_body, v["url"], self.transponder_config["headersEnabled"],self.transponder_config["extraUrlsToMask"],self.transponder_config["body"]);
                self.set_http_response_body(0, body_size, &new_body2.into_bytes());
            }
        }
        Action::Continue
    }
}