use taffy::prelude::*;
use std::collections::HashMap;
use taffy::TaffyError;

#[derive(Debug, Clone)]
pub struct AstRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct FlexStyles {
    pub flex_direction: String,
    pub padding: String,
    pub margin: String,
    pub align_items: String,
    pub justify_content: String,
    pub unsupported: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub node_type: String, // Box, Text, Image, Input, List, Svg
    pub rect: AstRect,
    pub styles: FlexStyles,
    pub text: Option<String>,
    pub value: Option<String>,
    pub children: Vec<Node>,
}

#[derive(Debug)]
pub enum ValidationError {
    UnsupportedProperty(String, String),
    InvalidPropertyValue(String, String),
    UnsupportedNodeType(String),
}

pub struct LayoutEngine {
    taffy: TaffyTree,
    node_metadata: HashMap<NodeId, String>, // NodeId -> Text content
    node_types: HashMap<NodeId, String>,   // NodeId -> Type
    node_values: HashMap<NodeId, String>,  // NodeId -> Value/Src
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
            node_metadata: HashMap::new(),
            node_types: HashMap::new(),
            node_values: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.taffy.clear();
        self.node_metadata.clear();
        self.node_types.clear();
        self.node_values.clear();
    }

    fn parse_length(val: &str) -> Result<LengthPercentageAuto, ValidationError> {
        let val = val.trim();
        if val == "auto" {
            return Ok(LengthPercentageAuto::auto());
        }
        if val.ends_with("px") {
            let px_str = &val[..val.len() - 2];
            if let Ok(px) = px_str.parse::<f32>() {
                return Ok(LengthPercentageAuto::length(px));
            }
        }
        if val.ends_with("%") {
            let pct_str = &val[..val.len() - 1];
            if let Ok(pct) = pct_str.parse::<f32>() {
                return Ok(LengthPercentageAuto::percent(pct / 100.0));
            }
        }
        if val == "0" || val == "0px" {
            return Ok(LengthPercentageAuto::length(0.0));
        }
        Err(ValidationError::InvalidPropertyValue("length".to_string(), val.to_string()))
    }

    fn parse_rect_length_auto(val: &str) -> Result<taffy::Rect<LengthPercentageAuto>, ValidationError> {
        let parts: Vec<&str> = val.split_whitespace().collect();
        match parts.len() {
            1 => {
                let l = Self::parse_length(parts[0])?;
                Ok(taffy::Rect { left: l.clone(), right: l.clone(), top: l.clone(), bottom: l })
            }
            2 => {
                let y = Self::parse_length(parts[0])?;
                let x = Self::parse_length(parts[1])?;
                Ok(taffy::Rect { left: x.clone(), right: x, top: y.clone(), bottom: y })
            }
            _ => Err(ValidationError::InvalidPropertyValue("padding/margin".to_string(), val.to_string()))
        }
    }

    fn parse_length_percentage(val: &str) -> Result<LengthPercentage, ValidationError> {
        let val = val.trim();
        if val.ends_with("px") {
            let px_str = &val[..val.len() - 2];
            if let Ok(px) = px_str.parse::<f32>() {
                return Ok(LengthPercentage::length(px));
            }
        }
        if val.ends_with("%") {
            let pct_str = &val[..val.len() - 1];
            if let Ok(pct) = pct_str.parse::<f32>() {
                return Ok(LengthPercentage::percent(pct / 100.0));
            }
        }
        if val == "0" || val == "0px" {
            return Ok(LengthPercentage::length(0.0));
        }
        Err(ValidationError::InvalidPropertyValue("length percentage".to_string(), val.to_string()))
    }

    fn parse_rect_length_percentage(val: &str) -> Result<taffy::Rect<LengthPercentage>, ValidationError> {
        let parts: Vec<&str> = val.split_whitespace().collect();
        match parts.len() {
            1 => {
                let l = Self::parse_length_percentage(parts[0])?;
                Ok(taffy::Rect { left: l.clone(), right: l.clone(), top: l.clone(), bottom: l })
            }
            2 => {
                let y = Self::parse_length_percentage(parts[0])?;
                let x = Self::parse_length_percentage(parts[1])?;
                Ok(taffy::Rect { left: x.clone(), right: x, top: y.clone(), bottom: y })
            }
            _ => Err(ValidationError::InvalidPropertyValue("padding/margin".to_string(), val.to_string()))
        }
    }

    pub fn build_tree(&mut self, node: &Node) -> Result<NodeId, ValidationError> {
        // Validate Node Type
        match node.node_type.as_str() {
            "Box" | "Text" | "Image" | "Input" | "List" | "Svg" => {}
            _ => return Err(ValidationError::UnsupportedNodeType(node.node_type.clone())),
        }

        // Map Styles to Taffy
        let mut style = Style::DEFAULT;

        style.display = Display::Flex;

        style.flex_direction = match node.styles.flex_direction.as_str() {
            "row" => FlexDirection::Row,
            "column" => FlexDirection::Column,
            "row-reverse" => FlexDirection::RowReverse,
            "column-reverse" => FlexDirection::ColumnReverse,
            val => return Err(ValidationError::InvalidPropertyValue("flex-direction".to_string(), val.to_string())),
        };

        style.align_items = match node.styles.align_items.as_str() {
            "flex-start" | "start" => Some(AlignItems::FlexStart),
            "flex-end" | "end" => Some(AlignItems::FlexEnd),
            "center" => Some(AlignItems::Center),
            "baseline" => Some(AlignItems::Baseline),
            "stretch" | "normal" => Some(AlignItems::Stretch),
            val => return Err(ValidationError::InvalidPropertyValue("align-items".to_string(), val.to_string())),
        };

        style.justify_content = match node.styles.justify_content.as_str() {
            "flex-start" | "start" | "normal" => Some(JustifyContent::FlexStart),
            "flex-end" | "end" => Some(JustifyContent::FlexEnd),
            "center" => Some(JustifyContent::Center),
            "space-between" => Some(JustifyContent::SpaceBetween),
            "space-around" => Some(JustifyContent::SpaceAround),
            "space-evenly" => Some(JustifyContent::SpaceEvenly),
            val => return Err(ValidationError::InvalidPropertyValue("justify-content".to_string(), val.to_string())),
        };

        style.padding = Self::parse_rect_length_percentage(&node.styles.padding)?;
        style.margin = Self::parse_rect_length_auto(&node.styles.margin)?;

        style.size = Size {
            width: Dimension::length(node.rect.width),
            height: Dimension::length(node.rect.height),
        };

        let mut child_ids = Vec::new();
        for child in &node.children {
            child_ids.push(self.build_tree(child)?);
        }

        let node_id = self.taffy.new_leaf(style).unwrap();
        self.taffy.set_children(node_id, &child_ids).unwrap();

        if let Some(text) = &node.text {
            self.node_metadata.insert(node_id, text.clone());
        }

        if let Some(value) = &node.value {
            self.node_values.insert(node_id, value.clone());
        }

        self.node_types.insert(node_id, node.node_type.clone());

        Ok(node_id)
    }

    pub fn add_child(&mut self, parent_id: NodeId, child_id: NodeId) -> Result<(), TaffyError> {
        let mut children = self.taffy.children(parent_id)?;
        children.push(child_id);
        self.taffy.set_children(parent_id, &children)
    }

    pub fn compute(&mut self, root_id: NodeId) -> Result<(), taffy::TaffyError> {
        self.taffy.compute_layout(root_id, Size::MAX_CONTENT)
    }

    pub fn layout(&self, id: NodeId) -> Option<&Layout> {
        self.taffy.layout(id).ok()
    }

    pub fn children(&self, id: NodeId) -> Option<Vec<NodeId>> {
        self.taffy.children(id).ok()
    }

    pub fn get_text(&self, id: NodeId) -> Option<&String> {
        self.node_metadata.get(&id)
    }

    pub fn get_type(&self, id: NodeId) -> Option<&String> {
        self.node_types.get(&id)
    }

    pub fn get_value(&self, id: NodeId) -> Option<&String> {
        self.node_values.get(&id)
    }

    pub fn node_count(&self) -> usize {
        self.taffy.total_node_count()
    }
}
