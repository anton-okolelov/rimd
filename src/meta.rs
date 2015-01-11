use std::error;
use std::io::{IoError,Reader};
use std::fmt;
use std::num::{FromPrimitive,Int};
use SMF;

pub enum MetaError {
    InvalidCommand,
    OtherErr(&'static str),
    IoError(IoError),
}

impl error::FromError<IoError> for MetaError {
    fn from_error(err: IoError) -> MetaError {
        MetaError::IoError(err)
    }
}

#[derive(FromPrimitive)]
pub enum MetaCommand {
    SequenceNumber = 0x00,
    TextEvent = 0x01,
    CopyrightNotice = 0x02,
    SequenceOrTrackName = 0x03,
    InstrumentName = 0x04,
    LyricText = 0x05,
    MarkerText = 0x06,
    CuePoint = 0x07,
    MIDIChannelPrefixAssignment = 0x20,
    MIDIPortPrefixAssignment = 0x21,
    EndOfTrack = 0x2F,
    TempoSetting = 0x51,
    SMPTEOffset = 0x54,
    TimeSignature = 0x58,
    KeySignature = 0x59,
    SequencerSpecificEvent = 0x7F,
    Unknown,
}

pub struct MetaEvent {
    pub command: MetaCommand,
    pub length: u64,
    pub data: Vec<u8>,
}

impl fmt::Show for MetaEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Meta Event: {}",
               match self.command {
                   MetaCommand::SequenceNumber => format!("Sequence Number"),
                   MetaCommand::TextEvent => {
                       format!("Text Event.  Len: {} Text: Foo",self.length)
                   },
                   MetaCommand::CopyrightNotice => format!("Copyright Notice"),
                   MetaCommand::SequenceOrTrackName => {
                       let text = match String::from_utf8(self.data.clone()) {
                           Ok(s) => s,
                           Err(_) => format!("[invalid string data]"),
                       };
                       format!("Sequence/Track Name, length: {}, name: {}",self.length,text)
                   },
                   MetaCommand::InstrumentName => format!("InstrumentName"),
                   MetaCommand::LyricText => format!("LyricText"),
                   MetaCommand::MarkerText => format!("MarkerText"),
                   MetaCommand::CuePoint => format!("CuePoint"),
                   MetaCommand::MIDIChannelPrefixAssignment => format!("MIDI Channel Prefix Assignment, channel: {}", self.data[0]+1),
                   MetaCommand::MIDIPortPrefixAssignment => format!("MIDI Port Prefix Assignment, port: {}", self.data[0]),
                   MetaCommand::EndOfTrack => format!("End Of Track"),
                   MetaCommand::TempoSetting => format!("Set Tempo, microseconds/quarter note: {}",self.data_as_u64(3)),
                   MetaCommand::SMPTEOffset => format!("SMPTEOffset"),
                   MetaCommand::TimeSignature => format!("Time Signature: {}/{}, {} ticks/metronome click, {} 32nd notes/quarter note",
                                                         self.data[0],
                                                         2u.pow(self.data[1] as uint),
                                                         self.data[2],
                                                         self.data[3]),
                   MetaCommand::KeySignature => format!("Key Signature, {} sharps/flats, {}",
                                                        self.data[0] as i8,
                                                        match self.data[1] {
                                                            0 => "Major",
                                                            1 => "Minor",
                                                            _ => "Invalid Signature",
                                                        }),
                   MetaCommand::SequencerSpecificEvent => format!("SequencerSpecificEvent"),
                   MetaCommand::Unknown => format!("Unknown, length: {}",self.data.len()),
               })
    }
}


impl MetaEvent {

    pub fn data_as_u64(&self,bytes: uint) -> u64 {
        let mut res = 0;
        for i in range(0,bytes) {
            res <<= 8;
            res |= self.data[i] as u64;
        }
        res
    }

    pub fn next_event(reader: &mut Reader) -> Result<MetaEvent, MetaError> {
        let command =
            match FromPrimitive::from_u8(try!(reader.read_byte())) {
                Some(c) => {c},
                None => MetaCommand::Unknown,
            };
        let len = match SMF::read_vtime(reader) {
            Ok(t) => { t }
            Err(_) => { return Err(MetaError::OtherErr("Couldn't read time for meta command")); }
        };
        let data = try!(reader.read_exact(len as uint));
        Ok(MetaEvent{
            command: command,
            length: len,
            data: data
        })
    }
}