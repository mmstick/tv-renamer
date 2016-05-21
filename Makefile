DESTDIR = /usr

all: gtk cli

gtk:
	cargo build --release --features "enable_gtk"
	mv target/release/tv-renamer target/release/tv-renamer-gtk

cli:
	cargo build --release

install: install-cli install-gtk

install-cli:
	install -d "${DESTDIR}/bin"
	install -m 755 target/release/tv-renamer "${DESTDIR}/bin/"

install-gtk:
	install -d "${DESTDIR}/bin"
	install -m 755 target/release/tv-renamer-gtk "${DESTDIR}/bin/"

uninstall:
	rm "${DESTDIR}/tv-renamer"
	rm "${DESTDIR}/tv-renamer-gtk"

clean:
	cargo clean
