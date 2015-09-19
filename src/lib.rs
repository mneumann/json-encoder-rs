#![feature(vec_push_all, slice_bytes, convert)]

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

    #[inline(always)]
    pub fn as_mut_ref(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    #[inline(always)]
    pub fn push(&mut self, byte: u8) {
       self.data.push(byte);
    }

    #[inline(always)]
    pub fn push_all(&mut self, bytes: &[u8]) {
        use std::slice::bytes::copy_memory;

        if bytes.is_empty() { return; }

        let remaining = self.data.capacity() - self.data.len();
        if remaining < bytes.len() {
            let missing = bytes.len() - remaining;
            self.data.reserve(missing);
        }

        unsafe {
            let end = self.data.len();
            self.data.set_len(end + bytes.len());
            copy_memory(bytes, &mut self.data.as_mut_slice()[end..]);
        }
    }

    #[inline(always)]
    pub fn push_all_around(&mut self, around: u8, bytes: &[u8]) {
       self.data.push(around);
       self.data.push_all(bytes);
       self.data.push(around);
    }


    #[inline(always)]
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
    pub fn with_capacity(capa: usize) -> JsonEncoder {
        JsonEncoder{buffer: Buffer::with_capacity(capa)}
    }

    #[inline]
    pub fn encode_raw(&mut self, raw: &[u8]) {
        self.buffer.push_all(raw);
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
        const CHARS: &'static [u8] = b"0123456789";
        const MAX_DIGITS: usize = 20;
        
        if value == 0 {
            self.buffer.push_all_around(b'"', b"0");
            return;
        }

        let mut digits: [u8; MAX_DIGITS] = [b'0'; MAX_DIGITS];
        let mut i = MAX_DIGITS;
        let mut n = value;
        while n > 0 {
            i -= 1;
            digits[i] = CHARS[(n % 10) as usize];
            n = n / 10;
        }

        self.buffer.push_all_around(b'"', &digits[i..]);
    }

    #[inline]
    pub fn encode_str(&mut self, s: &str) {
        self.buffer.push(b'"');
        self.escape_bytes(s.as_bytes());
        self.buffer.push(b'"');
    }

    #[inline]
    pub fn encode_i32(&mut self, value: i32) {
        if value == 0 {
            self.buffer.push(b'0');
        } else if value > 0 {
            self.encode_u31(value as u32);
        } else {
            self.buffer.push(b'-');
            self.encode_u31((-value) as u32);
        }
    }

    // encodes a 31-bit unsigned integer != 0
    #[inline]
    fn encode_u31(&mut self, value: u32) {
        const CHARS: &'static [u8] = b"0123456789";
        const MAX_DIGITS: usize = 10;
        debug_assert!(value != 0);

        let mut digits: [u8; MAX_DIGITS] = [b'0'; MAX_DIGITS];
        let mut i = MAX_DIGITS;
        let mut n = value;
        while n > 0 {
            i -= 1;
            digits[i] = CHARS[(n % 10) as usize];
            n = n / 10;
        }

        self.encode_raw(&digits[i..]);
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
            self.js.buffer.push(b',');
        } else {
            self.needs_sep = true;
        }
        self.js.encode_str_noescape(name);
        self.js.buffer.push(b':');

        f(self.js)
    }

    #[inline]
    pub fn encode_field_array<F, T>(&mut self, name: &str, mut f: F) -> T where F: FnMut(&mut JsonArrayEncoder) -> T {
        self.encode_field(name, |js| js.encode_array(|jsa| f(jsa)))
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
}
