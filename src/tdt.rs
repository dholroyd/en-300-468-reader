//! _Time and Date Table_ section data (EN 300 468 §5.2.5).
//!
//! TDT carries UTC time for the multiplex. It uses compact syntax
//! (no section syntax indicator, no CRC) on PID 0x14 with table_id 0x70.

use crate::time::{MjdTimestamp, MjdTimestampError};
use mpeg2ts_reader::{demultiplex, packet, psi};
use std::marker;

pub const TDT_PID: packet::Pid = packet::Pid::new(0x14);

const TDT_TABLE_ID: u8 = 0x70;

/// A parsed TDT section: just a 5-byte MJD+BCD UTC timestamp.
pub struct TdtSection<'buf> {
    data: &'buf [u8],
}
impl<'buf> TdtSection<'buf> {
    /// Parse the 5-byte timestamp from the section body.
    pub fn utc_time(&self) -> Result<MjdTimestamp<'buf>, MjdTimestampError> {
        MjdTimestamp::new(self.data)
    }
}

/// Consumer trait for receiving parsed TDT sections.
pub trait TdtConsumer<Ctx> {
    fn tdt(&mut self, ctx: &mut Ctx, section: &TdtSection<'_>);
}

/// Compact-syntax parser that validates the table_id and delivers
/// [`TdtSection`] values to the consumer.
pub struct TdtProcessor<Ctx: demultiplex::DemuxContext, C: TdtConsumer<Ctx>> {
    phantom: marker::PhantomData<Ctx>,
    consumer: C,
}

impl<Ctx: demultiplex::DemuxContext, C: TdtConsumer<Ctx>> TdtProcessor<Ctx, C> {
    pub fn new(consumer: C) -> Self {
        TdtProcessor {
            consumer,
            phantom: marker::PhantomData,
        }
    }
}

impl<Ctx: demultiplex::DemuxContext, C: TdtConsumer<Ctx>> psi::WholeCompactSyntaxPayloadParser
    for TdtProcessor<Ctx, C>
{
    type Context = Ctx;

    fn section(&mut self, ctx: &mut Self::Context, header: &psi::SectionCommonHeader, data: &[u8]) {
        if header.table_id != TDT_TABLE_ID {
            // PID 0x14 also carries TOT (0x73) - ignore it.
            return;
        }
        let body = &data[psi::SectionCommonHeader::SIZE..];
        if body.len() < 5 {
            log::warn!("TDT section too short: {} bytes (need 5)", body.len());
            return;
        }
        let section = TdtSection { data: &body[..5] };
        self.consumer.tdt(ctx, &section);
    }
}

type TdtSectionConsumer<Ctx, C> = psi::CompactSyntaxFramer<TdtProcessor<Ctx, C>>;

pub struct TdtPacketFilter<Ctx: demultiplex::DemuxContext, C: TdtConsumer<Ctx>> {
    framer: TdtSectionConsumer<Ctx, C>,
}
impl<Ctx: demultiplex::DemuxContext, C: TdtConsumer<Ctx>> TdtPacketFilter<Ctx, C> {
    pub fn new(consumer: C) -> Self {
        let proc = TdtProcessor::new(consumer);
        TdtPacketFilter {
            framer: psi::CompactSyntaxFramer::new(TDT_PID, proc),
        }
    }
}
impl<Ctx: demultiplex::DemuxContext, C: TdtConsumer<Ctx>> demultiplex::PacketFilter
    for TdtPacketFilter<Ctx, C>
{
    type Ctx = Ctx;

    fn consume(&mut self, ctx: &mut Self::Ctx, pk: &packet::Packet<'_>) {
        self.framer.consume(ctx, pk);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mpeg2ts_reader::demultiplex;
    use mpeg2ts_reader::psi;

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
                demultiplex::FilterRequest::ByStream { .. } => {
                    NullFilterSwitch::Nul(demultiplex::NullPacketFilter::default())
                }
                demultiplex::FilterRequest::Pmt {
                    pid,
                    program_number,
                } => NullFilterSwitch::Pmt(demultiplex::PmtPacketFilter::new(pid, program_number)),
                demultiplex::FilterRequest::Nit { .. } => {
                    NullFilterSwitch::Nul(demultiplex::NullPacketFilter::default())
                }
            }
        }
    }

    struct AssertConsumer {
        called: bool,
    }
    impl TdtConsumer<NullDemuxContext> for AssertConsumer {
        fn tdt(&mut self, _ctx: &mut NullDemuxContext, section: &TdtSection<'_>) {
            let ts = section.utc_time().unwrap();
            assert_eq!((1993, 10, 13), ts.date());
            assert_eq!(12, ts.hours());
            assert_eq!(30, ts.minutes());
            assert_eq!(45, ts.seconds());
            assert_eq!(750515445, ts.to_unix_timestamp());
            self.called = true;
        }
    }

    #[test]
    fn tdt_section_parse() {
        let mut ctx = NullDemuxContext::new();
        let mut processor = TdtProcessor::new(AssertConsumer { called: false });

        // TDT section: table_id=0x70, section_syntax_indicator=0,
        // section_length=5, payload = MJD+BCD for 1993-10-13 12:30:45
        let section = [
            0x70, 0x00, 0x05, // common header
            0xC0, 0x79, 0x12, 0x30, 0x45, // 5-byte MJD+BCD timestamp
        ];

        let header = psi::SectionCommonHeader::new(&section[..psi::SectionCommonHeader::SIZE]);
        psi::WholeCompactSyntaxPayloadParser::section(
            &mut processor,
            &mut ctx,
            &header,
            &section[..],
        );
        assert!(processor.consumer.called);
    }

    #[test]
    fn tdt_ignores_tot() {
        let mut ctx = NullDemuxContext::new();
        let mut processor = TdtProcessor::new(AssertConsumer { called: false });

        // TOT section: table_id=0x73 - should be ignored
        let section = [0x73, 0x00, 0x05, 0xC0, 0x79, 0x12, 0x30, 0x45];

        let header = psi::SectionCommonHeader::new(&section[..psi::SectionCommonHeader::SIZE]);
        psi::WholeCompactSyntaxPayloadParser::section(
            &mut processor,
            &mut ctx,
            &header,
            &section[..],
        );
        assert!(!processor.consumer.called);
    }
}
