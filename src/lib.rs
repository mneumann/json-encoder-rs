#![feature(test)]

#[cfg(test)]
extern crate test;

extern crate vec_byte_appender;

use std::ptr;
use vec_byte_appender::append_bytes_uninit;

pub struct Buffer {
   data: Vec<u8>
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer{data: Vec::new()}
    }

    pub fn with_capacity(capa: usize) -> Buffer {
        Buffer{data: Vec::with_capacity(capa)}
    }

    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
    }

    #[inline(always)]
    pub fn as_mut_ref(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    #[inline]
    pub fn push(&mut self, byte: u8) {
       self.data.push(byte);
    }

    #[inline]
    pub fn push_all(&mut self, bytes: &[u8]) {
        let len = bytes.len();
        if len == 0 { return }
        append_bytes_uninit(&mut self.data, len, |ext| unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), ext.as_mut_ptr(), len);
        });
    }

    // XXX: If bytes.len() > isize::max, this fails.
    #[inline]
    pub fn push_all_around(&mut self, around: u8, bytes: &[u8]) {
        let len = bytes.len();
        append_bytes_uninit(&mut self.data, len + 2, |ext| unsafe {
            ptr::write(ext.as_mut_ptr(), around);
            ptr::copy_nonoverlapping(bytes.as_ptr(),
                                     ext.as_mut_ptr().offset(1),
                                     len);
            ptr::write(ext.as_mut_ptr().offset(1+len as isize), around);
        });
    }

    // XXX: If before, after, bytes.len() > isize::max, this fails.
    #[inline]
    pub fn push_all_around2(&mut self, before: &[u8], bytes: &[u8], after: &[u8]) {
        append_bytes_uninit(&mut self.data, before.len() + bytes.len() + after.len(), |ext| unsafe {
            ptr::copy_nonoverlapping(before.as_ptr(),
                                     ext.as_mut_ptr(),
                                     before.len());
            ptr::copy_nonoverlapping(bytes.as_ptr(),
                                     ext.as_mut_ptr().offset(before.len() as isize),
                                     bytes.len());
            ptr::copy_nonoverlapping(after.as_ptr(),
                                     ext.as_mut_ptr().offset(before.len() as isize).offset(bytes.len() as isize),
                                     after.len());
        });
    }

    #[inline]
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }
}

pub struct JsonEncoder {
    buffer: Buffer
}

impl JsonEncoder {
    #[inline]
    pub fn new() -> JsonEncoder {
        JsonEncoder{buffer: Buffer::new()}
    }

    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    #[inline]
    pub fn with_capacity(capa: usize) -> JsonEncoder {
        JsonEncoder{buffer: Buffer::with_capacity(capa)}
    }

    #[inline]
    pub fn encode_raw(&mut self, raw: &[u8]) {
        self.buffer.push_all(raw);
    }

    #[inline]
    pub fn encode_raw_around(&mut self, around: u8, raw: &[u8]) {
        self.buffer.push_all_around(around, raw);
    }


    #[inline]
    pub fn into_vec(self) -> Vec<u8> {
        self.buffer.into_vec()
    }

    #[inline]
    pub fn with_buffer<F>(&mut self, mut f: F) where F: FnMut(&mut Buffer) {
        f(&mut self.buffer);
    }

    #[inline]
    pub fn encode_str_noescape(&mut self, raw_str: &str) {
        self.buffer.push_all_around(b'"', raw_str.as_bytes());
    }

    #[inline]
    fn escape_bytes(&mut self, bytes: &[u8]) {
        let mut start = 0;

        for (i, byte) in bytes.iter().enumerate() {
            let escaped = match *byte {
                b'"' => b"\\\"",
                b'\\' => b"\\\\",
                b'\x08' => b"\\b",
                b'\x0c' => b"\\f",
                b'\n' => b"\\n",
                b'\r' => b"\\r",
                b'\t' => b"\\t",
                _ => {
                    continue;
                }
            };

            if start < i {
                self.encode_raw(&bytes[start..i]);
            }

            self.encode_raw(escaped);

            start = i + 1;
        }

        if start != bytes.len() {
            self.encode_raw(&bytes[start..]);
        }
    }

    // encodes as decimal string
    #[inline]
    pub fn encode_decimal_str(&mut self, value: u64) {
        use std::mem;
        const CHARS: &'static [u8] = b"0123456789";
        const MAX_DIGITS: usize = 20;

        let mut digits: [u8; MAX_DIGITS] = unsafe{ mem::uninitialized() };
        let mut n = value;
        let mut start = MAX_DIGITS;
        for digit in digits.iter_mut().rev() {
            start -= 1;
            *digit = unsafe { *CHARS.get_unchecked((n % 10) as usize) };
            n = n / 10;
            if n == 0 {
                break;
            }
        }
        self.encode_raw_around(b'"', &digits[start..]);
    }

    #[inline]
    pub fn encode_str(&mut self, s: &str) {
        self.buffer.push(b'"');
        self.escape_bytes(s.as_bytes());
        self.buffer.push(b'"');
    }

    #[inline]
    pub fn encode_i32(&mut self, value: i32) {
        if value >= 0 {
            self.encode_u32(value as u32);
        } else {
            self.buffer.push(b'-');
            self.encode_u32((-value) as u32);
        }
    }

    /// encodes a 32-bit unsigned integer
    #[inline]
    pub fn encode_u32(&mut self, value: u32) {
        use std::mem;
        const CHARS: &'static [u8] = b"0123456789";
        const MAX_DIGITS: usize = 10;

        let mut digits: [u8; MAX_DIGITS] = unsafe{ mem::uninitialized() };
        let mut n = value;
        let mut start = MAX_DIGITS;
        for digit in digits.iter_mut().rev() {
            start -= 1;
            *digit = unsafe { *CHARS.get_unchecked((n % 10) as usize) };
            n = n / 10;
            if n == 0 {
                break;
            }
        }

        self.encode_raw(&digits[start..]);
    }

    /// encodes a 32-bit unsigned integer as hexadecimal
    #[inline]
    pub fn encode_hex_u32(&mut self, value: u32) {
        use std::mem;
        const CHARS: &'static [u8] = b"0123456789ABCDEF";
        const MAX_DIGITS: usize = 8;

        let mut digits: [u8; MAX_DIGITS] = unsafe{ mem::uninitialized() };
        let mut n = value;
        let mut start = MAX_DIGITS;
        for digit in digits.iter_mut().rev() {
            start -= 1;
            *digit = unsafe { *CHARS.get_unchecked((n & 15) as usize) };
            n = n / 16;
            if n == 0 {
                break;
            }
        }

        self.encode_raw(&digits[start..]);
    }

    #[inline]
    pub fn encode_hex_u32_fast(&mut self, n: u32) {
        const CHARS: &'static [u8] = b"0123456789ABCDEF";
        const MAX_DIGITS: usize = 8;

        let digits: [u8; MAX_DIGITS] = [
           unsafe { *CHARS.get_unchecked( ((n >> 28) & 15) as usize ) },
           unsafe { *CHARS.get_unchecked( ((n >> 24) & 15) as usize ) },
           unsafe { *CHARS.get_unchecked( ((n >> 20) & 15) as usize ) },
           unsafe { *CHARS.get_unchecked( ((n >> 16) & 15) as usize ) },
           unsafe { *CHARS.get_unchecked( ((n >> 12) & 15) as usize ) },
           unsafe { *CHARS.get_unchecked( ((n >>  8) & 15) as usize ) },
           unsafe { *CHARS.get_unchecked( ((n >>  4) & 15) as usize ) },
           unsafe { *CHARS.get_unchecked( ((n >>  0) & 15) as usize ) }
        ];

        self.encode_raw(&digits[..]);
    }

    #[inline]
    pub fn encode_obj<F, T>(&mut self, mut f: F) -> T where F: FnMut(&mut JsonObjectEncoder) -> T {
        self.buffer.push(b'{');
        let t = {
            f(&mut JsonObjectEncoder {js: self, needs_sep: false})
        };
        self.buffer.push(b'}');
        t
    }

    #[inline]
    pub fn encode_array<F, T>(&mut self, mut f: F) -> T where F: FnMut(&mut JsonArrayEncoder) -> T {
        self.buffer.push(b'[');
        let t = {
            f(&mut JsonArrayEncoder {js: self, needs_sep: false})
        };
        self.buffer.push(b']');
        t
    }

    #[inline]
    pub fn encode_array_nobrackets<F, T>(&mut self, mut f: F) -> T where F: FnMut(&mut JsonArrayEncoder) -> T {
        f(&mut JsonArrayEncoder {js: self, needs_sep: false})
    }

    #[inline]
    pub fn obj_single_str_field(name: &str, s: &str) -> Vec<u8> {
        let mut js = JsonEncoder::with_capacity(name.len() + s.len() + 2 + 2 + 1 + 2);
        js.encode_obj(|jso| jso.encode_field_str(name, s));
        js.into_vec()
    }

}

pub struct JsonObjectEncoder<'a> {
    js: &'a mut JsonEncoder,
    needs_sep: bool,
}

impl<'a> JsonObjectEncoder<'a> {
    // XXX: name MAY NOT include escapable characters
    #[inline]
    pub fn encode_field<F, T>(&mut self, name: &str, mut f: F) -> T where F: FnMut(&mut JsonEncoder) -> T {
        if self.needs_sep {
            self.js.buffer.push_all_around2(b",\"", name.as_bytes(), b"\":");
        } else {
            self.js.buffer.push_all_around2(b"\"", name.as_bytes(), b"\":");
            self.needs_sep = true;
        }
        f(self.js)
    }

    #[inline]
    pub fn encode_field_array<F, T>(&mut self, name: &str, mut f: F) -> T where F: FnMut(&mut JsonArrayEncoder) -> T {
        self.encode_field(name, |js| js.encode_array(|jsa| f(jsa)))
    }

    #[inline]
    pub fn encode_field_obj<F, T>(&mut self, name: &str, mut f: F) -> T where F: FnMut(&mut JsonObjectEncoder) -> T {
        self.encode_field(name, |js| js.encode_obj(|jso| f(jso)))
    }

    #[inline]
    pub fn encode_field_i32(&mut self, name: &str, val: i32) {
        self.encode_field(name, |js| js.encode_i32(val));
    }

    #[inline]
    pub fn encode_field_str(&mut self, name: &str, s: &str) {
        self.encode_field(name, |js| js.encode_str(s));
    }
}

pub struct JsonArrayEncoder<'a> {
    js: &'a mut JsonEncoder,
    needs_sep: bool,
}

impl<'a> JsonArrayEncoder<'a> {
    #[inline]
    pub fn encode_elm<F, T>(&mut self, mut f: F) -> T where F: FnMut(&mut JsonEncoder) -> T {
        if self.needs_sep {
            self.js.buffer.push(b',');
        } else {
            self.needs_sep = true;
        }
        f(self.js)
    }

    #[inline]
    pub fn encode_elm_i32(&mut self, val: i32) {
        self.encode_elm(|js| js.encode_i32(val));
    }

    #[inline]
    pub fn encode_elm_str(&mut self, s: &str) {
        self.encode_elm(|js| js.encode_str(s));
    }

    #[inline]
    pub fn encode_elm_obj<F, T>(&mut self, mut f: F) -> T where F: FnMut(&mut JsonObjectEncoder) -> T {
        self.encode_elm(|js| js.encode_obj(|jso| f(jso)))
    }
}


#[test]
fn test_json_obj_encoder() {
    use std::str;

    let mut js = JsonEncoder::new();
    js.encode_obj(|_|{});
    assert_eq!(b"{}", &js.into_vec()[..]);

    let mut js = JsonEncoder::new();
    js.encode_obj(|jso| {
        jso.encode_field("total", |js| js.encode_i32(31));
    });
    assert_eq!(b"{\"total\":31}", &js.into_vec()[..]);

    let mut js = JsonEncoder::new();
    js.encode_obj(|jso| {
        jso.encode_field("total", |js| js.encode_i32(31));
        jso.encode_field("next", |js| js.encode_str("abc"));
    });
    assert_eq!(b"{\"total\":31,\"next\":\"abc\"}", &js.into_vec()[..]);

    let mut js = JsonEncoder::new();
    js.encode_obj(|jso| {
        jso.encode_field("total", |js| js.encode_i32(31));
        jso.encode_field("next", |js| js.encode_str("abc"));
        jso.encode_field("tags", |js| {
                js.encode_array(|jsa| {
                        jsa.encode_elm(|js| js.encode_i32(1));
                        jsa.encode_elm(|js| js.encode_i32(2));
                });
        });
    });
    assert_eq!("{\"total\":31,\"next\":\"abc\",\"tags\":[1,2]}", str::from_utf8(&js.into_vec()[..]).unwrap());

    let json = JsonEncoder::obj_single_str_field("total", "abcdef");
    assert_eq!(b"{\"total\":\"abcdef\"}", &json[..]);

    let json = JsonEncoder::obj_single_str_field("total", "");
    assert_eq!(b"{\"total\":\"\"}", &json[..]);
}

#[bench]
fn bench_encode_i32(b: &mut test::Bencher) {
    let mut js = JsonEncoder::with_capacity(80);
    b.iter(|| {
        let n = test::black_box(10000);
        for _ in (0..n) {
            js.clear();
            js.encode_i32(123_456_789);
            js.encode_i32(321_654_987);
            js.encode_i32(231_564_857);
            js.encode_i32(189_123_456);
        }
    });
}

#[bench]
fn bench_encode_u32(b: &mut test::Bencher) {
    let mut js = JsonEncoder::with_capacity(80);
    b.iter(|| {
        let n = test::black_box(10000);
        for _ in (0..n) {
            js.clear();
            js.encode_u32(123_456_789);
            js.encode_u32(321_654_987);
            js.encode_u32(231_564_857);
            js.encode_u32(189_123_456);
        }
    });
}

#[bench]
fn bench_encode_hex_u32(b: &mut test::Bencher) {
    let mut js = JsonEncoder::with_capacity(80);
    b.iter(|| {
        let n = test::black_box(10000);
        for _ in (0..n) {
            js.clear();
            js.encode_hex_u32(123_456_789);
            js.encode_hex_u32(321_654_987);
            js.encode_hex_u32(231_564_857);
            js.encode_hex_u32(189_123_456);
        }
    });
}

#[bench]
fn bench_encode_hex_u32_fast(b: &mut test::Bencher) {
    let mut js = JsonEncoder::with_capacity(80);
    b.iter(|| {
        let n = test::black_box(10000);
        for _ in (0..n) {
            js.clear();
            js.encode_hex_u32_fast(123_456_789);
            js.encode_hex_u32_fast(321_654_987);
            js.encode_hex_u32_fast(231_564_857);
            js.encode_hex_u32_fast(189_123_456);
        }
    });
}

#[test]
fn test_encode_u32() {
    let mut js = JsonEncoder::new();
    js.encode_u32(0);
    let vec = js.into_vec();
    assert_eq!(b"0", &vec[..]);

    let mut js = JsonEncoder::new();
    js.encode_u32(123000);
    let vec = js.into_vec();
    assert_eq!(b"123000", &vec[..]);

    let mut js = JsonEncoder::new();
    js.encode_u32(1230001);
    let vec = js.into_vec();
    assert_eq!(b"1230001", &vec[..]);
}
