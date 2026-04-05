//! _Short Event Descriptor_ (tag 0x4D)
use crate::Text;
use mpeg2ts_reader::descriptor;
use std::fmt;

/// Provides the name and a short description of an event, as defined in
/// _ETSI EN 300 468_ table 57.
pub struct ShortEventDescriptor<'buf> {
    data: &'buf [u8],
}
impl<'buf> ShortEventDescriptor<'buf> {
    pub const TAG: u8 = 0x4D;

    pub fn new(
        tag: u8,
        data: &'buf [u8],
    ) -> Result<ShortEventDescriptor<'buf>, descriptor::DescriptorError> {
        assert_eq!(tag, Self::TAG);
        Ok(ShortEventDescriptor { data })
    }

    /// Three-character ISO 639-2 language code
    pub fn language_code(&self) -> &'buf [u8] {
        &self.data[0..3]
    }

    /// Three-character ISO 639-2 language code as a string, or `None` if the bytes are not valid
    /// UTF-8
    pub fn language_code_str(&self) -> Option<&'buf str> {
        std::str::from_utf8(self.language_code()).ok()
    }

    /// The name of the event
    pub fn event_name(&self) -> Result<Text<'buf>, super::TextError> {
        let (text, _) = Text::read(&self.data[3..])?;
        Ok(text)
    }

    /// A short description of the event
    pub fn text(&self) -> Result<Text<'buf>, super::TextError> {
        let (_, consumed) = Text::read(&self.data[3..])?;
        let (text, _) = Text::read(&self.data[3 + consumed..])?;
        Ok(text)
    }
}
impl<'buf> fmt::Debug for ShortEventDescriptor<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("ShortEventDescriptor")
            .field("language_code", &self.language_code_str())
            .field("event_name", &self.event_name())
            .field("text", &self.text())
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn short_event_descriptor() {
        // language_code = "eng"
        // event_name = "News at Ten" (ISO 8859-1, no prefix → 0x20..=0xff first byte)
        // text = "The latest headlines"
        #[rustfmt::skip]
        let data: Vec<u8> = vec![
            b'e', b'n', b'g', // ISO 639 language code
            0x0B, // event_name_length = 11
            b'N', b'e', b'w', b's', b' ', b'a', b't', b' ', b'T', b'e', b'n',
            0x14, // text_length = 20
            b'T', b'h', b'e', b' ', b'l', b'a', b't', b'e', b's', b't', b' ', b'h', b'e', b'a',
            b'd', b'l', b'i', b'n', b'e', b's',
        ];

        let desc = ShortEventDescriptor::new(0x4D, &data).unwrap();

        assert_eq!(desc.language_code(), b"eng");
        assert_eq!(desc.language_code_str(), Some("eng"));

        let name = desc.event_name().unwrap();
        assert_eq!(name.to_string().unwrap().as_ref(), "News at Ten");

        let text = desc.text().unwrap();
        assert_eq!(text.to_string().unwrap().as_ref(), "The latest headlines");
    }

    #[test]
    fn empty_text_field() {
        #[rustfmt::skip]
        let data: Vec<u8> = vec![
            b'f', b'r', b'a', // language code
            0x05, // event_name_length = 5
            b'T', b'i', b't', b'r', b'e',
            0x00, // text_length = 0
        ];

        let desc = ShortEventDescriptor::new(0x4D, &data).unwrap();
        assert_eq!(desc.language_code_str(), Some("fra"));
        assert!(desc.text().is_err());
    }
}
