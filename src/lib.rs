//! Types for reading _Service Information_ from a DVB MPEG Transport Stream, formatted according
//! to  [ETSI standard EN 300 486](http://www.etsi.org/deliver/etsi_en/300400_300499/300468/01.15.01_60/en_300468v011501p.pdf).
#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, future_incompatible)]

pub mod sdt;

use mpeg2ts_reader::descriptor::UnknownDescriptor;

use crate::sdt::ServiceDescriptor;
use std::borrow::Cow;
use std::fmt;

mpeg2ts_reader::descriptor_enum! {
    /// All descriptors supported by this crate.
    ///
    /// Future releases of this crate should replace most `UnknownDescriptor` with
    /// descriptor-specific implementations.
    #[derive(Debug)]
    En300_468Descriptors {
        Reserved 0|1|36..=63 => UnknownDescriptor,
        VideoStream 2 => UnknownDescriptor,
        AudioStream 3 => UnknownDescriptor,
        Hierarchy 4 => UnknownDescriptor,
        Registration 5 => UnknownDescriptor,
        DataStreamAlignment 6 => UnknownDescriptor,
        TargetBackgroundGrid 7 => UnknownDescriptor,
        VideoWindow 8 => UnknownDescriptor,
        CA 9 => UnknownDescriptor,
        ISO639Language 10 => UnknownDescriptor,
        SystemClock 11 => UnknownDescriptor,
        MultiplexBufferUtilization 12 => UnknownDescriptor,
        Copyright 13 => UnknownDescriptor,
        MaximumBitrate 14 => UnknownDescriptor,
        PrivateDataIndicator 15 => UnknownDescriptor,
        SmoothingBuffer 16 => UnknownDescriptor,
        STD 17 => UnknownDescriptor,
        IBP 18 => UnknownDescriptor,
        /// ISO IEC 13818-6
        IsoIec13818dash6 19..=26 => UnknownDescriptor,
        MPEG4Video 27 => UnknownDescriptor,
        MPEG4Audio 28 => UnknownDescriptor,
        IOD 29 => UnknownDescriptor,
        SL 30 => UnknownDescriptor,
        FMC 31 => UnknownDescriptor,
        ExternalESID 32 => UnknownDescriptor,
        MuxCode 33 => UnknownDescriptor,
        FmxBufferSize 34 => UnknownDescriptor,
        MultiplexBuffer 35 => UnknownDescriptor,
        UserPrivate 69..=70|103..=254 => UnknownDescriptor,

        // EN 300 480 specofic descriptors,
        NetworkName 0x40 => UnknownDescriptor,
        ServiceList 0x41 => UnknownDescriptor,
        Stuffing 0x42 => UnknownDescriptor,
        SatelliteDeliverySystem 0x43 => UnknownDescriptor,
        CableDeliverySystem 0x44 => UnknownDescriptor,
        BouquetName 0x47 => UnknownDescriptor,

        Service ServiceDescriptor::TAG => ServiceDescriptor,

        CountryAvailability 0x49 => UnknownDescriptor,
        Linkage 0x4A => UnknownDescriptor,
        NvodReference 0x4B => UnknownDescriptor,
        TimeShiftedService 0x4C => UnknownDescriptor,
        ShortEvent 0x4D => UnknownDescriptor,
        ExtendedEvent 0x4E => UnknownDescriptor,
        TimeShiftedEvent 0x4F => UnknownDescriptor,
        Component 0x50 => UnknownDescriptor,
        Mosaic 0x51 => UnknownDescriptor,
        StreamIdentifier 0x52 => UnknownDescriptor,
        CaIdentifier 0x53 => UnknownDescriptor,
        Content 0x54 => UnknownDescriptor,
        ParentalRating 0x55 => UnknownDescriptor,
        Teletext 0x56 => UnknownDescriptor,
        Telephone 0x57 => UnknownDescriptor,
        LocalTimeOffset 0x58 => UnknownDescriptor,
        Subtitling 0x59 => UnknownDescriptor,
        TerrestrialDeliverySystem 0x5A => UnknownDescriptor,
        MultilingualNetworkName 0x5B => UnknownDescriptor,
        MultilingualBouquetName 0x5C => UnknownDescriptor,
        MultilingualServiceName 0x5D => UnknownDescriptor,
        MultilingualComponent 0x5E => UnknownDescriptor,
        PrivateDataSpecifier 0x5F => UnknownDescriptor,
        ServiceMove 0x60 => UnknownDescriptor,
        ShortSmoothingBuffer 0x61 => UnknownDescriptor,
        FrequencyList 0x62 => UnknownDescriptor,
        PartialTransportStream 0x63 => UnknownDescriptor,
        DataBroadcast 0x64 => UnknownDescriptor,
        CaSystem 0x65 => UnknownDescriptor,
        DataBroadcastId 0x66 => UnknownDescriptor,
        Forbidden 0xFF => UnknownDescriptor,
    }
}

/// Text encodings as defined by _ETSI EN 300 468_, used by the [`Text type`](struct.Text.html).
#[derive(Debug)]
pub enum TextEncoding {
    Reserved1(u8),
    Reserved2(u8, u8),
    /// ISO 8859-1
    Iso88591,
    /// ISO 8859-2
    Iso88592,
    /// ISO 8859-3
    Iso88593,
    /// ISO 8859-4
    Iso88594,
    /// ISO 8859-5
    Iso88595,
    /// ISO 8859-6
    Iso88596,
    /// ISO 8859-7
    Iso88597,
    /// ISO 8859-8
    Iso88598,
    /// ISO 8859-9
    Iso88599,
    /// ISO 8859-10
    Iso885910,
    /// ISO 8859-11
    Iso885911,
    /// ISO 8859-13
    Iso885913,
    /// ISO 8859-14
    Iso885914,
    /// ISO 8859-15
    Iso885915,
    /// ISO 10646
    Iso10646,
    /// KSX1001-2004
    KSX1001_2004,
    /// GB-2312-1980
    GB2312_1980,
    /// Big5 subset of ISO/IEC 10646
    Big5,
    /// UTF-8,
    UTF8,
}

/// There are several pieces of metadata in the spec that may apply to the 'actual' transport
/// stream (i.e. the one containing the metadata) or some 'other' transport stream.  This wrapper
/// allows these cases to be discriminated.
///
/// The `Other` variant allows metadata to be announced for services that are actually broadcast
/// in a different multiplex (on a different frequency), for example.
pub enum ActualOther<T> {
    /// The wrapped information pertains to the current transport stream / network.
    Actual(T),
    /// The wrapped information pertains to some other transport stream / network.
    Other(T),
}
impl<T> ActualOther<T> {
    pub fn actual(&self) -> Option<&T> {
        match self {
            ActualOther::Actual(ref v) => Some(v),
            ActualOther::Other(_) => None,
        }
    }
    pub fn other(&self) -> Option<&T> {
        match self {
            ActualOther::Actual(_) => None,
            ActualOther::Other(ref v) => Some(v),
        }
    }
}

/// A problem encountered by [`Text::to_string()`](struct.Text.html#method.to_string).
#[derive(Debug)]
pub enum TextError {
    NotEnoughData { expected: usize, available: usize },
    DecodeFailure,
    UnsupportedEncoding(TextEncoding),
}

/// A wrapper around bytes representing text having embedded encoding information, with
/// functionality for trying to decode this a Rust `String`.
pub struct Text<'buf> {
    data: &'buf [u8],
}
impl<'buf> Text<'buf> {
    pub fn new(data: &'buf [u8]) -> Result<Text<'buf>, TextError> {
        if data.is_empty() {
            Err(TextError::NotEnoughData {
                expected: 1,
                available: 0,
            })
        } else {
            Ok(Text { data })
        }
    }
    pub fn encoding(&self) -> TextEncoding {
        let id = self.data[0];
        match id {
            0x20..=0xff => TextEncoding::Iso88591,
            0x01 => TextEncoding::Iso88595,
            0x02 => TextEncoding::Iso88596,
            0x03 => TextEncoding::Iso88597,
            0x04 => TextEncoding::Iso88598,
            0x05 => TextEncoding::Iso88599,
            0x06 => TextEncoding::Iso885910,
            0x07 => TextEncoding::Iso885911,
            0x08 => TextEncoding::Reserved1(id),
            0x09 => TextEncoding::Iso885913,
            0x0a => TextEncoding::Iso885914,
            0x0b => TextEncoding::Iso885915,
            0x0c..=0x0f => TextEncoding::Reserved1(id),
            0x10 => {
                let ids = (self.data[1], self.data[2]);
                match ids {
                    (0x00, 0x00) => TextEncoding::Reserved2(ids.0, ids.1),
                    (0x00, 0x01) => TextEncoding::Iso88591,
                    (0x00, 0x02) => TextEncoding::Iso88592,
                    (0x00, 0x03) => TextEncoding::Iso88593,
                    (0x00, 0x04) => TextEncoding::Iso88594,
                    (0x00, 0x05) => TextEncoding::Iso88595,
                    (0x00, 0x06) => TextEncoding::Iso88596,
                    (0x00, 0x07) => TextEncoding::Iso88597,
                    (0x00, 0x08) => TextEncoding::Iso88598,
                    (0x00, 0x09) => TextEncoding::Iso88599,
                    (0x00, 0x0a) => TextEncoding::Iso885910,
                    (0x00, 0x0b) => TextEncoding::Iso885911,
                    (0x00, 0x0c) => TextEncoding::Reserved2(ids.0, ids.1),
                    (0x00, 0x0d) => TextEncoding::Iso885913,
                    (0x00, 0x0e) => TextEncoding::Iso885914,
                    (0x00, 0x0f) => TextEncoding::Iso885915,
                    _ => TextEncoding::Reserved2(ids.0, ids.1),
                }
            }
            0x11 => TextEncoding::Iso10646,
            0x12 => TextEncoding::KSX1001_2004,
            0x13 => TextEncoding::GB2312_1980,
            0x14 => TextEncoding::Big5,
            0x15 => TextEncoding::UTF8,
            0x16..=0x1E => TextEncoding::Reserved1(id),
            0x1F => unimplemented!("encoding_type_id"),
            _ => unreachable!(),
        }
    }
    fn buffer(&self) -> Result<&'buf [u8], TextError> {
        Ok(&self.data[self.enc_prefix_len()?..])
    }
    fn enc_prefix_len(&self) -> Result<usize, TextError> {
        match self.data[0] {
            0x01..=0x0f | 0x11..=0x1e => Ok(1),
            0x1f => Ok(2), // encoding_type_id in second byte
            0x10 => {
                if self.data.len() < 3 {
                    Err(TextError::NotEnoughData {
                        expected: 3,
                        available: self.data.len(),
                    })
                } else {
                    Ok(3)
                }
            }
            0x20..=0xff => Ok(0),
            _ => unreachable!(),
        }
    }

    pub fn to_string(&self) -> Result<Cow<'_, str>, TextError> {
        let enc = self.encoding();
        match enc {
            TextEncoding::Iso88591 => Ok(encoding_rs::mem::decode_latin1(self.buffer()?)),
            TextEncoding::Iso88592 => encoding_rs::ISO_8859_2
                .decode_without_bom_handling_and_without_replacement(self.buffer()?)
                .ok_or(TextError::DecodeFailure),
            TextEncoding::Iso88593 => encoding_rs::ISO_8859_3
                .decode_without_bom_handling_and_without_replacement(self.buffer()?)
                .ok_or(TextError::DecodeFailure),
            TextEncoding::Iso88594 => encoding_rs::ISO_8859_4
                .decode_without_bom_handling_and_without_replacement(self.buffer()?)
                .ok_or(TextError::DecodeFailure),
            TextEncoding::Iso88595 => encoding_rs::ISO_8859_5
                .decode_without_bom_handling_and_without_replacement(self.buffer()?)
                .ok_or(TextError::DecodeFailure),
            TextEncoding::Iso88596 => encoding_rs::ISO_8859_6
                .decode_without_bom_handling_and_without_replacement(self.buffer()?)
                .ok_or(TextError::DecodeFailure),
            TextEncoding::Iso88597 => encoding_rs::ISO_8859_7
                .decode_without_bom_handling_and_without_replacement(self.buffer()?)
                .ok_or(TextError::DecodeFailure),
            TextEncoding::Iso88598 => encoding_rs::ISO_8859_8
                .decode_without_bom_handling_and_without_replacement(self.buffer()?)
                .ok_or(TextError::DecodeFailure),
            TextEncoding::Iso88599 => Err(TextError::UnsupportedEncoding(enc)),
            TextEncoding::Iso885910 => encoding_rs::ISO_8859_10
                .decode_without_bom_handling_and_without_replacement(self.buffer()?)
                .ok_or(TextError::DecodeFailure),
            TextEncoding::Iso885911 => Err(TextError::UnsupportedEncoding(enc)),
            TextEncoding::Iso885913 => encoding_rs::ISO_8859_13
                .decode_without_bom_handling_and_without_replacement(self.buffer()?)
                .ok_or(TextError::DecodeFailure),
            TextEncoding::Iso885914 => encoding_rs::ISO_8859_14
                .decode_without_bom_handling_and_without_replacement(self.buffer()?)
                .ok_or(TextError::DecodeFailure),
            TextEncoding::Iso885915 => encoding_rs::ISO_8859_15
                .decode_without_bom_handling_and_without_replacement(self.buffer()?)
                .ok_or(TextError::DecodeFailure),
            TextEncoding::Reserved1(..) => Err(TextError::UnsupportedEncoding(enc)),
            TextEncoding::Reserved2(..) => Err(TextError::UnsupportedEncoding(enc)),
            TextEncoding::Iso10646 => Err(TextError::UnsupportedEncoding(enc)),
            TextEncoding::KSX1001_2004 => Err(TextError::UnsupportedEncoding(enc)),
            TextEncoding::GB2312_1980 => Err(TextError::UnsupportedEncoding(enc)),
            TextEncoding::Big5 => encoding_rs::BIG5
                .decode_without_bom_handling_and_without_replacement(self.buffer()?)
                .ok_or(TextError::DecodeFailure),
            TextEncoding::UTF8 => encoding_rs::UTF_8
                .decode_without_bom_handling_and_without_replacement(self.buffer()?)
                .ok_or(TextError::DecodeFailure),
        }
    }

    /// Returns the string with any un-decodable entries replaced with the *Unicode Replacement
    /// Character*
    pub fn to_string_with_replacement(&self) -> Result<Cow<'_, str>, TextError> {
        let enc = self.encoding();
        match enc {
            TextEncoding::Iso88591 => Ok(encoding_rs::mem::decode_latin1(self.buffer()?)),
            TextEncoding::Iso88592 => Ok(encoding_rs::ISO_8859_2
                .decode_without_bom_handling(self.buffer()?)
                .0),
            TextEncoding::Iso88593 => Ok(encoding_rs::ISO_8859_3
                .decode_without_bom_handling(self.buffer()?)
                .0),
            TextEncoding::Iso88594 => Ok(encoding_rs::ISO_8859_4
                .decode_without_bom_handling(self.buffer()?)
                .0),
            TextEncoding::Iso88595 => Ok(encoding_rs::ISO_8859_5
                .decode_without_bom_handling(self.buffer()?)
                .0),
            TextEncoding::Iso88596 => Ok(encoding_rs::ISO_8859_6
                .decode_without_bom_handling(self.buffer()?)
                .0),
            TextEncoding::Iso88597 => Ok(encoding_rs::ISO_8859_7
                .decode_without_bom_handling(self.buffer()?)
                .0),
            TextEncoding::Iso88598 => Ok(encoding_rs::ISO_8859_8
                .decode_without_bom_handling(self.buffer()?)
                .0),
            TextEncoding::Iso88599 => Err(TextError::UnsupportedEncoding(enc)),
            TextEncoding::Iso885910 => Ok(encoding_rs::ISO_8859_10
                .decode_without_bom_handling(self.buffer()?)
                .0),
            TextEncoding::Iso885911 => Err(TextError::UnsupportedEncoding(enc)),
            TextEncoding::Iso885913 => Ok(encoding_rs::ISO_8859_13
                .decode_without_bom_handling(self.buffer()?)
                .0),
            TextEncoding::Iso885914 => Ok(encoding_rs::ISO_8859_14
                .decode_without_bom_handling(self.buffer()?)
                .0),
            TextEncoding::Iso885915 => Ok(encoding_rs::ISO_8859_15
                .decode_without_bom_handling(self.buffer()?)
                .0),
            TextEncoding::Reserved1(..) => Err(TextError::UnsupportedEncoding(enc)),
            TextEncoding::Reserved2(..) => Err(TextError::UnsupportedEncoding(enc)),
            TextEncoding::Iso10646 => Err(TextError::UnsupportedEncoding(enc)),
            TextEncoding::KSX1001_2004 => Err(TextError::UnsupportedEncoding(enc)),
            TextEncoding::GB2312_1980 => Err(TextError::UnsupportedEncoding(enc)),
            TextEncoding::Big5 => Ok(encoding_rs::BIG5
                .decode_without_bom_handling(self.buffer()?)
                .0),
            TextEncoding::UTF8 => Ok(encoding_rs::UTF_8
                .decode_without_bom_handling(self.buffer()?)
                .0),
        }
    }
}
impl<'buf> fmt::Debug for Text<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(&self.to_string_with_replacement(), f)
    }
}
