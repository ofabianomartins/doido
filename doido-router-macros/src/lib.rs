use proc_macro::TokenStream;

#[proc_macro]
pub fn routes(input: TokenStream) -> TokenStream {
    let _ = input;
    TokenStream::new()
}
