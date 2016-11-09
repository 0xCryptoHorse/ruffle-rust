use avm1::types::{Action, Value};
use avm1::opcode::OpCode;
use read::SwfRead;
use std::io::{Error, ErrorKind, Read, Result};

pub struct Reader<R: Read> {
    inner: R,
    version: u8,
}

impl<R: Read> SwfRead<R> for Reader<R> {
    fn get_inner(&mut self) -> &mut R {
        &mut self.inner
    }
}

impl<R: Read> Reader<R> {
    pub fn new(inner: R, version: u8) -> Reader<R> {
        Reader { inner: inner, version: version }
    }

    pub fn read_action_list(&mut self) -> Result<Vec<Action>> {
        let mut actions = Vec::new();
        while let Some(action) = try!(self.read_action()) {
            actions.push(action);
        }
        Ok(actions)
    }

    pub fn read_action(&mut self) -> Result<Option<Action>> {
        let (opcode, length) = try!(self.read_opcode_and_length());

        let mut action_reader = Reader::new(self.inner.by_ref().take(length as u64), self.version);

        use num::FromPrimitive;
        let action = match OpCode::from_u8(opcode) {
            Some(OpCode::End) => return Ok(None),

            Some(OpCode::GetUrl) => Action::GetUrl {
                url: try!(action_reader.read_c_string()),
                target: try!(action_reader.read_c_string()),
            },
            Some(OpCode::GotoFrame) => {
                let frame = try!(action_reader.read_u16());
                Action::GotoFrame(frame)
            },
            Some(OpCode::GotoLabel) => Action::GotoLabel(try!(action_reader.read_c_string())),
            Some(OpCode::NextFrame) => Action::NextFrame,
            Some(OpCode::Play) => Action::Play,
            Some(OpCode::Pop) => Action::Pop,
            Some(OpCode::PreviousFrame) => Action::PreviousFrame,
            // TODO: Verify correct version for complex types.
            Some(OpCode::Push) => {
                let mut values = vec![];
                while let Ok(value) = action_reader.read_push_value() {
                    values.push(value);
                };
                Action::Push(values)
            },
            Some(OpCode::SetTarget) => Action::SetTarget(try!(action_reader.read_c_string())),
            Some(OpCode::Stop) => Action::Stop,
            Some(OpCode::StopSounds) => Action::StopSounds,
            Some(OpCode::ToggleQuality) => Action::ToggleQuality,
            Some(OpCode::WaitForFrame) => Action::WaitForFrame {
                frame: try!(action_reader.read_u16()),
                num_actions_to_skip: try!(action_reader.read_u8()),
            },
            _ => {
                let mut data = Vec::with_capacity(length);
                try!(action_reader.inner.read_to_end(&mut data));
                Action::Unknown { opcode: opcode, data: data }
            }
        };

        Ok(Some(action))
    }

    pub fn read_opcode_and_length(&mut self) -> Result<(u8, usize)> {
        let opcode = try!(self.read_u8());
        let length = if opcode >= 0x80 {
            try!(self.read_u16()) as usize
        } else { 0 };
        Ok((opcode, length))
    }

    fn read_push_value(&mut self) -> Result<Value> {
        let value = match try!(self.read_u8()) {
            0 => Value::Str(try!(self.read_c_string())),
            1 => Value::Float(try!(self.read_f32())),
            2 => Value::Null,
            3 => Value::Undefined,
            4 => Value::Register(try!(self.read_u8())),
            5 => Value::Bool(try!(self.read_u8()) != 0),
            6 => Value::Double(try!(self.read_f64())),
            7 => Value::Int(try!(self.read_u32())),
            8 => Value::ConstantPool(try!(self.read_u8()) as u16),
            9 => Value::ConstantPool(try!(self.read_u16())),
            _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid value type in ActionPush")),
        };
        Ok(value)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use test_data;

    #[test]
    fn read_action() {
        for (swf_version, expected_action, action_bytes) in test_data::avm1_tests() {
            let mut reader = Reader::new(&action_bytes[..], swf_version);
            let parsed_action = reader.read_action().unwrap().unwrap();
            if parsed_action != expected_action {
                // Failed, result doesn't match.
                panic!(
                    "Incorrectly parsed action.\nRead:\n{:?}\n\nExpected:\n{:?}",
                    parsed_action,
                    expected_action
                );
            }
        }
    }
}