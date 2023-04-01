use async_trait::async_trait;

#[derive(Debug)]
pub struct CreateContext<T, R> {
    pub data: T,
    pub resources: R,
    
}
#[derive(Debug)]
pub struct ReadContext<R> {
    pub resources: R,
}
#[derive(Debug)]
pub enum Context<T, R> {
    Create(CreateContext<T, R>),
    Read(ReadContext<R>),
    Update,
    Delete,
}

// A trait that defines the handler interface
#[async_trait]
pub trait Handler<Input>: Send + Sync {
    async fn handle(&self, input: &mut Input) -> Result<(), String>;
}

// A struct that stores the async handlers and provides a method to execute them
pub struct HandlerRegistry<Input> {
    handlers: Vec<Box<dyn Handler<Input>>>,
}

impl<Input> HandlerRegistry<Input> {
    // Create a new, empty handler registry
    pub fn new() -> Self {
        HandlerRegistry { handlers: vec![] }
    }

    // Register a new async handler with the registry
    pub fn register<H: Handler<Input> + 'static>(&mut self, handler: H) {
        self.handlers.push(Box::new(handler));
    }

    // Execute all registered async handlers on the given input struct
    pub async fn exec(&self, input: &mut Input) -> Result<(), String> {
        for handler in &self.handlers {
            handler.handle(input).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn can_build_registry() {
        let mut registry = HandlerRegistry::new();

        struct MyHandler;
        #[derive(Debug)]
        struct Resources;

        #[derive(Debug)]
        struct MyData { foo: String }

        #[async_trait]
        impl Handler<Context<MyData, Resources>> for MyHandler {
            async fn handle(&self, input: &mut Context<MyData,Resources>) -> Result<(), String> {
                match input {
                    Context::Create(create_context) => {
                        create_context.data.foo = "hello world".to_string();
                    }
                    _ => {}
                }
                Ok(())
            }
        }

        registry.register(MyHandler);
        let mut input = Context::Create(CreateContext { data: MyData { foo: "init".to_string() }, resources: Resources });

        let _result = registry.exec(&mut input).await.unwrap();
        dbg!(input);
    }
}

