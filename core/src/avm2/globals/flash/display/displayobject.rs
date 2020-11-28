//! `flash.display.DisplayObject` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::TDisplayObject;
use crate::types::{Degrees, Percent};
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.DisplayObject`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `flash.display.DisplayObject`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `alpha`'s getter.
pub fn alpha<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj.alpha().into());
    }

    Ok(Value::Undefined)
}

/// Implements `alpha`'s setter.
pub fn set_alpha<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_alpha = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;
        dobj.set_alpha(activation.context.gc_context, new_alpha);
    }

    Ok(Value::Undefined)
}

/// Implements `height`'s getter.
pub fn height<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj.height().into());
    }

    Ok(Value::Undefined)
}

/// Implements `height`'s setter.
pub fn set_height<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_height = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;

        if new_height >= 0.0 {
            dobj.set_height(activation.context.gc_context, new_height);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `scaleY`'s getter.
pub fn scale_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(Value::Number(
            dobj.scale_y(activation.context.gc_context).into_unit(),
        ));
    }

    Ok(Value::Undefined)
}

/// Implements `scaleY`'s setter.
pub fn set_scale_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_scale = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;
        dobj.set_scale_y(activation.context.gc_context, Percent::from_unit(new_scale));
    }

    Ok(Value::Undefined)
}

/// Implements `width`'s getter.
pub fn width<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj.width().into());
    }

    Ok(Value::Undefined)
}

/// Implements `width`'s setter.
pub fn set_width<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_width = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;

        if new_width >= 0.0 {
            dobj.set_width(activation.context.gc_context, new_width);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `scaleX`'s getter.
pub fn scale_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(Value::Number(
            dobj.scale_x(activation.context.gc_context).into_unit(),
        ));
    }

    Ok(Value::Undefined)
}

/// Implements `scaleX`'s setter.
pub fn set_scale_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_scale = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;
        dobj.set_scale_x(activation.context.gc_context, Percent::from_unit(new_scale));
    }

    Ok(Value::Undefined)
}

/// Implements `x`'s getter.
pub fn x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj.x().into());
    }

    Ok(Value::Undefined)
}

/// Implements `x`'s setter.
pub fn set_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_x = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;

        dobj.set_x(activation.context.gc_context, new_x);
    }

    Ok(Value::Undefined)
}

/// Implements `y`'s getter.
pub fn y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj.y().into());
    }

    Ok(Value::Undefined)
}

/// Implements `y`'s setter.
pub fn set_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_y = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;

        dobj.set_y(activation.context.gc_context, new_y);
    }

    Ok(Value::Undefined)
}

/// Implements `rotation`'s getter.
pub fn rotation<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let rot: f64 = dobj.rotation(activation.context.gc_context).into();
        let rem = rot % 360.0;

        if rem <= 180.0 {
            return Ok(Value::Number(rem));
        } else {
            return Ok(Value::Number(rem - 360.0));
        }
    }

    Ok(Value::Undefined)
}

/// Implements `rotation`'s setter.
pub fn set_rotation<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_rotation = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;

        dobj.set_rotation(activation.context.gc_context, Degrees::from(new_rotation));
    }

    Ok(Value::Undefined)
}

/// Construct `DisplayObject`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "DisplayObject"),
        Some(QName::new(Namespace::package("flash.events"), "EventDispatcher").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::package(""), "alpha"),
        Method::from_builtin(alpha),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::package(""), "alpha"),
        Method::from_builtin(set_alpha),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::package(""), "height"),
        Method::from_builtin(height),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::package(""), "height"),
        Method::from_builtin(set_height),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::package(""), "scaleY"),
        Method::from_builtin(scale_y),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::package(""), "scaleY"),
        Method::from_builtin(set_scale_y),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::package(""), "width"),
        Method::from_builtin(width),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::package(""), "width"),
        Method::from_builtin(set_width),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::package(""), "scaleX"),
        Method::from_builtin(scale_x),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::package(""), "scaleX"),
        Method::from_builtin(set_scale_x),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::package(""), "x"),
        Method::from_builtin(x),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::package(""), "x"),
        Method::from_builtin(set_x),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::package(""), "y"),
        Method::from_builtin(y),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::package(""), "y"),
        Method::from_builtin(set_y),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::package(""), "rotation"),
        Method::from_builtin(rotation),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::package(""), "rotation"),
        Method::from_builtin(set_rotation),
    ));

    class
}
