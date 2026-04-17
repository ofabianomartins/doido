use crate::parser::{RouteDecl, RoutesInput};
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(input: RoutesInput) -> TokenStream {
    let mut route_stmts = Vec::new();

    for decl in input.decls {
        match decl {
            RouteDecl::Method { method, path, handler } => {
                let axum_method = syn::Ident::new(&method, proc_macro2::Span::call_site());
                route_stmts.push(quote! {
                    .route(#path, axum::routing::#axum_method(#handler))
                });
            }
        }
    }

    quote! {
        {
            axum::Router::new()
            #(#route_stmts)*
        }
    }
}
