use taffy::prelude::*;
use std::collections::HashMap;

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
    pub node_type: String, // Box, Text, Image, Input, List
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
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
        }
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
            "Box" | "Text" | "Image" | "Input" | "List" => {}
            _ => return Err(ValidationError::UnsupportedNodeType(node.node_type.clone())),
        }

        // Validate Unsupported Styles
        if let Some((k, v)) = node.styles.unsupported.iter().next() {
            return Err(ValidationError::UnsupportedProperty(k.clone(), v.clone()));
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

        Ok(node_id)
    }

    pub fn compute(&mut self, root_id: NodeId) -> Result<(), taffy::TaffyError> {
        self.taffy.compute_layout(root_id, Size::MAX_CONTENT)
    }

    pub fn print_layout(&self, id: NodeId, prefix: &str) {
        let layout = self.taffy.layout(id).unwrap();
        println!("{prefix} SIZE: {}x{}, POS: {},{}", layout.size.width, layout.size.height, layout.location.x, layout.location.y);
        for child in self.taffy.children(id).unwrap() {
            self.print_layout(child, &format!("{prefix}  "));
        }
    }
}