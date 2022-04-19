rootdir := ''
prefix := '/usr'
clean := '0'
debug := '0'
vendor := '0'
target := if debug == '1' { 'debug' } else { 'release' }
vendor_args := if vendor == '1' { '--frozen --offline' } else { '' }
debug_args := if debug == '1' { '' } else { '--release' }
cargo_args := vendor_args + ' ' + debug_args

sysconfdir := '/etc'
bindir := prefix + '/bin'
includedir := prefix + '/include'
libdir := prefix + '/lib'
sharedir := prefix + '/share'

package := 'pop_analytics_panel'
path_clib := rootdir + libdir + '/lib' + package + '.so'
path_header := rootdir + includedir + '/' + package + '.h'
path_pkgconfig := rootdir + libdir + '/pkgconfig/' + package + '.pc'
path_share := rootdir + sharedir + '/' + package

# Compiles all components of the library.
all: compile_clib compile_pkgconfig

# Compile and run the test application.
run_test: _extract_vendor
    cargo run {{cargo_args}}

# Compiles the C library.
compile_clib: _extract_vendor
    cargo build {{cargo_args}} --manifest-path ffi/Cargo.toml

# Compiles the C library's pkgconfig file.
compile_pkgconfig:
    cargo run -p tools --bin pkgconfig -- {{package}} {{libdir}} {{includedir}}

clean:
    cargo clean

distclean:
    rm -rf .cargo vendor vendor.tar target

install:
    install -Dm0644 target/{{target}}/lib{{package}}.so {{path_clib}}
    install -Dm0644 data/{{package}}.h {{path_header}}
    install -Dm0644 target/{{package}}.pc {{path_pkgconfig}}
    install -Dm0644 data/hp-privacy-statement-2021.pdf {{path_share}}/hp-privacy-statement-2021.pdf

uninstall:
    rm {{path_clib}} {{path_header}} {{path_pkgconfig}}

# Extracts vendored dependencies if vendor=1.
_extract_vendor:
    #!/usr/bin/env sh
    if test {{vendor}} = 1; then
        rm -rf vendor
        tar pxf vendor.tar
    fi

# Vendor Cargo dependencies locally.
vendor:
    mkdir -p .cargo
    cargo vendor --sync ffi/Cargo.toml \
        --sync tools/Cargo.toml \
        | head -n -1 > .cargo/config
    echo 'directory = "vendor"' >> .cargo/config
    tar pcf vendor.tar vendor
    rm -rf vendor
