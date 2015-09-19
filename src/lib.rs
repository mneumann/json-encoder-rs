#![feature(vec_push_all)]

pub struct JsonEncoder {
    buffer: Vec<u8>
}

impl JsonEncoder {
    pub fn new() -> JsonEncoder {
        JsonEncoder{buffer: Vec::new()}
    }

    #[inline(always)]
    pub fn encode_raw(&mut self, raw: &[u8]) {
        self.buffer.push_all(raw);
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buffer
    }

    #[inline(always)]
    pub fn encode_raw_str(&mut self, raw_str: &str) {
        self.buffer.push_all(raw_str.as_bytes());
    }

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

    pub fn encode_u64_decimal(&mut self, value: u64) {
        self.buffer.push(b'"');
        self._encode_u64_decimal(value);
        self.buffer.push(b'"');
    }

    // encodes as decimal string
    #[inline(always)]
    fn _encode_u64_decimal(&mut self, value: u64) {
        const CHARS: &'static [u8] = b"0123456789";
        const MAX_DIGITS: usize = 20;
        
        if value == 0 {
            self.buffer.push(b'0');
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

        self.encode_raw(&digits[i..]);
    }

    #[inline(always)]
    pub fn encode_str(&mut self, s: &str) {
        self.buffer.push(b'"');
        self.escape_bytes(s.as_bytes());
        self.buffer.push(b'"');
    }

    #[inline(always)]
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
}

pub struct JsonObjectEncoder {
    js: JsonEncoder,
    needs_sep: bool,
}

impl JsonObjectEncoder {
    pub fn new() -> JsonObjectEncoder {
        JsonObjectEncoder {
                js: JsonEncoder::new(),
                needs_sep: false
        }
    }

    pub fn begin(&mut self) {
        self.js.encode_raw(b"{");
    }

    pub fn end(&mut self) {
        self.js.encode_raw(b"}");
    }

    // XXX: name MAY NOT include escapable characters
    #[inline(always)]
    pub fn field<F:Fn(&mut JsonEncoder)>(&mut self, name: &str, f: F) {
        if self.needs_sep {
            self.js.buffer.push(b',');
        }
        self.js.buffer.push(b'"');
        self.js.encode_raw_str(name);
        self.js.buffer.push(b'"');
        self.js.buffer.push(b':');

        f(&mut self.js);
        self.needs_sep = true;
    }

    pub fn into_json_encoder(self) -> JsonEncoder {
        self.js
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.into_json_encoder().into_vec()
    }
}

#[test]
fn test_json_obj_encoder() {
    let mut jso = JsonObjectEncoder::new();
    jso.begin();
    jso.end();
    assert_eq!(b"{}", &jso.into_vec()[..]);

    let mut jso = JsonObjectEncoder::new();
    jso.begin();
    jso.field("total", |js| js.encode_i32(31));
    jso.end();
    assert_eq!(b"{\"total\":31}", &jso.into_vec()[..]);

    let mut jso = JsonObjectEncoder::new();
    jso.begin();
    jso.field("total", |js| js.encode_i32(31));
    jso.field("next", |js| js.encode_str("abc"));
    jso.end();
    assert_eq!(b"{\"total\":31,\"next\":\"abc\"}", &jso.into_vec()[..]);
}
