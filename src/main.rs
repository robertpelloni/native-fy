mod layout;

use std::collections::HashMap;
use std::sync::Arc;
use layout::{Node, AstRect, FlexStyles, LayoutEngine};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

#[derive(Default)]
struct NativefyApp {
    window: Option<Arc<Window>>,
}

impl ApplicationHandler for NativefyApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Native-fy UI Engine")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

            self.window = Some(Arc::new(event_loop.create_window(window_attributes).unwrap()));
            println!("Window successfully initialized and resumed!");
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Future wgpu rendering will go here
            }
            _ => (),
        }
    }
}

fn main() {
    println!("Initializing Native-fy Windowing Environment...");

    // Create a mock AST node matching the expected structure
    let root_node = Node {
        node_type: "Box".to_string(),
        rect: AstRect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 },
        styles: FlexStyles {
            flex_direction: "column".to_string(),
            padding: "20px".to_string(),
            margin: "0px".to_string(),
            align_items: "center".to_string(),
            justify_content: "center".to_string(),
            unsupported: HashMap::new(),
        },
        text: None,
        value: None,
        children: vec![
            Node {
                node_type: "Text".to_string(),
                rect: AstRect { x: 0.0, y: 0.0, width: 100.0, height: 20.0 },
                styles: FlexStyles {
                    flex_direction: "row".to_string(),
                    padding: "0px".to_string(),
                    margin: "10px".to_string(),
                    align_items: "normal".to_string(),
                    justify_content: "normal".to_string(),
                    unsupported: HashMap::new(),
                },
                text: Some("Test Text".to_string()),
                value: None,
                children: vec![],
            }
        ],
    };

    let mut engine = LayoutEngine::new();

    match engine.build_tree(&root_node) {
        Ok(root_id) => {
            println!("Successfully built Taffy tree. Computing layout...");
            if let Err(e) = engine.compute(root_id) {
                println!("Layout computation failed: {:?}", e);
            } else {
                engine.print_layout(root_id, "");
            }
        }
        Err(e) => {
            println!("Validation failed: {:?}", e);
        }
    }

    // Initialize winit event loop
    let event_loop = EventLoop::new().unwrap();
    let mut app = NativefyApp::default();

    println!("Starting event loop...");
    event_loop.run_app(&mut app).unwrap();
}