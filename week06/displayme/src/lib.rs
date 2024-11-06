extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// implement the following procedural `#[derive(DisplayMe)]` macro
/// It should be usable only on structs. When used on enums (or unions), it should produce a compile
/// error.
///
/// The macro should generate code that will implement the `Display` trait for the struct. The
/// specific format of the display implementation is defined by tests in the `assignments` crate.
#[proc_macro_derive(DisplayMe)]
pub fn derive_display_me(stream: TokenStream) -> TokenStream {
    // Parse the input token stream as an ADT (struct/enum/union) using the `syn` crate
    let input = parse_macro_input!(stream as DeriveInput);
    match input.data {
        Data::Struct(syn::DataStruct { fields, .. }) => {
            let name = &input.ident;

            let mut inner_display = quote! { write!(f,"struct {}", stringify!(#name))?;};

            match &fields {
                Fields::Named(fields) => {
                    inner_display.extend(quote! {
                        write!(f, " {{")?;
                    });
                    for (i, field) in fields.named.iter().enumerate() {
                        let identifier = field.ident.as_ref().unwrap();
                        inner_display.extend(quote! {
                            write!(f, "\n    {}: {}", stringify!(#identifier), &self.#identifier)?;
                        });
                        if i != fields.named.len() - 1 {
                            inner_display.extend(quote! {
                                write!(f, ",")?;
                            });
                        }
                    }
                    if !fields.named.is_empty() {
                        inner_display.extend(quote! {
                            write!(f, "\n")?;
                        });
                    }
                    inner_display.extend(quote! {
                        write!(f, "}}")?;
                    });
                }
                Fields::Unnamed(fields) => {
                    inner_display.extend(quote! {
                        write!(f, " (")?;
                    });
                    for (i, _) in fields.unnamed.iter().enumerate() {
                        let identifier = syn::Index::from(i);
                        inner_display.extend(quote! {
                            write!(f, "\n    {}: {}", stringify!(#identifier), &self.#identifier)?;
                        });
                        if i != fields.unnamed.len() - 1 {
                            inner_display.extend(quote! {
                                write!(f, ",")?;
                            });
                        }
                    }
                    if !fields.unnamed.is_empty() {
                        inner_display.extend(quote! {
                            write!(f, "\n")?;
                        });
                    }
                    inner_display.extend(quote! {
                        write!(f, ")")?;
                    });
                }
                Fields::Unit => {
                    inner_display.extend(quote! {
                        write!(f, ";")?;
                    });
                }
            };

            // Generate some tokens that will be appended after the struct
            let output = quote! {
                impl ::std::fmt::Display for #name {
                    fn fmt(
                        &self,
                        f: &mut ::std::fmt::Formatter<'_>,
                    ) -> ::std::result::Result<(), ::std::fmt::Error> {
                        #inner_display
                        Ok(())
                    }
                }
            };
            output.into()
        }
        Data::Enum(_) | Data::Union(_) => {
            syn::Error::new(input.span(), "DisplayMe can only be used on structs")
                .to_compile_error()
                .into()
        }
    }
}
