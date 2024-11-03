use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Meta, NestedMeta, Type};

#[proc_macro_derive(PusParameters, attributes(hash))]
pub fn derive_pus_parameters(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let mut get_param_matches = quote!();
    let mut set_param_matches = quote!();

    for field in fields {
        let field_name = field.ident.unwrap();
        let field_type = &field.ty;

        let attr = field
            .attrs
            .iter()
            .find(|attr| attr.path.is_ident("hash"))
            .expect(&format!(
                "Field {} missing #[hash] attribute",
                field_name
            ));

        let meta = attr.parse_meta().unwrap();
        let hash = match meta {
            Meta::List(list) => {
                let nested: Vec<NestedMeta> = list.nested.into_iter().collect();
                match nested.get(0) {
                    Some(NestedMeta::Lit(hash)) => hash.clone(),
                    _ => panic!("Expected #[hash(0x...)]"),
                }
            }
            _ => panic!("Expected #[hash(0x...)]"),
        };

        let set_conversion = match field_type {
            Type::Path(type_path) if type_path.path.is_ident("f32") => {
                quote! {
                    let val = f64::from_be_bytes(bytes);
                    self.#field_name = val as f32;
                }
            }
            Type::Path(type_path) if type_path.path.is_ident("u16") => {
                quote! {
                    let start = ((64 - 16) as usize).div_ceil(8);
                    self.#field_name = u16::from_be_bytes(bytes[start..].try_into().unwrap());
                }
            }
            Type::Path(type_path) if type_path.path.is_ident("u32") => {
                quote! {
                    let start = ((64 - 32) as usize).div_ceil(8);
                    self.#field_name = u32::from_be_bytes(bytes[start..].try_into().unwrap());
                }
            }
            Type::Path(type_path) if type_path.path.is_ident("u64") => {
                quote! {
                    self.#field_name = u64::from_be_bytes(bytes);
                }
            }
            _ => panic!("Unsupported field type"),
        };

        get_param_matches.extend(quote! {
            #hash => {
                let val = self.#field_name.to_be_bytes();
                writer.write_bytes(&val)
                    .map_err(ParameterError::WriteError)?;
                Ok(val.len() * 8)
            },
        });

        set_param_matches.extend(quote! {
            #hash => {
                let bytes = buffer.get_bits(64).to_be_bytes();
                #set_conversion
                true
            },
        });
    }

    let expanded = quote! {
        impl PusParameters for #name {
            fn get_parameter_as_be_bytes(
                &self,
                hash: u32,
                writer: &mut BitWriter,
            ) -> Result<usize, ParameterError> {
                match hash {
                    #get_param_matches
                    _ => Err(ParameterError::UnknownParameter(hash)),
                }
            }

            fn set_parameter_from_be_bytes(&mut self, hash: u32, buffer: &mut BitBuffer) -> bool {
                match hash {
                    #set_param_matches
                    _ => false,
                }
            }

            fn get_parameter_size(&self, _hash: u32) -> Option<usize> {
                None // Deprecated function
            }
        }
    };

    TokenStream::from(expanded)
}
