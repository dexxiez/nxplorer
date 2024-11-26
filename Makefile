PREFIX ?= /usr/local
CARGO = cargo
RUSTFLAGS ?= -A dead_code

.PHONY: all clean install uninstall

export RUSTFLAGS

all:
	$(CARGO) build --release --locked

clean:
	$(CARGO) clean
	rm -rf target/

install: all
	install -d $(DESTDIR)$(PREFIX)/bin
	install -m 755 target/release/nxplorer $(DESTDIR)$(PREFIX)/bin/nxplorer

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/nxplorer
