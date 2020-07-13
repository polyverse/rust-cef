extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use crate::proc_macro::TokenStream;
use std::convert::From;
use syn::{Attribute, DeriveInput, Error as SynError, Lit, Meta, MetaNameValue, NestedMeta};

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
    let input = parse_macro_input!(input as DeriveInput);

    // type name
    let name = &input.ident;

    // generics
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // attributes
    let attrs = input.attrs;
    for attr in attrs {
        let parsedattr = attr.parse_meta();
        println!("{:#?}", parsedattr);
    }

    // ToCef can be the default implementation
    let to_cef_impl = quote! {
        impl #impl_generics rust_cef::CefHeaderVersion for #name #ty_generics #where_clause {
            fn cef_header_version(&self) -> rust_cef::CefResult {
                Ok("0".to_owned())
            }
        }
    };

    TokenStream::from(to_cef_impl)
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
    let to_cef_impl = quote! {
        impl #impl_generics rust_cef::CefHeaderDeviceVendor for #name #ty_generics #where_clause {
            fn cef_header_device_vendor(&self) -> rust_cef::CefResult {
                Ok("polyverse".to_owned())
            }
        }
    };

    TokenStream::from(to_cef_impl)
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
    let to_cef_impl = quote! {
        impl #impl_generics rust_cef::CefHeaderDeviceProduct for #name #ty_generics #where_clause {
            fn cef_header_device_product(&self) -> rust_cef::CefResult {
                Ok("zerotect".to_owned())
            }
        }
    };

    TokenStream::from(to_cef_impl)
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
    let to_cef_impl = quote! {
        impl #impl_generics rust_cef::CefHeaderDeviceVersion for #name #ty_generics #where_clause {
            fn cef_header_device_version(&self) -> rust_cef::CefResult {
                Ok("V1".to_owned())
            }
        }
    };

    TokenStream::from(to_cef_impl)
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
    let to_cef_impl = quote! {
        impl #impl_generics rust_cef::CefHeaderDeviceEventClassID for #name #ty_generics #where_clause {
            fn cef_header_device_event_class_id(&self) -> rust_cef::CefResult {
                Ok("LinuxKernelTrap".to_owned())
            }
        }
    };

    TokenStream::from(to_cef_impl)
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
    let to_cef_impl = quote! {
        impl #impl_generics rust_cef::CefHeaderName for #name #ty_generics #where_clause {
            fn cef_header_name(&self) -> rust_cef::CefResult {
                Ok("Linux Kernel Trap".to_owned())
            }
        }
    };

    TokenStream::from(to_cef_impl)
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
    let to_cef_impl = quote! {
        impl #impl_generics rust_cef::CefHeaderSeverity for #name #ty_generics #where_clause {
            fn cef_header_severity(&self) -> rust_cef::CefResult {
                Ok("10".to_owned())
            }
        }
    };

    TokenStream::from(to_cef_impl)
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
    let to_cef_impl = quote! {
        impl #impl_generics rust_cef::CefExtensions for #name #ty_generics #where_clause {
            fn cef_extensions(&self) -> rust_cef::CefResult {
                Ok("customField=customValue".to_owned())
            }
        }
    };

    TokenStream::from(to_cef_impl)
}
