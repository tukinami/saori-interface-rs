//! SAORIのレスポンス
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
//! let result = case.to_encoded_bytes().unwrap_or(SaoriResponse::error_bytes());
//!
//! // testing
//! let expect_raw =
//!     "SAORI/1.0 200 OK\r\nCharset: UTF-8\r\nResult: 1\r\nValue0: aaa\r\nValue1: bbb\r\n\r\n\0";
//! let expect: Vec<i8> = expect_raw.as_bytes().iter().map(|v| *v as i8).collect();
//! assert_eq!(result, expect);
//! ```

use std::fmt::Display;

use crate::request::{SaoriCharset, SaoriRequest, SaoriVersion};

/// SAORIのレスポンス
#[derive(PartialEq, Debug)]
pub struct SaoriResponse {
    version: SaoriVersion,
    status: SaoriStatus,
    result: String,
    values: Vec<String>,
    charset: SaoriCharset,
}

/// SAORIのレスポンスのステータス
#[derive(PartialEq, Debug)]
pub enum SaoriStatus {
    OK,
    NoContent,
    BadRequest,
    InternalServerError,
}

/// SaoriResponseを処理中のエラー
#[derive(PartialEq, Debug)]
pub enum SaoriResponseError {
    DecodeFailed,
}

impl SaoriResponse {
    /// status がBad Request である自身を生成する
    pub fn new_bad_request() -> SaoriResponse {
        SaoriResponse {
            version: SaoriVersion::V1_0,
            status: SaoriStatus::BadRequest,
            result: String::new(),
            values: Vec::new(),
            charset: SaoriCharset::UTF8,
        }
    }

    /// リクエストから自身を生成する
    pub fn from_request(request: &SaoriRequest) -> SaoriResponse {
        SaoriResponse {
            version: request.version().clone(),
            status: SaoriStatus::NoContent,
            result: String::new(),
            values: Vec::new(),
            charset: request.charset().clone(),
        }
    }

    pub fn status(&self) -> &SaoriStatus {
        &self.status
    }

    pub fn set_status(&mut self, status: SaoriStatus) {
        self.status = status;
    }

    pub fn result(&self) -> &str {
        &self.result
    }

    pub fn set_result(&mut self, result: String) {
        self.result = result;

        self.on_change_result_and_value();
    }

    pub fn values(&self) -> &[String] {
        &self.values
    }

    /// `index`にあるValue*に値を適用する。
    pub fn set_value_at(&mut self, index: usize, value: String) {
        while self.values.len() <= index {
            self.values.push(String::new());
        }
        self.values[index] = value;
        self.on_change_result_and_value();
    }

    pub fn set_values(&mut self, values: Vec<String>) {
        self.values = values;

        self.on_change_result_and_value();
    }

    /// resultとvalueが変更されたときに呼ばれる
    /// statusの切替を行う(Ok <=> No Content)
    fn on_change_result_and_value(&mut self) {
        match self.status {
            SaoriStatus::BadRequest | SaoriStatus::InternalServerError => {}
            _ => {
                let actually_empty_values = self.values.iter().all(|v| v.is_empty());
                self.status = if self.result.is_empty()
                    && (self.values.is_empty() || actually_empty_values)
                {
                    SaoriStatus::NoContent
                } else {
                    SaoriStatus::OK
                };
            }
        }
    }

    /// 自身をエンコードされた文字バイト列にして返す
    pub fn to_encoded_bytes(&self) -> Result<Vec<i8>, SaoriResponseError> {
        let response = self.to_string();

        match self
            .charset
            .to_encoding()
            .encode(&response, encoding::EncoderTrap::Strict)
        {
            Ok(v) => Ok(v.iter().map(|v| *v as i8).collect()),
            Err(_) => Err(SaoriResponseError::DecodeFailed),
        }
    }

    /// エラー時の返答バイト列を返す
    pub fn error_bytes() -> Vec<i8> {
        const ERROR_RESPONCE: &str =
            "SAORI/1.0 500 Internal Server Error\r\nCharset: UTF-8\r\n\r\n\0";
        ERROR_RESPONCE.as_bytes().iter().map(|v| *v as i8).collect()
    }
}

impl Display for SaoriResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = format!(
            "{} {} {}\r\nCharset: {}\r\n",
            self.version.to_str(),
            self.status.to_code(),
            self.status.to_str(),
            self.charset.to_str()
        );
        if self.status == SaoriStatus::OK {
            if !self.result.is_empty() {
                // Result: {}\r\n
                result.push_str("Result: ");
                result.push_str(&self.result);
                result.push_str("\r\n");
            }
            for (index, value) in self.values.iter().enumerate() {
                // Value{}: {}\r\n
                result.push_str("Value");
                result.push_str(&index.to_string());
                result.push_str(": ");
                result.push_str(value);
                result.push_str("\r\n");
            }
        }
        write!(f, "{}\r\n\0", result)
    }
}

impl SaoriStatus {
    pub fn to_code(&self) -> u16 {
        match self {
            SaoriStatus::OK => 200,
            SaoriStatus::NoContent => 204,
            SaoriStatus::BadRequest => 400,
            SaoriStatus::InternalServerError => 500,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            SaoriStatus::OK => "OK",
            SaoriStatus::NoContent => "No Content",
            SaoriStatus::BadRequest => "Bad Request",
            SaoriStatus::InternalServerError => "Internal Server Error",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod saori_response {
        use super::*;

        mod new_bad_request {
            use super::*;

            #[test]
            fn checking_value() {
                let case = SaoriResponse::new_bad_request();
                assert_eq!(
                    case,
                    SaoriResponse {
                        version: SaoriVersion::V1_0,
                        status: SaoriStatus::BadRequest,
                        result: String::new(),
                        values: vec![],
                        charset: SaoriCharset::UTF8
                    }
                );
            }
        }

        mod from_request {
            use super::*;

            #[test]
            fn checking_value() {
                let request_raw = "EXECUTE SAORI/1.0\r\nCharset: Shift_JIS\r\n\r\n\0";
                let request = SaoriRequest::new(request_raw.as_bytes()).unwrap();
                let case = SaoriResponse::from_request(&request);
                assert_eq!(
                    case,
                    SaoriResponse {
                        version: SaoriVersion::V1_0,
                        status: SaoriStatus::NoContent,
                        result: String::new(),
                        values: vec![],
                        charset: SaoriCharset::ShiftJIS
                    }
                );
            }
        }

        mod set_result {
            use super::*;

            #[test]
            fn checking_value_no_empty() {
                let request_raw = "EXECUTE SAORI/1.0\r\nCharset: Shift_JIS\r\n\r\n\0";
                let request = SaoriRequest::new(request_raw.as_bytes()).unwrap();
                let mut case = SaoriResponse::from_request(&request);
                assert_eq!(case.status, SaoriStatus::NoContent);
                let case_result = "aaa".to_string();
                case.set_result(case_result.clone());
                assert_eq!(
                    case,
                    SaoriResponse {
                        version: SaoriVersion::V1_0,
                        status: SaoriStatus::OK,
                        result: case_result.clone(),
                        values: vec![],
                        charset: SaoriCharset::ShiftJIS
                    }
                );
            }

            #[test]
            fn checking_value_empty() {
                let request_raw = "EXECUTE SAORI/1.0\r\nCharset: Shift_JIS\r\n\r\n\0";
                let request = SaoriRequest::new(request_raw.as_bytes()).unwrap();
                let mut case = SaoriResponse::from_request(&request);
                assert_eq!(case.status, SaoriStatus::NoContent);
                let case_result = "".to_string();
                case.set_result(case_result.clone());
                assert_eq!(
                    case,
                    SaoriResponse {
                        version: SaoriVersion::V1_0,
                        status: SaoriStatus::NoContent,
                        result: case_result.clone(),
                        values: vec![],
                        charset: SaoriCharset::ShiftJIS
                    }
                );
            }
        }

        mod set_value_at {
            use super::*;

            #[test]
            fn checking_value_inner() {
                let request_raw = "EXECUTE SAORI/1.0\r\nCharset: Shift_JIS\r\n\r\n\0";
                let request = SaoriRequest::new(request_raw.as_bytes()).unwrap();
                let mut case = SaoriResponse::from_request(&request);
                let case_values = vec!["aaa".to_string(), "bbb".to_string()];
                case.values = case_values;
                assert_eq!(case.status, SaoriStatus::NoContent);
                let case_value = "bbb002".to_string();
                case.set_value_at(1, case_value);
                assert_eq!(
                    case,
                    SaoriResponse {
                        version: SaoriVersion::V1_0,
                        status: SaoriStatus::OK,
                        result: String::new(),
                        values: vec!["aaa".to_string(), "bbb002".to_string()],
                        charset: SaoriCharset::ShiftJIS
                    }
                );
            }

            #[test]
            fn checking_value_outer() {
                let request_raw = "EXECUTE SAORI/1.0\r\nCharset: Shift_JIS\r\n\r\n\0";
                let request = SaoriRequest::new(request_raw.as_bytes()).unwrap();
                let mut case = SaoriResponse::from_request(&request);
                assert_eq!(case.status, SaoriStatus::NoContent);
                let case_value = "bbb002".to_string();
                case.set_value_at(1, case_value);
                assert_eq!(
                    case,
                    SaoriResponse {
                        version: SaoriVersion::V1_0,
                        status: SaoriStatus::OK,
                        result: String::new(),
                        values: vec!["".to_string(), "bbb002".to_string()],
                        charset: SaoriCharset::ShiftJIS
                    }
                );
            }
        }

        mod set_values {
            use super::*;

            #[test]
            fn checking_value_no_empty() {
                let request_raw = "EXECUTE SAORI/1.0\r\nCharset: Shift_JIS\r\n\r\n\0";
                let request = SaoriRequest::new(request_raw.as_bytes()).unwrap();
                let mut case = SaoriResponse::from_request(&request);
                assert_eq!(case.status, SaoriStatus::NoContent);
                let case_values = vec!["aaa".to_string(), "bbb".to_string()];
                case.set_values(case_values.clone());
                assert_eq!(
                    case,
                    SaoriResponse {
                        version: SaoriVersion::V1_0,
                        status: SaoriStatus::OK,
                        result: String::new(),
                        values: case_values.clone(),
                        charset: SaoriCharset::ShiftJIS
                    }
                );
            }

            #[test]
            fn checking_value_empty() {
                let request_raw = "EXECUTE SAORI/1.0\r\nCharset: Shift_JIS\r\n\r\n\0";
                let request = SaoriRequest::new(request_raw.as_bytes()).unwrap();
                let mut case = SaoriResponse::from_request(&request);
                assert_eq!(case.status, SaoriStatus::NoContent);
                let case_values = vec![];
                case.set_values(case_values.clone());
                assert_eq!(
                    case,
                    SaoriResponse {
                        version: SaoriVersion::V1_0,
                        status: SaoriStatus::NoContent,
                        result: String::new(),
                        values: case_values.clone(),
                        charset: SaoriCharset::ShiftJIS
                    }
                );
            }
        }

        mod on_change_result_and_value {
            use super::*;

            #[test]
            fn no_content_when_value_actually_empty_and_no_result() {
                let mut case = SaoriResponse::new_bad_request();
                case.set_status(SaoriStatus::OK);
                assert_eq!(case.status(), &SaoriStatus::OK);
                let values = vec![String::new(), String::new(), String::new()];
                case.set_values(values);
                assert_eq!(case.status(), &SaoriStatus::NoContent);
            }

            #[test]
            fn no_content_when_empty_value_and_no_result() {
                let mut case = SaoriResponse::new_bad_request();
                case.set_status(SaoriStatus::OK);
                assert_eq!(case.status(), &SaoriStatus::OK);
                let values = vec![];
                case.set_values(values);
                assert_eq!(case.status(), &SaoriStatus::NoContent);
            }

            #[test]
            fn ok_when_some_values() {
                let mut case = SaoriResponse::new_bad_request();
                case.set_status(SaoriStatus::NoContent);
                assert_eq!(case.status(), &SaoriStatus::NoContent);
                let values = vec![String::new(), String::new(), String::new()];
                case.set_values(values);
                assert_eq!(case.status(), &SaoriStatus::NoContent);
                case.set_value_at(3, "aaa".to_string());
                assert_eq!(case.status(), &SaoriStatus::OK);
            }

            #[test]
            fn ok_when_result_is_some() {
                let mut case = SaoriResponse::new_bad_request();
                case.set_status(SaoriStatus::NoContent);
                assert_eq!(case.status(), &SaoriStatus::NoContent);
                let values = vec![String::new(), String::new(), String::new()];
                case.set_values(values);
                assert_eq!(case.status(), &SaoriStatus::NoContent);
                case.set_result("aaa".to_string());
                assert_eq!(case.status(), &SaoriStatus::OK);
            }
        }

        mod to_encoded_bytes {
            use encoding::{all::WINDOWS_31J, EncoderTrap, Encoding};

            use super::*;

            #[test]
            fn success_when_valid_request() {
                let request_raw = "EXECUTE SAORI/1.0\r\nCharset: Shift_JIS\r\n\r\n\0";
                let request = SaoriRequest::new(request_raw.as_bytes()).unwrap();
                let mut case = SaoriResponse::from_request(&request);
                case.set_result("1".to_string());
                case.set_values(vec!["aaa".to_string(), "bbb".to_string()]);
                let result = case.to_encoded_bytes().unwrap();
                let expect_raw =
                    "SAORI/1.0 200 OK\r\nCharset: Shift_JIS\r\nResult: 1\r\nValue0: aaa\r\nValue1: bbb\r\n\r\n\0";
                let expect = WINDOWS_31J.encode(expect_raw, EncoderTrap::Strict).unwrap();
                let expect: Vec<i8> = expect.iter().map(|v| *v as i8).collect();
                assert_eq!(result, expect);
            }
        }

        mod to_string {
            use super::*;

            #[test]
            fn checking_value_bad_request() {
                let case = SaoriResponse::new_bad_request();
                let result = case.to_string();
                let expect = "SAORI/1.0 400 Bad Request\r\nCharset: UTF-8\r\n\r\n\0".to_string();
                assert_eq!(result, expect);
            }

            #[test]
            fn checking_value_internal_server_error() {
                let mut case = SaoriResponse::new_bad_request();
                case.set_status(SaoriStatus::InternalServerError);
                case.set_result("1".to_string());
                let result = case.to_string();
                let expect =
                    "SAORI/1.0 500 Internal Server Error\r\nCharset: UTF-8\r\n\r\n\0".to_string();
                assert_eq!(result, expect);
            }

            #[test]
            fn checking_value_no_content() {
                let request_raw = "GET Version SAORI/1.0\r\nCharset: Shift_JIS\r\n\r\n\0";
                let request = SaoriRequest::new(request_raw.as_bytes()).unwrap();
                let case = SaoriResponse::from_request(&request);
                let result = case.to_string();
                let expect = "SAORI/1.0 204 No Content\r\nCharset: Shift_JIS\r\n\r\n\0".to_string();

                assert_eq!(result, expect);
            }

            #[test]
            fn checking_value_result_only() {
                let request_raw = "GET Version SAORI/1.0\r\nCharset: Shift_JIS\r\n\r\n\0";
                let request = SaoriRequest::new(request_raw.as_bytes()).unwrap();
                let mut case = SaoriResponse::from_request(&request);
                case.set_result("1".to_string());
                let result = case.to_string();
                let expect =
                    "SAORI/1.0 200 OK\r\nCharset: Shift_JIS\r\nResult: 1\r\n\r\n\0".to_string();

                assert_eq!(result, expect);
            }

            #[test]
            fn checking_value_result_with_values() {
                let request_raw = "EXECUTE SAORI/1.0\r\nCharset: Shift_JIS\r\n\r\n\0";
                let request = SaoriRequest::new(request_raw.as_bytes()).unwrap();
                let mut case = SaoriResponse::from_request(&request);
                case.set_result("1".to_string());
                case.set_values(vec!["aaa".to_string(), "bbb".to_string()]);
                let result = case.to_string();
                let expect =
                    "SAORI/1.0 200 OK\r\nCharset: Shift_JIS\r\nResult: 1\r\nValue0: aaa\r\nValue1: bbb\r\n\r\n\0".to_string();

                assert_eq!(result, expect);
            }

            #[test]
            fn checking_value_with_values_only() {
                let request_raw = "EXECUTE SAORI/1.0\r\nCharset: Shift_JIS\r\n\r\n\0";
                let request = SaoriRequest::new(request_raw.as_bytes()).unwrap();
                let mut case = SaoriResponse::from_request(&request);
                case.set_values(vec!["aaa".to_string(), "bbb".to_string()]);
                let result = case.to_string();
                let expect =
                    "SAORI/1.0 200 OK\r\nCharset: Shift_JIS\r\nValue0: aaa\r\nValue1: bbb\r\n\r\n\0".to_string();

                assert_eq!(result, expect);
            }
        }
    }
}
