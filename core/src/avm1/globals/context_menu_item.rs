use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::TObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::Object;
use crate::avm1::{ScriptObject, Value};
use crate::string::AvmString;
use gc_arena::MutationContext;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "copy" => method(copy; DONT_ENUM | DONT_DELETE);
};

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let caption = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .coerce_to_string(activation)?
        .to_string();
    let callback = args
        .get(1)
        .map(|v| v.to_owned().coerce_to_object(activation));
    let separator_before = args
        .get(2)
        .unwrap_or(&false.into())
        .to_owned()
        .as_bool(activation.swf_version());
    let enabled = args
        .get(3)
        .unwrap_or(&true.into())
        .to_owned()
        .as_bool(activation.swf_version());
    let visible = args
        .get(4)
        .unwrap_or(&true.into())
        .to_owned()
        .as_bool(activation.swf_version());

    this.set(
        "caption",
        AvmString::new(activation.context.gc_context, caption).into(),
        activation,
    )?;

    if let Some(callback) = callback {
        this.set("onSelect", callback.into(), activation)?;
    }

    this.set("separatorBefore", separator_before.into(), activation)?;
    this.set("enabled", enabled.into(), activation)?;
    this.set("visible", visible.into(), activation)?;

    Ok(this.into())
}

pub fn copy<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let caption = this
        .get("caption", activation)?
        .coerce_to_string(activation)?
        .to_string();
    let callback = this
        .get("onSelect", activation)?
        .coerce_to_object(activation);

    let enabled = this
        .get("enabled", activation)?
        .as_bool(activation.swf_version());
    let separator_before = this
        .get("separator_before", activation)?
        .as_bool(activation.swf_version());
    let visible = this
        .get("visible", activation)?
        .as_bool(activation.swf_version());

    let constructor = activation
        .context
        .avm1
        .prototypes
        .context_menu_item_constructor;
    let copy = constructor.construct(
        activation,
        &[
            AvmString::new(activation.context.gc_context, caption).into(),
            callback.into(),
            separator_before.into(),
            enabled.into(),
            visible.into(),
        ],
    )?;

    Ok(copy)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::object(gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    object.into()
}
