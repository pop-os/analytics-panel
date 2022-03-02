export prefix ?= /usr
sysconfdir ?= /etc
bindir = $(prefix)/bin
includedir = $(prefix)/include
libdir = $(prefix)/lib

PACKAGE=pop_analytics_panel

TARGET = debug
DEBUG ?= 1

.PHONY = all clean install uninstall vendor

ifeq ($(DEBUG),0)
	TARGET = release
	ARGS += --release
endif

VENDOR ?= 0
ifneq ($(VENDOR),0)
	ARGS += --frozen
endif

PKGCONFIG=target/$(PACKAGE).pc
FFI=target/$(TARGET)/lib$(PACKAGE).so
FFI=target/$(TARGET)/lib$(PACKAGE).so

all: Cargo.toml Cargo.lock src/lib.rs extract-vendor
	cargo build $(ARGS)
	cargo build $(ARGS) --manifest-path ffi/Cargo.toml
	cargo run -p tools --bin pkgconfig -- $(PACKAGE) $(libdir) $(includedir)

clean:
	cargo clean

distclean:
	rm -rf .cargo vendor vendor.tar target

vendor:
	mkdir -p .cargo
	cargo vendor --sync ffi/Cargo.toml \
		--sync tools/Cargo.toml \
		| head -n -1 > .cargo/config
	echo 'directory = "vendor"' >> .cargo/config
	tar pcf vendor.tar vendor
	rm -rf vendor

extract-vendor:
ifeq ($(VENDOR),1)
	rm -rf vendor; tar pxf vendor.tar
endif

INSTALL_CLIB=$(DESTDIR)$(libdir)/lib$(PACKAGE).so
INSTALL_HEADER=$(DESTDIR)/$(includedir)/${PACKAGE}.h
INSTALL_PKGCONF=$(DESTDIR)$(libdir)/pkgconfig/$(PACKAGE).pc

install:
	install -Dm0644 $(PKGCONFIG) $(INSTALL_PKGCONF)
	install -Dm0644 data/$(PACKAGE).h $(INSTALL_HEADER)
	install -Dm0644 target/$(TARGET)/lib$(PACKAGE).so $(INSTALL_CLIB)

uninstall:
	rm $(INSTALL_CLIB) $(INSTALL_HEADER) $(INSTALL_PKGCONF)