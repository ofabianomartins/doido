use crate::parser::{ResourceFilter, RouteDecl, RoutesInput};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};

fn is_active(action: &str, filter: &ResourceFilter) -> bool {
    match filter {
        ResourceFilter::All => true,
        ResourceFilter::Only(list) => list.iter().any(|a| a == action),
        ResourceFilter::Except(list) => !list.iter().any(|a| a == action),
    }
}

pub fn generate(input: RoutesInput) -> TokenStream {
    let mut route_stmts = Vec::new();
    let mut helper_fns = Vec::new();

    for decl in input.decls {
        match decl {
            RouteDecl::Method { method, path, handler } => {
                let axum_method = syn::Ident::new(&method, Span::call_site());
                route_stmts.push(quote! {
                    .route(#path, axum::routing::#axum_method(#handler))
                });
            }
            RouteDecl::Resources { resource_name, controller, filter } => {
                let name = resource_name.to_string();
                let singular = name.trim_end_matches('s').to_string();
                let base = format!("/{}", name);
                let base_new = format!("/{}/new", name);
                let base_id = format!("/{}/:id", name);
                let base_id_edit = format!("/{}/:id/edit", name);
                let ctrl = &controller;

                let mut collection = quote! { axum::routing::MethodRouter::new() };
                if is_active("index", &filter) {
                    collection = quote! { #collection.get(#ctrl::index) };
                }
                if is_active("create", &filter) {
                    collection = quote! { #collection.post(#ctrl::create) };
                }
                route_stmts.push(quote! { .route(#base, #collection) });

                if is_active("new", &filter) {
                    route_stmts.push(quote! { .route(#base_new, axum::routing::get(#ctrl::new)) });
                }

                let mut member = quote! { axum::routing::MethodRouter::new() };
                if is_active("show", &filter) {
                    member = quote! { #member.get(#ctrl::show) };
                }
                if is_active("update", &filter) {
                    member = quote! { #member.patch(#ctrl::update).put(#ctrl::update) };
                }
                if is_active("destroy", &filter) {
                    member = quote! { #member.delete(#ctrl::destroy) };
                }
                route_stmts.push(quote! { .route(#base_id, #member) });

                if is_active("edit", &filter) {
                    route_stmts.push(quote! { .route(#base_id_edit, axum::routing::get(#ctrl::edit)) });
                }

                // URL helpers (block-scoped to the routes! expansion)
                let collection_fn = format_ident!("{}_path", name);
                let new_fn = format_ident!("new_{}_path", singular);
                let member_fn = format_ident!("{}_path", singular);
                let edit_fn = format_ident!("edit_{}_path", singular);

                helper_fns.push(quote! {
                    #[allow(dead_code)]
                    fn #collection_fn() -> &'static str { #base }
                    #[allow(dead_code)]
                    fn #new_fn() -> &'static str { #base_new }
                    #[allow(dead_code)]
                    fn #member_fn(id: impl ::std::fmt::Display) -> String {
                        format!("{}/{}", #base, id)
                    }
                    #[allow(dead_code)]
                    fn #edit_fn(id: impl ::std::fmt::Display) -> String {
                        format!("{}/{}/edit", #base, id)
                    }
                });
            }
        }
    }

    quote! {
        {
            #(#helper_fns)*
            axum::Router::new()
            #(#route_stmts)*
        }
    }
}
