# saori-interface-rs

[GitHub repository](https://github.com/tukinami/saori-interface-rs)

## これは何?

デスクトップマスコット、「伺か」用DLLの一種、「SAORI」のためのライブラリです。

### 使い方

SAORIのリクエストとレスポンスを処理します。

`SaoriRequest` は、`SHIORI`から来たSAORIのリクエストからを表す構造体です。
`SaoriRequest::new` から作成できます。

`SaoriResponse` は、`SHIORI`へのレスポンスを表す構造体です。
`SaoriResponse::new_bad_request` から、空で`Status`が`400 Bad Request`のものを、
`SaoriResponse::from_request` から、 `SaoriRequest` にあった内容のものを作成できます。
また、SHIORIに値を返却するときは、 `SaoriResponse::to_encoded_bytes` で`Vec<i8>`に変換できます。


## 例

```rust
use saori_interface_rs::*;

let request_raw = "EXECUTE SAORI/1.0\r\nCharset: UTF-8\r\n\r\n\0";
let request = SaoriRequest::new(request_raw.as_bytes()).unwrap();

let mut case = SaoriResponse::from_request(&request);
case.set_result("1".to_string());
case.set_values(vec!["aaa".to_string(), "bbb".to_string()]);
let result = case.to_encoded_bytes().unwrap_or(SaoriResponse::error_bytes());

// testing
let expect_raw =
    "SAORI/1.0 200 OK\r\nCharset: UTF-8\r\nResult: 1\r\nValue0: aaa\r\nValue1: bbb\r\n\r\n\0";
let expect: Vec<i8> = expect_raw.as_bytes().iter().map(|v| *v as i8).collect();
assert_eq!(result, expect);
```

## 使用ライブラリ

いずれも敬称略。ありがとうございます。

+ [encoding\_rs](https://github.com/hsivonen/encoding_rs) / Henri Sivonen

## ライセンス

MITにて配布いたします。

## 作成者

月波 清火 (tukinami seika)

[GitHub](https://github.com/tukinami)
