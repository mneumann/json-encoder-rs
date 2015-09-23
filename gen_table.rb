require 'set'
C = {}
V = Set.new

def add(str, val)
    raise if V.include?(val)
    V << val
    str.each_char {|c|
        C[c.ord] = val
    }
end

def gen_rust
    puts %{const LUT: [u8; 256] = [}
    a = (0..255).map {|c| (C[c] || 0).to_s.rjust(2) + " /* " + c.to_s.rjust(3) + " */"}
    puts(a.each_slice(8).map{|s|
        "    " + s.join(", ")
    }.join(",\n"))
    puts %{];}
end

def gen_rust_bin
    puts %{const LUT_BIN: [u64; 4] = [}
    a = (0..255).map {|c| C[c] ? 1 : 0}

    puts([
        "    0b" + a[0...64].reverse.map{|i| i.to_s}.join,
        "    0b" + a[64...128].reverse.map{|i| i.to_s}.join,
        "    0b" + a[128...192].reverse.map{|i| i.to_s}.join,
        "    0b" + a[192...256].reverse.map{|i| i.to_s}.join
    ].join(",\n"))

    puts %{];}
end


add('"', '"'.ord)
add('\\', '\\'.ord)
add("\x08", 'b'.ord)
add("\x0c", 'f'.ord)
add("\n", 'n'.ord)
add("\r", 'r'.ord)
add("\t", 't'.ord)

gen_rust
gen_rust_bin
