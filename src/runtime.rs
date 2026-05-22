use rquickjs::{Context, Runtime, Function};

pub struct JsRuntime {
    pub runtime: Runtime,
    pub context: Context,
}

impl JsRuntime {
    pub fn new() -> Self {
        let runtime = Runtime::new().expect("failed to create QuickJS runtime");
        let context = Context::full(&runtime).expect("failed to create QuickJS context");

        // Set up the bridge
        context.with(|ctx| {
            let globals = ctx.globals();

            globals.set("_native_create_node", Function::new(ctx.clone(), |_type: String, _styles: rquickjs::Object, _text: Option<String>| {
                println!("Native: Creating node of type {}", _type);
                0 // Placeholder NodeId
            })).unwrap();

            globals.set("_native_set_style", Function::new(ctx.clone(), |_node_id: u32, _styles: rquickjs::Object| {
                println!("Native: Setting style for node {}", _node_id);
            })).unwrap();
        });

        Self {
            runtime,
            context,
        }
    }

    pub fn eval(&self, source: &str) {
        self.context.with(|ctx| {
            ctx.eval::<(), _>(source).expect("failed to evaluate JS");
        });
    }
}
