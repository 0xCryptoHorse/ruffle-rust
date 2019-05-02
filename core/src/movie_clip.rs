use crate::audio::AudioStreamHandle;
use crate::character::Character;
use crate::color_transform::ColorTransform;
use crate::display_object::{
    DisplayObject, DisplayObjectBase, DisplayObjectImpl, DisplayObjectUpdate,
};
use crate::matrix::Matrix;
use crate::player::{RenderContext, UpdateContext};
use bacon_rajan_cc::{Cc, Trace, Tracer};
use log::info;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use swf::read::SwfRead;

type Depth = i16;
type FrameNumber = u16;

pub struct MovieClip {
    base: DisplayObjectBase,
    tag_stream_start: Option<u64>,
    tag_stream_pos: u64,
    is_playing: bool,
    goto_queue: VecDeque<FrameNumber>,
    current_frame: FrameNumber,
    total_frames: FrameNumber,
    audio_stream: Option<AudioStreamHandle>,
    children: HashMap<Depth, Cc<RefCell<DisplayObject>>>,
}

impl_display_object!(MovieClip, base);

impl MovieClip {
    pub fn new() -> MovieClip {
        MovieClip {
            base: Default::default(),
            tag_stream_start: None,
            tag_stream_pos: 0,
            is_playing: true,
            goto_queue: VecDeque::new(),
            current_frame: 0,
            total_frames: 1,
            audio_stream: None,
            children: HashMap::new(),
        }
    }

    pub fn new_with_data(tag_stream_start: u64, num_frames: u16) -> MovieClip {
        MovieClip {
            base: Default::default(),
            tag_stream_start: Some(tag_stream_start),
            tag_stream_pos: tag_stream_start,
            is_playing: true,
            goto_queue: VecDeque::new(),
            current_frame: 0,
            audio_stream: None,
            total_frames: num_frames,
            children: HashMap::new(),
        }
    }

    pub fn playing(&self) -> bool {
        self.is_playing
    }

    pub fn next_frame(&mut self) {
        if self.current_frame + 1 <= self.total_frames {
            self.goto_frame(self.current_frame + 1, true);
        }
    }

    pub fn play(&mut self) {
        self.is_playing = true;
    }

    pub fn prev_frame(&mut self) {
        if self.current_frame > 1 {
            self.goto_frame(self.current_frame - 1, true);
        }
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
    }

    pub fn goto_frame(&mut self, frame: FrameNumber, stop: bool) {
        self.goto_queue.push_back(frame);

        if stop {
            self.stop();
        } else {
            self.play();
        }
    }

    fn run_goto_queue(&mut self, context: &mut UpdateContext) {
        let mut i = 0;
        while i < self.goto_queue.len() {
            let frame = self.goto_queue[i];
            if frame >= self.current_frame {
                // Advancing
                while self.current_frame + 1 < frame {
                    self.run_frame_internal(context, true);
                }
                self.run_frame_internal(context, false);
            } else {
                // Rewind
                // Reset everything to blank, start from frame 1,
                // and advance forward
                self.children.clear();
                self.tag_stream_pos = self.tag_stream_start.unwrap_or(0);
                self.current_frame = 0;
                while self.current_frame + 1 < frame {
                    self.run_frame_internal(context, true);
                }
                self.run_frame_internal(context, false);
            }

            i += 1;
        }

        self.goto_queue.clear();
    }

    pub fn run_place_object(
        children: &mut HashMap<Depth, Cc<RefCell<DisplayObject>>>,
        place_object: &swf::PlaceObject,
        context: &mut UpdateContext,
    ) {
        use swf::PlaceObjectAction;
        let character = match place_object.action {
            PlaceObjectAction::Place(id) => {
                // TODO(Herschel): Behavior when character doesn't exist/isn't a DisplayObject?
                let character =
                    if let Ok(character) = context.library.instantiate_display_object(id) {
                        Cc::new(RefCell::new(character))
                    } else {
                        return;
                    };

                // TODO(Herschel): Behavior when depth is occupied? (I think it replaces)
                children.insert(place_object.depth, character.clone());
                character
            }
            PlaceObjectAction::Modify => {
                if let Some(child) = children.get(&place_object.depth) {
                    child.clone()
                } else {
                    return;
                }
            }
            PlaceObjectAction::Replace(id) => {
                let character =
                    if let Ok(character) = context.library.instantiate_display_object(id) {
                        Cc::new(RefCell::new(character))
                    } else {
                        return;
                    };

                if let Some(prev_character) = children.insert(place_object.depth, character.clone())
                {
                    character
                        .borrow_mut()
                        .set_matrix(prev_character.borrow().get_matrix());
                    character
                        .borrow_mut()
                        .set_color_transform(prev_character.borrow().get_color_transform());
                }
                character
            }
        };

        let mut character = character.borrow_mut();
        if let Some(matrix) = &place_object.matrix {
            let m = matrix.clone();
            character.set_matrix(&Matrix::from(m));
        }

        if let Some(color_transform) = &place_object.color_transform {
            character.set_color_transform(&ColorTransform::from(color_transform.clone()));
        }
    }

    fn do_action(&mut self, context: &mut UpdateContext, data: &[u8]) {
        let mut action_context = crate::avm1::ActionContext {
            global_time: context.global_time,
            active_clip: self,
            audio: context.audio,
        };
        if let Err(e) = context.avm1.do_action(&mut action_context, &data[..]) {}
    }

    fn run_frame_internal(&mut self, context: &mut UpdateContext, only_display_actions: bool) {
        use swf::Tag;

        // Advance frame number.
        if self.current_frame < self.total_frames {
            self.current_frame += 1;
        } else {
            self.current_frame = 1;
            self.children.clear();
            if let Some(start) = self.tag_stream_start {
                self.tag_stream_pos = start;
            }
        }

        context
            .tag_stream
            .get_inner()
            .set_position(self.tag_stream_pos);

        let mut start_pos = self.tag_stream_pos;

        while let Ok(Some(tag)) = context.tag_stream.read_tag() {
            if only_display_actions {
                match tag {
                    Tag::ShowFrame => break,
                    Tag::PlaceObject(place_object) => {
                        MovieClip::run_place_object(&mut self.children, &*place_object, context)
                    }
                    Tag::RemoveObject { depth, .. } => {
                        // TODO(Herschel): How does the character ID work for RemoveObject?
                        self.children.remove(&depth);
                    }

                    // All non-display-list tags get ignored.
                    _ => (),
                }
            } else {
                match tag {
                    Tag::SetBackgroundColor(color) => *context.background_color = color,
                    Tag::DefineButton2(button) => {
                        if !context.library.contains_character(button.id) {
                            context
                                .library
                                .register_character(button.id, Character::Button(button));
                        }
                    }
                    Tag::DefineShape(shape) => {
                        if !context.library.contains_character(shape.id) {
                            let shape_handle = context.renderer.register_shape(&shape);
                            context.library.register_character(
                                shape.id,
                                Character::Graphic {
                                    shape_handle,
                                    x_min: shape.shape_bounds.x_min,
                                    y_min: shape.shape_bounds.y_min,
                                },
                            );
                        }
                    }
                    Tag::DefineSprite(sprite) => {
                        let pos = context.tag_stream.get_ref().position();
                        context.tag_stream.get_inner().set_position(start_pos);
                        context.tag_stream.read_tag_code_and_length().unwrap();
                        context.tag_stream.read_u32().unwrap();
                        let mc_start_pos = context.tag_stream.get_ref().position();
                        context.tag_stream.get_inner().set_position(pos);
                        if !context.library.contains_character(sprite.id) {
                            context.library.register_character(
                                sprite.id,
                                Character::MovieClip {
                                    num_frames: sprite.num_frames,
                                    tag_stream_start: mc_start_pos,
                                },
                            );
                        }
                    }

                    Tag::ShowFrame => break,
                    Tag::PlaceObject(place_object) => {
                        MovieClip::run_place_object(&mut self.children, &*place_object, context)
                    }
                    Tag::RemoveObject { depth, .. } => {
                        // TODO(Herschel): How does the character ID work for RemoveObject?
                        self.children.remove(&depth);
                    }

                    Tag::SoundStreamHead(info) => self.sound_stream_head(&info, context, 0, 1),
                    Tag::SoundStreamHead2(info) => self.sound_stream_head(&info, context, 0, 2),
                    Tag::SoundStreamBlock(samples) => {
                        self.sound_stream_block(&samples[..], context, 0)
                    }

                    Tag::JpegTables(_) => (),
                    Tag::DoAction(data) => self.do_action(context, &data[..]),
                    _ => info!("Umimplemented tag: {:?}", tag),
                }
                start_pos = context.tag_stream.get_ref().position();
            }
        }

        self.tag_stream_pos = context.tag_stream.get_ref().position();
    }

    fn sound_stream_head(
        &mut self,
        stream_info: &swf::SoundStreamInfo,
        context: &mut UpdateContext,
        _length: usize,
        _version: u8,
    ) {
        if self.audio_stream.is_none() {
            self.audio_stream = Some(context.audio.register_stream(stream_info));
        }
    }

    fn sound_stream_block(&mut self, samples: &[u8], context: &mut UpdateContext, _length: usize) {
        if let Some(stream) = self.audio_stream {
            context.audio.queue_stream_samples(stream, samples)
        }
    }
}

impl DisplayObjectUpdate for MovieClip {
    fn run_frame(&mut self, context: &mut UpdateContext) {
        if self.is_playing && self.tag_stream_start.is_some() {
            context
                .position_stack
                .push(context.tag_stream.get_ref().position());

            self.run_frame_internal(context, false);

            // TODO(Herschel): Verify order of execution for parent/children.
            // Parent first? Children first? Sorted by depth?
            for child in self.children.values() {
                child.borrow_mut().run_frame(context);
            }

            context
                .tag_stream
                .get_inner()
                .set_position(context.position_stack.pop().unwrap());
        }
    }

    fn run_post_frame(&mut self, context: &mut UpdateContext) {
        self.run_goto_queue(context);

        for child in self.children.values() {
            child.borrow_mut().run_post_frame(context);
        }
    }

    fn render(&self, context: &mut RenderContext) {
        context.transform_stack.push(self.transform());

        let mut sorted_children: Vec<_> = self.children.iter().collect();
        sorted_children.sort_by_key(|(depth, _)| *depth);

        for child in sorted_children {
            child.1.borrow_mut().render(context);
        }

        context.transform_stack.pop();
    }
}

impl Trace for MovieClip {
    fn trace(&mut self, tracer: &mut Tracer) {
        for child in self.children.values_mut() {
            child.trace(tracer);
        }
    }
}
