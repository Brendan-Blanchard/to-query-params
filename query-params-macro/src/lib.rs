//! QueryParams is a procedural macro for deriving a [`Hyper`]-centric representation
//! of that struct as query parameters that can be easily appended to query parameters in the Hyper
//! framework. *This crate is only meant to be tested and re-exported by the `QueryParams` crate,
//! and is not meant for direct consumption.*
//!
//! [`Hyper`]: https://crates.io/crates/hyper
use proc_macro::{self, TokenStream};
use quote::quote;
use std::collections::HashSet;
use std::vec::Vec;
use syn::__private::TokenStream2;
use syn::{parse_macro_input, Attribute, DeriveInput, Field, Fields, Ident, LitStr, Path, Type};

#[derive(Debug, Eq, PartialEq, Hash)]
enum FieldAttributes {
    Required,
    Excluded,
    Rename(String),
}

struct FieldDescription<'f> {
    pub field: &'f Field,
    pub field_name: String,
    pub ident: Ident,
    pub attributes: HashSet<FieldAttributes>,
}

/// [`QueryParams`] derives `fn to_query_params(&self) -> Vec<(String, String)>` for
/// any struct with field values supporting `.to_string()`.
///
/// Optional values are only included if present,
/// and fields marked `#[query(required)]` must be non-optional. Renaming and excluding of fields is
/// also available, using `#[query(rename = "new_name")]` or `#[query(exclude)]` on the field.
///
///
///
/// # Example: Query Params
/// QueryParams supports both required and optional fields, which won't be included in the output
/// if their value is None.
///
/// ```
/// # use query_params_macro::QueryParams;
/// # // trait defined here again since it can't be provided by macro crate
/// # pub trait ToQueryParams {
/// #    fn to_query_params(&self) -> Vec<(String, String)>;
/// # }
/// // Eq and PartialEq are just for assertions
/// #[derive(QueryParams, Debug, PartialEq, Eq)]
/// struct ProductRequest {
///     #[query(required)]
///     id: i32,
///     min_price: Option<i32>,
///     max_price: Option<i32>,
/// }
///
/// pub fn main() {
///     let request = ProductRequest {
///         id: 999, // will be included in output
///         min_price: None, // will *not* be included in output
///         max_price: Some(100), // will be included in output
///     };
///
///     let expected = vec![
///         ("id".into(), "999".into()),
///         ("max_price".into(), "100".into())
///     ];
///     
///     let query_params = request.to_query_params();
///
///     assert_eq!(expected, query_params);
/// }
/// ```
///
/// ## Attributes
/// QueryParams supports attributes under `#[query(...)]` on individual fields to carry metadata.
/// At this time, the available attributes are:
/// - required -- marks a field as required, meaning it can be `T` instead of `Option<T>` on the struct
/// and will always appear in the resulting `Vec`
/// - rename -- marks a field to be renamed when it is output in the resulting Vec.
/// E.g. `#[query(rename = "newName")]`
/// - exclude -- marks a field to never be included in the output query params
///
/// # Example: Renaming and Excluding
/// In some cases, names of query parameters are not valid identifiers, or don't adhere to Rust's
/// default style of "snake_case". [`QueryParams`] can rename individual fields when creating the
/// query parameters Vec if the attribute with the rename attribute: `#[query(rename = "new_name")]`.
///
/// In the below example, an API expects a type of product and a max price, given as
/// `type=something&maxPrice=123`, which would be and invalid identifier and a non-Rust style
/// field name respectively. A field containing local data that won't be included in the query
/// is also tagged as `#[query(exclude)]` to exclude it.
///
/// ```
/// # use query_params_macro::QueryParams;
/// # use urlencoding;
/// # // trait defined here again since it can't be provided by macro crate
/// # pub trait ToQueryParams {
/// #    fn to_query_params(&self) -> Vec<(String, String)>;
/// # }
/// // Eq and PartialEq are just for assertions
/// #[derive(QueryParams, Debug, PartialEq, Eq)]
/// struct ProductRequest {
///     #[query(required)]
///     id: i32,
///     #[query(rename = "type")]
///     product_type: Option<String>,
///     #[query(rename = "maxPrice")]
///     max_price: Option<i32>,
///     #[query(exclude)]
///     private_data: i32,
/// }
///
/// pub fn main() {
///     let request = ProductRequest {
///         id: 999,
///         product_type: Some("accessory".into()),
///         max_price: Some(100),
///         private_data: 42, // will not be part of the output
///     };
///
///     let expected = vec![
///         ("id".into(), "999".into()),
///         ("type".into(), "accessory".into()),
///         ("maxPrice".into(), "100".into())
///     ];
///     
///     let query_params = request.to_query_params();
///
///     assert_eq!(expected, query_params);
/// }
/// ```
#[proc_macro_derive(QueryParams, attributes(query))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input);
    let ident = ast.ident;

    let fields: &Fields = match ast.data {
        syn::Data::Struct(ref s) => &s.fields,
        _ => panic!("Can only derive QueryParams for structs."),
    };

    let named_fields: Vec<&Field> = fields
        .iter()
        .filter_map(|field| field.ident.as_ref().map(|_ident| field))
        .collect();

    let field_descriptions = named_fields
        .into_iter()
        .map(map_field_to_description)
        .filter(|field| !field.attributes.contains(&FieldAttributes::Excluded))
        .collect::<Vec<FieldDescription>>();

    let required_fields: Vec<&FieldDescription> = field_descriptions
        .iter()
        .filter(|desc| desc.attributes.contains(&FieldAttributes::Required))
        .collect();

    let req_names: Vec<String> = required_fields
        .iter()
        .map(|field| field.field_name.clone())
        .collect();

    let req_idents: Vec<&Ident> = required_fields.iter().map(|field| &field.ident).collect();

    let vec_definition = quote! {
        let mut query_params: ::std::vec::Vec<(String, String)> =
        vec![#(
            (
                ::urlencoding::encode(#req_names).into_owned(),
                ::urlencoding::encode(&self.#req_idents.to_string()).into_owned()
            )
        ),*];
    };

    let optional_fields: Vec<&FieldDescription> = field_descriptions
        .iter()
        .filter(|desc| !desc.attributes.contains(&FieldAttributes::Required))
        .collect();

    optional_fields.iter().for_each(validate_optional_field);

    let optional_assignments: TokenStream2 = optional_fields
        .iter()
        .map(|field| {
            let ident = &field.ident;
            let name = &field.field_name;
            quote! {
                if let Some(val) = &self.#ident {
                    query_params.push(
                        (
                            ::urlencoding::encode(#name).into_owned(),
                            ::urlencoding::encode(&val.to_string()).into_owned()
                        )
                    );
                }
            }
        })
        .collect();

    let trait_impl = quote! {
        impl ToQueryParams for #ident {
            fn to_query_params(&self) -> ::std::vec::Vec<(String, String)> {
                #vec_definition
                #optional_assignments
                query_params
            }
        }
    };

    trait_impl.into()
}

fn map_field_to_description(field: &Field) -> FieldDescription {
    let attributes = field
        .attrs
        .iter()
        .flat_map(parse_query_attributes)
        .collect::<HashSet<FieldAttributes>>();

    let mut desc = FieldDescription {
        field,
        field_name: field.ident.as_ref().unwrap().to_string(),
        ident: field.ident.clone().unwrap(),
        attributes,
    };

    let name = name_from_field_description(&desc);
    desc.field_name = name;
    desc
}

fn name_from_field_description(field: &FieldDescription) -> String {
    let mut name = field.ident.to_string();
    for attribute in field.attributes.iter() {
        if let FieldAttributes::Rename(rename) = attribute {
            name = (*rename).clone();
        }
    }

    name
}

fn parse_query_attributes(attr: &Attribute) -> Vec<FieldAttributes> {
    let mut attrs = Vec::new();

    if attr.path().is_ident("query") {
        attr.parse_nested_meta(|m| {
            if m.path.is_ident("required") {
                attrs.push(FieldAttributes::Required);
            }

            if m.path.is_ident("exclude") {
                attrs.push(FieldAttributes::Excluded);
            }

            if m.path.is_ident("rename") {
                let value = m.value().unwrap();
                let rename: LitStr = value.parse().unwrap();

                attrs.push(FieldAttributes::Rename(rename.value()));
            }

            Ok(())
        })
        .expect("Unsupported attribute found in #[query(...)] attribute");
    }

    attrs
}

fn validate_optional_field(field_desc: &&FieldDescription) {
    if let Type::Path(type_path) = &field_desc.field.ty {
        if !(type_path.qself.is_none() && path_is_option(&type_path.path)) {
            panic!("Non-optional types must be marked with #[query(required)] attribute")
        }
    }
}

fn path_is_option(path: &Path) -> bool {
    path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments.iter().next().unwrap().ident == "Option"
}
