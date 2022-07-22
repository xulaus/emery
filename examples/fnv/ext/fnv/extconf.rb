require 'mkmf'

raise "could not find cargo" unless find_executable('cargo')

require 'rbconfig'

def darwin?
    RbConfig::CONFIG["target_os"].include?("darwin")
end

raise "Unsupported platform" unless darwin?

rust_out = "#{$curdir}/target/release/libfnv.dylib"
lib_dir = "#{$curdir}/lib/fnv/"
lib = "#{$curdir}/fnv.bundle"

compile_command = "cargo build -q --release --target-dir=#{$curdir}/target/ --manifest-path=#{$srcdir}/Cargo.toml"
copy_command = "ld #{rust_out} -bundle -arch x86_64 -platform_version macos 12.0 12.0 -o #{lib}"


File.write('Makefile', <<~EOF)
build: Makefile
\t@#{compile_command}
\t@#{copy_command}
all: build
\t@echo "" > /dev/null
clean:
\trm -rf target
\trm -rf ../../lib/fnv
install: build
\t@echo "" > /dev/null
EOF
