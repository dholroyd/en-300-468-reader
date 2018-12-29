//! _Service Description Table_ section data
use crate::ActualOther;
use crate::Text;
use mpeg2ts_reader::{demultiplex, descriptor, packet, psi};
use std::fmt;
use std::marker;

#[derive(Debug)]
pub enum ServiceType {
    Reserved(u8),
    DigitalTelevision,
    DigitalRadioSound,
    Teletext,
    NvodReference,
    NvodTimeShifted,
    Mosaic,
    FmRadio,
    DvbSrm,
    AdvancedCodecDigitalRadioSound,
    H264AvcMosaic,
    DataBroadcast,
    RcsMap,
    RcsFls,
    DvbMhp,
    Mpeg2HdDigitalTelevision,
    H264AvcSdDigitalTelevision,
    H264AvcSdNvodTimeShifted,
    H264AvcSdNvodReference,
    H264AvcHdDigitalTelevision,
    H264AvcHdNvodTimeShifted,
    H264AvcHdNvodReference,
    H264AvcFrameCompatiblePlanoStereoscopicHdDigitalTelevision,
    H264AvcFrameCompatiblePlanoStereoscopicHdNvodTimeShifted,
    H264AvcFrameCompatiblePlanoStereoscopicHdNvodReference,
    HevcDigitalTelevision,
    UserDefined(u8),
}
impl ServiceType {
    fn from_id(id: u8) -> ServiceType {
        match id {
            0x00 => ServiceType::Reserved(id),
            0x01 => ServiceType::DigitalTelevision,
            0x02 => ServiceType::DigitalRadioSound,
            0x03 => ServiceType::Teletext,
            0x04 => ServiceType::NvodReference,
            0x05 => ServiceType::NvodTimeShifted,
            0x06 => ServiceType::Mosaic,
            0x07 => ServiceType::FmRadio,
            0x08 => ServiceType::DvbSrm,
            0x09 => ServiceType::Reserved(id),
            0x0a => ServiceType::AdvancedCodecDigitalRadioSound,
            0x0b => ServiceType::H264AvcMosaic,
            0x0c => ServiceType::DataBroadcast,
            0x0d => ServiceType::Reserved(id),
            0x0e => ServiceType::RcsMap,
            0x0f => ServiceType::RcsFls,
            0x10 => ServiceType::DvbMhp,
            0x11 => ServiceType::Mpeg2HdDigitalTelevision,
            0x12..=0x15 => ServiceType::Reserved(id),
            0x16 => ServiceType::H264AvcSdDigitalTelevision,
            0x17 => ServiceType::H264AvcSdNvodTimeShifted,
            0x18 => ServiceType::H264AvcSdNvodReference,
            0x19 => ServiceType::H264AvcHdDigitalTelevision,
            0x1a => ServiceType::H264AvcHdNvodTimeShifted,
            0x1b => ServiceType::H264AvcHdNvodReference,
            0x1c => ServiceType::H264AvcFrameCompatiblePlanoStereoscopicHdDigitalTelevision,
            0x1d => ServiceType::H264AvcFrameCompatiblePlanoStereoscopicHdNvodTimeShifted,
            0x1e => ServiceType::H264AvcFrameCompatiblePlanoStereoscopicHdNvodReference,
            0x1f => ServiceType::HevcDigitalTelevision,
            0x20..=0x7f => ServiceType::Reserved(id),
            0x80..=0xfe => ServiceType::UserDefined(id),
            0xff => ServiceType::Reserved(id),
            _ => unreachable!(),
        }
    }
}

pub struct ServiceDescriptor<'buf> {
    data: &'buf [u8],
}
impl<'buf> ServiceDescriptor<'buf> {
    pub const TAG: u8 = 0x48;

    pub fn new(
        tag: u8,
        data: &'buf [u8],
    ) -> Result<ServiceDescriptor<'buf>, descriptor::DescriptorError> {
        assert_eq!(tag, Self::TAG);
        Ok(ServiceDescriptor { data })
    }
    pub fn service_type(&self) -> ServiceType {
        ServiceType::from_id(self.data[0])
    }
    pub fn service_provider_name(&self) -> Result<Text<'buf>, super::TextError> {
        let service_provider_name_length = self.data[1] as usize;
        let end = 2 + service_provider_name_length;
        if end > self.data.len() {
            Err(super::TextError::NotEnoughData {
                expected: end,
                available: self.data.len(),
            })
        } else {
            Text::new(&self.data[2..end])
        }
    }
    pub fn service_name(&self) -> Result<Text<'buf>, super::TextError> {
        let service_provider_name_length = self.data[1] as usize;
        let start = 2 + service_provider_name_length;
        let service_name_length = self.data[start] as usize;
        let end = 1 + start + service_name_length;
        if end > self.data.len() {
            Err(super::TextError::NotEnoughData {
                expected: end,
                available: self.data.len(),
            })
        } else {
            Text::new(&self.data[1 + start..end])
        }
    }
}
impl<'buf> fmt::Debug for ServiceDescriptor<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("ServiceDescriptor")
            .field("service_type", &self.service_type())
            .field("service_provider_name", &self.service_provider_name())
            .field("service_name", &self.service_name())
            .finish()
    }
}

#[derive(Debug, PartialEq)]
pub enum RunningStatus {
    Undefined,
    NotRunning,
    StartsInAFewSeconds,
    Pausing,
    Running,
    ServiceOffAir,
    Reserved(u8),
}
impl RunningStatus {
    pub fn from_id(id: u8) -> RunningStatus {
        match id {
            0 => RunningStatus::Undefined,
            1 => RunningStatus::NotRunning,
            2 => RunningStatus::StartsInAFewSeconds,
            3 => RunningStatus::Pausing,
            4 => RunningStatus::Running,
            5 => RunningStatus::ServiceOffAir,
            6..=7 => RunningStatus::Reserved(id),
            _ => panic!(
                "Invalid running_status value {} (must be between 0 and 7)",
                id
            ),
        }
    }
}

pub struct Service<'buf> {
    data: &'buf [u8],
}
impl<'buf> Service<'buf> {
    fn new(data: &'buf [u8]) -> Service<'buf> {
        Service { data }
    }

    pub fn service_id(&self) -> u16 {
        u16::from(self.data[0]) << 8 | u16::from(self.data[1])
    }
    /// Event Information Table is present in the transport stream?
    pub fn eit_schedule_flag(&self) -> bool {
        self.data[2] & 0b10 != 0
    }
    /// Event Information Table present/following is present in the transport stream?
    pub fn eit_present_following_flag(&self) -> bool {
        self.data[2] & 0b1 != 0
    }
    pub fn running_status(&self) -> RunningStatus {
        RunningStatus::from_id(self.data[3] >> 5)
    }
    pub fn free_ca_mode(&self) -> bool {
        self.data[3] >> 4 & 0b1 != 0
    }
    fn descriptors_loop_length(&self) -> usize {
        usize::from(self.data[3] & 0b1111) << 8 | usize::from(self.data[4])
    }
    pub fn descriptors<Desc: descriptor::Descriptor<'buf>>(
        &self,
    ) -> descriptor::DescriptorIter<'buf, Desc> {
        let start = 5;
        let end = start + self.descriptors_loop_length();
        descriptor::DescriptorIter::new(&self.data[start..end])
    }
}
struct DescriptorsDebug<'buf, Desc: descriptor::Descriptor<'buf>>(
    &'buf Service<'buf>,
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
impl<'buf> fmt::Debug for Service<'buf> {
    fn fmt<'a>(&'a self, f: &'a mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("Service")
            .field("service_id", &self.service_id())
            .field("eit_schedule_flag", &self.eit_schedule_flag())
            .field(
                "eit_present_following_flag",
                &self.eit_present_following_flag(),
            )
            .field("running_status", &self.running_status())
            .field("free_ca_mode", &self.free_ca_mode())
            .field(
                "descriptors",
                &DescriptorsDebug::<'a, super::En300_468Descriptors<'a>>(self, marker::PhantomData),
            )
            .finish()
    }
}

struct ServiceIterator<'buf> {
    remaining_data: &'buf [u8],
}
impl<'buf> ServiceIterator<'buf> {
    pub fn new(data: &'buf [u8]) -> ServiceIterator<'buf> {
        ServiceIterator {
            remaining_data: data,
        }
    }
}
impl<'buf> Iterator for ServiceIterator<'buf> {
    type Item = Service<'buf>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_data.is_empty() {
            None
        } else {
            let descriptors_loop_length =
                u16::from(self.remaining_data[3] & 0b1111) << 8 | u16::from(self.remaining_data[4]);
            let size = 5 + descriptors_loop_length;
            let (head, tail) = self.remaining_data.split_at(size as usize);
            self.remaining_data = tail;
            let result = Some(Service::new(head));
            result
        }
    }
}
struct ServicesDebug<'buf>(&'buf SdtSection<'buf>);
impl<'buf> fmt::Debug for ServicesDebug<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_list().entries(self.0.services()).finish()
    }
}

pub struct SdtSection<'buf> {
    data: &'buf [u8],
}
impl<'buf> SdtSection<'buf> {
    pub fn new(data: &'buf [u8]) -> SdtSection<'buf> {
        assert!(data.len() > 3);
        SdtSection { data }
    }

    /// Borrow a reference to the underlying buffer holding SDT section data
    pub fn buffer(&self) -> &[u8] {
        self.data
    }

    pub fn original_network_id(&self) -> u16 {
        u16::from(self.data[0]) << 8 | u16::from(self.data[1])
    }
    pub fn services(&self) -> impl Iterator<Item = Service<'_>> {
        ServiceIterator::new(&self.data[3..])
    }
}
impl<'buf> fmt::Debug for SdtSection<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("SdtSection")
            .field("original_network_id", &self.original_network_id())
            .field("services", &ServicesDebug(self))
            .finish()
    }
}

pub struct SdtPacketFilter<Ctx: demultiplex::DemuxContext, C: SdtConsumer> {
    sdt_section_packet_consumer: psi::SectionPacketConsumer<
        psi::SectionSyntaxSectionProcessor<
            psi::DedupSectionSyntaxPayloadParser<
                psi::BufferSectionSyntaxParser<
                    psi::CrcCheckWholeSectionSyntaxPayloadParser<SdtProcessor<Ctx, C>>,
                >,
            >,
        >,
    >,
}
impl<Ctx: demultiplex::DemuxContext, C: SdtConsumer> SdtPacketFilter<Ctx, C> {
    pub fn new(consumer: C) -> SdtPacketFilter<Ctx, C> {
        let pat_proc = SdtProcessor::new(consumer);
        SdtPacketFilter {
            sdt_section_packet_consumer: psi::SectionPacketConsumer::new(
                psi::SectionSyntaxSectionProcessor::new(psi::DedupSectionSyntaxPayloadParser::new(
                    psi::BufferSectionSyntaxParser::new(
                        psi::CrcCheckWholeSectionSyntaxPayloadParser::new(pat_proc),
                    ),
                )),
            ),
        }
    }
}
impl<Ctx: demultiplex::DemuxContext, C: SdtConsumer> demultiplex::PacketFilter
    for SdtPacketFilter<Ctx, C>
{
    type Ctx = Ctx;

    fn consume(&mut self, ctx: &mut Self::Ctx, pk: &packet::Packet<'_>) {
        self.sdt_section_packet_consumer.consume(ctx, pk);
    }
}

pub trait SdtConsumer {
    fn consume(&mut self, sect: ActualOther<&SdtSection<'_>>);
}

pub struct SdtProcessor<Ctx: demultiplex::DemuxContext, C: SdtConsumer> {
    phantom: marker::PhantomData<Ctx>,
    consumer: C,
}

impl<Ctx: demultiplex::DemuxContext, C: SdtConsumer> SdtProcessor<Ctx, C> {
    pub fn new(consumer: C) -> SdtProcessor<Ctx, C> {
        SdtProcessor {
            consumer,
            phantom: marker::PhantomData,
        }
    }
}

impl<Ctx: demultiplex::DemuxContext, C: SdtConsumer> psi::WholeSectionSyntaxPayloadParser
    for SdtProcessor<Ctx, C>
{
    type Context = Ctx;

    fn section<'a>(
        &mut self,
        _ctx: &mut Self::Context,
        header: &psi::SectionCommonHeader,
        _table_syntax_header: &psi::TableSyntaxHeader<'_>,
        data: &'a [u8],
    ) {
        let start = psi::SectionCommonHeader::SIZE + psi::TableSyntaxHeader::SIZE;
        let end = data.len() - 4; // remove CRC bytes
        let sect = SdtSection::new(&data[start..end]);
        match header.table_id {
            0x42 => self.consumer.consume(ActualOther::Actual(&sect)),
            0x46 => self.consumer.consume(ActualOther::Other(&sect)),
            _ => log::warn!(
                "Expected SDT to have table id 0x42, but got {:#x} (original_network_id={})",
                header.table_id,
                sect.original_network_id()
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mpeg2ts_reader::psi::WholeSectionSyntaxPayloadParser;
    use mpeg2ts_reader::{packet, psi};

    mpeg2ts_reader::packet_filter_switch! {
        NullFilterSwitch<NullDemuxContext> {
            Pat: demultiplex::PatPacketFilter<NullDemuxContext>,
            Pmt: demultiplex::PmtPacketFilter<NullDemuxContext>,
            Nul: demultiplex::NullPacketFilter<NullDemuxContext>,
        }
    }
    mpeg2ts_reader::demux_context!(NullDemuxContext, NullStreamConstructor);
    pub struct NullStreamConstructor;
    impl demultiplex::StreamConstructor for NullStreamConstructor {
        type F = NullFilterSwitch;

        fn construct(&mut self, req: demultiplex::FilterRequest<'_, '_>) -> Self::F {
            match req {
                demultiplex::FilterRequest::ByPid(packet::Pid::PAT) => {
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

    struct AssertConsumer;
    impl SdtConsumer for AssertConsumer {
        fn consume(&mut self, sdt: ActualOther<&SdtSection<'_>>) {
            let sdt = sdt.actual().unwrap();
            assert_eq!(9018, sdt.original_network_id());
            let mut i = sdt.services();
            let a = i.next().unwrap();
            //assert_eq!(0x4440, a.service_id());
            assert!(a.eit_schedule_flag());
            assert!(a.eit_present_following_flag());
            assert_eq!(RunningStatus::Running, a.running_status());
            assert_eq!(25, sdt.services().count());
        }
    }

    #[test]
    fn it_works() {
        let mut ctx = NullDemuxContext::new(NullStreamConstructor);
        let mut processor = SdtProcessor::new(AssertConsumer);
        let section = vec![
            // common header
            0x42, 0x03, 0x6d, // table syntax header
            0x0D, 0x00, 0b00000001, 0xC1, 0x00,
            // Table data (originally from multiple TS packets)
            0x23, 0x3A, 0xFF, 0x10, 0x43, 0xFF, 0x80, 0x20, 0x48, 0x10, 0x01, 0x00, 0x0D, 0x42,
            0x42, 0x43, 0x20, 0x4F, 0x4E, 0x45, 0x20, 0x53, 0x6F, 0x75, 0x74, 0x68, 0x73, 0x0C,
            0x66, 0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x10, 0xBF,
            0xFF, 0x80, 0x1A, 0x48, 0x0A, 0x01, 0x00, 0x07, 0x42, 0x42, 0x43, 0x20, 0x54, 0x57,
            0x4F, 0x73, 0x0C, 0x66, 0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75,
            0x6B, 0x11, 0xC0, 0xFF, 0x80, 0x1B, 0x48, 0x0B, 0x01, 0x00, 0x08, 0x42, 0x42, 0x43,
            0x20, 0x46, 0x4F, 0x55, 0x52, 0x73, 0x0C, 0x66, 0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E,
            0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x12, 0x00, 0xFF, 0x80, 0x17, 0x48, 0x07, 0x01, 0x00,
            0x04, 0x43, 0x42, 0x42, 0x43, 0x73, 0x0C, 0x66, 0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E,
            0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x12, 0x40, 0xFF, 0x80, 0x1B, 0x48, 0x0B, 0x01, 0x00,
            0x08, 0x43, 0x42, 0x65, 0x65, 0x62, 0x69, 0x65, 0x73, 0x73, 0x0C, 0x66, 0x70, 0x2E,
            0x62, 0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x11, 0x00, 0xFF, 0x80, 0x1B,
            0x48, 0x0B, 0x01, 0x00, 0x08, 0x42, 0x42, 0x43, 0x20, 0x4E, 0x45, 0x57, 0x53, 0x73,
            0x0C, 0x66, 0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x12,
            0x80, 0xFF, 0x80, 0x21, 0x48, 0x11, 0x01, 0x00, 0x0E, 0x42, 0x42, 0x43, 0x20, 0x50,
            0x61, 0x72, 0x6C, 0x69, 0x61, 0x6D, 0x65, 0x6E, 0x74, 0x73, 0x0C, 0x66, 0x70, 0x2E,
            0x62, 0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x11, 0x40, 0xFD, 0x80, 0x21,
            0x48, 0x11, 0x01, 0x00, 0x0E, 0x42, 0x42, 0x43, 0x20, 0x52, 0x65, 0x64, 0x20, 0x42,
            0x75, 0x74, 0x74, 0x6F, 0x6E, 0x73, 0x0C, 0x66, 0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E,
            0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x1A, 0x40, 0xFF, 0x80, 0x1E, 0x48, 0x0E, 0x02, 0x00,
            0x0B, 0x42, 0x42, 0x43, 0x20, 0x52, 0x61, 0x64, 0x69, 0x6F, 0x20, 0x31, 0x73, 0x0C,
            0x66, 0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x17, 0x00,
            0xFF, 0x80, 0x1A, 0x48, 0x0A, 0x02, 0x00, 0x07, 0x42, 0x42, 0x43, 0x20, 0x52, 0x31,
            0x58, 0x73, 0x0C, 0x66, 0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75,
            0x6B, 0x1A, 0x80, 0xFF, 0x80, 0x1E, 0x48, 0x0E, 0x02, 0x00, 0x0B, 0x42, 0x42, 0x43,
            0x20, 0x52, 0x61, 0x64, 0x69, 0x6F, 0x20, 0x32, 0x73, 0x0C, 0x66, 0x70, 0x2E, 0x62,
            0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x1A, 0xC0, 0xFF, 0x80, 0x1E, 0x48,
            0x0E, 0x02, 0x00, 0x0B, 0x42, 0x42, 0x43, 0x20, 0x52, 0x61, 0x64, 0x69, 0x6F, 0x20,
            0x33, 0x73, 0x0C, 0x66, 0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75,
            0x6B, 0x1B, 0x00, 0xFF, 0x80, 0x1E, 0x48, 0x0E, 0x02, 0x00, 0x0B, 0x42, 0x42, 0x43,
            0x20, 0x52, 0x61, 0x64, 0x69, 0x6F, 0x20, 0x34, 0x73, 0x0C, 0x66, 0x70, 0x2E, 0x62,
            0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x16, 0x00, 0xFF, 0x80, 0x1A, 0x48,
            0x0A, 0x02, 0x00, 0x07, 0x42, 0x42, 0x43, 0x20, 0x52, 0x35, 0x4C, 0x73, 0x0C, 0x66,
            0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x16, 0x40, 0xFF,
            0x80, 0x1B, 0x48, 0x0B, 0x02, 0x00, 0x08, 0x42, 0x42, 0x43, 0x20, 0x52, 0x35, 0x53,
            0x58, 0x73, 0x0C, 0x66, 0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75,
            0x6B, 0x16, 0x80, 0xFF, 0x80, 0x1E, 0x48, 0x0E, 0x02, 0x00, 0x0B, 0x42, 0x42, 0x43,
            0x20, 0x36, 0x20, 0x4D, 0x75, 0x73, 0x69, 0x63, 0x73, 0x0C, 0x66, 0x70, 0x2E, 0x62,
            0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x16, 0xC0, 0xFF, 0x80, 0x21, 0x48,
            0x11, 0x02, 0x00, 0x0E, 0x42, 0x42, 0x43, 0x20, 0x52, 0x61, 0x64, 0x69, 0x6F, 0x20,
            0x34, 0x20, 0x45, 0x78, 0x73, 0x0C, 0x66, 0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E, 0x63,
            0x6F, 0x2E, 0x75, 0x6B, 0x17, 0x40, 0xFF, 0x80, 0x21, 0x48, 0x11, 0x02, 0x00, 0x0E,
            0x42, 0x42, 0x43, 0x20, 0x41, 0x73, 0x69, 0x61, 0x6E, 0x20, 0x4E, 0x65, 0x74, 0x2E,
            0x73, 0x0C, 0x66, 0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75, 0x6B,
            0x17, 0x80, 0xFF, 0x80, 0x20, 0x48, 0x10, 0x02, 0x00, 0x0D, 0x42, 0x42, 0x43, 0x20,
            0x57, 0x6F, 0x72, 0x6C, 0x64, 0x20, 0x53, 0x76, 0x2E, 0x73, 0x0C, 0x66, 0x70, 0x2E,
            0x62, 0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x1C, 0x00, 0xFF, 0x80, 0x1B,
            0x48, 0x0B, 0x01, 0x00, 0x08, 0x42, 0x42, 0x43, 0x20, 0x52, 0x42, 0x20, 0x31, 0x73,
            0x0C, 0x66, 0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x18,
            0x03, 0xFF, 0x80, 0x1D, 0x48, 0x0D, 0x02, 0x00, 0x0A, 0x42, 0x42, 0x43, 0x20, 0x53,
            0x6F, 0x6C, 0x65, 0x6E, 0x74, 0x73, 0x0C, 0x66, 0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E,
            0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x18, 0x4C, 0xFF, 0x80, 0x1D, 0x48, 0x0D, 0x02, 0x00,
            0x0A, 0x42, 0x42, 0x43, 0x20, 0x53, 0x75, 0x73, 0x73, 0x65, 0x78, 0x73, 0x0C, 0x66,
            0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x18, 0x83, 0xFF,
            0x80, 0x20, 0x48, 0x10, 0x02, 0x00, 0x0D, 0x42, 0x42, 0x43, 0x20, 0x42, 0x65, 0x72,
            0x6B, 0x73, 0x68, 0x69, 0x72, 0x65, 0x73, 0x0C, 0x66, 0x70, 0x2E, 0x62, 0x62, 0x63,
            0x2E, 0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x18, 0x43, 0xFF, 0x80, 0x24, 0x48, 0x14, 0x02,
            0x00, 0x11, 0x42, 0x42, 0x43, 0x20, 0x53, 0x6F, 0x6C, 0x65, 0x6E, 0x74, 0x20, 0x44,
            0x6F, 0x72, 0x73, 0x65, 0x74, 0x73, 0x0C, 0x66, 0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E,
            0x63, 0x6F, 0x2E, 0x75, 0x6B, 0x18, 0x81, 0xFF, 0x80, 0x20, 0x48, 0x10, 0x02, 0x00,
            0x0D, 0x42, 0x42, 0x43, 0x20, 0x57, 0x69, 0x6C, 0x74, 0x73, 0x68, 0x69, 0x72, 0x65,
            0x73, 0x0C, 0x66, 0x70, 0x2E, 0x62, 0x62, 0x63, 0x2E, 0x63, 0x6F, 0x2E, 0x75, 0x6B,
            0x65, 0x34, 0x57, 0x55, // CRC
        ];

        let header = psi::SectionCommonHeader::new(&section[..psi::SectionCommonHeader::SIZE]);
        let table_syntax_header =
            psi::TableSyntaxHeader::new(&section[psi::SectionCommonHeader::SIZE..]);
        processor.section(&mut ctx, &header, &table_syntax_header, &section[..]);
    }
}
