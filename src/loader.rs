use serde_json::{Map, Value};
use std::string;

use crate::syntax::*;
use crate::parser::{Event, EventReceiver, ParseResult, Parser};

pub struct Loader {
    value_stack: Vec<Jsona>,
    key_stack: Vec<Option<(Position, string::String)>>,
    annotation_name: Option<(Position, string::String)>,
    annotation_value_stack: Vec<Value>,
    annotation_key_stack: Vec<Option<string::String>>,
}

impl Loader {
    pub fn load_from_str(input: &str) -> ParseResult<Jsona> {
        let mut loader = Loader {
            value_stack: Vec::new(),
            key_stack: Vec::new(),
            annotation_name: None,
            annotation_value_stack: Vec::new(),
            annotation_key_stack: Vec::new(),
        };
        let mut parser = Parser::new(input.chars());
        parser.parse(&mut loader)?;
        Ok(loader.value_stack.pop().unwrap())
    }
    fn insert_ast_node(&mut self, node: Jsona) {
        if self.value_stack.is_empty() {
            self.value_stack.push(node);
        } else {
            let parent = self.value_stack.last_mut().unwrap();
            match *parent {
                Jsona::Array(Array {
                    ref mut elements, ..
                }) => elements.push(node),
                Jsona::Object(Object {
                    ref mut properties, ..
                }) => {
                    let cur_key = self.key_stack.pop().unwrap();
                    let new_key = match cur_key {
                        Some((position, key)) => {
                            properties.push(Property {
                                key,
                                position,
                                value: node,
                            });
                            None
                        }
                        None => {
                            if let Jsona::String(String {
                                value, position, ..
                            }) = node
                            {
                                Some((position, value))
                            } else {
                                unreachable!()
                            }
                        }
                    };
                    self.key_stack.push(new_key);
                }
                _ => unreachable!(),
            }
        }
    }
    fn insert_annotation_node(&mut self, node: Value) {
        if self.annotation_value_stack.is_empty() {
            self.annotation_value_stack.push(node);
        } else {
            let parent = self.annotation_value_stack.last_mut().unwrap();
            match *parent {
                Value::Array(ref mut elements) => elements.push(node),
                Value::Object(ref mut properties) => {
                    let cur_key = self.annotation_key_stack.pop().unwrap();
                    let new_key = match cur_key {
                        Some(key) => {
                            properties.insert(key, node);
                            None
                        }
                        None => {
                            if let Value::String(value) = node {
                                Some(value)
                            } else {
                                unreachable!()
                            }
                        }
                    };
                    self.annotation_key_stack.push(new_key);
                }
                _ => unreachable!(),
            }
        }
    }
    fn insert_annotation_value(&mut self, anno: Annotation) {
        let parent = self.value_stack.last_mut().unwrap();
        match *parent {
            Jsona::Array(Array {
                ref mut elements, ..
            }) => {
                if elements.len() > 0 {
                    let last_elem = elements.last_mut().unwrap();
                    last_elem.get_annotations_mut().push(anno)
                } else {
                    parent.get_annotations_mut().push(anno);
                }
            }
            Jsona::Object(Object {
                ref mut properties, ..
            }) => {
                if properties.len() > 0 {
                    let last_prop = properties.last_mut().unwrap();
                    last_prop.value.get_annotations_mut().push(anno)
                } else {
                    parent.get_annotations_mut().push(anno);
                }
            }
            _ => parent.get_annotations_mut().push(anno),
        }
    }
}

impl EventReceiver for Loader {
    fn on_event(&mut self, event: Event, position: Position) {
        match event {
            Event::AnnotationStart(value) => {
                self.annotation_name = Some((position, value));
            }
            Event::AnnotationEnd => {
                let (position, name) = self.annotation_name.take().unwrap();
                let value = self.annotation_value_stack.pop().unwrap();
                self.insert_annotation_value(Annotation {
                    name,
                    position,
                    value,
                });
            }
            Event::ArrayStart => {
                if self.annotation_name.is_none() {
                    self.value_stack.push(Jsona::Array(Array {
                        elements: Vec::new(),
                        annotations: Vec::new(),
                        position,
                    }));
                } else {
                    self.annotation_value_stack.push(Value::Array(Vec::new()));
                }
            }
            Event::ArrayStop => {
                if self.annotation_name.is_none() {
                    let node = self.value_stack.pop().unwrap();
                    self.insert_ast_node(node);
                } else {
                    let node = self.annotation_value_stack.pop().unwrap();
                    self.insert_annotation_node(node);
                }
            }
            Event::ObjectStart => {
                if self.annotation_name.is_none() {
                    self.key_stack.push(None);
                    self.value_stack.push(Jsona::Object(Object {
                        properties: Vec::new(),
                        annotations: Vec::new(),
                        position,
                    }));
                } else {
                    self.annotation_key_stack.push(None);
                    self.annotation_value_stack.push(Value::Object(Map::new()));
                }
            }
            Event::ObjectStop => {
                if self.annotation_name.is_none() {
                    self.key_stack.pop().unwrap();
                    let node = self.value_stack.pop().unwrap();
                    self.insert_ast_node(node);
                } else {
                    self.annotation_key_stack.pop().unwrap();
                    let node = self.annotation_value_stack.pop().unwrap();
                    self.insert_annotation_node(node);
                }
            }
            Event::Null => {
                if self.annotation_name.is_none() {
                    let node = Jsona::Null(Null {
                        annotations: Vec::new(),
                        position,
                    });
                    self.insert_ast_node(node);
                } else {
                    self.insert_annotation_node(Value::Null);
                }
            }
            Event::Float(value) => {
                if self.annotation_name.is_none() {
                    let node = Jsona::Float(Float {
                        value,
                        annotations: Vec::new(),
                        position,
                    });
                    self.insert_ast_node(node);
                } else {
                    self.insert_annotation_node(value.into());
                }
            }
            Event::Integer(value) => {
                if self.annotation_name.is_none() {
                    let node = Jsona::Integer(Integer {
                        value,
                        annotations: Vec::new(),
                        position,
                    });
                    self.insert_ast_node(node);
                } else {
                    self.insert_annotation_node(value.into());
                }
            }
            Event::Boolean(value) => {
                if self.annotation_name.is_none() {
                    let node = Jsona::Boolean(Boolean {
                        value,
                        annotations: Vec::new(),
                        position,
                    });
                    self.insert_ast_node(node);
                } else {
                    self.insert_annotation_node(value.into());
                }
            }
            Event::String(value) => {
                if self.annotation_name.is_none() {
                    let node = Jsona::String(String {
                        value,
                        annotations: Vec::new(),
                        position,
                    });
                    self.insert_ast_node(node);
                } else {
                    self.insert_annotation_node(value.into());
                }
            }
        }
    }
}
