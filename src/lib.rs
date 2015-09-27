#![feature(test)]
#![feature(link_llvm_intrinsics)]

#[cfg(test)]
extern crate test;

extern crate vec_byte_appender;

use std::ptr;
use vec_byte_appender::{append_bytes_uninit, append_bytes_uninit_flex};

extern {
    #[link_name = "llvm.expect.i64"]
    fn expect_u64(val: u64, expected_val: u64) -> u64;

    #[link_name = "llvm.expect.i8"]
    fn expect_u8(val: u8, expected_val: u8) -> u8;
}

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

    pub fn get_current_position(&self) -> usize {
        self.data.len()
    }

    pub fn set_current_position(&mut self, pos: usize) {
        self.data.truncate(pos);
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
    buffer: Buffer,
}

const LUT: [u8; 256] = [
     0 /*   0 */,  0 /*   1 */,  0 /*   2 */,  0 /*   3 */,  0 /*   4 */,  0 /*   5 */,  0 /*   6 */,  0 /*   7 */,
    98 /*   8 */, 116 /*   9 */, 110 /*  10 */,  0 /*  11 */, 102 /*  12 */, 114 /*  13 */,  0 /*  14 */,  0 /*  15 */,
     0 /*  16 */,  0 /*  17 */,  0 /*  18 */,  0 /*  19 */,  0 /*  20 */,  0 /*  21 */,  0 /*  22 */,  0 /*  23 */,
     0 /*  24 */,  0 /*  25 */,  0 /*  26 */,  0 /*  27 */,  0 /*  28 */,  0 /*  29 */,  0 /*  30 */,  0 /*  31 */,
     0 /*  32 */,  0 /*  33 */, 34 /*  34 */,  0 /*  35 */,  0 /*  36 */,  0 /*  37 */,  0 /*  38 */,  0 /*  39 */,
     0 /*  40 */,  0 /*  41 */,  0 /*  42 */,  0 /*  43 */,  0 /*  44 */,  0 /*  45 */,  0 /*  46 */,  0 /*  47 */,
     0 /*  48 */,  0 /*  49 */,  0 /*  50 */,  0 /*  51 */,  0 /*  52 */,  0 /*  53 */,  0 /*  54 */,  0 /*  55 */,
     0 /*  56 */,  0 /*  57 */,  0 /*  58 */,  0 /*  59 */,  0 /*  60 */,  0 /*  61 */,  0 /*  62 */,  0 /*  63 */,
     0 /*  64 */,  0 /*  65 */,  0 /*  66 */,  0 /*  67 */,  0 /*  68 */,  0 /*  69 */,  0 /*  70 */,  0 /*  71 */,
     0 /*  72 */,  0 /*  73 */,  0 /*  74 */,  0 /*  75 */,  0 /*  76 */,  0 /*  77 */,  0 /*  78 */,  0 /*  79 */,
     0 /*  80 */,  0 /*  81 */,  0 /*  82 */,  0 /*  83 */,  0 /*  84 */,  0 /*  85 */,  0 /*  86 */,  0 /*  87 */,
     0 /*  88 */,  0 /*  89 */,  0 /*  90 */,  0 /*  91 */, 92 /*  92 */,  0 /*  93 */,  0 /*  94 */,  0 /*  95 */,
     0 /*  96 */,  0 /*  97 */,  0 /*  98 */,  0 /*  99 */,  0 /* 100 */,  0 /* 101 */,  0 /* 102 */,  0 /* 103 */,
     0 /* 104 */,  0 /* 105 */,  0 /* 106 */,  0 /* 107 */,  0 /* 108 */,  0 /* 109 */,  0 /* 110 */,  0 /* 111 */,
     0 /* 112 */,  0 /* 113 */,  0 /* 114 */,  0 /* 115 */,  0 /* 116 */,  0 /* 117 */,  0 /* 118 */,  0 /* 119 */,
     0 /* 120 */,  0 /* 121 */,  0 /* 122 */,  0 /* 123 */,  0 /* 124 */,  0 /* 125 */,  0 /* 126 */,  0 /* 127 */,
     0 /* 128 */,  0 /* 129 */,  0 /* 130 */,  0 /* 131 */,  0 /* 132 */,  0 /* 133 */,  0 /* 134 */,  0 /* 135 */,
     0 /* 136 */,  0 /* 137 */,  0 /* 138 */,  0 /* 139 */,  0 /* 140 */,  0 /* 141 */,  0 /* 142 */,  0 /* 143 */,
     0 /* 144 */,  0 /* 145 */,  0 /* 146 */,  0 /* 147 */,  0 /* 148 */,  0 /* 149 */,  0 /* 150 */,  0 /* 151 */,
     0 /* 152 */,  0 /* 153 */,  0 /* 154 */,  0 /* 155 */,  0 /* 156 */,  0 /* 157 */,  0 /* 158 */,  0 /* 159 */,
     0 /* 160 */,  0 /* 161 */,  0 /* 162 */,  0 /* 163 */,  0 /* 164 */,  0 /* 165 */,  0 /* 166 */,  0 /* 167 */,
     0 /* 168 */,  0 /* 169 */,  0 /* 170 */,  0 /* 171 */,  0 /* 172 */,  0 /* 173 */,  0 /* 174 */,  0 /* 175 */,
     0 /* 176 */,  0 /* 177 */,  0 /* 178 */,  0 /* 179 */,  0 /* 180 */,  0 /* 181 */,  0 /* 182 */,  0 /* 183 */,
     0 /* 184 */,  0 /* 185 */,  0 /* 186 */,  0 /* 187 */,  0 /* 188 */,  0 /* 189 */,  0 /* 190 */,  0 /* 191 */,
     0 /* 192 */,  0 /* 193 */,  0 /* 194 */,  0 /* 195 */,  0 /* 196 */,  0 /* 197 */,  0 /* 198 */,  0 /* 199 */,
     0 /* 200 */,  0 /* 201 */,  0 /* 202 */,  0 /* 203 */,  0 /* 204 */,  0 /* 205 */,  0 /* 206 */,  0 /* 207 */,
     0 /* 208 */,  0 /* 209 */,  0 /* 210 */,  0 /* 211 */,  0 /* 212 */,  0 /* 213 */,  0 /* 214 */,  0 /* 215 */,
     0 /* 216 */,  0 /* 217 */,  0 /* 218 */,  0 /* 219 */,  0 /* 220 */,  0 /* 221 */,  0 /* 222 */,  0 /* 223 */,
     0 /* 224 */,  0 /* 225 */,  0 /* 226 */,  0 /* 227 */,  0 /* 228 */,  0 /* 229 */,  0 /* 230 */,  0 /* 231 */,
     0 /* 232 */,  0 /* 233 */,  0 /* 234 */,  0 /* 235 */,  0 /* 236 */,  0 /* 237 */,  0 /* 238 */,  0 /* 239 */,
     0 /* 240 */,  0 /* 241 */,  0 /* 242 */,  0 /* 243 */,  0 /* 244 */,  0 /* 245 */,  0 /* 246 */,  0 /* 247 */,
     0 /* 248 */,  0 /* 249 */,  0 /* 250 */,  0 /* 251 */,  0 /* 252 */,  0 /* 253 */,  0 /* 254 */,  0 /* 255 */
];

const LUT_BIN: [u64; 4] = [
    0b0000000000000000000000000000010000000000000000000011011100000000,
    0b0000000000000000000000000000000000010000000000000000000000000000,
    0b0000000000000000000000000000000000000000000000000000000000000000,
    0b0000000000000000000000000000000000000000000000000000000000000000
];

const LUT_HASH: u64 = LUT_BIN[0] | LUT_BIN[1] | LUT_BIN[2] | LUT_BIN[3];

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
    pub fn encode_str2(&mut self, s: &str) {
        self.buffer.push(b'"');
        self.escape_bytes(s.as_bytes());
        self.buffer.push(b'"');
    }

    #[inline]
    pub fn encode_str3(&mut self, s: &str) {
        let bytes = s.as_bytes();
        append_bytes_uninit_flex(&mut self.buffer.data, 2*bytes.len() + 2, |ext| {
            let dst = ext.as_mut_ptr();
            let mut count: isize = 0;

            unsafe { ptr::write(dst.offset(count), b'"'); }
            count += 1;

            let mut start = 0;
            for (i, &byte) in bytes.iter().enumerate() {
                let escaped2: u8 = match byte {
                    b'"' => b'"',
                    b'\\' => b'\\',
                    b'\x08' => b'b',
                    b'\x0c' => b'f',
                    b'\n' => b'n',
                    b'\r' => b'r',
                    b'\t' => b't',
                    _ => {
                        continue;
                    }
                };

                if start < i {
                    let wr_count = i - start;
                    unsafe { ptr::copy_nonoverlapping(bytes.as_ptr().offset(start as isize), dst.offset(count), wr_count); }
                    count += wr_count as isize;
                }

                unsafe { ptr::write(dst.offset(count), b'\\'); }
                count += 1;
                unsafe { ptr::write(dst.offset(count), escaped2); }
                count += 1;

                start = i + 1;
            }

            if start != bytes.len() {
               let wr_count = bytes.len() - start;
               unsafe { ptr::copy_nonoverlapping(bytes.as_ptr().offset(start as isize), dst.offset(count), wr_count); }
               count += wr_count as isize;
            }
            unsafe { ptr::write(dst.offset(count), b'"'); }
            count += 1;
            count as usize
        });
    }

    #[inline]
    pub fn encode_str_(&mut self, s: &str) {
        let bytes = s.as_bytes();
        append_bytes_uninit_flex(&mut self.buffer.data, 2*bytes.len() + 2, |ext| {
            let dst = ext.as_mut_ptr();

            unsafe { ptr::write(dst, b'"'); }
            let mut count: usize = 1;

            for &byte in bytes.iter() {
                let escaped2: u8 = unsafe { *LUT.get_unchecked(byte as usize) };
                if escaped2 != 0 {
                    unsafe { ptr::write(dst.offset(count as isize), b'\\'); }
                    count += 1;
                    unsafe { ptr::write(dst.offset(count as isize), escaped2); }
                    count += 1;
                } else {
                    unsafe { ptr::write(dst.offset(count as isize), byte); }
                    count += 1;
                }
            }

            unsafe { ptr::write(dst.offset(count as isize), b'"'); }
            count += 1;
            count
        });
    }
 
    #[inline]
    pub fn encode_str(&mut self, s: &str) {
        let bytes = s.as_bytes();
        append_bytes_uninit_flex(&mut self.buffer.data, 2*bytes.len() + 2, |ext| {
            let dst = ext.as_mut_ptr();

            unsafe { ptr::write(dst, b'"'); }
            let mut count: usize = 1;

            for &byte in bytes.iter() {
                //let lut_idx = (byte >> 6) as usize; // highest 2-bits
                //let reg = unsafe { *LUT_BIN.get_unchecked(lut_idx) };
                //let bit = ((LUT_HASH/*reg*/ >> (byte & 63)) as u8) & 1;

                if unsafe { expect_u64((LUT_HASH >> (byte & 63)) << 63, 0) } == 0 {
                    // likely
                    unsafe { ptr::write(dst.offset(count as isize), byte); }
                    count += 1;
                } else {
                    let escaped2: u8 = unsafe { *LUT.get_unchecked(byte as usize) };
                    if unsafe { expect_u8(escaped2, 0) } == 0 {
                        unsafe { ptr::write(dst.offset(count as isize), byte); }
                        count += 1;
                    } else {
                        unsafe { ptr::write(dst.offset(count as isize), b'\\'); }
                        count += 1;
                        unsafe { ptr::write(dst.offset(count as isize), escaped2); }
                        count += 1;
                    }
                }
            }

            unsafe { ptr::write(dst.offset(count as isize), b'"'); }
            count += 1;
            count
        });
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

    #[inline]
    pub fn obj<'a>(&'a mut self) -> JsonObj<'a> {
        JsonObj::open(self)
    }
}

pub struct JsonObj<'a> {
    js: &'a mut JsonEncoder,
    elm_count: usize,
}

pub struct JsonVal<'a> {
    js: &'a mut JsonEncoder,
}

pub struct JsonVec<'a> {
    js: &'a mut JsonEncoder,
    elm_count: usize,
}

pub trait JsonEncodable {
    fn encode(&self, &mut JsonEncoder);
}

impl<'a> JsonEncodable for &'a str {
    #[inline]
    fn encode(&self, js: &mut JsonEncoder) {
        js.encode_str(self);
    }
}

impl JsonEncodable for i32 {
    #[inline]
    fn encode(&self, js: &mut JsonEncoder) {
        js.encode_i32(*self);
    }
}

impl<'a> JsonVal<'a> {
    #[inline]
    pub fn value<T:JsonEncodable>(self, val: T) {
        val.encode(self.js);
    }

    #[inline]
    pub fn obj(self) -> JsonObj<'a> {
        JsonObj::open(self.js)
    }

    #[inline]
    pub fn vec(self) -> JsonVec<'a> {
        JsonVec::open(self.js)
    }
}

impl<'a> JsonObj<'a> {
    #[inline]
    fn open<'b>(js: &'b mut JsonEncoder) -> JsonObj<'b> {
        js.buffer.push(b'{');
        JsonObj {js: js, elm_count: 0}
    }

    #[inline]
    pub fn field<'b>(&'b mut self, name: &str) -> JsonVal<'b> {
        if self.elm_count > 0 {
            self.js.buffer.push_all_around2(b",\"", name.as_bytes(), b"\":");
        } else {
            self.js.buffer.push_all_around2(b"\"", name.as_bytes(), b"\":");
        }
        self.elm_count += 1;
        JsonVal {js: self.js} 
    }

    #[inline]
    pub fn end(self) {
        self.js.buffer.push(b'}');
    }
}


pub struct JsonVecSnapshot {
    elm_count: usize,
    pos: usize,
}

impl<'a> JsonVec<'a> {
    #[inline]
    fn open<'b>(js: &'b mut JsonEncoder) -> JsonVec<'b> {
        js.buffer.push(b'[');
        JsonVec {js: js, elm_count: 0}
    }

    #[inline]
    pub fn snapshot(&self) -> JsonVecSnapshot {
        JsonVecSnapshot {
            elm_count: self.elm_count,
            pos: self.js.buffer.get_current_position(),
        }
    }

    #[inline]
    pub fn rollback(&mut self, snapshot: JsonVecSnapshot) {
        self.elm_count = snapshot.elm_count;
        self.js.buffer.set_current_position(snapshot.pos);
    }

    #[inline]
    pub fn element<'b>(&'b mut self) -> JsonVal<'b> {
        if self.elm_count > 0 {
            self.js.buffer.push(b',');
        }
        self.elm_count += 1;
        JsonVal {js: self.js} 
    }

    #[inline]
    pub fn element_with_value<T:JsonEncodable>(&mut self, val: T) { 
        self.element().value(val);
    }

    #[inline]
    pub fn end(self) {
        self.js.buffer.push(b']');
    }
}


pub struct JsonObjectEncoder<'a> {
    js: &'a mut JsonEncoder,
    needs_sep: bool,
}

impl<'a> JsonObjectEncoder<'a> {

    #[inline]
    pub fn get_json_encoder<'b>(&'b mut self) -> &'b mut JsonEncoder {
        self.js
    }

    pub fn to_json_obj<'b>(&'b mut self) -> JsonObj<'b> {
        if self.needs_sep {
            JsonObj {js: self.js, elm_count: 1 /* XXX */}
        } else {
            JsonObj {js: self.js, elm_count: 0 /* XXX */}
        }
    }

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
    let v = js.into_vec(); 
    println!("{}", str::from_utf8(&v[..]).unwrap());
    assert_eq!(b"{\"total\":31,\"next\":\"abc\"}", &v[..]);

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

#[test]
fn test_json_empty_obj() {
    let mut js = JsonEncoder::new();
    {
        let obj = js.obj();
        obj.end();
    }

    assert_eq!(b"{}", &js.into_vec()[..]);
}

#[test]
fn test_json_single_field() {
    let mut js = JsonEncoder::new();
    {
        let mut obj = js.obj();
        obj.field("name").value("hallo");
        obj.end();
        // XXX: checkpoint
    }

    assert_eq!(b"{\"name\":\"hallo\"}", &js.into_vec()[..]);
}

#[test]
fn test_json_two_fields() {
    let mut js = JsonEncoder::new();
    {
        let mut obj = js.obj();
        obj.field("name").value("hallo");
        obj.field("i").value(123_i32);
        obj.end();
    }

    assert_eq!(b"{\"name\":\"hallo\",\"i\":123}", &js.into_vec()[..]);
}

#[test]
fn test_json_recursive_fields() {
    let mut js = JsonEncoder::new();
    {
        let mut obj = js.obj();
        obj.field("name").value("hallo");
        {
            let mut obj2 = obj.field("o").obj();
            obj2.field("name").value("hallo");
            obj2.end();
        }
        obj.end();
    }

    assert_eq!(&b"{\"name\":\"hallo\",\"o\":{\"name\":\"hallo\"}}"[..], &js.into_vec()[..]);
}

#[test]
fn test_json_recursive_fields_with_vec() {
    let mut js = JsonEncoder::new();
    {
        let mut obj = js.obj();
        obj.field("name").value("hallo");
        {
            let mut obj2 = obj.field("o").obj();
            obj2.field("name").value("hallo");
            {
                let mut v = obj2.field("a").vec();
                v.element_with_value(1i32);
                v.element_with_value("test");
                {
                    let obj3 = v.element().obj();
                    obj3.end();
                }
                v.end();
            }
            obj2.end();
        }
        obj.end();
    }

    assert_eq!(&b"{\"name\":\"hallo\",\"o\":{\"name\":\"hallo\",\"a\":[1,\"test\",{}]}}"[..], &js.into_vec()[..]);
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

#[cfg(test)]
const STR: &'static str = "A string that we want\nto \"escape\". A string that we want\nto \"escape\". A string that we want\nto \"escape\". ";

#[bench]
fn bench_encode_str_noescape(b: &mut test::Bencher) {
    let mut js = JsonEncoder::with_capacity(400);
    b.iter(|| {
        let n = test::black_box(100_0000);
        for _ in (0..n) {
            js.clear();
            js.encode_str_noescape(STR);
        }
    });
}

#[bench]
fn bench_encode_str(b: &mut test::Bencher) {
    let mut js = JsonEncoder::with_capacity(400);
    b.iter(|| {
        let n = test::black_box(100_0000);
        for _ in (0..n) {
            js.clear();
            js.encode_str(STR);
        }
    });
}

#[bench]
fn bench_encode_str_(b: &mut test::Bencher) {
    let mut js = JsonEncoder::with_capacity(400);
    b.iter(|| {
        let n = test::black_box(100_0000);
        for _ in (0..n) {
            js.clear();
            js.encode_str_(STR);
        }
    });
}

#[bench]
fn bench_encode_str3(b: &mut test::Bencher) {
    let mut js = JsonEncoder::with_capacity(400);
    b.iter(|| {
        let n = test::black_box(100_0000);
        for _ in (0..n) {
            js.clear();
            js.encode_str3(STR);
        }
    });
}


#[bench]
fn bench_encode_str2(b: &mut test::Bencher) {
    let mut js = JsonEncoder::with_capacity(400);
    b.iter(|| {
        let n = test::black_box(100_0000);
        for _ in (0..n) {
            js.clear();
            js.encode_str2(STR);
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
