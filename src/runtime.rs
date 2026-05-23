use rquickjs::{Context, Runtime, Function};
use std::sync::mpsc::Sender;
use std::collections::HashMap;

pub enum UiCommand {
    CreateNode {
        node_type: String,
        styles: HashMap<String, String>,
        text: Option<String>,
    },
    CreateNativeButton {
        text: String,
        styles: HashMap<String, String>,
    },
    UpdateImage {
        url: String,
        data: Vec<u8>,
    },
    SyncProtocol,
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
                let mut styles = HashMap::new();

                // Extract styles from JS object
                for key_res in _styles.keys::<String>() {
                    if let Ok(key) = key_res {
                        if let Ok(val) = _styles.get::<String, String>(key.clone()) {
                            styles.insert(key, val);
                        }
                    }
                }

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

            let tx_fetch = tx.clone();
            globals.set("_native_fetch", Function::new(ctx.clone(), move |url: String| {
                let tx = tx_fetch.clone();
                let url_clone = url.clone();
                std::thread::spawn(move || {
                    if let Ok(resp) = reqwest::blocking::get(&url_clone) {
                        if let Ok(bytes) = resp.bytes() {
                            let _ = tx.send(UiCommand::UpdateImage {
                                url: url_clone,
                                data: bytes.to_vec(),
                            });
                        }
                    }
                });
                "Asset loading started...".to_string()
            })).unwrap();

            let tx_sync = tx.clone();
            globals.set("_native_sync_protocol", Function::new(ctx.clone(), move || {
                let _ = tx_sync.send(UiCommand::SyncProtocol);
            })).unwrap();

            let tx_btn = tx.clone();
            globals.set("_native_create_button", Function::new(ctx.clone(), move |text: String, _styles: rquickjs::Object| {
                let mut styles = HashMap::new();
                for key_res in _styles.keys::<String>() {
                    if let Ok(key) = key_res {
                        if let Ok(val) = _styles.get::<String, String>(key.clone()) {
                            styles.insert(key, val);
                        }
                    }
                }
                let _ = tx_btn.send(UiCommand::CreateNativeButton { text, styles });
            })).unwrap();

            globals.set("_native_get_metadata", Function::new(ctx.clone(), || {
                let version = include_str!("../VERSION.md").trim();
                let todo = include_str!("../TODO.md");
                let mut meta = HashMap::new();
                meta.insert("version".to_string(), version.to_string());
                meta.insert("todo".to_string(), todo.to_string());
                meta
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
                "for (let i = 0; i < {}; i++) {{ _native_create_node('Box', {{ padding: '10px' }}, null); }}",
                count
            ));
            let duration = start.elapsed();
            println!("JS Performance: Created {} nodes with styles in {:?}", count, duration);

            // Verify nodes and styles received in channel
            for _ in 0..count {
                if let UiCommand::CreateNode { styles, .. } = rx.try_recv().expect("node not received") {
                    assert_eq!(styles.get("padding").unwrap(), "10px");
                }
            }
        }
    }
}
