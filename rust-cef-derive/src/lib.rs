extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use crate::proc_macro::TokenStream;
use std::convert::From;
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Error as SynError};

type DeriveResult = Result<TokenStream, TokenStream>;

#[proc_macro_derive(ToCef)]
pub fn to_cef_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // type name
    let name = &input.ident;

    // generics
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // ToCef can be the default implementation
    let to_cef_impl = quote! {
        impl #impl_generics rust_cef::ToCef for #name #ty_generics #where_clause {}
    };

    TokenStream::from(to_cef_impl)
}

#[proc_macro_derive(CefHeaderVersion, attributes(cef_header_version_fixed))]
pub fn cef_header_version_derive(input: TokenStream) -> TokenStream {
    cef_header_derive(input, "cef_header_version_fixed", "CefHeaderVersion", "cef_header_version")
}

#[proc_macro_derive(CefHeaderDeviceVendor, attributes(cef_header_device_vendor_fixed))]
pub fn cef_header_device_vendor_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // type name
    let name = &input.ident;

    // generics
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // ToCef can be the default implementation
    let trait_impl = quote! {
        impl #impl_generics rust_cef::CefHeaderDeviceVendor for #name #ty_generics #where_clause {
            fn cef_header_device_vendor(&self) -> rust_cef::CefResult {
                Ok("polyverse".to_owned())
            }
        }
    };

    TokenStream::from(trait_impl)
}

#[proc_macro_derive(CefHeaderDeviceProduct, attributes(cef_header_device_product_fixed))]
pub fn cef_header_device_product_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // type name
    let name = &input.ident;

    // generics
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // ToCef can be the default implementation
    let trait_impl = quote! {
        impl #impl_generics rust_cef::CefHeaderDeviceProduct for #name #ty_generics #where_clause {
            fn cef_header_device_product(&self) -> rust_cef::CefResult {
                Ok("zerotect".to_owned())
            }
        }
    };

    TokenStream::from(trait_impl)
}

#[proc_macro_derive(CefHeaderDeviceVersion, attributes(cef_header_device_version_fixed))]
pub fn cef_header_device_version_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // type name
    let name = &input.ident;

    // generics
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // ToCef can be the default implementation
    let trait_impl = quote! {
        impl #impl_generics rust_cef::CefHeaderDeviceVersion for #name #ty_generics #where_clause {
            fn cef_header_device_version(&self) -> rust_cef::CefResult {
                Ok("V1".to_owned())
            }
        }
    };

    TokenStream::from(trait_impl)
}

#[proc_macro_derive(
    CefHeaderDeviceEventClassID,
    attributes(cef_header_device_event_class_id_fixed)
)]
pub fn cef_header_device_event_class_id_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // type name
    let name = &input.ident;

    // generics
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // ToCef can be the default implementation
    let trait_impl = quote! {
        impl #impl_generics rust_cef::CefHeaderDeviceEventClassID for #name #ty_generics #where_clause {
            fn cef_header_device_event_class_id(&self) -> rust_cef::CefResult {
                Ok("LinuxKernelTrap".to_owned())
            }
        }
    };

    TokenStream::from(trait_impl)
}

#[proc_macro_derive(CefHeaderName, attributes(cef_header_name_fixed))]
pub fn cef_header_name_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // type name
    let name = &input.ident;

    // generics
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // ToCef can be the default implementation
    let trait_impl = quote! {
        impl #impl_generics rust_cef::CefHeaderName for #name #ty_generics #where_clause {
            fn cef_header_name(&self) -> rust_cef::CefResult {
                Ok("Linux Kernel Trap".to_owned())
            }
        }
    };

    TokenStream::from(trait_impl)
}

#[proc_macro_derive(CefHeaderSeverity, attributes(cef_header_severity_fixed))]
pub fn cef_header_severity_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // type name
    let name = &input.ident;

    // generics
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // ToCef can be the default implementation
    let trait_impl = quote! {
        impl #impl_generics rust_cef::CefHeaderSeverity for #name #ty_generics #where_clause {
            fn cef_header_severity(&self) -> rust_cef::CefResult {
                Ok("10".to_owned())
            }
        }
    };

    TokenStream::from(trait_impl)
}

#[proc_macro_derive(CefExtensions)]
pub fn cef_extensions_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // type name
    let name = &input.ident;

    // generics
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // ToCef can be the default implementation
    let trait_impl = quote! {
        impl #impl_generics rust_cef::CefExtensions for #name #ty_generics #where_clause {
            fn cef_extensions(&self) -> rust_cef::CefResult {
                Ok("customField=customValue".to_owned())
            }
        }
    };

    TokenStream::from(trait_impl)
}

fn cef_header_derive(input: TokenStream, fixed_value_attr: &str, trait_name: &str, method_name: &str, ) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Span
    let span = input.span();

    // type name
    let name = &input.ident;

    // generics
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // look in attributes for a fixed value
    let value_impl = match fixed_value_header(input.attrs, fixed_value_attr) {
        Some(ts) => ts,
        None => {
            println!("No fixed value attribute found: {}", fixed_value_attr);
            let message = format!(r#"When deriving trait {}, no attribute was found to define a fixed value for it (i.e. #[{}(value)])"#,
                trait_name, fixed_value_attr);
            return TokenStream::from(SynError::new(span, message).to_compile_error());
        },
    };

    let trait_name_ident = format_ident!("{}", trait_name);
    let method_name_ident = format_ident!("{}", method_name);

    // ToCef can be the default implementation
    let trait_impl = quote! {
        impl #impl_generics rust_cef::#trait_name_ident for #name #ty_generics #where_clause {
            fn #method_name_ident(&self) -> rust_cef::CefResult {
                #value_impl
            }
        }
    };

    TokenStream::from(trait_impl)
}

fn fixed_value_header(_attrs: Vec<Attribute>, _header: &str) -> Option<proc_macro2::TokenStream> {
    None
}

