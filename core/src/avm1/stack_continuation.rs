//! GC-compatible scope continuations

use crate::avm1::{Avm1, Error, Object, Value};
use crate::context::UpdateContext;
use gc_arena::{Collect, GcCell};

/// Represents some piece of native code that needs to live on stack. It's
/// called when the stack function returns, and is required to be garbage
/// collectable.
pub trait StackContinuation<'gc>: 'gc + Collect {
    /// Called when the current activation returns.
    ///
    /// This function is handed the return value of that particular activation.
    /// You are free to use it as you please. In general, however, if you intend
    /// to return to the previous activation frame, then you should push this
    /// return value on the stack.
    fn returned(
        &mut self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: GcCell<'gc, Object<'gc>>,
        return_value: Value<'gc>,
    ) -> Result<(), Error>;
}

/// Generate a continuation from some set of garbage-collected values.
///
/// The values specified *must* implement `Collect`, and will be traced for as
/// long as the continuation remains on the stack. Non-`Collect` values will
/// fail to compile.
#[allow(unused_macros)]
macro_rules! stack_continuation {
    ($( $name:ident: $type:ty ),*, | $avmname:ident, $ctxtname:ident, $thisname:ident, $retvalname:ident | $code:block) => {
        {
            use gc_arena::GcCell;
            use crate::avm1::Object;
            use crate::avm1::stack_continuation::StackContinuation;

            struct MyCont<'gc> {
                $(
                    pub $name: $type,
                ),*
            };

            unsafe impl<'gc> gc_arena::Collect for MyCont<'gc> {
                #[inline]
                fn trace(&self, cc: gc_arena::CollectionContext) {
                    $(
                        self.$name.trace(cc);
                    ),*
                }
            }

            impl<'gc> StackContinuation<'gc> for MyCont<'gc> {
                fn returned(&mut self, avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>, this: GcCell<'gc, Object<'gc>>, return_value: Value<'gc>) -> Result<(), Error> {
                    let $($name),* = $(self.$name),*;
                    let $avmname = avm;
                    let $ctxtname = context;
                    let $thisname = this;
                    let $retvalname = return_value;

                    $code
                }
            }

            let cont = MyCont{$($name),*};

            Box::new(cont)
        }
    };
}
