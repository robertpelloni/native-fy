mod layout;

use std::collections::HashMap;
use layout::{Node, AstRect, FlexStyles, LayoutEngine};

fn main() {
    println!("Hello, native-fy!");

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
                    margin: "0px 0px 10px".to_string(), // Invalid test case for simple parser
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
            println!("Validation failed (as expected due to complex margin string): {:?}", e);
        }
    }

    // A valid child
    let valid_root_node = Node {
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
                    margin: "10px".to_string(), // Valid!
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

    match engine.build_tree(&valid_root_node) {
        Ok(root_id) => {
            println!("\nValidating second node...");
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
}