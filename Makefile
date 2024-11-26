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
	sudo install -d $(DESTDIR)$(PREFIX)/bin
	sudo install -m 755 target/release/nxplorer $(DESTDIR)$(PREFIX)/bin/nxplorer

uninstall:
	sudo rm -f $(DESTDIR)$(PREFIX)/bin/nxplorer
