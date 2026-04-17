use syn::{
    parse::{Parse, ParseStream},
    Expr, LitStr, Ident, Result, Token,
};

pub enum RouteDecl {
    Method { method: String, path: LitStr, handler: Expr },
}

pub struct RoutesInput {
    pub decls: Vec<RouteDecl>,
}

impl Parse for RoutesInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut decls = Vec::new();
        while !input.is_empty() {
            let method_ident: Ident = input.parse()?;
            let _bang: Token![!] = input.parse()?;
            let content;
            syn::parenthesized!(content in input);
            let path: LitStr = content.parse()?;
            let _comma: Token![,] = content.parse()?;
            let handler: Expr = content.parse()?;
            let _semi: Option<Token![;]> = input.parse().ok();
            decls.push(RouteDecl::Method {
                method: method_ident.to_string(),
                path,
                handler,
            });
        }
        Ok(RoutesInput { decls })
    }
}
