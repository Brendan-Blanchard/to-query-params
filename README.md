# to-query-params
A procedural macro and trait for converting arbitrary structs into `Vec<(String, String)>` for use as query parameters.

`QueryParams` is meant to simplify the conversion of arbitrary structs into query parameters, largely for use with the 
[Hyper](https://crates.io/crates/hyper) HTTP framework. 

The macro does no URL encoding of strings at this time, and is meant to do the work of creating the `Vec<(String, String)>`,
which can be tedious for large or repetitive structs. A method may be added to the `ToQueryParams` trait to make url encoding
available in the future.

![badge](https://github.com/Brendan-Blanchard/to-query-params/actions/workflows/main.yml/badge.svg) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Usage:

```rust
use query_params::{ToQueryParams, QueryParams};

 // Eq and PartialEq are just for assertions
 #[derive(QueryParams, Debug, PartialEq, Eq)]
 struct ProductRequest {
     #[query(required)] // mark required fields as not Option<T>
     id: i32,
     #[query(rename = "type")]
     product_type: String,
     min_price: Option<i32>,
     max_price: Option<i32>,
 }

 pub fn main() {
     let request = ProductRequest {
         id: 999,
         product_type: "accessory".to_string(),
         min_price: None,
         max_price: Some(100),
     };

     let expected = vec![("id", "999".into()), ("type", "accessory".into()), ("max_price", "100".into())];
     
     let query_params = request.to_query_params();

     assert_eq!(expected, query_params);
 }
```