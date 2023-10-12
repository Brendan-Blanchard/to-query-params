//! QueryParams is a procedural macro for deriving a [`Hyper`] centric representation
//! of that struct as query parameters that can be easily appended to query parameters in the Hyper
//! framework.
//! [`Hyper`]: https://crates.io/crates/hyper
use proc_macro::{self, TokenStream};
use quote::quote;
use std::collections::HashSet;
use std::vec::Vec;
use syn::__private::TokenStream2;
use syn::{parse_macro_input, Attribute, DeriveInput, Field, Fields, Ident, LitStr};

#[derive(Debug, Eq, PartialEq, Hash)]
enum FieldAttributes {
    Required,
    Rename(String),
}

#[derive(Debug)]
struct FieldDescription {
    pub field_name: String,
    pub ident: Ident,
    pub attributes: HashSet<FieldAttributes>,
}

/// [`QueryParams`] derives `fn to_query_params(&self) -> Vec<(&'static str, String)>` for
/// any structs with values supporting `.to_string`. Optional values are only included if present,
/// and fields marked `#[query(required)]` must be non-optional. Renaming of fields is also available,
/// using `#[query(rename = "other_name")]` on the field.
///
/// # Example: Query Params
/// QueryParams supports both required and optional fields, which won't be included in the output
/// if their value is None.
///
/// ```
/// # use query_params_macro::QueryParams;
///
/// # pub trait ToQueryParams { // trait defined here again since it can be provided by macro crate
/// #    fn to_query_params(&self) -> Vec<(&'static str, String)>;
/// # }
///
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
///     let expected = vec![("id", "999".into()), ("max_price", "100".into())];
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
        .iter()
        .map(map_field_to_description)
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
        let mut query_params: ::std::vec::Vec<(&'static str, String)> = vec![#((#req_names, self.#req_idents.to_string())),*];
    };

    let optional_fields: Vec<&FieldDescription> = field_descriptions
        .iter()
        .filter(|desc| !desc.attributes.contains(&FieldAttributes::Required))
        .collect();

    let optional_assignments: TokenStream2 = optional_fields
        .iter()
        .map(|field| {
            let ident = &field.ident;
            let name = &field.field_name;
            quote! {
                if let Some(val) = &self.#ident {
                    query_params.push((#name, val.to_string()));
                }
            }
        })
        .collect();

    let trait_impl = quote! {
        impl ToQueryParams for #ident {
            fn to_query_params(&self) -> ::std::vec::Vec<(&'static str, String)> {
                #vec_definition
                #optional_assignments
                query_params
            }
        }
    };

    trait_impl.into()
}

fn map_field_to_description(field: &&Field) -> FieldDescription {
    let attributes = field
        .attrs
        .iter()
        .flat_map(parse_query_attributes)
        .collect::<HashSet<FieldAttributes>>();

    let mut desc = FieldDescription {
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

            if m.path.is_ident("rename") {
                let value = m.value().unwrap();
                let rename: LitStr = value.parse().unwrap();

                attrs.push(FieldAttributes::Rename(rename.value()));
            }

            Ok(())
        })
        .expect("Parsing should not fail.");
    }

    attrs
}
