use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Meta, NestedMeta, Type};

/// Derives the PusParameters trait implementation for a struct.
///
/// This derive macro automatically implements the PusParameters trait for structs containing
/// fields that represent spacecraft parameters. Each field must be annotated with a `#[hash]`
/// attribute specifying a unique 32-bit identifier.
///
/// # Supported Field Types
/// - `f32`: 32-bit floating point
/// - `u8`: 8-bit unsigned integer
/// - `u16`: 16-bit unsigned integer
/// - `u32`: 32-bit unsigned integer
/// - `u64`: 64-bit unsigned integer
/// - `i8`: 8-bit signed integer
/// - `i16`: 16-bit signed integer
/// - `i32`: 32-bit signed integer
/// - `i64`: 64-bit signed integer
///
/// # Example
/// ```
/// #[derive(PusParameters)]
/// struct MyParameters {
///     #[hash(0xABCD0001)]
///     temperature: f32,
///     #[hash(0xABCD0002)] 
///     voltage: u16,
/// }
/// ```
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
            Type::Path(type_path) => {
                let type_ident = type_path.path.get_ident().unwrap();
                let type_name = type_ident.to_string();
                // Special handling for floating point types
                if type_name == "f32" {
                    quote! {
                        let val = f64::from_be_bytes(bytes);
                        self.#field_name = val as f32;
                    }
                } else if type_name == "f64" {
                    quote! {
                        self.#field_name = f64::from_be_bytes(bytes);
                    }
                } else if type_name.len() == 3 {
                    // Handle integer types by parsing the bit size from type name
                    let bits: usize = type_name[1..].parse().unwrap_or(0);
                    if bits > 0 && bits <= 64 {
                        quote! {
                            let start = ((64 - #bits) as usize).div_ceil(8);
                            self.#field_name = #type_ident::from_be_bytes(bytes[start..].try_into().unwrap());
                        }
                    } else {
                        panic!("Unsupported field `{}` type {}", field_name, type_name)
                    }
                } else {
                    panic!("Unsupported field `{}` type {}", field_name, type_name)
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
