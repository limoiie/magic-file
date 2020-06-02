use std::io::{BufReader, Read, Seek, SeekFrom};

use crate::ext_buf;
use crate::magic_match::{MagicMatch, MResult};
use crate::raw_bytes::{split_at_null, to_bytes};

#[derive(Debug, PartialEq)]
enum TarType {
    GNUUnixStandardTar,
    UnixStandardTar,
    OldFashionTar,
}

#[derive(Default, Debug, PartialEq)]
struct MagicTarMatcher {
    typ: Option<TarType>,
}

impl<S: Seek + Read> MagicMatch<S> for MagicTarMatcher {
    fn magic_match(&mut self, buf: &mut BufReader<S>) -> MResult {
        let raw_record = ext_buf::cast_at::<S, TarRecord>(buf, SeekFrom::Start(0))?;
        unsafe {
            let record = &*raw_record;
            self.typ = if record.is_checksum_valid() {
                Some(record.typ())
            } else {
                None
            };
        }
        Ok(())
    }
}

#[repr(C)]
struct TarRecord {
    name: [u8; 100],
    mode: [u8; 8],
    uid: [u8; 8],
    gid: [u8; 8],
    size: [u8; 12],
    mtime: [u8; 12],
    chksum: [u8; 8],
    linkflag: u8,
    linkname: [u8; 100],
    magic: [u8; 8],
    uname: [u8; 32],
    gname: [u8; 32],
    devmajor: [u8; 8],
    devminor: [u8; 8],
}

impl TarRecord {
    fn is_checksum_valid(&self) -> bool {
        self.get_checksum() == self.compute_checksum()
    }

    fn typ(&self) -> TarType {
        let (magic, _) = split_at_null(&self.magic);
        if let Ok(magic) = std::str::from_utf8(magic) {
            match magic {
                "ustar  " => return TarType::GNUUnixStandardTar,
                "ustar" => return TarType::UnixStandardTar,
                _ => {}
            }
        }
        TarType::OldFashionTar
    }

    /// Get the checksum stored in the [chksum] field
    ///
    /// This method will not compute the chksum
    fn get_checksum(&self) -> i32 {
        // the check sum field [chksum] is stored in the form of octal string which
        // ends with a '\0'
        let (chksum, _) = split_at_null(&self.chksum);
        let src = std::str::from_utf8(chksum).unwrap();
        i32::from_str_radix(src, 8).unwrap()
    }

    /// Compute the checksum by summing up over the bytes in the instance
    fn compute_checksum(&self) -> i32 {
        // the check sum is computed by accumulating all the bytes in the datastruct
        // with taking all the [chksum] field as blanks
        to_bytes(self).iter().fold(0i32, |acc, &x| acc + x as i32)
            // take the bytes in chksum as whitespaces
            - self.chksum.iter().fold(0i32, |acc, &x| acc + x as i32)
            + self.chksum.iter().fold(0i32, |acc, &x| acc + ' ' as i32)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    #[test]
    fn test() {
        let path = "/Users/ligengwang/Downloads/opensource/sqlite.tar";
        let file = File::open(path).unwrap();
        let mut buff = BufReader::new(file);
        let mut matcher = MagicTarMatcher { typ: None };
        matcher.magic_match(&mut buff).unwrap();
        println!("{:?}", matcher.typ);
    }
}
