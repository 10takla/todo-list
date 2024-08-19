use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Fields, FieldsNamed, ItemStruct};

#[proc_macro_derive(Table)]
pub fn table(input: TokenStream) -> TokenStream {
    let ItemStruct { ident, fields, .. } = parse_macro_input!(input);

    let Fields::Named(FieldsNamed { named, .. }) = fields else {
        panic!()
    };

    let idents = named
        .into_iter()
        .map(|syn::Field { ident, ty, .. }| {
            let avail_types = &["String", "Date", "bool"];
            avail_types
                .iter()
                .find(|&&type_| type_ == ty.to_token_stream().to_string())
                .expect(&format!("available types: {}", avail_types.join(", ")));
            ident
        })
        .collect::<Vec<_>>();
    let len = idents.len();

    quote! {
        use serde_json::Value;
        use crate::traits::Table;
        impl Table<#len> for #ident {
            type Key = String;
            type Value = Value;
            fn get_keys() -> [Self::Key; #len] {
                [#(Self::Key::from(stringify!(#idents))),*]
            }
            fn get_values(&self) -> [Self::Value; #len] {
                let serialized_task = serde_json::to_value(self).unwrap();
                Task::get_keys()
                    .map(|key| serialized_task.get(key).unwrap().clone())
            }
            fn get_entries(&self) -> [(Self::Key, Self::Value); #len] {
                let serialized_task = serde_json::to_value(self).unwrap();
                Task::get_keys()
                    .map(|key| (key.clone(), serialized_task.get(key).unwrap().clone()))
            }
            fn get_value(&self, key: &Self::Key) -> Option<Self::Value> {
                serde_json::to_value(self).unwrap().get(key).cloned()
            }
        }
    }
    .into()
}
