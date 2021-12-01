use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::events::{EventData, KeyModifiers};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.events.MouseEvent`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, args)?; // Event uses the first three parameters
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            let local_x = args
                .get(3)
                .cloned()
                .unwrap_or_else(|| f64::NAN.into())
                .coerce_to_number(activation)?;
            let local_y = args
                .get(4)
                .cloned()
                .unwrap_or_else(|| f64::NAN.into())
                .coerce_to_number(activation)?;
            let related_object = args
                .get(5)
                .cloned()
                .unwrap_or(Value::Null)
                .coerce_to_object(activation)
                .ok()
                .and_then(|o| o.as_display_object());
            let ctrl_key = args
                .get(6)
                .cloned()
                .unwrap_or_else(|| false.into())
                .coerce_to_boolean();
            let alt_key = args
                .get(7)
                .cloned()
                .unwrap_or_else(|| false.into())
                .coerce_to_boolean();
            let shift_key = args
                .get(8)
                .cloned()
                .unwrap_or_else(|| false.into())
                .coerce_to_boolean();
            let button_down = args
                .get(9)
                .cloned()
                .unwrap_or_else(|| false.into())
                .coerce_to_boolean();
            let delta = args
                .get(10)
                .cloned()
                .unwrap_or_else(|| 0.into())
                .coerce_to_i32(activation)?;

            let mut modifiers = KeyModifiers::default();
            if ctrl_key {
                modifiers.insert(KeyModifiers::CTRL);
            }

            if alt_key {
                modifiers.insert(KeyModifiers::ALT);
            }

            if shift_key {
                modifiers.insert(KeyModifiers::SHIFT);
            }

            evt.set_event_data(EventData::MouseEvent {
                local_x,
                local_y,
                related_object,
                modifiers,
                button_down,
                delta,
            });
        }
    }

    Ok(Value::Undefined)
}

/// Implements `flash.events.MouseEvent`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `altKey`'s getter.
pub fn alt_key<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::MouseEvent { modifiers, .. } = evt.event_data() {
                return Ok(modifiers.contains(KeyModifiers::ALT).into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `altKey`'s setter.
pub fn set_alt_key<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::MouseEvent { modifiers, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_boolean();

                if value {
                    modifiers.insert(KeyModifiers::ALT);
                } else {
                    modifiers.remove(KeyModifiers::ALT);
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `commandKey`'s getter.
pub fn command_key<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::MouseEvent { modifiers, .. } = evt.event_data() {
                return Ok(modifiers.contains(KeyModifiers::COMMAND).into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `commandKey`'s setter.
pub fn set_command_key<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::MouseEvent { modifiers, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_boolean();

                if value {
                    modifiers.insert(KeyModifiers::COMMAND);
                } else {
                    modifiers.remove(KeyModifiers::COMMAND);
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `ctrlKey`/`controlKey`'s getter.
pub fn control_key<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::MouseEvent { modifiers, .. } = evt.event_data() {
                return Ok(modifiers.contains(KeyModifiers::CTRL).into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `ctrlKey`/`controlKey`'s setter.
pub fn set_control_key<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::MouseEvent { modifiers, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_boolean();

                if value {
                    modifiers.insert(KeyModifiers::CTRL);
                } else {
                    modifiers.remove(KeyModifiers::CTRL);
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `shiftKey`'s getter.
pub fn shift_key<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::MouseEvent { modifiers, .. } = evt.event_data() {
                return Ok(modifiers.contains(KeyModifiers::SHIFT).into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `shiftKey`'s setter.
pub fn set_shift_key<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::MouseEvent { modifiers, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_boolean();

                if value {
                    modifiers.insert(KeyModifiers::SHIFT);
                } else {
                    modifiers.remove(KeyModifiers::SHIFT);
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `buttonDown`'s getter.
pub fn button_down<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::MouseEvent { button_down, .. } = evt.event_data() {
                return Ok(Value::Bool(*button_down));
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `buttonDown`'s setter.
pub fn set_button_down<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::MouseEvent { button_down, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_boolean();

                *button_down = value;
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `delta`'s getter.
pub fn delta<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::MouseEvent { delta, .. } = evt.event_data() {
                return Ok(Value::Integer(*delta));
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `delta`'s setter.
pub fn set_delta<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::MouseEvent { delta, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_i32(activation)?;

                *delta = value;
            }
        }
    }

    Ok(Value::Undefined)
}

/// Stubs `isRelatedObjectInaccessible`'s getter.
pub fn is_related_object_inaccessible<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(false.into())
}

/// Construct `MouseEvent`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.events"), "MouseEvent"),
        Some(QName::new(Namespace::package("flash.events"), "Event").into()),
        Method::from_builtin(instance_init, "<MouseEvent instance initializer>", mc),
        Method::from_builtin(class_init, "<MouseEvent class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    const CONSTANTS: &[(&str, &str)] = &[
        ("CLICK", "click"),
        ("CONTEXT_MENU", "contextMenu"),
        ("DOUBLE_CLICK", "doubleClick"),
        ("MIDDLE_CLICK", "middleClick"),
        ("MIDDLE_MOUSE_DOWN", "middleMouseDown"),
        ("MIDDLE_MOUSE_UP", "middleMouseUp"),
        ("MOUSE_DOWN", "mouseDown"),
        ("MOUSE_MOVE", "mouseMove"),
        ("MOUSE_OUT", "mouseOut"),
        ("MOUSE_OVER", "mouseOver"),
        ("MOUSE_UP", "mouseUp"),
        ("MOUSE_WHEEL", "mouseWheel"),
        ("RELEASE_OUTSIDE", "releaseOutside"),
        ("RIGHT_CLICK", "rightClick"),
        ("RIGHT_MOUSE_DOWN", "rightMouseDown"),
        ("RIGHT_MOUSE_UP", "rightMouseUp"),
        ("ROLL_OUT", "rollOut"),
        ("ROLL_OVER", "rollOver"),
    ];

    write.define_public_constant_string_class_traits(CONSTANTS);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("altKey", Some(alt_key), Some(set_alt_key)),
        ("commandKey", Some(command_key), Some(set_command_key)),
        ("controlKey", Some(control_key), Some(set_control_key)),
        ("ctrlKey", Some(control_key), Some(set_control_key)),
        ("shiftKey", Some(shift_key), Some(set_shift_key)),
        ("buttonDown", Some(button_down), Some(set_button_down)),
        ("delta", Some(delta), Some(set_delta)),
        (
            "isRelatedObjectInaccessible",
            Some(is_related_object_inaccessible),
            None,
        ),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    class
}
