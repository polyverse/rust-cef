extern crate proc_macro;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use crate::proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(ToCef)]
pub fn writable_template_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // type name
    let name = &input.ident;

    // generics
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics rust_cef::ToCef for #name #ty_generics #where_clause {
            fn write(&self, dest: &::std::path::Path) -> ::std::io::Result<()> {
                let mut file = ::std::io::BufWriter::new(::std::fs::File::create(dest)?);
                file.write(self.render().unwrap().trim().as_bytes())?;

                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}
