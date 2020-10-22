use std::error::Error;
use std::fmt::{self, Display};

use crate::value::{Amap, Object, Value};

#[derive(Copy, Clone, Debug)]
pub enum EmitError {
    FmtError(fmt::Error),
}

impl Error for EmitError {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

impl Display for EmitError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EmitError::FmtError(ref err) => Display::fmt(err, formatter),
        }
    }
}

impl From<fmt::Error> for EmitError {
    fn from(f: fmt::Error) -> Self {
        EmitError::FmtError(f)
    }
}

pub type EmitResult = Result<(), EmitError>;
pub struct Emitter<'a> {
    writer: &'a mut dyn fmt::Write,
    indent: usize,
    level: usize,
}

impl<'a> Emitter<'a> {
    pub fn new(writer: &'a mut dyn fmt::Write) -> Self {
        Self {
            writer,
            indent: 2,
            level: 0,
        }
    }
    pub fn set_indent(&mut self, indent: usize) {
        self.indent = indent;
    }
    pub fn emit(&mut self, data: &(Value, Option<Amap>)) -> EmitResult {
        self.emit_doc_annotations(&data.1)?;
        writeln!(self.writer)?;
        self.emit_value(&data.0)?;
        Ok(())
    }
    pub fn emit_doc_annotations(&mut self, annotations: &Option<Amap>) -> EmitResult {
        if let Some(a) = annotations {
            for (name, args) in a.iter() {
                write!(self.writer, "@{}", name)?;
                let len = args.len();
                if len > 0 {
                    write!(self.writer, "(")?;
                    for (i, (k, v)) in args.iter().enumerate() {
                        self.write_string(k.as_str(), false)?;
                        write!(self.writer, " = ")?;
                        self.write_string(v.as_str(), true)?;
                        if i < len - 1 {
                            write!(self.writer, ", ")?;
                        }
                    }
                    write!(self.writer, ")")?;
                }
                writeln!(self.writer)?;
            }
        }
        Ok(())
    }
    fn emit_annotations(&mut self, annotations: &Option<Amap>) -> EmitResult {
        if let Some(a) = annotations {
            for (name, args) in a.iter() {
                write!(self.writer, " @{}", name)?;
                let len = args.len();
                if len > 0 {
                    write!(self.writer, "(")?;
                    for (i, (k, v)) in args.iter().enumerate() {
                        self.write_string(k.as_str(), false)?;
                        write!(self.writer, " = ")?;
                        self.write_string(v.as_str(), true)?;
                        if i < len - 1 {
                            write!(self.writer, ", ")?;
                        }
                    }
                    write!(self.writer, ")")?;
                }
            }
        }
        Ok(())
    }
    pub fn emit_value(&mut self, node: &Value) -> EmitResult {
        self.emit_node(node, false)?;
        if node.is_scalar() {
            self.emit_annotations(node.get_annotations())?;
        }
        Ok(())
    }
    fn emit_node(&mut self, node: &Value, comma: bool) -> EmitResult {
        match *node {
            Value::Null{ .. } => {
                self.writer.write_str("null")?;
                if comma {
                    write!(self.writer, ",")?;
                }
                Ok(())
            }
            Value::Boolean{ value, .. } => {
                if value {
                    self.writer.write_str("true")?;
                } else {
                    self.writer.write_str("false")?;
                }
                if comma {
                    write!(self.writer, ",")?;
                }
                Ok(())
            }
            Value::Integer{ value, .. } => {
                write!(self.writer, "{}", value)?;
                if comma {
                    write!(self.writer, ",")?;
                }
                Ok(())
            }
            Value::Float{ value, .. } => {
                write!(self.writer, "{}", value)?;
                if comma {
                    write!(self.writer, ",")?;
                }
                Ok(())
            }
            Value::String{ref value, .. } => {
                self.write_string(value.as_str(), true)?;
                if comma {
                    write!(self.writer, ",")?;
                }
                Ok(())
            }
            Value::Array{ref value, ref annotations} => {
                self.emit_array(value, annotations, comma)?;
                Ok(())
            }
            Value::Object{ref value, ref annotations} => {
                self.emit_object(value, annotations, comma)?;
                Ok(())
            }
        }
    }
    fn write_indent(&mut self) -> EmitResult {
        for _ in 0..self.level {
            for _ in 0..self.indent {
                write!(self.writer, " ")?;
            }
        }
        Ok(())
    }
    fn emit_array(&mut self, v: &[Value], a: &Option<Amap>, comma: bool) -> EmitResult {
        if v.is_empty() {
            write!(self.writer, "[]")?;
            if comma {
                write!(self.writer, ",")?;
            }
            self.emit_annotations(a)?;
        } else {
            write!(self.writer, "[")?;
            self.emit_annotations(a)?;
            writeln!(self.writer)?;
            self.level += 1;
            for (i, x) in v.iter().enumerate() {
                self.write_indent()?;
                self.emit_node(x, i < v.len() - 1)?;
                if x.is_scalar() {
                    self.emit_annotations(x.get_annotations())?;
                }
                writeln!(self.writer)?;
            }
            self.level -= 1;
            self.write_indent()?;
            write!(self.writer, "]")?;
            if comma {
                write!(self.writer, ",")?;
            }
        }
        Ok(())
    }
    fn emit_object(&mut self, o: &Object, a: &Option<Amap>, comma: bool) -> EmitResult {
        if o.is_empty() {
            write!(self.writer, "{{}}")?;
            if comma {
                write!(self.writer, ",")?;
            }
            self.emit_annotations(a)?;
        } else {
            write!(self.writer, "{{")?;
            self.emit_annotations(a)?;
            writeln!(self.writer)?;
            self.level += 1;
            for (i, (k, v)) in o.iter().enumerate() {
                self.write_indent()?;
                self.write_string(k.as_str(), false)?;
                write!(self.writer, ": ")?;
                self.emit_node(v, i < o.len() - 1)?;
                if v.is_scalar() {
                    self.emit_annotations(v.get_annotations())?;
                }
                writeln!(self.writer)?;
            }
            self.level -= 1;
            self.write_indent()?;
            write!(self.writer, "}}")?;
            if comma {
                write!(self.writer, ",")?;
            }
        }
        Ok(())
    }
    fn write_string(&mut self, s: &str, quota: bool) -> EmitResult {
        if quota || need_quotes(s) {
            escape_str(self.writer, s)?;
        } else {
            write!(self.writer, "{}", s)?;
        }
        Ok(())
    }
}

/// Check if the string requires quoting.
fn need_quotes(string: &str) -> bool {
    fn need_quotes_spaces(string: &str) -> bool {
        string.starts_with(' ') || string.ends_with(' ')
    }

    string == ""
        || need_quotes_spaces(string)
        || string.starts_with(|character: char| match character {
            '&' | '*' | '?' | '|' | '-' | '<' | '>' | '=' | '!' | '%' | '@' => true,
            _ => false,
        })
        || string.contains(|character: char| match character {
            ':'
            | '{'
            | '}'
            | '['
            | ']'
            | ','
            | '#'
            | '`'
            | '\"'
            | '\''
            | '\\'
            | '\0'..='\x06'
            | '\t'
            | '\n'
            | '\r'
            | '\x0e'..='\x1a'
            | '\x1c'..='\x1f' => true,
            _ => false,
        })
        || string.starts_with('.')
        || string.starts_with("0x")
        || string.parse::<i64>().is_ok()
        || string.parse::<f64>().is_ok()
}

fn escape_str(wr: &mut dyn fmt::Write, v: &str) -> Result<(), fmt::Error> {
    wr.write_str("\"")?;

    let mut start = 0;

    for (i, byte) in v.bytes().enumerate() {
        let escaped = match byte {
            b'"' => "\\\"",
            b'\\' => "\\\\",
            b'\x00' => "\\u0000",
            b'\x01' => "\\u0001",
            b'\x02' => "\\u0002",
            b'\x03' => "\\u0003",
            b'\x04' => "\\u0004",
            b'\x05' => "\\u0005",
            b'\x06' => "\\u0006",
            b'\x07' => "\\u0007",
            b'\x08' => "\\b",
            b'\t' => "\\t",
            b'\n' => "\\n",
            b'\x0b' => "\\u000b",
            b'\x0c' => "\\f",
            b'\r' => "\\r",
            b'\x0e' => "\\u000e",
            b'\x0f' => "\\u000f",
            b'\x10' => "\\u0010",
            b'\x11' => "\\u0011",
            b'\x12' => "\\u0012",
            b'\x13' => "\\u0013",
            b'\x14' => "\\u0014",
            b'\x15' => "\\u0015",
            b'\x16' => "\\u0016",
            b'\x17' => "\\u0017",
            b'\x18' => "\\u0018",
            b'\x19' => "\\u0019",
            b'\x1a' => "\\u001a",
            b'\x1b' => "\\u001b",
            b'\x1c' => "\\u001c",
            b'\x1d' => "\\u001d",
            b'\x1e' => "\\u001e",
            b'\x1f' => "\\u001f",
            b'\x7f' => "\\u007f",
            _ => continue,
        };

        if start < i {
            wr.write_str(&v[start..i])?;
        }

        wr.write_str(escaped)?;

        start = i + 1;
    }

    if start != v.len() {
        wr.write_str(&v[start..])?;
    }

    wr.write_str("\"")?;
    Ok(())
}
