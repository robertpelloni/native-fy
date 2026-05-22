use rquickjs::{Context, Runtime, Function};
use std::sync::mpsc::Sender;

pub enum UiCommand {
    CreateNode {
        node_type: String,
        styles: std::collections::HashMap<String, String>,
        text: Option<String>,
    },
}

pub struct JsRuntime {
    pub runtime: Runtime,
    pub context: Context,
}

impl JsRuntime {
    pub fn new(tx: Sender<UiCommand>) -> Self {
        let runtime = Runtime::new().expect("failed to create QuickJS runtime");
        let context = Context::full(&runtime).expect("failed to create QuickJS context");

        // Set up the bridge
        context.with(|ctx| {
            let globals = ctx.globals();

            let tx_create = tx.clone();
            globals.set("_native_create_node", Function::new(ctx.clone(), move |_type: String, _styles: rquickjs::Object, _text: Option<String>| {
                let styles = std::collections::HashMap::new();
                let _ = tx_create.send(UiCommand::CreateNode {
                    node_type: _type,
                    styles,
                    text: _text,
                });
                0 // Placeholder NodeId
            })).unwrap();

            globals.set("_native_set_style", Function::new(ctx.clone(), |_node_id: u32, _styles: rquickjs::Object| {
                // println!("Native: Setting style for node {}", _node_id);
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

    pub fn dispatch_click(&self, x: f32, y: f32) {
        self.context.with(|ctx| {
            let globals = ctx.globals();
            if let Ok(handler) = globals.get::<_, Function>("_native_on_event") {
                let data = rquickjs::Object::new(ctx.clone()).unwrap();
                let _ = data.set("x", x);
                let _ = data.set("y", y);
                let _ = handler.call::<(String, rquickjs::Object), ()>(("click".to_string(), data));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Instant;

    #[test]
    fn test_js_node_creation_performance() {
        let (tx, rx) = mpsc::channel();
        let runtime = JsRuntime::new(tx);

        let counts = [100, 500, 1000];
        for count in counts {
            let start = Instant::now();
            runtime.eval(&format!(
                "for (let i = 0; i < {}; i++) {{ _native_create_node('Box', {{}}, null); }}",
                count
            ));
            let duration = start.elapsed();
            println!("JS Performance: Created {} nodes in {:?}", count, duration);

            // Verify nodes received in channel
            for _ in 0..count {
                rx.try_recv().expect("node not received in channel");
            }
        }
    }
}
