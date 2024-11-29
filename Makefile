PREFIX ?= /usr/local
CARGO = cargo
RUSTFLAGS ?= -A dead_code

.PHONY: all build release rebuild clean stats install uninstall

export RUSTFLAGS

build:
	$(CARGO) build
	@echo "Build complete - Not optimized"

release: clean
	$(CARGO) build --release --locked
	@strip target/release/nxplorer
	@echo "Build complete - Optimized"

clean:
	$(CARGO) clean
	rm -rf target/

stats: release
	@du -sh target/release/nxplorer | awk '{print "nxplorer binary size: " $$1}'
	@file target/release/nxplorer
	@ldd target/release/nxplorer

rebuild: clean all

	

install:
	sudo install -d $(DESTDIR)$(PREFIX)/bin
	sudo install -m 755 target/release/nxplorer $(DESTDIR)$(PREFIX)/bin/nxplorer

uninstall:
	sudo rm -f $(DESTDIR)$(PREFIX)/bin/nxplorer

all: build
