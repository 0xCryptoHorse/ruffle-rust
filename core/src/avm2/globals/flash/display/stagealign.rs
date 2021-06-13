//! `flash.display.StageAlign` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.StageAlign`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.StageAlign`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Construct `StageAlign`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "StageAlign"),
        Some(QName::new(Namespace::package(""), "Object").into()),
        Method::from_builtin_only(instance_init, "<StageAlign instance initializer>", mc),
        Method::from_builtin_only(class_init, "<StageAlign class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED | ClassAttributes::FINAL);

    const CONSTANTS: &[(&str, &str)] = &[
        ("BOTTOM", "B"),
        ("BOTTOM_LEFT", "BL"),
        ("BOTTOM_RIGHT", "BR"),
        ("LEFT", "L"),
        ("RIGHT", "R"),
        ("TOP", "T"),
        ("TOP_LEFT", "TL"),
        ("TOP_RIGHT", "TR"),
    ];
    write.define_public_constant_string_class_traits(CONSTANTS);

    class
}
