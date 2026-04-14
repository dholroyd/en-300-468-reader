//! DVB timestamp types shared across table parsers.

use std::fmt;

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

fn is_valid_bcd(byte: u8) -> bool {
    (byte >> 4) <= 9 && (byte & 0x0f) <= 9
}

/// A DVB timestamp encoded as a 5-byte Modified Julian Date plus BCD-encoded UTC time,
/// as defined in EN 300 468 Annex C.
pub struct MjdTimestamp<'buf> {
    data: &'buf [u8],
}

impl<'buf> MjdTimestamp<'buf> {
    pub fn new(data: &'buf [u8]) -> Result<MjdTimestamp<'buf>, MjdTimestampError> {
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

    /// Convert to nanoseconds since 1970-01-01 00:00:00 UTC.
    ///
    /// The DVB BCD time only has second precision, so the sub-second part
    /// is always zero.
    pub fn to_unix_nanos(&self) -> i64 {
        self.to_unix_timestamp() * 1_000_000_000
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

#[cfg(test)]
mod test {
    use super::*;

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
    fn mjd_timestamp_to_unix_nanos() {
        let data = [0xC0, 0x79, 0x12, 0x30, 0x45];
        let ts = MjdTimestamp::new(&data).unwrap();
        assert_eq!(750515445_000_000_000, ts.to_unix_nanos());
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
        let data = [0xC0, 0x79, 0x24, 0x30, 0x45];
        assert!(matches!(
            MjdTimestamp::new(&data),
            Err(MjdTimestampError::HoursOutOfRange(24))
        ));
    }

    #[test]
    fn mjd_timestamp_minutes_out_of_range() {
        let data = [0xC0, 0x79, 0x12, 0x60, 0x45];
        assert!(matches!(
            MjdTimestamp::new(&data),
            Err(MjdTimestampError::MinutesOutOfRange(60))
        ));
    }

    #[test]
    fn mjd_timestamp_seconds_out_of_range() {
        let data = [0xC0, 0x79, 0x12, 0x30, 0x60];
        assert!(matches!(
            MjdTimestamp::new(&data),
            Err(MjdTimestampError::SecondsOutOfRange(60))
        ));
    }
}
