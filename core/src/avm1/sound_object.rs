//! AVM1 object type to represent Sound objects.

use crate::avm1::function::Executable;
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ObjectPtr, ScriptObject, TObject, Value};
use crate::backend::audio::SoundHandle;
use crate::context::UpdateContext;
use crate::display_object::DisplayObject;
use enumset::EnumSet;
use gc_arena::{Collect, GcCell, MutationContext};
use std::collections::HashSet;
use std::fmt;

/// A SounObject that is tied to a sound from the AudioBackend.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct SoundObject<'gc>(GcCell<'gc, SoundObjectData<'gc>>);

pub struct SoundObjectData<'gc> {
    /// The underlying script object.
    ///
    /// This is used to handle "expando properties" on AVM1 display nodes, as
    /// well as the underlying prototype chain.
    base: ScriptObject<'gc>,

    /// The sound that is attached to this object.
    sound: Option<SoundHandle>,

    /// Sounds in AVM1 are tied to a speicifc movie clip.
    owner: Option<DisplayObject<'gc>>,
}

unsafe impl<'gc> Collect for SoundObjectData<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.base.trace(cc);
        self.owner.trace(cc);
    }
}

impl fmt::Debug for SoundObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SoundObject")
            .field("sound", &self.0.read().sound)
            .field("owner", &self.0.read().owner)
            .finish()
    }
}

impl<'gc> SoundObject<'gc> {
    pub fn empty_sound(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
    ) -> SoundObject<'gc> {
        SoundObject(GcCell::allocate(
            gc_context,
            SoundObjectData {
                base: ScriptObject::object(gc_context, proto),
                sound: None,
                owner: None,
            },
        ))
    }

    pub fn sound(self) -> Option<SoundHandle> {
        self.0.read().sound
    }

    pub fn set_sound(self, gc_context: MutationContext<'gc, '_>, sound: Option<SoundHandle>) {
        self.0.write(gc_context).sound = sound;
    }

    pub fn owner(self) -> Option<DisplayObject<'gc>> {
        self.0.read().owner
    }

    pub fn set_owner(
        self,
        gc_context: MutationContext<'gc, '_>,
        owner: Option<DisplayObject<'gc>>,
    ) {
        self.0.write(gc_context).owner = owner;
    }

    fn base(self) -> ScriptObject<'gc> {
        self.0.read().base
    }
}

impl<'gc> TObject<'gc> for SoundObject<'gc> {
    fn get_local(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<ReturnValue<'gc>, Error> {
        self.base().get_local(name, avm, context, this)
    }

    fn set(
        &self,
        name: &str,
        value: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        self.base().set(name, value, avm, context)
    }

    fn call(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<ReturnValue<'gc>, Error> {
        self.base().call(avm, context, this, args)
    }

    #[allow(clippy::new_ret_no_self)]
    fn new(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        _this: Object<'gc>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        Ok(SoundObject::empty_sound(context.gc_context, Some(avm.prototypes.sound)).into())
    }

    fn delete(&self, gc_context: MutationContext<'gc, '_>, name: &str) -> bool {
        self.base().delete(gc_context, name)
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.base().proto()
    }

    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base()
            .define_value(gc_context, name, value, attributes)
    }

    fn set_attributes(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        name: Option<&str>,
        set_attributes: EnumSet<Attribute>,
        clear_attributes: EnumSet<Attribute>,
    ) {
        self.base()
            .set_attributes(gc_context, name, set_attributes, clear_attributes)
    }

    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Executable<'gc>,
        set: Option<Executable<'gc>>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base()
            .add_property(gc_context, name, get, set, attributes)
    }

    fn has_property(&self, name: &str) -> bool {
        self.base().has_property(name)
    }

    fn has_own_property(&self, name: &str) -> bool {
        self.base().has_own_property(name)
    }

    fn is_property_overwritable(&self, name: &str) -> bool {
        self.base().is_property_overwritable(name)
    }

    fn is_property_enumerable(&self, name: &str) -> bool {
        self.base().is_property_enumerable(name)
    }

    fn get_keys(&self) -> HashSet<String> {
        self.base().get_keys()
    }

    fn as_string(&self) -> String {
        self.base().as_string()
    }

    fn type_of(&self) -> &'static str {
        self.base().type_of()
    }

    fn interfaces(&self) -> Vec<Object<'gc>> {
        self.base().interfaces()
    }

    fn set_interfaces(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        iface_list: Vec<Object<'gc>>,
    ) {
        self.base().set_interfaces(gc_context, iface_list)
    }

    fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
        Some(self.base())
    }

    fn as_display_object(&self) -> Option<DisplayObject<'gc>> {
        None
    }

    fn as_executable(&self) -> Option<Executable<'gc>> {
        None
    }

    fn as_sound_object(&self) -> Option<SoundObject<'gc>> {
        Some(*self)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn length(&self) -> usize {
        self.base().length()
    }

    fn array(&self) -> Vec<Value<'gc>> {
        self.base().array()
    }

    fn set_length(&self, gc_context: MutationContext<'gc, '_>, length: usize) {
        self.base().set_length(gc_context, length)
    }

    fn array_element(&self, index: usize) -> Value<'gc> {
        self.base().array_element(index)
    }

    fn set_array_element(
        &self,
        index: usize,
        value: Value<'gc>,
        gc_context: MutationContext<'gc, '_>,
    ) -> usize {
        self.base().set_array_element(index, value, gc_context)
    }

    fn delete_array_element(&self, index: usize, gc_context: MutationContext<'gc, '_>) {
        self.base().delete_array_element(index, gc_context)
    }
}
