use handlerx::{Context, CreateContext, Handler, HandlerRegistry, ReadContext};
use modelx;
use modelx::{FieldType, Modelx};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

pub trait Module {
    fn options(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }
    fn options_embed(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }
    fn static_init(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }
    fn init(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }
    fn init_inject(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }
    fn boot(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }
}

pub struct Model {}

pub struct Http {}

pub struct DB {}

pub struct Resources {}

impl Module for DB {
    fn options(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }

    fn options_embed(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }

    fn static_init(modelx: &Modelx) -> TokenStream {
        // Iterate through each definition in the Modelx struct
        let impls = modelx.definitions.iter().map(|(name, definition)| {
            // let struct_ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            let struct_name = format!("Model::{}", name);
            let struct_path: syn::Type = syn::parse_str(&struct_name).unwrap();
            let post_statements = definition.fields.iter().map(|(field_name, field)| {
                let prop_access_str = format!("&self.{}", &field_name);
                let prop_access: syn::Expr = syn::parse_str(&prop_access_str).unwrap();
                let quote = match field.field_type {
                    FieldType::String | FieldType::UUID => "\"",
                    FieldType::Integer => "",
                };
                if field.is_key {
                    quote! {
                        post_statement.push(format!("{} = {}{}{}", #field_name, #quote, #prop_access, #quote));
                        post_statement.push(",".to_string());
                    }
                } else {
                    quote! {
                        if let Some(x) = #prop_access {
                            post_statement.push(format!("{} = {}{}{}", #field_name, #quote, x, #quote));
                            post_statement.push(",".to_string());
                        }
                    }
                }
            });
            quote! {
                #[async_trait::async_trait]
                impl DB for #struct_path {
                    async fn createDB(&self, db: &surrealdb::Datastore) {
                       // TODO: - Wait for surrealdb beta 0.9 and use better variant.
                       //       - This trait should use the createContext
                       let ses = surrealdb::Session::for_kv().with_ns("test").with_db("test");
                        let mut post_statement: Vec<String> = vec![];
                        post_statement.push(format!("CREATE {} SET ", #name));
                        #(#post_statements)*
                        post_statement.pop();
                        post_statement.push(";".to_string());
                        let ast = post_statement.join("");
                        db.execute(&ast, &ses, None, false).await.unwrap();
                        // TODO: check for error
                    }
                    async fn readDB(db: &surrealdb::Datastore) -> Self {
                       let ses = surrealdb::Session::for_kv().with_ns("test").with_db("test");
                       let ast = format!("SELECT * FROM {};", #name);
                       let result = db.execute(&ast, &ses, None, false).await.unwrap();
                       let result = result.into_iter().next().map(|rp| rp.result).transpose().unwrap();
                       // TODO: Wait for better API
                    		//  let result = match result {
                    		// Some(surrealdb::sql::Value::Array(arr)) => {
                    		//           let it = arr.into_iter().map(|v| match v {
                    		//               surrealdb::sql::Value::Object(object) => Ok(object),
                    		//               _ => Err("A record was not an Object"),
                    		//           });
                    		//           Ok(it)
                    		//       },
                    		//       _ => Err("nope")
                    		//  };
                    		//
                    		//  let result: Self = result.unwrap().next().unwrap().unwrap().into();
                    		//  dbg!(&result);
                       Self { ..Default::default() }
                    }
                }
            }
        });
        quote! {

            #[async_trait::async_trait]
            pub trait DB {
                async fn createDB(&self, db: &surrealdb::Datastore) {}
                async fn readDB(db: &surrealdb::Datastore) -> Self;
            }
            #(#impls)*
        }
    }

    fn init(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }

    fn init_inject(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }

    fn boot(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }
}

impl Module for Resources {
    fn options(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }

    fn options_embed(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }

    fn static_init(modelx: &Modelx) -> TokenStream {
        let processors = modelx.definitions.iter().map(|(name, _definition)| {
            let struct_field_name = syn::Ident::new(name, proc_macro2::Span::call_site());
            let struct_name = format!("Model::{}", name);
            let struct_path: syn::Type = syn::parse_str(&struct_name).unwrap();
            quote! {
                #struct_field_name: handlerx::HandlerRegistry<handlerx::Context<#struct_path, std::sync::Arc<Resources>>>,
            }
        });
        let processors_new = modelx.definitions.iter().map(|(name, _definition)| {
            let struct_field_name = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote! {
                #struct_field_name: handlerx::HandlerRegistry::new(),
            }
        });
        let processors_db = modelx.definitions.iter()
            .filter(|(_name, definition)| definition.annotations.iter().any(|anno| anno == "@db"))
            .map(|(name, _definition)| {
            let struct_field_name = syn::Ident::new(name, proc_macro2::Span::call_site());
            let handler_name_string = format!("Handler_{}", name);
            let handler_name = syn::Ident::new(&handler_name_string, proc_macro2::Span::call_site());
            let struct_full_name = format!("Model::{}", name);
            let struct_path: syn::Type = syn::parse_str(&struct_full_name).unwrap();
            quote! {
                struct #handler_name;

                #[async_trait::async_trait]
                impl handlerx::Handler<handlerx::Context<#struct_path, std::sync::Arc<Resources>>> for #handler_name {
                    async fn handle(&self, input: &mut handlerx::Context<#struct_path, std::sync::Arc<Resources>>) -> Result<(), String> {
                        println!("Holly molly, it finally works!");
                        match input {
                            handlerx::Context::Create(create_context) => {
                                println!("Write to database");
                                let _result = &create_context.data.createDB(&create_context.resources.db).await;
                            }
                            handlerx::Context::Read(read_context) => {
                               let _result: #struct_path = DB::readDB(&read_context.resources.db).await;
                            }
                            _ => {
                                println!("Not implemented");
                            }
                        }
                        Ok(())
                    }
                }
                new_processor.#struct_field_name.register(#handler_name);
            }
        });
        quote! {
            pub struct Processor {
                #(#processors)*
            }
            impl Processor {
                pub fn new() -> Self {
                    let mut new_processor = Self {
                        #(#processors_new)*
                    };
                    #(#processors_db)*
                    new_processor
                }
            }
            pub struct Resources {
              pub db: surrealdb::Datastore,
              pub processors: Processor,
            }
            impl Resources {
                pub async fn init() -> Self {
                    Resources {
                        db: surrealdb::Datastore::new("memory").await.unwrap(),
                        processors: Processor::new(),
                    }
                }
            }
            impl std::fmt::Debug for Resources {
               fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                 f.debug_struct("Resources").finish()
               }
            }
        }
    }

    fn init(_modelx: &Modelx) -> TokenStream {
        quote! {
            resources: std::sync::Arc::new(Resources::init().await),
        }
    }

    fn init_inject(_modelx: &Modelx) -> TokenStream {
        quote! {
            resources: std::sync::Arc<Resources>,
        }
    }

    fn boot(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }
}

impl Module for Model {
    fn options(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }

    fn options_embed(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }

    fn init(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }

    fn init_inject(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }

    fn boot(_modelx: &Modelx) -> TokenStream {
        quote! {}
    }

    fn static_init(modelx: &Modelx) -> TokenStream {
        // Iterate through each definition in the Modelx struct
        let types = modelx.definitions.iter().map(|(name, definition)| {
            let struct_ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            let fields = definition.fields.iter().map(|(field_name, field)| {
                let field_name = syn::Ident::new(field_name, proc_macro2::Span::call_site());
                let field_type = match field.field_type {
                    FieldType::String => quote! { String },
                    FieldType::UUID => quote! { String },
                    FieldType::Integer => quote! { i32 },
                };
                if field.is_key {
                    quote! {
                        pub #field_name: #field_type
                    }
                } else {
                    quote! {
                        pub #field_name: Option<#field_type>
                    }
                }
            });

            quote! {
                #[derive(Debug, serde::Serialize, serde::Deserialize, Default, sqlx::FromRow)]
                pub struct #struct_ident {
                    #(#fields),*
                }
            }
        });
        quote! {
            pub mod Model {
                #(#types)*
            }
        }
    }
}

impl Module for Http {
    fn options(_modelx: &Modelx) -> TokenStream {
        quote! {
            #[derive(Debug)]
            pub struct OptHttp {
                pub port: u16
            }

            impl Default for OptHttp {
                fn default() -> Self {
                    OptHttp {
                        port: 8080,
                    }
                }
            }
        }
    }

    fn options_embed(_modelx: &Modelx) -> TokenStream {
        quote! {
            pub http: OptHttp,
        }
    }

    fn static_init(modelx: &Modelx) -> TokenStream {
        let http_routes = modelx
            .definitions
            .iter()
            .filter(|(_name, definition)| definition.annotations.iter().any(|anno| anno == "@http"))
            .map(|(name, _definition)| {
                let endpoint = format!("/{}", name);
                let fn_name_get = format!("axum_get_{}", name);
                let fn_name_post = format!("axum_post_{}", name);
                let fn_ident_get = syn::Ident::new(&fn_name_get, proc_macro2::Span::call_site());
                let fn_ident_post = syn::Ident::new(&fn_name_post, proc_macro2::Span::call_site());
                quote! {
                    router = router.route(#endpoint, axum::routing::get(Self::#fn_ident_get));
                    router = router.route(#endpoint, axum::routing::post(Self::#fn_ident_post));
                }
            });

        let mut deploy_statements: Vec<String> = vec![];
        modelx
            .definitions
            .iter()
            .filter(|(_name, definition)| definition.annotations.iter().any(|anno| anno == "@http"))
            .for_each(|(name, definition)| {
                let mut deploy_sql = "".to_string();
                deploy_sql.push_str(&format!("\nDEFINE TABLE {} SCHEMAFULL;", &name));
                definition.fields.iter().for_each(|(field_name, field)| {
                    let sql_type = match &field.field_type {
                        FieldType::String => "string",
                        FieldType::UUID => "string",
                        FieldType::Integer => "int",
                    };
                    let _sql_key = if field.is_key { "PRIMARY KEY" } else { "" };
                    deploy_sql.push_str(&format!(
                        "\nDEFINE FIELD {} on TABLE {} TYPE {};",
                        &field_name, &name, &sql_type
                    ));
                });
                deploy_statements.push(deploy_sql);
            });
        let fn_name = "axum_deploy";
        let fn_ident = syn::Ident::new(&fn_name, proc_macro2::Span::call_site());
        let execute_deploy_ns = quote! {
           let ses = surrealdb::Session::for_kv().with_ns("test").with_db("test");
        };
        let execute_deploy_statements = deploy_statements.iter().map(|statement| {
            quote! {
               println!("{}", #statement);
               resources.db.execute(#statement, &ses, None, false).await.unwrap();
            }
        });
        let axum_deploy_endpoint = quote! {
               pub async fn #fn_ident(axum::Extension(resources): axum::Extension<std::sync::Arc<Resources>>) -> impl axum::response::IntoResponse {
                   println!("Deploy");
                   #execute_deploy_ns
                   #(#execute_deploy_statements)*
                   (axum::http::StatusCode::OK, "success")
             }
        };
        let axum_endpoints = modelx.definitions.iter().filter(|(_name, definition)| definition.annotations.iter().any(|anno| anno == "@http")).map(|(name, definition)| {
        let fn_name_get = format!("axum_get_{}", name);
        let struct_name = format!("Model::{}", name);
        let plain_struct_path: syn::Type = syn::parse_str(&name).unwrap();
        let struct_path: syn::Type = syn::parse_str(&struct_name).unwrap();
        let fn_ident_get = syn::Ident::new(&fn_name_get, proc_macro2::Span::call_site());
        let fn_name_post = format!("axum_post_{}", name);
        let fn_ident_post = syn::Ident::new(&fn_name_post, proc_macro2::Span::call_site());

         quote! {
           pub async fn #fn_ident_post(axum::Extension(resources): axum::Extension<std::sync::Arc<Resources>>, axum::extract::Json(payload): axum::extract::Json<#struct_path>) -> impl axum::response::IntoResponse {
               println!("POST for {}", #name);
               let mut create_context = handlerx::CreateContext::<#struct_path, std::sync::Arc<Resources>>{
                   data: payload,
                   resources: resources.clone(),
               };
               let mut context = handlerx::Context::Create(create_context);
               resources.processors.#plain_struct_path.exec(&mut context).await;

               // let result = &create_context.data.createDB(&resources.db).await;
               let result = "ok".to_string();
               (axum::http::StatusCode::CREATED, axum::Json(result))
           }
           pub async fn #fn_ident_get(axum::Extension(resources): axum::Extension<std::sync::Arc<Resources>>) -> impl axum::response::IntoResponse {
               println!("GET for {}", #name);
               let read_context = handlerx::ReadContext{ resources: resources.clone() };
               let mut context = handlerx::Context::Read(read_context);
               resources.processors.#plain_struct_path.exec(&mut context).await;
               let result = "ok".to_string();
               (axum::http::StatusCode::OK, axum::Json(result))
         }
      }
    });

        quote! {
            #[derive(Debug)]
            pub struct Http {
               pub router: axum::Router,
            }

            impl Http {
              pub async fn init() -> Self {
                      let mut router = axum::Router::new().route("/health", axum::routing::get(Self::health));
                      router = router.route("/deploy", axum::routing::get(Self::axum_deploy));
                      #(#http_routes)*
                      Self {
                        router
                      }
                }
                #(#axum_endpoints)*
                #axum_deploy_endpoint
                pub async fn health() -> impl axum::response::IntoResponse {
                      (axum::http::StatusCode::OK, "healthy")
                }
                pub async fn boot(mut self, resources: std::sync::Arc<Resources>, port: u16) {
                  self.router =  self.router.layer(axum::extract::Extension(resources));
                  let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
                  println!("Listening on http://localhost:{}", &port);
                  axum::Server::bind(&addr)
                      .serve(self.router.into_make_service())
                      .await
                      .unwrap();
                }
            }

        }
    }

    fn init(_modelx: &Modelx) -> TokenStream {
        quote! {
            http: Some(Http::init().await),
        }
    }

    fn init_inject(_modelx: &Modelx) -> TokenStream {
        quote! {
            http: Option<Http>,
        }
    }

    fn boot(_modelx: &Modelx) -> TokenStream {
        quote! {
            let http = self.http.take().unwrap();
            let resources = self.resources.clone();
            http.boot(resources, *&self.options.http.port).await;
        }
    }
}
