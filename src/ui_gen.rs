use crate::layout::{Node, AstRect, FlexStyles, LayoutEngine};
use taffy::prelude::NodeId;
use std::collections::HashMap;

pub fn generate_ui_tree(engine: &mut LayoutEngine) -> NodeId {

    let new_node = Node {
        node_type: "Box".to_string(),
        rect: AstRect { x: 256.0, y: 108.0, width: 768.0, height: 117.0 },
        styles: FlexStyles {
            flex_direction: "row".to_string(),
            padding: "0px".to_string(),
            margin: "0px".to_string(),
            align_items: "normal".to_string(),
            justify_content: "normal".to_string(),
            unsupported: HashMap::new(),
        },
        text: None,
        value: None,
        children: vec![
            Node {
                node_type: "Text".to_string(),
                rect: AstRect { x: 256.0, y: 108.0, width: 768.0, height: 28.0 },
                styles: FlexStyles {
                    flex_direction: "row".to_string(),
                    padding: "0px".to_string(),
                    margin: "0px".to_string(),
                    align_items: "normal".to_string(),
                    justify_content: "normal".to_string(),
                    unsupported: HashMap::new(),
                },
                text: Some("Example Domain".to_string()),
                value: None,
                children: vec![],
            }
        ]
    };
    engine.build_tree(&new_node).unwrap_or(taffy::prelude::NodeId::new(0))

}