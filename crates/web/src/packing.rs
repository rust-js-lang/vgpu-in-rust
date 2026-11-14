//! WGSL-reflected buffer packing. Rust callers receive structured errors through
//! Result; shader layouts remain owned by the original WGSL asset compiler.
use crate::reflection::{HostShareableLayout, LayoutMember, WgslType};
use js::{ArrayBuffer, DataView, Kind, Unknown, array, array_buffer, classify, data_view, object};

#[cfg_attr(rust_js, rust_js::untagged)]
#[derive(Clone, Debug, PartialEq)]
pub enum DetailValue {
    Text(String),
    Number(f64),
}
impl From<&str> for DetailValue {
    fn from(value: &str) -> Self {
        Self::Text(value.into())
    }
}
impl From<String> for DetailValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}
impl From<f64> for DetailValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

#[derive(Debug)]
pub struct PackingDetail {
    pub reason: String,
    pub path: String,
    pub expected: DetailValue,
    pub actual: DetailValue,
    pub r#type: Option<String>,
}
#[derive(Debug)]
pub struct PackingError {
    pub code: &'static str,
    pub message: String,
    pub fix: Option<String>,
    pub r#where: &'static str,
    pub detail: Option<PackingDetail>,
}

pub fn try_pack(
    layout: &HostShareableLayout,
    value: Option<&Unknown>,
) -> Result<&'static ArrayBuffer, PackingError> {
    let size = layout.size().ok_or_else(|| {
        unsupported(format!(
            "No se puede inferir byteLength para layout runtime-sized '{}'.",
            layout.name()
        ))
    })?;
    validate_value(layout, value, "$", 0.0, size)?;
    let bytes = array_buffer::new(size);
    let view = data_view::new(bytes, None, None);
    write_value(view, layout, 0.0, value);
    Ok(bytes)
}

fn write_value(
    view: &DataView,
    layout: &HostShareableLayout,
    offset: f64,
    value: Option<&Unknown>,
) {
    if let Some(members) = layout.members() {
        for member in &members {
            write_value(
                view,
                member.layout(),
                offset + member.offset(),
                value.and_then(|value| js::get(value, &member.name())),
            );
        }
        return;
    }
    let kind = layout.wgsl_type();
    match kind.kind().as_str() {
        "scalar" => write_scalar(view, offset, &kind.name().expect("validated scalar"), value),
        "atomic" => write_scalar(
            view,
            offset,
            &scalar_name(kind.element().expect("validated element")).expect("validated scalar"),
            value,
        ),
        "vector" => {
            let element = kind.element().expect("validated element");
            let scalar = scalar_name(element).expect("validated scalar");
            let stride = scalar_size(element).expect("validated scalar");
            for index in 0..kind.width().expect("validated width") as usize {
                write_scalar(
                    view,
                    offset + index as f64 * stride,
                    &scalar,
                    component(value, index),
                );
            }
        }
        "matrix" => {
            let element = kind.element().expect("validated element");
            let scalar = scalar_name(element).expect("validated scalar");
            let scalar_size = scalar_size(element).expect("validated scalar");
            let rows = kind.rows().expect("validated rows") as usize;
            for column in 0..kind.columns().expect("validated columns") as usize {
                for row in 0..rows {
                    write_scalar(
                        view,
                        offset
                            + column as f64 * layout.stride().expect("validated stride")
                            + row as f64 * scalar_size,
                        &scalar,
                        component(value, column * rows + row),
                    );
                }
            }
        }
        "array" => {
            let count = length(value).expect("validated length") as usize;
            for index in 0..count {
                write_value(
                    view,
                    layout.element().expect("validated element"),
                    offset + index as f64 * layout.stride().expect("validated stride"),
                    js::get(value, &index),
                );
            }
        }
        _ => unreachable!("validated layout"),
    }
}

fn write_scalar(view: &DataView, offset: f64, kind: &str, value: Option<&Unknown>) {
    if kind == "bool" {
        view.set_uint32(offset, u32::from(js::truthy(value)), true);
        return;
    }
    // The host applies ToNumber at write time too. A getter may change its
    // value after validation; unary + preserves native coercion and throws.
    let value = js::to_number(value).unwrap();
    match kind {
        "f32" => view.set_float32(offset, value, true),
        "i32" => view.set_int32(offset, value, true),
        "u32" => view.set_uint32(offset, value, true),
        "f16" => view.set_uint16(
            offset,
            u32::from(vgpu_prism::half::float32_to_float16(value as f32)),
            true,
        ),
        _ => unreachable!("validated scalar"),
    }
}

fn component(value: Option<&Unknown>, index: usize) -> Option<&'static Unknown> {
    Some(
        value
            .and_then(|value| js::get(value, &index))
            .unwrap_or_else(|| js::unknown_of(0.0)),
    )
}

fn validate_value(
    layout: &HostShareableLayout,
    value: Option<&Unknown>,
    path: &str,
    offset: f64,
    bytes: f64,
) -> Result<(), PackingError> {
    if let Some(members) = layout.members() {
        return validate_struct(&members, value, path, offset, bytes);
    }
    let kind = layout.wgsl_type();
    match kind.kind().as_str() {
        "scalar" => {
            validate_scalar(&kind.name().expect("scalar name"), value, path)?;
            validate_range(path, offset, scalar_size(kind)?, bytes)
        }
        "atomic" => {
            validate_scalar(
                &scalar_name(kind.element().expect("atomic element"))?,
                value,
                path,
            )?;
            validate_range(path, offset, 4.0, bytes)
        }
        "vector" => {
            let count = kind.width().expect("vector width");
            exact_array_like(value, path, count)?;
            let element = kind.element().expect("vector element");
            let scalar = scalar_name(element)?;
            let size = scalar_size(element)?;
            for index in 0..count as usize {
                let path = format!("{path}[{index}]");
                validate_scalar(&scalar, js::get(value, &index), &path)?;
                validate_range(&path, offset + index as f64 * size, size, bytes)?;
            }
            Ok(())
        }
        "matrix" => {
            let columns = kind.columns().expect("matrix columns") as usize;
            let rows = kind.rows().expect("matrix rows") as usize;
            exact_array_like(value, path, (columns * rows) as f64)?;
            let element = kind.element().expect("matrix element");
            let scalar = scalar_name(element)?;
            let scalar_size = scalar_size(element)?;
            let stride = layout.stride().ok_or_else(|| {
                unsupported(format!("Matrix layout '{}' sin stride.", layout.name()))
            })?;
            for column in 0..columns {
                for row in 0..rows {
                    let path = format!("{path}[{column}][{row}]");
                    validate_scalar(&scalar, js::get(value, &(column * rows + row)), &path)?;
                    validate_range(
                        &path,
                        offset + column as f64 * stride + row as f64 * scalar_size,
                        scalar_size,
                        bytes,
                    )?;
                }
            }
            Ok(())
        }
        "array" => {
            let count = array_like(value, path)?;
            if let Some(expected) = kind.count()
                && count != expected
            {
                return Err(invalid("shape", path, expected.into(), count.into(), None));
            }
            let stride = layout.stride();
            let element = layout.element();
            let (Some(stride), Some(element)) = (stride, element) else {
                return Err(unsupported(format!(
                    "Array layout '{}' sin stride o element layout.",
                    layout.name()
                )));
            };
            if layout.runtime_sized().unwrap_or(false) {
                let expected = stride * count;
                if !safe_integer(stride) || stride < 0.0 || !safe_integer(expected) {
                    return Err(invalid(
                        "extent",
                        path,
                        "a safe byte extent".into(),
                        js::number::to_string(expected, None).into(),
                        None,
                    ));
                }
                let actual = layout.size().unwrap_or(bytes - offset);
                if actual != expected {
                    return Err(invalid(
                        "extent",
                        path,
                        expected.into(),
                        actual.into(),
                        None,
                    ));
                }
            }
            for index in 0..count as usize {
                validate_value(
                    element,
                    js::get(value, &index),
                    &format!("{path}[{index}]"),
                    offset + index as f64 * stride,
                    bytes,
                )?;
            }
            Ok(())
        }
        kind => Err(unsupported(format!("No hay validator para layout {kind}."))),
    }
}

fn validate_struct(
    members: &[&LayoutMember],
    value: Option<&Unknown>,
    path: &str,
    base: f64,
    bytes: f64,
) -> Result<(), PackingError> {
    let plain = matches!(classify(value), Kind::Object(_)) && !array_buffer::is_view(value) && {
        let prototype = object::get_prototype_of(value);
        prototype.is_none() || prototype == Some(object::prototype)
    };
    if !plain {
        return Err(invalid(
            "type",
            path,
            "an object".into(),
            value_kind(value).into(),
            None,
        ));
    }
    let value = value.expect("plain record");
    if let Some(unknown) = object::keys(value)
        .iter()
        .find(|name| !members.iter().any(|member| member.name() == **name))
    {
        return Err(invalid(
            "unknown-field",
            &format!("{path}.{unknown}"),
            "a declared field".into(),
            unknown.clone().into(),
            None,
        ));
    }
    for member in members {
        let path = format!("{path}.{}", member.name());
        if !object::has_own(value, &member.name()) {
            return Err(invalid(
                "missing-field",
                &path,
                "a required field".into(),
                "missing".into(),
                None,
            ));
        }
        validate_value(
            member.layout(),
            js::get(value, &member.name()),
            &path,
            base + member.offset(),
            bytes,
        )?;
    }
    Ok(())
}

fn validate_scalar(kind: &str, value: Option<&Unknown>, path: &str) -> Result<(), PackingError> {
    if kind == "bool" {
        return if matches!(classify(value), Kind::Bool(_)) {
            Ok(())
        } else {
            Err(invalid(
                "type",
                path,
                "bool".into(),
                value_kind(value).into(),
                Some(kind),
            ))
        };
    }
    let Kind::Number(number) = classify(value) else {
        return Err(invalid(
            "type",
            path,
            kind.into(),
            value_kind(value).into(),
            Some(kind),
        ));
    };
    let limits = match kind {
        "i32" => Some((
            -2147483648.0,
            2147483647.0,
            "an integer in [-2147483648, 2147483647]",
        )),
        "u32" => Some((0.0, 4294967295.0, "an integer in [0, 4294967295]")),
        _ => None,
    };
    if let Some((min, max, expected)) = limits
        && (!number.is_finite() || number.fract() != 0.0 || number < min || number > max)
    {
        return Err(invalid(
            "integer-range",
            path,
            expected.into(),
            js::number::to_string(number, None).into(),
            Some(kind),
        ));
    }
    Ok(())
}
fn array_like(value: Option<&Unknown>, path: &str) -> Result<f64, PackingError> {
    if array::is_array(value)
        || (array_buffer::is_view(value) && !data_view::is(value) && js::has(value, "length"))
    {
        return Ok(length(value).expect("array length"));
    }
    Err(invalid(
        "type",
        path,
        "an array or typed array".into(),
        value_kind(value).into(),
        None,
    ))
}
fn exact_array_like(
    value: Option<&Unknown>,
    path: &str,
    expected: f64,
) -> Result<(), PackingError> {
    let actual = array_like(value, path)?;
    if actual != expected {
        return Err(invalid("shape", path, expected.into(), actual.into(), None));
    }
    Ok(())
}
fn length(value: Option<&Unknown>) -> Option<f64> {
    match classify(js::get(value, "length")) {
        Kind::Number(value) => Some(value),
        _ => None,
    }
}
fn validate_range(path: &str, offset: f64, size: f64, bytes: f64) -> Result<(), PackingError> {
    if !safe_integer(offset)
        || !safe_integer(size)
        || offset < 0.0
        || size < 0.0
        || offset > bytes - size
    {
        return Err(invalid(
            "buffer-range",
            path,
            format!(
                "{} bytes inside {}",
                js::number::to_string(size, None),
                js::number::to_string(bytes, None)
            )
            .into(),
            offset.into(),
            None,
        ));
    }
    Ok(())
}
fn safe_integer(value: f64) -> bool {
    value.is_finite() && value.fract() == 0.0 && value.abs() <= 9007199254740991.0
}
fn scalar_name(kind: &WgslType) -> Result<String, PackingError> {
    if kind.kind() != "scalar" {
        return Err(unsupported(format!("Expected scalar, got {}", kind.kind())));
    }
    Ok(kind.name().expect("scalar name"))
}
fn scalar_size(kind: &WgslType) -> Result<f64, PackingError> {
    Ok(if scalar_name(kind)? == "f16" {
        2.0
    } else {
        4.0
    })
}
fn value_kind(value: Option<&Unknown>) -> String {
    if array::is_array(value) {
        return "array".into();
    }
    if array_buffer::is_view(value) {
        return js::string(js::get(js::get(value, "constructor"), "name"));
    }
    match classify(value) {
        Kind::Null => "null",
        Kind::Undefined => "undefined",
        Kind::String(_) => "string",
        Kind::Number(_) => "number",
        Kind::Bool(_) => "boolean",
        Kind::BigInt(_) => "bigint",
        Kind::Symbol(_) => "symbol",
        Kind::Function(_) => "function",
        _ => "object",
    }
    .into()
}
fn unsupported(message: String) -> PackingError {
    PackingError {
        code: "VGPU-RING1-UNSUPPORTED",
        message,
        fix: None,
        r#where: "set",
        detail: None,
    }
}
fn text(value: &DetailValue) -> String {
    match value {
        DetailValue::Text(value) => value.clone(),
        DetailValue::Number(value) => js::number::to_string(*value, None),
    }
}
fn invalid(
    reason: &str,
    path: &str,
    expected: DetailValue,
    actual: DetailValue,
    kind: Option<&str>,
) -> PackingError {
    let message = match reason {
        "shape" => format!(
            "expected exactly {} values, received {}",
            text(&expected),
            text(&actual)
        ),
        "missing-field" => "required field is missing".into(),
        "unknown-field" => format!("field '{}' is not declared by the layout", text(&actual)),
        "integer-range" => format!(
            "{} is outside the exact {} range",
            text(&actual),
            kind.unwrap_or("undefined")
        ),
        "extent" => format!(
            "runtime extent requires {} bytes, received {}",
            text(&expected),
            text(&actual)
        ),
        "buffer-range" => format!(
            "write at byte {} exceeds {}",
            text(&actual),
            text(&expected)
        ),
        _ => format!("expected {}, received {}", text(&expected), text(&actual)),
    };
    PackingError { code: "VGPU-SET-VALUE-INVALID", message: format!("Invalid WGSL value at '{path}': {message}."), fix: Some("Pass the exact reflected structure, vector, matrix, and array shapes; use integral in-range values for i32/u32.".into()), r#where: "set", detail: Some(PackingDetail { reason: reason.into(), path: path.into(), expected, actual, r#type: kind.map(String::from) }) }
}
