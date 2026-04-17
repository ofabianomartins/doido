use proc_macro2::TokenStream;
use quote::quote;
use syn::{FnArg, ImplItem, ItemImpl, Pat, PatIdent, PatType, Result, parse2};

fn is_action_method(method: &syn::ImplItemFn) -> bool {
    if method.sig.asyncness.is_none() {
        return false;
    }
    method.sig.inputs.iter().any(|arg| {
        if let FnArg::Typed(PatType { pat, .. }) = arg {
            if let Pat::Ident(PatIdent { ident, .. }) = pat.as_ref() {
                return ident == "ctx";
            }
        }
        false
    })
}

pub fn expand_controller(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let mut impl_block: ItemImpl = parse2(item)?;
    let self_ty = impl_block.self_ty.clone();

    let mut handler_fns: Vec<TokenStream> = Vec::new();

    for impl_item in &impl_block.items {
        let ImplItem::Fn(method) = impl_item else { continue };
        if !is_action_method(method) {
            continue;
        }

        let fn_name = &method.sig.ident;
        let body = &method.block;

        handler_fns.push(quote! {
            pub async fn #fn_name(
                req: ::axum::extract::Request,
            ) -> ::axum::response::Response {
                let (parts, body) = req.into_parts();
                let ctx = ::doido_controller::Context::from_request(parts, body);
                #body
            }
        });
    }

    // Remove action methods from the impl block (replaced by handler fns below)
    impl_block.items.retain(|item| {
        if let ImplItem::Fn(method) = item {
            return !is_action_method(method);
        }
        true
    });

    Ok(quote! {
        #impl_block
        impl #self_ty {
            #(#handler_fns)*
        }
    })
}
