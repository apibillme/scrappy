use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

pub fn expand(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let (_, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut generics = ast.generics.clone();
    generics.params.push(parse_quote!(_A: scrappy_actor::Actor));
    generics
        .params
        .push(parse_quote!(_M: scrappy_actor::Message<Result = #name #ty_generics>));
    let (impl_generics, _, _) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::scrappy_actor::dev::MessageResponse<_A, _M> for #name #ty_generics #where_clause {
            fn handle<R: scrappy_actor::dev::ResponseChannel<_M>>(self, _: &mut _A::Context, tx: Option<R>) {
                if let Some(tx) = tx {
                    tx.send(self);
                }
            }
        }
    }
}
