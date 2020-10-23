//! MovieClip prototype

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::globals::display_object::{self, AVM_DEPTH_BIAS, AVM_MAX_DEPTH};
use crate::avm1::globals::matrix::gradient_object_to_matrix;
use crate::avm1::property::Attribute::*;
use crate::avm1::{AvmString, Object, ScriptObject, TObject, Value};
use crate::avm_error;
use crate::avm_warn;
use crate::backend::navigator::NavigationMethod;
use crate::display_object::{DisplayObject, EditText, MovieClip, TDisplayObject};
use crate::ecma_conversions::f64_to_wrapping_i32;
use crate::prelude::*;
use crate::shape_utils::DrawCommand;
use crate::tag_utils::SwfSlice;
use crate::vminterface::Instantiator;
use gc_arena::MutationContext;
use std::borrow::Cow;
use swf::{
    FillStyle, Gradient, GradientInterpolation, GradientRecord, GradientSpread, LineCapStyle,
    LineJoinStyle, LineStyle, Twips,
};

/// Implements `MovieClip`
pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

macro_rules! with_movie_clip {
    ( $gc_context: ident, $object:ident, $fn_proto: expr, $($name:expr => $fn:expr),* ) => {{
        $(
            $object.force_set_function(
                $name,
                |activation: &mut Activation<'_, 'gc, '_>, this, args| -> Result<Value<'gc>, Error<'gc>> {
                    if let Some(display_object) = this.as_display_object() {
                        if let Some(movie_clip) = display_object.as_movie_clip() {
                            return $fn(movie_clip, activation, args);
                        }
                    }
                    Ok(Value::Undefined)
                } as crate::avm1::function::NativeFunction<'gc>,
                $gc_context,
                DontDelete | ReadOnly | DontEnum,
                $fn_proto
            );
        )*
    }};
}

macro_rules! with_movie_clip_props {
    ($obj:ident, $gc:ident, $fn_proto:ident, $($name:literal => [$get:ident $(, $set:ident)*],)*) => {
        $(
            $obj.add_property(
                $gc,
                $name,
                with_movie_clip_props!(getter $gc, $fn_proto, $get),
                with_movie_clip_props!(setter $gc, $fn_proto, $($set),*),
                DontDelete | DontEnum,
            );
        )*
    };

    (getter $gc:ident, $fn_proto:ident, $get:ident) => {
        FunctionObject::function(
            $gc,
            Executable::Native(
                |activation: &mut Activation<'_, 'gc, '_>, this, _args| -> Result<Value<'gc>, Error<'gc>> {
                    if let Some(display_object) = this.as_display_object() {
                        if let Some(movie_clip) = display_object.as_movie_clip() {
                            return $get(movie_clip, activation);
                        }
                    }
                    Ok(Value::Undefined)
                } as crate::avm1::function::NativeFunction<'gc>
            ),
            Some($fn_proto),
            $fn_proto
        )
    };

    (setter $gc:ident, $fn_proto:ident, $set:ident) => {
        Some(FunctionObject::function(
            $gc,
            Executable::Native(
                |activation: &mut Activation<'_, 'gc, '_>, this, args| -> Result<Value<'gc>, Error<'gc>> {
                    if let Some(display_object) = this.as_display_object() {
                        if let Some(movie_clip) = display_object.as_movie_clip() {
                            let value = args
                                .get(0)
                                .unwrap_or(&Value::Undefined)
                                .clone();
                            $set(movie_clip, activation, value)?;
                        }
                    }
                    Ok(Value::Undefined)
                } as crate::avm1::function::NativeFunction<'gc>
            ),
            Some($fn_proto),
            $fn_proto)
        )
    };

    (setter $gc:ident, $fn_proto:ident,) => {
        None
    };
}

#[allow(clippy::comparison_chain)]
pub fn hit_test<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() > 1 {
        let x = args.get(0).unwrap().coerce_to_f64(activation)?;
        let y = args.get(1).unwrap().coerce_to_f64(activation)?;
        let shape = args
            .get(2)
            .map(|v| v.as_bool(activation.current_swf_version()))
            .unwrap_or(false);
        if x.is_finite() && y.is_finite() {
            // The docs say the point is in "Stage coordinates", but actually they are in root coordinates.
            // root can be moved via _root._x etc., so we actually have to transform from root to world space.
            let point = movie_clip
                .root()
                .local_to_global((Twips::from_pixels(x), Twips::from_pixels(y)));
            let ret = if shape {
                movie_clip.hit_test_shape(&mut activation.context, point)
            } else {
                movie_clip.hit_test_bounds(point)
            };
            return Ok(ret.into());
        }
    } else if args.len() == 1 {
        let other = args
            .get(0)
            .unwrap()
            .coerce_to_object(activation)
            .as_display_object();
        if let Some(other) = other {
            return Ok(other
                .world_bounds()
                .intersects(&movie_clip.world_bounds())
                .into());
        }
    }

    Ok(false.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    display_object::define_display_object_proto(gc_context, object, fn_proto);

    with_movie_clip!(
        gc_context,
        object,
        Some(fn_proto),
        "attachMovie" => attach_movie,
        "createEmptyMovieClip" => create_empty_movie_clip,
        "createTextField" => create_text_field,
        "duplicateMovieClip" => duplicate_movie_clip,
        "getBounds" => get_bounds,
        "getBytesLoaded" => get_bytes_loaded,
        "getBytesTotal" => get_bytes_total,
        "getNextHighestDepth" => get_next_highest_depth,
        "getRect" => get_rect,
        "getURL" => get_url,
        "globalToLocal" => global_to_local,
        "gotoAndPlay" => goto_and_play,
        "gotoAndStop" => goto_and_stop,
        "hitTest" => hit_test,
        "loadMovie" => load_movie,
        "loadVariables" => load_variables,
        "localToGlobal" => local_to_global,
        "nextFrame" => next_frame,
        "play" => play,
        "prevFrame" => prev_frame,
        "removeMovieClip" => remove_movie_clip,
        "startDrag" => start_drag,
        "stop" => stop,
        "stopDrag" => stop_drag,
        "swapDepths" => swap_depths,
        "unloadMovie" => unload_movie,
        "beginFill" => begin_fill,
        "beginGradientFill" => begin_gradient_fill,
        "moveTo" => move_to,
        "lineTo" => line_to,
        "curveTo" => curve_to,
        "endFill" => end_fill,
        "lineStyle" => line_style,
        "clear" => clear
    );

    with_movie_clip_props!(
        proto, gc_context, fn_proto,
        "transform" => [transform, set_transform],
    );

    object.into()
}

fn line_style<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(width) = args.get(0) {
        let width = Twips::from_pixels(width.coerce_to_f64(activation)?.min(255.0).max(0.0));
        let color = if let Some(rgb) = args.get(1) {
            let rgb = rgb.coerce_to_u32(activation)?;
            let alpha = if let Some(alpha) = args.get(2) {
                alpha.coerce_to_f64(activation)?.min(100.0).max(0.0)
            } else {
                100.0
            } as f32
                / 100.0
                * 255.0;
            Color::from_rgb(rgb, alpha as u8)
        } else {
            Color::from_rgb(0, 255)
        };
        let is_pixel_hinted = args
            .get(3)
            .map_or(false, |v| v.as_bool(activation.current_swf_version()));
        let (allow_scale_x, allow_scale_y) = match args
            .get(4)
            .and_then(|v| v.coerce_to_string(activation).ok())
            .as_deref()
        {
            Some("normal") => (true, true),
            Some("vertical") => (true, false),
            Some("horizontal") => (false, true),
            _ => (false, false),
        };
        let cap_style = match args
            .get(5)
            .and_then(|v| v.coerce_to_string(activation).ok())
            .as_deref()
        {
            Some("square") => LineCapStyle::Square,
            Some("none") => LineCapStyle::None,
            _ => LineCapStyle::Round,
        };
        let join_style = match args
            .get(6)
            .and_then(|v| v.coerce_to_string(activation).ok())
            .as_deref()
        {
            Some("miter") => {
                if let Some(limit) = args.get(7) {
                    LineJoinStyle::Miter(limit.coerce_to_f64(activation)?.max(0.0).min(255.0) as f32)
                } else {
                    LineJoinStyle::Miter(3.0)
                }
            }
            Some("bevel") => LineJoinStyle::Bevel,
            _ => LineJoinStyle::Round,
        };
        movie_clip.set_line_style(
            &mut activation.context,
            Some(LineStyle {
                width,
                color,
                start_cap: cap_style,
                end_cap: cap_style,
                join_style,
                fill_style: None,
                allow_scale_x,
                allow_scale_y,
                is_pixel_hinted,
                allow_close: false,
            }),
        );
    } else {
        movie_clip.set_line_style(&mut activation.context, None);
    }
    Ok(Value::Undefined)
}

fn begin_fill<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(rgb) = args.get(0) {
        let rgb = rgb.coerce_to_u32(activation)?;
        let alpha = if let Some(alpha) = args.get(1) {
            alpha.coerce_to_f64(activation)?.min(100.0).max(0.0)
        } else {
            100.0
        } as f32
            / 100.0
            * 255.0;
        movie_clip.set_fill_style(
            &mut activation.context,
            Some(FillStyle::Color(Color::from_rgb(rgb, alpha as u8))),
        );
    } else {
        movie_clip.set_fill_style(&mut activation.context, None);
    }
    Ok(Value::Undefined)
}

fn begin_gradient_fill<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(method), Some(colors), Some(alphas), Some(ratios), Some(matrix)) = (
        args.get(0),
        args.get(1),
        args.get(2),
        args.get(3),
        args.get(4),
    ) {
        let method = method.coerce_to_string(activation)?;
        let colors = colors.coerce_to_object(activation).array();
        let alphas = alphas.coerce_to_object(activation).array();
        let ratios = ratios.coerce_to_object(activation).array();
        let matrix_object = matrix.coerce_to_object(activation);
        if colors.len() != alphas.len() || colors.len() != ratios.len() {
            avm_warn!(
                activation,
                "beginGradientFill() received different sized arrays for colors, alphas and ratios"
            );
            return Ok(Value::Undefined);
        }
        let mut records = Vec::with_capacity(colors.len());
        for i in 0..colors.len() {
            let ratio = ratios[i].coerce_to_f64(activation)?.min(255.0).max(0.0);
            let rgb = colors[i].coerce_to_u32(activation)?;
            let alpha = alphas[i].coerce_to_f64(activation)?.min(100.0).max(0.0);
            records.push(GradientRecord {
                ratio: ratio as u8,
                color: Color::from_rgb(rgb, (alpha / 100.0 * 255.0) as u8),
            });
        }
        let matrix = gradient_object_to_matrix(matrix_object, activation)?;
        let spread = match args
            .get(5)
            .and_then(|v| v.coerce_to_string(activation).ok())
            .as_deref()
        {
            Some("reflect") => GradientSpread::Reflect,
            Some("repeat") => GradientSpread::Repeat,
            _ => GradientSpread::Pad,
        };
        let interpolation = match args
            .get(6)
            .and_then(|v| v.coerce_to_string(activation).ok())
            .as_deref()
        {
            Some("linearRGB") => GradientInterpolation::LinearRGB,
            _ => GradientInterpolation::RGB,
        };

        let gradient = Gradient {
            matrix,
            spread,
            interpolation,
            records,
        };
        let style = match method.as_ref() {
            "linear" => FillStyle::LinearGradient(gradient),
            "radial" => {
                if let Some(focal_point) = args.get(7) {
                    FillStyle::FocalGradient {
                        gradient,
                        focal_point: focal_point.coerce_to_f64(activation)? as f32,
                    }
                } else {
                    FillStyle::RadialGradient(gradient)
                }
            }
            other => {
                avm_warn!(
                    activation,
                    "beginGradientFill() received invalid fill type {:?}",
                    other
                );
                return Ok(Value::Undefined);
            }
        };
        movie_clip.set_fill_style(&mut activation.context, Some(style));
    } else {
        movie_clip.set_fill_style(&mut activation.context, None);
    }
    Ok(Value::Undefined)
}

fn move_to<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(x), Some(y)) = (args.get(0), args.get(1)) {
        let x = x.coerce_to_f64(activation)?;
        let y = y.coerce_to_f64(activation)?;
        movie_clip.draw_command(
            &mut activation.context,
            DrawCommand::MoveTo {
                x: Twips::from_pixels(x),
                y: Twips::from_pixels(y),
            },
        );
    }
    Ok(Value::Undefined)
}

fn line_to<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(x), Some(y)) = (args.get(0), args.get(1)) {
        let x = x.coerce_to_f64(activation)?;
        let y = y.coerce_to_f64(activation)?;
        movie_clip.draw_command(
            &mut activation.context,
            DrawCommand::LineTo {
                x: Twips::from_pixels(x),
                y: Twips::from_pixels(y),
            },
        );
    }
    Ok(Value::Undefined)
}

fn curve_to<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(x1), Some(y1), Some(x2), Some(y2)) =
        (args.get(0), args.get(1), args.get(2), args.get(3))
    {
        let x1 = x1.coerce_to_f64(activation)?;
        let y1 = y1.coerce_to_f64(activation)?;
        let x2 = x2.coerce_to_f64(activation)?;
        let y2 = y2.coerce_to_f64(activation)?;
        movie_clip.draw_command(
            &mut activation.context,
            DrawCommand::CurveTo {
                x1: Twips::from_pixels(x1),
                y1: Twips::from_pixels(y1),
                x2: Twips::from_pixels(x2),
                y2: Twips::from_pixels(y2),
            },
        );
    }
    Ok(Value::Undefined)
}

fn end_fill<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    movie_clip.set_fill_style(&mut activation.context, None);
    Ok(Value::Undefined)
}

fn clear<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    movie_clip.clear(&mut activation.context);
    Ok(Value::Undefined)
}

fn attach_movie<'gc>(
    mut movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let (export_name, new_instance_name, depth) = match &args[0..3] {
        [export_name, new_instance_name, depth] => (
            export_name.coerce_to_string(activation)?,
            new_instance_name.coerce_to_string(activation)?,
            depth
                .coerce_to_i32(activation)?
                .wrapping_add(AVM_DEPTH_BIAS),
        ),
        _ => {
            avm_error!(activation, "MovieClip.attachMovie: Too few parameters");
            return Ok(Value::Undefined);
        }
    };
    let init_object = args.get(3);

    // TODO: What is the derivation of this max value? It shows up a few times in the AVM...
    // 2^31 - 16777220
    if depth < 0 || depth > AVM_MAX_DEPTH {
        return Ok(Value::Undefined);
    }

    if let Ok(new_clip) = activation
        .context
        .library
        .library_for_movie(movie_clip.movie().unwrap())
        .ok_or_else(|| "Movie is missing!".into())
        .and_then(|l| l.instantiate_by_export_name(&export_name, activation.context.gc_context))
    {
        // Set name and attach to parent.
        new_clip.set_name(activation.context.gc_context, &new_instance_name);
        movie_clip.add_child_from_avm(&mut activation.context, new_clip, depth);
        let init_object = if let Some(Value::Object(init_object)) = init_object {
            Some(init_object.to_owned())
        } else {
            None
        };
        new_clip.post_instantiation(
            &mut activation.context,
            new_clip,
            init_object,
            Instantiator::Avm1,
            true,
        );

        Ok(new_clip.object().coerce_to_object(activation).into())
    } else {
        avm_warn!(activation, "Unable to attach '{}'", export_name);
        Ok(Value::Undefined)
    }
}

fn create_empty_movie_clip<'gc>(
    mut movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let (new_instance_name, depth) = match &args[0..2] {
        [new_instance_name, depth] => (
            new_instance_name.coerce_to_string(activation)?,
            depth
                .coerce_to_i32(activation)?
                .wrapping_add(AVM_DEPTH_BIAS),
        ),
        _ => {
            avm_error!(activation, "MovieClip.attachMovie: Too few parameters");
            return Ok(Value::Undefined);
        }
    };

    // Create empty movie clip.
    let swf_movie = movie_clip
        .movie()
        .or_else(|| activation.base_clip().movie())
        .unwrap();
    let new_clip = MovieClip::new(SwfSlice::empty(swf_movie), activation.context.gc_context);

    // Set name and attach to parent.
    new_clip.set_name(activation.context.gc_context, &new_instance_name);
    movie_clip.add_child_from_avm(&mut activation.context, new_clip.into(), depth);
    new_clip.post_instantiation(
        &mut activation.context,
        new_clip.into(),
        None,
        Instantiator::Avm1,
        true,
    );

    Ok(new_clip.object())
}

fn create_text_field<'gc>(
    mut movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let movie = activation.base_clip().movie().unwrap();
    let instance_name = args.get(0).cloned().unwrap_or(Value::Undefined);
    let depth = args
        .get(1)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_f64(activation)?;
    let x = args
        .get(2)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_f64(activation)?;
    let y = args
        .get(3)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_f64(activation)?;
    let width = args
        .get(4)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_f64(activation)?;
    let height = args
        .get(5)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_f64(activation)?;

    let text_field: DisplayObject<'gc> =
        EditText::new(&mut activation.context, movie, x, y, width, height).into();
    text_field.set_name(
        activation.context.gc_context,
        &instance_name.coerce_to_string(activation)?,
    );
    movie_clip.add_child_from_avm(
        &mut activation.context,
        text_field,
        (depth as Depth).wrapping_add(AVM_DEPTH_BIAS),
    );
    text_field.post_instantiation(
        &mut activation.context,
        text_field,
        None,
        Instantiator::Avm1,
        false,
    );

    if activation.current_swf_version() >= 8 {
        //SWF8+ returns the `TextField` instance here
        Ok(text_field.object())
    } else {
        Ok(Value::Undefined)
    }
}

fn duplicate_movie_clip<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // duplicateMovieClip method uses biased depth compared to CloneSprite
    duplicate_movie_clip_with_bias(movie_clip, activation, args, AVM_DEPTH_BIAS)
}

pub fn duplicate_movie_clip_with_bias<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
    depth_bias: i32,
) -> Result<Value<'gc>, Error<'gc>> {
    let (new_instance_name, depth) = match &args[0..2] {
        [new_instance_name, depth] => (
            new_instance_name.coerce_to_string(activation)?,
            depth.coerce_to_i32(activation)?.wrapping_add(depth_bias),
        ),
        _ => {
            avm_error!(activation, "MovieClip.attachMovie: Too few parameters");
            return Ok(Value::Undefined);
        }
    };
    let init_object = args.get(2);

    // Can't duplicate the root!
    let mut parent = if let Some(parent) = movie_clip.parent().and_then(|o| o.as_movie_clip()) {
        parent
    } else {
        return Ok(Value::Undefined);
    };

    // TODO: What is the derivation of this max value? It shows up a few times in the AVM...
    // 2^31 - 16777220
    if depth < 0 || depth > AVM_MAX_DEPTH {
        return Ok(Value::Undefined);
    }

    if let Ok(new_clip) = activation
        .context
        .library
        .library_for_movie(movie_clip.movie().unwrap())
        .ok_or_else(|| "Movie is missing!".into())
        .and_then(|l| l.instantiate_by_id(movie_clip.id(), activation.context.gc_context))
    {
        // Set name and attach to parent.
        new_clip.set_name(activation.context.gc_context, &new_instance_name);
        parent.add_child_from_avm(&mut activation.context, new_clip, depth);

        // Copy display properties from previous clip to new clip.
        new_clip.set_matrix(activation.context.gc_context, &*movie_clip.matrix());
        new_clip.set_color_transform(
            activation.context.gc_context,
            &*movie_clip.color_transform(),
        );
        new_clip.as_movie_clip().unwrap().set_clip_actions(
            activation.context.gc_context,
            movie_clip.clip_actions().to_vec(),
        );
        // TODO: Any other properties we should copy...?
        // Definitely not ScriptObject properties.

        let init_object = init_object.map(|v| v.coerce_to_object(activation));
        new_clip.post_instantiation(
            &mut activation.context,
            new_clip,
            init_object,
            Instantiator::Avm1,
            true,
        );

        Ok(new_clip.object().coerce_to_object(activation).into())
    } else {
        avm_warn!(
            activation,
            "Unable to duplicate clip '{}'",
            movie_clip.name()
        );
        Ok(Value::Undefined)
    }
}

fn get_bytes_loaded<'gc>(
    _movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.swf.header().uncompressed_length.into())
}

fn get_bytes_total<'gc>(
    _movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.swf.header().uncompressed_length.into())
}

fn get_next_highest_depth<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.current_swf_version() >= 7 {
        let depth = std::cmp::max(
            movie_clip
                .highest_depth()
                .unwrap_or(0)
                .wrapping_sub(AVM_DEPTH_BIAS - 1),
            0,
        );
        Ok(depth.into())
    } else {
        Ok(Value::Undefined)
    }
}

fn goto_and_play<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    goto_frame(movie_clip, activation, args, false, 0)
}

fn goto_and_stop<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    goto_frame(movie_clip, activation, args, true, 0)
}

pub fn goto_frame<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
    stop: bool,
    scene_offset: u16,
) -> Result<Value<'gc>, Error<'gc>> {
    let mut call_frame = None;

    match args.get(0).cloned().unwrap_or(Value::Undefined) {
        // A direct goto only runs if n is an integer
        Value::Number(n) if n.fract() == 0.0 => {
            // Frame #
            // Gotoing <= 0 has no effect.
            // Gotoing greater than _totalframes jumps to the last frame.
            // Wraps around as an i32.
            // TODO: -1 +1 here to match Flash's behavior.
            // We probably want to change our frame representation to 0-based.
            // Scene offset is only used by GotoFrame2 global opcode.
            call_frame = Some((movie_clip, f64_to_wrapping_i32(n)));
        }
        val => {
            // Coerce to string and search for a frame label.
            // This can direct other clips than the one this method was called on!
            let frame_path = val.coerce_to_string(activation)?;
            if let Some((clip, frame)) =
                activation.resolve_variable_path(movie_clip.into(), &frame_path)?
            {
                if let Some(clip) = clip.as_display_object().and_then(|o| o.as_movie_clip()) {
                    if let Ok(frame) = frame.parse().map(f64_to_wrapping_i32) {
                        // First try to parse as a frame number.
                        call_frame = Some((clip, frame));
                    } else if let Some(frame) = clip.frame_label_to_number(&frame) {
                        // Otherwise, it's a frame label.
                        call_frame = Some((clip, frame as i32));
                    }
                }
            }
        }
    }

    if let Some((clip, mut frame)) = call_frame {
        frame = frame.wrapping_sub(1);
        frame = frame.wrapping_add(i32::from(scene_offset));
        frame = frame.saturating_add(1);
        if frame > 0 {
            clip.goto_frame(&mut activation.context, frame as u16, stop);
        }
    }
    Ok(Value::Undefined)
}

fn next_frame<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    movie_clip.next_frame(&mut activation.context);
    Ok(Value::Undefined)
}

fn play<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    movie_clip.play(&mut activation.context);
    Ok(Value::Undefined)
}

fn prev_frame<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    movie_clip.prev_frame(&mut activation.context);
    Ok(Value::Undefined)
}

pub fn remove_movie_clip<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let depth = movie_clip.depth().wrapping_sub(0);
    // Can only remove positive depths (when offset by the AVM depth bias).
    // Generally this prevents you from removing non-dynamically created clips,
    // although you can get around it with swapDepths.
    // TODO: Figure out the derivation of this range.
    if depth >= AVM_DEPTH_BIAS && depth < 2_130_706_416 {
        // Need a parent to remove from.
        let mut parent = if let Some(parent) = movie_clip.parent().and_then(|o| o.as_movie_clip()) {
            parent
        } else {
            return Ok(Value::Undefined);
        };

        parent.remove_child_from_avm(&mut activation.context, movie_clip.into());
    }
    Ok(Value::Undefined)
}

fn start_drag<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    crate::avm1::start_drag(movie_clip.into(), activation, args);
    Ok(Value::Undefined)
}

fn stop<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    movie_clip.stop(&mut activation.context);
    Ok(Value::Undefined)
}

fn stop_drag<'gc>(
    _movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // It doesn't matter which clip we call this on; it simply stops any active drag.
    *activation.context.drag_object = None;
    Ok(Value::Undefined)
}

fn swap_depths<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let arg = args.get(0).cloned().unwrap_or(Value::Undefined);

    let parent = if let Some(parent) = movie_clip.parent().and_then(|o| o.as_movie_clip()) {
        parent
    } else {
        return Ok(Value::Undefined);
    };

    let mut depth = None;
    if let Value::Number(n) = arg {
        depth = Some(crate::ecma_conversions::f64_to_wrapping_i32(n).wrapping_add(AVM_DEPTH_BIAS));
    } else if let Some(target) = activation.resolve_target_display_object(movie_clip.into(), arg)? {
        if let Some(target_parent) = target.parent() {
            if DisplayObject::ptr_eq(target_parent, parent.into()) {
                depth = Some(target.depth())
            } else {
                avm_warn!(
                    activation,
                    "MovieClip.swapDepths: Objects do not have the same parent"
                );
            }
        }
    } else {
        avm_warn!(activation, "MovieClip.swapDepths: Invalid target");
    };

    if let Some(depth) = depth {
        if depth < 0 || depth > AVM_MAX_DEPTH {
            // Depth out of range; no action.
            return Ok(Value::Undefined);
        }

        if depth != movie_clip.depth() {
            parent.swap_child_to_depth(&mut activation.context, movie_clip.into(), depth);
        }
    }

    Ok(Value::Undefined)
}

fn local_to_global<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Value::Object(point) = args.get(0).unwrap_or(&Value::Undefined) {
        // localToGlobal does no coercion; it fails if the properties are not numbers.
        // It does not search the prototype chain.
        if let (Value::Number(x), Value::Number(y)) = (
            point.get_local("x", activation, *point)?,
            point.get_local("y", activation, *point)?,
        ) {
            let x = Twips::from_pixels(x);
            let y = Twips::from_pixels(y);
            let (out_x, out_y) = movie_clip.local_to_global((x, y));
            point.set("x", out_x.to_pixels().into(), activation)?;
            point.set("y", out_y.to_pixels().into(), activation)?;
        } else {
            avm_warn!(
                activation,
                "MovieClip.localToGlobal: Invalid x and y properties"
            );
        }
    } else {
        avm_warn!(
            activation,
            "MovieClip.localToGlobal: Missing point parameter"
        );
    }

    Ok(Value::Undefined)
}

fn get_bounds<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let target = match args.get(0) {
        Some(Value::String(s)) if s.is_empty() => None,
        Some(Value::Object(o)) if o.as_display_object().is_some() => o.as_display_object(),
        Some(val) => {
            let path = val.coerce_to_string(activation)?;
            activation.resolve_target_display_object(
                movie_clip.into(),
                AvmString::new(activation.context.gc_context, path.to_string()).into(),
            )?
        }
        None => Some(movie_clip.into()),
    };

    if let Some(target) = target {
        let bounds = movie_clip.bounds();
        let out_bounds = if DisplayObject::ptr_eq(movie_clip.into(), target) {
            // Getting the clips bounds in its own coordinate space; no AABB transform needed.
            bounds
        } else {
            // Transform AABB to target space.
            // Calculate the matrix to transform into the target coordinate space, and transform the above AABB.
            // Note that this doesn't produce as tight of an AABB as if we had used `bounds_with_transform` with
            // the final matrix, but this matches Flash's behavior.
            let to_global_matrix = movie_clip.local_to_global_matrix();
            let to_target_matrix = target.global_to_local_matrix();
            let bounds_transform = to_target_matrix * to_global_matrix;
            bounds.transform(&bounds_transform)
        };

        let out = ScriptObject::object(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes.object),
        );
        out.set("xMin", out_bounds.x_min.to_pixels().into(), activation)?;
        out.set("yMin", out_bounds.y_min.to_pixels().into(), activation)?;
        out.set("xMax", out_bounds.x_max.to_pixels().into(), activation)?;
        out.set("yMax", out_bounds.y_max.to_pixels().into(), activation)?;
        Ok(out.into())
    } else {
        Ok(Value::Undefined)
    }
}

fn get_rect<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: This should get the bounds ignoring strokes. Always equal to or smaller than getBounds.
    // Just defer to getBounds for now. Will have to store edge_bounds vs. shape_bounds in Graphic.
    get_bounds(movie_clip, activation, args)
}

#[allow(unused_must_use)] //can't use errors yet
pub fn get_url<'gc>(
    _movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,

    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    use crate::avm1::fscommand;

    //TODO: Error behavior if no arguments are present
    if let Some(url_val) = args.get(0) {
        let url = url_val.coerce_to_string(activation)?;
        if let Some(fscommand) = fscommand::parse(&url) {
            fscommand::handle(fscommand, activation);
            return Ok(Value::Undefined);
        }

        let window = if let Some(window) = args.get(1) {
            Some(window.coerce_to_string(activation)?.to_string())
        } else {
            None
        };
        let method = match args.get(2) {
            Some(Value::String(s)) if *s == "GET" => Some(NavigationMethod::GET),
            Some(Value::String(s)) if *s == "POST" => Some(NavigationMethod::POST),
            _ => None,
        };
        let vars_method = method.map(|m| (m, activation.locals_into_form_values()));

        activation
            .context
            .navigator
            .navigate_to_url(url.to_string(), window, vars_method);
    }

    Ok(Value::Undefined)
}

fn global_to_local<'gc>(
    movie_clip: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Value::Object(point) = args.get(0).unwrap_or(&Value::Undefined) {
        // globalToLocal does no coercion; it fails if the properties are not numbers.
        // It does not search the prototype chain.
        if let (Value::Number(x), Value::Number(y)) = (
            point.get_local("x", activation, *point)?,
            point.get_local("y", activation, *point)?,
        ) {
            let x = Twips::from_pixels(x);
            let y = Twips::from_pixels(y);
            let (out_x, out_y) = movie_clip.global_to_local((x, y));
            point.set("x", out_x.to_pixels().into(), activation)?;
            point.set("y", out_y.to_pixels().into(), activation)?;
        } else {
            avm_warn!(
                activation,
                "MovieClip.globalToLocal: Invalid x and y properties"
            );
        }
    } else {
        avm_warn!(
            activation,
            "MovieClip.globalToLocal: Missing point parameter"
        );
    }

    Ok(Value::Undefined)
}

fn load_movie<'gc>(
    target: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let url_val = args.get(0).cloned().unwrap_or(Value::Undefined);
    let url = url_val.coerce_to_string(activation)?;
    let method = args.get(1).cloned().unwrap_or(Value::Undefined);
    let method = NavigationMethod::from_method_str(&method.coerce_to_string(activation)?);
    let (url, opts) = activation.locals_into_request_options(Cow::Borrowed(&url), method);
    let fetch = activation.context.navigator.fetch(&url, opts);
    let process = activation.context.load_manager.load_movie_into_clip(
        activation.context.player.clone().unwrap(),
        DisplayObject::MovieClip(target),
        fetch,
        url.to_string(),
        None,
    );

    activation.context.navigator.spawn_future(process);

    Ok(Value::Undefined)
}

fn load_variables<'gc>(
    target: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let url_val = args.get(0).cloned().unwrap_or(Value::Undefined);
    let url = url_val.coerce_to_string(activation)?;
    let method = args.get(1).cloned().unwrap_or(Value::Undefined);
    let method = NavigationMethod::from_method_str(&method.coerce_to_string(activation)?);
    let (url, opts) = activation.locals_into_request_options(Cow::Borrowed(&url), method);
    let fetch = activation.context.navigator.fetch(&url, opts);
    let target = target.object().coerce_to_object(activation);
    let process = activation.context.load_manager.load_form_into_object(
        activation.context.player.clone().unwrap(),
        target,
        fetch,
    );

    activation.context.navigator.spawn_future(process);

    Ok(Value::Undefined)
}

fn unload_movie<'gc>(
    mut target: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    target.unload(&mut activation.context);
    target.replace_with_movie(activation.context.gc_context, None);

    Ok(Value::Undefined)
}

fn transform<'gc>(
    this: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    let constructor = activation.context.avm1.prototypes.transform_constructor;
    let cloned = constructor.construct(activation, &[this.object()])?;
    Ok(cloned.into())
}

fn set_transform<'gc>(
    this: MovieClip<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let transform = value.coerce_to_object(activation);
    crate::avm1::globals::transform::apply_to_display_object(activation, transform, this.into())?;
    Ok(())
}
