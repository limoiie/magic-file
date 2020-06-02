use std::io::{BufReader, Error, ErrorKind, Result, Seek, SeekFrom, BufRead, Read};
use std::mem;

/// Cast the memory at the [pos] in the [buf] into a raw pointer of the given
/// type [T]
pub(crate) fn cast_at<S: Seek + Read, T>(buf: &mut BufReader<S>, pos: SeekFrom) -> Result<*const T> {
    seek_relative(buf, pos)?;
    buf.fill_buf();

    let typ_size = mem::size_of::<T>();
    if typ_size <= buf.buffer().len() {
        Ok(buf.buffer().as_ptr() as *const T)
    } else {
        Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Too big type size({}) to fit into buffer({})! Try to realloc buffer",
                typ_size, buf.buffer().len()
            )
        ))
    }
}

/// Returns the current seek position from the start of the stream.
///
/// Though a mutable buf is required, there will be no change after
/// the seek
fn stream_position<R: Seek>(buf: &mut BufReader<R>) -> Result<u64> {
    buf.get_mut().seek(SeekFrom::Current(0))
}

/// Return the length of the whole stream
///
/// Though a mutable buf is required, there will be no change after
/// the seek. The cursor will be reset as what it is before the calling
fn stream_len<R: Seek>(buf: &mut BufReader<R>) -> Result<u64> {
    let old_pos = stream_position(buf)?;
    let len = buf.get_mut().seek(SeekFrom::End(0))?;

    if old_pos != len {
        buf.get_mut().seek(SeekFrom::Start(old_pos))?;
    }
    Ok(len)
}

/// Move the cursor to the given position
///
/// The position [pos] will be transformed to a relative one so that
/// the efficiency version seek of [`BufReader`] can be leveraged.
///
/// [`BufReader`]: std.io.BufReader.html
fn seek_relative<R: Seek>(buf: &mut BufReader<R>, pos: SeekFrom) -> Result<u64> {
    let current = stream_position(buf)?;
    let (relative, start) = match pos {
        SeekFrom::Start(start) => (start as i64 - current as i64, start),
        SeekFrom::End(end) => {
            let start = (stream_len(buf)? as i64 + end) as u64;
            (start as i64 - current as i64, start)
        }
        SeekFrom::Current(relative) => {
            let start = (current as i64 + relative) as u64;
            (relative, start)
        }
    };
    buf.seek_relative(relative)?;
    Ok(start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(C)]
    #[derive(Debug, PartialEq)]
    struct A {
        age: u32,
        name: [u8; 4],
    }

    unsafe fn as_raw_bytes<T>(t: &mut T) -> Vec<u8> {
        let size = mem::size_of::<T>();
        let x = t as *mut T as *mut u8;
        Vec::from_raw_parts(x, size, size)
    }

    #[test]
    fn test() {
        let mut a = A {
            age: 0,
            name: [1, 2, 3, 4],
        };
        unsafe {
            let raw_a = as_raw_bytes(&mut a);
            let a_ = raw_a.as_ptr() as *const A;
            let a_ = &*a_;
            assert_eq!(a.name, a_.name);
            assert_eq!(a.age, a_.age);
            // println!("name: {:?}, age: {:?}", a.name, a.age)
        }
    }
}
