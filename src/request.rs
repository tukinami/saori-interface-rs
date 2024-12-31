//! SHIORIから来たSAORIのリクエストからを表す
//!
//! # Examples
//!
//! ```
//! use saori_interface_rs::*;
//!
//! let request_raw = "EXECUTE SAORI/1.0\r\nCharset: UTF-8\r\n\r\n\0";
//! let request = SaoriRequest::new(request_raw.as_bytes()).unwrap();
//!
//! // testing
//! assert_eq!(request.charset(), &SaoriCharset::UTF8);
//! assert_eq!(request.command(), &SaoriCommand::Execute);
//! assert_eq!(request.version(), &SaoriVersion::V1_0);
//! assert!(request.security_level().is_none());
//! assert!(request.arguments().is_empty());
//! assert!(request.sender().is_none());
//! ```

use encoding_rs::{Encoding, EUC_JP, ISO_2022_JP, SHIFT_JIS, UTF_8};

const SAORI_PREFIX_CHARSET: &str = "Charset: ";
const SAORI_COMMAND_GET_VERSION: &str = "GET Version ";
const SAORI_COMMAND_EXECUTE: &str = "EXECUTE ";
const SAORI_PREFIX_SECULITY_LEVEL: &str = "SecurityLevel: ";
const SAORI_PREFIX_ARGUMENT: &str = "Argument";
const SAORI_PREFIX_SENDER: &str = "Sender: ";

/// SAORIのリクエストを処理中のエラー
#[derive(Debug, PartialEq)]
pub enum SaoriRequestError {
    Charset(SaoriRequestCharsetError),
    VersionLine(SaoriRequestVersionLineError),
    Argument(SaoriRequestArgumentError),
}

/// SAORIのリクエストを処理中のエラー: Charset関連
#[derive(Debug, PartialEq)]
pub enum SaoriRequestCharsetError {
    DecodeFailed,
    UnsupportedCharset,
}

/// SAORIのリクエストを処理中のエラー: Version関連
#[derive(Debug, PartialEq)]
pub enum SaoriRequestVersionLineError {
    EmptyRequest,
    NoVersion,
    NoCommand,
}

/// SAORIのリクエストを処理中のエラー: Argument関連
#[derive(Debug, PartialEq)]
pub enum SaoriRequestArgumentError {
    InvalidSeparator,
    NoIndex,
}

/// SHIORIから来たSAORIのリクエストからを表す
#[derive(PartialEq, Debug)]
pub struct SaoriRequest {
    charset: SaoriCharset,
    command: SaoriCommand,
    version: SaoriVersion,
    security_level: Option<SaoriSecurityLevel>,
    arguments: Vec<String>,
    sender: Option<String>,
}

/// SAORIのCharset
#[derive(PartialEq, Debug, Clone)]
pub enum SaoriCharset {
    ShiftJIS,
    EucJP,
    UTF8,
    ISO2022JP,
}

/// SAORIのコマンド
#[derive(PartialEq, Debug)]
pub enum SaoriCommand {
    Execute,
    GetVersion,
}

/// SAORIのバージョン
#[derive(PartialEq, Debug, Clone)]
pub enum SaoriVersion {
    V1_0,
}

/// SAORIのSecurityLevel
#[derive(PartialEq, Debug, Clone)]
pub enum SaoriSecurityLevel {
    Local,
    External,
}

impl From<SaoriRequestCharsetError> for SaoriRequestError {
    fn from(e: SaoriRequestCharsetError) -> SaoriRequestError {
        SaoriRequestError::Charset(e)
    }
}

impl From<SaoriRequestVersionLineError> for SaoriRequestError {
    fn from(e: SaoriRequestVersionLineError) -> SaoriRequestError {
        SaoriRequestError::VersionLine(e)
    }
}

impl From<SaoriRequestArgumentError> for SaoriRequestError {
    fn from(e: SaoriRequestArgumentError) -> SaoriRequestError {
        SaoriRequestError::Argument(e)
    }
}

impl SaoriRequest {
    pub fn new(bytes: &[u8]) -> Result<SaoriRequest, SaoriRequestError> {
        let (body, charset) = SaoriRequest::read_contents_and_charset(bytes)?;

        let mut lines = body.lines();
        let (command, version) = SaoriRequest::parse_version_and_command(lines.next())?;

        let mut security_level: Option<SaoriSecurityLevel> = None;
        let mut arguments: Vec<String> = Vec::new();
        let mut sender: Option<String> = None;

        for line in lines {
            SaoriRequest::parse_security_level(line, &mut security_level);
            SaoriRequest::parse_arguments(line, &mut arguments)?;
            SaoriRequest::parse_sender(line, &mut sender);
        }

        Ok(SaoriRequest {
            charset,
            command,
            version,
            security_level,
            arguments,
            sender,
        })
    }

    fn read_contents_and_charset(
        bytes: &[u8],
    ) -> Result<(String, SaoriCharset), SaoriRequestError> {
        let temp_string = String::from_utf8_lossy(bytes);
        let mut temp_lines = temp_string.lines();

        let charset =
            if let Some(body) = temp_lines.find_map(|v| v.strip_prefix(SAORI_PREFIX_CHARSET)) {
                SaoriCharset::try_from(body)?
            } else {
                SaoriCharset::ShiftJIS
            };

        let (contents, _used_encoding, has_error) = charset.to_encoding().decode(bytes);

        if has_error {
            Err(SaoriRequestError::Charset(
                SaoriRequestCharsetError::DecodeFailed,
            ))
        } else {
            Ok((contents.to_string(), charset))
        }
    }

    fn parse_version_and_command(
        line: Option<&str>,
    ) -> Result<(SaoriCommand, SaoriVersion), SaoriRequestError> {
        let line = line.ok_or(SaoriRequestError::VersionLine(
            SaoriRequestVersionLineError::EmptyRequest,
        ))?;

        let (command, remain) = if let Some(remain) = line.strip_prefix(SAORI_COMMAND_GET_VERSION) {
            (SaoriCommand::GetVersion, remain)
        } else if let Some(remain) = line.strip_prefix(SAORI_COMMAND_EXECUTE) {
            (SaoriCommand::Execute, remain)
        } else {
            return Err(SaoriRequestError::VersionLine(
                SaoriRequestVersionLineError::NoCommand,
            ));
        };

        let version = match remain {
            r if r == SaoriVersion::V1_0.to_str() => SaoriVersion::V1_0,
            _ => {
                return Err(SaoriRequestError::VersionLine(
                    SaoriRequestVersionLineError::NoVersion,
                ))
            }
        };

        Ok((command, version))
    }

    fn parse_security_level(line: &str, security_level: &mut Option<SaoriSecurityLevel>) {
        if let Some(body) = line.strip_prefix(SAORI_PREFIX_SECULITY_LEVEL) {
            *security_level = match body {
                b if b == SaoriSecurityLevel::Local.to_str() => Some(SaoriSecurityLevel::Local),
                b if b == SaoriSecurityLevel::External.to_str() => {
                    Some(SaoriSecurityLevel::External)
                }
                _ => return,
            };
        }
    }

    fn parse_arguments(line: &str, arguments: &mut Vec<String>) -> Result<(), SaoriRequestError> {
        if let Some(contents) = line.strip_prefix(SAORI_PREFIX_ARGUMENT) {
            let (index_raw, value) =
                contents
                    .split_once(": ")
                    .ok_or(SaoriRequestError::Argument(
                        SaoriRequestArgumentError::InvalidSeparator,
                    ))?;
            let index = index_raw
                .parse::<usize>()
                .map_err(|_| SaoriRequestError::Argument(SaoriRequestArgumentError::NoIndex))?;

            while arguments.len() <= index {
                arguments.push(String::new());
            }
            arguments[index] = value.to_string()
        }

        Ok(())
    }

    fn parse_sender(line: &str, sender: &mut Option<String>) {
        if let Some(body) = line.strip_prefix(SAORI_PREFIX_SENDER) {
            *sender = Some(body.to_string())
        }
    }

    pub fn charset(&self) -> &SaoriCharset {
        &self.charset
    }
    pub fn command(&self) -> &SaoriCommand {
        &self.command
    }
    pub fn version(&self) -> &SaoriVersion {
        &self.version
    }
    pub fn security_level(&self) -> Option<&SaoriSecurityLevel> {
        self.security_level.as_ref()
    }
    pub fn arguments(&self) -> &Vec<String> {
        &self.arguments
    }
    pub fn sender(&self) -> Option<&String> {
        self.sender.as_ref()
    }
}

impl SaoriCharset {
    pub fn to_str(&self) -> &'static str {
        match self {
            SaoriCharset::ShiftJIS => "Shift_JIS",
            SaoriCharset::EucJP => "EUC-JP",
            SaoriCharset::UTF8 => "UTF-8",
            SaoriCharset::ISO2022JP => "ISO-2022-JP",
        }
    }

    pub fn to_encoding(&self) -> &'static Encoding {
        match self {
            SaoriCharset::ShiftJIS => SHIFT_JIS,
            SaoriCharset::EucJP => EUC_JP,
            SaoriCharset::UTF8 => UTF_8,
            SaoriCharset::ISO2022JP => ISO_2022_JP,
        }
    }
}

impl TryFrom<&str> for SaoriCharset {
    type Error = SaoriRequestCharsetError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            v if v == SaoriCharset::ShiftJIS.to_str() => Ok(SaoriCharset::ShiftJIS),
            v if v == SaoriCharset::EucJP.to_str() => Ok(SaoriCharset::EucJP),
            v if v == SaoriCharset::UTF8.to_str() => Ok(SaoriCharset::UTF8),
            v if v == SaoriCharset::ISO2022JP.to_str() => Ok(SaoriCharset::ISO2022JP),
            _ => Err(SaoriRequestCharsetError::UnsupportedCharset),
        }
    }
}

impl SaoriCommand {
    pub fn to_str(&self) -> &'static str {
        match self {
            SaoriCommand::Execute => "EXECUTE",
            SaoriCommand::GetVersion => "GET Version",
        }
    }
}

impl SaoriVersion {
    pub fn to_str(&self) -> &'static str {
        match self {
            SaoriVersion::V1_0 => "SAORI/1.0",
        }
    }
}

impl SaoriSecurityLevel {
    pub fn to_str(&self) -> &'static str {
        match self {
            SaoriSecurityLevel::Local => "Local",
            SaoriSecurityLevel::External => "External",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod saori_request {
        use super::*;

        mod new {
            use super::*;

            #[test]
            fn success_when_valid_bytes() {
                let case_raw = "GET Version SAORI/1.0\r\nCharset: Shift_JIS\r\n\r\n";
                let (case, _, _) = SHIFT_JIS.encode(&case_raw);
                let result = SaoriRequest::new(&case).unwrap();
                assert_eq!(result.charset(), &SaoriCharset::ShiftJIS);
                assert_eq!(result.command(), &SaoriCommand::GetVersion);
                assert_eq!(result.version(), &SaoriVersion::V1_0);
                assert_eq!(result.security_level(), None);
                assert_eq!(result.arguments(), &Vec::<String>::new());
                assert_eq!(result.sender(), None);
            }

            #[test]
            fn failed_when_invalid_bytes() {
                let case_raw = "GET SAORI/1.0\r\nCharset: Shift_JIS\r\n\r\n";
                let (case, _, _) = SHIFT_JIS.encode(&case_raw);
                assert!(SaoriRequest::new(&case).is_err());
            }
        }

        mod read_contents_and_charset {
            use super::*;

            #[test]
            fn success_when_valid_bytes() {
                let case_raw = "GET Version SAORI/1.0\r\nCharset: Shift_JIS\r\n\r\n";
                let (case, _, _) = SHIFT_JIS.encode(&case_raw);
                let (contents, charset) = SaoriRequest::read_contents_and_charset(&case).unwrap();
                assert_eq!(contents.as_str(), case_raw);
                assert_eq!(charset, SaoriCharset::ShiftJIS);
            }

            #[test]
            fn failed_when_invalid_bytes() {
                let case_raw =
                    "EXECUTE SHIORI/1.0\r\nCharset: UTF-8\r\nArgument0: あいうえお\r\n\r\n";
                let (case, _, _) = SHIFT_JIS.encode(&case_raw);
                assert!(SaoriRequest::read_contents_and_charset(&case).is_err());
            }
        }

        mod parse_versioni_and_command {
            use super::*;

            #[test]
            fn success_when_valid_str_get_version() {
                let case = Some("GET Version SAORI/1.0");
                let (command, version) = SaoriRequest::parse_version_and_command(case).unwrap();
                assert_eq!(command, SaoriCommand::GetVersion);
                assert_eq!(version, SaoriVersion::V1_0);
            }

            #[test]
            fn success_when_valid_str_execute() {
                let case = Some("EXECUTE SAORI/1.0");
                let (command, version) = SaoriRequest::parse_version_and_command(case).unwrap();
                assert_eq!(command, SaoriCommand::Execute);
                assert_eq!(version, SaoriVersion::V1_0);
            }

            #[test]
            fn failed_when_invalid_command() {
                let case = Some("SOMETHINGWRONG SAORI/1.0");
                assert!(SaoriRequest::parse_version_and_command(case).is_err());
            }

            #[test]
            fn failed_when_invalid_version() {
                let case = Some("EXECUTE SAORI1.0");
                assert!(SaoriRequest::parse_version_and_command(case).is_err());
            }

            #[test]
            fn failed_when_none() {
                let case = None;
                assert!(SaoriRequest::parse_version_and_command(case).is_err());
            }
        }

        mod parse_security_level {
            use super::*;

            #[test]
            fn execute_when_valid_str_local() {
                let case = "SecurityLevel: Local";
                let mut security_level = None;
                SaoriRequest::parse_security_level(case, &mut security_level);
                assert_eq!(security_level, Some(SaoriSecurityLevel::Local));
            }

            #[test]
            fn execute_when_valid_str_external() {
                let case = "SecurityLevel: External";
                let mut security_level = None;
                SaoriRequest::parse_security_level(case, &mut security_level);
                assert_eq!(security_level, Some(SaoriSecurityLevel::External));
            }

            #[test]
            fn nothing_when_invalid_str() {
                let case = "Argument2: aaa";
                let mut security_level = None;
                SaoriRequest::parse_security_level(case, &mut security_level);
                assert!(security_level.is_none());
            }
        }

        mod parse_arguments {
            use super::*;

            #[test]
            fn success_when_valid_str_inner() {
                let case = "Argument2: あああ";
                let mut arguments = vec!["".to_string(), "".to_string(), "".to_string()];
                SaoriRequest::parse_arguments(case, &mut arguments).unwrap();
                assert_eq!(
                    arguments,
                    vec!["".to_string(), "".to_string(), "あああ".to_string(),]
                );
            }

            #[test]
            fn success_when_valid_str_outer() {
                let case = "Argument2: あああ";
                let mut arguments = vec!["".to_string()];
                SaoriRequest::parse_arguments(case, &mut arguments).unwrap();
                assert_eq!(
                    arguments,
                    vec!["".to_string(), "".to_string(), "あああ".to_string(),]
                );
            }

            #[test]
            fn failed_when_invalid_separator() {
                let case = "Argument2 aaa";
                let mut arguments = Vec::new();
                let result = SaoriRequest::parse_arguments(case, &mut arguments);
                assert!(result.is_err());
            }

            #[test]
            fn failed_when_invalid_no_index() {
                let case = "Argumentaaa: aaa";
                let mut arguments = Vec::new();
                let result = SaoriRequest::parse_arguments(case, &mut arguments);
                assert!(result.is_err());
            }
        }

        mod parse_sender {
            use super::*;

            #[test]
            fn execute_when_valid_str() {
                let case = "Sender: materia";
                let mut sender = None;
                SaoriRequest::parse_sender(&case, &mut sender);
                assert_eq!(sender, Some("materia".to_string()));
            }

            #[test]
            fn nothing_when_invalid_str() {
                let case = "Argument3: aaaa";
                let mut sender = None;
                SaoriRequest::parse_sender(&case, &mut sender);
                assert!(sender.is_none());
            }
        }
    }
}
