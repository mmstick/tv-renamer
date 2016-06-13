DESTDIR = /usr

all: gtk

gtk:
	cargo build --release --features "enable_gtk"
	echo "#!/bin/sh" > "target/release/tv-renamer-gtk"
	echo "tv-renamer gtk" >> "target/release/tv-renamer-gtk"

cli:
	cargo build --release

install-cli: install-docs
	install -Dm 755 "target/release/tv-renamer" "${DESTDIR}/bin/"

install-gtk: install-cli install-docs
	install -Dm 755 "target/release/tv-renamer-gtk" "${DESTDIR}/bin/"
	install -Dm 644 "assets/tv-renamer.desktop" "${DESTDIR}/share/applications/"

install-docs:
	install -Dm 644 README.md "${DESTDIR}/share/doc/tv-renamer/README"
	install -Dm 644 LICENSE "${DESTDIR}/share/licenses/tv-renamer/COPYING"

create-tar:
	install -Dm 755 "target/release/tv-renamer-gtk" "tv-renamer/bin/"
	install -Dm 755 "target/release/tv-renamer" "tv-renamer/bin/"
	install -Dm 644 "assets/tv-renamer.desktop" "tv-renamer/share/applications/"
	tar cf - "tv-renamer" | xz -zf > "tv-renamer-binaries.tar.xz"

uninstall:
	rm "${DESTDIR}/tv-renamer"
	rm "${DESTDIR}/tv-renamer-gtk"

clean:
	cargo clean
