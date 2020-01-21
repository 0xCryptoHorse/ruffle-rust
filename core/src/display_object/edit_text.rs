//! `EditText` display object and support code.
use crate::avm1::globals::text_field::attach_virtual_properties;
use crate::avm1::{Object, StageObject, Value};
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, TDisplayObject};
use crate::font::{Glyph, TextFormat};
use crate::library::Library;
use crate::prelude::*;
use crate::transform::Transform;
use gc_arena::{Collect, Gc, GcCell, MutationContext};

/// A dynamic text field.
/// The text in this text field can be changed dynamically.
/// It may be selectable or editable by the user, depending on the text field properties.
///
/// In the Flash IDE, this is created by changing the text field type to "Dynamic".
/// In AS2, this is created using `MovieClip.createTextField`.
/// In AS3, this is created with the `TextField` class. (https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/text/TextField.html)
///
/// (SWF19 DefineEditText pp. 171-174)
#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct EditText<'gc>(GcCell<'gc, EditTextData<'gc>>);

#[derive(Clone, Debug)]
pub struct EditTextData<'gc> {
    /// DisplayObject common properties.
    base: DisplayObjectBase<'gc>,

    /// Static data shared among all instances of this `EditText`.
    static_data: Gc<'gc, EditTextStatic>,

    /// The current text displayed by this text field.
    text: String,

    /// The text formatting for newly inserted text spans.
    new_format: TextFormat,

    /// If the text is in multi-line mode or single-line mode.
    is_multiline: bool,

    /// If the text is word-wrapped.
    is_word_wrap: bool,

    /// Cached breakpoints of where to make newlines.
    cached_break_points: Option<Vec<usize>>,

    // The AVM1 object handle
    object: Option<Object<'gc>>,
}

impl<'gc> EditText<'gc> {
    /// Creates a new `EditText` from an SWF `DefineEditText` tag.
    pub fn from_swf_tag(context: &mut UpdateContext<'_, 'gc, '_>, swf_tag: swf::EditText) -> Self {
        let is_multiline = swf_tag.is_multiline;
        let is_word_wrap = swf_tag.is_word_wrap;

        EditText(GcCell::allocate(
            context.gc_context,
            EditTextData {
                base: Default::default(),
                text: swf_tag.initial_text.clone().unwrap_or_default(),
                new_format: TextFormat::default(),
                static_data: gc_arena::Gc::allocate(context.gc_context, EditTextStatic(swf_tag)),
                is_multiline,
                is_word_wrap,
                object: None,
                cached_break_points: None,
            },
        ))
    }

    /// Create a new, dynamic `EditText`.
    pub fn new(
        context: &mut UpdateContext<'_, 'gc, '_>,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Self {
        let swf_tag = swf::EditText {
            id: 0, //TODO: Dynamic text fields don't have a character ID?
            bounds: swf::Rectangle {
                x_min: Twips::from_pixels(x),
                x_max: Twips::from_pixels(x + height),
                y_min: Twips::from_pixels(y),
                y_max: Twips::from_pixels(y + width),
            },
            font_id: None,
            font_class_name: None,
            height: Some(Twips::from_pixels(height)),
            color: Some(swf::Color {
                r: 0,
                g: 0,
                b: 0,
                a: 0xFF,
            }),
            max_length: Some(width as u16),
            layout: Some(swf::TextLayout {
                align: swf::TextAlign::Left,
                left_margin: Twips::from_pixels(0.0),
                right_margin: Twips::from_pixels(0.0),
                indent: Twips::from_pixels(0.0),
                leading: Twips::from_pixels(0.0),
            }),
            variable_name: "".to_string(), //TODO: should be null
            initial_text: None,
            is_word_wrap: false,
            is_multiline: false,
            is_password: false,
            is_read_only: true,
            is_auto_size: false,
            is_selectable: true,
            has_border: false,
            was_static: false,
            is_html: false,
            is_device_font: false,
        };

        Self::from_swf_tag(context, swf_tag)
    }

    // TODO: This needs to strip away HTML
    pub fn text(self) -> String {
        self.0.read().text.to_owned()
    }

    pub fn set_text(self, text: String, gc_context: MutationContext<'gc, '_>) {
        self.0.write(gc_context).cached_break_points = None;
        self.0.write(gc_context).text = text;
    }

    pub fn new_text_format(self) -> TextFormat {
        self.0.read().new_format.clone()
    }

    pub fn set_new_text_format(self, tf: TextFormat, gc_context: MutationContext<'gc, '_>) {
        self.0.write(gc_context).cached_break_points = None;
        self.0.write(gc_context).new_format = tf;
    }

    pub fn is_multiline(self) -> bool {
        self.0.read().is_multiline
    }

    pub fn set_multiline(self, is_multiline: bool, gc_context: MutationContext<'gc, '_>) {
        self.0.write(gc_context).cached_break_points = None;
        self.0.write(gc_context).is_multiline = is_multiline;
    }

    pub fn is_word_wrap(self) -> bool {
        self.0.read().is_word_wrap
    }

    pub fn set_word_wrap(self, is_word_wrap: bool, gc_context: MutationContext<'gc, '_>) {
        self.0.write(gc_context).cached_break_points = None;
        self.0.write(gc_context).is_word_wrap = is_word_wrap;
    }

    /// Construct a base text transform for this `EditText`, to be used for
    /// evaluating fonts.
    ///
    /// The `text_transform` constitutes the base transform that all text is
    /// written into.
    ///
    /// The `text_transform` is separate from and relative to the base
    /// transform that this `EditText` automatically gets by virtue of being a
    /// `DisplayObject`.
    pub fn text_transform(self) -> Transform {
        let edit_text = self.0.read();
        let static_data = &edit_text.static_data.0;

        // TODO: Many of these properties should change be instance members instead
        // of static data, because they can be altered via ActionScript.
        let color = static_data.color.as_ref().unwrap_or_else(|| &swf::Color {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        });

        let mut transform: Transform = Default::default();
        transform.color_transform.r_mult = f32::from(color.r) / 255.0;
        transform.color_transform.g_mult = f32::from(color.g) / 255.0;
        transform.color_transform.b_mult = f32::from(color.b) / 255.0;
        transform.color_transform.a_mult = f32::from(color.a) / 255.0;

        if let Some(layout) = &static_data.layout {
            transform.matrix.tx += layout.left_margin.get() as f32;
            transform.matrix.ty -= layout.leading.get() as f32;
        }

        transform
    }

    /// Given a text transform, produce a new transform at the start of the
    /// next line of text.
    ///
    /// This function takes the current font size and the transform to adjust,
    /// and returns the adjusted transform.
    pub fn newline(self, height: f32, mut transform: Transform) -> Transform {
        let edit_text = self.0.read();
        let static_data = &edit_text.static_data.0;

        transform.matrix.tx = 0.0;
        transform.matrix.ty += height * Twips::TWIPS_PER_PIXEL as f32;
        if let Some(layout) = &static_data.layout {
            transform.matrix.ty += layout.leading.get() as f32;
        }

        transform
    }

    pub fn line_width(self) -> f32 {
        let edit_text = self.0.read();

        self.width() as f32
            - edit_text
                .static_data
                .0
                .layout
                .as_ref()
                .map(|l| l.right_margin.to_pixels() as f32)
                .unwrap_or(0.0)
            - edit_text
                .static_data
                .0
                .layout
                .as_ref()
                .map(|l| l.left_margin.to_pixels() as f32)
                .unwrap_or(0.0)
    }

    /// Compute all "break points" between lines.
    ///
    /// The breakpoints are the character indicies of every point in the string
    /// which needs to have a newline in place, exclusive of the start and end
    /// of the string. Breakpoints are always placed after the whitespace
    /// character that caused a newline and represent the start of new lines.
    /// For example, if this function returns `vec![5, 15]`, then the following
    /// slicing operations would yield the intended line chunks:
    ///
    ///   `vec![&text[0..5], &text[5..15], &text[15..]]`
    ///
    /// Breakpoints are caused by either the index of a natural newline, or the
    /// index of a space that overflowed the current text width. Both of the
    /// above conditions are predicated on the equivalent flags being enabled
    /// in the underlying `EditText`. If no flags are enabled, or there are no
    /// break points for the string, then this returns an empty `Vec`.
    ///
    /// The given set of break points should be cached for later use as
    /// calculating them is a relatively expensive operation.
    fn line_breaks(self, library: &Library<'gc>) -> Vec<usize> {
        let edit_text = self.0.read();
        let static_data = &edit_text.static_data.0;
        let font_id = static_data.font_id.unwrap_or(0);

        if edit_text.is_multiline {
            if let Some(font) = library
                .get_font(font_id)
                .filter(|font| font.has_glyphs())
                .or_else(|| library.device_font())
            {
                let mut breakpoints = vec![];
                let mut break_base = 0;
                let height = static_data
                    .height
                    .map(|v| v.to_pixels() as f32)
                    .unwrap_or_else(|| font.scale());

                for natural_line in edit_text.text.split('\n') {
                    if break_base != 0 {
                        breakpoints.push(break_base);
                    }

                    for breakpoint in font.split_wrapped_lines(
                        natural_line,
                        height,
                        self.line_width(),
                        edit_text.static_data.0.is_html,
                    ) {
                        breakpoints.push(break_base + breakpoint);
                    }

                    break_base += natural_line.len() + 1;
                }

                breakpoints
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    /// Get the most recent line breaks, taking cache into account.
    ///
    /// This function is separate from `line_breaks` since there are some
    /// contexts where we don't have the ability to update a cache (such as
    /// rendering).
    fn line_breaks_cached(
        self,
        gc_context: MutationContext<'gc, '_>,
        library: &Library<'gc>,
    ) -> Vec<usize> {
        {
            let edit_text = self.0.read();
            if let Some(lbrk) = &edit_text.cached_break_points {
                return lbrk.clone();
            }
        }

        let lbrk = self.line_breaks(library);

        self.0.write(gc_context).cached_break_points = Some(lbrk.clone());

        lbrk
    }

    /// Measure the width and height of the `EditText`'s current text load.
    ///
    /// The returned tuple should be interpreted as width, then height.
    pub fn measure_text(self, context: &mut UpdateContext<'_, 'gc, '_>) -> (f32, f32) {
        let breakpoints = self.line_breaks_cached(context.gc_context, context.library);

        let edit_text = self.0.read();
        let static_data = &edit_text.static_data.0;
        let font_id = static_data.font_id.unwrap_or(0);

        let mut size: (f32, f32) = (0.0, 0.0);

        if let Some(font) = context
            .library
            .get_font(font_id)
            .filter(|font| font.has_glyphs())
            .or_else(|| context.library.device_font())
        {
            let mut start = 0;
            let mut chunks = vec![];
            for breakpoint in breakpoints {
                chunks.push(&edit_text.text[start..breakpoint]);
                start = breakpoint;
            }

            chunks.push(&edit_text.text[start..]);

            let height = static_data
                .height
                .map(|v| v.to_pixels() as f32)
                .unwrap_or_else(|| font.scale());

            for chunk in chunks {
                let chunk_size = font.measure(chunk, height, edit_text.static_data.0.is_html);

                size.0 = size.0.max(chunk_size.0);
                if let Some(layout) = &static_data.layout {
                    size.1 += layout.leading.to_pixels() as f32;
                }
                size.1 += chunk_size.1;
            }
        }

        size
    }
}

impl<'gc> TDisplayObject<'gc> for EditText<'gc> {
    impl_display_object!(base);

    fn id(&self) -> CharacterId {
        self.0.read().static_data.0.id
    }

    fn run_frame(&mut self, _context: &mut UpdateContext) {
        // Noop
    }

    fn as_edit_text(&self) -> Option<EditText<'gc>> {
        Some(*self)
    }

    fn post_instantiation(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        display_object: DisplayObject<'gc>,
        proto: Object<'gc>,
    ) {
        let mut text = self.0.write(gc_context);
        if text.object.is_none() {
            let object =
                StageObject::for_display_object(gc_context, display_object, Some(proto)).into();

            attach_virtual_properties(gc_context, object);

            text.object = Some(object);
        }
    }

    fn object(&self) -> Value<'gc> {
        self.0
            .read()
            .object
            .map(Value::from)
            .unwrap_or(Value::Undefined)
    }

    fn self_bounds(&self) -> BoundingBox {
        self.0.read().static_data.0.bounds.clone().into()
    }

    fn render(&self, context: &mut RenderContext<'_, 'gc>) {
        context.transform_stack.push(&*self.transform());

        let mut text_transform = self.text_transform();

        let edit_text = self.0.read();
        let static_data = &edit_text.static_data.0;
        let font_id = static_data.font_id.unwrap_or(0);

        // If the font can't be found or has no glyph information, use the "device font" instead.
        // We're cheating a bit and not actually rendering text using the OS/web.
        // Instead, we embed an SWF version of Noto Sans to use as the "device font", and render
        // it the same as any other SWF outline text.
        if let Some(font) = context
            .library
            .get_font(font_id)
            .filter(|font| font.has_glyphs())
            .or_else(|| context.library.device_font())
        {
            let height = static_data
                .height
                .map(|v| v.to_pixels() as f32)
                .unwrap_or_else(|| font.scale());

            let breakpoints = edit_text
                .cached_break_points
                .clone()
                .unwrap_or_else(|| self.line_breaks(context.library));
            let mut start = 0;
            let mut chunks = vec![];
            for breakpoint in breakpoints {
                chunks.push(&edit_text.text[start..breakpoint]);
                start = breakpoint;
            }

            chunks.push(&edit_text.text[start..]);

            for chunk in chunks {
                font.evaluate(
                    chunk,
                    text_transform.clone(),
                    height,
                    edit_text.static_data.0.is_html,
                    |transform, glyph: &Glyph| {
                        // Render glyph.
                        context.transform_stack.push(transform);
                        context
                            .renderer
                            .render_shape(glyph.shape, context.transform_stack.transform());
                        context.transform_stack.pop();
                    },
                );

                text_transform = self.newline(height, text_transform);
            }
        }

        context.transform_stack.pop();
    }

    fn allow_as_mask(&self) -> bool {
        false
    }
}

unsafe impl<'gc> gc_arena::Collect for EditTextData<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.base.trace(cc);
        self.static_data.trace(cc);
        self.object.trace(cc);
    }
}

/// Static data shared between all instances of a text object.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct EditTextStatic(swf::EditText);

unsafe impl<'gc> gc_arena::Collect for EditTextStatic {
    #[inline]
    fn needs_trace() -> bool {
        false
    }
}
