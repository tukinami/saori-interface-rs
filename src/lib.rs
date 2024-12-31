//! SAORIのリクエストとレスポンスを処理します。
//!
//! [`SaoriRequest`] は、`SHIORI`から来たSAORIのリクエストからを表す構造体です。
//! [`SaoriRequest::new`] から作成できます。
//!
//! [`SaoriResponse`] は、`SHIORI`へのレスポンスを表す構造体です。
//! [`SaoriResponse::new_bad_request`] から、空で`Status`が`400 Bad Request`のものを、
//! [`SaoriResponse::from_request`] から、 [`SaoriRequest`] にあった内容のものを作成できます。
//! また、SHIORIに値を返却するときは、 [`SaoriResponse::to_encoded_bytes`] で`Vec<i8>`に変換できます。
//!
//! # Examples
//!
//! ```
//! use saori_interface_rs::*;
//!
//! let request_raw = "EXECUTE SAORI/1.0\r\nCharset: UTF-8\r\n\r\n\0";
//! let request = SaoriRequest::new(request_raw.as_bytes()).unwrap();
//!
//! let mut case = SaoriResponse::from_request(&request);
//! case.set_result("1".to_string());
//! case.set_values(vec!["aaa".to_string(), "bbb".to_string()]);
//! let result = case.to_encoded_bytes().unwrap();
//!
//! // testing
//! let expect_raw =
//!     "SAORI/1.0 200 OK\r\nCharset: UTF-8\r\nResult: 1\r\nValue0: aaa\r\nValue1: bbb\r\n\r\n\0";
//! let expect: Vec<i8> = expect_raw.as_bytes().iter().map(|v| *v as i8).collect();
//! assert_eq!(result, expect);
//! ```
//!
//! [`SaoriRequest`]: crate::request::SaoriRequest
//! [`SaoriRequest::new`]: crate::request::SaoriRequest::new
//! [`SaoriResponse`]: crate::response::SaoriResponse
//! [`SaoriResponse::new_bad_request`]: crate::response::SaoriResponse::new_bad_request
//! [`SaoriResponse::to_encoded_bytes`]: crate::response::SaoriResponse::to_encoded_bytes

pub mod request;
pub mod response;

pub use request::*;
pub use response::*;
