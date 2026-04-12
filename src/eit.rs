//! _Event Information Table_ section data
use crate::sdt::RunningStatus;
use crate::ActualOther;
use mpeg2ts_reader::{demultiplex, descriptor, packet, psi};
use std::fmt;
use std::marker;

pub const EIT_PID: packet::Pid = packet::Pid::new(0x12);

/// A BCD-encoded duration (3 bytes: HH:MM:SS), as used in EIT event entries.
#[derive(Clone, Copy)]
pub struct BcdDuration {
    raw: u32,
}
impl BcdDuration {
    fn new(raw: u32) -> Self {
        BcdDuration { raw }
    }

    /// The raw 24-bit BCD value.
    pub fn raw(&self) -> u32 {
        self.raw
    }

    pub fn hours(&self) -> u8 {
        let tens = ((self.raw >> 20) & 0xf) as u8;
        let units = ((self.raw >> 16) & 0xf) as u8;
        tens * 10 + units
    }

    pub fn minutes(&self) -> u8 {
        let tens = ((self.raw >> 12) & 0xf) as u8;
        let units = ((self.raw >> 8) & 0xf) as u8;
        tens * 10 + units
    }

    pub fn seconds(&self) -> u8 {
        let tens = ((self.raw >> 4) & 0xf) as u8;
        let units = (self.raw & 0xf) as u8;
        tens * 10 + units
    }

    /// Total duration in seconds.
    pub fn as_seconds(&self) -> u32 {
        u32::from(self.hours()) * 3600 + u32::from(self.minutes()) * 60 + u32::from(self.seconds())
    }

    pub fn as_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.as_seconds().into())
    }
}
impl fmt::Debug for BcdDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{:02}:{:02}:{:02}",
            self.hours(),
            self.minutes(),
            self.seconds()
        )
    }
}
impl fmt::Display for BcdDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{:02}:{:02}:{:02}",
            self.hours(),
            self.minutes(),
            self.seconds()
        )
    }
}

/// Error returned when an [`MjdTimestamp`] contains invalid data.
#[derive(Debug)]
pub enum MjdTimestampError {
    /// A BCD nibble had a value greater than 9.
    InvalidBcd { byte_offset: usize, value: u8 },
    /// Hours value was 24 or greater.
    HoursOutOfRange(u8),
    /// Minutes value was 60 or greater.
    MinutesOutOfRange(u8),
    /// Seconds value was 60 or greater.
    SecondsOutOfRange(u8),
}
impl fmt::Display for MjdTimestampError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MjdTimestampError::InvalidBcd { byte_offset, value } => {
                write!(
                    f,
                    "invalid BCD byte at offset {}: {:#04x}",
                    byte_offset, value
                )
            }
            MjdTimestampError::HoursOutOfRange(h) => write!(f, "hours out of range: {}", h),
            MjdTimestampError::MinutesOutOfRange(m) => write!(f, "minutes out of range: {}", m),
            MjdTimestampError::SecondsOutOfRange(s) => write!(f, "seconds out of range: {}", s),
        }
    }
}

/// A DVB timestamp encoded as a 5-byte Modified Julian Date plus BCD-encoded UTC time,
/// as defined in EN 300 468 Annex C.
pub struct MjdTimestamp<'buf> {
    data: &'buf [u8],
}

fn is_valid_bcd(byte: u8) -> bool {
    (byte >> 4) <= 9 && (byte & 0x0f) <= 9
}

impl<'buf> MjdTimestamp<'buf> {
    fn new(data: &'buf [u8]) -> Result<MjdTimestamp<'buf>, MjdTimestampError> {
        assert_eq!(data.len(), 5);
        for i in 0..3 {
            if !is_valid_bcd(data[2 + i]) {
                return Err(MjdTimestampError::InvalidBcd {
                    byte_offset: 2 + i,
                    value: data[2 + i],
                });
            }
        }
        let hours = (data[2] >> 4) * 10 + (data[2] & 0x0f);
        let minutes = (data[3] >> 4) * 10 + (data[3] & 0x0f);
        let seconds = (data[4] >> 4) * 10 + (data[4] & 0x0f);
        if hours >= 24 {
            return Err(MjdTimestampError::HoursOutOfRange(hours));
        }
        if minutes >= 60 {
            return Err(MjdTimestampError::MinutesOutOfRange(minutes));
        }
        if seconds >= 60 {
            return Err(MjdTimestampError::SecondsOutOfRange(seconds));
        }
        Ok(MjdTimestamp { data })
    }

    fn mjd(&self) -> u16 {
        u16::from(self.data[0]) << 8 | u16::from(self.data[1])
    }

    /// Calendar date (year, month, day) derived from the MJD value using the
    /// EN 300 468 Annex C algorithm.
    pub fn date(&self) -> (i32, i32, i32) {
        let mjd = i64::from(self.mjd());
        let y_prime = ((mjd as f64 - 15078.2) / 365.25) as i64;
        let m_prime = ((mjd as f64 - 14956.1 - (y_prime as f64 * 365.25).floor()) / 30.6001) as i64;
        let d = mjd - 14956 - (y_prime as f64 * 365.25) as i64 - (m_prime as f64 * 30.6001) as i64;
        let k = if m_prime == 14 || m_prime == 15 { 1 } else { 0 };
        let y = y_prime + k + 1900;
        let m = m_prime - 1 - k * 12;
        (y as i32, m as i32, d as i32)
    }

    /// Hours (BCD-decoded)
    pub fn hours(&self) -> u8 {
        (self.data[2] >> 4) * 10 + (self.data[2] & 0x0f)
    }

    /// Minutes (BCD-decoded)
    pub fn minutes(&self) -> u8 {
        (self.data[3] >> 4) * 10 + (self.data[3] & 0x0f)
    }

    /// Seconds (BCD-decoded)
    pub fn seconds(&self) -> u8 {
        (self.data[4] >> 4) * 10 + (self.data[4] & 0x0f)
    }

    /// Convert to Unix timestamp (seconds since 1970-01-01 00:00:00 UTC).
    ///
    /// Uses the MJD-to-calendar-date algorithm from EN 300 468 Annex C.
    pub fn to_unix_timestamp(&self) -> i64 {
        // MJD of Unix epoch (1970-01-01) is 40587
        let days_since_epoch = i64::from(self.mjd()) - 40587;
        days_since_epoch * 86400
            + i64::from(self.hours()) * 3600
            + i64::from(self.minutes()) * 60
            + i64::from(self.seconds())
    }
}
impl fmt::Debug for MjdTimestamp<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let (y, m, d) = self.date();
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            y,
            m,
            d,
            self.hours(),
            self.minutes(),
            self.seconds(),
        )
    }
}

pub struct Event<'buf> {
    data: &'buf [u8],
}
impl<'buf> Event<'buf> {
    fn new(data: &'buf [u8]) -> Event<'buf> {
        Event { data }
    }

    pub fn event_id(&self) -> u16 {
        u16::from(self.data[0]) << 8 | u16::from(self.data[1])
    }

    pub fn start_time(&self) -> Result<MjdTimestamp<'buf>, MjdTimestampError> {
        MjdTimestamp::new(&self.data[2..7])
    }

    /// Duration as a BCD-encoded value (hours, minutes, seconds).
    pub fn duration(&self) -> BcdDuration {
        BcdDuration::new(
            u32::from(self.data[7]) << 16 | u32::from(self.data[8]) << 8 | u32::from(self.data[9]),
        )
    }

    pub fn running_status(&self) -> RunningStatus {
        RunningStatus::from_id(self.data[10] >> 5)
    }

    pub fn free_ca_mode(&self) -> bool {
        self.data[10] >> 4 & 0b1 != 0
    }

    fn descriptors_loop_length(&self) -> usize {
        usize::from(self.data[10] & 0b1111) << 8 | usize::from(self.data[11])
    }

    pub fn descriptors<Desc: descriptor::Descriptor<'buf>>(
        &self,
    ) -> descriptor::DescriptorIter<'buf, Desc> {
        let start = 12;
        let end = start + self.descriptors_loop_length();
        descriptor::DescriptorIter::new(&self.data[start..end])
    }
}
struct DescriptorsDebug<'buf, Desc: descriptor::Descriptor<'buf>>(
    &'buf Event<'buf>,
    marker::PhantomData<Desc>,
);
impl<'buf, Desc: descriptor::Descriptor<'buf> + fmt::Debug> fmt::Debug
    for DescriptorsDebug<'buf, Desc>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_list()
            .entries(self.0.descriptors::<Desc>())
            .finish()
    }
}
impl<'buf> fmt::Debug for Event<'buf> {
    fn fmt<'a>(&'a self, f: &'a mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("Event")
            .field("event_id", &self.event_id())
            .field(
                "start_time",
                &self.start_time().as_ref().map_err(|e| format!("{}", e)),
            )
            .field("duration", &self.duration())
            .field("running_status", &self.running_status())
            .field("free_ca_mode", &self.free_ca_mode())
            .field(
                "descriptors",
                &DescriptorsDebug::<'a, super::En300_468Descriptors<'a>>(self, marker::PhantomData),
            )
            .finish()
    }
}

/// Error returned when event data is truncated.
#[derive(Debug)]
pub struct EventDataTruncated {
    pub expected: usize,
    pub available: usize,
}
impl fmt::Display for EventDataTruncated {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EIT event data truncated: need {} bytes, have {}",
            self.expected, self.available
        )
    }
}

struct EventIterator<'buf> {
    remaining_data: &'buf [u8],
}
impl<'buf> EventIterator<'buf> {
    pub fn new(data: &'buf [u8]) -> EventIterator<'buf> {
        EventIterator {
            remaining_data: data,
        }
    }
}
impl<'buf> Iterator for EventIterator<'buf> {
    type Item = Result<Event<'buf>, EventDataTruncated>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_data.is_empty() {
            None
        } else if self.remaining_data.len() < 12 {
            let err = EventDataTruncated {
                expected: 12,
                available: self.remaining_data.len(),
            };
            self.remaining_data = &[];
            Some(Err(err))
        } else {
            let descriptors_loop_length = usize::from(self.remaining_data[10] & 0b1111) << 8
                | usize::from(self.remaining_data[11]);
            let size = 12 + descriptors_loop_length;
            if size > self.remaining_data.len() {
                let err = EventDataTruncated {
                    expected: size,
                    available: self.remaining_data.len(),
                };
                self.remaining_data = &[];
                return Some(Err(err));
            }
            let (head, tail) = self.remaining_data.split_at(size);
            self.remaining_data = tail;
            Some(Ok(Event::new(head)))
        }
    }
}
struct EventsDebug<'buf>(&'buf EitSection<'buf>);
impl<'buf> fmt::Debug for EventsDebug<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_list().entries(self.0.events()).finish()
    }
}

pub struct EitSection<'buf> {
    data: &'buf [u8],
}
impl<'buf> EitSection<'buf> {
    pub fn new(data: &'buf [u8]) -> EitSection<'buf> {
        assert!(data.len() >= 6);
        EitSection { data }
    }

    /// Borrow a reference to the underlying buffer holding EIT section data
    pub fn buffer(&self) -> &[u8] {
        self.data
    }

    pub fn transport_stream_id(&self) -> u16 {
        u16::from(self.data[0]) << 8 | u16::from(self.data[1])
    }

    pub fn original_network_id(&self) -> u16 {
        u16::from(self.data[2]) << 8 | u16::from(self.data[3])
    }

    pub fn segment_last_section_number(&self) -> u8 {
        self.data[4]
    }

    pub fn last_table_id(&self) -> u8 {
        self.data[5]
    }

    pub fn events(&self) -> impl Iterator<Item = Result<Event<'_>, EventDataTruncated>> {
        EventIterator::new(&self.data[6..])
    }
}
impl<'buf> fmt::Debug for EitSection<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("EitSection")
            .field("transport_stream_id", &self.transport_stream_id())
            .field("original_network_id", &self.original_network_id())
            .field(
                "segment_last_section_number",
                &self.segment_last_section_number(),
            )
            .field("last_table_id", &self.last_table_id())
            .field("events", &EventsDebug(self))
            .finish()
    }
}

type EitSectionPacketConsumer<Ctx, C> = psi::SectionSyntaxFramer<
    psi::DedupSectionSyntaxPayloadParser<
        psi::CrcCheckWholeSectionSyntaxPayloadParser<EitProcessor<Ctx, C>>,
    >,
>;

pub struct EitPacketFilter<Ctx: demultiplex::DemuxContext, C: EitConsumer> {
    eit_section_packet_consumer: EitSectionPacketConsumer<Ctx, C>,
}
impl<Ctx: demultiplex::DemuxContext, C: EitConsumer> EitPacketFilter<Ctx, C> {
    pub fn new(pid: packet::Pid, consumer: C) -> EitPacketFilter<Ctx, C> {
        let proc = EitProcessor::new(consumer);
        EitPacketFilter {
            eit_section_packet_consumer: psi::SectionSyntaxFramer::new(
                pid,
                psi::DedupSectionSyntaxPayloadParser::new(
                    psi::CrcCheckWholeSectionSyntaxPayloadParser::new(pid, proc),
                ),
            ),
        }
    }
}
impl<Ctx: demultiplex::DemuxContext, C: EitConsumer> demultiplex::PacketFilter
    for EitPacketFilter<Ctx, C>
{
    type Ctx = Ctx;

    fn consume(&mut self, ctx: &mut Self::Ctx, pk: &packet::Packet<'_>) {
        self.eit_section_packet_consumer.consume(ctx, pk);
    }
}

pub trait EitConsumer {
    fn present_following(
        &mut self,
        _service_id: u16,
        _section_number: u8,
        _sect: ActualOther<&EitSection<'_>>,
    ) {
    }
    fn schedule(
        &mut self,
        _service_id: u16,
        _section_number: u8,
        _sect: ActualOther<&EitSection<'_>>,
    ) {
    }
}

pub struct EitProcessor<Ctx: demultiplex::DemuxContext, C: EitConsumer> {
    phantom: marker::PhantomData<Ctx>,
    consumer: C,
}

impl<Ctx: demultiplex::DemuxContext, C: EitConsumer> EitProcessor<Ctx, C> {
    pub fn new(consumer: C) -> EitProcessor<Ctx, C> {
        EitProcessor {
            consumer,
            phantom: marker::PhantomData,
        }
    }
}

impl<Ctx: demultiplex::DemuxContext, C: EitConsumer> psi::WholeSectionSyntaxPayloadParser
    for EitProcessor<Ctx, C>
{
    type Context = Ctx;

    fn section(
        &mut self,
        _ctx: &mut Self::Context,
        header: &psi::SectionCommonHeader,
        table_syntax_header: &psi::TableSyntaxHeader<'_>,
        data: &[u8],
    ) {
        let service_id = table_syntax_header.id();
        let section_number = table_syntax_header.section_number();
        let start = psi::SectionCommonHeader::SIZE + psi::TableSyntaxHeader::SIZE;
        if data.len() < start + 4 {
            log::warn!(
                "EIT section too short: {} bytes (need at least {})",
                data.len(),
                start + 4,
            );
            return;
        }
        let end = data.len() - 4; // remove CRC bytes
        let sect = EitSection::new(&data[start..end]);
        match header.table_id {
            0x4E => self.consumer.present_following(
                service_id,
                section_number,
                ActualOther::Actual(&sect),
            ),
            0x4F => self.consumer.present_following(
                service_id,
                section_number,
                ActualOther::Other(&sect),
            ),
            0x50..=0x5F => {
                self.consumer
                    .schedule(service_id, section_number, ActualOther::Actual(&sect))
            }
            0x60..=0x6F => {
                self.consumer
                    .schedule(service_id, section_number, ActualOther::Other(&sect))
            }
            _ => log::warn!(
                "Expected EIT table id 0x4E-0x6F, but got {:#x}",
                header.table_id,
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mpeg2ts_reader::psi;
    use mpeg2ts_reader::psi::WholeSectionSyntaxPayloadParser;

    mpeg2ts_reader::packet_filter_switch! {
        NullFilterSwitch<NullDemuxContext> {
            Pat: demultiplex::PatPacketFilter<NullDemuxContext>,
            Pmt: demultiplex::PmtPacketFilter<NullDemuxContext>,
            Nul: demultiplex::NullPacketFilter<NullDemuxContext>,
        }
    }
    mpeg2ts_reader::demux_context!(NullDemuxContext, NullFilterSwitch);
    impl NullDemuxContext {
        fn do_construct(&mut self, req: demultiplex::FilterRequest<'_, '_>) -> NullFilterSwitch {
            match req {
                demultiplex::FilterRequest::ByPid(psi::pat::PAT_PID) => {
                    NullFilterSwitch::Pat(demultiplex::PatPacketFilter::default())
                }
                demultiplex::FilterRequest::ByPid(_) => {
                    NullFilterSwitch::Nul(demultiplex::NullPacketFilter::default())
                }
                demultiplex::FilterRequest::ByStream {
                    program_pid: _,
                    stream_type: _,
                    pmt: _,
                    stream_info: _,
                } => NullFilterSwitch::Nul(demultiplex::NullPacketFilter::default()),
                demultiplex::FilterRequest::Pmt {
                    pid,
                    program_number,
                } => NullFilterSwitch::Pmt(demultiplex::PmtPacketFilter::new(pid, program_number)),
                demultiplex::FilterRequest::Nit { pid: _ } => {
                    NullFilterSwitch::Nul(demultiplex::NullPacketFilter::default())
                }
            }
        }
    }

    #[test]
    fn mjd_timestamp_fields() {
        // MJD 0xC079 = 49273 = 1993-10-13, time 12:30:45 BCD
        let data = [0xC0, 0x79, 0x12, 0x30, 0x45];
        let ts = MjdTimestamp::new(&data).unwrap();
        assert_eq!((1993, 10, 13), ts.date());
        assert_eq!(12, ts.hours());
        assert_eq!(30, ts.minutes());
        assert_eq!(45, ts.seconds());
    }

    #[test]
    fn mjd_timestamp_to_unix() {
        // 1993-10-13 12:30:45 UTC = Unix timestamp 750515445
        let data = [0xC0, 0x79, 0x12, 0x30, 0x45];
        let ts = MjdTimestamp::new(&data).unwrap();
        assert_eq!(750515445, ts.to_unix_timestamp());
    }

    #[test]
    fn mjd_timestamp_unix_epoch() {
        // MJD 40587 = 1970-01-01, time 00:00:00
        let mjd: u16 = 40587;
        let data = [(mjd >> 8) as u8, mjd as u8, 0x00, 0x00, 0x00];
        let ts = MjdTimestamp::new(&data).unwrap();
        assert_eq!(0, ts.to_unix_timestamp());
    }

    #[test]
    fn mjd_timestamp_debug() {
        let data = [0xC0, 0x79, 0x12, 0x30, 0x45];
        let ts = MjdTimestamp::new(&data).unwrap();
        assert_eq!("1993-10-13 12:30:45", format!("{:?}", ts));
    }

    #[test]
    fn mjd_timestamp_invalid_bcd() {
        // 0xAB has nibble A (10) which is invalid BCD
        let data = [0xC0, 0x79, 0xAB, 0x30, 0x45];
        assert!(matches!(
            MjdTimestamp::new(&data),
            Err(MjdTimestampError::InvalidBcd {
                byte_offset: 2,
                value: 0xAB
            })
        ));
    }

    #[test]
    fn mjd_timestamp_hours_out_of_range() {
        // 24 hours = 0x24 in BCD
        let data = [0xC0, 0x79, 0x24, 0x30, 0x45];
        assert!(matches!(
            MjdTimestamp::new(&data),
            Err(MjdTimestampError::HoursOutOfRange(24))
        ));
    }

    #[test]
    fn mjd_timestamp_minutes_out_of_range() {
        // 60 minutes = 0x60 in BCD
        let data = [0xC0, 0x79, 0x12, 0x60, 0x45];
        assert!(matches!(
            MjdTimestamp::new(&data),
            Err(MjdTimestampError::MinutesOutOfRange(60))
        ));
    }

    #[test]
    fn mjd_timestamp_seconds_out_of_range() {
        // 60 seconds = 0x60 in BCD
        let data = [0xC0, 0x79, 0x12, 0x30, 0x60];
        assert!(matches!(
            MjdTimestamp::new(&data),
            Err(MjdTimestampError::SecondsOutOfRange(60))
        ));
    }

    struct AssertConsumer;
    impl EitConsumer for AssertConsumer {
        fn present_following(
            &mut self,
            service_id: u16,
            section_number: u8,
            eit: ActualOther<&EitSection<'_>>,
        ) {
            assert_eq!(0x0440, service_id);
            assert_eq!(0, section_number);
            let eit = eit.actual().unwrap();
            assert_eq!(9018, eit.original_network_id());
            assert_eq!(0x01, eit.segment_last_section_number());
            assert_eq!(0x4E, eit.last_table_id());
            assert_eq!(1, eit.events().count());
            let event = eit.events().next().unwrap().unwrap();
            assert_eq!(0x1234, event.event_id());
            let st = event.start_time().unwrap();
            assert_eq!((1993, 10, 13), st.date());
            assert_eq!(12, st.hours());
            assert_eq!(30, st.minutes());
            assert_eq!(0, st.seconds());
            let dur = event.duration();
            assert_eq!(1, dur.hours());
            assert_eq!(30, dur.minutes());
            assert_eq!(0, dur.seconds());
            assert_eq!(5400, dur.as_seconds());
            assert_eq!(RunningStatus::Running, event.running_status());
            assert!(!event.free_ca_mode());
        }
    }

    #[test]
    fn eit_present_following() {
        let mut ctx = NullDemuxContext::new();
        let mut processor = EitProcessor::new(AssertConsumer);

        // Build an EIT p/f actual section (table_id 0x4E)
        let mut section = vec![
            // common header: table_id=0x4E, section_syntax_indicator=1
            0x4E, 0x80, 0x00,
            // table syntax header: service_id=0x0440, version=0, current=1,
            // section_number=0, last_section_number=1
            0x04, 0x40, 0b00000001, 0x00, 0x01,
            // EIT section data:
            // transport_stream_id=0x0001
            0x00, 0x01, // original_network_id=9018 (0x233A)
            0x23, 0x3A, // segment_last_section_number=1
            0x01, // last_table_id=0x4E
            0x4E, // Event: event_id=0x1234
            0x12, 0x34, // start_time: MJD=0xC079, UTC=12:30:00 BCD
            0xC0, 0x79, 0x12, 0x30, 0x00, // duration: 01:30:00 BCD
            0x01, 0x30, 0x00,
            // running_status=4 (running), free_ca_mode=0, descriptors_loop_length=0
            0x80, 0x00,
        ];

        // Set section_length
        let section_length = section.len() - 3 + 4;
        section[1] = 0x80 | ((section_length >> 8) as u8 & 0x0F);
        section[2] = section_length as u8;

        // Append CRC32
        let crc = mpeg2ts_reader::mpegts_crc::sum32(&section);
        section.extend_from_slice(&crc.to_be_bytes());

        let header = psi::SectionCommonHeader::new(&section[..psi::SectionCommonHeader::SIZE]);
        let table_syntax_header =
            psi::TableSyntaxHeader::new(&section[psi::SectionCommonHeader::SIZE..]);
        processor.section(&mut ctx, &header, &table_syntax_header, &section[..]);
    }
}
