pub mod wrap;
mod utils;

pub use wrap::*;
use polywrap_wasm_rs::JSON;
use crate::imported::http_module;
use crate::utils::{handle_unspecified_rpc_error, RpcData, to_rpc_data};

pub fn query<'a>(input: InputQuery) -> Option<Response> {

    let http_response: HttpResponse = match HttpModule::post(&http_module::InputPost {
        url: input.url,
        request: Some(HttpRequest {
            headers: Some(vec![
            HttpHeader { key: String::from("Content-Type"), value: String::from("application/json-rpc") },
            HttpHeader { key: String::from("Accept"), value: String::from("application/json") },
            ]),
            url_params: None,
            response_type: HttpResponseType::TEXT,
            body: Some(RpcData::from(&input.request).stringify()),
        }),
    }) {
        Ok(Some(v)) => v,
        Ok(None) => panic!("Did not receive HTTP response"),
        Err(e) => panic!("{}", e),
    };

    // handle json rpc error
    if httpResponse.status == 400 || httpResponse.status == 404 || httpResponse.status == 500 {
        // TODO: how to handle json rpc notification (i.e. no request id) when error occurs?
        return match http_response.body {
            Some(v) => Some(JSON::from_str::<Response>(v.as_str()).unwrap()),
            // handle unexpected missing response body
            None => {
                let id: i32 = input.request.id.unwrap_or(i32::MIN);
                return Some(Response {
                    result: None,
                    error: Some(handle_unspecified_rpc_error(&http_response)),
                    id,
                });
            }
        };
    }

    // handle json rpc success
    if httpResponse.status >= 200 && httpResponse.status <= 299 {
        if input.request.id.is_none() {
            // response was not requested
            return None;
        }
        return match http_response.body {
            Some(v) => Some(JSON::from_str::<Response>(v.as_str()).unwrap()),
            // handle unexpected missing response body
            None => Some(Response {
                result: None,
                error: None,
                id: input.request.id.unwrap()
            })
        };
    }

    panic!("Unexpected HTTP response status: {}", http_response.status);
}


