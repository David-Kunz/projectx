use corex;
use corex::Module;
use modelx::{FieldType, Modelx};
use handlerx;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, LitStr};

#[proc_macro_attribute]
pub fn import(attr: TokenStream, input: TokenStream) -> TokenStream {
    let filepath = parse_macro_input!(attr as LitStr).value();
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct
    let struct_name = &input.ident;

    // Load the Modelx struct from the filepath
    let modelx = Modelx::load(filepath);

    let http_options = corex::Http::options(&modelx);
    let http_options_embed = corex::Http::options_embed(&modelx);
    let options = quote! {
        #http_options
        #[derive(Default, Debug)]
        pub struct Opt {
            #http_options_embed
        }
    };

    let http_boot = corex::Http::boot(&modelx);
    let http_init_inject = corex::Http::init_inject(&modelx);
    let process_init_inject = corex::Resources::init_inject(&modelx);
    let boot = quote! {
        pub async fn boot(&mut self) {
            #http_boot
        }
    };

    let http_static_init = corex::Http::static_init(&modelx);
    let model_static_init = corex::Model::static_init(&modelx);
    let db_static_init = corex::DB::static_init(&modelx);
    let process_static_init = corex::Resources::static_init(&modelx);
    let static_init = quote! {
        #model_static_init
        #http_static_init
        #db_static_init
        #process_static_init
    };

    let http_init = corex::Http::init(&modelx);
    let process_init = corex::Resources::init(&modelx);

    // Return the generated structs as a TokenStream
    TokenStream::from(quote! {
        #static_init
        #options

        #[derive(Debug)]
        pub struct #struct_name {
            options: Opt,
            #http_init_inject
            #process_init_inject
        }
        impl #struct_name {
            pub async fn init(options: Opt) -> Self {
                Self {
                  options,
                  #http_init
                  #process_init
                }
            }
            #boot
        }
    })
}
